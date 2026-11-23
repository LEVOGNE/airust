use airust::{
    documents::{DocumentLimits, DocumentSnapshot, SourceFingerprint, TextSpace},
    retrieval::{citations::*, *},
    Engine, KnowledgeEntry,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(airust::VERSION, "0.4.16");
    let mcp=airust::mcp::McpServer::from_entries([KnowledgeEntry::new("mcp","Rust request deadlines")],airust::mcp::McpLimits{default_timeout_ms:1,..Default::default()})?;
    assert_eq!(mcp.tool_definitions().len(),5);
    assert_eq!(mcp.stats().in_flight,0);
    assert_eq!(mcp.limits().default_timeout_ms,1);
    let covered_index=Engine::from_entries([KnowledgeEntry::new("x","Provenance Store revision identity."),KnowledgeEntry::new("y","ProvenanceStore revision.")])?;
    let opts=RetrievalOptions::new(2);
    let source=LexicalCoverageSource::new(&covered_index);
    let inspected=source.inspect("Which ProvenanceStore revision identity?",&opts)?;
    assert!(inspected.hits().iter().all(|h| h.score().is_finite() && h.query_coverage()<=1.0));
    let covered=Retriever::new(source).query("Which ProvenanceStore revision identity?",&opts)?;
    assert!(covered.recipe().starts_with("airust-lexical-coverage-v1:"));
    assert!(covered.hits().iter().all(|h| h.evidence.lexical_rank.is_some() && h.evidence.semantic_rank.is_none()));
    let question_settings = LexicalQueryOptions::default();
    let trace = question_settings.analyze("Welche ProvenanceStore import limits?")?;
    assert!(trace.added_terms().iter().any(|term| term == "provenance"));
    assert!(trace.removed_terms().iter().any(|term| term == "welche"));
    let question_index = Engine::from_entries([KnowledgeEntry::new(
        "limits", "Provenance store import limits.\n\nThese limits do not establish source truth.",
    )])?;
    let prepared = Retriever::new(LexicalSource::new(&question_index).with_query_analysis(question_settings))
        .query("Welche ProvenanceStore import limits?", &RetrievalOptions::new(1)
            .with_relevant_excerpts(ExcerptOptions::default().with_leading_context()))?;
    assert!(prepared.recipe().starts_with("airust-lexical-query-v1:"));
    assert!(prepared.hits()[0].excerpt.contains("do not establish source truth"));
    let engine = Engine::from_entries([
        KnowledgeEntry::new("a", "Intro.\n\nRust protects memory.\n\nAppendix."),
        KnowledgeEntry::new("b", "Intro.\n\nRust protects memory.\n\nAppendix."),
        KnowledgeEntry::new("c", "Rust ownership explains borrowing."),
    ])?;
    let options = RetrievalOptions::new(2)
        .with_distinct_evidence()
        .with_relevant_excerpts(ExcerptOptions::default().with_paragraph_boundaries());
    let response = Retriever::new(LexicalSource::new(&engine)).query("Rust memory", &options)?;
    assert_eq!(response.hits().len(), 2);
    assert_eq!(response.selection().unwrap().duplicates_skipped(), 1);
    let hit = response
        .hits()
        .iter()
        .find(|h| h.excerpt.contains("protects memory"))
        .unwrap();
    assert_eq!(hit.excerpt, "Rust protects memory.\n");
    let limits = CitationLimits::default();
    let answer = CitedAnswer::new(
        Some("Rust protects memory.".into()),
        vec![Citation::new(&hit.id).with_quote("Rust protects memory.")],
        &limits,
    )?;
    let report = CitationRegistry::new(&response).check(&answer, &limits)?;
    assert!(report.quotes_verified());
    assert_eq!(
        report.checks()[0]
            .span()
            .unwrap()
            .slice(&engine.get(&hit.id).unwrap().text),
        Some("Rust protects memory.")
    );
    let wrong = CitedAnswer::new(
        Some("Claim".into()),
        vec![Citation::new("unseen").with_quote("Rust")],
        &limits,
    )?;
    assert!(!CitationRegistry::new(&response)
        .check(&wrong, &limits)?
        .references_valid());
    assert!(CitedAnswer::from_json(r#"{"answer":"null","citations":[]}"#, &limits).is_err());
    let abstained = CitedAnswer::from_json(r#"{"answer":null,"citations":[]}"#, &limits)?;
    assert!(CitationRegistry::new(&response)
        .check(&abstained, &limits)?
        .abstained());
    let dl = DocumentLimits::default();
    let source = SourceFingerprint::from_bytes(b"source", &dl)?;
    let snapshot = DocumentSnapshot::new(
        "doc",
        source,
        "recipe",
        TextSpace::Supplied,
        "Grüße Rust".into(),
        &dl,
    )?;
    let parent = snapshot.passage(0, snapshot.text().len())?;
    let entry = parent.to_entry(&snapshot)?;
    let id = entry.id.clone();
    parent.validate_entry(&snapshot, &entry)?;
    let index = Engine::from_entries([entry])?;
    let r = Retriever::new(LexicalSource::new(&index)).query("Rust", &RetrievalOptions::new(1))?;
    let a = CitedAnswer::new(
        Some("Rust".into()),
        vec![Citation::new(id).with_quote("Rust")],
        &limits,
    )?;
    let receipt = CitationRegistry::new(&r).check(&a, &limits)?;
    let span = receipt.checks()[0].span().unwrap();
    assert_eq!(
        parent
            .subspan(&snapshot, span.start, span.end)?
            .resolve(&snapshot)?,
        "Rust"
    );
    println!(
        "{}",
        serde_json::json!({"version":airust::VERSION,"independent_consumer":"passed","scope":"BM25 selection, UTF8 quotation, rejected unknown source, strict null, immutable document projection"})
    );
    Ok(())
}
