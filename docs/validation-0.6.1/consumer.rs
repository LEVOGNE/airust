use airust::{
    documents::{DocumentLimits, DocumentSnapshot, SourceFingerprint, TextSpace},
    execution::{CancellationToken, ExecutionError, WorkControl},
    mcp::{McpLimits, McpServer},
    retrieval::{LexicalSource, RetrievalError, RetrievalOptions, Retriever},
    Engine, KnowledgeEntry,
};

struct ApplicationSearch;
impl airust::mcp::McpSearchBackend for ApplicationSearch {
    fn retrieve(
        &self,
        index: &Engine,
        query: &str,
        algorithm: airust::SearchAlgorithm,
        options: &RetrievalOptions,
    ) -> Result<airust::retrieval::RetrievalResponse, RetrievalError> {
        Retriever::new(
            LexicalSource::new(index).with_options(airust::SearchOptions {
                algorithm,
                ..Default::default()
            }),
        )
        .query(query, options)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(airust::VERSION, "0.6.1");
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

    let directory = tempfile::tempdir()?;
    let training_path = directory.path().join("training.json");
    let mut training = airust::KnowledgeBase::new();
    training.add_example("Grüße 東京".into(), "answer", 1.0);
    training.save(Some(training_path.clone()))?;
    let bytes = std::fs::read(&training_path)?;
    assert!(airust::KnowledgeBase::load_bounded(training_path.clone(), bytes.len() - 1).is_err());
    let mut loaded = airust::KnowledgeBase::load_bounded(training_path.clone(), bytes.len())?;
    let retained = loaded.clone();
    loaded.remove_example(0)?;
    assert!(loaded.get_examples().is_empty());
    assert_eq!(retained.get_examples()[0].input, "Grüße 東京");
    retained.save(None)?;
    assert_eq!(std::fs::read(&training_path)?, bytes);

    let invalid = format!("\n\n{}", serde_json::to_string(&"Grüße 東京".repeat(1000))?);
    match Engine::from_json(&invalid) {
        Err(airust::IndexError::Json(error)) => {
            assert!(error.is_data());
            assert_eq!(error.line(), 3);
            assert_eq!(error.column(), invalid.len() - 2);
            assert!(error.to_string().len() <= 2048);
            assert!(error.to_string().contains("[truncated]"));
        }
        _ => panic!("large malformed input did not retain a bounded positioned JSON error"),
    }

    let limits = DocumentLimits::default();
    let snapshot = DocumentSnapshot::from_pages(
        "manual:060",
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

    let backend = ApplicationSearch;
    let response = airust::mcp::McpSearchBackend::retrieve(
        &backend,
        &engine,
        "Rust",
        airust::SearchAlgorithm::Bm25,
        &RetrievalOptions::new(1),
    )?;
    assert_eq!(response.hits().len(), 1);
    let server =
        McpServer::new([entry], [snapshot], McpLimits::default())?.with_search_backend(backend);
    let stats = server.stats();
    assert_eq!((stats.entries, stats.documents, stats.in_flight), (1, 1, 0));
    let tools = server.tool_definitions();
    assert_eq!(tools.len(), 5);
    for tool in tools {
        let schema = serde_json::to_string(tool.output_schema.as_ref().unwrap())?;
        assert!(schema.contains("0.6.1"));
    }
    println!(
        "{}",
        serde_json::json!({
            "version": airust::VERSION,
            "core": "passed",
            "unicode": "passed",
            "clone_isolation": "passed",
            "training_copy_on_write_and_bounded_load": "passed",
            "bounded_multiline_json_error": "passed",
            "invalid_update_rollback": "passed",
            "json_round_trip": "passed",
            "document_spans_and_round_trip": "passed",
            "typed_cancellation": "passed",
            "mcp_stats_and_five_versioned_schemas": "passed"
        })
    );
    Ok(())
}
