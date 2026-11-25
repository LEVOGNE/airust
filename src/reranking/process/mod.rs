#![doc = include_str!("../../../docs/LOCAL-MODEL-PROCESSES.md")]

mod protocol;
#[cfg(all(test, unix, not(miri)))]
mod tests;
mod transport;

use super::{local::FastEmbedReranker, RerankError, Reranker};
use protocol::{Init, Reply, Request};
use std::{
    io::BufReader,
    path::{Path, PathBuf},
    time::Duration,
};

/// Reserved startup role understood by the bundled worker and reference pilot.
pub const WORKER_ARGUMENT: &str = "--airust-reranker-worker";

/// Application-owned executable and per-native-call time bounds. The trusted
/// worker must implement this package's protocol and must not spawn descendants
/// retaining its stdio pipes. This is resource isolation, not a security sandbox.
#[derive(Debug, Clone)]
pub struct ProcessOptions {
    executable: PathBuf,
    startup_timeout: Duration,
    request_timeout: Duration,
}

impl ProcessOptions {
    pub fn new(executable: impl Into<PathBuf>) -> Self {
        Self {
            executable: executable.into(),
            startup_timeout: Duration::from_secs(60),
            request_timeout: Duration::from_secs(30),
        }
    }

    pub fn with_timeouts(
        mut self,
        startup: Duration,
        request: Duration,
    ) -> Result<Self, RerankError> {
        if [startup, request]
            .iter()
            .any(|t| t.is_zero() || *t > Duration::from_secs(120))
        {
            return Err(RerankError::Configuration(
                "worker timeouts must be positive and at most 120 seconds",
            ));
        }
        self.startup_timeout = startup;
        self.request_timeout = request;
        Ok(self)
    }
}

/// One private native-model process and one bounded I/O thread. Dropping the
/// scorer closes its input, waits for normal exit, then kills/reaps if necessary.
/// Parent-owned snapshot directories survive until the worker has stopped.
pub struct ProcessReranker {
    transport: transport::Transport,
    _snapshot: tempfile::TempDir,
    recipe: String,
    next_id: u64,
    timeout: Duration,
}

impl ProcessReranker {
    pub fn multilingual_minilm_v2_from_snapshot(
        options: ProcessOptions,
        model_directory: impl AsRef<Path>,
        runtime_library: impl AsRef<Path>,
        threads: usize,
        snapshot_directory: impl AsRef<Path>,
    ) -> Result<Self, RerankError> {
        Self::load(
            options,
            model_directory.as_ref(),
            runtime_library.as_ref(),
            threads,
            snapshot_directory.as_ref(),
        )
    }

    fn load(
        options: ProcessOptions,
        model_directory: &Path,
        runtime_library: &Path,
        threads: usize,
        snapshots: &Path,
    ) -> Result<Self, RerankError> {
        if threads == 0 || threads > 64 {
            return Err(RerankError::Configuration(
                "worker thread count must be 1..64",
            ));
        }
        for path in [
            model_directory,
            runtime_library,
            options.executable.as_path(),
        ] {
            if path.as_os_str().len() > 32 * 1024 {
                return Err(RerankError::Configuration("worker path exceeds 32 KiB"));
            }
        }
        let model_directory = model_directory.canonicalize().map_err(provider)?;
        let runtime_library = runtime_library.canonicalize().map_err(provider)?;
        let snapshot_parent = snapshots.canonicalize().map_err(provider)?;
        let snapshot = tempfile::Builder::new()
            .prefix("airust-model-")
            .tempdir_in(snapshot_parent)
            .map_err(provider)?;
        let init = Init {
            version: crate::VERSION.into(),
            model_directory,
            runtime_library,
            threads,
            snapshot_directory: snapshot.path().to_path_buf(),
        };
        let frame = serde_json::to_vec(&init).map_err(|_| {
            RerankError::Configuration("worker configuration must have UTF-8 paths")
        })?;
        if frame.len() >= protocol::REQUEST_BYTES {
            return Err(RerankError::Budget("worker configuration"));
        }
        let mut transport = transport::Transport::spawn(&options, frame)?;
        let reply = transport.receive(options.startup_timeout)?;
        let reply = serde_json::from_slice::<Reply>(&reply)
            .map_err(|_| RerankError::Output("invalid worker startup reply"))?;
        let recipe = match reply {
            Reply::Ready {
                version,
                pid,
                recipe,
            } if version == crate::VERSION
                && pid == transport.id()
                && !recipe.is_empty()
                && recipe.len() <= 1024 =>
            {
                recipe
            }
            Reply::Failure {
                id: 0,
                output_error,
                message,
            } => return Err(remote_error(output_error, message)),
            _ => return Err(RerankError::Output("worker startup identity mismatch")),
        };
        Ok(Self {
            transport,
            _snapshot: snapshot,
            recipe,
            next_id: 1,
            timeout: options.request_timeout,
        })
    }

