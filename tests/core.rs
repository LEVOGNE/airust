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
fn ascii_normalization_matches_general_unicode_reference_and_whitespace() {
    use unicode_normalization::UnicodeNormalization;
    fn reference(source: &str) -> String {
        source
            .nfkc()
            .flat_map(char::to_lowercase)
            .nfkc()
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }
    let all_ascii: String = (0..=127).map(char::from).collect();
    let mut cases = vec![
        String::new(),
        all_ascii.clone(),
        all_ascii.chars().rev().collect(),
        " \t\r\n\u{b}\u{c} ".into(),
        "A\u{b}B\u{c}C\r\nD\0E".into(),
        "  ＲＵＳＴ\u{a0}cafe\u{301}\u{2003}İ\tEND ".into(),
    ];
    let mut seed = 19u64;
    for _ in 0..64 {
        cases.push(
            (0..48)
                .map(|_| {
                    seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                    char::from((seed >> 32) as u8 & 127)
                })
                .collect(),
        );
    }
    for source in cases {
        let normalized = text::normalize(&source);
        assert_eq!(normalized, reference(&source));
        assert_eq!(text::normalize(&normalized), normalized);
    }
    assert_eq!(text::normalize("A\u{b}B"), "a b");
    assert_eq!(text::normalize("A\u{1c}B"), "a\u{1c}b");
}

#[test]
fn merged_and_wide_postings_preserve_complete_scoped_candidate_sets() {
    use std::collections::BTreeSet;
    let documents = if cfg!(miri) { 6 } else { 96 };
    let mut index = Engine::from_entries((0..documents).map(|n| {
        let body = (0..68)
            .filter(|t| (n + t) % 5 < 2)
            .map(|t| format!("term{t} "))
            .collect::<String>();
        KnowledgeEntry::new(format!("prefix/ä/{n:03}"), body)
            .with_metadata(json!({"scope": if n%2==0 {"even"} else {"odd"}}))
    }))
    .unwrap();
    for count in [0, 1, 2, 7, 32, 33, 64] {
        let query = (0..count).map(|n| format!("term{n} ")).collect::<String>() + " missing";
        let wanted: BTreeSet<_> = text::tokenize(&query).into_iter().collect();
        for scope in [None, Some(Context::new().with_metadata("scope", "even"))] {
            let expected: BTreeSet<_> = index
                .entries()
                .filter(|e| scope.as_ref().is_none_or(|s| s.includes(e)))
                .filter(|e| {
                    text::tokenize(&e.text)
                        .iter()
                        .any(|word| wanted.contains(word))
                })
                .map(|e| e.id.as_str())
                .collect();
            for algorithm in [SearchAlgorithm::Bm25, SearchAlgorithm::Tfidf] {
                let hits = index.search_with_options(
                    &query,
                    &SearchOptions {
                        limit: 96,
                        algorithm,
                        context: scope.clone(),
                        ..Default::default()
                    },
                );
                assert_eq!(
                    hits.iter()
                        .map(|h| h.entry.id.as_str())
                        .collect::<BTreeSet<_>>(),
                    expected
                );
                assert_eq!(hits.len(), expected.len());
                for hit in hits {
                    let words: BTreeSet<_> = text::tokenize(&hit.entry.text).into_iter().collect();
                    assert_eq!(
                        hit.matched_terms.iter().collect::<BTreeSet<_>>(),
                        words.intersection(&wanted).collect()
                    );
                }
            }
        }
    }
    index.remove("prefix/ä/000");
    index
        .upsert(KnowledgeEntry::new("prefix/ä/001", "term0 term67"))
        .unwrap();
    assert_eq!(
        index
            .search("term0 term67", 96)
            .iter()
            .map(|h| h.entry.id.as_str())
            .collect::<BTreeSet<_>>(),
        index
            .entries()
            .filter(|e| text::tokenize(&e.text)
                .iter()
                .any(|t| t == "term0" || t == "term67"))
            .map(|e| e.id.as_str())
            .collect()
    );
}

