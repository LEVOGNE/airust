#![cfg(feature = "hybrid")]

use airust::{
    hybrid::{HybridSearchOptions, SemanticFailurePolicy},
    semantic::{
        Embedding, EmbeddingError, EmbeddingProvider, EmbeddingPurpose, EmbeddingSpace,
        SemanticIndex,
    },
    Bm25Config, Context, KnowledgeEntry, SearchAlgorithm, SearchOptions,
};
use serde_json::json;

fn space() -> EmbeddingSpace {
    EmbeddingSpace::new("hybrid-fixture-v1", 2).unwrap()
}

struct Provider {
    space: EmbeddingSpace,
    calls: usize,
    mode: u8,
}
impl Provider {
    fn new() -> Self {
        Self {
            space: space(),
            calls: 0,
            mode: 0,
        }
    }
}
impl EmbeddingProvider for Provider {
    fn space(&self) -> &EmbeddingSpace {
        &self.space
    }
    fn embed(
        &mut self,
        texts: &[&str],
        purpose: EmbeddingPurpose,
    ) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        self.calls += 1;
        assert_eq!(purpose, EmbeddingPurpose::Query);
        assert_eq!(texts.len(), 1);
        match self.mode {
            1 => Err(EmbeddingError::Provider("offline".into())),
            2 => Ok(vec![vec![1.0]]),
            3 => {
                self.space = EmbeddingSpace::new("changed", 2)?;
                Ok(vec![vec![1.0, 0.0]])
            }
            4 => Ok(Vec::new()),
            5 => Ok(vec![vec![f32::NAN, 0.0]]),
            _ => Ok(vec![vec![1.0, 0.0]]),
        }
    }
}

fn index() -> SemanticIndex {
    let mut index = SemanticIndex::new(space());
    for (id, text, values, scope) in [
        ("a", "Rust memory safety", vec![0.0, 1.0], "allowed"),
        ("b", "Python scripts", vec![1.0, 0.0], "denied"),
        ("c", "Rust search", vec![0.8, 0.6], "allowed"),
    ] {
        index
            .upsert_embedded(
                KnowledgeEntry::new(id, text).with_metadata(json!({"scope":scope})),
                Embedding::new(space(), values).unwrap(),
            )
            .unwrap();
    }
    index
}

#[test]
fn fusion_matches_hand_calculated_component_ranks_and_deduplicates_ids() {
    let index = index();
    let mut provider = Provider::new();
    let outcome = index
        .hybrid_search("Rust", &mut provider, &HybridSearchOptions::new(3))
        .unwrap();
    assert!(outcome.semantic_used());
    assert!(outcome.fallback_reason().is_none());
    let hits = outcome.results();
    assert_eq!(
        hits.iter().map(|h| h.entry.id.as_str()).collect::<Vec<_>>(),
        ["c", "a", "b"]
    );
    let c = &hits[0];
    assert_eq!(c.lexical_rank, Some(1));
    assert_eq!(c.semantic_rank, Some(2));
    assert_eq!(c.score, 0.5 / 61.0 + 0.5 / 62.0);
    assert_eq!(hits[1].score, 0.5 / 62.0 + 0.5 / 63.0);
    assert_eq!(hits[2].score, 0.5 / 61.0);
    assert_eq!(c.matched_terms, ["rust"]);
    assert!(std::ptr::eq(c.entry, index.lexical().get("c").unwrap()));
    assert_eq!(provider.calls, 1);
}

#[test]
fn weights_and_smoothing_can_change_the_top_result() {
    let index = index();
    let mut provider = Provider::new();
    let options = HybridSearchOptions::new(3)
        .with_weights(1.0, 3.0)
        .unwrap()
        .with_rrf_k(0.0)
        .unwrap();
    let hits = index
        .hybrid_search("Rust", &mut provider, &options)
        .unwrap()
        .into_results();
    assert_eq!(
        hits.iter().map(|h| h.entry.id.as_str()).collect::<Vec<_>>(),
        ["b", "c", "a"]
    );
    assert_eq!(hits[0].score, 0.75);
    assert_eq!(hits[1].score, 0.25 + 0.75 / 2.0);
    assert_eq!(options.lexical_weight(), 0.25);
    assert_eq!(options.semantic_weight(), 0.75);
}

#[test]
fn filters_apply_before_each_component_and_before_result_limit() {
    let index = index();
    let mut provider = Provider::new();
    let options =
        HybridSearchOptions::new(1).with_context(Context::new().with_metadata("scope", "allowed"));
    let outcome = index
        .hybrid_search("Rust", &mut provider, &options)
        .unwrap();
    assert_eq!(outcome.results()[0].entry.id, "c");
    assert_eq!(outcome.results()[0].semantic_rank, Some(1));
    assert_eq!(outcome.results()[0].lexical_rank, Some(1));
    let options = options.with_context(Context::new().with_entry("missing"));
    assert!(index
        .hybrid_search("Rust", &mut provider, &options)
        .unwrap()
        .results()
        .is_empty());
}

