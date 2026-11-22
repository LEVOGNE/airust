#![cfg(feature = "retrieval")]
use airust::{retrieval::*, Context, Engine, KnowledgeEntry};
use std::collections::BTreeSet;
use unicode_segmentation::UnicodeSegmentation;

fn select<'a>(text: &'a str, query: &str, budget: usize) -> SelectedExcerpt<'a> {
    select_excerpt(
        text,
        query,
        &RankingEvidence::new(RankingKind::Bm25, 1.0),
        &ExcerptAnalyzer::Legacy,
        &ExcerptOptions::default().with_paragraph_boundaries(),
        budget,
    )
    .unwrap()
}
#[test]
fn a_complete_paragraph_replaces_the_surrounding_keyword_window() {
    let text = "Unrelated introduction.\n\nRust protects memory.\n\nUnrelated appendix.";
    let selected = select(text, "Rust memory", 200);
    assert_eq!(selected.text, "Rust protects memory.\n");
    assert_eq!(selected.span.slice(text), Some(selected.text));
    assert!(selected.span.start > 0 && selected.span.end < text.len());
}
#[test]
fn paragraph_scoring_prefers_distinct_terms_then_occurrences_and_earliest_ties() {
    let text = "Rust Rust Rust.\n\nRust memory.\n\nRust memory.\n\nRust memory memory.";
    assert_eq!(select(text, "Rust memory", 100).text, "Rust memory memory.");
    let ties = "Rust memory.\n\nRust memory.";
    assert_eq!(select(ties, "Rust memory", 100).span.start, 0);
}
#[test]
fn an_oversized_paragraph_falls_back_to_the_original_grapheme_safe_window() {
    let text = format!("{} Rust memory {}", "背景👩🏽‍💻 ".repeat(20), "é ".repeat(20));
    let settings = ExcerptOptions::default();
    let evidence = RankingEvidence::new(RankingKind::Bm25, 1.0);
    let old = select_excerpt(
        &text,
        "Rust memory",
        &evidence,
        &ExcerptAnalyzer::Legacy,
        &settings,
        64,
    )
    .unwrap();
    let new = select(&text, "Rust memory", 64);
    assert_eq!(old.text, new.text);
    assert_eq!(old.span, new.span);
}
#[test]
fn phrase_evidence_crossing_paragraphs_is_not_destroyed_by_compaction() {
    let text = "red\n\ncar\n\nred elsewhere.";
    let mut evidence = RankingEvidence::new(RankingKind::TextBm25, 1.0);
    evidence.phrase_spans = vec![ExcerptSpan { start: 0, end: 8 }];
    let selected = select_excerpt(
        text,
        "red car",
        &evidence,
        &ExcerptAnalyzer::Legacy,
        &ExcerptOptions::default().with_paragraph_boundaries(),
        20,
    )
    .unwrap();
    assert!(selected.text.contains("red\n\ncar"));
    assert_eq!(selected.reason, ExcerptReason::Phrase);
}
#[test]
fn unicode_crlf_and_small_budgets_preserve_original_highlight_ranges() {
    let texts = [
        "Intro\r\n\r\nGrüße Rust e\u{301}👩🏽‍💻\r\n\r\nEnd",
        "Rust\n\nRust",
        "\n\nRust\n\n",
        "背景 Rust 🇩🇪\n \t\nRust Rust",
    ];
    for text in texts {
        let boundaries: BTreeSet<_> = text
            .grapheme_indices(true)
            .map(|(p, _)| p)
            .chain(std::iter::once(text.len()))
            .collect();
        for budget in 0..=text.len() + 2 {
            let selected = select(text, "Rust", budget);
            assert_eq!(selected.span.slice(text), Some(selected.text));
            assert!(selected.text.len() <= budget);
            assert!(
                boundaries.contains(&selected.span.start)
                    && boundaries.contains(&selected.span.end)
            );
            for m in selected.matches {
                let relative = m.span.relative_to(selected.span).unwrap();
                assert_eq!(relative.slice(selected.text), m.span.slice(text));
            }
        }
    }
}
#[test]
fn paragraph_sweep_matches_an_independent_exhaustive_block_oracle() {
    let blocks = ["alpha", "beta alpha", "alpha beta beta", "gamma", "beta"];
    for first in blocks {
        for second in blocks {
            for third in blocks {
                let parts = [first, second, third];
                let text = parts.join("\n\n");
                for budget in [0, 1, 4, 5, 8, 12, 16, 24, 64] {
                    let original = select_excerpt(
                        &text,
                        "alpha beta",
                        &RankingEvidence::new(RankingKind::Bm25, 1.0),
                        &ExcerptAnalyzer::Legacy,
                        &ExcerptOptions::default(),
                        budget,
                    )
                    .unwrap();
                    let required: BTreeSet<_> = original
                        .matches
                        .iter()
                        .filter_map(|m| m.term.as_deref())
                        .collect();
                    let mut offset = 0;
                    let mut best = None;
                    let mut score = (0, 0);
                    for (i, p) in parts.iter().enumerate() {
                        let end = offset + p.len() + usize::from(i < 2);
                        let words: Vec<_> = p
                            .split_whitespace()
                            .filter(|w| ["alpha", "beta"].contains(w))
                            .collect();
                        let distinct = words.iter().copied().collect::<BTreeSet<_>>().len();
                        let current = (distinct, words.len());
                        if end - offset <= budget
                            && required.iter().all(|term| words.contains(term))
                            && current > score
                        {
                            score = current;
                            best = Some(ExcerptSpan { start: offset, end });
                        }
                        offset += p.len() + 2;
                    }
                    let actual = select(&text, "alpha beta", budget);
                    if let Some(expected) = best {
                        assert_eq!(actual.span, expected, "{text:?}, budget{budget}");
                    } else {
                        let old = select_excerpt(
                            &text,
                            "alpha beta",
                            &RankingEvidence::new(RankingKind::Bm25, 1.0),
                            &ExcerptAnalyzer::Legacy,
                            &ExcerptOptions::default(),
                            budget,
                        )
                        .unwrap();
                        assert_eq!(actual.span, old.span);
                    }
                }
            }
        }
    }
}
#[test]
fn compaction_preserves_original_window_terms_and_every_retained_phrase_hint() {
    let text = "alpha evidence.\n\nbeta qualification.";
    let original = select_excerpt(
        text,
        "alpha beta",
        &RankingEvidence::new(RankingKind::Bm25, 1.0),
        &ExcerptAnalyzer::Legacy,
        &ExcerptOptions::default(),
        100,
    )
    .unwrap();
    let compact = select(text, "alpha beta", 100);
    assert_eq!(compact.span, original.span);
    assert!(compact.text.contains("alpha") && compact.text.contains("beta"));
    let text = "red car\n\nblue sky";
    let mut evidence = RankingEvidence::new(RankingKind::TextBm25, 1.0);
    evidence.phrase_spans = vec![
        ExcerptSpan { start: 0, end: 7 },
        ExcerptSpan { start: 9, end: 17 },
    ];
    let compact = select_excerpt(
        text,
        "red car blue sky",
        &evidence,
        &ExcerptAnalyzer::Legacy,
        &ExcerptOptions::default().with_paragraph_boundaries(),
        100,
    )
    .unwrap();
    for phrase in &evidence.phrase_spans {
        assert!(phrase.relative_to(compact.span).is_some());
    }
    assert_eq!(compact.text, text);
}
fn duplicate_index() -> Engine {
    Engine::from_entries([
        KnowledgeEntry::new("a", "Rust Rust."),
        KnowledgeEntry::new("b", "Rust Rust."),
        KnowledgeEntry::new("c", "Rust safety explanation."),
    ])
    .unwrap()
}
#[test]
fn distinct_delivery_refills_from_bm25_candidates_and_retains_original_source_ranks() {
    let engine = duplicate_index();
    let plain = Retriever::new(LexicalSource::new(&engine))
        .query("Rust", &RetrievalOptions::new(2))
        .unwrap();
    assert_eq!(plain.hits()[0].excerpt, plain.hits()[1].excerpt);
    assert!(plain.selection().is_none());
    assert!(!plain.to_json().contains("selection"));
    let chosen = Retriever::new(LexicalSource::new(&engine))
        .query("Rust", &RetrievalOptions::new(2).with_distinct_evidence())
        .unwrap();
    assert_eq!(
        chosen
            .hits()
            .iter()
            .map(|h| h.id.as_str())
            .collect::<Vec<_>>(),
        ["a", "c"]
    );
    assert_eq!(chosen.hits()[1].source_rank, 3);
    assert_eq!(chosen.selection().unwrap().considered(), 3);
    assert_eq!(chosen.selection().unwrap().duplicates_skipped(), 1);
    assert_eq!(
        chosen.hits()[0].evidence.score.to_bits(),
        plain.hits()[0].evidence.score.to_bits()
    );
    assert_eq!(chosen.to_json(), serde_json::to_string(&chosen).unwrap());
}
#[test]
fn skipped_duplicates_do_not_spend_delivered_text_budget() {
    let engine = duplicate_index();
    let bytes = "Rust Rust.".len() + "Rust safety explanation.".len();
    let opts = RetrievalOptions::new(2)
        .with_distinct_evidence()
        .with_byte_budgets(1024, bytes, 100, 8192)
        .unwrap();
    let r = Retriever::new(LexicalSource::new(&engine))
        .query("Rust", &opts)
        .unwrap();
    assert_eq!(
        r.hits().iter().map(|h| h.excerpt.len()).sum::<usize>(),
        bytes
    );
    assert!(r.hits().iter().all(|h| !h.text_truncated));
}
#[test]
fn opting_out_restores_identical_legacy_json_and_empty_work_is_explicit() {
    let engine = duplicate_index();
    let mut retriever = Retriever::new(LexicalSource::new(&engine));
    let old = retriever.query("Rust", &RetrievalOptions::new(2)).unwrap();
    let reverted = retriever
        .query(
            "Rust",
            &RetrievalOptions::new(2)
                .with_distinct_evidence()
                .without_distinct_evidence(),
        )
        .unwrap();
    assert_eq!(old.to_json(), reverted.to_json());
    for opts in [
        RetrievalOptions::new(0).with_distinct_evidence(),
        RetrievalOptions::new(2)
            .with_distinct_evidence()
            .with_byte_budgets(1024, 0, 100, 8192)
            .unwrap(),
    ] {
        let r = retriever.query("Rust", &opts).unwrap();
        assert!(r.hits().is_empty());
        assert_eq!(r.selection().unwrap().considered(), 0);
    }
}
#[test]
fn exact_text_deduplication_keeps_negation_and_word_order_distinctions() {
    let engine = Engine::from_entries([
        KnowledgeEntry::new("a", "A supports B"),
        KnowledgeEntry::new("b", "B supports A"),
        KnowledgeEntry::new("c", "A does not support B"),
    ])
    .unwrap();
    let r = Retriever::new(LexicalSource::new(&engine))
        .query("A B", &RetrievalOptions::new(3).with_distinct_evidence())
        .unwrap();
    assert_eq!(r.hits().len(), 3);
    assert_eq!(r.selection().unwrap().duplicates_skipped(), 0);
}
#[test]
fn scope_and_small_output_budget_remain_enforced_after_refill() {
    let engine = duplicate_index();
    let opts = RetrievalOptions::new(2)
        .with_distinct_evidence()
        .with_context(Context::new().with_entry("b").with_entry("c"));
    let r = Retriever::new(LexicalSource::new(&engine))
        .query("Rust", &opts)
        .unwrap();
    assert_eq!(
        r.hits().iter().map(|h| h.id.as_str()).collect::<Vec<_>>(),
        ["b", "c"]
    );
    let tight = opts.with_byte_budgets(1024, 100, 100, 1).unwrap();
    assert!(Retriever::new(LexicalSource::new(&engine))
        .query("Rust", &tight)
        .is_err());
    assert_eq!(engine.len(), 3);
}

