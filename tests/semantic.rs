#![cfg(feature = "embeddings")]

use airust::{
    semantic::{
        cosine_similarity, embed_batch, Embedding, EmbeddingError, EmbeddingProvider,
        EmbeddingPurpose, EmbeddingSpace, SemanticIndex, SemanticSearchOptions,
    },
    Context, Engine, KnowledgeEntry,
};
use serde_json::json;

fn space() -> EmbeddingSpace {
    EmbeddingSpace::new("fixture-model:revision1:mean", 3).unwrap()
}
fn vector(values: [f32; 3]) -> Embedding {
    Embedding::new(space(), values.to_vec()).unwrap()
}

struct Provider {
    space: EmbeddingSpace,
    calls: usize,
    batches: Vec<usize>,
    purposes: Vec<EmbeddingPurpose>,
    fail_at: usize,
    wrong_count: bool,
    invalid_at: usize,
    change_at: usize,
}

impl Provider {
    fn new() -> Self {
        Self {
            space: space(),
            calls: 0,
            batches: Vec::new(),
            purposes: Vec::new(),
            fail_at: usize::MAX,
            wrong_count: false,
            invalid_at: usize::MAX,
            change_at: usize::MAX,
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
        self.batches.push(texts.len());
        self.purposes.push(purpose);
        if self.calls == self.fail_at {
            return Err(EmbeddingError::Provider("fixture failure".into()));
        }
        if self.calls == self.change_at {
            self.space = EmbeddingSpace::new("different recipe", 3)?;
        }
        if self.wrong_count {
            return Ok(Vec::new());
        }
        Ok(texts
            .iter()
            .map(|text| {
                if self.calls == self.invalid_at {
                    vec![f32::NAN, 0.0, 1.0]
                } else if text.contains("Rust") {
                    vec![1.0, 0.0, 0.0]
                } else {
                    vec![0.0, 1.0, 0.0]
                }
            })
            .collect())
    }
}

#[test]
fn spaces_reject_missing_identity_and_zero_dimensions() {
    for id in ["", " \t"] {
        assert!(EmbeddingSpace::new(id, 3).is_err());
    }
    assert!(EmbeddingSpace::new("model", 0).is_err());
    assert_eq!(space().dimensions(), 3);
    assert_eq!(space().model(), "fixture-model:revision1:mean");
}

#[test]
fn vectors_validate_dimensions_finite_values_and_nonzero_norm() {
    for values in [
        vec![],
        vec![1.0],
        vec![1.0; 4],
        vec![0.0; 3],
        vec![-0.0; 3],
        vec![f32::NAN, 1.0, 2.0],
        vec![1.0, f32::INFINITY, 2.0],
        vec![1.0, 2.0, f32::NEG_INFINITY],
    ] {
        assert!(Embedding::new(space(), values).is_err());
    }
    let embedding = vector([3.0, 4.0, 0.0]);
    assert!((embedding.values()[0] - 0.6).abs() < 1e-6);
    assert!((embedding.values()[1] - 0.8).abs() < 1e-6);
}

#[test]
fn cosine_is_robust_for_extreme_finite_values() {
    for magnitude in [f32::MAX, f32::MIN_POSITIVE, f32::from_bits(1)] {
        let a = [magnitude, magnitude, 0.0];
        assert!((cosine_similarity(&a, &a).unwrap() - 1.0).abs() < 1e-12);
        assert!((vector(a).cosine(&vector(a)).unwrap() - 1.0).abs() < 1e-12);
    }
    assert_eq!(cosine_similarity(&[1.0, 0.0], &[0.0, 1.0]).unwrap(), 0.0);
    assert_eq!(cosine_similarity(&[1.0, 0.0], &[-1.0, 0.0]).unwrap(), -1.0);
    assert!(cosine_similarity(&[], &[]).is_err());
    assert!(cosine_similarity(&[1.0], &[1.0, 2.0]).is_err());
}

#[test]
fn equal_dimensions_do_not_make_different_model_recipes_compatible() {
    let a = vector([1.0, 0.0, 0.0]);
    let b = Embedding::new(
        EmbeddingSpace::new("another-model", 3).unwrap(),
        vec![1.0, 0.0, 0.0],
    )
    .unwrap();
    assert!(matches!(
        a.cosine(&b),
        Err(EmbeddingError::SpaceMismatch { .. })
    ));
    let mut index = SemanticIndex::new(space());
    assert!(index
        .upsert_embedded(KnowledgeEntry::new("a", "text"), b.clone())
        .is_err());
    assert!(index.is_empty());
    assert!(index
        .search_embedding(&b, &SemanticSearchOptions::default())
        .is_err());
}

#[test]
fn batch_boundary_checks_empty_inputs_output_count_and_model_changes() {
    let mut provider = Provider::new();
    assert!(embed_batch(&mut provider, &[], EmbeddingPurpose::Document)
        .unwrap()
        .is_empty());
    assert!(embed_batch(&mut provider, &["\t"], EmbeddingPurpose::Query).is_err());
    assert_eq!(provider.calls, 0);
    provider.wrong_count = true;
    assert!(matches!(
        embed_batch(&mut provider, &["Rust"], EmbeddingPurpose::Query),
        Err(EmbeddingError::BatchCount { .. })
    ));
    provider.wrong_count = false;
    provider.change_at = 2;
    assert!(matches!(
        embed_batch(&mut provider, &["Rust"], EmbeddingPurpose::Query),
        Err(EmbeddingError::SpaceMismatch { .. })
    ));
}

#[test]
fn provider_preserves_input_text_and_query_document_purposes() {
    struct ExactProvider(EmbeddingSpace);
    impl EmbeddingProvider for ExactProvider {
        fn space(&self) -> &EmbeddingSpace {
            &self.0
        }
        fn embed(
            &mut self,
            texts: &[&str],
            purpose: EmbeddingPurpose,
        ) -> Result<Vec<Vec<f32>>, EmbeddingError> {
            assert_eq!(texts, ["  Straße CAFÉ हिंदी 中文  "]);
            assert_eq!(purpose, EmbeddingPurpose::Query);
            Ok(vec![vec![1.0, 0.0, 0.0]])
        }
    }
    let mut provider = ExactProvider(space());
    assert_eq!(
        embed_batch(
            &mut provider,
            &["  Straße CAFÉ हिंदी 中文  "],
            EmbeddingPurpose::Query
        )
        .unwrap()
        .len(),
        1
    );
}

#[test]
fn semantic_index_reuses_entries_metadata_and_lexical_updates() {
    let mut provider = Provider::new();
    let mut index = SemanticIndex::new(space());
    index
        .upsert(
            KnowledgeEntry::new("a", "Rust record").with_metadata(json!({"scope":"private"})),
            &mut provider,
        )
        .unwrap();
    index
        .upsert(KnowledgeEntry::new("b", "Python record"), &mut provider)
        .unwrap();
    let options = SemanticSearchOptions::new(2).with_min_score(0.5).unwrap();
    let hits = index.search("Rust", &mut provider, &options).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].entry.id, "a");
    assert_eq!(hits[0].entry.metadata.as_ref().unwrap()["scope"], "private");
    assert!(std::ptr::eq(
        hits[0].entry,
        index.lexical().get("a").unwrap()
    ));
    assert_eq!(
        provider.purposes,
        [
            EmbeddingPurpose::Document,
            EmbeddingPurpose::Document,
            EmbeddingPurpose::Query
        ]
    );
    let old = index
        .upsert(KnowledgeEntry::new("a", "Python changed"), &mut provider)
        .unwrap()
        .unwrap();
    assert_eq!(old.text, "Rust record");
    assert!(index.lexical().search("Rust", 10).is_empty());
    assert!(index
        .search("Rust", &mut provider, &options)
        .unwrap()
        .is_empty());
    assert!(index.remove("a").is_some());
    assert!(index.embedding("a").is_none());
    assert!(index.remove("missing").is_none());
    index.clear();
    assert!(index.is_empty());
    assert_eq!(index.lexical().stats().tokens, 0);
}

