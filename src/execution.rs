#![doc = include_str!("../docs/EXECUTION.md")]
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use thiserror::Error;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum ExecutionError {
    #[error("operation cancelled")]
    Cancelled,
    #[error("operation time budget exceeded")]
    DeadlineExceeded,
    #[error("time budget exceeds the monotonic clock range")]
    InvalidBudget,
}
#[derive(Debug, Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}
impl CancellationToken {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn cancel(&self) -> bool {
        !self.cancelled.swap(true, Ordering::AcqRel)
    }
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}
#[derive(Debug, Clone, Default)]
pub struct WorkControl {
    token: CancellationToken,
    parents: Vec<CancellationToken>,
    deadline: Option<Instant>,
}
impl WorkControl {
    pub fn new(token: CancellationToken) -> Self {
        Self {
            token,
            parents: Vec::new(),
            deadline: None,
        }
    }
    pub fn with_deadline(mut self, deadline: Instant) -> Self {
        self.deadline = Some(self.deadline.map_or(deadline, |old| old.min(deadline)));
        self
    }
    pub fn with_timeout(self, timeout: Duration) -> Result<Self, ExecutionError> {
        let deadline = Instant::now()
            .checked_add(timeout)
            .ok_or(ExecutionError::InvalidBudget)?;
        Ok(self.with_deadline(deadline))
    }
    pub fn deadline(&self) -> Option<Instant> {
        self.deadline
    }
    pub fn remaining(&self) -> Option<Duration> {
        self.deadline
            .map(|deadline| deadline.saturating_duration_since(Instant::now()))
    }
    pub fn token(&self) -> &CancellationToken {
        &self.token
    }
    pub fn combined(&self, other: &Self) -> Self {
        let mut combined = self.clone();
        if let Some(deadline) = other.deadline {
            combined = combined.with_deadline(deadline);
        }
        for token in std::iter::once(&other.token).chain(other.parents.iter()) {
            if !Arc::ptr_eq(&combined.token.cancelled, &token.cancelled)
                && !combined
                    .parents
                    .iter()
                    .any(|p| Arc::ptr_eq(&p.cancelled, &token.cancelled))
            {
                combined.parents.push(token.clone());
            }
        }
        combined
    }
    pub fn checkpoint(&self) -> Result<(), ExecutionError> {
        if self.token.is_cancelled() || self.parents.iter().any(CancellationToken::is_cancelled) {
            Err(ExecutionError::Cancelled)
        } else if self
            .deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            Err(ExecutionError::DeadlineExceeded)
        } else {
            Ok(())
        }
    }
}
