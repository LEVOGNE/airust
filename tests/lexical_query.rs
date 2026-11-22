#![cfg(feature = "retrieval")]
use airust::{retrieval::*, Context, Engine, KnowledgeEntry, SearchOptions};

#[test]
fn query_preparation_is_explicit_and_preserves_original_identifiers_and_negation() {
    let options = LexicalQueryOptions::default();
    let a = options
        .analyze("Welche HTTP2Server Optionen sind nicht optional?")
        .unwrap();
    assert_eq!(
        a.effective_query(),
        "http2server optionen nicht optional http 2 server"
    );
    assert_eq!(a.added_terms(), ["http", "2", "server"]);
    assert_eq!(a.removed_terms(), ["welche", "sind"]);
    assert!(!a.original_fallback());
    for term in [
        "not", "nicht", "no", "kein", "keine", "never", "ohne", "only",
    ] {
        assert!(options
            .analyze(&format!("Which {term} record?"))
            .unwrap()
            .effective_query()
            .split(' ')
            .any(|t| t == term));
    }
}
#[test]
fn quoted_scoped_and_underscore_terms_keep_meaningful_function_word_names() {
    let o = LexicalQueryOptions::default();
    for query in [
        "which `at` method",
        "which \"with\" method",
        "which Point::at method",
        "which with_budget method",
        "which `with method",
    ] {
        let a = o.analyze(query).unwrap();
        let expected = if query.contains("at") && !query.contains("budget") {
            "at"
        } else {
            "with"
        };
        assert!(
            a.effective_query().split(' ').any(|t| t == expected),
            "{query}: {}",
            a.effective_query()
        );
    }
    let a = o.analyze("with \"at `nested` with\" method").unwrap();
    assert!(a.effective_query().contains("at"));
}
#[test]
fn empty_filter_falls_back_and_punctuation_has_no_terms() {
    let o = LexicalQueryOptions::default();
    let a = o.analyze("is the and").unwrap();
    assert!(a.original_fallback());
    assert!(a.removed_terms().is_empty());
    assert_eq!(a.effective_query(), "is the and");
    assert_eq!(o.analyze("...! 🦀").unwrap().effective_query(), "");
    assert!(o.analyze("  \n").is_err());
}
#[test]
fn identifier_split_boundaries_preserve_original_unicode_and_opaque_names() {
    let o = LexicalQueryOptions::default().without_word_filter();
    for (input, parts) in [
        ("XMLParser", vec!["xml", "parser"]),
        ("HTTP2Server", vec!["http", "2", "server"]),
        ("foo42Bar", vec!["foo", "42", "bar"]),
        ("ProvenanceStore", vec!["provenance", "store"]),
    ] {
        let a = o.analyze(input).unwrap();
        assert_eq!(a.added_terms(), parts);
        assert!(a.effective_query().starts_with(&input.to_lowercase()));
    }
    assert!(o
        .analyze("überHTTPServer")
        .unwrap()
        .added_terms()
        .is_empty());
    assert_eq!(
        o.analyze("ＨＴＴＰServer").unwrap().effective_query(),
        "httpserver http server"
    );
    assert_eq!(
        o.analyze("Cafe\u{301}HTTP").unwrap().effective_query(),
        "caféhttp"
    );
    assert_eq!(
        o.clone()
            .without_identifier_parts()
            .analyze("HTTP2Server")
            .unwrap()
            .effective_query(),
        "http2server"
    );
}
#[test]
fn every_budget_rejects_instead_of_silently_truncating() {
    let o = LexicalQueryOptions::default();
    for args in [(0, 1, 1), (1, 0, 1), (1, 1, 0)] {
        assert!(o.clone().with_limits(args.0, args.1, args.2).is_err());
    }
    assert!(o
        .clone()
        .with_limits(2, 32, 32)
        .unwrap()
        .analyze("Rust")
        .is_err());
    assert!(o
        .clone()
        .with_limits(128, 3, 32)
        .unwrap()
        .analyze("Rust")
        .is_err());
    assert!(o
        .clone()
        .with_limits(128, 128, 2)
        .unwrap()
        .analyze("one two three")
        .is_err());
    assert!(o
        .clone()
        .with_limits(128, 128, 2)
        .unwrap()
        .analyze("HTTP2Server")
        .is_err());
    let exact = o
        .clone()
        .with_limits(4, 4, 1)
        .unwrap()
        .analyze("Rust")
        .unwrap();
    assert_eq!(exact.effective_query(), "rust");
    assert_eq!(
        o.analyze("Rust Rust Rust").unwrap().effective_query(),
        "rust"
    );
    let expansion = "A1".repeat(5000);
    assert!(o.analyze(&expansion).is_ok());
}
#[test]
fn source_scores_match_an_independent_explicit_query_and_keep_scope() {
    let engine = Engine::from_entries([
        KnowledgeEntry::new("target", "Project notes. Provenance Store import limits.")
            .with_metadata(serde_json::json!({"tenant":"a"})),
        KnowledgeEntry::new("literal", "ProvenanceStore constructor.")
            .with_metadata(serde_json::json!({"tenant":"a"})),
        KnowledgeEntry::new("other", "Provenance Store import limits.")
            .with_metadata(serde_json::json!({"tenant":"b"})),
    ])
    .unwrap();
    let settings = LexicalQueryOptions::default();
    let q = "Welche ProvenanceStore import limits?";
    let analyzed = settings.analyze(q).unwrap();
    let request =
        RetrievalOptions::new(2).with_context(Context::new().with_metadata("tenant", "a"));
    let mut source = LexicalSource::new(&engine).with_query_analysis(settings);
    let batch = source.retrieve(q, &request).unwrap();
    let oracle = engine.search_with_options(
        analyzed.effective_query(),
        &SearchOptions {
            limit: request.candidates(),
            context: request.context().cloned(),
            ..Default::default()
        },
    );
    assert_eq!(batch.candidates.len(), oracle.len());
    for (a, b) in batch.candidates.iter().zip(oracle) {
        assert_eq!(a.entry.id, b.entry.id);
        assert_eq!(a.evidence.score.to_bits(), b.score.to_bits());
        assert_eq!(a.evidence.matched_terms, b.matched_terms);
    }
    assert!(batch.recipe.starts_with("airust-lexical-query-v1:"));
    assert!(batch.candidates.iter().all(|c| c.entry.id != "other"));
}
#[test]
fn source_reuse_has_no_stale_query_and_conflicting_scopes_return_nothing() {
    let engine = Engine::from_entries([
        KnowledgeEntry::new("a", "Provenance Store.")
            .with_metadata(serde_json::json!({"tenant":"a"})),
        KnowledgeEntry::new("b", "HTTP Server.").with_metadata(serde_json::json!({"tenant":"b"})),
    ])
    .unwrap();
    let source = LexicalSource::new(&engine)
        .with_query_analysis(LexicalQueryOptions::default())
        .with_options(SearchOptions {
            context: Some(Context::new().with_metadata("tenant", "a")),
            ..Default::default()
        });
    let mut r = Retriever::new(source);
    assert_eq!(
        r.query("ProvenanceStore", &RetrievalOptions::new(2))
            .unwrap()
            .hits()[0]
            .id,
        "a"
    );
    assert!(r
        .query("HTTPServer", &RetrievalOptions::new(2))
        .unwrap()
        .hits()
        .is_empty());
    let denied = r
        .query(
            "ProvenanceStore",
            &RetrievalOptions::new(2).with_context(Context::new().with_metadata("tenant", "b")),
        )
        .unwrap();
    assert!(denied.hits().is_empty());
    assert!(denied.recipe().starts_with("airust-lexical-query-v1:"));
}
#[test]
fn literal_source_and_default_engine_keep_identical_json() {
    let e = Engine::from_entries([
        KnowledgeEntry::new("a", "Which Rust at server?"),
        KnowledgeEntry::new("b", "Rust servers"),
    ])
    .unwrap();
    let q = "Which Rust at server?";
    let opts = RetrievalOptions::new(2);
    let a = Retriever::new(LexicalSource::new(&e))
        .query(q, &opts)
        .unwrap();
    let b = Retriever::new(LexicalSource::new(&e).with_options(SearchOptions::default()))
        .query(q, &opts)
        .unwrap();
    assert_eq!(a.to_json(), b.to_json());
}
#[cfg(feature = "execution")]
#[test]
fn cancelled_preparation_does_not_publish_partial_results() {
    use airust::execution::WorkControl;
    let e = Engine::from_entries([KnowledgeEntry::new("a", "HTTP Server")]).unwrap();
    let control = WorkControl::default();
    control.token().cancel();
    assert!(Retriever::new(
        LexicalSource::new(&e).with_query_analysis(LexicalQueryOptions::default())
    )
    .query(
        "HTTPServer",
        &RetrievalOptions::new(1).with_control(control)
    )
    .is_err());
}
