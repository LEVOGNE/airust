use airust::{
    compare, text, Bm25Config, Context, Engine, IndexError, KnowledgeEntry, MatchOptions,
    MatchStrategy, RelationKind, SearchAlgorithm, SearchOptions,
};
use serde_json::json;

fn engine() -> Engine {
    Engine::from_entries([
        KnowledgeEntry::new("a", "rust rust")
            .with_metadata(json!({"project": "alpha", "year": 2026})),
        KnowledgeEntry::new("b", "rust python python").with_metadata(json!({"project": "beta"})),
        KnowledgeEntry::new("c", "python"),
    ])
    .unwrap()
}

#[test]
fn tokenizer_preserves_identifiers_and_separates_punctuation() {
    assert_eq!(
        text::tokenize("Rust-Python; RFC 9110 / V0.1.8"),
        ["rust", "python", "rfc", "9110", "v0", "1", "8"]
    );
    assert_eq!(
        text::tokenize("café cafe\u{301} हिंदी 中文"),
        ["café", "café", "हिंदी", "中文"]
    );
    assert!(text::tokenize("--- 😀 \u{301}").is_empty());
}

#[test]
fn unicode_normalization_keeps_accents_and_punctuation() {
    assert_eq!(text::normalize("  ＲＵＳＴ\tﬁle\nCAFÉ  "), "rust file café");
    assert_eq!(text::normalize("CAFÉ"), text::normalize("cafe\u{301}"));
    assert_ne!(text::normalize("café"), text::normalize("cafe"));
    assert_ne!(text::normalize("Rust?"), text::normalize("Rust!"));
}

#[test]
fn comparison_is_symmetric_and_lexical() {
    assert_eq!(compare("cat", "cats").character, 0.75);
    assert_eq!(compare("cat", "cats"), compare("cats", "cat"));
    assert_eq!(compare("rust python", "python rust").token, 1.0);
    assert_eq!(compare("rust python", "rust go").token, 1.0 / 3.0);
    assert!(compare(" Café ", "cafe\u{301}").exact);
    assert_eq!(compare("", "").character, 0.0);
    assert!(!compare(" ", " ").exact);
    // Different meanings and synonyms require evidence beyond these algorithms.
    assert_eq!(
        compare("Auto anmelden", "Kfz-Zulassung durchführen").token,
        0.0
    );
}

#[test]
fn bm25_agrees_with_hand_calculated_reference() {
    let index = engine();
    let results = index.search("rust", 10);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].entry.id, "a");
    let idf = (1.0_f64 + (3.0 - 2.0 + 0.5) / (2.0 + 0.5)).ln();
    let expected_a = idf * (2.0 * 2.2) / (2.0 + 1.2);
    let expected_b = idf * 2.2 / (1.0 + 1.2 * (0.25 + 0.75 * 3.0 / 2.0));
    assert!((results[0].score - expected_a).abs() < 1e-12);
    assert!((results[1].score - expected_b).abs() < 1e-12);
    assert_eq!(results[0].matched_terms, ["rust"]);
    assert_eq!(results[0].entry.metadata.as_ref().unwrap()["year"], 2026);
    assert_eq!(results[0].score, index.search("rust rust rust", 1)[0].score);
}

#[test]
fn tfidf_is_cosine_similarity_with_smoothed_idf() {
    let index = engine();
    let options = SearchOptions {
        algorithm: SearchAlgorithm::Tfidf,
        ..Default::default()
    };
    let results = index.search_with_options("rust", &options);
    assert_eq!(results[0].entry.id, "a");
    assert!((results[0].score - 1.0).abs() < 1e-12);
    // Both terms have df=2. Their IDF values cancel: dot([1,0],[1,2]) / sqrt(5).
    assert!((results[1].score - 1.0 / 5.0_f64.sqrt()).abs() < 1e-12);
    assert!(results.iter().all(|hit| (0.0..=1.0).contains(&hit.score)));
    assert!(index.search_with_options("unknown", &options).is_empty());
}

#[test]
fn empty_and_unknown_queries_return_no_results() {
    for query in ["", "---", "never-present", "\u{301}"] {
        assert!(engine().search(query, 10).is_empty());
    }
    assert!(Engine::new().search("rust", 10).is_empty());
    assert!(engine().search("rust", 0).is_empty());
}

#[test]
fn ties_and_limits_are_deterministic() {
    let index = Engine::from_entries([
        KnowledgeEntry::new("z", "rust"),
        KnowledgeEntry::new("a", "rust"),
    ])
    .unwrap();
    assert_eq!(index.search("rust", 1)[0].entry.id, "a");
    assert_eq!(index.find_matches("rust", 1)[0].entry.id, "a");
    assert_eq!(index.search("rust", 99).len(), 2);
}

