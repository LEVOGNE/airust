use airust::{
    documents::{DocumentLimits, DocumentSnapshot, SourceFingerprint, TextSpace},
    execution::{CancellationToken, ExecutionError, WorkControl},
    mcp::{McpLimits, McpServer},
    retrieval::{LexicalSource, RetrievalError, RetrievalOptions, Retriever},
    Engine, KnowledgeEntry,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(airust::VERSION, "0.5.2");
    let mut engine = Engine::from_entries([
        KnowledgeEntry::new("rust", "Rust ownership protects memory."),
        KnowledgeEntry::new("python", "Python helps with scripting."),
        KnowledgeEntry::new("unicode", "Café Grüße 東京"),
    ])?;
    assert_eq!(engine.search("Rust memory", 1)[0].entry.id, "rust");
    assert_eq!(engine.search("cafe\u{301}", 1)[0].entry.id, "unicode");
    let old = engine.clone();
    engine.upsert(KnowledgeEntry::new("python", "Rust ownership automation."))?;
    assert_eq!(
        old.get("python").unwrap().text,
        "Python helps with scripting."
    );
    assert_ne!(old.get("python"), engine.get("python"));
    let before = engine.to_json()?;
    assert!(engine.upsert(KnowledgeEntry::new("rust", " \t\n")).is_err());
    assert_eq!(engine.to_json()?, before);
    assert_eq!(Engine::from_json(&before)?.to_json()?, before);

    let limits = DocumentLimits::default();
    let snapshot = DocumentSnapshot::from_pages(
        "manual:052",
        SourceFingerprint::from_bytes(b"release-consumer-source", &limits)?,
        "release-consumer-v1",
        TextSpace::Cleaned,
        [(3, "Café — Rust ownership protects memory.".to_owned())],
        &limits,
    )?;
    let passage = snapshot.page_passage(3)?;
    let entry = passage.to_entry(&snapshot)?;
    passage.validate_entry(&snapshot, &entry)?;
    let start = entry.text.find("Rust").unwrap();
    let located = passage.subspan(&snapshot, start, start + "Rust".len())?;
    assert_eq!(located.resolve(&snapshot)?, "Rust");
    let restored = DocumentSnapshot::from_json(&snapshot.to_json(limits.max_json_bytes)?, &limits)?;
    let restored_passage =
        restored.passage_from_json(&located.to_json(limits.max_json_bytes)?, &limits)?;
    assert_eq!(restored_passage.resolve(&restored)?, "Rust");

    let token = CancellationToken::new();
    let options = RetrievalOptions::new(2).with_control(WorkControl::new(token.clone()));
    assert!(!Retriever::new(LexicalSource::new(&engine))
        .query("Rust", &options)?
        .hits()
        .is_empty());
    token.cancel();
    assert!(matches!(
        Retriever::new(LexicalSource::new(&engine)).query("Rust", &options),
        Err(RetrievalError::Execution(ExecutionError::Cancelled))
    ));

    let server = McpServer::new([entry], [snapshot], McpLimits::default())?;
    let stats = server.stats();
    assert_eq!((stats.entries, stats.documents, stats.in_flight), (1, 1, 0));
    let tools = server.tool_definitions();
    assert_eq!(tools.len(), 5);
    for tool in tools {
        let schema = serde_json::to_string(tool.output_schema.as_ref().unwrap())?;
        assert!(schema.contains("0.5.2"));
    }
    println!(
        "{}",
        serde_json::json!({
            "version": airust::VERSION,
            "core": "passed",
            "unicode": "passed",
            "clone_isolation": "passed",
            "invalid_update_rollback": "passed",
            "json_round_trip": "passed",
            "document_spans_and_round_trip": "passed",
            "typed_cancellation": "passed",
            "mcp_stats_and_five_versioned_schemas": "passed"
        })
    );
    Ok(())
}
