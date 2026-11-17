#![cfg(feature = "retrieval")]
use airust::{retrieval::*, Context, Engine, KnowledgeEntry};
use std::collections::BTreeSet;
use unicode_segmentation::UnicodeSegmentation;
fn evidence() -> RankingEvidence {
    RankingEvidence::new(RankingKind::Bm25, 1.0)
}
fn select<'a>(text: &'a str, query: &str, bytes: usize) -> SelectedExcerpt<'a> {
    select_excerpt(
        text,
        query,
        &evidence(),
        &ExcerptAnalyzer::Legacy,
        &ExcerptOptions::default(),
        bytes,
    )
    .unwrap()
}
fn verify(text: &str, result: &SelectedExcerpt<'_>, bytes: usize) {
    let boundaries: BTreeSet<_> = text
        .grapheme_indices(true)
        .map(|(p, _)| p)
        .chain(std::iter::once(text.len()))
        .collect();
    assert_eq!(result.span.slice(text), Some(result.text));
    assert!(result.text.len() <= bytes);
    assert!(boundaries.contains(&result.span.start) && boundaries.contains(&result.span.end));
    for m in &result.matches {
        assert!(boundaries.contains(&m.span.start) && boundaries.contains(&m.span.end));
        let relative = m.span.relative_to(result.span).unwrap();
        assert_eq!(relative.slice(result.text), m.span.slice(text));
    }
}
#[test]
fn late_hits_replace_an_unrelated_prefix_and_preserve_exact_original_bytes() {
    let text = format!(
        "{}\nThe specific answer is Rust memory safety.\n{}",
        "An unrelated introduction. ".repeat(150),
        "Appendix. ".repeat(10)
    );
    let index = Engine::from_entries([KnowledgeEntry::new("long", &text)]).unwrap();
    let base = RetrievalOptions::new(1)
        .with_byte_budgets(1024, 90, 90, 8192)
        .unwrap();
    let prefix = Retriever::new(LexicalSource::new(&index))
        .query("Rust memory safety", &base)
        .unwrap();
    assert_eq!(prefix.hits()[0].span.start, 0);
    assert!(!prefix.hits()[0].excerpt.contains("Rust"));
    assert!(prefix.excerpt_details().is_none());
    assert!(!prefix.to_json().contains("excerpt_details"));
    let response = Retriever::new(LexicalSource::new(&index))
        .query(
            "Rust memory safety",
            &base.with_relevant_excerpts(ExcerptOptions::default()),
        )
        .unwrap();
    let h = &response.hits()[0];
    assert!(h.span.start > 3000);
    assert!(h.excerpt.contains("Rust memory safety"));
    assert_eq!(h.span.slice(&text), Some(h.excerpt.as_str()));
    assert_eq!(
        response.excerpt_details().unwrap()[0].reason,
        ExcerptReason::Terms
    );
    assert_eq!(response.excerpt_details().unwrap()[0].matches.len(), 3);
    assert_eq!(
        serde_json::to_string(&response).unwrap(),
        response.to_json()
    );
    assert_eq!(
        h.evidence.score.to_bits(),
        prefix.hits()[0].evidence.score.to_bits()
    );
}
#[test]
fn distinct_terms_beat_repetitions_and_equal_windows_are_deterministic() {
    let text = format!(
        "{}{}alpha beta gamma{}alpha beta gamma",
        "alpha ".repeat(10),
        " gap ".repeat(30),
        " gap ".repeat(30)
    );
    let result = select(&text, "alpha beta gamma", 30);
    assert!(result.text.contains("alpha beta gamma"));
    assert!(result.span.start < text.rfind("alpha beta gamma").unwrap());
    assert_eq!(result.text, select(&text, "alpha beta gamma", 30).text);
    verify(&text, &result, 30);
}
#[test]
fn complete_phrase_evidence_has_priority_over_a_broader_word_cluster() {
    let text = "alpha beta gamma delta. A long separating paragraph here. alpha beta.";
    let start = text.rfind("alpha beta").unwrap();
    let mut evidence = evidence();
    evidence.phrase_spans.push(ExcerptSpan {
        start,
        end: start + 10,
    });
    let result = select_excerpt(
        text,
        "alpha beta gamma delta",
        &evidence,
        &ExcerptAnalyzer::Legacy,
        &ExcerptOptions::default(),
        24,
    )
    .unwrap();
    assert_eq!(result.reason, ExcerptReason::Phrase);
    assert!(result.span.start > 30);
    assert!(result
        .matches
        .iter()
        .any(|m| m.kind == ExcerptMatchKind::Phrase && m.span.start == start));
    verify(text, &result, 24);
}
#[test]
fn missing_lexical_evidence_and_too_wide_matches_have_distinct_fallbacks() {
    let missing = select("This document has no literal match.", "zebra", 12);
    assert_eq!(missing.reason, ExcerptReason::NoLexicalMatch);
    assert_eq!(missing.span.start, 0);
    assert!(missing.matches.is_empty());
    let wide = select("extraordinarilylongword tail", "extraordinarilylongword", 5);
    assert_eq!(wide.reason, ExcerptReason::NoFittingMatch);
    assert!(wide.matches.is_empty());
    let empty = select("🦀 body", "body", 0);
    assert_eq!(empty.reason, ExcerptReason::EmptyBudget);
    verify("🦀 body", &empty, 0);
}
#[test]
fn compatibility_expansion_combining_marks_and_apostrophes_keep_source_coordinates() {
    for (text, query) in [
        ("An introduction. ＣＡＦＥ and a tail.", "cafe"),
        ("First. cafe\u{301} is here.", "café"),
        ("First. ﬃ is here.", "ffi"),
        ("First. ㍿ is here.", "株式会社"),
        ("First. Rust’s ownership.", "ownership"),
    ] {
        let result = select(text, query, 28);
        assert_eq!(result.reason, ExcerptReason::Terms, "{text}");
        assert!(result.matches.iter().all(|m| m.span.slice(text).is_some()));
        verify(text, &result, 28);
    }
}
#[test]
fn flags_zwj_crlf_and_large_graphemes_are_never_split() {
    let text = "🇩🇪 👨‍👩‍👧‍👦\r\nA cafe\u{301} rust 🧑‍🚀 end";
    for bytes in 0..70 {
        let result = select(text, "rust café", bytes);
        verify(text, &result, bytes);
    }
    let huge = format!("a{} rust", "\u{301}".repeat(10000));
    let result = select(&huge, "rust", 16);
    assert_eq!(result.text, " rust");
    verify(&huge, &result, 16);
    let missing = select(&huge, "absent", 16);
    assert!(missing.text.is_empty());
    verify(&huge, &missing, 16);
}
#[test]
fn malformed_phrase_hints_are_errors_and_do_not_become_fake_matches() {
    for span in [
        ExcerptSpan { start: 0, end: 0 },
        ExcerptSpan { start: 2, end: 3 },
        ExcerptSpan { start: 0, end: 999 },
        ExcerptSpan { start: 1, end: 3 },
    ] {
        let mut e = evidence();
        e.phrase_spans.push(span);
        assert!(matches!(
            select_excerpt(
                "a\u{301} rust",
                "rust",
                &e,
                &ExcerptAnalyzer::Legacy,
                &ExcerptOptions::default(),
                20
            ),
            Err(RetrievalError::Output(_))
        ));
    }
}
#[test]
fn analysis_query_match_and_normalization_budgets_fail_explicitly() {
    let settings = ExcerptOptions::default()
        .with_analysis_limits(3, 30, 10)
        .unwrap();
    assert!(matches!(
        select_excerpt(
            "long text",
            "x",
            &evidence(),
            &ExcerptAnalyzer::Legacy,
            &settings,
            10
        ),
        Err(RetrievalError::Budget("excerpt input bytes"))
    ));
    let settings = ExcerptOptions::default()
        .with_analysis_limits(100, 4, 10)
        .unwrap();
    assert!(matches!(
        select_excerpt(
            "㍿",
            "x",
            &evidence(),
            &ExcerptAnalyzer::Legacy,
            &settings,
            10
        ),
        Err(RetrievalError::Budget("excerpt normalized bytes"))
    ));
    let settings = ExcerptOptions::default()
        .with_analysis_limits(100, 100, 2)
        .unwrap();
    assert!(select_excerpt(
        "x x x",
        "x",
        &evidence(),
        &ExcerptAnalyzer::Legacy,
        &settings,
        10
    )
    .is_err());
    let settings = ExcerptOptions::default().with_query_limits(2, 1).unwrap();
    assert!(select_excerpt(
        "body",
        "long",
        &evidence(),
        &ExcerptAnalyzer::Legacy,
        &settings,
        10
    )
    .is_err());
    let mut e = evidence();
    e.matched_terms = vec!["a".into(), "b".into()];
    assert!(select_excerpt("a b", "a", &e, &ExcerptAnalyzer::Legacy, &settings, 10).is_err());
    let settings = ExcerptOptions::default().with_match_limits(2, 1).unwrap();
    assert!(matches!(
        select_excerpt(
            "x x x",
            "x",
            &evidence(),
            &ExcerptAnalyzer::Legacy,
            &settings,
            10
        ),
        Err(RetrievalError::Budget("excerpt match count"))
    ));
}
#[test]
fn bounded_highlights_keep_the_full_window_and_disclose_omissions() {
    let text = "rust rust rust rust";
    let result = select_excerpt(
        text,
        "rust",
        &evidence(),
        &ExcerptAnalyzer::Legacy,
        &ExcerptOptions::default().with_match_limits(20, 2).unwrap(),
        30,
    )
    .unwrap();
    assert_eq!(result.text, text);
    assert_eq!(result.matches.len(), 2);
    assert!(result.matches_truncated);
    verify(text, &result, 30);
}
#[test]
fn invalid_options_and_relative_positions_are_checked() {
    assert!(ExcerptOptions::default()
        .with_analysis_limits(0, 1, 1)
        .is_err());
    assert!(ExcerptOptions::default().with_query_limits(1, 0).is_err());
    assert!(ExcerptOptions::default().with_match_limits(1, 0).is_err());
    assert!(ExcerptSpan { start: 3, end: 1 }
        .relative_to(ExcerptSpan { start: 0, end: 5 })
        .is_none());
    assert!(ExcerptSpan { start: 0, end: 2 }
        .relative_to(ExcerptSpan { start: 1, end: 3 })
        .is_none());
    assert!(ExcerptSpan { start: 1, end: 99 }.slice("rust").is_none());
    assert!(select_excerpt(
        "rust",
        " ",
        &evidence(),
        &ExcerptAnalyzer::Legacy,
        &ExcerptOptions::default(),
        0
    )
    .is_err());
}
#[test]
fn json_output_tail_removal_keeps_hit_and_excerpt_details_aligned() {
    let index = Engine::from_entries([
        KnowledgeEntry::new("a", "Prefix intro. Rust memory safety."),
        KnowledgeEntry::new("b", "Prefix intro. Rust ownership memory."),
    ])
    .unwrap();
    for bytes in (1..5000).step_by(19) {
        let options = RetrievalOptions::new(2)
            .with_relevant_excerpts(ExcerptOptions::default())
            .with_byte_budgets(100, 90, 45, bytes)
            .unwrap();
        match Retriever::new(LexicalSource::new(&index)).query("Rust memory", &options) {
            Ok(response) => {
                assert!(response.to_json().len() <= bytes);
                let parsed: serde_json::Value = serde_json::from_str(response.to_json()).unwrap();
                assert_eq!(
                    parsed["hits"].as_array().unwrap().len(),
                    parsed["excerpt_details"].as_array().unwrap().len()
                );
                let details = response.excerpt_details().unwrap();
                assert_eq!(details.len(), response.hits().len());
                for (h, d) in response.hits().iter().zip(details) {
                    assert_eq!(h.id, d.id);
                    assert!(d
                        .matches
                        .iter()
                        .all(|m| m.span.relative_to(h.span).is_some()));
                }
            }
            Err(e) => assert!(matches!(e, RetrievalError::Budget("output envelope bytes"))),
        }
    }
}
#[test]
fn relevant_and_reset_prefix_modes_preserve_old_json_and_shared_text_budgets() {
    let index = Engine::from_entries([
        KnowledgeEntry::new("a", "🦀éabc Rust"),
        KnowledgeEntry::new("b", "🦀éabc Rust"),
    ])
    .unwrap();
    let options = RetrievalOptions::new(2)
        .with_byte_budgets(100, 7, 5, 10000)
        .unwrap();
    let mut r = Retriever::new(LexicalSource::new(&index));
    let original = r.query("Rust", &options).unwrap();
    let reset = r
        .query(
            "Rust",
            &options
                .clone()
                .with_relevant_excerpts(ExcerptOptions::default())
                .with_prefix_excerpts(),
        )
        .unwrap();
    assert_eq!(original.to_json(), reset.to_json());
    let relevant = r
        .query(
            "Rust",
            &options.with_relevant_excerpts(ExcerptOptions::default()),
        )
        .unwrap();
    assert!(
        relevant
            .hits()
            .iter()
            .map(|h| h.excerpt.len())
            .sum::<usize>()
            <= 7
    );
}
#[test]
fn scope_excludes_large_denied_records_before_excerpt_analysis() {
    let index = Engine::from_entries([
        KnowledgeEntry::new("a", "Rust safety"),
        KnowledgeEntry::new("private", "Rust ".repeat(1000)),
    ])
    .unwrap();
    let options = RetrievalOptions::new(1)
        .with_context(Context::new().with_entry("a"))
        .with_relevant_excerpts(
            ExcerptOptions::default()
                .with_analysis_limits(100, 100, 30)
                .unwrap(),
        );
    let response = Retriever::new(LexicalSource::new(&index))
        .query("Rust", &options)
        .unwrap();
    assert_eq!(response.hits()[0].id, "a");
}
#[test]
fn seeded_mixed_unicode_excerpts_and_highlights_always_round_trip() {
    let fragments = [
        "a",
        "é",
        "e\u{301}",
        "ß",
        "ﬃ",
        "㍿",
        "🦀",
        "👩‍🚀",
        "🇩🇪",
        "\r\n",
        "東京",
        " ",
        "rust",
        "\u{600}a",
    ];
    let mut seed = 31u64;
    for _ in 0..600 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let text = (0..35)
            .map(|i| fragments[((seed >> ((i % 8) * 8)) as usize + i) % fragments.len()])
            .collect::<Vec<_>>()
            .join(" ");
        let bytes = (seed as usize) % 140;
        let result = select(&text, "rust é ffi 株式会社 a", bytes);
        verify(&text, &result, bytes);
    }
}
#[cfg(feature = "advanced-search")]
#[test]
fn german_stems_and_analyzer_configuration_survive_borrowed_dynamic_sources() {
    use airust::text_search::{AnalyzerConfig, TextIndex};
    let text = format!(
        "{} Die Häuser stehen neben schönen Gärten.",
        "Einleitung. ".repeat(70)
    );
    let index =
        TextIndex::from_entries(AnalyzerConfig::german(), [KnowledgeEntry::new("de", &text)])
            .unwrap();
    let mut source: Box<dyn RetrievalSource> = Box::new(TextSource::new(&index));
    let options = RetrievalOptions::new(1)
        .with_byte_budgets(100, 70, 70, 10000)
        .unwrap()
        .with_relevant_excerpts(ExcerptOptions::default());
    let response = Retriever::new(&mut source)
        .query("Haus Garten", &options)
        .unwrap();
    let h = &response.hits()[0];
    assert!(h.span.start > 700);
    assert!(h.excerpt.contains("Häuser") && h.excerpt.contains("Gärten"));
    let detail = &response.excerpt_details().unwrap()[0];
    assert!(detail
        .matches
        .iter()
        .any(|m| m.span.slice(&text) == Some("Häuser")));
    assert!(detail
        .matches
        .iter()
        .any(|m| m.span.slice(&text) == Some("Gärten")));
    let phrase = Retriever::new(TextSource::new(&index))
        .query("\"schönen Garten\"", &options)
        .unwrap();
    assert_eq!(
        phrase.excerpt_details().unwrap()[0].reason,
        ExcerptReason::Phrase
    );
}
#[cfg(feature = "reranking")]
#[test]
fn reranking_sees_full_records_and_excerpt_details_follow_its_new_order() {
    use airust::reranking::{RerankError, RerankOptions, Reranker};
    struct Ranker;
    impl Reranker for Ranker {
        fn recipe(&self) -> &str {
            "excerpt-order-test"
        }
        fn score(&mut self, _: &str, docs: &[&str]) -> Result<Vec<f64>, RerankError> {
            assert!(docs.iter().all(|d| d.len() > 500));
            Ok(docs
                .iter()
                .map(|d| if d.contains("special") { 5.0 } else { 0.0 })
                .collect())
        }
    }
    let index = Engine::from_entries([
        KnowledgeEntry::new(
            "a",
            format!("{} Rust memory safety.", "Introduction. ".repeat(50)),
        ),
        KnowledgeEntry::new(
            "b",
            format!("{} Rust special memory.", "Introduction. ".repeat(50)),
        ),
    ])
    .unwrap();
    let response = Retriever::new(LexicalSource::new(&index))
        .with_reranker(&mut Ranker, RerankOptions::new(2))
        .query(
            "Rust memory",
            &RetrievalOptions::new(2)
                .with_relevant_excerpts(ExcerptOptions::default())
                .with_byte_budgets(100, 100, 50, 10000)
                .unwrap(),
        )
        .unwrap();
    assert_eq!(response.hits()[0].id, "b");
    assert_eq!(response.hits()[0].rerank_score, Some(5.0));
    for (h, d) in response
        .hits()
        .iter()
        .zip(response.excerpt_details().unwrap())
    {
        assert_eq!(h.id, d.id);
        assert!(h.excerpt.contains("Rust"));
        assert!(h.span.start > 500);
    }
}
#[cfg(feature = "embeddings")]
#[test]
fn semantic_paraphrases_without_word_overlap_report_no_lexical_localization() {
    use airust::semantic::{
        EmbeddingError, EmbeddingProvider, EmbeddingPurpose, EmbeddingSpace, SemanticIndex,
    };
    struct Provider(EmbeddingSpace);
    impl EmbeddingProvider for Provider {
        fn space(&self) -> &EmbeddingSpace {
            &self.0
        }
        fn embed(
            &mut self,
            texts: &[&str],
            _: EmbeddingPurpose,
        ) -> Result<Vec<Vec<f32>>, EmbeddingError> {
            Ok(texts.iter().map(|_| vec![1.0, 0.0]).collect())
        }
    }
    let mut p = Provider(EmbeddingSpace::new("handwritten-paraphrase", 2).unwrap());
    let index = SemanticIndex::from_index(
        Engine::from_entries([KnowledgeEntry::new("de", "Kfz-Zulassung durchführen.")]).unwrap(),
        &mut p,
        8,
    )
    .unwrap();
    let response = Retriever::new(SemanticSource::new(&index, &mut p))
        .query(
            "register a vehicle",
            &RetrievalOptions::new(1).with_relevant_excerpts(ExcerptOptions::default()),
        )
        .unwrap();
    assert_eq!(response.hits()[0].evidence.kind, RankingKind::Cosine);
    assert_eq!(
        response.excerpt_details().unwrap()[0].reason,
        ExcerptReason::NoLexicalMatch
    );
    assert!(response.excerpt_details().unwrap()[0].matches.is_empty());
}

