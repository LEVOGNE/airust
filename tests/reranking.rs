#![cfg(feature = "reranking")]

use airust::{
    reranking::{rerank, RerankError, RerankFailurePolicy, RerankOptions, Rerankable, Reranker},
    Context, Engine, KnowledgeEntry,
};
use serde_json::json;
use std::cell::Cell;

struct Scorer {
    recipe: String,
    calls: Vec<(String, Vec<String>)>,
    failure: u8,
    fail_on: usize,
}
impl Default for Scorer {
    fn default() -> Self {
        Self {
            recipe: "test-scorer-v1".into(),
            calls: Vec::new(),
            failure: 0,
            fail_on: 1,
        }
    }
}
impl Reranker for Scorer {
    fn recipe(&self) -> &str {
        &self.recipe
    }
    fn score(&mut self, query: &str, documents: &[&str]) -> Result<Vec<f64>, RerankError> {
        self.calls.push((
            query.into(),
            documents.iter().map(|s| (*s).into()).collect(),
        ));
        if self.calls.len() == self.fail_on {
            match self.failure {
                1 => return Err(RerankError::Provider("offline".into())),
                2 => return Ok(vec![]),
                3 => return Ok(vec![f64::NAN; documents.len()]),
                4 => {
                    self.recipe = "changed-v2".into();
                }
                5 => {
                    self.recipe = "changed-v2".into();
                    return Err(RerankError::Provider("offline".into()));
                }
                6 => return Err(RerankError::Output("bad indexes")),
                7 => return Ok(vec![f64::INFINITY; documents.len()]),
                8 => return Ok(vec![f64::NEG_INFINITY; documents.len()]),
                9 => return Ok(vec![1.0; documents.len() + 1]),
                10 => return Err(RerankError::NonFiniteScore(2)),
                _ => {}
            }
        }
        Ok(documents
            .iter()
            .map(|s| s.parse::<f64>().unwrap_or(0.0))
            .collect())
    }
}
fn entries() -> Vec<KnowledgeEntry> {
    ["-2", "3", "0", "3"]
        .into_iter()
        .enumerate()
        .map(|(i, t)| KnowledgeEntry::new(format!("id-{i}"), t))
        .collect()
}
fn ids<T: Rerankable>(result: &airust::reranking::RerankOutcome<'_, T>) -> Vec<String> {
    result
        .results()
        .iter()
        .map(|r| r.entry().id.clone())
        .collect()
}
fn fallback() -> RerankOptions {
    RerankOptions::new(3).with_failure_policy(RerankFailurePolicy::KeepOriginalOrder)
}

#[test]
fn scores_reorder_candidates_preserve_ties_and_borrow_the_original_records() {
    let records = entries();
    let mut provider = Scorer::default();
    let outcome = rerank(
        "specific query",
        &records,
        &mut provider,
        &RerankOptions::new(3),
    )
    .unwrap();
    assert_eq!(ids(&outcome), ["id-1", "id-3", "id-2"]);
    assert_eq!(
        outcome
            .results()
            .iter()
            .map(|r| r.original_rank)
            .collect::<Vec<_>>(),
        [2, 4, 3]
    );
    assert_eq!(outcome.results()[0].score, Some(3.0));
    assert!(std::ptr::eq(outcome.results()[0].entry(), &records[1]));
    assert!(std::ptr::eq(outcome.results()[0].candidate, &records[1]));
    assert!(outcome.reranker_used());
    assert_eq!(outcome.scored_candidates(), 4);
    assert_eq!(outcome.received_candidates(), 4);
    assert_eq!(outcome.eligible_candidates(), 4);
    assert_eq!(outcome.selected_candidates(), 4);
    assert_eq!(outcome.recipe(), Some("test-scorer-v1"));
    assert!(outcome.fallback_reason().is_none());
}

#[test]
fn signed_zero_ties_preserve_input_order_instead_of_reversing_by_float_bits() {
    let input = [
        KnowledgeEntry::new("z", "-0"),
        KnowledgeEntry::new("a", "0"),
    ];
    let out = rerank("q", &input, &mut Scorer::default(), &RerankOptions::new(2)).unwrap();
    assert_eq!(ids(&out), ["z", "a"]);
}

#[test]
fn candidate_cutoff_is_before_scoring_and_the_unscored_tail_is_never_returned() {
    let records = entries();
    let mut provider = Scorer::default();
    let out = rerank(
        "q",
        &records,
        &mut provider,
        &RerankOptions::new(1).with_candidates(2).unwrap(),
    )
    .unwrap();
    assert_eq!(ids(&out), ["id-1"]);
    assert_eq!(provider.calls[0].1, ["-2", "3"]);
    assert_eq!(out.eligible_candidates(), 4);
    assert_eq!(out.selected_candidates(), 2);
}