#[test]
fn single_known_term_keeps_scores_ties_scopes_and_limits_on_small_indices() {
    let sizes: &[usize] = if cfg!(miri) {
        &[1, 4]
    } else {
        &[1, 4, 16, 32, 33]
    };
    for &size in sizes {
        let index = Engine::from_entries((0..size).rev().map(|n| {
            KnowledgeEntry::new(format!("資料/{n:03}"), "café")
                .with_metadata(json!({"group": n % 2}))
        }))
        .unwrap();
        for algorithm in [SearchAlgorithm::Bm25, SearchAlgorithm::Tfidf] {
            let idf = (1.0 + 0.5 / (size as f64 + 0.5)).ln();
            let expected_score = match algorithm {
                SearchAlgorithm::Bm25 => idf * (1.0 + 1.0 / 1.2) / (1.0 + 1.0 / 1.2),
                SearchAlgorithm::Tfidf => 1.0,
            };
            for context in [None, Some(Context::new().with_metadata("group", 0))] {
                let expected: Vec<_> = index
                    .entries()
                    .filter(|e| context.as_ref().is_none_or(|c| c.includes(e)))
                    .map(|e| e.id.as_str())
                    .collect();
                for limit in [0, 1, size + 1] {
                    let options = SearchOptions {
                        limit,
                        algorithm,
                        context: context.clone(),
                        ..Default::default()
                    };
                    for _ in 0..2 {
                        let hits = index.search_with_options("CAFÉ cafe\u{301} absent", &options);
                        assert_eq!(
                            hits.iter().map(|h| h.entry.id.as_str()).collect::<Vec<_>>(),
                            expected.iter().copied().take(limit).collect::<Vec<_>>()
                        );
                        for hit in hits {
                            assert_eq!(hit.score.to_bits(), expected_score.to_bits());
                            assert_eq!(hit.matched_terms, ["café"]);
                        }
                    }
                    assert!(index.search_with_options("absent", &options).is_empty());
                }
            }
        }
    }
}

#[test]
fn frequencies_reuse_normalized_keys_without_losing_repetitions_or_unicode() {
    use std::collections::BTreeMap;
    let mut expected = BTreeMap::new();
    for (term, count) in [("rust", 3), ("café", 2), ("हिंदी", 2), ("中文", 2)] {
        expected.insert(term.to_owned(), count);
    }
    assert_eq!(
        text::term_frequencies("ＲＵＳＴ rust RUST café cafe\u{301} हिंदी हिंदी 中文 中文 🦀"),
        expected
    );
    let repeated = "cafe\u{301} ＲＵＳＴ ".repeat(2048);
    assert_eq!(
        text::term_frequencies(&repeated),
        BTreeMap::from([("café".to_owned(), 2048), ("rust".to_owned(), 2048)])
    );
    assert!(text::term_frequencies("\u{301} ... 🦀").is_empty());
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

    assert_eq!(compare("Car registration", "Vehicle licensing").token, 0.0);
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

    assert!((results[1].score - 1.0 / 5.0_f64.sqrt()).abs() < 1e-12);
    assert!(results.iter().all(|hit| (0.0..=1.0).contains(&hit.score)));
    assert!(index.search_with_options("unknown", &options).is_empty());
}