#[test]
fn context_and_id_ties_apply_before_top_k() {
    let mut index = SemanticIndex::new(space());
    for (id, scope) in [("z", "allowed"), ("a", "denied"), ("b", "allowed")] {
        index
            .upsert_embedded(
                KnowledgeEntry::new(id, "same").with_metadata(json!({"scope":scope})),
                vector([1.0, 0.0, 0.0]),
            )
            .unwrap();
    }
    let query = vector([1.0, 0.0, 0.0]);
    assert_eq!(
        index
            .search_embedding(&query, &SemanticSearchOptions::new(1))
            .unwrap()[0]
            .entry
            .id,
        "a"
    );
    let options = SemanticSearchOptions::new(1)
        .with_context(Context::new().with_metadata("scope", "allowed"));
    assert_eq!(
        index.search_embedding(&query, &options).unwrap()[0]
            .entry
            .id,
        "b"
    );
    let options = options.with_context(Context::new().with_entry("absent"));
    assert!(index.search_embedding(&query, &options).unwrap().is_empty());
}

#[test]
fn provider_failures_and_invalid_replacements_preserve_previous_state() {
    let mut provider = Provider::new();
    let mut index = SemanticIndex::new(space());
    index
        .upsert(KnowledgeEntry::new("a", "Rust original"), &mut provider)
        .unwrap();
    let before = index.lexical().to_json().unwrap();
    let old_vector = index.embedding("a").unwrap().to_vec();
    provider.fail_at = 2;
    assert!(index
        .upsert(KnowledgeEntry::new("a", "replacement"), &mut provider)
        .is_err());
    assert!(index
        .upsert(KnowledgeEntry::new("a", " \n"), &mut provider)
        .is_err());
    assert_eq!(provider.calls, 2);
    assert_eq!(index.lexical().to_json().unwrap(), before);
    assert_eq!(index.embedding("a").unwrap(), old_vector);
}

