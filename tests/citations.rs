#![cfg(feature = "retrieval")]
use airust::{
    retrieval::{citations::*, *},
    Context, Engine, KnowledgeEntry,
};

fn response(text: &str, query: &str, options: &RetrievalOptions) -> RetrievalResponse {
    let engine = Engine::from_entries([KnowledgeEntry::new("source", text)]).unwrap();
    Retriever::new(LexicalSource::new(&engine))
        .query(query, options)
        .unwrap()
}
fn answer(citations: Vec<Citation>) -> CitedAnswer {
    CitedAnswer::new(
        Some("A caller supplied claim.".into()),
        citations,
        &CitationLimits::default(),
    )
    .unwrap()
}
fn status(response: &RetrievalResponse, citation: Citation) -> CitationStatus {
    CitationRegistry::new(response)
        .check(&answer(vec![citation]), &CitationLimits::default())
        .unwrap()
        .checks()[0]
        .status()
}
#[test]
fn unique_quotes_resolve_to_original_bytes_after_prefix_truncation() {
    let text = format!(
        "{}\n\nGrüße Rust protects memory.\n\n{}",
        "Background ".repeat(100),
        "Appendix ".repeat(100)
    );
    let options = RetrievalOptions::new(1)
        .with_byte_budgets(1024, 80, 80, 8192)
        .unwrap()
        .with_relevant_excerpts(ExcerptOptions::default().with_paragraph_boundaries());
    let r = response(&text, "Rust memory", &options);
    assert!(r.hits()[0].span.start > 0);
    let result = CitationRegistry::new(&r)
        .check(
            &answer(vec![Citation::new("source").with_quote("Grüße Rust")]),
            &CitationLimits::default(),
        )
        .unwrap();
    assert!(result.references_valid() && result.quotes_verified() && !result.abstained());
    let span = result.checks()[0].span().unwrap();
    assert_eq!(span.slice(&text), Some("Grüße Rust"));
    assert_eq!(span.start, text.find("Grüße").unwrap());
    assert_eq!(
        status(&r, Citation::new("source").with_quote("Background")),
        CitationStatus::QuoteNotFound
    );
}
#[test]
fn overlapping_and_repeated_quotes_require_an_explicit_valid_span() {
    let r = response("aaaa Rust Rust", "aaaa", &RetrievalOptions::new(1));
    for quote in ["aaa", "Rust"] {
        assert_eq!(
            status(&r, Citation::new("source").with_quote(quote)),
            CitationStatus::AmbiguousQuote
        );
    }
    assert_eq!(
        status(
            &r,
            Citation::new("source")
                .with_quote("aaa")
                .with_span(ExcerptSpan { start: 1, end: 4 })
        ),
        CitationStatus::QuoteVerified
    );
    assert_eq!(
        status(
            &r,
            Citation::new("source")
                .with_quote("Rust")
                .with_span(ExcerptSpan { start: 10, end: 14 })
        ),
        CitationStatus::QuoteVerified
    );
}
#[test]
fn ids_and_spans_fail_closed_without_silent_normalization() {
    let text = "Grüße Rust";
    let r = response(text, "Rust", &RetrievalOptions::new(1));
    assert_eq!(
        status(&r, Citation::new("/etc/passwd").with_quote("Rust")),
        CitationStatus::UnknownReference
    );
    assert_eq!(
        status(&r, Citation::new("source").with_quote("rust")),
        CitationStatus::QuoteNotFound
    );
    assert_eq!(
        status(&r, Citation::new("source").with_quote("   ")),
        CitationStatus::EmptyQuote
    );
    assert_eq!(
        status(
            &r,
            Citation::new("source").with_span(ExcerptSpan { start: 0, end: 1 })
        ),
        CitationStatus::MissingQuote
    );
    for span in [
        ExcerptSpan { start: 3, end: 4 },
        ExcerptSpan { start: 9, end: 1 },
        ExcerptSpan {
            start: 0,
            end: usize::MAX,
        },
    ] {
        assert_eq!(
            status(
                &r,
                Citation::new("source").with_quote("Rust").with_span(span)
            ),
            CitationStatus::InvalidSpan
        );
    }
    assert_eq!(
        status(
            &r,
            Citation::new("source")
                .with_quote("Rust")
                .with_span(ExcerptSpan { start: 0, end: 2 })
        ),
        CitationStatus::QuoteMismatch
    );
}
#[test]
fn id_existence_and_abstention_never_claim_quote_or_truth_verification() {
    let r = response("Rust works.", "Rust", &RetrievalOptions::new(1));
    let limits = CitationLimits::default();
    let ids = CitationRegistry::new(&r)
        .check(&answer(vec![Citation::new("source")]), &limits)
        .unwrap();
    assert!(ids.references_valid());
    assert!(!ids.quotes_verified());
    let abstain = CitedAnswer::from_json(r#"{"answer":null,"citations":[]}"#, &limits).unwrap();
    let report = CitationRegistry::new(&r).check(&abstain, &limits).unwrap();
    assert!(report.abstained() && report.references_valid());
    assert!(!report.quotes_verified());
    let false_claim = CitedAnswer::new(
        Some("The Moon is made of cheese.".into()),
        vec![Citation::new("source").with_quote("Rust works.")],
        &limits,
    )
    .unwrap();
    assert!(CitationRegistry::new(&r)
        .check(&false_claim, &limits)
        .unwrap()
        .quotes_verified());
}
#[test]
fn duplicates_and_mixed_invalid_references_stay_visible() {
    let r = response("Rust memory safety", "Rust", &RetrievalOptions::new(1));
    let c = Citation::new("source").with_quote("Rust");
    let report = CitationRegistry::new(&r)
        .check(
            &answer(vec![c.clone(), c, Citation::new("unknown")]),
            &CitationLimits::default(),
        )
        .unwrap();
    assert!(!report.references_valid() && !report.quotes_verified());
    assert_eq!(
        report
            .checks()
            .iter()
            .map(CitationCheck::status)
            .collect::<Vec<_>>(),
        vec![
            CitationStatus::QuoteVerified,
            CitationStatus::DuplicateReference,
            CitationStatus::UnknownReference
        ]
    );
    let varied = answer(vec![
        Citation::new("source").with_quote("Rust"),
        Citation::new("source").with_quote("memory"),
    ]);
    assert!(CitationRegistry::new(&r)
        .check(&varied, &CitationLimits::default())
        .unwrap()
        .quotes_verified());
}
#[test]
fn strict_answer_json_rejects_missing_duplicate_unknown_and_wrong_typed_fields() {
    let limits = CitationLimits::default();
    for json in [
        r#"{"citations":[]}"#,
        r#"{"answer":null}"#,
        r#"{"answer":null,"answer":null,"citations":[]}"#,
        r#"{"answer":null,"citations":[],"fetch":"https://example.invalid"}"#,
        r#"{"answer":"null","citations":[]}"#,
        r#"{"answer":" NULL ","citations":[{"id":"source"}]}"#,
        r#"{"answer":" ","citations":[{"id":"source"}]}"#,
        r#"{"answer":"Claim","citations":[]}"#,
        r#"{"answer":null,"citations":[{"id":"source"}]}"#,
        r#"{"answer":"Claim","citations":[{"id":"source","quote":42}]}"#,
        r#"{"answer":"Claim","citations":[{"id":"source","quote":"x","revision":"fake"}]}"#,
        r#"{"answer":"Claim","citations":[{"id":"source","span":{"start":0,"end":1,"page":2}}]}"#,
        r#"{"answer":"Claim","citations":[{"id":"bad\u0000id"}]}"#,
    ] {
        assert!(CitedAnswer::from_json(json, &limits).is_err(), "{json}");
    }
}
#[test]
fn encoded_decoded_byte_and_count_limits_are_enforced_on_every_boundary() {
    let limits = CitationLimits::default();
    assert!(limits.clone().with_input_bytes(0).is_err());
    assert!(limits.clone().with_answer_bytes(0).is_err());
    assert!(limits.clone().with_citations(0).is_err());
    assert!(limits.clone().with_reference_bytes(0, 1).is_err());
    let json = r#"{"answer":"Claim","citations":[{"id":"source","quote":"Rust"}]}"#;
    assert!(matches!(
        CitedAnswer::from_json(
            json,
            &limits.clone().with_input_bytes(json.len() - 1).unwrap()
        ),
        Err(CitationError::Budget(_))
    ));
    assert!(
        CitedAnswer::from_json(json, &limits.clone().with_input_bytes(json.len()).unwrap()).is_ok()
    );
    assert!(CitedAnswer::new(
        Some("éé".into()),
        vec![Citation::new("source")],
        &limits.clone().with_answer_bytes(3).unwrap()
    )
    .is_err());
    assert!(CitedAnswer::new(
        Some("Claim".into()),
        vec![Citation::new("source").with_quote("éé")],
        &limits.clone().with_reference_bytes(10, 3).unwrap()
    )
    .is_err());
    let r = response("Rust", "Rust", &RetrievalOptions::new(1));
    let a = answer(vec![Citation::new("source"), Citation::new("other")]);
    assert!(CitationRegistry::new(&r)
        .check(&a, &limits.clone().with_input_bytes(3).unwrap())
        .is_err());
    assert!(CitationRegistry::new(&r)
        .check(&a, &limits.with_citations(1).unwrap())
        .is_err());
}
#[test]
fn scopes_old_views_and_output_truncation_define_the_allowed_registry() {
    let mut index = Engine::from_entries([
        KnowledgeEntry::new("a", "Rust old proof"),
        KnowledgeEntry::new("b", "Rust other proof"),
        KnowledgeEntry::new("c", "Rust third proof"),
    ])
    .unwrap();
    let options = RetrievalOptions::new(3).with_context(Context::new().with_entry("a"));
    let old = Retriever::new(LexicalSource::new(&index))
        .query("Rust", &options)
        .unwrap();
    assert_eq!(
        status(&old, Citation::new("b")),
        CitationStatus::UnknownReference
    );
    index
        .upsert(KnowledgeEntry::new("a", "Rust NEW proof"))
        .unwrap();
    let fresh = Retriever::new(LexicalSource::new(&index))
        .query("Rust", &options)
        .unwrap();
    assert_eq!(
        status(&old, Citation::new("a").with_quote("old proof")),
        CitationStatus::QuoteVerified
    );
    assert_eq!(
        status(&fresh, Citation::new("a").with_quote("old proof")),
        CitationStatus::QuoteNotFound
    );
    let tight = RetrievalOptions::new(3)
        .with_byte_budgets(1024, 2048, 2048, old.to_json().len() + 64)
        .unwrap();
    let clipped = Retriever::new(LexicalSource::new(&index))
        .query("Rust", &tight)
        .unwrap();
    assert!(clipped.hits().len() < 3 && !clipped.hits().is_empty());
    for id in ["a", "b", "c"] {
        let valid = clipped.hits().iter().any(|h| h.id == id);
        assert_eq!(
            status(&clipped, Citation::new(id)) == CitationStatus::ReferenceOnly,
            valid
        );
    }
}
#[test]
fn malformed_model_output_cannot_mutate_the_retained_response() {
    let r = response("Rust", "Rust", &RetrievalOptions::new(1));
    let before = r.to_json().to_owned();
    for count in 1..150 {
        let input = "[".repeat(count) + &"]".repeat(count);
        assert!(CitedAnswer::from_json(&input, &CitationLimits::default()).is_err());
        let _ = CitationRegistry::new(&r)
            .check(
                &answer(vec![Citation::new("wrong")]),
                &CitationLimits::default(),
            )
            .unwrap();
    }
    assert_eq!(r.to_json(), before);
}
#[test]
fn every_unicode_substring_agrees_with_an_independent_overlapping_occurrence_oracle() {
    for text in ["Rust ababababa", "Rust ééé é", "Rust 👩‍💻👩‍💻 e\u{301}e\u{301}"]
    {
        let r = response(text, "Rust", &RetrievalOptions::new(1));
        let boundaries: Vec<_> = text
            .char_indices()
            .map(|(p, _)| p)
            .chain(std::iter::once(text.len()))
            .collect();
        for (position, start) in boundaries.iter().enumerate() {
            for end in boundaries.iter().skip(position + 1) {
                let quote = &text[*start..*end];
                let count = text
                    .char_indices()
                    .filter(|(p, _)| text[*p..].starts_with(quote))
                    .count();
                let expected = if quote.trim().is_empty() {
                    CitationStatus::EmptyQuote
                } else if count > 1 {
                    CitationStatus::AmbiguousQuote
                } else {
                    CitationStatus::QuoteVerified
                };
                assert_eq!(
                    status(&r, Citation::new("source").with_quote(quote)),
                    expected,
                    "{quote:?}"
                );
                let located = status(
                    &r,
                    Citation::new("source")
                        .with_quote(quote)
                        .with_span(ExcerptSpan {
                            start: *start,
                            end: *end,
                        }),
                );
                assert_eq!(
                    located,
                    if quote.trim().is_empty() {
                        CitationStatus::EmptyQuote
                    } else {
                        CitationStatus::QuoteVerified
                    }
                );
            }
        }
    }
}

#[cfg(feature = "documents")]
#[test]
fn verified_entry_span_projects_only_into_its_retained_document_revision() {
    use airust::documents::{DocumentLimits, DocumentSnapshot, SourceFingerprint, TextSpace};
    let limits = DocumentLimits::default();
    let source = SourceFingerprint::from_bytes(b"source", &limits).unwrap();
    let snapshot = DocumentSnapshot::new(
        "doc",
        source.clone(),
        "v1",
        TextSpace::Supplied,
        "Intro. Grüße Rust memory safety.".into(),
        &limits,
    )
    .unwrap();
    let parent = snapshot.passage(7, snapshot.text().len()).unwrap();
    let entry = parent.to_entry(&snapshot).unwrap();
    parent.validate_entry(&snapshot, &entry).unwrap();
    let id = entry.id.clone();
    let engine = Engine::from_entries([entry]).unwrap();
    let r = Retriever::new(LexicalSource::new(&engine))
        .query("Rust", &RetrievalOptions::new(1))
        .unwrap();
    let report = CitationRegistry::new(&r)
        .check(
            &answer(vec![Citation::new(id).with_quote("Grüße Rust")]),
            &CitationLimits::default(),
        )
        .unwrap();
    let span = report.checks()[0].span().unwrap();
    let passage = parent.subspan(&snapshot, span.start, span.end).unwrap();
    assert_eq!(passage.resolve(&snapshot).unwrap(), "Grüße Rust");
    let replacement = DocumentSnapshot::new(
        "doc",
        source,
        "v2",
        TextSpace::Supplied,
        snapshot.text().to_owned(),
        &limits,
    )
    .unwrap();
    assert!(parent.subspan(&replacement, span.start, span.end).is_err());
}
