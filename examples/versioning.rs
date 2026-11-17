use airust::{
    versioning::{ChangeMetadata, ExpectedRevision, VersionedIndex},
    KnowledgeEntry,
};
use std::error::Error;
fn main() -> Result<(), Box<dyn Error>> {
    let mut index = VersionedIndex::new().with_history_limit(8);
    let first = index.upsert(
        KnowledgeEntry::new("note", "Original Rust note"),
        ExpectedRevision::Absent,
        ChangeMetadata::default().with_reason("Create")?,
    )?;
    let second = index.upsert(
        KnowledgeEntry::new("note", "Updated Rust note"),
        ExpectedRevision::Exact(first),
        ChangeMetadata::default().with_reason("Edit")?,
    )?;
    assert!(index.compare("note", first, second)?.text_changed());
    let restored = index.restore(
        "note",
        first,
        ExpectedRevision::Exact(second),
        ChangeMetadata::default().with_reason("Restore original")?,
    )?;
    assert!(restored > second);
    assert_eq!(
        index.index().get("note").unwrap().text,
        "Original Rust note"
    );
    let reloaded = VersionedIndex::from_json(&index.to_json()?)?;
    assert_eq!(reloaded.revision("note"), Some(restored));
    println!(
        "Current revision: {}; retained snapshots: {}",
        restored.get(),
        reloaded.history_len()
    );
    Ok(())
}
