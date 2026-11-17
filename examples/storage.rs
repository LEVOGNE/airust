use airust::{
    storage::{read_snapshot, MemoryStore, SnapshotOptions, StorageBackend, StoreChange},
    KnowledgeEntry,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut store = MemoryStore::new("example")?;
    let initial = store.revision()?;
    let committed = store.apply(
        &initial,
        &[StoreChange::Put(KnowledgeEntry::new(
            "rust",
            "Rust ownership prevents memory errors.",
        ))],
    )?;
    assert!(store
        .apply(&initial, &[StoreChange::Delete("rust".into())])
        .is_err());
    let loaded = read_snapshot(&mut store, &SnapshotOptions::default())?.into_index()?;
    assert_eq!(loaded.revision(), &committed.revision);
    assert_eq!(loaded.index().search("Rust", 1)[0].entry.id, "rust");
    println!(
        "Committed generation {}. Store and derived search index remain separate.",
        loaded.revision().generation()
    );
    Ok(())
}
