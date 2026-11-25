#![forbid(unsafe_code)]
use airust::{
    execution::{CancellationToken, ExecutionError, ProgressEvent, WorkControl, WorkPhase},
    reranking::{
        local::FastEmbedReranker, rerank, RerankError, RerankFailurePolicy, RerankOptions, Reranker,
    },
    KnowledgeEntry,
};
use serde::Deserialize;
use std::{
    error::Error,
    io::{self, BufRead, Read, Write},
    time::{Duration, Instant},
};
#[derive(Deserialize)]
struct Prefix {
    id: String,
    query: String,
    candidates: Vec<KnowledgeEntry>,
}
struct Counted<'a> {
    native: &'a mut dyn Reranker,
    calls: usize,
}
impl Reranker for Counted<'_> {
    fn recipe(&self) -> &str {
        self.native.recipe()
    }
    fn score(&mut self, q: &str, d: &[&str]) -> Result<Vec<f64>, RerankError> {
        self.calls += 1;
        self.native.score(q, d)
    }
}
fn event(
    stage: &str,
    cycle: usize,
    mode: &str,
    extra: serde_json::Value,
) -> Result<(), Box<dyn Error>> {
    println!(
        "{}",
        serde_json::json!({"pid":std::process::id(),"stage":stage,"cycle":cycle,"mode":mode,"extra":extra})
    );
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().lock().take(10).read_line(&mut line)?;
    if line != "continue\n" && line != "continue\r\n" {
        return Err("controller handshake failed".into());
    }
    Ok(())
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some(airust::reranking::process::WORKER_ARGUMENT) {
        airust::reranking::process::run_worker()?;
        return Ok(());
    }
    if args.len() != 7 {
        return Err(
            "expected PREFIXES MODEL_DIR RUNTIME CYCLES BACKEND MODE SNAPSHOT_PARENT".into(),
        );
    }
    if !["in_process", "process"].contains(&args[4].as_str()) {
        return Err("backend must be in_process or process".into());
    }
    let cycles: usize = args[3].parse()?;
    assert!((8..=100).contains(&cycles));
    let file = std::fs::File::open(&args[0])?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > 1024 * 1024 {
        return Err("prefixes must be a regular file of at most 1 MiB".into());
    }
    let mut bytes = Vec::new();
    file.take(1024 * 1024 + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 != metadata.len() {
        return Err("prefix file changed or exceeded its bound".into());
    }
    let prefixes: Vec<Prefix> = serde_json::from_slice(&bytes)?;
    assert_eq!(prefixes.len(), 4);
    assert!(prefixes.iter().all(|c| c.candidates.len() == 20));
    let snapshots = tempfile::tempdir_in(&args[6])?;
    event(
        "initial",
        0,
        "none",
        serde_json::json!({"snapshot_files":0}),
    )?;
    for cycle in 0..cycles {
        let snapshot = match args[5].as_str() {
            "snapshot" => true,
            "memory" => false,
            "mixed" => cycle % 2 == 1,
            _ => panic!("mode"),
        };
        let mode = if snapshot { "snapshot" } else { "memory" };
        let case = &prefixes[cycle % prefixes.len()];
        let loading = Instant::now();
        let mut model: Box<dyn Reranker> = if args[4] == "process" {
            use airust::reranking::process::{ProcessOptions, ProcessReranker};
            let options = ProcessOptions::new(std::env::current_exe()?);
            if !snapshot {
                return Err("isolated workers require verified snapshot loading".into());
            }
            Box::new(ProcessReranker::multilingual_minilm_v2_from_snapshot(
                options,
                &args[1],
                &args[2],
                2,
                snapshots.path(),
            )?)
        } else if snapshot {
            Box::new(FastEmbedReranker::multilingual_minilm_v2_from_snapshot(
                &args[1],
                &args[2],
                2,
                snapshots.path(),
            )?)
        } else {
            Box::new(FastEmbedReranker::multilingual_minilm_v2(
                &args[1], &args[2], 2,
            )?)
        };
        let load_ms = loading.elapsed().as_secs_f64() * 1000.0;
        assert_eq!(
            std::fs::read_dir(snapshots.path())?.count(),
            usize::from(snapshot)
        );
        event(
            "loaded",
            cycle,
            mode,
            serde_json::json!({"recipe":model.recipe(),"load_ms":load_ms,"snapshot_files":usize::from(snapshot)}),
        )?;
        let options = RerankOptions::new(20)
            .with_candidates(20)?
            .with_batch_size(1)?;
        let first_start = Instant::now();
        let before = rerank(&case.query, &case.candidates, model.as_mut(), &options)?;
        let first_ms = first_start.elapsed().as_secs_f64() * 1000.0;
        let signature: Vec<_> = before
            .results()
            .iter()
            .map(|r| (r.entry().id.clone(), r.score.unwrap().to_bits()))
            .collect();
        let cancel = CancellationToken::new();
        let observed = cancel.clone();
        let control = WorkControl::new(cancel).with_observer(move |e: &ProgressEvent| {
            if e.phase() == WorkPhase::Reranking && e.completed() >= 1 {
                observed.cancel();
            }
        });
        let mut counted = Counted {
            native: model.as_mut(),
            calls: 0,
        };
        let cancelled = rerank(
            &case.query,
            &case.candidates,
            &mut counted,
            &options
                .clone()
                .with_control(control)
                .with_failure_policy(RerankFailurePolicy::KeepOriginalOrder),
        );
        assert!(matches!(
            cancelled,
            Err(RerankError::Execution(ExecutionError::Cancelled))
        ));
        assert_eq!(counted.calls, 1);
        let deadline = WorkControl::new(CancellationToken::new()).with_deadline(Instant::now());
        let expired = rerank(
            &case.query,
            &case.candidates,
            &mut counted,
            &options.clone().with_control(deadline),
        );
        assert!(matches!(
            expired,
            Err(RerankError::Execution(ExecutionError::DeadlineExceeded))
        ));
        assert_eq!(counted.calls, 1);
        let recovery_start = Instant::now();
        let after = rerank(&case.query, &case.candidates, &mut counted, &options)?;
        let recovery_ms = recovery_start.elapsed().as_secs_f64() * 1000.0;
        let recovered: Vec<_> = after
            .results()
            .iter()
            .map(|r| (r.entry().id.clone(), r.score.unwrap().to_bits()))
            .collect();
        assert_eq!(signature, recovered);
        event(
            "recovered",
            cycle,
            mode,
            serde_json::json!({"case_id":case.id,"cancelled_after_native_batches":1,"expired_deadline_extra_calls":0,"recovery_scores_bits_equal":true,"signature":signature,"first_ms":first_ms,"recovery_ms":recovery_ms,"snapshot_files":usize::from(snapshot)}),
        )?;
        drop(model);
        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(std::fs::read_dir(snapshots.path())?.count(), 0);
        event(
            "dropped",
            cycle,
            mode,
            serde_json::json!({"snapshot_files":0}),
        )?;
    }
    // A full verified-copy constructor failure must clean its copy too.
    let invalid_runtime = snapshots.path().join("invalid-runtime");
    std::fs::write(&invalid_runtime, b"invalid native runtime")?;
    let error = if args[4] == "process" {
        airust::reranking::process::ProcessReranker::multilingual_minilm_v2_from_snapshot(
            airust::reranking::process::ProcessOptions::new(std::env::current_exe()?),
            &args[1],
            &invalid_runtime,
            2,
            snapshots.path(),
        )
        .map(|_| ())
    } else {
        FastEmbedReranker::multilingual_minilm_v2_from_snapshot(
            &args[1],
            &invalid_runtime,
            2,
            snapshots.path(),
        )
        .map(|_| ())
    };
    assert!(matches!(error, Err(RerankError::Provider(_))));
    std::fs::remove_file(&invalid_runtime)?;
    assert_eq!(std::fs::read_dir(snapshots.path())?.count(), 0);
    event(
        "complete",
        cycles,
        "none",
        serde_json::json!({"all_native_cancel_recoveries":true,"cycles":cycles,"failed_runtime_snapshot_cleanup":true}),
    )?;
    Ok(())
}