#[test]
fn scope_is_applied_before_the_cutoff_and_private_text_or_metadata_never_reaches_the_provider() {
    let records = [
        KnowledgeEntry::new("private", "PRIVATE TEXT")
            .with_metadata(json!({"scope":"denied","secret":"PRIVATE METADATA"})),
        KnowledgeEntry::new("allowed", "3")
            .with_metadata(json!({"scope":"ok","secret":"NOT SENT"})),
    ];
    let mut provider = Scorer::default();
    let options = RerankOptions::new(1)
        .with_candidates(1)
        .unwrap()
        .with_context(Context::new().with_metadata("scope", "ok"));
    let out = rerank("q", &records, &mut provider, &options).unwrap();
    assert_eq!(ids(&out), ["allowed"]);
    assert_eq!(out.results()[0].original_rank, 2);
    assert_eq!(provider.calls, [("q".into(), vec!["3".into()])]);
    assert_eq!(out.eligible_candidates(), 1);
}

#[test]
fn empty_denied_and_zero_limit_work_never_call_the_provider() {
    let records = entries();
    let mut provider = Scorer {
        recipe: String::new(),
        ..Scorer::default()
    };
    for (input, options, eligible) in [
        (&[][..], RerankOptions::new(2), 0),
        (&records[..], RerankOptions::new(0), 4),
        (
            &records[..],
            RerankOptions::new(2).with_context(Context::new().with_entry("missing")),
            0,
        ),
    ] {
        let out = rerank("q", input, &mut provider, &options).unwrap();
        assert!(out.results().is_empty());
        assert!(!out.reranker_used());
        assert_eq!(out.recipe(), None);
        assert_eq!(out.eligible_candidates(), eligible);
    }
    assert!(provider.calls.is_empty());
}

#[test]
fn empty_context_id_set_retains_the_existing_unrestricted_context_convention() {
    let records = entries();
    let out = rerank(
        "q",
        &records,
        &mut Scorer::default(),
        &RerankOptions::new(4).with_context(Context::new()),
    )
    .unwrap();
    assert_eq!(out.selected_candidates(), 4);
    assert!(Context::new()
        .with_entry("a")
        .intersection(&Context::new().with_entry("b"))
        .is_none());
}

#[test]
fn batches_are_bounded_preserve_original_utf8_and_return_exactly_one_score_per_document() {
    let records = [
        KnowledgeEntry::new("a", "Ärger über Straße"),
        KnowledgeEntry::new("b", "العربية"),
        KnowledgeEntry::new("c", "東京"),
    ];
    let mut provider = Scorer::default();
    let out = rerank(
        "  Fahrzeug anmelden?  ",
        &records,
        &mut provider,
        &RerankOptions::new(3).with_batch_size(2).unwrap(),
    )
    .unwrap();
    assert_eq!(provider.calls.len(), 2);
    assert_eq!(provider.calls[0].0, "  Fahrzeug anmelden?  ");
    assert_eq!(provider.calls[0].1, ["Ärger über Straße", "العربية"]);
    assert_eq!(provider.calls[1].1, ["東京"]);
    assert_eq!(ids(&out), ["a", "b", "c"]);
}

#[test]
fn all_selected_records_are_validated_before_any_inference_even_across_batches() {
    for (field, value) in [("text", " "), ("ID", "")] {
        let mut records = entries();
        if field == "text" {
            records[3].text = value.into()
        } else {
            records[3].id = value.into()
        }
        let mut provider = Scorer::default();
        let result = rerank(
            "q",
            &records,
            &mut provider,
            &RerankOptions::new(4).with_batch_size(1).unwrap(),
        );
        assert!(matches!(result,Err(RerankError::InvalidCandidate{rank:4,field:f})if f==field));
        assert!(provider.calls.is_empty());
    }
}

#[test]
fn duplicate_selected_ids_fail_before_inference_but_outside_scope_and_tail_are_not_scored() {
    let mut records = entries();
    records[3].id = records[0].id.clone();
    let mut provider = Scorer::default();
    assert!(matches!(
        rerank("q", &records, &mut provider, &RerankOptions::new(4)),
        Err(RerankError::DuplicateCandidate(4))
    ));
    assert!(provider.calls.is_empty());
    assert!(rerank(
        "q",
        &records,
        &mut provider,
        &RerankOptions::new(2).with_candidates(2).unwrap()
    )
    .is_ok());
}

