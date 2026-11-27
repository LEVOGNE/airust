use super::{ProcessOptions, ProcessReranker};
use crate::reranking::{RerankError, Reranker};
use std::{
    path::PathBuf,
    sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError},
    time::{Duration, Instant},
};

struct Factory {
    options: ProcessOptions,
    model_directory: PathBuf,
    runtime_library: PathBuf,
    threads: usize,
    snapshot_directory: PathBuf,
}

impl Factory {
    fn open(&self) -> Result<ProcessReranker, RerankError> {
        ProcessReranker::multilingual_minilm_v2_from_snapshot(
            self.options.clone(),
            &self.model_directory,
            &self.runtime_library,
            self.threads,
            &self.snapshot_directory,
        )
    }
}

#[derive(Default)]
struct Slots {
    idle: Vec<ProcessReranker>,
    live: usize,
    waiting: usize,
}

struct Shared {
    factory: Factory,
    slots: Mutex<Slots>,
    released: Condvar,
    max_workers: usize,
    queue_limit: usize,
}

impl Shared {
    fn slots(&self) -> MutexGuard<'_, Slots> {
        self.slots.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn release(&self, worker: Option<ProcessReranker>) {
        let mut slots = self.slots();
        match worker {
            Some(worker) => slots.idle.push(worker),
            None => slots.live -= 1,
        }
        drop(slots);
        self.released.notify_one();
    }
}

/// Bounded admission for native-model workers. At most `max_workers` processes
/// exist at once and at most `queue_limit` callers wait; further callers get
/// [`RerankError::Busy`] immediately. A worker that fails is retired and
/// replaced on demand. Dropping the pool and every lease reaps all workers.
pub struct ProcessPool {
    shared: Arc<Shared>,
}

impl ProcessPool {
    pub fn multilingual_minilm_v2_from_snapshot(
        options: ProcessOptions,
        model_directory: impl Into<PathBuf>,
        runtime_library: impl Into<PathBuf>,
        threads: usize,
        snapshot_directory: impl Into<PathBuf>,
        max_workers: usize,
        queue_limit: usize,
    ) -> Result<Self, RerankError> {
        if !(1..=64).contains(&max_workers) || queue_limit > 4096 {
            return Err(RerankError::Configuration(
                "pool needs 1..64 workers and a queue limit of at most 4096",
            ));
        }
        Ok(Self {
            shared: Arc::new(Shared {
                factory: Factory {
                    options,
                    model_directory: model_directory.into(),
                    runtime_library: runtime_library.into(),
                    threads,
                    snapshot_directory: snapshot_directory.into(),
                },
                slots: Mutex::new(Slots::default()),
                released: Condvar::new(),
                max_workers,
                queue_limit,
            }),
        })
    }

    /// Wait at most `wait` for a worker; returns `Busy` when the queue is full
    /// or the wait expires.
    pub fn acquire(&self, wait: Duration) -> Result<PooledReranker, RerankError> {
        let deadline = Instant::now()
            .checked_add(wait)
            .ok_or(RerankError::Configuration(
                "pool wait exceeds the clock range",
            ))?;
        self.admit(deadline, || Ok(()))
    }

    /// Like [`ProcessPool::acquire`], bounded by the control's deadline and
    /// cancelled through its token.
    #[cfg(feature = "execution")]
    pub fn acquire_controlled(
        &self,
        control: &crate::execution::WorkControl,
    ) -> Result<PooledReranker, RerankError> {
        let deadline = control
            .deadline()
            .unwrap_or_else(|| Instant::now() + Duration::from_secs(3600));
        self.admit(deadline, || control.checkpoint().map_err(RerankError::from))
    }

    pub fn live_workers(&self) -> usize {
        self.shared.slots().live
    }

    fn admit(
        &self,
        deadline: Instant,
        checkpoint: impl Fn() -> Result<(), RerankError>,
    ) -> Result<PooledReranker, RerankError> {
        let shared = &self.shared;
        let mut slots = shared.slots();
        let mut queued = false;
        let outcome = loop {
            if let Err(error) = checkpoint() {
                break Err(error);
            }
            if let Some(worker) = slots.idle.pop() {
                break Ok(Some(worker));
            }
            if slots.live < shared.max_workers {
                slots.live += 1;
                break Ok(None);
            }
            if !queued {
                if slots.waiting >= shared.queue_limit {
                    return Err(RerankError::Busy("worker queue is full"));
                }
                slots.waiting += 1;
                queued = true;
            }
            let now = Instant::now();
            if now >= deadline {
                break Err(RerankError::Busy("no worker became available in time"));
            }
            let pause = (deadline - now).min(Duration::from_millis(20));
            slots = shared
                .released
                .wait_timeout(slots, pause)
                .unwrap_or_else(PoisonError::into_inner)
                .0;
        };
        if queued {
            slots.waiting -= 1;
        }
        drop(slots);
        let worker = match outcome? {
            Some(worker) => worker,
            None => match shared.factory.open() {
                Ok(worker) => worker,
                Err(error) => {
                    shared.release(None);
                    return Err(error);
                }
            },
        };
        Ok(PooledReranker {
            shared: Arc::clone(shared),
            worker: Some(worker),
            failed: false,
        })
    }
}

/// One leased worker. Returning it keeps a healthy process for reuse; a lease
/// whose scoring failed retires its process so the next caller gets a new one.
pub struct PooledReranker {
    shared: Arc<Shared>,
    worker: Option<ProcessReranker>,
    failed: bool,
}

impl PooledReranker {
    pub fn worker_id(&self) -> Option<u32> {
        self.worker.as_ref().map(ProcessReranker::worker_id)
    }
}

impl Reranker for PooledReranker {
    fn recipe(&self) -> &str {
        self.worker.as_ref().map_or("", |worker| worker.recipe())
    }

    fn score(&mut self, query: &str, documents: &[&str]) -> Result<Vec<f64>, RerankError> {
        let worker = self
            .worker
            .as_mut()
            .ok_or(RerankError::Output("pooled worker already released"))?;
        let result = worker.score(query, documents);
        if let Err(error) = &result {
            self.failed = !matches!(
                error,
                RerankError::Budget(_) | RerankError::Configuration(_) | RerankError::EmptyQuery
            );
        }
        result
    }
}

impl Drop for PooledReranker {
    fn drop(&mut self) {
        let worker = self.worker.take();
        if self.failed {
            drop(worker);
            self.shared.release(None);
        } else {
            self.shared.release(worker);
        }
    }
}
