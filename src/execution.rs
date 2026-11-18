#![doc = include_str!("../docs/EXECUTION.md")]
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
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
#[derive(Clone, Default)]
pub struct WorkControl {
    token: CancellationToken,
    parents: Vec<CancellationToken>,
    deadline: Option<Instant>,
    observers: Vec<Arc<ObserverSlot>>,
}
impl WorkControl {
    pub fn new(token: CancellationToken) -> Self {
        Self {
            token,
            parents: Vec::new(),
            deadline: None,
            observers: Vec::new(),
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
    pub fn with_observer(mut self, observer: impl ProgressObserver + 'static) -> Self {
        self.observers = vec![Arc::new(ObserverSlot {
            callback: Box::new(observer),
            busy: AtomicBool::new(false),
            sequence: AtomicUsize::new(0),
            panics: AtomicUsize::new(0),
            dropped: AtomicUsize::new(0),
        })];
        self
    }
    pub fn observer_failures(&self) -> usize {
        self.observers.iter().fold(0usize, |sum, slot| {
            sum.saturating_add(slot.panics.load(Ordering::Relaxed))
        })
    }
    pub fn dropped_events(&self) -> usize {
        self.observers.iter().fold(0usize, |sum, slot| {
            sum.saturating_add(slot.dropped.load(Ordering::Relaxed))
        })
    }
    pub fn report(&self, event: ProgressEvent) {
        for slot in &self.observers {
            if slot
                .busy
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_err()
            {
                increment(&slot.dropped);
                continue;
            }
            let mut event = event;
            event.sequence = increment(&slot.sequence);
            if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                slot.callback.on_progress(&event)
            }))
            .is_err()
            {
                increment(&slot.panics);
            }
            slot.busy.store(false, Ordering::Release);
        }
    }
    pub(crate) fn progress(&self, phase: WorkPhase, completed: usize, total: Option<usize>) {
        if self.observers.is_empty() {
            return;
        }
        if let Ok(event) = ProgressEvent::new(phase, completed, total) {
            self.report(event);
        }
    }
    pub fn token(&self) -> &CancellationToken {
        &self.token
    }
    pub fn combined(&self, other: &Self) -> Self {
        let mut combined = self.clone();
        for slot in &other.observers {
            if !combined.observers.iter().any(|own| Arc::ptr_eq(own, slot)) {
                combined.observers.push(slot.clone());
            }
        }
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

impl std::fmt::Debug for WorkControl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkControl")
            .field("token", &self.token)
            .field("parents", &self.parents)
            .field("deadline", &self.deadline)
            .field("observers", &self.observers.len())
            .finish()
    }
}
fn increment(value: &AtomicUsize) -> usize {
    let mut observed = value.load(Ordering::Relaxed);
    loop {
        let next = observed.saturating_add(1);
        match value.compare_exchange_weak(observed, next, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => return next,
            Err(current) => observed = current,
        }
    }
}
struct ObserverSlot {
    callback: Box<dyn ProgressObserver>,
    busy: AtomicBool,
    sequence: AtomicUsize,
    panics: AtomicUsize,
    dropped: AtomicUsize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum WorkPhase {
    Query,
    Retrieving,
    CollectingCandidates,
    Ranking,
    Embedding,
    Reranking,
    Fusing,
    SelectingExcerpts,
    AssessingEvidence,
    Serializing,
    ValidatingDataset,
    StagingDataset,
    AnalyzingText,
    Publishing,
    Routing,
    Features,
    Training,
    Reasoning,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("progress completed count exceeds its total")]
pub struct ProgressError;
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct ProgressEvent {
    phase: WorkPhase,
    completed: usize,
    total: Option<usize>,
    sequence: usize,
}
impl ProgressEvent {
    pub fn new(
        phase: WorkPhase,
        completed: usize,
        total: Option<usize>,
    ) -> Result<Self, ProgressError> {
        if total.is_some_and(|total| completed > total) {
            return Err(ProgressError);
        }
        Ok(Self {
            phase,
            completed,
            total,
            sequence: 0,
        })
    }
    pub fn phase(&self) -> WorkPhase {
        self.phase
    }
    pub fn completed(&self) -> usize {
        self.completed
    }
    pub fn total(&self) -> Option<usize> {
        self.total
    }
    pub fn sequence(&self) -> usize {
        self.sequence
    }
    pub fn fraction(&self) -> Option<f64> {
        self.total.map(|total| {
            if total == 0 {
                1.0
            } else {
                self.completed as f64 / total as f64
            }
        })
    }
}
pub trait ProgressObserver: Send + Sync + std::panic::RefUnwindSafe {
    fn on_progress(&self, event: &ProgressEvent);
}
impl<F: Fn(&ProgressEvent) + Send + Sync + std::panic::RefUnwindSafe> ProgressObserver for F {
    fn on_progress(&self, event: &ProgressEvent) {
        self(event);
    }
}

#[cfg(test)]
mod counter_tests {
    use super::*;
    #[test]
    fn concurrent_counters_saturate_without_wrapping_or_losing_early_values() {
        let value = Arc::new(AtomicUsize::new(usize::MAX - 3));
        let values = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..16)
                .map(|_| {
                    let value = value.clone();
                    scope.spawn(move || increment(&value))
                })
                .collect();
            handles
                .into_iter()
                .map(|handle| handle.join().unwrap())
                .collect::<std::collections::BTreeSet<_>>()
        });
        assert_eq!(
            values,
            [usize::MAX - 2, usize::MAX - 1, usize::MAX]
                .into_iter()
                .collect()
        );
        assert_eq!(value.load(Ordering::Relaxed), usize::MAX);
    }
}
