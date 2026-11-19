use airust::{disk_index::*, retrieval::*, storage::StoreChange, KnowledgeEntry};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let folder = tempfile::tempdir()?;
    let path = folder.path().join("index.sqlite");
    let mut index = DiskIndex::open(&path, "demo", DiskOptions::default())?;
    let rev = index.revision()?;
    let committed = index.apply(
        &rev,
        &[StoreChange::Put(KnowledgeEntry::new(
            "a",
            "Rust ownership prevents memory errors",
        ))],
        &DiskWork::default(),
    )?;
    drop(index);
    let mut index = DiskIndex::open(&path, "demo", DiskOptions::default())?;
    assert_eq!(index.revision()?, committed);
    let mut retriever = Retriever::new(DiskSource::new(&mut index));
    let response = retriever.query("Rust ownership", &RetrievalOptions::new(3))?;
    assert_eq!(response.hits()[0].id, "a");
    println!("{}", response.to_json());
    Ok(())
}
