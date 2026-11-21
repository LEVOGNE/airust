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
use tokio::io::{AsyncRead, ReadBuf};
use tokio_util::sync::CancellationToken;

pub(super) struct Session {
    pub stop: CancellationToken,
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
}
impl Session {
    pub fn new(limits: McpLimits) -> Arc<Self> {
        Arc::new(Self {
            stop: CancellationToken::new(),
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
                _ => {}
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
