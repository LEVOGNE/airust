use super::McpLimits;
use rmcp::{
    model::{ClientNotification, JsonRpcMessage, RequestId},
    service::{RxJsonRpcMessage, TxJsonRpcMessage},
    transport::Transport,
    RoleServer,
};
use std::{
    collections::BTreeSet,
    io,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
    time::Instant,
};
use tokio::{
    io::{AsyncRead, ReadBuf},
    sync::{OwnedSemaphorePermit, Semaphore},
};
use tokio_util::{
    sync::CancellationToken,
    task::{task_tracker::TaskTrackerToken, TaskTracker},
};

#[derive(Debug)]
pub(super) enum WorkerAdmissionError {
    Closed,
    Busy,
}
pub(super) struct WorkerLease {
    _permit: OwnedSemaphorePermit,
    _tracker: TaskTrackerToken,
}
impl WorkerLease {
    pub fn spawn<F, T>(self, work: F) -> tokio::task::JoinHandle<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        tokio::task::spawn_blocking(move || {
            let result = work();
            drop(self);
            result
        })
    }
}

pub(super) struct Session {
    pub stop: CancellationToken,
    workers: TaskTracker,
    pub progress_tokens: Mutex<BTreeSet<String>>,
    pub limits: McpLimits,
    state: Mutex<State>,
}
struct State {
    window: Instant,
    messages: usize,
    frames: usize,
    bytes: usize,
    total_bytes: usize,
    seen: BTreeSet<String>,
    pending: BTreeSet<String>,
    reason: Option<&'static str>,
    closing: bool,
}
impl Session {
    pub fn reserve_worker(
        &self,
        slots: Arc<Semaphore>,
    ) -> Result<WorkerLease, WorkerAdmissionError> {
        let state = self
            .state
            .lock()
            .map_err(|_| WorkerAdmissionError::Closed)?;
        if state.closing || state.reason.is_some() || self.stop.is_cancelled() {
            return Err(WorkerAdmissionError::Closed);
        }
        let permit = slots
            .try_acquire_owned()
            .map_err(|_| WorkerAdmissionError::Busy)?;
        let lease = WorkerLease {
            _permit: permit,
            _tracker: self.workers.token(),
        };
        drop(state);
        Ok(lease)
    }
    pub async fn shutdown_workers(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.closing = true;
        }
        self.stop.cancel();
        self.workers.close();
        self.workers.wait().await;
    }
    pub fn new(limits: McpLimits) -> Arc<Self> {
        Arc::new(Self {
            stop: CancellationToken::new(),
            workers: TaskTracker::new(),
            progress_tokens: Mutex::new(BTreeSet::new()),
            limits,
            state: Mutex::new(State {
                window: Instant::now(),
                messages: 0,
                frames: 0,
                bytes: 0,
                total_bytes: 0,
                seen: BTreeSet::new(),
                pending: BTreeSet::new(),
                reason: None,
                closing: false,
            }),
        })
    }
    pub fn fail(&self, reason: &'static str) {
        if let Ok(mut s) = self.state.lock() {
            s.reason.get_or_insert(reason);
        }
        self.stop.cancel();
    }
    pub fn reason(&self) -> Option<&'static str> {
        self.state.lock().ok().and_then(|s| s.reason)
    }
    fn with_state(&self, action: impl FnOnce(&mut State) -> Result<(), &'static str>) -> bool {
        let result = self
            .state
            .lock()
            .map_err(|_| "session lock poisoned")
            .and_then(|mut s| {
                if s.window.elapsed().as_secs() >= 1 {
                    s.window = Instant::now();
                    s.messages = 0;
                    s.frames = 0;
                    s.bytes = 0;
                }
                action(&mut s)
            });
        if let Err(reason) = result {
            self.fail(reason);
            false
        } else {
            true
        }
    }
    fn bytes(&self, count: usize, frames: usize) -> bool {
        self.with_state(|s| {
            s.frames = s.frames.checked_add(frames).ok_or("frame rate limit")?;
            if s.frames > self.limits.max_messages_per_second {
                return Err("frame rate limit");
            }
            s.bytes = s.bytes.checked_add(count).ok_or("inbound byte limit")?;
            s.total_bytes = s
                .total_bytes
                .checked_add(count)
                .ok_or("inbound byte limit")?;
            if s.bytes > self.limits.max_inbound_bytes_per_second {
                return Err("inbound byte rate limit");
            }
            if s.total_bytes > self.limits.max_inbound_bytes {
                return Err("session inbound byte limit");
            }
            Ok(())
        })
    }
    fn accept(&self, message: &RxJsonRpcMessage<RoleServer>) -> bool {
        self.with_state(|s| {
            s.messages = s.messages.checked_add(1).ok_or("message rate limit")?;
            if s.messages > self.limits.max_messages_per_second {
                return Err("message rate limit");
            }
            match message {
                JsonRpcMessage::Request(request) => {
                    let key = id_key(&request.id);
                    if key.len() > self.limits.max_request_id_bytes {
                        return Err("request ID byte limit");
                    }
                    if s.seen.len() >= self.limits.max_requests_per_session {
                        return Err("session request count limit");
                    }
                    if !s.seen.insert(key.clone()) {
                        return Err("request ID reuse");
                    }
                    if s.pending.len() >= self.limits.max_pending_requests {
                        return Err("pending request count limit");
                    }
                    s.pending.insert(key);
                }
                JsonRpcMessage::Notification(n) => {
                    if let ClientNotification::CancelledNotification(cancelled) = &n.notification {
                        if let Some(id) = &cancelled.params.request_id {
                            s.pending.remove(&id_key(id));
                        }
                    }
                }
                JsonRpcMessage::Response(_) | JsonRpcMessage::Error(_) => {}
            }
            Ok(())
        })
    }
    fn responded(&self, key: &str) {
        if let Ok(mut s) = self.state.lock() {
            s.pending.remove(key);
        }
    }
}
fn id_key(id: &RequestId) -> String {
    serde_json::to_string(id).unwrap_or_default()
}

