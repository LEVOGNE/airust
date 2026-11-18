#![doc = include_str!("../docs/EXECUTION.md")]
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use thiserror::Error;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum ExecutionError {
    #[error("operation cancelled")]
    Cancelled,
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
}
impl WorkControl {
    pub fn new(token: CancellationToken) -> Self {
        Self {
            token,
            parents: Vec::new(),
        }
    }
    pub fn token(&self) -> &CancellationToken {
        &self.token
    }
    pub fn combined(&self, other: &Self) -> Self {
        let mut combined = self.clone();
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
        } else {
            Ok(())
        }
    }
}
