#![forbid(unsafe_code)]
use airust::{
    reranking::{
        process::{ProcessOptions, ProcessReranker, WORKER_ARGUMENT},
        rerank, RerankError, RerankOptions, Reranker,
    },
    KnowledgeEntry,
};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    error::Error,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

#[derive(Deserialize)]
struct Prefix {
    id: String,
    query: String,
    candidates: Vec<KnowledgeEntry>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Fault {
    None,
    Kill,
    Timeout,
    InvalidRuntime,
}

impl Fault {
    fn for_cycle(cycle: usize) -> Self {
        match cycle % 10 {
            3 => Self::Kill,
            6 => Self::Timeout,
            8 => Self::InvalidRuntime,
            _ => Self::None,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::None => "normal",
            Self::Kill => "kill",
            Self::Timeout => "timeout",
            Self::InvalidRuntime => "invalid_runtime",
        }
    }
}

struct Host;

impl Host {
    fn rss_bytes() -> u64 {
        Command::new("ps")
            .args(["-o", "rss=", "-p", &std::process::id().to_string()])
            .output()
            .ok()
            .and_then(|out| String::from_utf8(out.stdout).ok())
            .and_then(|text| text.trim().parse::<u64>().ok())
            .map_or(0, |kib| kib * 1024)
    }

    fn open_descriptors() -> usize {
        ["/proc/self/fd", "/dev/fd"]
            .iter()
            .find_map(|dir| std::fs::read_dir(dir).ok())
            .map_or(0, |entries| entries.count())
    }

    fn process_state(pid: u32) -> Option<String> {
        let out = Command::new("ps")
            .args(["-o", "stat=", "-p", &pid.to_string()])
            .output()
            .ok()?;
        let state = String::from_utf8(out.stdout).ok()?.trim().to_owned();
        (!state.is_empty()).then_some(state)
    }

    fn kill(pid: u32) -> bool {
        Command::new("/bin/kill")
            .args(["-9", &pid.to_string()])
            .status()
            .is_ok_and(|status| status.success())
    }
}

struct Soak {
    worker: PathBuf,
    model: PathBuf,
    runtime: PathBuf,
    invalid_runtime: PathBuf,
    snapshots: tempfile::TempDir,
    prefixes: Vec<Prefix>,
    oracle: BTreeMap<String, Vec<(String, u64)>>,
    options: RerankOptions,
}

impl Soak {
    fn new(args: &[String]) -> Result<Self, Box<dyn Error>> {
        let file = std::fs::File::open(&args[0])?;
        if !file.metadata()?.is_file() || file.metadata()?.len() > 1024 * 1024 {
            return Err("prefixes must be a regular file of at most 1 MiB".into());
        }
        let mut bytes = Vec::new();
        file.take(1024 * 1024 + 1).read_to_end(&mut bytes)?;
        let prefixes: Vec<Prefix> = serde_json::from_slice(&bytes)?;
        if prefixes.len() != 4 || prefixes.iter().any(|p| p.candidates.len() != 20) {
            return Err("expected four prefixes with twenty candidates".into());
        }
        let snapshots = tempfile::tempdir_in(&args[4])?;
        let invalid_runtime = snapshots.path().join("invalid-runtime");
        std::fs::write(&invalid_runtime, b"invalid native runtime")?;
        let invalid_runtime = invalid_runtime.canonicalize()?;
        Ok(Self {
            worker: std::env::current_exe()?,
            model: PathBuf::from(&args[1]),
            runtime: PathBuf::from(&args[2]),
            invalid_runtime,
            snapshots,
            prefixes,
            oracle: BTreeMap::new(),
            options: RerankOptions::new(20)
                .with_candidates(20)?
                .with_batch_size(1)?,
        })
    }

    fn private_entries(&self) -> usize {
        std::fs::read_dir(self.snapshots.path())
            .map_or(usize::MAX, |entries| entries.count())
            .saturating_sub(1)
    }

    fn open(
        &self,
        options: ProcessOptions,
        runtime: &Path,
    ) -> Result<ProcessReranker, RerankError> {
        ProcessReranker::multilingual_minilm_v2_from_snapshot(
            options,
            &self.model,
            runtime,
            2,
            self.snapshots.path(),
        )
    }

    fn signature(
        &self,
        case: &Prefix,
        model: &mut dyn Reranker,
    ) -> Result<Vec<(String, u64)>, RerankError> {
        let outcome = rerank(&case.query, &case.candidates, model, &self.options)?;
        Ok(outcome
            .results()
            .iter()
            .map(|r| (r.entry().id.clone(), r.score.map_or(u64::MAX, f64::to_bits)))
            .collect())
    }