#[test]
fn wide_sparse_scores_match_full_scan_bits_for_both_rankers_and_cache_states() {
    use std::collections::BTreeMap;
    let count = if cfg!(miri) { 4 } else { 1024 };
    let mut records: Vec<_> = (0..count)
        .map(|n| {
            let width = [4, 16, 17, 40][n % 4];
            KnowledgeEntry::new(
                format!("prefix/é/{n:03}"),
                "common ".to_owned()
                    + &(0..width)
                        .map(|j| format!("term{} ", (n * 7 + j) % 256))
                        .collect::<String>(),
            )
            .with_metadata(json!({"scope": if n%2==0 {"even"}else{"odd"}}))
        })
        .collect();
    records.push(KnowledgeEntry::new(
        "vocabulary",
        "common ".to_owned() + &(0..256).map(|n| format!("term{n} ")).collect::<String>(),
    ));
    let index = Engine::from_entries(records).unwrap();
    let frequencies: BTreeMap<_, _> = index
        .entries()
        .map(|e| (e.id.clone(), text::term_frequencies(&e.text)))
        .collect();
    let mut df = BTreeMap::<String, usize>::new();
    for words in frequencies.values() {
        for word in words.keys() {
            *df.entry(word.clone()).or_default() += 1;
        }
    }
    let sizes: &[usize] = if cfg!(miri) {
        &[0, 1, 3, 32, 33, 129]
    } else {
        &[0, 1, 2, 3, 32, 33, 64, 129, 256]
    };
    for size in sizes {
        let query = if *size == 0 {
            "common common unsupported".to_owned()
        } else {
            (0..*size).map(|n| format!("term{n} ")).collect::<String>()
                + " common term0 term0 unsupported"
        };
        let terms = text::term_frequencies(&query);
        let known: Vec<_> = terms
            .iter()
            .filter(|(word, _)| df.contains_key(*word))
            .collect();
        for algorithm in [SearchAlgorithm::Bm25, SearchAlgorithm::Tfidf] {
            for context in [None, Some(Context::new().with_metadata("scope", "even"))] {
                let options = SearchOptions {
                    limit: 100,
                    algorithm,
                    context,
                    ..Default::default()
                };
                let idf = |word: &str| match algorithm {
                    SearchAlgorithm::Bm25 => (1.0
                        + (index.len() as f64 - df[word] as f64 + 0.5) / (df[word] as f64 + 0.5))
                        .ln(),
                    SearchAlgorithm::Tfidf => {
                        ((index.len() as f64 + 1.0) / (df[word] as f64 + 1.0)).ln() + 1.0
                    }
                };
                let query_norm = known
                    .iter()
                    .map(|(word, n)| (**n as f64 * idf(word)).powi(2))
                    .sum::<f64>()
                    .sqrt();
                let mut expected = Vec::new();
                for entry in index
                    .entries()
                    .filter(|e| options.context.as_ref().is_none_or(|c| c.includes(e)))
                {
                    let words = &frequencies[&entry.id];
                    let mut score = 0.0;
                    let mut matched = Vec::new();
                    let length: usize = words.values().sum();
                    let norm = 1.0 - options.bm25.b()
                        + options.bm25.b() * length as f64
                            / (index.stats().tokens as f64 / index.len() as f64);
                    for (word, n) in &known {
                        if let Some(tf) = words.get(*word) {
                            matched.push((*word).clone());
                            let weight = idf(word);
                            let tf = *tf as f64;
                            score += match algorithm {
                                SearchAlgorithm::Bm25 => {
                                    weight * tf * (1.0 + 1.0 / options.bm25.k1())
                                        / (norm + tf / options.bm25.k1())
                                }
                                SearchAlgorithm::Tfidf => **n as f64 * tf * weight * weight,
                            };
                        }
                    }
                    if algorithm == SearchAlgorithm::Tfidf {
                        let doc_norm = words
                            .iter()
                            .map(|(word, n)| (*n as f64 * idf(word)).powi(2))
                            .sum::<f64>()
                            .sqrt();
                        score = if doc_norm > 0.0 && query_norm > 0.0 {
                            (score / (doc_norm * query_norm)).clamp(0.0, 1.0)
                        } else {
                            0.0
                        };
                    }
                    if score > 0.0 {
                        expected.push((entry.id.clone(), score, matched));
                    }
                }
                expected.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
                let expected: Vec<_> = expected
                    .into_iter()
                    .map(|(id, score, words)| (id, score.to_bits(), words))
                    .collect();
                for limit in [100, index.len()] {
                    let options = SearchOptions {
                        limit,
                        ..options.clone()
                    };
                    for _ in 0..2 {
                        let actual: Vec<_> = index
                            .search_with_options(&query, &options)
                            .into_iter()
                            .map(|h| (h.entry.id.clone(), h.score.to_bits(), h.matched_terms))
                            .collect();
                        assert_eq!(actual.as_slice(), &expected[..expected.len().min(limit)]);
                    }
                }
            }
        }
    }
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
        KnowledgeEntry::new("a", "Rust is safe"),
        KnowledgeEntry::new("b", "Rust is unsafe"),
        KnowledgeEntry::new("c", "Python is practical"),
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

#[test]
fn bounded_rankings_preserve_full_order_and_large_limits_do_not_preallocate() {
    let index = Engine::from_entries((0..200).rev().map(|id| {
        KnowledgeEntry::new(
            format!("{id:04}"),
            format!("shared rust {}", "rust ".repeat(id % 5)),
        )
    }))
    .unwrap();
    for algorithm in [SearchAlgorithm::Bm25, SearchAlgorithm::Tfidf] {
        let all = index.search_with_options(
            "rust",
            &SearchOptions {
                algorithm,
                limit: usize::MAX,
                ..Default::default()
            },
        );
        for limit in [0, 1, 3, 17, 200, 300] {
            let subset = index.search_with_options(
                "rust",
                &SearchOptions {
                    algorithm,
                    limit,
                    ..Default::default()
                },
            );
            assert_eq!(
                subset
                    .iter()
                    .map(|hit| (&hit.entry.id, hit.score))
                    .collect::<Vec<_>>(),
                all.iter()
                    .take(limit)
                    .map(|hit| (&hit.entry.id, hit.score))
                    .collect::<Vec<_>>()
            );
        }
    }
    for limit in [1, 7, 200] {
        let all = index.find_matches("shared rust rust", usize::MAX);
        let subset = index.find_matches("shared rust rust", limit);
        assert_eq!(
            subset.iter().map(|hit| &hit.entry.id).collect::<Vec<_>>(),
            all.iter()
                .take(limit)
                .map(|hit| &hit.entry.id)
                .collect::<Vec<_>>()
        );
        let all = index.relate("0000", usize::MAX).unwrap();
        assert_eq!(
            index.relate("0000", limit).unwrap(),
            all.into_iter().take(limit).collect::<Vec<_>>()
        );
    }
}
