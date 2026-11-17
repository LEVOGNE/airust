#![cfg(feature = "advanced-search")]
use airust::{
    text_search::{
        AnalyzerConfig, Stemming, TermOperator, TextAnalyzer, TextIndex, TextSearchError,
        TextSearchOptions, WordTokenizer,
    },
    Context, Engine, KnowledgeEntry, SearchAlgorithm, SearchOptions,
};
use serde_json::json;
fn analyze(text: &str) -> Vec<airust::text_search::Token> {
    TextAnalyzer::new(AnalyzerConfig::default())
        .analyze(text)
        .unwrap()
}
fn index() -> TextIndex {
    TextIndex::from_entries(
        AnalyzerConfig::default(),
        [
            KnowledgeEntry::new("a", "red bright car"),
            KnowledgeEntry::new("b", "red car red car"),
            KnowledgeEntry::new("c", "car red"),
            KnowledgeEntry::new("d", "red car blue bird").with_metadata(json!({"scope":"private"})),
        ],
    )
    .unwrap()
}
fn ids(index: &TextIndex, query: &str, options: &TextSearchOptions) -> Vec<String> {
    index
        .search(query, options)
        .unwrap()
        .into_iter()
        .map(|r| r.entry.id.clone())
        .collect()
}
#[test]
fn unicode_words_preserve_apostrophes_decimals_and_original_spans() {
    let text = "Äpfel l’homme can't 32.3 東京 👩‍💻";
    let tokens = analyze(text);
    let terms: Vec<_> = tokens.iter().map(|t| t.term.as_str()).collect();
    assert!(terms.contains(&"l'homme"));
    assert!(terms.contains(&"can't"));
    assert!(terms.contains(&"32.3"));
    for (i, t) in tokens.iter().enumerate() {
        assert_eq!(t.position, i);
        assert!(t.span.slice(text).is_some());
        assert!(t.span.end > t.span.start);
    }
}
#[test]
fn compatibility_expansion_combining_marks_and_utf8_ranges_stay_in_original_coordinates() {
    for text in [
        "ﬃ ㎏ ½ ＡＢＣ",
        "Fu\u{308}hrerschein",
        "İSTANBUL",
        "ㄱㅏ",
        "a\u{301}\u{327}",
    ] {
        let tokens = analyze(text);
        assert!(!tokens.is_empty());
        for t in tokens {
            let source = t.span.slice(text).unwrap();
            assert!(!source.is_empty());
            assert!(analyze(source).iter().any(|r| r.term == t.term));
        }
    }
    assert_eq!(analyze("ㄱㅏ")[0].term, analyze("가")[0].term);
}
#[test]
fn tokenizer_choice_and_apostrophe_folding_are_explicit() {
    let source = "l’homme foo_bar 32.3";
    let mut cfg = AnalyzerConfig::default().with_tokenizer(WordTokenizer::Alphanumeric);
    let a = TextAnalyzer::new(cfg.clone()).analyze(source).unwrap();
    assert_eq!(
        a.iter().map(|t| t.term.as_str()).collect::<Vec<_>>(),
        ["l", "homme", "foo", "bar", "32", "3"]
    );
    cfg = cfg
        .with_tokenizer(WordTokenizer::UnicodeWords)
        .with_apostrophe_folding(false);
    assert_eq!(
        TextAnalyzer::new(cfg).analyze(source).unwrap()[0].term,
        "l’homme"
    );
}
#[test]
fn german_noun_forms_match_only_when_selected() {
    let german = TextIndex::from_entries(
        AnalyzerConfig::german(),
        [KnowledgeEntry::new(
            "a",
            "Häusern Führerscheine Katzen Straßen",
        )],
    )
    .unwrap();
    for query in ["Haus", "Führerschein", "Katze", "Straße"] {
        assert_eq!(ids(&german, query, &TextSearchOptions::new(5)), ["a"]);
    }
    let plain =
        TextIndex::from_entries(AnalyzerConfig::default(), german.entries().cloned()).unwrap();
    assert!(ids(&plain, "Haus", &TextSearchOptions::new(5)).is_empty());
    let a = TextAnalyzer::new(AnalyzerConfig::german());
    assert_ne!(
        a.analyze("gehen").unwrap()[0].term,
        a.analyze("ging").unwrap()[0].term
    );
}
#[test]
fn required_phrase_checks_order_adjacency_and_runs_before_top_k() {
    let i = index();
    let hits = i.search("\"red car\"", &TextSearchOptions::new(1)).unwrap();
    assert_eq!(hits.len(), 1);
    assert!(matches!(hits[0].entry.id.as_str(), "b" | "d"));
    assert!(!ids(&i, "\"red car\"", &TextSearchOptions::new(10)).contains(&"a".into()));
    assert_eq!(ids(&i, "\"car red\"", &TextSearchOptions::new(10)).len(), 2);
}
#[test]
fn all_phrases_and_unquoted_term_operator_are_enforced() {
    let i = index();
    assert_eq!(
        ids(&i, "\"red car\" \"blue bird\"", &TextSearchOptions::new(10)),
        ["d"]
    );
    assert_eq!(
        ids(&i, "\"red car\" blue unknown", &TextSearchOptions::new(10)),
        ["d"]
    );
    assert!(ids(
        &i,
        "\"red car\" blue unknown",
        &TextSearchOptions::new(10).with_operator(TermOperator::All)
    )
    .is_empty());
}
#[test]
fn phrase_evidence_is_bounded_reports_truncation_and_keeps_overlaps() {
    let i = TextIndex::from_entries(
        AnalyzerConfig::default(),
        [KnowledgeEntry::new("x", "a a a a")],
    )
    .unwrap();
    let hits = i
        .search(
            "\"a a\"",
            &TextSearchOptions::new(1).with_max_spans(2).unwrap(),
        )
        .unwrap();
    let p = &hits[0].phrases[0];
    assert_eq!(p.spans.len(), 2);
    assert!(p.truncated);
    assert_eq!(
        p.spans
            .iter()
            .map(|s| s.slice(&hits[0].entry.text).unwrap())
            .collect::<Vec<_>>(),
        ["a a", "a a"]
    );
    assert_eq!(p.spans[1].start, 2);
}
#[test]
fn malformed_quoted_queries_and_query_budgets_fail_explicitly() {
    let i = index();
    for q in ["", "  ", "\"red", "\"\"", "\"!!!\""] {
        assert!(
            matches!(
                i.search(q, &TextSearchOptions::new(5)),
                Err(TextSearchError::Query(_))
            ),
            "{q}"
        );
    }
    for (q, o) in [
        (
            "red",
            TextSearchOptions::new(2)
                .with_query_limits(2, 5, 5, 5)
                .unwrap(),
        ),
        (
            "red car",
            TextSearchOptions::new(2)
                .with_query_limits(100, 1, 5, 5)
                .unwrap(),
        ),
        (
            "\"red\" \"car\"",
            TextSearchOptions::new(2)
                .with_query_limits(100, 5, 1, 5)
                .unwrap(),
        ),
        (
            "\"red car\"",
            TextSearchOptions::new(2)
                .with_query_limits(100, 5, 5, 1)
                .unwrap(),
        ),
    ] {
        assert!(matches!(i.search(q, &o), Err(TextSearchError::Limit(_))));
    }
    assert!(i
        .search("!!!", &TextSearchOptions::new(5))
        .unwrap()
        .is_empty());
}
#[test]
fn literal_backslashes_and_escaped_quotes_do_not_become_unclosed_phrases() {
    let i = TextIndex::from_entries(
        AnalyzerConfig::default(),
        [KnowledgeEntry::new("a", r#"C:\Users\alice says "hello""#)],
    )
    .unwrap();
    assert_eq!(
        ids(&i, r#"C:\Users\alice"#, &TextSearchOptions::new(5)),
        ["a"]
    );
    assert_eq!(ids(&i, r#"\"hello\""#, &TextSearchOptions::new(5)), ["a"]);
}
#[test]
fn scope_precedes_top_k_and_excluded_phrase_matches_do_not_reenter() {
    let i = index();
    let o = TextSearchOptions::new(1).with_context(Context::new().with_entry("b"));
    assert_eq!(ids(&i, "\"red car\"", &o), ["b"]);
    assert!(ids(
        &i,
        "\"red car\"",
        &o.with_context(Context::new().with_entry("missing"))
    )
    .is_empty());
}
#[test]
fn stemming_phrases_keep_original_text_instead_of_stems() {
    let i = TextIndex::from_entries(
        AnalyzerConfig::german(),
        [KnowledgeEntry::new("a", "Großen Häusern helfen Gärten.")],
    )
    .unwrap();
    let h = i
        .search("\"große Häuser\"", &TextSearchOptions::new(5))
        .unwrap();
    assert_eq!(h.len(), 1);
    assert_eq!(
        h[0].phrases[0].spans[0].slice(&h[0].entry.text),
        Some("Großen Häusern")
    );
}
#[test]
fn failed_replacement_leaves_both_source_postings_and_positions_unchanged() {
    let cfg = AnalyzerConfig::default().with_limits(30, 100, 3).unwrap();
    let mut i = TextIndex::from_entries(cfg, [KnowledgeEntry::new("a", "red car")]).unwrap();
    let before = i.stats();
    for text in ["x".repeat(31), "a b c d".into(), " ".into()] {
        assert!(i.upsert(KnowledgeEntry::new("a", text)).is_err());
        assert_eq!(i.stats(), before);
        assert_eq!(ids(&i, "\"red car\"", &TextSearchOptions::new(3)), ["a"]);
    }
    i.upsert(KnowledgeEntry::new("a", "blue bird")).unwrap();
    assert!(ids(&i, "\"red car\"", &TextSearchOptions::new(3)).is_empty());
    assert_eq!(i.remove("a").unwrap().text, "blue bird");
    assert!(i.tokens("a").is_none());
    i.clear();
    assert_eq!(i.stats().tokens, 0);
}
#[test]
fn analysis_limits_reject_expansion_and_token_counts_without_partial_output() {
    for (text, cfg) in [
        (
            "ﬃ",
            AnalyzerConfig::default().with_limits(100, 2, 10).unwrap(),
        ),
        (
            "a b c",
            AnalyzerConfig::default().with_limits(100, 100, 2).unwrap(),
        ),
        (
            "a b",
            AnalyzerConfig::default().with_limits(2, 100, 10).unwrap(),
        ),
    ] {
        assert!(matches!(
            TextAnalyzer::new(cfg).analyze(text),
            Err(TextSearchError::Limit(_))
        ));
    }
    assert!(AnalyzerConfig::default().with_limits(0, 1, 1).is_err());
    assert!(TextSearchOptions::new(1).with_max_spans(0).is_err());
    assert!(TextSearchOptions::new(1)
        .with_query_limits(1, 0, 1, 1)
        .is_err());
}
#[test]
fn shared_legacy_alphanumeric_scores_equal_engine_without_changing_default_analyzer() {
    let records = [
        KnowledgeEntry::new("a", "Rust search memory"),
        KnowledgeEntry::new("b", "Rust owns memory"),
    ];
    let legacy = Engine::from_entries(records.clone()).unwrap();
    let advanced = TextIndex::from_entries(
        AnalyzerConfig::default()
            .with_tokenizer(WordTokenizer::Alphanumeric)
            .with_stemming(Stemming::None),
        records,
    )
    .unwrap();
    for algorithm in [SearchAlgorithm::Bm25, SearchAlgorithm::Tfidf] {
        let options = SearchOptions {
            algorithm,
            limit: 10,
            ..SearchOptions::default()
        };
        let old = legacy.search_with_options("Rust memory", &options);
        let new = advanced
            .search(
                "Rust memory",
                &TextSearchOptions::new(10).with_search(options),
            )
            .unwrap();
        assert_eq!(old.len(), new.len());
        for (a, b) in old.iter().zip(new) {
            assert_eq!(a.entry.id, b.entry.id);
            assert_eq!(a.score.to_bits(), b.score.to_bits());
            assert_eq!(a.matched_terms, b.matched_terms);
        }
    }
    assert_eq!(airust::text::tokenize("can't"), ["can", "t"]);
}
#[test]
fn incremental_updates_equal_rebuilding_positions_and_rankings() {
    let mut i = index();
    for step in 0..150 {
        let id = format!("m{}", step % 13);
        if step % 4 == 0 {
            i.remove(&id);
        } else {
            i.upsert(KnowledgeEntry::new(
                id,
                format!("red {} car red car", step % 7),
            ))
            .unwrap();
        }
        let fresh =
            TextIndex::from_entries(AnalyzerConfig::default(), i.entries().cloned()).unwrap();
        assert_eq!(i.stats(), fresh.stats());
        for q in ["red", "\"red car\"", "\"car red\"", "\"red 2\""] {
            let a = i.search(q, &TextSearchOptions::new(20)).unwrap();
            let b = fresh.search(q, &TextSearchOptions::new(20)).unwrap();
            assert_eq!(
                serde_json::to_value(a).unwrap(),
                serde_json::to_value(b).unwrap()
            );
        }
    }
}
#[test]
fn phrase_algorithm_agrees_with_independent_window_reference_for_seeded_sequences() {
    let mut seed = 5721u64;
    for case in 0..500 {
        let mut words = Vec::new();
        for _ in 0..(case % 41) {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            words.push(["a", "b", "c"][(seed >> 32) as usize % 3]);
        }
        let text = words.join(" ");
        if text.is_empty() {
            continue;
        }
        let i =
            TextIndex::from_entries(AnalyzerConfig::default(), [KnowledgeEntry::new("x", text)])
                .unwrap();
        for phrase in [["a", "a"], ["b", "c"], ["c", "a"]] {
            let expected = words.windows(2).filter(|w| *w == phrase).count();
            let q = format!("\"{}\"", phrase.join(" "));
            let hits = i
                .search(&q, &TextSearchOptions::new(1).with_max_spans(100).unwrap())
                .unwrap();
            assert_eq!(hits.len(), usize::from(expected > 0));
            if expected > 0 {
                assert_eq!(hits[0].phrases[0].spans.len(), expected);
            }
        }
    }
}
#[cfg(feature = "reranking")]
#[test]
fn advanced_results_can_be_reranked_without_copying_the_source() {
    struct P;
    impl airust::reranking::Reranker for P {
        fn recipe(&self) -> &str {
            "test"
        }
        fn score(
            &mut self,
            _: &str,
            docs: &[&str],
        ) -> Result<Vec<f64>, airust::reranking::RerankError> {
            Ok(vec![1.0; docs.len()])
        }
    }
    let i = index();
    let candidates = i
        .search("\"red car\"", &TextSearchOptions::new(10))
        .unwrap();
    let result = airust::reranking::rerank(
        "red car",
        &candidates,
        &mut P,
        &airust::reranking::RerankOptions::new(10),
    )
    .unwrap();
    assert!(std::ptr::eq(
        result.results()[0].entry(),
        candidates[0].entry
    ));
    assert!(!result.results()[0].candidate.phrases.is_empty());
}

#[test]
fn seeded_unicode_sequences_never_produce_invalid_or_unbounded_original_spans() {
    let alphabet = [
        'a', 'é', '\u{301}', '\u{327}', 'İ', 'ß', 'ﬃ', '㎏', '½', '\u{FDFA}', 'Ａ', 'ㄱ', 'ㅏ',
        '東', '\u{200D}', '👩', ' ', '\n', '’', '\u{0}',
    ];
    let mut seed = 88307u64;
    for case in 0..600 {
        let mut text = String::new();
        for _ in 0..case % 61 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            text.push(alphabet[(seed >> 32) as usize % alphabet.len()]);
        }
        for tokenizer in [WordTokenizer::UnicodeWords, WordTokenizer::Alphanumeric] {
            let analyzer = TextAnalyzer::new(AnalyzerConfig::german().with_tokenizer(tokenizer));
            let tokens = analyzer.analyze(&text).unwrap();
            for (i, token) in tokens.iter().enumerate() {
                assert_eq!(token.position, i);
                assert!(!token.term.is_empty());
                assert!(token.span.start < token.span.end);
                assert!(token.span.slice(&text).is_some());
                if i > 0 {
                    assert!(token.span.start >= tokens[i - 1].span.start);
                }
            }
        }
    }
}
#[test]
fn invalid_user_constructed_spans_return_none_instead_of_panicking() {
    use airust::text_search::TextSpan;
    for span in [
        TextSpan { start: 1, end: 2 },
        TextSpan { start: 3, end: 1 },
        TextSpan {
            start: 0,
            end: usize::MAX,
        },
    ] {
        assert_eq!(span.slice("é"), None);
    }
}

#[test]
fn a_higher_scoring_nonphrase_or_excluded_entry_cannot_hide_the_valid_top_one() {
    let records = [
        KnowledgeEntry::new("wrong", "car red red red red").with_metadata(json!({"allowed":false})),
        KnowledgeEntry::new("right", format!("red car {}", "padding ".repeat(60)))
            .with_metadata(json!({"allowed":true})),
    ];
    let legacy = Engine::from_entries(records.clone()).unwrap();
    assert_eq!(legacy.search("red car", 1)[0].entry.id, "wrong");
    let advanced = TextIndex::from_entries(AnalyzerConfig::default(), records).unwrap();
    assert_eq!(
        ids(&advanced, "\"red car\"", &TextSearchOptions::new(1)),
        ["right"]
    );
    assert_eq!(
        ids(
            &advanced,
            "red car",
            &TextSearchOptions::new(1).with_context(Context::new().with_metadata("allowed", true))
        ),
        ["right"]
    );
}
