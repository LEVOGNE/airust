use super::protocol::{read_frame, write_frame, RESPONSE_BYTES};
use super::{ProcessOptions, WORKER_ARGUMENT};
use crate::reranking::RerankError;
use std::{
    io::BufReader,
    process::{Child, Command, Stdio},
    sync::mpsc::{self, Receiver, SyncSender},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

#[derive(Clone, Copy)]
enum IoFailure {
    Connection,
    Frame,
}

pub(super) struct Transport {
    child: Child,
    requests: Option<SyncSender<Vec<u8>>>,
    replies: Receiver<Result<Vec<u8>, IoFailure>>,
    io: Option<JoinHandle<()>>,
}

impl Transport {
    pub fn spawn(options: &ProcessOptions, init: Vec<u8>) -> Result<Self, RerankError> {
        let executable = options.executable.canonicalize().map_err(provider)?;
        if !executable.is_file() {
            return Err(RerankError::Configuration(
                "worker executable must be a regular file",
            ));
        }
        let mut child = Command::new(executable)
            .arg(WORKER_ARGUMENT)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(provider)?;
        let mut input = child.stdin.take().expect("piped stdin");
        let mut output = BufReader::new(child.stdout.take().expect("piped stdout"));
        let (send, commands) = mpsc::sync_channel::<Vec<u8>>(1);
        let (responses, receive) = mpsc::sync_channel(1);
        let io = thread::Builder::new()
            .name("airust-model-io".into())
            .spawn(move || {
                let exchange = |input: &mut std::process::ChildStdin,
                                output: &mut BufReader<std::process::ChildStdout>,
                                frame: &[u8]| {
                    write_frame(input, frame).map_err(|_| IoFailure::Connection)?;
                    read_frame(output, RESPONSE_BYTES)
                        .map_err(|e| {
                            if e.kind() == std::io::ErrorKind::InvalidData {
                                IoFailure::Frame
                            } else {
                                IoFailure::Connection
                            }
                        })?
                        .ok_or(IoFailure::Connection)
                };
                let first = exchange(&mut input, &mut output, &init);
                let failed = first.is_err();
                if responses.send(first).is_err() || failed {
                    return;
                }
                while let Ok(frame) = commands.recv() {
                    let reply = exchange(&mut input, &mut output, &frame);
                    let failed = reply.is_err();
                    if responses.send(reply).is_err() || failed {
                        return;
                    }
                }
                // Closing stdin signals normal teardown to the trusted worker.
            });
        let io = match io {
            Ok(io) => io,
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(provider(error));
            }
        };
        Ok(Self {
            child,
            requests: Some(send),
            replies: receive,
            io: Some(io),
        })
    }

    pub fn id(&self) -> u32 {
        self.child.id()
    }

    pub fn receive(&mut self, timeout: Duration) -> Result<Vec<u8>, RerankError> {
        match self.replies.recv_timeout(timeout) {
            Ok(Ok(frame)) => Ok(frame),
            Ok(Err(IoFailure::Frame)) => {
                self.abort();
                Err(RerankError::Output("invalid worker response frame"))
            }
            Ok(Err(IoFailure::Connection)) => {
                self.abort();
                Err(RerankError::Provider(
                    "local model worker connection closed".into(),
                ))
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                self.abort();
                Err(RerankError::Provider(
                    "local model worker timed out and was stopped".into(),
                ))
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                self.abort();
                Err(RerankError::Provider(
                    "local model worker disconnected".into(),
                ))
            }
        }
    }

    pub fn request(&mut self, frame: Vec<u8>, timeout: Duration) -> Result<Vec<u8>, RerankError> {
        let Some(sender) = &self.requests else {
            return Err(RerankError::Provider(
                "local model worker is stopped".into(),
            ));
        };
        if sender.send(frame).is_err() {
            self.abort();
            return Err(RerankError::Provider(
                "local model worker is stopped".into(),
            ));
        }
        self.receive(timeout)
    }

    pub fn abort(&mut self) {
        self.requests.take();
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(io) = self.io.take() {
            let _ = io.join();
        }
    }
}

impl Drop for Transport {
    fn drop(&mut self) {
        if self.io.is_none() {
            return;
        }
        self.requests.take();
        // No request is in flight after a successful synchronous call. Error and
        // timeout paths kill first, releasing the worker's pipe endpoints.
        if let Some(io) = self.io.take() {
            let _ = io.join();
        }
        let until = Instant::now() + Duration::from_secs(1);
        loop {
            match self.child.try_wait() {
                Ok(Some(_)) => return,
                Ok(None) if Instant::now() < until => thread::sleep(Duration::from_millis(5)),
                _ => break,
            }
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn provider(error: std::io::Error) -> RerankError {
    RerankError::Provider(super::protocol::bounded_message(error))
}