#[test]
fn zero_weights_skip_components_and_semantic_only_can_find_no_overlap() {
    let index = index();
    let mut provider = Provider::new();
    provider.mode = 1;
    let lexical = HybridSearchOptions::new(3).with_weights(1.0, 0.0).unwrap();
    let outcome = index
        .hybrid_search("Rust", &mut provider, &lexical)
        .unwrap();
    assert_eq!(provider.calls, 0);
    assert!(!outcome.semantic_used());
    assert_eq!(outcome.results().len(), 2);
    assert!(outcome.results().iter().all(|h| h.semantic_rank.is_none()));
    provider.mode = 0;
    let semantic = HybridSearchOptions::new(1).with_weights(0.0, 1.0).unwrap();
    let outcome = index
        .hybrid_search("Rust", &mut provider, &semantic)
        .unwrap();
    assert_eq!(outcome.results()[0].entry.id, "b");
    assert!(outcome.results()[0].lexical_rank.is_none());
}

#[test]
fn provider_fallback_is_explicit_and_can_recover_a_semantic_only_configuration() {
    let index = index();
    let mut provider = Provider::new();
    provider.mode = 1;
    assert!(index
        .hybrid_search("Rust", &mut provider, &HybridSearchOptions::new(3))
        .is_err());
    let options = HybridSearchOptions::new(3)
        .with_weights(0.0, 1.0)
        .unwrap()
        .with_failure_policy(SemanticFailurePolicy::LexicalOnly);
    let outcome = index
        .hybrid_search("Rust", &mut provider, &options)
        .unwrap();
    assert!(!outcome.semantic_used());
    assert!(outcome.fallback_reason().unwrap().contains("offline"));
    assert_eq!(outcome.results()[0].entry.id, "c");
    assert_eq!(outcome.results()[0].score, 1.0 / 61.0);
    assert!(outcome.results().iter().all(|h| h.semantic_score.is_none()));
    let value = serde_json::to_value(&outcome).unwrap();
    assert_eq!(value["semantic_used"], false);
    assert!(value["fallback_reason"].is_string());
}

#[test]
fn integrity_failures_never_become_a_provider_fallback() {
    let index = index();
    for mode in 2..=5 {
        let mut provider = Provider::new();
        provider.mode = mode;
        let options =
            HybridSearchOptions::new(3).with_failure_policy(SemanticFailurePolicy::LexicalOnly);
        assert!(index
            .hybrid_search("Rust", &mut provider, &options)
            .is_err());
    }
    let mut provider = Provider::new();
    provider.space = EmbeddingSpace::new("wrong-model", 2).unwrap();
    let options =
        HybridSearchOptions::new(3).with_failure_policy(SemanticFailurePolicy::LexicalOnly);
    assert!(index
        .hybrid_search("Rust", &mut provider, &options)
        .is_err());
    assert_eq!(provider.calls, 0);
}

#[test]
fn configuration_rejects_nonfinite_or_incoherent_settings_and_handles_extreme_weights() {
    for (a, b) in [
        (0.0, 0.0),
        (-1.0, 1.0),
        (1.0, f64::NAN),
        (f64::INFINITY, 1.0),
    ] {
        assert!(HybridSearchOptions::default().with_weights(a, b).is_err());
    }
    for k in [-1.0, f64::NAN, f64::INFINITY] {
        assert!(HybridSearchOptions::default().with_rrf_k(k).is_err());
    }
    for threshold in [-1.01, 1.01, f64::NAN] {
        assert!(HybridSearchOptions::default()
            .with_min_score(threshold)
            .is_err());
    }
    assert!(HybridSearchOptions::new(4).with_candidates(3).is_err());
    assert!(HybridSearchOptions::new(0).with_candidates(0).is_err());
    for (a, b) in [
        (f64::MAX, f64::MAX),
        (f64::MIN_POSITIVE, f64::MIN_POSITIVE),
        (f64::MAX, f64::MIN_POSITIVE),
    ] {
        let options = HybridSearchOptions::new(3)
            .with_weights(a, b)
            .unwrap()
            .with_rrf_k(f64::MAX)
            .unwrap();
        let mut provider = Provider::new();
        let index = index();
        let outcome = index
            .hybrid_search("Rust", &mut provider, &options)
            .unwrap();
        assert!(outcome
            .results()
            .iter()
            .all(|h| h.score > 0.0 && h.score.is_finite()));
        assert!((options.lexical_weight() + options.semantic_weight() - 1.0).abs() < 1e-15);
    }
}

