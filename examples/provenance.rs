use airust::provenance::{
    CodePosition, CodeSpan, ProvenanceStatus, ProvenanceStore, SourceReference,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let text = "fn main() {}\n";
    let span = CodeSpan::new(CodePosition::new(1, 1)?, CodePosition::new(1, 13)?)?;
    let reference = SourceReference::new("source-main-v1")?
        .with_document("sample-project")?
        .with_code("src/main.rs", span)?
        .with_revision("example-commit-1")?;
    reference.validate_source_text(text)?;
    let mut store = ProvenanceStore::new();
    store.insert_source(reference)?;
    store.bind("entry-a", "source-main-v1")?;
    store.bind("entry-b", "source-main-v1")?;
    store.merge_entries("combined", ["entry-a", "entry-b"])?;
    assert_eq!(store.source_count(), 1);
    assert_eq!(store.status("unknown"), ProvenanceStatus::Missing);
    let encoded = store.to_json()?;
    assert_eq!(ProvenanceStore::from_json(&encoded)?, store);
    println!("{encoded}");
    Ok(())
}
