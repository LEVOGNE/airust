#![cfg(feature = "hnsw")]

use airust::{
    ann::{AnnSearchMode, AnnSearchOptions, HnswConfig, HnswIndex},
    semantic::{
        Embedding, EmbeddingError, EmbeddingProvider, EmbeddingPurpose, EmbeddingSpace,
        SemanticIndex, SemanticSearchOptions,
    },
    Context, KnowledgeEntry,
};
use serde_json::json;

fn space() -> EmbeddingSpace {
    EmbeddingSpace::new("ann-test", 3).unwrap()
}
fn vector(v: [f32; 3]) -> Embedding {
    Embedding::new(space(), v.to_vec()).unwrap()
}
fn index() -> SemanticIndex {
    let mut index = SemanticIndex::new(space());
    for (id, v) in [
        ("a", [1.0, 0.0, 0.0]),
        ("b", [0.8, 0.2, 0.0]),
        ("c", [0.0, 1.0, 0.0]),
        ("d", [-1.0, 0.0, 0.0]),
    ] {
        index
            .upsert_embedded(
                KnowledgeEntry::new(id, format!("content {id}"))
                    .with_metadata(json!({"tenant":if id=="d"{"hidden"}else{"visible"}})),
                vector(v),
            )
            .unwrap();
    }
    index
}
fn config() -> HnswConfig {
    HnswConfig::default().with_exact_threshold(0)
}
fn signature(result: &[airust::semantic::SemanticResult<'_>]) -> Vec<(String, u64)> {
    result
        .iter()
        .map(|r| (r.entry.id.clone(), r.score.to_bits()))
        .collect()
}
struct Provider {
    space: EmbeddingSpace,
    fail: bool,
    calls: usize,
}
impl EmbeddingProvider for Provider {
    fn space(&self) -> &EmbeddingSpace {
        &self.space
    }
    fn embed(
        &mut self,
        texts: &[&str],
        _: EmbeddingPurpose,
    ) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        self.calls += 1;
        if self.fail {
            return Err(EmbeddingError::Provider("offline".into()));
        }
        Ok(texts.iter().map(|_| vec![1.0, 0.0, 0.0]).collect())
    }
}
#[test]
fn validated_parameters_cannot_reach_upstream_exit_or_bad_beam_paths() {
    for (m, l, e) in [
        (0, 16, 200),
        (1, 16, 200),
        (65, 16, 200),
        (usize::MAX, 16, 200),
        (16, 0, 200),
        (16, 17, 200),
        (16, 16, 32),
        (16, 16, usize::MAX),
    ] {
        assert!(HnswConfig::new(m, l, e).is_err());
    }
    assert!(HnswConfig::new(2, 1, 5).is_ok());
    assert!(AnnSearchOptions::new(1).with_ef_search(0).is_err());
    assert!(AnnSearchOptions::new(1).with_ef_search(usize::MAX).is_err());
}
#[test]
fn complete_small_candidate_windows_preserve_exact_scores_ties_and_thresholds() {
    let ann = HnswIndex::from_semantic(index(), config());
    let query = vector([1.0, 0.0, 0.0]);
    for minimum in [-1.0, 0.0, 0.9, 1.0] {
        let semantic = SemanticSearchOptions::new(4)
            .with_min_score(minimum)
            .unwrap();
        let exact = ann.semantic().search_embedding(&query, &semantic).unwrap();
        let options = AnnSearchOptions::new(4).with_semantic_options(semantic);
        let approximate = ann.search_embedding(&query, &options).unwrap();
        assert_eq!(approximate.mode(), AnnSearchMode::Hnsw);
        assert_eq!(signature(approximate.results()), signature(&exact));
    }
    assert_eq!(ann.indexed_points(), 4);
    assert_eq!(ann.pending_changes(), 0);
}
#[test]
fn scoped_queries_use_exact_authorized_candidates_even_when_unscoped_top_k_is_hidden() {
    let ann = HnswIndex::from_semantic(index(), config());
    let options = AnnSearchOptions::new(1)
        .with_ef_search(1)
        .unwrap()
        .with_semantic_options(
            SemanticSearchOptions::new(1)
                .with_context(Context::new().with_metadata("tenant", "visible")),
        );
    let result = ann
        .search_embedding(&vector([-1.0, 0.0, 0.0]), &options)
        .unwrap();
    assert_eq!(result.mode(), AnnSearchMode::ExactScope);
    assert_eq!(result.results()[0].entry.id, "c");
    assert_eq!(result.candidates_scored(), 3);
}
#[test]
fn insert_replace_delete_reinsert_and_rebuild_never_return_stale_vectors() {
    let mut ann = HnswIndex::from_semantic(index(), config());
    let query = vector([1.0, 0.0, 0.0]);
    let options = AnnSearchOptions::new(10);
    for step in 0..150 {
        let id = format!("item-{}", step * 17 % 29);
        if step % 3 == 0 {
            ann.remove(&id);
        } else {
            ann.upsert_embedded(
                KnowledgeEntry::new(&id, format!("revision {step}")),
                vector([1.0, (step % 11) as f32, 0.5]),
            )
            .unwrap();
        }
        if step % 7 == 0 {
            ann.remove("a");
        }
        if step % 11 == 0 {
            ann.upsert_embedded(KnowledgeEntry::new("a", "new a"), vector([0.0, 0.0, 1.0]))
                .unwrap();
        }
        let actual = ann.search_embedding(&query, &options).unwrap();
        let expected = ann
            .semantic()
            .search_embedding(&query, options.semantic_options())
            .unwrap();
        assert_eq!(signature(actual.results()), signature(&expected));
        if step % 19 == 0 {
            ann.rebuild();
            assert_eq!(ann.pending_changes(), 0);
            assert_eq!(ann.indexed_points(), ann.len());
        }
    }
    ann.clear();
    assert!(ann.is_empty());
    assert_eq!(ann.indexed_points(), 0);
}
#[test]
fn stale_policy_large_windows_and_small_corpora_report_exact_fallbacks() {
    let mut ann = HnswIndex::from_semantic(index(), config().with_max_pending(0));
    let query = vector([1.0, 0.0, 0.0]);
    let wide = AnnSearchOptions::new(4).with_ef_search(1).unwrap();
    assert_eq!(
        ann.search_embedding(&query, &wide).unwrap().mode(),
        AnnSearchMode::ExactResultWindow
    );
    ann.remove("a");
    assert!(ann.needs_rebuild());
    assert_eq!(
        ann.search_embedding(&query, &AnnSearchOptions::new(1))
            .unwrap()
            .mode(),
        AnnSearchMode::ExactPendingChanges
    );
    ann.rebuild();
    assert!(!ann.needs_rebuild());
    let small = HnswIndex::from_semantic(ann.into_semantic(), HnswConfig::default());
    assert_eq!(
        small
            .search_embedding(&query, &AnnSearchOptions::new(1))
            .unwrap()
            .mode(),
        AnnSearchMode::ExactSmallIndex
    );
}
#[test]
fn invalid_updates_and_failed_batches_leave_overlay_and_source_state_unchanged() {
    let mut ann = HnswIndex::from_semantic(index(), config());
    let before = ann.semantic().lexical().to_json().unwrap();
    let wrong = Embedding::new(
        EmbeddingSpace::new("wrong", 3).unwrap(),
        vec![1.0, 0.0, 0.0],
    )
    .unwrap();
    assert!(ann
        .upsert_embedded(KnowledgeEntry::new("a", "replace"), wrong.clone())
        .is_err());
    assert!(ann
        .search_embedding(&wrong, &AnnSearchOptions::new(1))
        .is_err());
    let mut provider = Provider {
        space: space(),
        fail: true,
        calls: 0,
    };
    assert!(ann
        .upsert_batch(vec![KnowledgeEntry::new("x", "new")], &mut provider, 1)
        .is_err());
    assert_eq!(ann.pending_changes(), 0);
    assert_eq!(ann.semantic().lexical().to_json().unwrap(), before);
    provider.fail = false;
    assert_eq!(
        ann.upsert_batch(vec![KnowledgeEntry::new("x", "new")], &mut provider, 1)
            .unwrap(),
        1
    );
    assert_eq!(ann.pending_changes(), 1);
    assert!(ann
        .upsert(KnowledgeEntry::new("", "invalid"), &mut provider)
        .is_err());
    assert_eq!(ann.pending_changes(), 1);
}
#[test]
fn empty_queries_and_zero_limits_have_explicit_provider_behavior() {
    let ann = HnswIndex::from_semantic(SemanticIndex::new(space()), config());
    let mut provider = Provider {
        space: space(),
        fail: true,
        calls: 0,
    };
    assert!(ann
        .search(" ", &mut provider, &AnnSearchOptions::new(1))
        .is_err());
    assert_eq!(
        ann.search("query", &mut provider, &AnnSearchOptions::new(1))
            .unwrap()
            .mode(),
        AnnSearchMode::Empty
    );
    let ann = HnswIndex::from_semantic(index(), config());
    assert!(ann
        .search("query", &mut provider, &AnnSearchOptions::new(1))
        .is_err());
    assert_eq!(provider.calls, 1);
    assert_eq!(
        ann.search("query", &mut provider, &AnnSearchOptions::new(0))
            .unwrap()
            .mode(),
        AnnSearchMode::Empty
    );
    assert_eq!(provider.calls, 1);
}
#[test]
fn pinned_serial_builds_and_identical_vectors_have_stable_small_corpus_ordering() {
    let mut source = index();
    for id in ["tie-z", "tie-a"] {
        source
            .upsert_embedded(
                KnowledgeEntry::new(id, "same vector"),
                vector([1.0, 0.0, 0.0]),
            )
            .unwrap();
    }
    let first = HnswIndex::from_semantic(source.clone(), config());
    let second = HnswIndex::from_semantic(source, config());
    let query = vector([1.0, 0.0, 0.0]);
    let options = AnnSearchOptions::new(6);
    let a = first.search_embedding(&query, &options).unwrap();
    let b = second.search_embedding(&query, &options).unwrap();
    assert_eq!(signature(a.results()), signature(b.results()));
    assert_eq!(
        a.results()
            .iter()
            .take(3)
            .map(|r| r.entry.id.as_str())
            .collect::<Vec<_>>(),
        ["a", "tie-a", "tie-z"]
    );
}

