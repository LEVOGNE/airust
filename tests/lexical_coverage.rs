#![cfg(feature = "retrieval")]
use airust::{retrieval::*, Context, Engine, KnowledgeEntry, SearchAlgorithm, SearchOptions};

fn engine() -> Engine {
    Engine::from_entries([
        KnowledgeEntry::new("a", "Which original query question contains the words?")
            .with_metadata(serde_json::json!({"tenant":"a"})),
        KnowledgeEntry::new("b", "ProvenanceStore revision identity.")
            .with_metadata(serde_json::json!({"tenant":"a"})),
        KnowledgeEntry::new("c", "Provenance Store revision identity fingerprint.")
            .with_metadata(serde_json::json!({"tenant":"a"})),
        KnowledgeEntry::new("z", "Provenance Store revision identity fingerprint.")
            .with_metadata(serde_json::json!({"tenant":"b"})),
    ])
    .unwrap()
}

#[test]
fn coverage_components_match_independent_search_and_original_text_tokens() {
    let index = engine();
    let query = "Which ProvenanceStore revision identity?";
    for algorithm in [SearchAlgorithm::Bm25, SearchAlgorithm::Tfidf] {
        for weight in [0.0, 0.5, 1.0] {
            let config = LexicalCoverageOptions::default()
                .with_coverage_weight(weight)
                .unwrap();
            let context = Context::new().with_metadata("tenant", "a");
            let request = RetrievalOptions::new(2)
                .with_candidates(2)
                .unwrap()
                .with_context(context.clone());
            let search = SearchOptions {
                algorithm,
                limit: 2,
                context: Some(context),
                ..Default::default()
            };
            let analysis = config.query_options().analyze(query).unwrap();
            let terms: std::collections::BTreeSet<_> =
                airust::text::tokenize(analysis.effective_query())
                    .into_iter()
                    .collect();
            let raw = index.search_with_options(analysis.effective_query(), &search);
            let max = raw[0].score;
            let source = LexicalCoverageSource::new(&index)
                .with_options(search)
                .with_coverage_options(config);
            let batch = source.inspect(query, &request).unwrap();
            assert_eq!(batch.hits().len(), raw.len());
            for h in batch.hits() {
                let at = raw.iter().position(|x| x.entry.id == h.entry().id).unwrap();
                let tokens: std::collections::BTreeSet<_> = airust::text::tokenize(&h.entry().text)
                    .into_iter()
                    .collect();
                let count = terms.intersection(&tokens).count();
                assert_eq!(h.lexical_rank(), at + 1);
                assert_eq!(h.lexical_score(), raw[at].score);
                assert_eq!(h.query_terms(), terms.len());
                assert_eq!(h.matched_query_terms(), count);
                assert_eq!(h.normalized_score(), raw[at].score / max);
                let expected = (1.0 - weight) * raw[at].score / max
                    + weight * count as f64 / terms.len() as f64;
                assert!((h.score() - expected).abs() < 1e-15);
            }
            assert!(batch
                .hits()
                .windows(2)
                .all(|w| w[0].score() >= w[1].score()));
        }
    }
}

#[test]
fn coverage_counts_unique_prepared_terms_including_unknown_and_unicode_terms() {
    let index = Engine::from_entries([
        KnowledgeEntry::new("a", "ProvenanceStore provenance store Café हिंदी rust"),
        KnowledgeEntry::new("b", "Café हिंदी rust rust"),
    ])
    .unwrap();
    for query in [
        "Which ProvenanceStore ProvenanceStore Café cafe\u{301} हिंदी unknown?",
        "ＲＵＳＴ rust rust",
        "...🦀",
        "not unrecorded",
    ] {
        let settings = LexicalCoverageOptions::default();
        let effective = settings.query_options().analyze(query).unwrap();
        let query_set: std::collections::BTreeSet<_> =
            airust::text::tokenize(effective.effective_query())
                .into_iter()
                .collect();
        let source = LexicalCoverageSource::new(&index).with_coverage_options(settings);
        let batch = source.inspect(query, &RetrievalOptions::new(10)).unwrap();
        for hit in batch.hits() {
            let entry_terms: std::collections::BTreeSet<_> =
                airust::text::tokenize(&hit.entry().text)
                    .into_iter()
                    .collect();
            assert_eq!(hit.query_terms(), query_set.len());
            assert_eq!(
                hit.matched_query_terms(),
                query_set.intersection(&entry_terms).count()
            );
            assert_eq!(hit.matched_terms().len(), hit.matched_query_terms());
        }
    }
}

#[test]
fn configuration_rejects_invalid_weights_and_documents_extreme_legal_weights() {
    for w in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0, 1.00001] {
        assert!(LexicalCoverageOptions::default()
            .with_coverage_weight(w)
            .is_err());
    }
    assert_eq!(LexicalCoverageOptions::default().coverage_weight(), 0.5);
    for w in [0.0, 1.0] {
        assert_eq!(
            LexicalCoverageOptions::default()
                .with_coverage_weight(w)
                .unwrap()
                .coverage_weight(),
            w
        );
    }
}

