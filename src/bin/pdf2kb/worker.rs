use airust::{agent::TrainingExample, knowledge::KnowledgeBase, PdfImportLimits, PdfLoaderConfig};
use std::{
    error::Error,
    fs,
    io::{self, Read},
    path::Path,
    process::{Child, Command, ExitStatus, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

const DIAGNOSTIC_BYTES: u64 = 256 * 1024;

struct OwnedWorker(Child);
impl Drop for OwnedWorker {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn capture(mut pipe: impl Read, failed: &AtomicBool) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let read = pipe
        .by_ref()
        .take(DIAGNOSTIC_BYTES + 1)
        .read_to_end(&mut bytes);
    if read.is_err() || bytes.len() as u64 > DIAGNOSTIC_BYTES {
        failed.store(true, Ordering::Release);
        return Err(io::Error::other(
            "worker diagnostic capture failed or exceeded 256 KiB",
        ));
    }
    read?;
    Ok(bytes)
}

fn wait_worker(
    child: &mut Child,
    started: Instant,
    timeout: Duration,
    capture_failed: &AtomicBool,
) -> Result<ExitStatus, Box<dyn Error>> {
    let mut notified = Duration::ZERO;
    loop {
        let status = child.try_wait()?;
        if capture_failed.load(Ordering::Acquire) {
            let _ = child.kill();
            child.wait()?;
            return Err(
                "PDF worker diagnostic capture failed or exceeded 256 KiB; output not published"
                    .into(),
            );
        }
        let elapsed = started.elapsed();
        if elapsed >= timeout {
            let _ = child.kill();
            child.wait()?;
            return Err(format!("PDF worker time budget of {} seconds exceeded; worker stopped and reaped; output not published", timeout.as_secs_f64()).into());
        }
        if let Some(status) = status {
            return Ok(status);
        }
        if elapsed.saturating_sub(notified) >= Duration::from_secs(5) {
            diagnostic!(
                "PDF worker running: {:.1} seconds elapsed of {} seconds",
                elapsed.as_secs_f64(),
                timeout.as_secs_f64()
            );
            notified = elapsed;
        }
        thread::sleep(Duration::from_millis(10).min(timeout.saturating_sub(elapsed)));
    }
}

fn read_result(path: &Path, maximum: u64) -> Result<KnowledgeBase, Box<dyn Error>> {
    let file = fs::File::open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > maximum {
        return Err("PDF worker result is not a regular file within the result byte budget".into());
    }
    let mut bytes = Vec::new();
    file.take(maximum + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > maximum {
        return Err("PDF worker result exceeded the result byte budget".into());
    }
    let examples: Vec<TrainingExample> = serde_json::from_slice(&bytes)?;
    if examples.is_empty() {
        return Err("PDF worker returned no usable knowledge".into());
    }
    for example in &examples {
        example.validate()?;
    }
    Ok(KnowledgeBase::from_examples(examples))
}

pub(super) fn convert(
    config: &PdfLoaderConfig,
    limits: PdfImportLimits,
    input: &Path,
    verbose: bool,
    timeout: Duration,
    result_limit: u64,
) -> Result<(KnowledgeBase, Vec<u8>), Box<dyn Error>> {
    if !limits.max_file_bytes().is_multiple_of(1024 * 1024)
        || !limits.max_text_bytes().is_multiple_of(1024 * 1024)
    {
        return Err("worker CLI budgets must be whole MiB values".into());
    }
    let started = Instant::now();
    started.checked_add(timeout).ok_or("timeout is too large")?;
    let folder = tempfile::Builder::new()
        .prefix("airust-pdf-worker-")
        .tempdir()?;
    let result_path = folder.path().join("result.json");
    let mut command = Command::new(std::env::current_exe()?);
    for (name, value) in [
        ("--min-chunk", config.min_chunk_size.to_string()),
        ("--max-chunk", config.max_chunk_size.to_string()),
        ("--overlap", config.chunk_overlap.to_string()),
        ("--weight", config.default_weight.to_string()),
        ("--max-pages", limits.max_pages().to_string()),
        ("--max-chunks", limits.max_chunks().to_string()),
    ] {
        command.arg(name).arg(value);
    }
    command
        .arg("--max-file-mib")
        .arg((limits.max_file_bytes() / (1024 * 1024)).to_string());
    command
        .arg("--max-text-mib")
        .arg((limits.max_text_bytes() / (1024 * 1024)).to_string());
    for (enabled, flag) in [
        (!config.include_metadata, "--no-metadata"),
        (!config.split_by_sentence, "--no-sentence-split"),
        (!config.split_by_paragraph, "--no-paragraph-split"),
        (!config.clean_text, "--no-clean"),
        (!config.dehyphenate, "--no-dehyphenate"),
        (!config.remove_page_numbers, "--keep-page-numbers"),
        (verbose, "--verbose"),
    ] {
        if enabled {
            command.arg(flag);
        }
    }
    command
        .arg("--")
        .arg(input)
        .arg(&result_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    diagnostic!(
        "Starting PDF worker with a {} second time budget",
        timeout.as_secs_f64()
    );
    let mut worker = OwnedWorker(command.spawn()?);
    let failed = Arc::new(AtomicBool::new(false));
    let stdout = worker.0.stdout.take().ok_or("missing worker stdout")?;
    let stderr = worker.0.stderr.take().ok_or("missing worker stderr")?;
    let flag = failed.clone();
    let out = thread::Builder::new()
        .name("pdf-worker-stdout".into())
        .spawn(move || capture(stdout, &flag))?;
    let flag = failed.clone();
    let err = thread::Builder::new()
        .name("pdf-worker-stderr".into())
        .spawn(move || capture(stderr, &flag))?;
    let status = wait_worker(&mut worker.0, started, timeout, &failed);
    drop(worker);
    let stdout = out.join().map_err(|_| "worker stdout reader panicked")?;
    let stderr = err.join().map_err(|_| "worker stderr reader panicked")?;
    let status = status?;
    let stdout = stdout?;
    let stderr = stderr?;
    if !status.success() {
        let diagnostic: String = String::from_utf8_lossy(&stderr)
            .chars()
            .take(2048)
            .collect();
        return Err(
            format!("PDF worker failed ({status}); output not published: {diagnostic}").into(),
        );
    }
    let knowledge = read_result(&result_path, result_limit)?;
    diagnostic!(
        "PDF worker completed in {:.2} seconds; output validated for publication",
        started.elapsed().as_secs_f64()
    );
    Ok((knowledge, stdout))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diagnostic_capture_has_an_exact_bound() {
        let failed = AtomicBool::new(false);
        assert_eq!(
            capture(io::repeat(b'x').take(DIAGNOSTIC_BYTES), &failed)
                .unwrap()
                .len() as u64,
            DIAGNOSTIC_BYTES
        );
        assert!(!failed.load(Ordering::Acquire));
        assert!(capture(io::repeat(b'x').take(DIAGNOSTIC_BYTES + 1), &failed).is_err());
        assert!(failed.load(Ordering::Acquire));
    }

    #[cfg(unix)]
    #[test]
    fn timeout_stops_and_reaps_the_owned_child() {
        let mut worker = OwnedWorker(
            Command::new("sh")
                .args(["-c", "exec sleep 30"])
                .spawn()
                .unwrap(),
        );
        let result = wait_worker(
            &mut worker.0,
            Instant::now(),
            Duration::from_millis(30),
            &AtomicBool::new(false),
        );
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("stopped and reaped"));
        assert!(worker.0.try_wait().unwrap().is_some());
    }

    #[cfg(unix)]
    #[test]
    fn diagnostic_failure_stops_and_reaps_the_owned_child() {
        let mut worker = OwnedWorker(
            Command::new("sh")
                .args(["-c", "exec sleep 30"])
                .spawn()
                .unwrap(),
        );
        let result = wait_worker(
            &mut worker.0,
            Instant::now(),
            Duration::from_secs(5),
            &AtomicBool::new(true),
        );
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("diagnostic capture"));
        assert!(worker.0.try_wait().unwrap().is_some());
    }

    #[cfg(unix)]
    #[test]
    fn nonzero_worker_exit_is_preserved_for_the_parent() {
        let mut worker = OwnedWorker(Command::new("sh").args(["-c", "exit 7"]).spawn().unwrap());
        let status = wait_worker(
            &mut worker.0,
            Instant::now(),
            Duration::from_secs(5),
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(status.code(), Some(7));
        assert!(worker.0.try_wait().unwrap().is_some());
    }

    #[test]
    fn result_reader_checks_bytes_and_knowledge_before_returning() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("result.json");
        fs::write(&path, b"[]").unwrap();
        assert!(read_result(&path, 2)
            .err()
            .unwrap()
            .to_string()
            .contains("no usable knowledge"));
        assert!(read_result(&path, 1).is_err());
        fs::write(&path, b"malformed").unwrap();
        assert!(read_result(&path, 100).is_err());
        fs::write(&path, br#"[{"input":"text","output":"text","weight":-1}]"#).unwrap();
        assert!(read_result(&path, 100).is_err());
    }
}