    pub fn worker_id(&self) -> u32 {
        self.transport.id()
    }
}

impl Reranker for ProcessReranker {
    fn recipe(&self) -> &str {
        &self.recipe
    }

    fn score(&mut self, query: &str, documents: &[&str]) -> Result<Vec<f64>, RerankError> {
        validate_text(query, documents)?;
        if documents.is_empty() {
            return Ok(Vec::new());
        }
        let id = self.next_id;
        self.next_id = id
            .checked_add(1)
            .ok_or(RerankError::Configuration("worker request ID exhausted"))?;
        // Validate bytes before cloning/serialization; JSON escaping has a
        // bounded sixfold expansion inside the separate wire-frame limit.
        let request = Request {
            id,
            query: query.into(),
            documents: documents.iter().map(|s| (*s).into()).collect(),
        };
        let frame = serde_json::to_vec(&request)
            .map_err(|_| RerankError::Output("worker request serialization failed"))?;
        if frame.len() >= protocol::REQUEST_BYTES {
            return Err(RerankError::Budget("worker frame"));
        }
        let frame = self.transport.request(frame, self.timeout)?;
        let reply = serde_json::from_slice::<Reply>(&frame)
            .map_err(|_| RerankError::Output("invalid worker score reply"));
        let result = match reply {
            Ok(Reply::Scores {
                id: returned,
                scores,
            }) if returned == id => {
                let scores: Vec<f64> = scores.into_iter().map(f64::from_bits).collect();
                if scores.len() != documents.len() {
                    Err(RerankError::ScoreCount {
                        expected: documents.len(),
                        actual: scores.len(),
                    })
                } else if let Some(i) = scores.iter().position(|s| !s.is_finite()) {
                    Err(RerankError::NonFiniteScore(i + 1))
                } else {
                    Ok(scores)
                }
            }
            Ok(Reply::Failure {
                id: returned,
                output_error,
                message,
            }) if returned == id => Err(remote_error(output_error, message)),
            _ => Err(RerankError::Output(
                "worker response identity or shape mismatch",
            )),
        };
        if result.is_err() {
            self.transport.abort();
        }
        result
    }
}

fn validate_text(query: &str, documents: &[&str]) -> Result<(), RerankError> {
    if query.len() > 16 * 1024
        || documents.len() > protocol::DOCUMENTS
        || documents.iter().any(|d| d.len() > 64 * 1024)
    {
        return Err(RerankError::Budget("worker input"));
    }
    let total = documents
        .iter()
        .try_fold(query.len(), |sum, d| sum.checked_add(d.len()))
        .ok_or(RerankError::Budget("worker input"))?;
    if total > protocol::TEXT_BYTES {
        return Err(RerankError::Budget("worker input"));
    }
    if query.trim().is_empty() {
        return Err(RerankError::EmptyQuery);
    }
    if documents.iter().any(|d| d.trim().is_empty()) {
        return Err(RerankError::Output("empty document passed to local scorer"));
    }
    Ok(())
}

fn remote_error(output: bool, message: String) -> RerankError {
    if output {
        RerankError::Output("malformed native worker output")
    } else {
        RerankError::Provider(protocol::bounded_message(message))
    }
}

fn provider(error: std::io::Error) -> RerankError {
    RerankError::Provider(protocol::bounded_message(error))
}