#[test]
fn narrow_large_index_search_includes_new_best_and_excludes_replaced_deleted_base_points() {
    let mut semantic = SemanticIndex::new(space());
    for n in 0..700 {
        semantic
            .upsert_embedded(
                KnowledgeEntry::new(format!("base-{n:04}"), format!("record {n}")),
                vector([1.0, n as f32 / 700.0 + 0.1, 0.2]),
            )
            .unwrap();
    }
    let mut ann = HnswIndex::from_semantic(semantic, config());
    let options = AnnSearchOptions::new(3).with_ef_search(10).unwrap();
    let query = vector([1.0, 0.0, 0.0]);
    ann.upsert_embedded(
        KnowledgeEntry::new("new-best", "live overlay"),
        query.clone(),
    )
    .unwrap();
    ann.upsert_embedded(
        KnowledgeEntry::new("base-0000", "opposite replacement"),
        vector([-1.0, 0.0, 0.0]),
    )
    .unwrap();
    for n in 1..50 {
        ann.remove(&format!("base-{n:04}"));
    }
    for compact in [false, true] {
        if compact {
            ann.rebuild();
        }
        let actual = ann.search_embedding(&query, &options).unwrap();
        assert_eq!(actual.mode(), AnnSearchMode::Hnsw);
        assert_eq!(actual.results()[0].entry.id, "new-best");
        assert!(actual
            .results()
            .iter()
            .all(|r| r.entry.id == "new-best" || r.entry.id.as_str() >= "base-0050"));
        for hit in actual.results() {
            let exact = airust::semantic::cosine_similarity(
                query.values(),
                ann.semantic().embedding(&hit.entry.id).unwrap(),
            )
            .unwrap();
            assert_eq!(exact.to_bits(), hit.score.to_bits());
        }
    }
}
