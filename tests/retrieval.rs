#![cfg(feature = "retrieval")]
use airust::{retrieval::*, Context, Engine, KnowledgeEntry, SearchOptions};
use serde_json::json;
fn engine() -> Engine {
    Engine::from_entries(vec![
        KnowledgeEntry::new("a", "Rust memory safety 🦀")
            .with_metadata(json!({"tenant":"one","secret":"private metadata"})),
        KnowledgeEntry::new("b", "Rust memory Rust memory").with_metadata(json!({"tenant":"two"})),
        KnowledgeEntry::new("c", "unrelated topic"),
    ])
    .unwrap()
}
#[test]
fn lexical_evidence_agrees_with_original_ranking_and_response_owns_only_selected_data() {
    let response = {
        let index = engine();
        let expected = index.search("Rust memory", 2);
        let response = Retriever::new(LexicalSource::new(&index))
            .query("Rust memory", &RetrievalOptions::new(2))
            .unwrap();
        for (a, b) in response.hits().iter().zip(expected) {
            assert_eq!(a.id, b.entry.id);
            assert_eq!(a.evidence.score.to_bits(), b.score.to_bits());
            assert_eq!(a.evidence.matched_terms, b.matched_terms);
            assert_eq!(a.metadata_status, MetadataStatus::Omitted);
        }
        response
    };
    assert_eq!(response.hits().len(), 2);
    assert!(!response.to_json().contains("private metadata"));
    assert_eq!(
        serde_json::to_string(&response).unwrap(),
        response.to_json()
    );
    assert_eq!(response.schema_version(), 1);
}
#[test]
fn scope_is_intersected_before_top_k_and_empty_intersection_remains_denied() {
    let index = engine();
    let source = LexicalSource::new(&index).with_options(SearchOptions {
        context: Some(Context::new().with_entry("a")),
        ..SearchOptions::default()
    });
    let mut r = Retriever::new(source);
    let options =
        RetrievalOptions::new(1).with_context(Context::new().with_metadata("tenant", "one"));
    assert_eq!(r.query("Rust", &options).unwrap().hits()[0].id, "a");
    assert!(r
        .query(
            "Rust",
            &options.with_context(Context::new().with_entry("b"))
        )
        .unwrap()
        .hits()
        .is_empty());
}
#[test]
fn utf8_excerpts_and_total_text_bytes_are_exact_and_metadata_is_explicit() {
    let index = Engine::from_entries(vec![
        KnowledgeEntry::new("x", "🦀éabc").with_metadata(json!({"a":"\n\n\n"})),
        KnowledgeEntry::new("y", "🦀éabc"),
    ])
    .unwrap();
    let mut r = Retriever::new(LexicalSource::new(&index));
    let opts = RetrievalOptions::new(2)
        .with_byte_budgets(100, 7, 5, 10000)
        .unwrap()
        .with_metadata(100)
        .unwrap();
    let response = r.query("éabc", &opts).unwrap();
    assert_eq!(response.hits()[0].excerpt, "🦀");
    assert_eq!(response.hits()[1].excerpt, "");
    assert!(response.truncated());
    for h in response.hits() {
        assert_eq!(h.span.start, 0);
        assert_eq!(h.span.end, h.excerpt.len());
    }
    assert_eq!(response.hits()[0].metadata_status, MetadataStatus::Included);
    assert_eq!(response.hits()[1].metadata_status, MetadataStatus::Missing);
    let small = r.query("éabc", &opts.with_metadata(5).unwrap()).unwrap();
    assert_eq!(
        small.hits()[0].metadata_status,
        MetadataStatus::BudgetExceeded
    );
}
#[derive(Clone, Copy)]
enum Mode {
    Valid,
    Duplicate,
    Nan,
    OutOfScope,
    Span,
    TooMany,
    HugeEvidence,
    HugeRecipe,
}
struct Source {
    entries: Vec<KnowledgeEntry>,
    mode: Mode,
    calls: usize,
}
impl Source {
    fn new(mode: Mode) -> Self {
        Self {
            entries: vec![
                KnowledgeEntry::new("a", "original text 🦀")
                    .with_metadata(json!({"private":"\n".repeat(100)})),
                KnowledgeEntry::new("b", "original other text"),
            ],
            mode,
            calls: 0,
        }
    }
}
impl RetrievalSource for Source {
    fn retrieve<'a>(
        &'a mut self,
        _: &str,
        o: &RetrievalOptions,
    ) -> Result<RetrievalBatch<'a>, RetrievalError> {
        self.calls += 1;
        let mut hits: Vec<_> = self
            .entries
            .iter()
            .map(|entry| RetrievalCandidate {
                entry,
                evidence: RankingEvidence::new(RankingKind::Custom, 1.0),
            })
            .collect();
        match self.mode {
            Mode::Valid => {}
            Mode::Duplicate => hits[1].entry = hits[0].entry,
            Mode::Nan => hits[0].evidence.score = f64::NAN,
            Mode::OutOfScope => {}
            Mode::Span => hits[0]
                .evidence
                .phrase_spans
                .push(ExcerptSpan { start: 15, end: 16 }),
            Mode::TooMany => {
                for _ in 0..o.candidates() {
                    hits.push(RetrievalCandidate {
                        entry: &self.entries[0],
                        evidence: RankingEvidence::new(RankingKind::Custom, 1.0),
                    });
                }
            }
            Mode::HugeEvidence => hits[0].evidence.matched_terms.push("x".repeat(10000)),
            Mode::HugeRecipe => {}
        }
        Ok(RetrievalBatch::new(
            if matches!(self.mode, Mode::HugeRecipe) {
                "x".repeat(1025)
            } else {
                "custom-v1".into()
            },
            hits,
        ))
    }
}
#[test]
fn malformed_custom_sources_are_rejected_before_output() {
    for mode in [
        Mode::Duplicate,
        Mode::Nan,
        Mode::OutOfScope,
        Mode::Span,
        Mode::TooMany,
        Mode::HugeEvidence,
        Mode::HugeRecipe,
    ] {
        let mut r = Retriever::new(Source::new(mode));
        let mut opts = RetrievalOptions::new(2)
            .with_evidence_limits(32, 1024)
            .unwrap();
        if matches!(mode, Mode::OutOfScope) {
            opts = opts.with_context(Context::new().with_entry("a"));
        }
        assert!(r.query("query", &opts).is_err());
    }
}
#[test]
fn output_budget_counts_envelope_ids_escapes_and_evidence_without_partial_json() {
    for bytes in (1..4096).step_by(17) {
        let mut r = Retriever::new(Source::new(Mode::Valid));
        let options = RetrievalOptions::new(2)
            .with_byte_budgets(100, 100, 100, bytes)
            .unwrap()
            .with_metadata(4096)
            .unwrap();
        match r.query("query", &options) {
            Ok(response) => {
                assert!(response.to_json().len() <= bytes);
                let decoded: serde_json::Value = serde_json::from_str(response.to_json()).unwrap();
                assert_eq!(
                    decoded["hits"].as_array().unwrap().len(),
                    response.hits().len()
                );
                assert_eq!(
                    serde_json::to_string(&response).unwrap(),
                    response.to_json()
                );
                if response.hits().len() < 2 {
                    assert!(response.truncated());
                }
            }
            Err(e) => assert!(matches!(e, RetrievalError::Budget("output envelope bytes"))),
        }
    }
}
#[test]
fn zero_limit_skips_source_and_query_limits_fail_before_source() {
    struct Never;
    impl RetrievalSource for Never {
        fn retrieve<'a>(
            &'a mut self,
            _: &str,
            _: &RetrievalOptions,
        ) -> Result<RetrievalBatch<'a>, RetrievalError> {
            panic!("source should not be called")
        }
    }
    let mut r = Retriever::new(Never);
    assert!(r
        .query("valid", &RetrievalOptions::new(0))
        .unwrap()
        .hits()
        .is_empty());
    assert!(r.query(" ", &RetrievalOptions::default()).is_err());
    assert!(r
        .query(
            "long",
            &RetrievalOptions::default()
                .with_byte_budgets(2, 10, 10, 1000)
                .unwrap()
        )
        .is_err());
}
#[test]
fn metadata_depth_is_checked_before_serialization_and_clone() {
    let mut source = Source::new(Mode::Valid);
    let mut value = json!(0);
    for _ in 0..100 {
        value = json!([value]);
    }
    source.entries[0].metadata = Some(value);
    let response = Retriever::new(source)
        .query(
            "query",
            &RetrievalOptions::new(2).with_metadata(100000).unwrap(),
        )
        .unwrap();
    assert_eq!(
        response.hits()[0].metadata_status,
        MetadataStatus::BudgetExceeded
    );
    assert!(response.hits()[0].metadata.is_none());
}
#[test]
fn zero_text_budget_keeps_evidence_without_unbounded_text() {
    let response = Retriever::new(Source::new(Mode::Valid))
        .query(
            "query",
            &RetrievalOptions::new(2)
                .with_byte_budgets(100, 0, 100, 10000)
                .unwrap(),
        )
        .unwrap();
    assert_eq!(response.hits().len(), 2);
    assert!(response
        .hits()
        .iter()
        .all(|h| h.excerpt.is_empty() && h.text_truncated));
}
#[test]
fn invalid_options_and_id_budgets_are_explicit() {
    assert!(RetrievalOptions::new(3).with_candidates(2).is_err());
    assert!(RetrievalOptions::new(0).with_candidates(0).is_err());
    assert!(RetrievalOptions::default()
        .with_byte_budgets(0, 1, 1, 1)
        .is_err());
    assert!(RetrievalOptions::default()
        .with_byte_budgets(1, 1, 0, 1)
        .is_err());
    assert!(RetrievalOptions::default()
        .with_evidence_limits(0, 1)
        .is_err());
    assert!(RetrievalOptions::default().with_metadata(0).is_err());
    let mut source = Source::new(Mode::Valid);
    source.entries[0].id = "long".into();
    assert!(matches!(
        Retriever::new(source).query(
            "query",
            &RetrievalOptions::default()
                .with_evidence_limits(2, 1000)
                .unwrap()
        ),
        Err(RetrievalError::Budget("ID bytes"))
    ));
}
#[test]
fn seeded_utf8_and_json_budget_cases_never_exceed_configured_bytes() {
    let fragments = ["a", "🦀", "e\u{301}", "\"", "\n", "東京", "ß"];
    let mut seed = 7u64;
    for _ in 0..500 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let text = (0..24)
            .map(|i| fragments[((seed >> ((i % 8) * 8)) as usize) % fragments.len()])
            .collect::<String>();
        let original = text.clone();
        let mut source = Source::new(Mode::Valid);
        source.entries.truncate(1);
        source.entries[0].text = text;
        let cap = (seed as usize) % 65;
        let r = Retriever::new(source)
            .query(
                "query",
                &RetrievalOptions::new(1)
                    .with_byte_budgets(100, cap, 31, 4096)
                    .unwrap(),
            )
            .unwrap();
        let h = &r.hits()[0];
        assert!(h.excerpt.len() <= cap.min(31));
        assert_eq!(
            original.get(h.span.start..h.span.end),
            Some(h.excerpt.as_str())
        );
        assert!(r.to_json().len() <= 4096);
    }
}
#[cfg(feature = "advanced-search")]
#[test]
fn text_source_preserves_german_phrases_scope_and_original_spans() {
    use airust::text_search::{AnalyzerConfig, TextIndex};
    let index = TextIndex::from_entries(
        AnalyzerConfig::german(),
        vec![
            KnowledgeEntry::new("a", "Häuser mit roten Türen"),
            KnowledgeEntry::new("b", "Haus mit sehr roten Türen"),
        ],
    )
    .unwrap();
    let response = Retriever::new(TextSource::new(&index))
        .query("\"Haus mit\"", &RetrievalOptions::new(1))
        .unwrap();
    assert_eq!(response.hits()[0].id, "a");
    let span = response.hits()[0].evidence.phrase_spans[0];
    assert_eq!(
        &index.get("a").unwrap().text[span.start..span.end],
        "Häuser mit"
    );
    assert_eq!(response.hits()[0].evidence.kind, RankingKind::TextBm25);
    assert!(Retriever::new(TextSource::new(&index))
        .query("\"unclosed", &RetrievalOptions::default())
        .is_err());
}
#[cfg(feature = "reranking")]
mod rerank_tests {
    use super::*;
    use airust::reranking::{RerankError, RerankFailurePolicy, RerankOptions, Reranker};
    struct Scorer {
        calls: usize,
        fail: bool,
    }
    impl Reranker for Scorer {
        fn recipe(&self) -> &str {
            "test-scorer"
        }
        fn score(&mut self, _: &str, docs: &[&str]) -> Result<Vec<f64>, RerankError> {
            self.calls += 1;
            assert!(docs.iter().all(|d| d.contains("original")));
            if self.fail {
                Err(RerankError::Provider("offline".into()))
            } else {
                Ok(docs.iter().enumerate().map(|(i, _)| i as f64).collect())
            }
        }
    }
    #[test]
    fn reranking_keeps_source_evidence_and_separate_scores() {
        let mut scorer = Scorer {
            calls: 0,
            fail: false,
        };
        let response = Retriever::new(Source::new(Mode::Valid))
            .with_reranker(&mut scorer, RerankOptions::new(2))
            .query("query", &RetrievalOptions::new(2))
            .unwrap();
        assert_eq!(response.hits()[0].id, "b");
        assert_eq!(response.hits()[0].source_rank, 2);
        assert_eq!(response.hits()[0].evidence.score, 1.0);
        assert_eq!(response.hits()[0].rerank_score, Some(1.0));
        assert!(response.reranking().unwrap().used);
        assert_eq!(response.reranking().unwrap().scored, 2);
    }
    #[test]
    fn malformed_or_out_of_scope_source_never_reaches_scorer() {
        for mode in [Mode::OutOfScope, Mode::Nan, Mode::Duplicate] {
            let mut scorer = Scorer {
                calls: 0,
                fail: false,
            };
            let r = Retriever::new(Source::new(mode))
                .with_reranker(&mut scorer, RerankOptions::new(2))
                .query(
                    "query",
                    &RetrievalOptions::new(2).with_context(Context::new().with_entry("a")),
                );
            assert!(r.is_err());
            assert_eq!(scorer.calls, 0);
        }
    }
    #[test]
    fn explicit_fallback_preserves_order_with_no_invented_scores() {
        let mut scorer = Scorer {
            calls: 0,
            fail: true,
        };
        let options =
            RerankOptions::new(2).with_failure_policy(RerankFailurePolicy::KeepOriginalOrder);
        let response = Retriever::new(Source::new(Mode::Valid))
            .with_reranker(&mut scorer, options)
            .query("query", &RetrievalOptions::new(2))
            .unwrap();
        assert_eq!(response.hits()[0].id, "a");
        assert!(response.hits().iter().all(|h| h.rerank_score.is_none()));
        assert!(!response.reranking().unwrap().used);
        assert!(response.reranking().unwrap().fallback_reason.is_some());
    }
    #[test]
    fn provider_errors_remain_errors_by_default() {
        let mut scorer = Scorer {
            calls: 0,
            fail: true,
        };
        assert!(Retriever::new(Source::new(Mode::Valid))
            .with_reranker(&mut scorer, RerankOptions::new(2))
            .query("query", &RetrievalOptions::new(2))
            .is_err());
    }
}
#[cfg(feature = "embeddings")]
mod semantic_tests {
    use super::*;
    use airust::semantic::{
        EmbeddingError, EmbeddingProvider, EmbeddingPurpose, EmbeddingSpace, SemanticIndex,
    };
    struct Provider {
        space: EmbeddingSpace,
        calls: usize,
        fail: bool,
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
                Err(EmbeddingError::Provider("offline".into()))
            } else {
                Ok(texts
                    .iter()
                    .map(|t| {
                        if t.contains("Rust") {
                            vec![1.0, 0.0]
                        } else {
                            vec![0.0, 1.0]
                        }
                    })
                    .collect())
            }
        }
    }
    fn fixture() -> (SemanticIndex, Provider) {
        let mut p = Provider {
            space: EmbeddingSpace::new("handwritten-test-v1", 2).unwrap(),
            calls: 0,
            fail: false,
        };
        let index = SemanticIndex::from_index(engine(), &mut p, 16).unwrap();
        p.calls = 0;
        (index, p)
    }
    #[test]
    fn semantic_source_preserves_cosine_and_access_scope() {
        let (index, mut p) = fixture();
        let r = Retriever::new(SemanticSource::new(&index, &mut p))
            .query(
                "Rust",
                &RetrievalOptions::new(1).with_context(Context::new().with_entry("a")),
            )
            .unwrap();
        assert_eq!(r.hits()[0].id, "a");
        assert_eq!(r.hits()[0].evidence.kind, RankingKind::Cosine);
        assert_eq!(r.hits()[0].evidence.score, 1.0);
    }
    #[test]
    fn deny_all_intersection_does_not_call_embedding_provider() {
        use airust::semantic::SemanticSearchOptions;
        let (index, mut p) = fixture();
        let source = SemanticSource::new(&index, &mut p).with_options(
            SemanticSearchOptions::default().with_context(Context::new().with_entry("a")),
        );
        let r = Retriever::new(source)
            .query(
                "Rust",
                &RetrievalOptions::new(1).with_context(Context::new().with_entry("b")),
            )
            .unwrap();
        assert!(r.hits().is_empty());
        assert_eq!(p.calls, 0);
    }
    #[cfg(feature = "hybrid")]
    #[test]
    fn hybrid_keeps_component_evidence_and_explicit_failure() {
        use airust::hybrid::{HybridSearchOptions, SemanticFailurePolicy};
        let (index, mut p) = fixture();
        let r = Retriever::new(HybridSource::new(&index, &mut p))
            .query("Rust", &RetrievalOptions::new(2))
            .unwrap();
        assert!(r
            .hits()
            .iter()
            .all(|h| h.evidence.lexical_score.is_some() && h.evidence.semantic_score.is_some()));
        p.fail = true;
        let opts =
            HybridSearchOptions::default().with_failure_policy(SemanticFailurePolicy::LexicalOnly);
        let r = Retriever::new(HybridSource::new(&index, &mut p).with_options(opts))
            .query("Rust", &RetrievalOptions::new(2))
            .unwrap();
        assert!(r.fallback_reason().is_some());
        assert!(r.hits().iter().all(|h| h.evidence.semantic_score.is_none()));
    }
}

#[test]
fn borrowed_and_dynamic_sources_follow_the_same_pipeline() {
    let mut source: Box<dyn RetrievalSource> = Box::new(Source::new(Mode::Valid));
    let response = Retriever::new(&mut source)
        .query("query", &RetrievalOptions::new(1))
        .unwrap();
    assert_eq!(response.hits()[0].id, "a");
    assert!(response.truncated());
}