pub(super) struct BudgetReader<R> {
    inner: R,
    session: Arc<Session>,
}
impl<R> BudgetReader<R> {
    pub fn new(inner: R, session: Arc<Session>) -> Self {
        Self { inner, session }
    }
}
impl<R: AsyncRead + Unpin> AsyncRead for BudgetReader<R> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buffer: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        if self.session.stop.is_cancelled() {
            return Poll::Ready(Err(io::Error::other("MCP session closed")));
        }
        let before = buffer.filled().len();
        match Pin::new(&mut self.inner).poll_read(cx, buffer) {
            Poll::Ready(Ok(())) => {
                let count = buffer.filled().len() - before;
                let frames = buffer.filled()[before..]
                    .iter()
                    .filter(|&&b| b == b'\n')
                    .count();
                if self.session.bytes(count, frames) {
                    Poll::Ready(Ok(()))
                } else {
                    Poll::Ready(Err(io::Error::other("MCP inbound budget exceeded")))
                }
            }
            other => other,
        }
    }
}

pub(super) struct GuardedTransport<T> {
    inner: T,
    session: Arc<Session>,
}
impl<T> GuardedTransport<T> {
    pub fn new(inner: T, session: Arc<Session>) -> Self {
        Self { inner, session }
    }
}
impl<T: Transport<RoleServer, Error = io::Error> + 'static> Transport<RoleServer>
    for GuardedTransport<T>
{
    type Error = io::Error;
    fn send(
        &mut self,
        message: TxJsonRpcMessage<RoleServer>,
    ) -> impl std::future::Future<Output = io::Result<()>> + Send + 'static {
        let key = match &message {
            JsonRpcMessage::Response(r) => Some(id_key(&r.id)),
            JsonRpcMessage::Error(r) => r.id.as_ref().map(id_key),
            _ => None,
        };
        let session = self.session.clone();
        let send = self.inner.send(message);
        async move {
            let result = tokio::select! {biased;_=session.stop.cancelled()=>Err(io::Error::other("MCP session closed")),result=send=>result};
            if result.is_ok() {
                if let Some(key) = key {
                    session.responded(&key);
                }
            }
            result
        }
    }
    async fn receive(&mut self) -> Option<RxJsonRpcMessage<RoleServer>> {
        let message = tokio::select! {biased;_=self.session.stop.cancelled()=>return None,message=self.inner.receive()=>message};
        let Some(message) = message else {
            self.session.stop.cancel();
            return None;
        };
        self.session.accept(&message).then_some(message)
    }
    async fn close(&mut self) -> io::Result<()> {
        self.session.stop.cancel();
        self.inner.close().await
    }
}