#[test]
fn byte_limits_fail_before_provider_calls_without_silently_truncating_utf8() {
    let records = [
        KnowledgeEntry::new("a", "éé"),
        KnowledgeEntry::new("b", "abc"),
    ];
    for (options, error) in [
        (
            RerankOptions::new(2).with_byte_budgets(1, 10, 100).unwrap(),
            "query",
        ),
        (
            RerankOptions::new(2).with_byte_budgets(10, 3, 100).unwrap(),
            "document",
        ),
        (
            RerankOptions::new(2).with_byte_budgets(10, 10, 10).unwrap(),
            "query/document pairs",
        ),
    ] {
        let mut provider = Scorer::default();
        assert!(
            matches!(rerank("qq",&records,&mut provider,&options),Err(RerankError::Budget(kind)) if kind==error)
        );
        assert!(provider.calls.is_empty());
    }
    let out = rerank(
        "qq",
        &records,
        &mut Scorer::default(),
        &RerankOptions::new(2).with_byte_budgets(2, 4, 11).unwrap(),
    )
    .unwrap();
    assert_eq!(out.scored_candidates(), 2);
}

#[test]
fn invalid_options_and_queries_are_explicit_errors() {
    for count in [0, 2] {
        assert!(RerankOptions::new(3).with_candidates(count).is_err());
    }
    assert!(RerankOptions::new(1).with_batch_size(0).is_err());
    for budgets in [(0, 1, 1), (1, 0, 1), (1, 1, 0)] {
        assert!(RerankOptions::new(1)
            .with_byte_budgets(budgets.0, budgets.1, budgets.2)
            .is_err());
    }
    for score in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(RerankOptions::new(1).with_min_score(score).is_err());
    }
    for query in ["", " \n\t"] {
        assert!(matches!(
            rerank(
                query,
                &entries(),
                &mut Scorer::default(),
                &RerankOptions::new(0)
            ),
            Err(RerankError::EmptyQuery)
        ));
    }
}

#[test]
fn finite_thresholds_use_raw_scores_and_may_abstain_without_fabricating_relevance() {
    let records = entries();
    let out = rerank(
        "q",
        &records,
        &mut Scorer::default(),
        &RerankOptions::new(4).with_min_score(3.0).unwrap(),
    )
    .unwrap();
    assert_eq!(ids(&out), ["id-1", "id-3"]);
    let out = rerank(
        "q",
        &records,
        &mut Scorer::default(),
        &RerankOptions::new(4).with_min_score(4.0).unwrap(),
    )
    .unwrap();
    assert!(out.results().is_empty());
    assert!(out.reranker_used());
    assert_eq!(out.scored_candidates(), 4);
}

#[test]
fn non_finite_wrong_count_and_protocol_failures_never_trigger_a_fallback() {
    for failure in [2, 3, 6, 7, 8, 9] {
        let mut provider = Scorer {
            failure,
            ..Scorer::default()
        };
        assert!(rerank("q", &entries(), &mut provider, &fallback()).is_err());
    }
}

#[test]
fn provider_non_finite_batch_position_is_reported_as_the_original_input_rank() {
    let mut provider = Scorer {
        failure: 10,
        fail_on: 2,
        ..Scorer::default()
    };
    assert_eq!(
        rerank(
            "q",
            &entries(),
            &mut provider,
            &RerankOptions::new(4).with_batch_size(2).unwrap()
        )
        .err(),
        Some(RerankError::NonFiniteScore(4))
    );
}

#[test]
fn default_provider_failure_is_an_error_and_opt_in_fallback_has_no_invented_scores() {
    let records = entries();
    let mut provider = Scorer {
        failure: 1,
        ..Scorer::default()
    };
    assert!(matches!(
        rerank("q", &records, &mut provider, &RerankOptions::new(3)),
        Err(RerankError::Provider(_))
    ));
    let out = rerank(
        "q",
        &records,
        &mut Scorer {
            failure: 1,
            ..Scorer::default()
        },
        &fallback(),
    )
    .unwrap();
    assert_eq!(ids(&out), ["id-0", "id-1", "id-2"]);
    assert!(out.results().iter().all(|r| r.score.is_none()));
    assert!(!out.reranker_used());
    assert_eq!(out.scored_candidates(), 0);
    assert!(out.fallback_reason().unwrap().contains("offline"));
}

#[test]
fn later_batch_failure_discards_all_partial_scores_and_restores_the_scoped_original_prefix() {
    let records = entries();
    let mut provider = Scorer {
        failure: 1,
        fail_on: 2,
        ..Scorer::default()
    };
    let out = rerank(
        "q",
        &records,
        &mut provider,
        &fallback().with_batch_size(2).unwrap(),
    )
    .unwrap();
    assert_eq!(ids(&out), ["id-0", "id-1", "id-2"]);
    assert_eq!(out.scored_candidates(), 2);
    assert!(out.results().iter().all(|r| r.score.is_none()));
}

