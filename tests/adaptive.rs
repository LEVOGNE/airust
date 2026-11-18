#![cfg(feature = "adaptive")]
use airust::{adaptive::*, retrieval::*, Context, Engine, KnowledgeEntry};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
struct Fixed {
    entry: KnowledgeEntry,
    calls: Arc<AtomicUsize>,
    fail: bool,
}
impl RetrievalSource for Fixed {
    fn retrieve<'a>(
        &'a mut self,
        _: &str,
        options: &RetrievalOptions,
    ) -> Result<RetrievalBatch<'a>, RetrievalError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        if self.fail {
            return Err(RetrievalError::Source("fixture failure".into()));
        }
        let hits = if options
            .context()
            .is_none_or(|scope| scope.includes(&self.entry))
        {
            vec![RetrievalCandidate {
                entry: &self.entry,
                evidence: RankingEvidence::new(RankingKind::Custom, 1.0),
            }]
        } else {
            vec![]
        };
        Ok(RetrievalBatch::new("fixed-original-recipe", hits))
    }
}
fn fixed(calls: Arc<AtomicUsize>) -> Fixed {
    Fixed {
        entry: KnowledgeEntry::new("a", "Rust ownership prevents memory errors"),
        calls,
        fail: false,
    }
}
#[test]
fn short_and_long_routes_use_only_explicitly_registered_backends() {
    let lexical = Arc::new(AtomicUsize::new(0));
    let model = Arc::new(AtomicUsize::new(0));
    let source = AdaptiveSource::default()
        .with_source("words", SourceRole::Lexical, fixed(lexical.clone()))
        .unwrap()
        .with_source("configured-model", SourceRole::Hybrid, fixed(model.clone()))
        .unwrap();
    let mut retriever = Retriever::new(source);
    retriever
        .query("Rust ownership", &RetrievalOptions::new(1))
        .unwrap();
    assert_eq!(retriever.source().last_route().unwrap().source(), "words");
    assert_eq!(model.load(Ordering::Relaxed), 0);
    retriever
        .query(
            "How does Rust ownership prevent memory errors",
            &RetrievalOptions::new(1),
        )
        .unwrap();
    assert_eq!(
        retriever.source().last_route().unwrap().source(),
        "configured-model"
    );
    assert_eq!(model.load(Ordering::Relaxed), 1);
    assert_eq!(lexical.load(Ordering::Relaxed), 1);
}
#[test]
fn model_free_long_query_works_without_loading_or_requiring_a_model() {
    let calls = Arc::new(AtomicUsize::new(0));
    let source = AdaptiveSource::default()
        .with_source("only-words", SourceRole::Lexical, fixed(calls.clone()))
        .unwrap();
    let mut retriever = Retriever::new(source);
    retriever
        .query(
            "How does Rust ownership prevent memory errors",
            &RetrievalOptions::new(1),
        )
        .unwrap();
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    assert_eq!(retriever.source().available().len(), 1);
}
#[test]
fn real_lexical_recipe_scores_json_and_assessment_remain_original() {
    let index = Engine::from_entries([KnowledgeEntry::new("a", "Rust ownership")]).unwrap();
    let options = RetrievalOptions::new(1).with_evidence_policy(EvidencePolicy::default());
    let baseline = Retriever::new(LexicalSource::new(&index))
        .query("Rust ownership", &options)
        .unwrap();
    let source = AdaptiveSource::default()
        .with_source("words", SourceRole::Lexical, LexicalSource::new(&index))
        .unwrap();
    let mut retriever = Retriever::new(source);
    let actual = retriever.query("Rust ownership", &options).unwrap();
    assert_eq!(actual.to_json(), baseline.to_json());
    let trace = retriever.source().last_route().unwrap();
    assert_eq!(trace.recipe(), Some(actual.recipe()));
    assert_eq!(trace.state(), RouteState::Returned);
    assert_eq!(trace.received_candidates(), Some(1));
    assert!(!actual.to_json().contains("route"));
}
#[test]
fn quote_queries_require_positional_capability_instead_of_silently_dropping_phrase_semantics() {
    let calls = Arc::new(AtomicUsize::new(0));
    let source = AdaptiveSource::default()
        .with_source("words", SourceRole::Lexical, fixed(calls.clone()))
        .unwrap();
    assert!(matches!(
        Retriever::new(source).query("\"Rust ownership\"", &RetrievalOptions::new(1)),
        Err(RetrievalError::Configuration(_))
    ));
    assert_eq!(calls.load(Ordering::Relaxed), 0);
}
#[test]
fn source_failures_do_not_silently_try_another_backend() {
    let failed = Arc::new(AtomicUsize::new(0));
    let unused = Arc::new(AtomicUsize::new(0));
    let mut source = fixed(failed.clone());
    source.fail = true;
    let source = AdaptiveSource::default()
        .with_source("text-failure", SourceRole::Text, source)
        .unwrap()
        .with_source("other", SourceRole::Lexical, fixed(unused.clone()))
        .unwrap();
    let mut retriever = Retriever::new(source);
    assert!(matches!(
        retriever.query("Rust", &RetrievalOptions::new(1)),
        Err(RetrievalError::Source(_))
    ));
    assert_eq!(
        retriever.source().last_route().unwrap().state(),
        RouteState::Failed
    );
    assert_eq!(failed.load(Ordering::Relaxed), 1);
    assert_eq!(unused.load(Ordering::Relaxed), 0);
}
#[test]
fn registration_and_policy_outputs_are_bounded_and_must_name_an_existing_backend() {
    assert!(RouteChoice::new("", "reason").is_err());
    assert!(RouteChoice::new("name", "bad\nreason").is_err());
    assert!(RouteChoice::new("x".repeat(65), "reason").is_err());
    let calls = Arc::new(AtomicUsize::new(0));
    assert!(AdaptiveSource::default()
        .with_source("bad\nname", SourceRole::Custom, fixed(calls.clone()))
        .is_err());
    let source = AdaptiveSource::default()
        .with_source("same", SourceRole::Lexical, fixed(calls.clone()))
        .unwrap();
    assert!(source
        .with_source("same", SourceRole::Lexical, fixed(calls.clone()))
        .is_err());
    struct Unknown;
    impl RoutingPolicy for Unknown {
        fn choose(&self, _: &str, _: &[BackendInfo]) -> Result<RouteChoice, RetrievalError> {
            RouteChoice::new("not-registered", "caller policy")
        }
    }
    let source = AdaptiveSource::new(Unknown)
        .with_source("actual", SourceRole::Lexical, fixed(calls.clone()))
        .unwrap();
    assert!(Retriever::new(source)
        .query("Rust", &RetrievalOptions::new(1))
        .is_err());
    assert_eq!(calls.load(Ordering::Relaxed), 0);
}
#[test]
fn context_and_budget_options_reach_the_selected_source_unchanged() {
    let index = Engine::from_entries([
        KnowledgeEntry::new("allowed", "Rust ownership"),
        KnowledgeEntry::new("private", "Rust ownership"),
    ])
    .unwrap();
    let source = AdaptiveSource::default()
        .with_source("words", SourceRole::Lexical, LexicalSource::new(&index))
        .unwrap();
    let mut retriever = Retriever::new(source);
    let options = RetrievalOptions::new(1)
        .with_context(Context::new().with_entry("allowed"))
        .with_byte_budgets(128, 128, 128, 4096)
        .unwrap();
    let result = retriever.query("Rust ownership", &options).unwrap();
    assert_eq!(result.hits()[0].id, "allowed");
    assert!(result.to_json().len() <= 4096);
    assert_eq!(
        retriever
            .source()
            .last_route()
            .unwrap()
            .received_candidates(),
        Some(1)
    );
}
#[test]
fn zero_limits_and_early_assessment_caps_never_invoke_source_preflight() {
    struct Never;
    impl RetrievalSource for Never {
        fn prepare_query(&mut self, _: &str, _: &RetrievalOptions) -> Result<(), RetrievalError> {
            panic!("unexpected preflight")
        }
        fn retrieve<'a>(
            &'a mut self,
            _: &str,
            _: &RetrievalOptions,
        ) -> Result<RetrievalBatch<'a>, RetrievalError> {
            panic!("unexpected source")
        }
    }
    assert!(Retriever::new(Never)
        .query("Rust", &RetrievalOptions::new(0))
        .unwrap()
        .hits()
        .is_empty());
    let limits = AssessmentLimits::default().with_hit_limit(1).unwrap();
    let options = RetrievalOptions::new(2)
        .with_evidence_policy(EvidencePolicy::default().with_limits(limits));
    assert!(matches!(
        Retriever::new(Never).query("Rust", &options),
        Err(RetrievalError::Budget(_))
    ));
}
#[test]
fn preflight_errors_are_technical_errors_and_never_no_answer_results() {
    struct Reject;
    impl RetrievalSource for Reject {
        fn prepare_query(&mut self, _: &str, _: &RetrievalOptions) -> Result<(), RetrievalError> {
            Err(RetrievalError::Source("preflight failed".into()))
        }
        fn retrieve<'a>(
            &'a mut self,
            _: &str,
            _: &RetrievalOptions,
        ) -> Result<RetrievalBatch<'a>, RetrievalError> {
            panic!("rejected source called")
        }
    }
    let source = AdaptiveSource::default()
        .with_source("reject", SourceRole::Lexical, Reject)
        .unwrap();
    let mut retriever = Retriever::new(source);
    assert!(matches!(
        retriever.query(
            "Rust",
            &RetrievalOptions::new(1).with_evidence_policy(EvidencePolicy::default())
        ),
        Err(RetrievalError::Source(_))
    ));
    assert_eq!(
        retriever.source().last_route().unwrap().state(),
        RouteState::Failed
    );
}
#[cfg(feature = "advanced-search")]
#[test]
fn stateful_policy_is_called_once_and_the_frozen_analyzer_matches_the_executed_route() {
    use airust::text_search::*;
    struct Alternating(Arc<AtomicUsize>);
    impl RoutingPolicy for Alternating {
        fn choose(&self, _: &str, _: &[BackendInfo]) -> Result<RouteChoice, RetrievalError> {
            RouteChoice::new(
                if self.0.fetch_add(1, Ordering::Relaxed).is_multiple_of(2) {
                    "german"
                } else {
                    "legacy"
                },
                "alternating fixture",
            )
        }
    }
    let calls = Arc::new(AtomicUsize::new(0));
    let original = format!("{} Häuser haben Türen.", "Einleitung. ".repeat(80));
    let text = TextIndex::from_entries(
        AnalyzerConfig::german(),
        [KnowledgeEntry::new("a", &original)],
    )
    .unwrap();
    let lexical = Engine::from_entries([KnowledgeEntry::new("a", &original)]).unwrap();
    let source = AdaptiveSource::new(Alternating(calls.clone()))
        .with_source("german", SourceRole::Text, TextSource::new(&text))
        .unwrap()
        .with_source("legacy", SourceRole::Lexical, LexicalSource::new(&lexical))
        .unwrap();
    let mut retriever = Retriever::new(source);
    let options = RetrievalOptions::new(1)
        .with_relevant_excerpts(ExcerptOptions::default())
        .with_evidence_policy(EvidencePolicy::default());
    let first = retriever.query("Haus", &options).unwrap();
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    assert_eq!(
        first.assessment().unwrap().verdict(),
        EvidenceVerdict::MeetsPolicy
    );
    assert!(first.hits()[0].excerpt.contains("Häuser"));
    assert_eq!(retriever.source().last_route().unwrap().source(), "german");
    let second = retriever.query("Haus", &options).unwrap();
    assert_eq!(calls.load(Ordering::Relaxed), 2);
    assert!(second.hits().is_empty());
    assert_eq!(retriever.source().last_route().unwrap().source(), "legacy");
}
#[cfg(feature = "advanced-search")]
#[test]
fn quoted_default_route_keeps_phrase_filtering_and_original_spans() {
    use airust::text_search::*;
    let text = TextIndex::from_entries(
        AnalyzerConfig::german(),
        [
            KnowledgeEntry::new("good", "Häuser haben Türen"),
            KnowledgeEntry::new("wrong", "Türen haben Häuser"),
        ],
    )
    .unwrap();
    let source = AdaptiveSource::default()
        .with_source("text", SourceRole::Text, TextSource::new(&text))
        .unwrap();
    let result = Retriever::new(source)
        .query(
            "\"Haus haben\"",
            &RetrievalOptions::new(10).with_relevant_excerpts(ExcerptOptions::default()),
        )
        .unwrap();
    assert_eq!(result.hits().len(), 1);
    assert_eq!(result.hits()[0].id, "good");
    assert!(!result.hits()[0].evidence.phrase_spans.is_empty());
}
#[cfg(feature = "execution")]
#[test]
fn routing_callback_cancellation_does_not_call_the_policy_or_provider() {
    use airust::execution::*;
    struct NeverPolicy;
    impl RoutingPolicy for NeverPolicy {
        fn choose(&self, _: &str, _: &[BackendInfo]) -> Result<RouteChoice, RetrievalError> {
            panic!("cancelled policy called")
        }
    }
    let calls = Arc::new(AtomicUsize::new(0));
    let token = CancellationToken::new();
    let cancel = token.clone();
    let control = WorkControl::new(token).with_observer(move |event: &ProgressEvent| {
        if event.phase() == WorkPhase::Routing {
            cancel.cancel();
        }
    });
    let source = AdaptiveSource::new(NeverPolicy)
        .with_source("words", SourceRole::Lexical, fixed(calls.clone()))
        .unwrap();
    assert!(matches!(
        Retriever::new(source).query("Rust", &RetrievalOptions::new(1).with_control(control)),
        Err(RetrievalError::Execution(ExecutionError::Cancelled))
    ));
    assert_eq!(calls.load(Ordering::Relaxed), 0);
}
#[cfg(feature = "advanced-search")]
#[test]
fn boxed_and_borrowed_adaptive_sources_forward_preflight_before_analyzer_selection() {
    use airust::text_search::*;
    let text = TextIndex::from_entries(
        AnalyzerConfig::german(),
        [KnowledgeEntry::new("a", "Häuser haben Türen")],
    )
    .unwrap();
    let options = RetrievalOptions::new(1).with_evidence_policy(EvidencePolicy::default());
    let source = AdaptiveSource::default()
        .with_source("text", SourceRole::Text, TextSource::new(&text))
        .unwrap();
    let result = Retriever::new(Box::new(source))
        .query("Haus", &options)
        .unwrap();
    assert_eq!(
        result.assessment().unwrap().verdict(),
        EvidenceVerdict::MeetsPolicy
    );
    let mut source = AdaptiveSource::default()
        .with_source("text", SourceRole::Text, TextSource::new(&text))
        .unwrap();
    let result = Retriever::new(&mut source).query("Haus", &options).unwrap();
    assert_eq!(
        result.assessment().unwrap().verdict(),
        EvidenceVerdict::MeetsPolicy
    );
    assert_eq!(source.last_route().unwrap().state(), RouteState::Returned);
}
