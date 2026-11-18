#![cfg(feature = "retrieval")]
use airust::{retrieval::*, Context, Engine, KnowledgeEntry};
use std::collections::BTreeSet;
fn options() -> RetrievalOptions {
    RetrievalOptions::new(10).with_evidence_policy(EvidencePolicy::default())
}
fn run(texts: &[&str], query: &str, options: &RetrievalOptions) -> RetrievalResponse {
    let index = Engine::from_entries(
        texts
            .iter()
            .enumerate()
            .map(|(i, t)| KnowledgeEntry::new(format!("id{i}"), *t)),
    )
    .unwrap();
    Retriever::new(LexicalSource::new(&index))
        .query(query, options)
        .unwrap()
}
#[test]
fn exact_partial_and_absent_evidence_have_explicit_verdicts() {
    let exact = run(
        &["Rust ownership prevents memory errors"],
        "Rust ownership",
        &options(),
    );
    assert_eq!(
        exact.assessment().unwrap().verdict(),
        EvidenceVerdict::MeetsPolicy
    );
    assert_eq!(
        exact
            .assessment()
            .unwrap()
            .qualified_ranks()
            .collect::<Vec<_>>(),
        [1]
    );
    let partial = run(&["Rust scripting examples"], "Rust ownership", &options());
    let a = partial.assessment().unwrap();
    assert_eq!(a.verdict(), EvidenceVerdict::Insufficient);
    assert_eq!(a.observations()[0].coverage, Some(0.5));
    assert!(a.reasons().contains(&AssessmentReason::LowTermCoverage));
    let absent = run(&["Python scripting"], "volcano", &options());
    assert_eq!(
        absent.assessment().unwrap().verdict(),
        EvidenceVerdict::Insufficient
    );
    assert!(absent
        .assessment()
        .unwrap()
        .reasons()
        .contains(&AssessmentReason::NoResults));
}
#[test]
fn coverage_is_per_excerpt_and_never_unions_unrelated_documents() {
    let r = run(
        &["Rust language", "ownership explanation"],
        "Rust ownership",
        &options(),
    );
    let a = r.assessment().unwrap();
    assert_eq!(a.qualified_hits(), 0);
    assert_eq!(a.verdict(), EvidenceVerdict::Insufficient);
    assert!(a.observations().iter().all(|o| o.coverage == Some(0.5)));
}
#[test]
fn only_delivered_text_is_assessed_and_relevant_excerpts_can_restore_coverage() {
    let text = format!(
        "{} Rust ownership prevents memory errors.",
        "Background. ".repeat(100)
    );
    let base = options().with_byte_budgets(100, 100, 100, 10000).unwrap();
    let prefix = run(&[&text], "Rust ownership", &base);
    assert_eq!(
        prefix.assessment().unwrap().verdict(),
        EvidenceVerdict::Insufficient
    );
    let located = run(
        &[&text],
        "Rust ownership",
        &base.with_relevant_excerpts(ExcerptOptions::default()),
    );
    assert_eq!(
        located.assessment().unwrap().verdict(),
        EvidenceVerdict::MeetsPolicy
    );
}
#[test]
fn disabled_or_removed_policy_preserves_original_json_and_ranking() {
    let index = Engine::from_entries([
        KnowledgeEntry::new("a", "Rust ownership"),
        KnowledgeEntry::new("b", "Rust alone"),
    ])
    .unwrap();
    let mut r = Retriever::new(LexicalSource::new(&index));
    let original = r
        .query("Rust ownership", &RetrievalOptions::new(2))
        .unwrap();
    let reset = r
        .query(
            "Rust ownership",
            &options()
                .without_evidence_policy()
                .with_candidates(100)
                .unwrap(),
        )
        .unwrap();
    assert!(original.assessment().is_none());
    assert!(!original.to_json().contains("assessment"));
    let assessed = r
        .query(
            "Rust ownership",
            &RetrievalOptions::new(2).with_evidence_policy(EvidencePolicy::default()),
        )
        .unwrap();
    for (a, b) in original.hits().iter().zip(assessed.hits()) {
        assert_eq!(a.id, b.id);
        assert_eq!(a.evidence.score.to_bits(), b.evidence.score.to_bits());
        assert_eq!(a.excerpt, b.excerpt);
    }
    assert!(reset.assessment().is_none());
    assert_eq!(
        serde_json::to_string(&assessed).unwrap(),
        assessed.to_json()
    );
    let standalone = assess_response(
        "Rust ownership",
        &original,
        &ExcerptAnalyzer::Legacy,
        &EvidencePolicy::default(),
    )
    .unwrap();
    assert_eq!(
        standalone.verdict(),
        assessed.assessment().unwrap().verdict()
    );
}
#[test]
fn stopwords_repetitions_and_no_informative_terms_are_explicit() {
    let r = run(&["Rust Rust Rust"], "the Rust Rust", &options());
    assert_eq!(r.assessment().unwrap().query_terms(), 1);
    assert_eq!(
        r.assessment().unwrap().verdict(),
        EvidenceVerdict::MeetsPolicy
    );
    let stop = run(&["the and"], "the and", &options());
    assert_eq!(
        stop.assessment().unwrap().verdict(),
        EvidenceVerdict::Undetermined
    );
    assert!(stop
        .assessment()
        .unwrap()
        .reasons()
        .contains(&AssessmentReason::NoInformativeTerms));
    let literal = run(
        &["the and"],
        "the and",
        &RetrievalOptions::new(1)
            .with_evidence_policy(EvidencePolicy::default().with_ignore_stopwords(false)),
    );
    assert_eq!(
        literal.assessment().unwrap().verdict(),
        EvidenceVerdict::MeetsPolicy
    );
}
#[test]
fn zero_limit_is_not_requested_and_never_calls_a_source() {
    struct Never;
    impl RetrievalSource for Never {
        fn retrieve<'a>(
            &'a mut self,
            _: &str,
            _: &RetrievalOptions,
        ) -> Result<RetrievalBatch<'a>, RetrievalError> {
            panic!("unexpected retrieval")
        }
    }
    let r = Retriever::new(Never)
        .query(
            "valid",
            &RetrievalOptions::new(0).with_evidence_policy(EvidencePolicy::default()),
        )
        .unwrap();
    assert_eq!(
        r.assessment().unwrap().verdict(),
        EvidenceVerdict::NotRequested
    );
    assert_eq!(r.requested_limit(), 0);
}
#[test]
fn access_scope_and_empty_text_budgets_cannot_turn_into_sufficient_evidence() {
    let r = run(
        &["Rust ownership", "Rust partial"],
        "Rust ownership",
        &options().with_context(Context::new().with_entry("id1")),
    );
    assert_eq!(
        r.assessment().unwrap().verdict(),
        EvidenceVerdict::Insufficient
    );
    let r = run(
        &["Rust ownership"],
        "Rust ownership",
        &options().with_byte_budgets(100, 0, 100, 10000).unwrap(),
    );
    assert_eq!(
        r.assessment().unwrap().verdict(),
        EvidenceVerdict::Insufficient
    );
    assert!(r
        .assessment()
        .unwrap()
        .reasons()
        .contains(&AssessmentReason::InsufficientText));
}
struct Source {
    entries: Vec<KnowledgeEntry>,
    kind: RankingKind,
    scores: Vec<f64>,
    recipe: String,
    fallback: bool,
    fail: bool,
}
impl Source {
    fn new(kind: RankingKind, scores: Vec<f64>) -> Self {
        Self {
            entries: scores
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    KnowledgeEntry::new(format!("s{i}"), "Vehicle registration at the authority")
                })
                .collect(),
            kind,
            scores,
            recipe: "fixture-v1".into(),
            fallback: false,
            fail: false,
        }
    }
}
impl RetrievalSource for Source {
    fn retrieve<'a>(
        &'a mut self,
        _: &str,
        _: &RetrievalOptions,
    ) -> Result<RetrievalBatch<'a>, RetrievalError> {
        if self.fail {
            return Err(RetrievalError::Source("offline".into()));
        }
        let mut batch = RetrievalBatch::new(
            &self.recipe,
            self.entries
                .iter()
                .zip(&self.scores)
                .map(|(entry, score)| {
                    let mut evidence = RankingEvidence::new(self.kind, *score);
                    evidence.matched_terms = vec!["invented".into()];
                    RetrievalCandidate { entry, evidence }
                })
                .collect(),
        );
        if self.fallback {
            batch.fallback_reason = Some("model unavailable".into());
        }
        Ok(batch)
    }
}
fn scored(kind: RankingKind, minimum: f64) -> EvidencePolicy {
    EvidencePolicy::scored(
        ScoreGate::new(ScoreScale::Retrieval(kind), "fixture-v1", minimum).unwrap(),
    )
}
#[test]
fn uncalibrated_semantic_hybrid_and_custom_scores_are_undetermined() {
    for kind in [
        RankingKind::Cosine,
        RankingKind::ReciprocalRankFusion,
        RankingKind::Custom,
    ] {
        let r = Retriever::new(Source::new(kind, vec![0.99]))
            .query("register a car", &options())
            .unwrap();
        assert_eq!(
            r.assessment().unwrap().verdict(),
            EvidenceVerdict::Undetermined
        );
        assert!(r
            .assessment()
            .unwrap()
            .reasons()
            .contains(&AssessmentReason::UncalibratedScoreScale));
    }
}
#[test]
fn score_gates_are_recipe_and_scale_bound_and_threshold_equality_is_inclusive() {
    let policy = scored(RankingKind::Cosine, 0.8);
    for (score, verdict) in [
        (0.799, EvidenceVerdict::Insufficient),
        (0.8, EvidenceVerdict::MeetsPolicy),
        (0.9, EvidenceVerdict::MeetsPolicy),
    ] {
        let r = Retriever::new(Source::new(RankingKind::Cosine, vec![score]))
            .query(
                "Ein Auto anmelden",
                &RetrievalOptions::new(1).with_evidence_policy(policy.clone()),
            )
            .unwrap();
        assert_eq!(r.assessment().unwrap().verdict(), verdict);
        assert_eq!(
            r.assessment().unwrap().observations()[0].observed_score,
            Some(score)
        );
    }
    let wrong = Retriever::new(Source::new(RankingKind::Bm25, vec![0.99]))
        .query(
            "car",
            &RetrievalOptions::new(1).with_evidence_policy(policy.clone()),
        )
        .unwrap();
    assert_eq!(
        wrong.assessment().unwrap().verdict(),
        EvidenceVerdict::Undetermined
    );
    let mut changed = Source::new(RankingKind::Cosine, vec![0.99]);
    changed.recipe = "fixture-v2".into();
    let r = Retriever::new(changed)
        .query(
            "car",
            &RetrievalOptions::new(1).with_evidence_policy(policy),
        )
        .unwrap();
    assert_eq!(
        r.assessment().unwrap().verdict(),
        EvidenceVerdict::Undetermined
    );
    assert!(r
        .assessment()
        .unwrap()
        .reasons()
        .contains(&AssessmentReason::ScoreRecipeMismatch));
}
#[test]
fn fake_claimed_terms_do_not_replace_actual_excerpt_evidence() {
    let mut source = Source::new(RankingKind::Bm25, vec![100.0]);
    source.entries[0].text = "A recipe for bread".into();
    let r = Retriever::new(source)
        .query("Rust ownership", &options())
        .unwrap();
    assert_eq!(
        r.assessment().unwrap().verdict(),
        EvidenceVerdict::Insufficient
    );
    assert_eq!(r.assessment().unwrap().observations()[0].matched_terms, 0);
}
#[test]
fn explicit_fallback_is_undetermined_including_an_empty_fallback() {
    for scores in [vec![0.99], vec![]] {
        let mut source = Source::new(RankingKind::Cosine, scores);
        source.fallback = true;
        let r = Retriever::new(source)
            .query(
                "car",
                &RetrievalOptions::new(1).with_evidence_policy(scored(RankingKind::Cosine, 0.8)),
            )
            .unwrap();
        assert_eq!(
            r.assessment().unwrap().verdict(),
            EvidenceVerdict::Undetermined
        );
        assert!(r
            .assessment()
            .unwrap()
            .reasons()
            .contains(&AssessmentReason::ProviderFallback));
    }
}
#[test]
fn source_errors_and_invalid_numeric_evidence_stay_errors() {
    let mut source = Source::new(RankingKind::Cosine, vec![0.9]);
    source.fail = true;
    assert!(matches!(
        Retriever::new(source).query("car", &options()),
        Err(RetrievalError::Source(_))
    ));
    for score in [f64::NAN, f64::INFINITY, 1.1, -1.1] {
        assert!(
            Retriever::new(Source::new(RankingKind::Cosine, vec![score]))
                .query(
                    "car",
                    &RetrievalOptions::new(1)
                        .with_evidence_policy(scored(RankingKind::Cosine, 0.8))
                )
                .is_err()
        );
    }
}
#[test]
fn score_and_lexical_policy_requires_both_criteria_in_the_same_hit() {
    let gate = ScoreGate::new(
        ScoreScale::Retrieval(RankingKind::Cosine),
        "fixture-v1",
        0.8,
    )
    .unwrap();
    let policy = EvidencePolicy::scored_and_lexical(gate, 1.0).unwrap();
    for (query, score, expected) in [
        ("Vehicle registration", 0.9, EvidenceVerdict::MeetsPolicy),
        ("Vehicle registration", 0.7, EvidenceVerdict::Insufficient),
        ("Rust ownership", 0.9, EvidenceVerdict::Insufficient),
    ] {
        let r = Retriever::new(Source::new(RankingKind::Cosine, vec![score]))
            .query(
                query,
                &RetrievalOptions::new(1).with_evidence_policy(policy.clone()),
            )
            .unwrap();
        assert_eq!(r.assessment().unwrap().verdict(), expected);
    }
}
#[test]
fn limits_fail_before_source_work_and_configuration_is_checked() {
    struct Never;
    impl RetrievalSource for Never {
        fn retrieve<'a>(
            &'a mut self,
            _: &str,
            _: &RetrievalOptions,
        ) -> Result<RetrievalBatch<'a>, RetrievalError> {
            panic!("unexpected source")
        }
    }
    let p = EvidencePolicy::default()
        .with_limits(AssessmentLimits::default().with_query_limits(3, 1).unwrap());
    assert!(matches!(
        Retriever::new(Never).query(
            "long query",
            &RetrievalOptions::new(1).with_evidence_policy(p)
        ),
        Err(RetrievalError::Budget(_))
    ));
    let p = EvidencePolicy::default()
        .with_limits(AssessmentLimits::default().with_hit_limit(1).unwrap());
    assert!(Retriever::new(Never)
        .query("q", &RetrievalOptions::new(2).with_evidence_policy(p))
        .is_err());
    for value in [f64::NAN, 0.0, -0.1, 1.1] {
        assert!(EvidencePolicy::lexical(value).is_err());
    }
    assert!(EvidencePolicy::default().with_min_hits(0).is_err());
    assert!(EvidencePolicy::default().with_min_text_bytes(0).is_err());
    assert!(EvidencePolicy::default().with_label("bad\nlabel").is_err());
    assert!(AssessmentLimits::default().with_query_limits(0, 1).is_err());
    assert!(AssessmentLimits::default()
        .with_text_limits(1, 0, 1)
        .is_err());
    assert!(AssessmentLimits::default().with_hit_limit(0).is_err());
    for (scale, score) in [
        (ScoreScale::Retrieval(RankingKind::Cosine), 1.01),
        (ScoreScale::Retrieval(RankingKind::Bm25), -0.01),
        (ScoreScale::Rerank, f64::NAN),
    ] {
        assert!(ScoreGate::new(scale, "r", score).is_err());
    }
}
#[test]
fn json_tail_removal_reassesses_only_retained_hits_and_never_keeps_stale_acceptance() {
    let index = Engine::from_entries([
        KnowledgeEntry::new("a", "Rust ownership"),
        KnowledgeEntry::new("b", "Rust ownership"),
    ])
    .unwrap();
    let policy = EvidencePolicy::default().with_min_hits(2).unwrap();
    let mut saw_pass = false;
    let mut saw_reduced = false;
    for cap in (1..6000).step_by(23) {
        let r = Retriever::new(LexicalSource::new(&index)).query(
            "Rust ownership",
            &RetrievalOptions::new(2)
                .with_evidence_policy(policy.clone())
                .with_byte_budgets(100, 100, 100, cap)
                .unwrap(),
        );
        match r {
            Ok(response) => {
                assert!(response.to_json().len() <= cap);
                let a = response.assessment().unwrap();
                assert_eq!(a.observations().len(), response.hits().len());
                assert_eq!(a.returned_hits(), response.hits().len());
                assert!(a
                    .qualified_ranks()
                    .all(|rank| rank <= response.hits().len()));
                if response.hits().len() == 2 {
                    saw_pass = true;
                    assert_eq!(a.verdict(), EvidenceVerdict::MeetsPolicy);
                } else {
                    saw_reduced = true;
                    assert_eq!(a.verdict(), EvidenceVerdict::Insufficient);
                    assert!(a.reasons().contains(&AssessmentReason::OutputBudget));
                }
                assert_eq!(
                    serde_json::to_string(&response).unwrap(),
                    response.to_json()
                );
            }
            Err(e) => assert!(matches!(e, RetrievalError::Budget("output envelope bytes"))),
        }
    }
    assert!(saw_pass && saw_reduced);
}
#[test]
fn seeded_coverage_agrees_with_an_independent_set_reference() {
    let mut seed = 71u64;
    let words = ["rust", "memory", "ownership", "safety", "python", "storage"];
    for _ in 0..600 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let query: BTreeSet<_> = (0..6)
            .filter(|i| seed & (1 << i) != 0)
            .map(|i| words[i])
            .collect();
        if query.is_empty() {
            continue;
        }
        let text: BTreeSet<_> = (0..6)
            .filter(|i| seed & (1 << (i + 8)) != 0)
            .map(|i| words[i])
            .collect();
        if text.is_empty() {
            continue;
        }
        let q = query.iter().copied().collect::<Vec<_>>().join(" ");
        let t = text.iter().copied().collect::<Vec<_>>().join(" ");
        let mut source = Source::new(RankingKind::Bm25, vec![1.0]);
        source.entries[0].text = t;
        let r = Retriever::new(source).query(&q, &options()).unwrap();
        let o = &r.assessment().unwrap().observations()[0];
        assert_eq!(o.matched_terms, query.intersection(&text).count());
        assert_eq!(
            o.coverage,
            Some(query.intersection(&text).count() as f64 / query.len() as f64)
        );
        assert_eq!(o.qualifies, Some(query.is_subset(&text)));
    }
}
#[cfg(feature = "advanced-search")]
#[test]
fn german_word_forms_and_compatibility_composition_use_the_source_analyzer() {
    use airust::text_search::{AnalyzerConfig, TextIndex};
    let index = TextIndex::from_entries(
        AnalyzerConfig::german(),
        [KnowledgeEntry::new("de", "Häuser und Gärten")],
    )
    .unwrap();
    let r = Retriever::new(TextSource::new(&index))
        .query(
            "Haus Garten",
            &RetrievalOptions::new(1).with_evidence_policy(EvidencePolicy::default()),
        )
        .unwrap();
    assert_eq!(
        r.assessment().unwrap().verdict(),
        EvidenceVerdict::MeetsPolicy
    );
    let korean = run(&["ㄱㅏ"], "가", &options());
    assert_eq!(
        korean.assessment().unwrap().verdict(),
        EvidenceVerdict::MeetsPolicy
    );
}
#[cfg(feature = "reranking")]
#[test]
fn rerank_gates_keep_original_scores_and_refuse_fallback_or_wrong_recipes() {
    use airust::reranking::{RerankError, RerankFailurePolicy, RerankOptions, Reranker};
    struct Ranker(bool);
    impl Reranker for Ranker {
        fn recipe(&self) -> &str {
            "ranker-v1"
        }
        fn score(&mut self, _: &str, docs: &[&str]) -> Result<Vec<f64>, RerankError> {
            if self.0 {
                Err(RerankError::Provider("offline".into()))
            } else {
                Ok(docs.iter().map(|_| 2.0).collect())
            }
        }
    }
    let index = Engine::from_entries([KnowledgeEntry::new("a", "Rust ownership")]).unwrap();
    let gate = ScoreGate::new(ScoreScale::Rerank, "ranker-v1", 1.0).unwrap();
    let options = RetrievalOptions::new(1).with_evidence_policy(EvidencePolicy::scored(gate));
    let r = Retriever::new(LexicalSource::new(&index))
        .with_reranker(&mut Ranker(false), RerankOptions::new(1))
        .query("Rust", &options)
        .unwrap();
    assert_eq!(
        r.assessment().unwrap().verdict(),
        EvidenceVerdict::MeetsPolicy
    );
    assert_eq!(r.hits()[0].rerank_score, Some(2.0));
    assert_ne!(r.hits()[0].evidence.score, 2.0);
    let r = Retriever::new(LexicalSource::new(&index))
        .with_reranker(
            &mut Ranker(true),
            RerankOptions::new(1).with_failure_policy(RerankFailurePolicy::KeepOriginalOrder),
        )
        .query("Rust", &options)
        .unwrap();
    assert_eq!(
        r.assessment().unwrap().verdict(),
        EvidenceVerdict::Undetermined
    );
    let missing = Retriever::new(LexicalSource::new(&index))
        .query("Rust", &options)
        .unwrap();
    assert_eq!(
        missing.assessment().unwrap().verdict(),
        EvidenceVerdict::Undetermined
    );
    assert!(missing
        .assessment()
        .unwrap()
        .reasons()
        .contains(&AssessmentReason::ScoreUnavailable));
}
