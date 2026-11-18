#![cfg(feature = "execution")]
use airust::{execution::*, Engine, KnowledgeEntry, SearchOptions};
#[test]
fn cancellation_is_shared_one_way_and_first_signal_wins() {
    let token = CancellationToken::new();
    let control = WorkControl::new(token.clone());
    assert!(control.checkpoint().is_ok());
    let wins = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..32)
            .map(|_| {
                let token = token.clone();
                scope.spawn(move || usize::from(token.cancel()))
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .sum::<usize>()
    });
    assert_eq!(wins, 1);
    assert_eq!(control.checkpoint(), Err(ExecutionError::Cancelled));
    assert!(!token.cancel());
    assert!(WorkControl::default().checkpoint().is_ok());
}
#[test]
fn controlled_core_search_preserves_ranking_and_rejects_cancelled_zero_limits() {
    let index = Engine::from_entries(
        (0..500).map(|i| KnowledgeEntry::new(i.to_string(), format!("Rust ownership topic{i}"))),
    )
    .unwrap();
    let token = CancellationToken::new();
    let control = WorkControl::new(token.clone());
    for algorithm in [
        airust::SearchAlgorithm::Bm25,
        airust::SearchAlgorithm::Tfidf,
    ] {
        let options = SearchOptions {
            algorithm,
            limit: 23,
            ..SearchOptions::default()
        };
        let expected = index.search_with_options("Rust topic5", &options);
        let actual = index
            .search_controlled("Rust topic5", &options, &control)
            .unwrap();
        assert_eq!(
            actual
                .iter()
                .map(|r| (&r.entry.id, r.score.to_bits()))
                .collect::<Vec<_>>(),
            expected
                .iter()
                .map(|r| (&r.entry.id, r.score.to_bits()))
                .collect::<Vec<_>>()
        );
    }
    token.cancel();
    assert!(matches!(
        index.search_controlled(
            "Rust",
            &SearchOptions {
                limit: 0,
                ..SearchOptions::default()
            },
            &control
        ),
        Err(ExecutionError::Cancelled)
    ));
}
#[cfg(feature = "retrieval")]
#[test]
fn retrieval_never_calls_a_source_if_already_cancelled() {
    use airust::retrieval::*;
    struct Never;
    impl RetrievalSource for Never {
        fn retrieve<'a>(
            &'a mut self,
            _: &str,
            _: &RetrievalOptions,
        ) -> Result<RetrievalBatch<'a>, RetrievalError> {
            panic!("source must not run")
        }
    }
    let token = CancellationToken::new();
    token.cancel();
    for limit in [0, 3] {
        let result = Retriever::new(Never).query(
            "Rust",
            &RetrievalOptions::new(limit).with_control(WorkControl::new(token.clone())),
        );
        assert!(matches!(
            result,
            Err(RetrievalError::Execution(ExecutionError::Cancelled))
        ));
    }
}
#[cfg(feature = "retrieval")]
#[test]
fn cancellation_during_custom_source_overrides_fallback_and_errors() {
    use airust::retrieval::*;
    struct Source(CancellationToken, bool);
    impl RetrievalSource for Source {
        fn retrieve<'a>(
            &'a mut self,
            _: &str,
            _: &RetrievalOptions,
        ) -> Result<RetrievalBatch<'a>, RetrievalError> {
            self.0.cancel();
            if self.1 {
                Err(RetrievalError::Source("provider failed".into()))
            } else {
                {
                    let mut batch = RetrievalBatch::new("cancelled", vec![]);
                    batch.fallback_reason = Some("degraded".into());
                    Ok(batch)
                }
            }
        }
    }
    for fail in [false, true] {
        let token = CancellationToken::new();
        let result = Retriever::new(Source(token.clone(), fail)).query(
            "Rust",
            &RetrievalOptions::new(3).with_control(WorkControl::new(token)),
        );
        assert!(matches!(
            result,
            Err(RetrievalError::Execution(ExecutionError::Cancelled))
        ));
    }
}
#[cfg(feature = "reranking")]
#[test]
fn cancellation_during_reranking_does_not_keep_original_order() {
    use airust::reranking::*;
    struct Scorer(CancellationToken, usize);
    impl Reranker for Scorer {
        fn recipe(&self) -> &str {
            "cancel-fixture"
        }
        fn score(&mut self, _: &str, docs: &[&str]) -> Result<Vec<f64>, RerankError> {
            self.1 += 1;
            self.0.cancel();
            assert_eq!(docs.len(), 1);
            Err(RerankError::Provider("blocked then cancelled".into()))
        }
    }
    let token = CancellationToken::new();
    let mut scorer = Scorer(token.clone(), 0);
    let entries = [
        KnowledgeEntry::new("a", "Rust ownership"),
        KnowledgeEntry::new("b", "Rust memory"),
    ];
    let options = RerankOptions::new(2)
        .with_batch_size(1)
        .unwrap()
        .with_failure_policy(RerankFailurePolicy::KeepOriginalOrder)
        .with_control(WorkControl::new(token));
    assert!(matches!(
        rerank("Rust", &entries, &mut scorer, &options),
        Err(RerankError::Execution(ExecutionError::Cancelled))
    ));
    assert_eq!(scorer.1, 1);
}
#[cfg(all(feature = "retrieval", feature = "reranking"))]
#[test]
fn request_control_does_not_erase_existing_reranking_control() {
    use airust::{reranking::*, retrieval::*};
    struct Never;
    impl Reranker for Never {
        fn recipe(&self) -> &str {
            "never"
        }
        fn score(&mut self, _: &str, _: &[&str]) -> Result<Vec<f64>, RerankError> {
            panic!("cancelled scorer called")
        }
    }
    let token = CancellationToken::new();
    token.cancel();
    let engine = Engine::from_entries([KnowledgeEntry::new("a", "Rust memory")]).unwrap();
    let result = Retriever::new(LexicalSource::new(&engine))
        .with_reranker(
            &mut Never,
            RerankOptions::new(1).with_control(WorkControl::new(token)),
        )
        .query(
            "Rust",
            &RetrievalOptions::new(1).with_control(WorkControl::default()),
        );
    assert!(matches!(
        result,
        Err(RetrievalError::Rerank(RerankError::Execution(
            ExecutionError::Cancelled
        )))
    ));
}
#[cfg(feature = "advanced-search")]
#[test]
fn controlled_phrase_search_keeps_german_evidence() {
    use airust::text_search::*;
    let token = CancellationToken::new();
    let control = WorkControl::new(token.clone());
    let index = TextIndex::from_entries_controlled(
        AnalyzerConfig::german(),
        [KnowledgeEntry::new("a", "Häuser haben Türen")],
        &control,
    )
    .unwrap();
    let options = TextSearchOptions::new(3);
    let actual = index
        .search_controlled("\"Haus haben\"", &options, &control)
        .unwrap();
    let expected = index.search("\"Haus haben\"", &options).unwrap();
    assert_eq!(
        serde_json::to_string(&actual).unwrap(),
        serde_json::to_string(&expected).unwrap()
    );
    token.cancel();
    assert!(matches!(
        index.search_controlled("Haus", &options, &control),
        Err(TextSearchError::Execution(ExecutionError::Cancelled))
    ));
    assert!(TextIndex::from_entries_controlled(AnalyzerConfig::german(), [], &control).is_err());
}
#[cfg(feature = "snapshots")]
#[test]
fn cancelled_preparation_and_publication_leave_the_old_revision_intact() {
    use airust::snapshots::*;
    let manager = DatasetManager::new("cancel", DatasetConfig::default()).unwrap();
    let old = manager.snapshot().unwrap();
    let update = || {
        DatasetUpdate::new(vec![DatasetChange::Put(DatasetRecord::new(
            KnowledgeEntry::new("a", "Rust"),
        ))])
    };
    let candidate = manager.prepare(old.revision(), update(), None).unwrap();
    let token = CancellationToken::new();
    token.cancel();
    let control = WorkControl::new(token);
    assert!(matches!(
        manager.prepare_controlled(old.revision(), update(), None, &control),
        Err(DatasetError::Execution(ExecutionError::Cancelled))
    ));
    assert!(matches!(
        manager.prepare_replace_controlled(old.revision(), vec![], None, None, &control),
        Err(DatasetError::Execution(ExecutionError::Cancelled))
    ));
    assert!(matches!(
        manager.prepare_model_controlled(old.revision(), None, &control),
        Err(DatasetError::Execution(ExecutionError::Cancelled))
    ));
    assert!(matches!(
        manager.publish_controlled(candidate, &control),
        Err(DatasetError::Execution(ExecutionError::Cancelled))
    ));
    assert_eq!(manager.snapshot().unwrap().revision(), old.revision());
    assert!(old.lexical().is_empty());
    let published = manager
        .publish_controlled(
            manager
                .prepare_controlled(old.revision(), update(), None, &WorkControl::default())
                .unwrap(),
            &WorkControl::default(),
        )
        .unwrap();
    assert_eq!(published.lexical().len(), 1);
    assert!(old.lexical().is_empty());
}
#[cfg(feature = "snapshots")]
#[test]
fn provider_cancellation_between_snapshot_batches_does_not_publish_or_fallback() {
    use airust::{semantic::*, snapshots::*};
    struct Provider(EmbeddingSpace, CancellationToken, usize, bool);
    impl EmbeddingProvider for Provider {
        fn space(&self) -> &EmbeddingSpace {
            &self.0
        }
        fn embed(
            &mut self,
            texts: &[&str],
            _: EmbeddingPurpose,
        ) -> Result<Vec<Vec<f32>>, EmbeddingError> {
            self.2 += 1;
            self.1.cancel();
            if self.3 {
                Err(EmbeddingError::Provider("failure".into()))
            } else {
                Ok(vec![vec![1.0, 0.0]; texts.len()])
            }
        }
    }
    for fail in [false, true] {
        let token = CancellationToken::new();
        let control = WorkControl::new(token.clone());
        let space = EmbeddingSpace::new("cancel-model", 2).unwrap();
        let manager = DatasetManager::new(
            "cancel-provider",
            DatasetConfig::default()
                .with_embedding_space(space.clone())
                .unwrap()
                .with_batch_size(1)
                .unwrap(),
        )
        .unwrap();
        let old = manager.snapshot().unwrap();
        let records = (0..4)
            .map(|i| DatasetRecord::new(KnowledgeEntry::new(i.to_string(), "Rust")))
            .collect();
        let mut provider = Provider(space, token, 0, fail);
        assert!(matches!(
            manager.prepare_replace_controlled(
                old.revision(),
                records,
                None,
                Some(&mut provider),
                &control
            ),
            Err(DatasetError::Embedding(EmbeddingError::Execution(
                ExecutionError::Cancelled
            )))
        ));
        assert_eq!(provider.2, 1);
        assert_eq!(manager.snapshot().unwrap().revision(), old.revision());
        assert_eq!(manager.snapshot().unwrap().lexical().len(), 0);
    }
}
#[cfg(all(feature = "hybrid", feature = "retrieval"))]
#[test]
fn hybrid_provider_cancellation_is_typed_and_never_lexical_fallback() {
    use airust::{hybrid::*, retrieval::*, semantic::*};
    struct Provider(EmbeddingSpace, CancellationToken, bool);
    impl EmbeddingProvider for Provider {
        fn space(&self) -> &EmbeddingSpace {
            &self.0
        }
        fn embed(
            &mut self,
            texts: &[&str],
            _: EmbeddingPurpose,
        ) -> Result<Vec<Vec<f32>>, EmbeddingError> {
            if self.2 {
                self.1.cancel();
                return Err(EmbeddingError::Provider("failure".into()));
            }
            Ok(vec![vec![1.0, 0.0]; texts.len()])
        }
    }
    let token = CancellationToken::new();
    let mut provider = Provider(
        EmbeddingSpace::new("cancel-model", 2).unwrap(),
        token.clone(),
        false,
    );
    let index = SemanticIndex::from_index(
        Engine::from_entries([KnowledgeEntry::new("a", "Rust")]).unwrap(),
        &mut provider,
        1,
    )
    .unwrap();
    provider.2 = true;
    let options =
        HybridSearchOptions::new(1).with_failure_policy(SemanticFailurePolicy::LexicalOnly);
    let result = Retriever::new(HybridSource::new(&index, &mut provider).with_options(options))
        .query(
            "Rust",
            &RetrievalOptions::new(1).with_control(WorkControl::new(token)),
        );
    assert!(matches!(
        result,
        Err(RetrievalError::Execution(ExecutionError::Cancelled))
    ));
}
#[test]
fn combined_controls_preserve_both_signals_without_cancelling_parents() {
    for cancel_first in [false, true] {
        let a = WorkControl::default();
        let b = WorkControl::default();
        let combined = a.combined(&b).combined(&a);
        assert!(combined.checkpoint().is_ok());
        if cancel_first {
            a.token().cancel();
            assert!(!b.token().is_cancelled());
        } else {
            b.token().cancel();
            assert!(!a.token().is_cancelled());
        }
        assert_eq!(combined.checkpoint(), Err(ExecutionError::Cancelled));
    }
}