#[test]
fn mutation_removes_stale_postings_and_updates_statistics() {
    let mut index = engine();
    assert_eq!(index.stats().tokens, 6);
    let old = index
        .upsert(KnowledgeEntry::new("a", "golang"))
        .unwrap()
        .unwrap();
    assert_eq!(old.text, "rust rust");
    assert_eq!(index.stats().tokens, 5);
    assert_eq!(index.search("rust", 10)[0].entry.id, "b");
    index.remove("b").unwrap();
    assert!(index.search("rust", 10).is_empty());
    assert_eq!(index.stats().terms, 2);
    assert!(index.remove("missing").is_none());
    index.clear();
    assert_eq!(index.stats().tokens, 0);
    assert_eq!(index.stats().terms, 0);
    assert!(index.is_empty());
}

#[test]
fn invalid_replacement_leaves_previous_data_intact() {
    let mut index = engine();
    let before = index.to_json().unwrap();
    assert!(matches!(
        index.upsert(KnowledgeEntry::new(" ", "valid")),
        Err(IndexError::EmptyId)
    ));
    assert!(matches!(
        index.upsert(KnowledgeEntry::new("a", "\n\t")),
        Err(IndexError::EmptyText(_))
    ));
    assert_eq!(before, index.to_json().unwrap());
}