#[test]
fn a_score_threshold_cannot_be_bypassed_by_an_unscored_fallback() {
    let mut provider = Scorer {
        failure: 1,
        ..Scorer::default()
    };
    assert!(matches!(
        rerank(
            "q",
            &entries(),
            &mut provider,
            &fallback().with_min_score(0.0).unwrap()
        ),
        Err(RerankError::Provider(_))
    ));
}

#[test]
fn provider_recipe_is_checked_before_inference_and_after_every_success_or_error() {
    for recipe in ["".into(), " ".into(), "x".repeat(1025)] {
        let mut p = Scorer {
            recipe,
            ..Scorer::default()
        };
        assert!(matches!(
            rerank("q", &entries(), &mut p, &fallback()),
            Err(RerankError::InvalidRecipe)
        ));
        assert!(p.calls.is_empty());
    }
    for failure in [4, 5] {
        for fail_on in [1, 2] {
            let mut p = Scorer {
                failure,
                fail_on,
                ..Scorer::default()
            };
            assert!(matches!(
                rerank(
                    "q",
                    &entries(),
                    &mut p,
                    &fallback().with_batch_size(2).unwrap()
                ),
                Err(RerankError::RecipeChanged)
            ));
        }
    }
}

#[test]
fn lexical_results_retain_original_scores_terms_and_borrowed_metadata() {
    let index = Engine::from_entries([
        KnowledgeEntry::new("a", "Rust ownership").with_metadata(json!({"path":"src/lib.rs"})),
        KnowledgeEntry::new("b", "Rust pointers"),
    ])
    .unwrap();
    let results = index.search("Rust", 5);
    let out = rerank(
        "Rust",
        &results,
        &mut Scorer::default(),
        &RerankOptions::new(5),
    )
    .unwrap();
    assert_eq!(out.results()[0].candidate.score, results[0].score);
    assert_eq!(out.results()[0].candidate.matched_terms, ["rust"]);
    assert_eq!(out.results()[0].entry().metadata, results[0].entry.metadata);
    assert!(serde_json::to_string(&out)
        .unwrap()
        .contains("matched_terms"));
}

#[test]
fn each_candidate_entry_is_captured_once_so_a_stateful_adapter_cannot_swap_text_after_scope_checks()
{
    struct Rotating {
        calls: Cell<usize>,
        allowed: KnowledgeEntry,
        denied: KnowledgeEntry,
    }
    impl Rerankable for Rotating {
        fn entry(&self) -> &KnowledgeEntry {
            let call = self.calls.get();
            self.calls.set(call + 1);
            if call == 0 {
                &self.allowed
            } else {
                &self.denied
            }
        }
    }
    let records = [Rotating {
        calls: Cell::new(0),
        allowed: KnowledgeEntry::new("allowed", "1").with_metadata(json!({"scope":"ok"})),
        denied: KnowledgeEntry::new("denied", "PRIVATE"),
    }];
    let mut p = Scorer::default();
    let out = rerank(
        "q",
        &records,
        &mut p,
        &RerankOptions::new(1).with_context(Context::new().with_metadata("scope", "ok")),
    )
    .unwrap();
    assert_eq!(p.calls[0].1, ["1"]);
    assert_eq!(ids(&out), ["allowed"]);
    assert_eq!(records[0].calls.get(), 1);
}

#[test]
fn all_small_score_permutations_match_an_independent_order_and_input_remains_unchanged() {
    for a in -2..=2 {
        for b in -2..=2 {
            for c in -2..=2 {
                let input = [a, b, c]
                    .map(|value| value.to_string())
                    .into_iter()
                    .enumerate()
                    .map(|(i, t)| KnowledgeEntry::new(i.to_string(), t))
                    .collect::<Vec<_>>();
                let before = input.clone();
                let out = rerank(
                    "q",
                    &input,
                    &mut Scorer::default(),
                    &RerankOptions::new(3).with_batch_size(2).unwrap(),
                )
                .unwrap();
                let mut expected = [(0, a), (1, b), (2, c)];
                expected.sort_by_key(|&(i, score)| (-score, i));
                assert_eq!(
                    ids(&out),
                    expected.iter().map(|v| v.0.to_string()).collect::<Vec<_>>()
                );
                assert_eq!(input, before);
            }
        }
    }
}

