use airust::{
    provenance::SourceReference,
    snapshots::*,
    text_search::{AnalyzerConfig, TextSearchOptions},
    KnowledgeEntry,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = DatasetManager::new(
        "incremental",
        DatasetConfig::default().with_text_analyzer(AnalyzerConfig::german()),
    )?;
    let empty = manager.snapshot()?;
    let entry = KnowledgeEntry::new("house", "Häuser haben rote Türen")
        .with_metadata(serde_json::json!({"version":1}));
    let first = manager.publish(manager.prepare_replace(
        empty.revision(),
        vec![
            DatasetRecord::new(entry.clone()),
            DatasetRecord::new(KnowledgeEntry::new("other", "Rust ownership")),
        ],
        None,
        None,
    )?)?;
    let changed = entry.with_metadata(serde_json::json!({"version":2}));
    let next = manager.publish(manager.prepare(
        first.revision(),
        DatasetUpdate::new(vec![DatasetChange::Put(DatasetRecord::new(
            changed.clone(),
        ))]),
        None,
    )?)?;
    assert_eq!(next.stats().metadata_only_puts, 1);
    assert_eq!(next.stats().text_entries_rebuilt, 0);
    assert_eq!(next.stats().text_entries_reused, 2);
    assert_eq!(
        next.text_index().unwrap().get("house").unwrap().metadata,
        changed.metadata
    );
    assert_eq!(
        first.lexical().get("house").unwrap().metadata,
        Some(serde_json::json!({"version":1}))
    );
    assert_eq!(
        next.text_index()
            .unwrap()
            .search("\"Haus haben\"", &TextSearchOptions::new(1))?[0]
            .entry
            .id,
        "house"
    );
    let source = DatasetRecord::new(changed).with_sources(vec![SourceReference::new("manual")?]);
    let third = manager.publish(manager.prepare(
        next.revision(),
        DatasetUpdate::new(vec![DatasetChange::Put(source)]),
        None,
    )?)?;
    assert!(std::ptr::eq(
        next.text_index().unwrap(),
        third.text_index().unwrap()
    ));
    assert_eq!(third.stats().unchanged_puts, 1);
    println!("Metadata updates reused {} text analyses; source-only changes kept the complete positional projection.",third.stats().text_entries_reused);
    Ok(())
}