#[test]
fn leading_context_retains_qualifications_when_the_complete_entry_fits() {
    let text = "Rust protects memory.\n\nThis does not imply logical correctness.\n\nAppendix.";
    let e = RankingEvidence::new(RankingKind::Bm25, 1.0);
    let s = select_excerpt(
        text,
        "Rust memory",
        &e,
        &ExcerptAnalyzer::Legacy,
        &ExcerptOptions::default().with_leading_context(),
        200,
    )
    .unwrap();
    assert_eq!(s.text, text);
    assert_eq!(
        s.span,
        ExcerptSpan {
            start: 0,
            end: text.len()
        }
    );
}
#[test]
fn leading_context_retains_the_prefix_only_when_query_coverage_is_preserved() {
    let e = RankingEvidence::new(RankingKind::Bm25, 1.0);
    let o = ExcerptOptions::default().with_leading_context();
    let repeated = "Rust context. Intro. Rust context. More material at end.";
    let s = select_excerpt(
        repeated,
        "Rust context",
        &e,
        &ExcerptAnalyzer::Legacy,
        &o,
        32,
    )
    .unwrap();
    assert_eq!(s.span.start, 0);
    assert_eq!(s.text, repeated.get(..32).unwrap());
    let tail = "intro background unrelated material with no matches. Rust memory.";
    let s = select_excerpt(tail, "Rust memory", &e, &ExcerptAnalyzer::Legacy, &o, 18).unwrap();
    assert!(s.span.start > 0);
    assert!(s.text.contains("Rust memory"));
}
#[test]
fn leading_context_preserves_explicit_phrase_locations_and_last_builder_wins() {
    let text = "red car. some background. red car.";
    let mut e = RankingEvidence::new(RankingKind::TextBm25, 1.0);
    e.phrase_spans.push(ExcerptSpan { start: 25, end: 32 });
    let o = ExcerptOptions::default()
        .with_paragraph_boundaries()
        .with_leading_context();
    assert!(o.leading_context());
    assert!(!o.paragraph_boundaries());
    let s = select_excerpt(text, "red car", &e, &ExcerptAnalyzer::Legacy, &o, 12).unwrap();
    assert!(s.span.start > 0);
    assert!(s.matches.iter().any(|m| m.span.start == 25));
    let other = o.with_paragraph_boundaries();
    assert!(!other.leading_context());
    assert!(other.paragraph_boundaries());
}
#[test]
fn leading_context_matches_an_independent_utf8_window_coverage_oracle() {
    let e = RankingEvidence::new(RankingKind::Bm25, 1.0);
    for text in [
        "α Rust\r\n\r\nmemory e\u{301}👩🏽‍💻. Rust memory.",
        "not a guarantee.\n\nRust memory.\n\nRust Rust memory.",
        "Rust\n\n背景\n\nRust",
    ] {
        for budget in 0..=text.len() + 2 {
            let plain = select_excerpt(
                text,
                "Rust memory",
                &e,
                &ExcerptAnalyzer::Legacy,
                &ExcerptOptions::default(),
                budget,
            )
            .unwrap();
            let got = select_excerpt(
                text,
                "Rust memory",
                &e,
                &ExcerptAnalyzer::Legacy,
                &ExcerptOptions::default().with_leading_context(),
                budget,
            )
            .unwrap();
            let prefix_end = text
                .grapheme_indices(true)
                .map(|(i, _)| i)
                .chain([text.len()])
                .filter(|i| *i <= budget)
                .max()
                .unwrap_or(0);
            let prefix = &text[..prefix_end];
            let wanted = airust::text::unique_terms(plain.text);
            let found = airust::text::unique_terms(prefix);
            let retains = ["rust", "memory"]
                .iter()
                .all(|t| !wanted.contains(*t) || found.contains(*t));
            if retains && matches!(plain.reason, ExcerptReason::Terms) {
                assert_eq!(
                    got.span,
                    ExcerptSpan {
                        start: 0,
                        end: prefix_end
                    },
                    "{text:?} {budget}"
                );
            } else if !retains {
                assert_eq!(got.span, plain.span);
            }
            assert!(got.text.len() <= budget);
            assert_eq!(got.span.slice(text), Some(got.text));
            for m in got.matches {
                assert!(m.span.start >= got.span.start && m.span.end <= got.span.end);
                assert!(m.span.slice(text).is_some());
            }
        }
    }
}
