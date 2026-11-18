#![cfg(feature = "execution")]
use airust::execution::*;
use std::sync::{Arc, Mutex};
#[test]
fn events_validate_counts_and_expose_unknown_totals() {
    assert!(ProgressEvent::new(WorkPhase::Ranking, 2, Some(1)).is_err());
    assert_eq!(
        ProgressEvent::new(WorkPhase::Ranking, 0, None)
            .unwrap()
            .fraction(),
        None
    );
    assert_eq!(
        ProgressEvent::new(WorkPhase::Ranking, 1, Some(4))
            .unwrap()
            .fraction(),
        Some(0.25)
    );
    assert_eq!(
        ProgressEvent::new(WorkPhase::Ranking, 0, Some(0))
            .unwrap()
            .fraction(),
        Some(1.0)
    );
}
#[test]
fn combined_controls_preserve_observers_without_duplicate_delivery() {
    let a = Arc::new(Mutex::new(vec![]));
    let b = Arc::new(Mutex::new(vec![]));
    let a_capture = a.clone();
    let b_capture = b.clone();
    let left = WorkControl::default()
        .with_observer(move |e: &ProgressEvent| a_capture.lock().unwrap().push(e.sequence()));
    let right = WorkControl::default()
        .with_observer(move |e: &ProgressEvent| b_capture.lock().unwrap().push(e.sequence()));
    let combined = left.combined(&right).combined(&left);
    for n in 0..4 {
        combined.report(ProgressEvent::new(WorkPhase::Ranking, n, Some(4)).unwrap());
    }
    assert_eq!(*a.lock().unwrap(), [1, 2, 3, 4]);
    assert_eq!(*b.lock().unwrap(), [1, 2, 3, 4]);
    assert_eq!(combined.dropped_events(), 0);
}
#[test]
fn panicking_observers_are_isolated_and_release_the_busy_flag() {
    let control =
        WorkControl::default().with_observer(|_: &ProgressEvent| panic!("telemetry failure"));
    let event = ProgressEvent::new(WorkPhase::Query, 0, Some(1)).unwrap();
    control.report(event);
    control.report(event);
    assert_eq!(control.observer_failures(), 2);
    assert_eq!(control.dropped_events(), 0);
    assert!(control.checkpoint().is_ok());
}
#[test]
fn concurrent_notifications_drop_instead_of_blocking() {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Barrier,
    };
    let entered = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let calls = Arc::new(AtomicUsize::new(0));
    let a = entered.clone();
    let b = release.clone();
    let c = calls.clone();
    let control = WorkControl::default().with_observer(move |_: &ProgressEvent| {
        if c.fetch_add(1, Ordering::Relaxed) == 0 {
            a.wait();
            b.wait();
        }
    });
    let cloned = control.clone();
    let event = ProgressEvent::new(WorkPhase::Query, 0, None).unwrap();
    let thread = std::thread::spawn(move || cloned.report(event));
    entered.wait();
    control.report(event);
    assert_eq!(control.dropped_events(), 1);
    release.wait();
    thread.join().unwrap();
    control.report(event);
    assert_eq!(calls.load(Ordering::Relaxed), 2);
}
#[test]
fn reentrant_notifications_are_dropped_without_recursion() {
    let holder = Arc::new(Mutex::new(None::<WorkControl>));
    let captured = holder.clone();
    let control = WorkControl::default().with_observer(move |event: &ProgressEvent| {
        let control = captured.lock().unwrap().as_ref().unwrap().clone();
        control.report(*event);
    });
    *holder.lock().unwrap() = Some(control.clone());
    control.report(ProgressEvent::new(WorkPhase::Query, 0, None).unwrap());
    assert_eq!(control.dropped_events(), 1);
    assert_eq!(control.observer_failures(), 0);
    *holder.lock().unwrap() = None;
}
#[cfg(feature = "retrieval")]
#[test]
fn retrieval_reports_real_counts_and_finishes_only_after_success() {
    use airust::{retrieval::*, Engine, KnowledgeEntry};
    let index = Engine::from_entries(
        (0..1300).map(|i| KnowledgeEntry::new(i.to_string(), "Rust ownership")),
    )
    .unwrap();
    let events = Arc::new(Mutex::new(vec![]));
    let output = events.clone();
    let control = WorkControl::default()
        .with_observer(move |e: &ProgressEvent| output.lock().unwrap().push(*e));
    let options = RetrievalOptions::new(3)
        .with_control(control)
        .with_evidence_policy(EvidencePolicy::default());
    let response = Retriever::new(LexicalSource::new(&index))
        .query("Rust", &options)
        .unwrap();
    assert_eq!(response.hits().len(), 3);
    let events = events.lock().unwrap();
    assert!(events
        .windows(2)
        .all(|w| w[0].sequence() + 1 == w[1].sequence()));
    assert!(events
        .iter()
        .any(|e| e.phase() == WorkPhase::CollectingCandidates
            && e.completed() == 1300
            && e.total() == Some(1300)));
    assert!(events.iter().any(|e| e.phase() == WorkPhase::Ranking
        && e.completed() == 1300
        && e.total() == Some(1300)));
    assert!(events
        .iter()
        .any(|e| e.phase() == WorkPhase::AssessingEvidence
            && e.completed() == 3
            && e.total() == Some(3)));
    assert_eq!(events.last().unwrap().phase(), WorkPhase::Query);
    assert_eq!(events.last().unwrap().fraction(), Some(1.0));
    let plain = Retriever::new(LexicalSource::new(&index))
        .query(
            "Rust",
            &options.clone().with_control(WorkControl::default()),
        )
        .unwrap();
    assert_eq!(response.to_json(), plain.to_json());
}
#[cfg(feature = "retrieval")]
#[test]
fn observer_cancellation_stops_work_without_a_completed_query() {
    use airust::{retrieval::*, Engine, KnowledgeEntry};
    let token = CancellationToken::new();
    let cancel = token.clone();
    let events = Arc::new(Mutex::new(vec![]));
    let output = events.clone();
    let control = WorkControl::new(token).with_observer(move |e: &ProgressEvent| {
        output.lock().unwrap().push(*e);
        if e.phase() == WorkPhase::CollectingCandidates && e.completed() >= 512 {
            cancel.cancel();
        }
    });
    let index = Engine::from_entries((0..2000).map(|i| KnowledgeEntry::new(i.to_string(), "Rust")))
        .unwrap();
    let result = Retriever::new(LexicalSource::new(&index))
        .query("Rust", &RetrievalOptions::new(3).with_control(control));
    assert!(matches!(
        result,
        Err(RetrievalError::Execution(ExecutionError::Cancelled))
    ));
    assert!(!events
        .lock()
        .unwrap()
        .iter()
        .any(|e| e.phase() == WorkPhase::Query && e.completed() == 1));
}
#[cfg(feature = "retrieval")]
#[test]
fn observer_can_cancel_before_the_source_is_invoked() {
    use airust::retrieval::*;
    struct Never;
    impl RetrievalSource for Never {
        fn retrieve<'a>(
            &'a mut self,
            _: &str,
            _: &RetrievalOptions,
        ) -> Result<RetrievalBatch<'a>, RetrievalError> {
            panic!("cancelled source called")
        }
    }
    let token = CancellationToken::new();
    let cancel = token.clone();
    let control = WorkControl::new(token).with_observer(move |e: &ProgressEvent| {
        if e.phase() == WorkPhase::Retrieving {
            cancel.cancel();
        }
    });
    assert!(matches!(
        Retriever::new(Never).query("Rust", &RetrievalOptions::new(1).with_control(control)),
        Err(RetrievalError::Execution(ExecutionError::Cancelled))
    ));
}
#[cfg(feature = "snapshots")]
#[test]
fn publication_notifications_can_read_the_manager_and_cancel_after_commit() {
    use airust::{snapshots::*, KnowledgeEntry};
    let manager = DatasetManager::new("progress", DatasetConfig::default()).unwrap();
    let old = manager.snapshot().unwrap();
    let prepared = manager
        .prepare(
            old.revision(),
            DatasetUpdate::new(vec![DatasetChange::Put(DatasetRecord::new(
                KnowledgeEntry::new("a", "Rust"),
            ))]),
            None,
        )
        .unwrap();
    let observed = manager.clone();
    let token = CancellationToken::new();
    let cancel = token.clone();
    let control = WorkControl::new(token.clone()).with_observer(move |e: &ProgressEvent| {
        if e.phase() == WorkPhase::Publishing {
            let manager = observed.clone();
            let (tx, rx) = std::sync::mpsc::channel();
            let reader = std::thread::spawn(move || {
                tx.send(manager.snapshot().unwrap().lexical().len())
                    .unwrap()
            });
            let length = rx
                .recv_timeout(std::time::Duration::from_secs(2))
                .expect("publication observer ran inside a lock");
            reader.join().unwrap();
            assert_eq!(length, e.completed());
            if e.completed() == 1 {
                cancel.cancel();
            }
        }
    });
    let published = manager.publish_controlled(prepared, &control).unwrap();
    assert_eq!(published.lexical().len(), 1);
    assert!(token.is_cancelled());
    assert_eq!(control.observer_failures(), 0);
    assert_eq!(manager.snapshot().unwrap().revision(), published.revision());
}
#[cfg(feature = "snapshots")]
#[test]
fn observer_cancellation_before_commit_keeps_old_data() {
    use airust::{snapshots::*, KnowledgeEntry};
    let manager = DatasetManager::new("before", DatasetConfig::default()).unwrap();
    let old = manager.snapshot().unwrap();
    let prepared = manager
        .prepare(
            old.revision(),
            DatasetUpdate::new(vec![DatasetChange::Put(DatasetRecord::new(
                KnowledgeEntry::new("a", "Rust"),
            ))]),
            None,
        )
        .unwrap();
    let token = CancellationToken::new();
    let cancel = token.clone();
    let control = WorkControl::new(token).with_observer(move |e: &ProgressEvent| {
        if e.phase() == WorkPhase::Publishing && e.completed() == 0 {
            cancel.cancel();
        }
    });
    assert!(matches!(
        manager.publish_controlled(prepared, &control),
        Err(DatasetError::Execution(ExecutionError::Cancelled))
    ));
    assert_eq!(manager.snapshot().unwrap().revision(), old.revision());
    assert!(manager.snapshot().unwrap().lexical().is_empty());
}
#[cfg(feature = "reranking")]
#[test]
fn reranking_reports_real_batch_counts_and_can_cancel_before_scoring() {
    use airust::{reranking::*, KnowledgeEntry};
    struct Scorer;
    impl Reranker for Scorer {
        fn recipe(&self) -> &str {
            "progress"
        }
        fn score(&mut self, _: &str, texts: &[&str]) -> Result<Vec<f64>, RerankError> {
            Ok(vec![1.0; texts.len()])
        }
    }
    let events = Arc::new(Mutex::new(vec![]));
    let out = events.clone();
    let control =
        WorkControl::default().with_observer(move |e: &ProgressEvent| out.lock().unwrap().push(*e));
    let entries = [
        KnowledgeEntry::new("a", "Rust"),
        KnowledgeEntry::new("b", "Rust memory"),
        KnowledgeEntry::new("c", "Rust ownership"),
    ];
    let options = RerankOptions::new(3)
        .with_batch_size(2)
        .unwrap()
        .with_control(control);
    assert_eq!(
        rerank("Rust", &entries, &mut Scorer, &options)
            .unwrap()
            .scored_candidates(),
        3
    );
    assert_eq!(
        events
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.phase() == WorkPhase::Reranking)
            .map(|e| (e.completed(), e.total()))
            .collect::<Vec<_>>(),
        [(0, Some(3)), (2, Some(3)), (3, Some(3))]
    );
    struct Never;
    impl Reranker for Never {
        fn recipe(&self) -> &str {
            "never"
        }
        fn score(&mut self, _: &str, _: &[&str]) -> Result<Vec<f64>, RerankError> {
            panic!("cancelled provider called")
        }
    }
    let token = CancellationToken::new();
    let cancel = token.clone();
    let control = WorkControl::new(token).with_observer(move |e: &ProgressEvent| {
        if e.phase() == WorkPhase::Reranking {
            cancel.cancel();
        }
    });
    assert!(matches!(
        rerank(
            "Rust",
            &entries,
            &mut Never,
            &RerankOptions::new(3).with_control(control)
        ),
        Err(RerankError::Execution(ExecutionError::Cancelled))
    ));
}
#[cfg(feature = "retrieval")]
#[test]
fn completed_response_survives_a_panicking_cancelling_completion_callback() {
    use airust::{retrieval::*, Engine, KnowledgeEntry};
    let token = CancellationToken::new();
    let cancel = token.clone();
    let control = WorkControl::new(token.clone()).with_observer(move |e: &ProgressEvent| {
        if e.phase() == WorkPhase::Query && e.completed() == 1 {
            cancel.cancel();
            panic!("completion display failed");
        }
    });
    let index = Engine::from_entries([KnowledgeEntry::new("a", "Rust")]).unwrap();
    let response = Retriever::new(LexicalSource::new(&index))
        .query(
            "Rust",
            &RetrievalOptions::new(1).with_control(control.clone()),
        )
        .unwrap();
    assert_eq!(response.hits().len(), 1);
    assert_eq!(control.observer_failures(), 1);
    assert!(token.is_cancelled());
}
#[cfg(all(feature = "retrieval", feature = "hybrid"))]
#[test]
fn hybrid_progress_exposes_embedding_and_real_fusion_contributions() {
    use airust::{retrieval::*, semantic::*, Engine, KnowledgeEntry};
    struct Provider(EmbeddingSpace);
    impl EmbeddingProvider for Provider {
        fn space(&self) -> &EmbeddingSpace {
            &self.0
        }
        fn embed(
            &mut self,
            texts: &[&str],
            _: EmbeddingPurpose,
        ) -> Result<Vec<Vec<f32>>, EmbeddingError> {
            Ok(vec![vec![1.0, 0.0]; texts.len()])
        }
    }
    let mut provider = Provider(EmbeddingSpace::new("progress", 2).unwrap());
    let index = SemanticIndex::from_index(
        Engine::from_entries([
            KnowledgeEntry::new("a", "Rust"),
            KnowledgeEntry::new("b", "Rust memory"),
        ])
        .unwrap(),
        &mut provider,
        2,
    )
    .unwrap();
    let events = Arc::new(Mutex::new(vec![]));
    let out = events.clone();
    let control =
        WorkControl::default().with_observer(move |e: &ProgressEvent| out.lock().unwrap().push(*e));
    let response = Retriever::new(HybridSource::new(&index, &mut provider))
        .query("Rust", &RetrievalOptions::new(2).with_control(control))
        .unwrap();
    assert_eq!(response.hits().len(), 2);
    let events = events.lock().unwrap();
    assert!(events
        .iter()
        .any(|e| e.phase() == WorkPhase::Embedding && e.completed() == 1 && e.total() == Some(1)));
    assert!(events
        .iter()
        .any(|e| e.phase() == WorkPhase::Fusing && e.completed() == 4 && e.total() == Some(4)));
}

#[test]
fn controls_preserve_their_original_thread_and_unwind_auto_traits() {
    fn check_traits<T: Send + Sync + std::panic::UnwindSafe + std::panic::RefUnwindSafe>() {}
    check_traits::<WorkControl>();
    let control = WorkControl::default().with_observer(|_: &ProgressEvent| {});
    assert!(std::panic::catch_unwind(
        || control.report(ProgressEvent::new(WorkPhase::Query, 0, None).unwrap())
    )
    .is_ok());
}
