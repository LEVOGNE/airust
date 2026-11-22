use super::transport::Session;
use std::{
    future::Future,
    time::{Duration, Instant},
};
use tokio_util::sync::CancellationToken;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Delivery {
    Sent,
    Cancelled,
    Deadline,
    Transport,
}

pub(super) async fn deliver<F, E>(
    send: F,
    cancellation: &CancellationToken,
    session: Option<&Session>,
    deadline: Instant,
) -> Delivery
where
    F: Future<Output = Result<(), E>>,
{
    let stopped = async {
        if let Some(session) = session {
            session.stop.cancelled().await;
        } else {
            std::future::pending::<()>().await;
        }
    };
    tokio::select! {biased;
        _=cancellation.cancelled()=>Delivery::Cancelled,
        _=stopped=>Delivery::Cancelled,
        _=tokio::time::sleep_until(tokio::time::Instant::from_std(deadline))=>Delivery::Deadline,
        result=tokio::time::timeout(Duration::from_millis(100),send)=>{
            if matches!(result,Ok(Ok(()))) { Delivery::Sent } else { Delivery::Transport }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::McpLimits;

    #[tokio::test]
    async fn expired_request_wins_over_an_already_ready_notification() {
        let token = CancellationToken::new();
        assert_eq!(
            deliver(
                std::future::ready(Ok::<_, ()>(())),
                &token,
                None,
                Instant::now() - Duration::from_secs(1)
            )
            .await,
            Delivery::Deadline
        );
    }

    #[tokio::test]
    async fn short_deadline_interrupts_a_stalled_send_and_session_remains_usable() {
        let token = CancellationToken::new();
        let session = Session::new(McpLimits::default());
        let result = tokio::time::timeout(
            Duration::from_secs(2),
            deliver(
                std::future::pending::<Result<(), ()>>(),
                &token,
                Some(&session),
                Instant::now() + Duration::from_millis(5),
            ),
        )
        .await
        .unwrap();
        assert_eq!(result, Delivery::Deadline);
        assert!(session.reason().is_none());
        assert!(!session.stop.is_cancelled());
        assert_eq!(
            deliver(
                std::future::ready(Ok::<_, ()>(())),
                &token,
                Some(&session),
                Instant::now() + Duration::from_secs(2)
            )
            .await,
            Delivery::Sent
        );
    }

    #[tokio::test]
    async fn request_and_session_cancellation_interrupt_notification_delivery() {
        let token = CancellationToken::new();
        let session = Session::new(McpLimits::default());
        token.cancel();
        assert_eq!(
            deliver(
                std::future::pending::<Result<(), ()>>(),
                &token,
                Some(&session),
                Instant::now() + Duration::from_secs(2)
            )
            .await,
            Delivery::Cancelled
        );
        let token = CancellationToken::new();
        session.stop.cancel();
        assert_eq!(
            deliver(
                std::future::pending::<Result<(), ()>>(),
                &token,
                Some(&session),
                Instant::now() + Duration::from_secs(2)
            )
            .await,
            Delivery::Cancelled
        );
    }

    #[tokio::test]
    async fn actual_delivery_error_and_transport_stall_remain_distinct_from_deadline() {
        let token = CancellationToken::new();
        let deadline = Instant::now() + Duration::from_secs(2);
        assert_eq!(
            deliver(std::future::ready(Err::<(), _>(())), &token, None, deadline).await,
            Delivery::Transport
        );
        assert_eq!(
            deliver(
                std::future::pending::<Result<(), ()>>(),
                &token,
                None,
                deadline
            )
            .await,
            Delivery::Transport
        );
    }
}
