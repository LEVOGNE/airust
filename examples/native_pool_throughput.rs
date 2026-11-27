#![forbid(unsafe_code)]
use airust::{
    reranking::{
        process::{ProcessOptions, ProcessPool, ProcessReranker, WORKER_ARGUMENT},
        rerank, RerankOptions, Reranker,
    },
    KnowledgeEntry,
};
use serde::Deserialize;
use std::{
    error::Error,
    io::Read,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

#[derive(Deserialize)]
struct Prefix {
    query: String,
    candidates: Vec<KnowledgeEntry>,
}

struct Workload {
    prefixes: Vec<Prefix>,
    options: RerankOptions,
}

impl Workload {
    fn load(path: &str) -> Result<Self, Box<dyn Error>> {
        let file = std::fs::File::open(path)?;
        let mut bytes = Vec::new();
        file.take(1024 * 1024 + 1).read_to_end(&mut bytes)?;
        let prefixes: Vec<Prefix> = serde_json::from_slice(&bytes)?;
        if prefixes.len() != 4 || bytes.len() > 1024 * 1024 {
            return Err("expected four prefixes within 1 MiB".into());
        }
        Ok(Self {
            prefixes,
            options: RerankOptions::new(20)
                .with_candidates(20)?
                .with_batch_size(1)?,
        })
    }

    fn signature(
        &self,
        case: usize,
        model: &mut dyn Reranker,
    ) -> Result<Vec<(String, u64)>, Box<dyn Error>> {
        let prefix = &self.prefixes[case];
        let outcome = rerank(&prefix.query, &prefix.candidates, model, &self.options)?;
        Ok(outcome
            .results()
            .iter()
            .map(|r| (r.entry().id.clone(), r.score.map_or(u64::MAX, f64::to_bits)))
            .collect())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some(WORKER_ARGUMENT) {
        airust::reranking::process::run_worker()?;
        return Ok(());
    }
    if args.len() != 6 {
        return Err(
            "expected PREFIXES MODEL_DIR RUNTIME SNAPSHOT_PARENT CALLERS REQUESTS_PER_CALLER"
                .into(),
        );
    }
    let callers: usize = args[4].parse()?;
    let requests: usize = args[5].parse()?;
    if !(1..=32).contains(&callers) || !(1..=200).contains(&requests) {
        return Err("callers 1..32, requests 1..200".into());
    }
    let workload = Arc::new(Workload::load(&args[0])?);
    let worker = std::env::current_exe()?;
    let mut reference = ProcessReranker::multilingual_minilm_v2_from_snapshot(
        ProcessOptions::new(&worker),
        &args[1],
        &args[2],
        2,
        &args[3],
    )?;
    let oracle: Arc<Vec<_>> = Arc::new(
        (0..4)
            .map(|case| workload.signature(case, &mut reference))
            .collect::<Result<_, _>>()?,
    );
    drop(reference);
    for max_workers in [1usize, 2] {
        let pool = Arc::new(ProcessPool::multilingual_minilm_v2_from_snapshot(
            ProcessOptions::new(&worker),
            &args[1],
            &args[2],
            2,
            &args[3],
            max_workers,
            callers,
        )?);
        let peak = Arc::new(AtomicUsize::new(0));
        let started = Instant::now();
        let threads: Vec<_> = (0..callers)
            .map(|caller| {
                let (pool, peak, workload, oracle) = (
                    Arc::clone(&pool),
                    Arc::clone(&peak),
                    Arc::clone(&workload),
                    Arc::clone(&oracle),
                );
                std::thread::spawn(move || -> Result<Vec<f64>, String> {
                    let mut latencies = Vec::with_capacity(requests);
                    for request in 0..requests {
                        let case = (caller + request) % 4;
                        let begin = Instant::now();
                        let mut lease = pool
                            .acquire(Duration::from_secs(600))
                            .map_err(|e| format!("{e:?}"))?;
                        peak.fetch_max(pool.live_workers(), Ordering::SeqCst);
                        let signature = workload
                            .signature(case, &mut lease)
                            .map_err(|e| e.to_string())?;
                        drop(lease);
                        if signature != oracle[case] {
                            return Err("score signature differs from single-worker oracle".into());
                        }
                        latencies.push(begin.elapsed().as_secs_f64() * 1000.0);
                    }
                    Ok(latencies)
                })
            })
            .collect();
        let mut latencies = Vec::new();
        for thread in threads {
            latencies.extend(thread.join().map_err(|_| "caller panicked")??);
        }
        let elapsed = started.elapsed().as_secs_f64();
        let mut ordered = latencies.clone();
        ordered.sort_by(f64::total_cmp);
        let rank = |p: f64| ordered[((ordered.len() as f64 * p).ceil() as usize).saturating_sub(1)];
        println!(
            "{}",
            serde_json::json!({
                "max_workers": max_workers,
                "callers": callers,
                "requests": latencies.len(),
                "all_bits_equal": true,
                "peak_live_workers": peak.load(Ordering::SeqCst),
                "elapsed_s": elapsed,
                "requests_per_minute": latencies.len() as f64 * 60.0 / elapsed,
                "p50_ms": rank(0.5),
                "p95_ms": rank(0.95),
                "latencies_ms": latencies,
            })
        );
        drop(pool);
        if std::fs::read_dir(&args[3])?.count() != 0 {
            return Err("snapshot parent not empty after pool drop".into());
        }
    }
    Ok(())
}