#[test]
fn zero_limit_empty_index_and_empty_query_do_not_call_provider() {
    let mut provider = Provider::new();
    provider.mode = 1;
    assert!(index()
        .hybrid_search("Rust", &mut provider, &HybridSearchOptions::new(0))
        .unwrap()
        .results()
        .is_empty());
    assert!(SemanticIndex::new(space())
        .hybrid_search("Rust", &mut provider, &HybridSearchOptions::new(10))
        .unwrap()
        .results()
        .is_empty());
    assert!(index()
        .hybrid_search("\t ", &mut provider, &HybridSearchOptions::default())
        .is_err());
    assert_eq!(provider.calls, 0);
}

#[test]
fn tfidf_and_bm25_configuration_are_preserved_in_component_evidence() {
    let index = index();
    let mut provider = Provider::new();
    let bm25 = Bm25Config::new(2.0, 0.3).unwrap();
    let options = HybridSearchOptions::new(3)
        .with_lexical_algorithm(SearchAlgorithm::Tfidf)
        .with_bm25(bm25);
    let reference = index.lexical().search_with_options(
        "Rust",
        &SearchOptions {
            limit: 100,
            algorithm: SearchAlgorithm::Tfidf,
            ..Default::default()
        },
    );
    let outcome = index
        .hybrid_search("Rust", &mut provider, &options)
        .unwrap();
    for hit in outcome
        .results()
        .iter()
        .filter(|h| h.lexical_rank.is_some())
    {
        let reference = reference
            .iter()
            .find(|r| r.entry.id == hit.entry.id)
            .unwrap();
        assert_eq!(hit.lexical_score, Some(reference.score));
    }
    assert_eq!(options.lexical_algorithm(), SearchAlgorithm::Tfidf);
    assert_eq!(options.bm25(), bm25);
}

#[test]
fn equal_fused_scores_have_stable_id_order() {
    let mut index = SemanticIndex::new(space());
    index
        .upsert_embedded(
            KnowledgeEntry::new("a", "Rust Rust"),
            Embedding::new(space(), vec![0.0, 1.0]).unwrap(),
        )
        .unwrap();
    index
        .upsert_embedded(
            KnowledgeEntry::new("z", "Rust"),
            Embedding::new(space(), vec![1.0, 0.0]).unwrap(),
        )
        .unwrap();
    let mut provider = Provider::new();
    let outcome = index
        .hybrid_search("Rust", &mut provider, &HybridSearchOptions::new(2))
        .unwrap();
    assert_eq!(outcome.results()[0].score, outcome.results()[1].score);
    assert_eq!(outcome.results()[0].entry.id, "a");
}

#[test]
fn cosine_cutoff_and_candidate_budget_change_only_the_configured_evidence() {
    let index = index();
    let mut provider = Provider::new();
    let options = HybridSearchOptions::new(3).with_min_score(0.5).unwrap();
    let outcome = index
        .hybrid_search("Rust", &mut provider, &options)
        .unwrap();
    let a = outcome
        .results()
        .iter()
        .find(|h| h.entry.id == "a")
        .unwrap();
    assert!(a.semantic_rank.is_none());
    let options = HybridSearchOptions::new(1).with_candidates(1).unwrap();
    let outcome = index
        .hybrid_search("Rust", &mut provider, &options)
        .unwrap();
    assert_eq!(outcome.results().len(), 1);
    assert!(outcome.results()[0].entry.id == "b" || outcome.results()[0].entry.id == "c");
    assert_eq!(options.candidates(), 1);
}

#[test]
fn large_candidate_sets_are_unique_scoped_and_match_full_reference_prefixes() {
    let mut index = SemanticIndex::new(space());
    for n in 0..5000 {
        index
            .upsert_embedded(
                KnowledgeEntry::new(format!("{n:06}"), "Rust common")
                    .with_metadata(json!({"allowed":n%3!=0})),
                Embedding::new(space(), vec![1.0, (n % 17) as f32]).unwrap(),
            )
            .unwrap();
    }
    let mut provider = Provider::new();
    let options =
        HybridSearchOptions::new(5000).with_context(Context::new().with_metadata("allowed", true));
    let full = index
        .hybrid_search("Rust", &mut provider, &options)
        .unwrap()
        .into_results();
    assert_eq!(full.len(), 3333);
    let ids: std::collections::BTreeSet<_> = full.iter().map(|h| h.entry.id.as_str()).collect();
    assert_eq!(ids.len(), full.len());
    for limit in [1, 10, 100] {
        let options = HybridSearchOptions::new(limit)
            .with_candidates(5000)
            .unwrap()
            .with_context(Context::new().with_metadata("allowed", true));
        let actual = index
            .hybrid_search("Rust", &mut provider, &options)
            .unwrap();
        assert_eq!(
            actual
                .results()
                .iter()
                .map(|h| (&h.entry.id, h.score))
                .collect::<Vec<_>>(),
            full.iter()
                .take(limit)
                .map(|h| (&h.entry.id, h.score))
                .collect::<Vec<_>>()
        );
    }
}