    fn cycle(&mut self, cycle: usize) -> serde_json::Value {
        let fault = Fault::for_cycle(cycle);
        let case_index = cycle % self.prefixes.len();
        let started = Instant::now();
        let mut record = serde_json::json!({"cycle": cycle, "fault": fault.name(), "case": self.prefixes[case_index].id});
        let (pid, outcome) = match fault {
            Fault::InvalidRuntime => {
                let runtime = self.invalid_runtime.clone();
                let result = self.open(ProcessOptions::new(&self.worker), &runtime);
                (
                    None,
                    result
                        .map(|_| ())
                        .err()
                        .map_or(Err("invalid runtime was accepted".to_owned()), |e| {
                            Ok(format!("{e:?}"))
                        }),
                )
            }
            Fault::Timeout => match ProcessOptions::new(&self.worker)
                .with_timeouts(Duration::from_secs(60), Duration::from_millis(1))
                .map_err(|e| format!("{e:?}"))
                .and_then(|options| {
                    self.open(options, &self.runtime.clone())
                        .map_err(|e| format!("{e:?}"))
                }) {
                Ok(mut model) => {
                    let pid = model.worker_id();
                    let request = Instant::now();
                    let result = self.signature(&self.prefixes[case_index], &mut model);
                    record["fault_ms"] = (request.elapsed().as_secs_f64() * 1000.0).into();
                    drop(model);
                    (
                        Some(pid),
                        result
                            .err()
                            .map_or(Err("timeout did not fail".to_owned()), |e| {
                                Ok(format!("{e:?}"))
                            }),
                    )
                }
                Err(error) => (None, Err(error)),
            },
            Fault::None | Fault::Kill => {
                match self.open(ProcessOptions::new(&self.worker), &self.runtime.clone()) {
                    Ok(mut model) => {
                        let pid = model.worker_id();
                        record["load_ms"] = (started.elapsed().as_secs_f64() * 1000.0).into();
                        let first = Instant::now();
                        let result = self.signature(&self.prefixes[case_index], &mut model);
                        record["first_ms"] = (first.elapsed().as_secs_f64() * 1000.0).into();
                        let checked = match result {
                            Ok(signature) => {
                                let key = self.prefixes[case_index].id.clone();
                                let expected =
                                    self.oracle.entry(key).or_insert_with(|| signature.clone());
                                if *expected == signature {
                                    Ok("bits_equal".to_owned())
                                } else {
                                    Err("score signature differs from oracle".to_owned())
                                }
                            }
                            Err(error) => Err(format!("{error:?}")),
                        };
                        let outcome = if fault == Fault::Kill && checked.is_ok() {
                            let killed = Host::kill(pid);
                            let request = Instant::now();
                            let after = self.signature(&self.prefixes[case_index], &mut model);
                            record["fault_ms"] = (request.elapsed().as_secs_f64() * 1000.0).into();
                            match (killed, after) {
                                (true, Err(error)) => Ok(format!("{error:?}")),
                                (false, _) => Err("kill signal failed".to_owned()),
                                (true, Ok(_)) => Err("killed worker returned scores".to_owned()),
                            }
                        } else {
                            checked
                        };
                        drop(model);
                        (Some(pid), outcome)
                    }
                    Err(error) => (None, Err(format!("{error:?}"))),
                }
            }
        };
        std::thread::sleep(Duration::from_millis(50));
        let state = pid.and_then(Host::process_state);
        record["ok"] = outcome.is_ok().into();
        record["detail"] = match outcome {
            Ok(text) | Err(text) => text.chars().take(240).collect::<String>().into(),
        };
        record["worker_pid"] = pid.into();
        record["worker_survived"] = state.as_deref().is_some_and(|s| !s.starts_with('Z')).into();
        record["worker_zombie"] = state.as_deref().is_some_and(|s| s.starts_with('Z')).into();
        record["private_entries"] = self.private_entries().into();
        record["rss_bytes"] = Host::rss_bytes().into();
        record["open_descriptors"] = Host::open_descriptors().into();
        record["elapsed_ms"] = (started.elapsed().as_secs_f64() * 1000.0).into();
        record
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some(WORKER_ARGUMENT) {
        airust::reranking::process::run_worker()?;
        return Ok(());
    }
    if args.len() != 5 {
        return Err("expected PREFIXES MODEL_DIR RUNTIME SECONDS SNAPSHOT_PARENT".into());
    }
    let seconds: u64 = args[3].parse()?;
    if !(60..=86_400).contains(&seconds) {
        return Err("SECONDS must be 60..86400".into());
    }
    let mut soak = Soak::new(&args)?;
    let deadline = Instant::now() + Duration::from_secs(seconds);
    let mut cycle = 0;
    let mut out = std::io::stdout().lock();
    while Instant::now() < deadline {
        let record = soak.cycle(cycle);
        writeln!(out, "{record}")?;
        out.flush()?;
        cycle += 1;
    }
    writeln!(
        out,
        "{}",
        serde_json::json!({"complete": true, "cycles": cycle})
    )?;
    Ok(())
}
