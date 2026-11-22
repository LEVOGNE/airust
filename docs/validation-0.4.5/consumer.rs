use airust::{
    documents::{DocumentLimits, DocumentSnapshot, SourceFingerprint, TextSpace},
    retrieval::{citations::*, *},
    Engine, KnowledgeEntry,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(airust::VERSION, "0.4.5");
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