#[test]
fn batches_are_atomic_across_provider_chunks_and_validate_before_inference() {
    let mut index = SemanticIndex::new(space());
    index
        .upsert_embedded(
            KnowledgeEntry::new("old", "original"),
            vector([1.0, 0.0, 0.0]),
        )
        .unwrap();
    let before = index.lexical().to_json().unwrap();
    for failure in [0, 1, 2] {
        let mut provider = Provider::new();
        match failure {
            0 => provider.fail_at = 2,
            1 => provider.invalid_at = 2,
            _ => provider.change_at = 2,
        }
        let entries = (0..5)
            .map(|n| KnowledgeEntry::new(n.to_string(), "Rust"))
            .collect();
        assert!(index.upsert_batch(entries, &mut provider, 2).is_err());
        assert_eq!(provider.batches, [2, 2]);
        assert_eq!(index.lexical().to_json().unwrap(), before);
        assert!(index.embedding("0").is_none());
    }
    let mut provider = Provider::new();
    assert!(index
        .upsert_batch(
            vec![
                KnowledgeEntry::new("x", "Rust"),
                KnowledgeEntry::new("x", "Python")
            ],
            &mut provider,
            2
        )
        .is_err());
    assert!(index
        .upsert_batch(
            vec![
                KnowledgeEntry::new("x", "Rust"),
                KnowledgeEntry::new("y", " ")
            ],
            &mut provider,
            2
        )
        .is_err());
    assert!(index.upsert_batch(Vec::new(), &mut provider, 0).is_err());
    assert_eq!(provider.calls, 0);
    let entries = (0..5)
        .map(|n| KnowledgeEntry::new(n.to_string(), "Rust"))
        .collect();
    assert_eq!(index.upsert_batch(entries, &mut provider, 2).unwrap(), 5);
    assert_eq!(provider.batches, [2, 2, 1]);
    assert_eq!(index.len(), 6);
}

