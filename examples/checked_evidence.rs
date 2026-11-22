use airust::{
    retrieval::{
        citations::{Citation, CitationLimits, CitationRegistry, CitedAnswer},
        ExcerptOptions, LexicalSource, RetrievalOptions, Retriever,
    },
    Engine, KnowledgeEntry,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = Engine::from_entries([
        KnowledgeEntry::new(
            "manual",
            "Introduction.\n\nRust protects memory.\n\nAppendix.",
        ),
        KnowledgeEntry::new(
            "copy",
            "Other introduction.\n\nRust protects memory.\n\nOther appendix.",
        ),
        KnowledgeEntry::new("detail", "Rust ownership checks borrowing."),
    ])?;
    let options = RetrievalOptions::new(3)
        .with_candidates(20)?
        .with_relevant_excerpts(ExcerptOptions::default().with_paragraph_boundaries())
        .with_distinct_evidence();
    let response = Retriever::new(LexicalSource::new(&engine)).query("Rust memory", &options)?;
    let source = response
        .hits()
        .iter()
        .find(|h| h.excerpt.contains("Rust protects memory."))
        .unwrap();
    let limits = CitationLimits::default();
    let answer = CitedAnswer::new(
        Some("Rust protects memory.".into()),
        vec![Citation::new(&source.id).with_quote("Rust protects memory.")],
        &limits,
    )?;
    let report = CitationRegistry::new(&response).check(&answer, &limits)?;
    assert!(report.quotes_verified());
    println!(
        "{}",
        serde_json::json!({
            "retrieval": response,
            "citation_check": report,
            "semantic_truth_verified": false,
        })
    );
    Ok(())
}