#[test]
fn broader_term_coverage_can_promote_a_passage_and_exact_ties_use_ids() {
    let index = Engine::from_entries([
        KnowledgeEntry::new("z", "alpha beta gamma"),
        KnowledgeEntry::new("a", "alpha beta gamma"),
        KnowledgeEntry::new("partial", "alpha alpha alpha alpha alpha alpha"),
    ])
    .unwrap();
    let config = LexicalCoverageOptions::default()
        .with_query_options(
            LexicalQueryOptions::default()
                .without_word_filter()
                .without_identifier_parts(),
        )
        .with_coverage_weight(1.0)
        .unwrap();
    let source = LexicalCoverageSource::new(&index).with_coverage_options(config);
    let batch = source
        .inspect("alpha beta gamma", &RetrievalOptions::new(3))
        .unwrap();
    assert_eq!(
        batch
            .hits()
            .iter()
            .map(|h| h.entry().id.as_str())
            .collect::<Vec<_>>(),
        ["a", "z", "partial"]
    );
    assert_eq!(batch.hits()[0].query_coverage(), 1.0);
    assert_eq!(batch.hits()[2].query_coverage(), 1.0 / 3.0);
}

#[test]
fn scope_intersection_and_disjoint_scope_apply_before_analysis() {
    let index = engine();
    let search = SearchOptions {
        context: Some(Context::new().with_metadata("tenant", "a")),
        ..Default::default()
    };
    let source = LexicalCoverageSource::new(&index).with_options(search);
    let request =
        RetrievalOptions::new(10).with_context(Context::new().with_metadata("tenant", "b"));
    let batch = source.inspect("ProvenanceStore", &request).unwrap();
    assert!(batch.hits().is_empty());
    assert!(batch.analysis().is_none());
    let request = RetrievalOptions::new(10).with_context(Context::new().with_entry("b"));
    let batch = source.inspect("ProvenanceStore", &request).unwrap();
    assert_eq!(batch.hits().len(), 1);
    assert_eq!(batch.hits()[0].entry().id, "b");
}

#[test]
fn response_keeps_source_spans_component_scores_and_has_no_last_query_state() {
    let index = engine();
    let q = "Which ProvenanceStore revision identity?";
    let options = RetrievalOptions::new(3)
        .with_relevant_excerpts(ExcerptOptions::default().with_leading_context())
        .with_distinct_evidence();
    let response = Retriever::new(LexicalCoverageSource::new(&index))
        .query(q, &options)
        .unwrap();
    assert!(response.recipe().starts_with("airust-lexical-coverage-v1:"));
    for h in response.hits() {
        assert_eq!(
            h.span.slice(&index.get(&h.id).unwrap().text),
            Some(h.excerpt.as_str())
        );
        assert_eq!(h.evidence.kind, RankingKind::Custom);
        assert!(h.evidence.lexical_rank.is_some());
        assert!(h.evidence.lexical_score.is_some());
        assert!(h.evidence.semantic_rank.is_none());
        assert!(h.evidence.semantic_score.is_none());
    }
    let mut source = LexicalCoverageSource::new(&index);
    let _ = source.retrieve(q, &options).unwrap();
    let current = source.retrieve("identity", &options).unwrap();
    let mut fresh = LexicalCoverageSource::new(&index);
    let expected = fresh.retrieve("identity", &options).unwrap();
    assert_eq!(
        current
            .candidates
            .iter()
            .map(|h| (&h.entry.id, h.evidence.score.to_bits()))
            .collect::<Vec<_>>(),
        expected
            .candidates
            .iter()
            .map(|h| (&h.entry.id, h.evidence.score.to_bits()))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        Retriever::new(LexicalSource::new(&index))
            .query(q, &options)
            .unwrap()
            .recipe(),
        "airust-lexical-v1"
    );
}

#[test]
fn blank_punctuation_and_preparation_limits_fail_closed_or_return_empty() {
    let index = engine();
    let source = LexicalCoverageSource::new(&index);
    let request = RetrievalOptions::new(2);
    assert!(source.inspect(" ", &request).is_err());
    assert!(source
        .inspect("🦀...!", &request)
        .unwrap()
        .hits()
        .is_empty());
    let short = request.with_byte_budgets(3, 200, 100, 2048).unwrap();
    assert!(matches!(
        source.inspect("Rust", &short),
        Err(RetrievalError::Budget(_))
    ));
    let source = source.with_coverage_options(
        LexicalCoverageOptions::default().with_query_options(
            LexicalQueryOptions::default()
                .with_limits(100, 100, 1)
                .unwrap(),
        ),
    );
    assert!(source
        .inspect("HTTP2Server", &RetrievalOptions::new(2))
        .is_err());
}

#[cfg(feature = "execution")]
#[test]
fn cancelled_work_returns_error_without_partial_evidence() {
    let index = engine();
    let control = airust::execution::WorkControl::default();
    control.token().cancel();
    let options = RetrievalOptions::new(2).with_control(control);
    assert!(matches!(
        Retriever::new(LexicalCoverageSource::new(&index)).query("store", &options),
        Err(RetrievalError::Execution(_))
    ));
}