#[test]
fn converting_an_existing_index_preserves_lexical_scores_and_uses_batches() {
    let engine =
        Engine::from_entries((0..7).map(|n| KnowledgeEntry::new(n.to_string(), "Rust text")))
            .unwrap();
    let expected: Vec<_> = engine
        .search("Rust", 10)
        .iter()
        .map(|hit| hit.score)
        .collect();
    let mut provider = Provider::new();
    let index = SemanticIndex::from_index(engine, &mut provider, 3).unwrap();
    assert_eq!(provider.batches, [3, 3, 1]);
    assert_eq!(
        index
            .lexical()
            .search("Rust", 10)
            .iter()
            .map(|hit| hit.score)
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(index.len(), 7);
    assert!(SemanticIndex::from_index(Engine::new(), &mut provider, 0).is_err());
}

#[test]
fn exact_ranking_matches_full_reference_for_all_limits() {
    let space = EmbeddingSpace::new("generated-reference-v1", 7).unwrap();
    let mut index = SemanticIndex::new(space.clone());
    for n in 0..143 {
        let values = (0..7)
            .map(|d| ((n * 29 + d * 17) % 71) as f32 - 35.0)
            .collect();
        index
            .upsert_embedded(
                KnowledgeEntry::new(format!("{n:04}"), "text"),
                Embedding::new(space.clone(), values).unwrap(),
            )
            .unwrap();
    }
    let query = Embedding::new(space, vec![1.0, -2.0, 3.0, -4.0, 5.0, -6.0, 7.0]).unwrap();
    let mut reference: Vec<_> = index
        .lexical()
        .entries()
        .map(|entry| {
            (
                entry.id.clone(),
                cosine_similarity(query.values(), index.embedding(&entry.id).unwrap()).unwrap(),
            )
        })
        .collect();
    reference.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    for limit in [0, 1, 3, 20, 143, 200, usize::MAX] {
        let found = index
            .search_embedding(&query, &SemanticSearchOptions::new(limit))
            .unwrap();
        let actual: Vec<_> = found
            .iter()
            .map(|hit| (hit.entry.id.clone(), hit.score))
            .collect();
        assert_eq!(
            actual,
            reference.iter().take(limit).cloned().collect::<Vec<_>>()
        );
    }
}

#[test]
fn thresholds_empty_queries_and_zero_limits_are_explicit() {
    for score in [f64::NAN, f64::INFINITY, -1.01, 1.01] {
        assert!(SemanticSearchOptions::new(1).with_min_score(score).is_err());
    }
    let mut provider = Provider::new();
    let index = SemanticIndex::new(space());
    assert!(index
        .search(" ", &mut provider, &SemanticSearchOptions::default())
        .is_err());
    assert!(index
        .search("Rust", &mut provider, &SemanticSearchOptions::default())
        .unwrap()
        .is_empty());
    assert_eq!(provider.calls, 0);
    assert_eq!(SemanticSearchOptions::default().min_score(), -1.0);
    assert_eq!(SemanticSearchOptions::default().limit(), 10);
    assert!(SemanticSearchOptions::default().context().is_none());
}

#[cfg(feature = "local-embeddings")]
#[test]
fn local_provider_requires_explicit_files_and_positive_threads() {
    use airust::semantic::local::FastEmbedProvider;
    let directory = tempfile::tempdir().unwrap();
    assert!(FastEmbedProvider::multilingual_e5_small(
        directory.path(),
        directory.path().join("missing"),
        1
    )
    .is_err());
    assert!(FastEmbedProvider::multilingual_e5_small(
        directory.path(),
        directory.path().join("missing"),
        0
    )
    .is_err());
}