#[test]
fn import_validates_ids_and_preserves_metadata() {
    let original = engine();
    let restored = Engine::from_json(&original.to_json().unwrap()).unwrap();
    assert_eq!(
        original.entries().collect::<Vec<_>>(),
        restored.entries().collect::<Vec<_>>()
    );
    assert_eq!(
        original.search("rust", 10)[0].score,
        restored.search("rust", 10)[0].score
    );
    assert!(matches!(
        Engine::from_json(r#"[{"id":"a","text":"one"},{"id":"a","text":"two"}]"#),
        Err(IndexError::DuplicateId(_))
    ));
    assert!(Engine::from_json("{}").is_err());
    assert!(Engine::from_json(r#"[{"id":"a","text":" "}]"#).is_err());
}

#[test]
fn incremental_updates_equal_a_fresh_index_for_both_rankers() {
    let mut index = Engine::new();
    for step in 0..90 {
        let id = format!("id-{}", step % 17);
        if step % 5 == 0 {
            index.remove(&id);
        } else {
            let entry = KnowledgeEntry::new(
                id,
                format!("term{} common {}", step % 7, "extra ".repeat(step % 4)),
            );
            index.upsert(entry).unwrap();
        }
        let rebuilt = Engine::from_entries(index.entries().cloned()).unwrap();
        assert_eq!(index.stats(), rebuilt.stats());
        for algorithm in [SearchAlgorithm::Bm25, SearchAlgorithm::Tfidf] {
            let options = SearchOptions {
                algorithm,
                limit: 50,
                ..Default::default()
            };
            let scores = |engine: &Engine| {
                engine
                    .search_with_options("term1 common", &options)
                    .iter()
                    .map(|hit| (hit.entry.id.clone(), hit.score))
                    .collect::<Vec<_>>()
            };
            assert_eq!(scores(&index), scores(&rebuilt), "step {step}");
        }
    }
}

#[test]
fn context_filters_ids_and_exact_metadata_without_changing_scores() {
    let index = engine();
    let context = Context::new()
        .with_entry("a")
        .with_entry("b")
        .with_metadata("project", "alpha");
    let hits = index.search_in_context("rust", &context, 10);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].entry.id, "a");
    assert_eq!(hits[0].score, index.search("rust", 10)[0].score);
    assert_eq!(
        index.search_in_context("rust", &Context::new(), 10).len(),
        2
    );
    assert!(index
        .search_in_context("rust", &Context::new().with_metadata("year", "2026"), 10)
        .is_empty());
    assert!(index
        .search_in_context("rust", &Context::new().with_entry("absent"), 10)
        .is_empty());
}

#[test]
fn matching_exposes_independent_character_and_token_evidence() {
    let index = engine();
    assert_eq!(index.r#match("RUST rust", 1)[0].entry.id, "a");
    let options = MatchOptions {
        strategy: MatchStrategy::TokenOverlap,
        min_similarity: 0.4,
        limit: 10,
    };
    let matches = index.match_with_options("python rust", &options).unwrap();
    assert_eq!(matches[0].entry.id, "b");
    assert_eq!(matches[0].score, 1.0);
    assert!(!matches[0].similarity.exact);
    assert!(index.find_matches("", 10).is_empty());
    assert!(index.find_matches("rust", 0).is_empty());
}

#[test]
fn duplicate_groups_follow_unicode_normalization_and_updates() {
    let mut index = Engine::from_entries([
        KnowledgeEntry::new("1", " Café "),
        KnowledgeEntry::new("2", "CAFE\u{301}"),
        KnowledgeEntry::new("3", "cafe"),
        KnowledgeEntry::new("4", "café!"),
    ])
    .unwrap();
    assert_eq!(
        index.duplicates()[0]
            .iter()
            .map(|e| e.id.as_str())
            .collect::<Vec<_>>(),
        ["1", "2"]
    );
    let exact = MatchOptions {
        strategy: MatchStrategy::Exact,
        min_similarity: 1.0,
        limit: 10,
    };
    assert_eq!(index.match_with_options("café", &exact).unwrap().len(), 2);
    index.upsert(KnowledgeEntry::new("2", "different")).unwrap();
    assert!(index.duplicates().is_empty());
    index.remove("1");
    assert!(index.match_with_options("café", &exact).unwrap().is_empty());
}

#[test]
fn emoji_only_entries_can_match_without_producing_invalid_search_scores() {
    let index = Engine::from_entries([KnowledgeEntry::new("emoji", "😀")]).unwrap();
    assert!(index.search("😀", 10).is_empty());
    assert_eq!(index.find_matches("😀", 1)[0].score, 1.0);
    assert!(index.relate("emoji", 10).unwrap().is_empty());
}

#[test]
fn configuration_rejects_nan_infinity_and_invalid_ranges() {
    for (k1, b) in [
        (0.0, 0.75),
        (-1.0, 0.75),
        (f64::NAN, 0.0),
        (f64::INFINITY, 0.0),
        (1.2, -0.1),
        (1.2, 1.1),
        (1.2, f64::NAN),
    ] {
        assert!(Bm25Config::new(k1, b).is_err());
    }
    for threshold in [-0.1, 1.1, f64::NAN, f64::INFINITY] {
        assert!(engine()
            .match_with_options(
                "rust",
                &MatchOptions {
                    min_similarity: threshold,
                    ..Default::default()
                }
            )
            .is_err());
    }
    for k1 in [f64::MIN_POSITIVE, 1.2, f64::MAX] {
        let options = SearchOptions {
            bm25: Bm25Config::new(k1, 0.75).unwrap(),
            ..Default::default()
        };
        let index = engine();
        let hits = index.search_with_options("rust", &options);
        assert_eq!(hits.len(), 2);
        assert!(hits.iter().all(|hit| hit.score.is_finite()));
    }
}

#[test]
fn relations_are_explained_proposals_and_never_invent_logical_types() {
    let mut index = Engine::from_entries([
        KnowledgeEntry::new("a", "Rust ist sicher"),
        KnowledgeEntry::new("b", "Rust ist unsicher"),
        KnowledgeEntry::new("c", "Python ist praktisch"),
    ])
    .unwrap();
    let before = index.to_json().unwrap();
    let suggestions = index.relate("a", 5).unwrap();
    assert_eq!(suggestions.len(), 1);
    assert_eq!(suggestions[0].target_id, "b");
    assert_eq!(suggestions[0].kind, RelationKind::Related);
    assert_eq!(suggestions[0].shared_terms, ["rust"]);
    assert_eq!(suggestions[0].score, 1.0 / 3.0);
    assert_eq!(before, index.to_json().unwrap());
    assert!(matches!(
        index.relate("missing", 1),
        Err(IndexError::UnknownId(_))
    ));
    assert!(index.relate("a", 0).unwrap().is_empty());
    index.remove("b");
    assert!(index.relate("a", 5).unwrap().is_empty());
}

#[test]
fn engine_and_context_are_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Engine>();
    assert_send_sync::<Context>();
}

#[test]
fn match_candidate_pruning_agrees_with_full_pairwise_comparison() {
    let values = [
        "hello",
        "helo",
        "hellos",
        "äöü",
        "üöä",
        "a",
        "longer sentence",
        "rust python",
        "python rust",
    ];
    let index = Engine::from_entries(
        values
            .iter()
            .enumerate()
            .map(|(i, value)| KnowledgeEntry::new(i.to_string(), *value)),
    )
    .unwrap();
    for query in values {
        for strategy in [
            MatchStrategy::Fuzzy,
            MatchStrategy::TokenOverlap,
            MatchStrategy::Exact,
        ] {
            for threshold in [0.0, 0.5, 0.8, 1.0] {
                let options = MatchOptions {
                    strategy,
                    min_similarity: threshold,
                    limit: 100,
                };
                let mut expected = index
                    .entries()
                    .filter_map(|entry| {
                        let evidence = compare(query, &entry.text);
                        let score = match strategy {
                            MatchStrategy::Exact => {
                                if evidence.exact {
                                    1.0
                                } else {
                                    0.0
                                }
                            }
                            MatchStrategy::Fuzzy => evidence.character,
                            MatchStrategy::TokenOverlap => evidence.token,
                        };
                        (score > 0.0 && score >= threshold).then_some((entry.id.as_str(), score))
                    })
                    .collect::<Vec<_>>();
                expected.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(b.0)));
                let results = index.match_with_options(query, &options).unwrap();
                let actual = results
                    .iter()
                    .map(|hit| (hit.entry.id.as_str(), hit.score))
                    .collect::<Vec<_>>();
                assert_eq!(
                    actual, expected,
                    "query={query:?}, strategy={strategy:?}, threshold={threshold}"
                );
            }
        }
    }
}