#[cfg(test)]
mod worker_tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    async fn closing(session: &Session) {
        tokio::time::timeout(Duration::from_secs(2), async {
            while !session.state.lock().unwrap().closing {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn closed_session_rejects_late_work_without_reserving_global_permits() {
        let session = Session::new(McpLimits::default());
        session.shutdown_workers().await;
        let slots = Arc::new(Semaphore::new(1));
        assert!(matches!(
            session.reserve_worker(slots.clone()),
            Err(WorkerAdmissionError::Closed)
        ));
        assert_eq!(slots.available_permits(), 1);
    }

    #[tokio::test]
    async fn shutdown_waits_for_starting_work_and_dropped_reservations_release_both_resources() {
        let session = Session::new(McpLimits::default());
        let slots = Arc::new(Semaphore::new(1));
        let lease = session.reserve_worker(slots.clone()).unwrap();
        let stopped = session.clone();
        let shutdown = tokio::spawn(async move { stopped.shutdown_workers().await });
        closing(&session).await;
        assert!(!shutdown.is_finished());
        assert!(matches!(
            session.reserve_worker(slots.clone()),
            Err(WorkerAdmissionError::Closed)
        ));
        drop(lease);
        tokio::time::timeout(Duration::from_secs(2), shutdown)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(slots.available_permits(), 1);
    }

    #[tokio::test]
    async fn shutdown_waits_for_the_blocking_closure_after_its_handler_has_gone() {
        let session = Session::new(McpLimits::default());
        let slots = Arc::new(Semaphore::new(1));
        let lease = session.reserve_worker(slots.clone()).unwrap();
        let (started, ready) = tokio::sync::oneshot::channel();
        let (release, wait) = std::sync::mpsc::channel();
        let calls = Arc::new(AtomicUsize::new(0));
        let observed = calls.clone();
        let worker = lease.spawn(move || {
            started.send(()).unwrap();
            wait.recv_timeout(Duration::from_secs(2)).unwrap();
            calls.fetch_add(1, Ordering::SeqCst);
        });
        ready.await.unwrap();
        drop(worker);
        let stopped = session.clone();
        let shutdown = tokio::spawn(async move { stopped.shutdown_workers().await });
        closing(&session).await;
        assert!(!shutdown.is_finished());
        assert_eq!(slots.available_permits(), 0);
        release.send(()).unwrap();
        tokio::time::timeout(Duration::from_secs(2), shutdown)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(observed.load(Ordering::SeqCst), 1);
        assert_eq!(slots.available_permits(), 1);
    }

    #[tokio::test]
    async fn panicking_worker_releases_its_permit_and_tracking_token() {
        let session = Session::new(McpLimits::default());
        let slots = Arc::new(Semaphore::new(1));
        let lease = session.reserve_worker(slots.clone()).unwrap();
        let worker = lease.spawn(|| panic!("injected worker panic"));
        assert!(worker.await.unwrap_err().is_panic());
        tokio::time::timeout(Duration::from_secs(2), session.shutdown_workers())
            .await
            .unwrap();
        assert_eq!(slots.available_permits(), 1);
    }
}