#[test]
fn highlight_omissions_also_mark_the_complete_response_truncated() {
    let index = Engine::from_entries([KnowledgeEntry::new("a", "rust rust rust")]).unwrap();
    let options = RetrievalOptions::new(1)
        .with_relevant_excerpts(ExcerptOptions::default().with_match_limits(10, 1).unwrap());
    let response = Retriever::new(LexicalSource::new(&index))
        .query("rust", &options)
        .unwrap();
    assert!(!response.hits()[0].text_truncated);
    assert!(response.excerpt_details().unwrap()[0].matches_truncated);
    assert!(response.truncated());
}

#[test]
fn matches_at_document_edges_use_available_context_without_splitting_words_as_evidence() {
    let text = format!("{} rust", "background ".repeat(30));
    let selected = select(&text, "rust", 80);
    assert_eq!(selected.text.len(), 80);
    assert_eq!(selected.span.end, text.len());
    verify(&text, &selected, 80);
    let selected = select("rust short", "rust", 100);
    assert_eq!(selected.text, "rust short");
    let mut e = evidence();
    e.matched_terms = vec!["x".repeat(6), "y".repeat(6)];
    let options = ExcerptOptions::default()
        .with_analysis_limits(100, 10, 10)
        .unwrap();
    assert!(matches!(
        select_excerpt("rust", "r", &e, &ExcerptAnalyzer::Legacy, &options, 20),
        Err(RetrievalError::Budget("excerpt term bytes"))
    ));
    e.matched_terms = vec![String::new()];
    assert!(matches!(
        select_excerpt("rust", "r", &e, &ExcerptAnalyzer::Legacy, &options, 20),
        Err(RetrievalError::Output("invalid excerpt term"))
    ));
}