#[cfg(feature = "hybrid")]
#[test]
fn semantic_and_hybrid_candidates_keep_component_evidence_and_assemble_context_without_index_changes(
) {
    use airust::{
        hybrid::HybridSearchOptions,
        semantic::{Embedding, EmbeddingSpace, SemanticIndex, SemanticSearchOptions},
    };
    let space = EmbeddingSpace::new("fixed", 2).unwrap();
    let mut index = SemanticIndex::new(space.clone());
    for (id, text, vector) in [
        ("a", "Rust memory", vec![1.0, 0.0]),
        ("b", "Rust scripts", vec![0.8, 0.6]),
    ] {
        index
            .upsert_embedded(
                KnowledgeEntry::new(id, text),
                Embedding::new(space.clone(), vector).unwrap(),
            )
            .unwrap();
    }
    let query = Embedding::new(space, vec![1.0, 0.0]).unwrap();
    let semantic = index
        .search_embedding(&query, &SemanticSearchOptions::new(2))
        .unwrap();
    let out = rerank(
        "Rust",
        &semantic,
        &mut Scorer::default(),
        &RerankOptions::new(2),
    )
    .unwrap();
    assert_eq!(out.results()[0].candidate.score, semantic[0].score);
    struct EmbProvider(EmbeddingSpace);
    impl airust::semantic::EmbeddingProvider for EmbProvider {
        fn space(&self) -> &EmbeddingSpace {
            &self.0
        }
        fn embed(
            &mut self,
            texts: &[&str],
            _: airust::semantic::EmbeddingPurpose,
        ) -> Result<Vec<Vec<f32>>, airust::semantic::EmbeddingError> {
            Ok(vec![vec![1.0, 0.0]; texts.len()])
        }
    }
    let mut p = EmbProvider(index.space().clone());
    let hybrid = index
        .hybrid_search("Rust", &mut p, &HybridSearchOptions::new(2))
        .unwrap();
    let ranked = rerank(
        "Rust",
        hybrid.results(),
        &mut Scorer::default(),
        &RerankOptions::new(2),
    )
    .unwrap();
    assert_eq!(ranked.results()[0].candidate.lexical_rank, Some(1));
    assert_eq!(ranked.results()[0].candidate.semantic_rank, Some(1));
    #[cfg(feature = "context-assembly")]
    {
        use airust::context_engine::{ContextAssembler, ContextOptions};
        let ids: Vec<_> = ranked
            .results()
            .iter()
            .map(|r| r.entry().id.as_str())
            .collect();
        let bundle = ContextAssembler::from_semantic(&index)
            .assemble(ids, &ContextOptions::default())
            .unwrap();
        assert_eq!(bundle.items().len(), 2);
    }
    assert_eq!(index.len(), 2);
}

#[test]
fn scoped_prefixes_cutoffs_and_batches_match_a_reference_across_seeded_cases() {
    let mut seed = 0x91e24bcd_u64;
    for case in 0..500usize {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let count = (seed as usize) % 31;
        let cap = 1 + ((seed >> 8) as usize) % 9;
        let limit = ((seed >> 16) as usize) % (cap + 1);
        let batch = 1 + ((seed >> 24) as usize) % 7;
        let mut records = Vec::new();
        let mut reference = Vec::new();
        for i in 0..count {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let score = ((seed >> 32) % 21) as i64 - 10;
            let allowed = seed & 3 != 0;
            records.push(
                KnowledgeEntry::new(i.to_string(), score.to_string())
                    .with_metadata(json!({"allowed":allowed})),
            );
            if allowed && reference.len() < cap {
                reference.push((i, score));
            }
        }
        reference.sort_by_key(|&(i, score)| (-score, i));
        reference.truncate(limit);
        let options = RerankOptions::new(limit)
            .with_candidates(cap)
            .unwrap()
            .with_batch_size(batch)
            .unwrap()
            .with_context(Context::new().with_metadata("allowed", true));
        let mut provider = Scorer::default();
        let out = rerank("q", &records, &mut provider, &options).unwrap();
        assert_eq!(
            ids(&out),
            reference
                .iter()
                .map(|v| v.0.to_string())
                .collect::<Vec<_>>(),
            "seeded case {case}"
        );
        assert_eq!(
            out.eligible_candidates(),
            records
                .iter()
                .filter(|e| e.metadata.as_ref().unwrap()["allowed"] == true)
                .count()
        );
        assert!(provider.calls.iter().all(|(_, docs)| docs.len() <= batch));
        for (hit, expected) in out.results().iter().zip(reference) {
            assert_eq!(hit.original_rank, expected.0 + 1);
            assert_eq!(hit.score, Some(expected.1 as f64));
        }
    }
}
