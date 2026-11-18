#![cfg(feature = "execution")]
use airust::{execution::*, Engine, KnowledgeEntry, SearchOptions};
use std::time::{Duration, Instant};
#[test]
fn deadlines_are_optional_and_overflow_is_a_typed_error() {
    let control = WorkControl::default();
    assert!(control.deadline().is_none());
    assert!(control.remaining().is_none());
    assert!(control.checkpoint().is_ok());
    assert!(matches!(
        control.with_timeout(Duration::MAX),
        Err(ExecutionError::InvalidBudget)
    ));
}
#[test]
fn zero_and_past_deadlines_fail_without_a_nanosecond_timing_assumption() {
    for control in [
        WorkControl::default().with_timeout(Duration::ZERO).unwrap(),
        WorkControl::default().with_deadline(Instant::now() - Duration::from_secs(1)),
    ] {
        assert_eq!(control.checkpoint(), Err(ExecutionError::DeadlineExceeded));
        assert_eq!(control.remaining(), Some(Duration::ZERO));
        assert!(Engine::new()
            .search_controlled(
                "Rust",
                &SearchOptions {
                    limit: 0,
                    ..SearchOptions::default()
                },
                &control
            )
            .is_err());
    }
}
#[test]
fn inherited_and_combined_deadlines_can_only_get_shorter() {
    let now = Instant::now();
    let early = now + Duration::from_secs(10);
    let later = now + Duration::from_secs(20);
    let parent = WorkControl::default().with_deadline(early);
    let child = parent
        .clone()
        .with_deadline(later)
        .with_timeout(Duration::from_secs(100))
        .unwrap();
    assert_eq!(child.deadline(), Some(early));
    for combined in [
        parent.combined(&WorkControl::default().with_deadline(later)),
        WorkControl::default()
            .with_deadline(later)
            .combined(&parent),
    ] {
        assert_eq!(combined.deadline(), Some(early));
        assert!(combined.remaining().unwrap() <= Duration::from_secs(10));
    }
    assert_eq!(
        child
            .with_deadline(now - Duration::from_secs(1))
            .checkpoint(),
        Err(ExecutionError::DeadlineExceeded)
    );
    assert!(parent.checkpoint().is_ok());
}
#[test]
fn cancellation_wins_when_both_conditions_are_set() {
    let token = CancellationToken::new();
    token.cancel();
    let control = WorkControl::new(token).with_deadline(Instant::now() - Duration::from_secs(1));
    assert_eq!(control.checkpoint(), Err(ExecutionError::Cancelled));
}
#[test]
fn deadline_control_preserves_completed_ranking() {
    let index = Engine::from_entries([
        KnowledgeEntry::new("a", "Rust memory"),
        KnowledgeEntry::new("b", "Rust ownership"),
    ])
    .unwrap();
    let control = WorkControl::default()
        .with_timeout(Duration::from_secs(30))
        .unwrap();
    assert_eq!(
        index
            .search_controlled("Rust", &SearchOptions::default(), &control)
            .unwrap()
            .iter()
            .map(|h| (&h.entry.id, h.score.to_bits()))
            .collect::<Vec<_>>(),
        index
            .search("Rust", 10)
            .iter()
            .map(|h| (&h.entry.id, h.score.to_bits()))
            .collect::<Vec<_>>()
    );
}
#[cfg(feature = "retrieval")]
#[test]
fn late_custom_source_return_is_deadline_error_instead_of_fallback() {
    use airust::retrieval::*;
    struct Slow;
    impl RetrievalSource for Slow {
        fn retrieve<'a>(
            &'a mut self,
            _: &str,
            _: &RetrievalOptions,
        ) -> Result<RetrievalBatch<'a>, RetrievalError> {
            std::thread::sleep(Duration::from_millis(100));
            let mut batch = RetrievalBatch::new("slow", vec![]);
            batch.fallback_reason = Some("fallback".into());
            Ok(batch)
        }
    }
    let control = WorkControl::default()
        .with_timeout(Duration::from_millis(30))
        .unwrap();
    assert!(matches!(
        Retriever::new(Slow).query("Rust", &RetrievalOptions::new(3).with_control(control)),
        Err(RetrievalError::Execution(ExecutionError::DeadlineExceeded))
    ));
}
#[cfg(feature = "reranking")]
#[test]
fn expired_reranking_never_invokes_provider_or_fallback() {
    use airust::reranking::*;
    struct Never;
    impl Reranker for Never {
        fn recipe(&self) -> &str {
            "never"
        }
        fn score(&mut self, _: &str, _: &[&str]) -> Result<Vec<f64>, RerankError> {
            panic!("expired scorer called")
        }
    }
    let options = RerankOptions::new(1)
        .with_failure_policy(RerankFailurePolicy::KeepOriginalOrder)
        .with_control(WorkControl::default().with_timeout(Duration::ZERO).unwrap());
    assert!(matches!(
        rerank(
            "Rust",
            &[KnowledgeEntry::new("a", "Rust")],
            &mut Never,
            &options
        ),
        Err(RerankError::Execution(ExecutionError::DeadlineExceeded))
    ));
}
#[cfg(feature = "snapshots")]
#[test]
fn expired_publication_leaves_the_dataset_revision_unchanged() {
    use airust::snapshots::*;
    let manager = DatasetManager::new("deadline", DatasetConfig::default()).unwrap();
    let old = manager.snapshot().unwrap();
    let candidate = manager
        .prepare(
            old.revision(),
            DatasetUpdate::new(vec![DatasetChange::Put(DatasetRecord::new(
                KnowledgeEntry::new("a", "Rust"),
            ))]),
            None,
        )
        .unwrap();
    let control = WorkControl::default().with_timeout(Duration::ZERO).unwrap();
    assert!(matches!(
        manager.publish_controlled(candidate, &control),
        Err(DatasetError::Execution(ExecutionError::DeadlineExceeded))
    ));
    assert_eq!(manager.snapshot().unwrap().revision(), old.revision());
    assert!(manager.snapshot().unwrap().lexical().is_empty());
}