/// Run the trusted worker role on stdin/stdout. The application must dispatch
/// `WORKER_ARGUMENT` before starting a server or loading any other model.
pub fn run_worker() -> Result<(), RerankError> {
    let mut input = BufReader::new(std::io::stdin().lock());
    let mut output = std::io::stdout().lock();
    let frame = protocol::read_frame(&mut input, protocol::REQUEST_BYTES)
        .map_err(provider)?
        .ok_or(RerankError::Output("worker initialization missing"))?;
    let init: Init = serde_json::from_slice(&frame)
        .map_err(|_| RerankError::Configuration("invalid worker initialization"))?;
    if init.version != crate::VERSION || init.threads == 0 || init.threads > 64 {
        return Err(RerankError::Configuration(
            "worker initialization version or threads mismatch",
        ));
    }
    let model = FastEmbedReranker::multilingual_minilm_v2_from_snapshot(
        init.model_directory,
        init.runtime_library,
        init.threads,
        init.snapshot_directory,
    );
    let mut model = match model {
        Ok(model) => model,
        Err(error) => {
            reply(&mut output, &failure(0, error))?;
            return Ok(());
        }
    };
    reply(
        &mut output,
        &Reply::Ready {
            version: crate::VERSION.into(),
            pid: std::process::id(),
            recipe: model.recipe().into(),
        },
    )?;
    let mut expected = 1u64;
    while let Some(frame) =
        protocol::read_frame(&mut input, protocol::REQUEST_BYTES).map_err(provider)?
    {
        let request: Request = serde_json::from_slice(&frame)
            .map_err(|_| RerankError::Output("invalid worker request"))?;
        if request.id != expected {
            return Err(RerankError::Output("worker request sequence mismatch"));
        }
        expected = expected
            .checked_add(1)
            .ok_or(RerankError::Output("worker request sequence exhausted"))?;
        let documents: Vec<_> = request.documents.iter().map(String::as_str).collect();
        let result = validate_text(&request.query, &documents)
            .and_then(|()| model.score(&request.query, &documents));
        let response = match result {
            Ok(scores) => Reply::Scores {
                id: request.id,
                scores: scores.into_iter().map(f64::to_bits).collect(),
            },
            Err(e) => failure(request.id, e),
        };
        reply(&mut output, &response)?;
    }
    Ok(())
}

fn failure(id: u64, error: RerankError) -> Reply {
    Reply::Failure {
        id,
        output_error: !matches!(error, RerankError::Provider(_)),
        message: protocol::bounded_message(error),
    }
}

fn reply(writer: &mut impl std::io::Write, value: &Reply) -> Result<(), RerankError> {
    let frame = serde_json::to_vec(value)
        .map_err(|_| RerankError::Output("worker output serialization failed"))?;
    if frame.len() >= protocol::RESPONSE_BYTES {
        return Err(RerankError::Budget("worker response"));
    }
    protocol::write_frame(writer, &frame).map_err(provider)
}

#[cfg(test)]
mod validation_tests {
    use super::*;

    #[test]
    fn input_limits_reject_expansion_before_worker_work() {
        assert!(validate_text("query", &[]).is_ok());
        assert_eq!(validate_text("  ", &["text"]), Err(RerankError::EmptyQuery));
        assert!(matches!(
            validate_text("query", &[" "]),
            Err(RerankError::Output(_))
        ));
        let query = "é".repeat(8192);
        assert!(validate_text(&query, &["text"]).is_ok());
        assert!(matches!(
            validate_text(&(query + "é"), &["text"]),
            Err(RerankError::Budget(_))
        ));
        assert!(validate_text("query", &["text"; 128]).is_ok());
        assert!(matches!(
            validate_text("query", &["text"; 129]),
            Err(RerankError::Budget(_))
        ));
        let document = "x".repeat(64 * 1024);
        assert!(validate_text("query", &[document.as_str(); 31]).is_ok());
        assert!(matches!(
            validate_text("query", &[document.as_str(); 32]),
            Err(RerankError::Budget(_))
        ));
        assert!(matches!(
            validate_text("query", &[&(document + "x")]),
            Err(RerankError::Budget(_))
        ));
    }

    #[test]
    fn timeout_bounds_are_checked_without_launching_processes() {
        for timeout in [Duration::ZERO, Duration::from_secs(121)] {
            assert!(ProcessOptions::new("unused")
                .with_timeouts(timeout, Duration::from_secs(1))
                .is_err());
            assert!(ProcessOptions::new("unused")
                .with_timeouts(Duration::from_secs(1), timeout)
                .is_err());
        }
        assert!(ProcessOptions::new("unused")
            .with_timeouts(Duration::from_nanos(1), Duration::from_secs(120))
            .is_ok());
    }
}
