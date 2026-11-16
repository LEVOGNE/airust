use airust::{
    context_engine::{ContextAssembler, ContextOptions},
    provenance::{ProvenanceStatus, ProvenanceStore, SourceReference},
    Engine, KnowledgeEntry,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let index = Engine::from_entries([
        KnowledgeEntry::new("note-1", "Rust provides memory safety."),
        KnowledgeEntry::new("note-2", "RUST provides memory safety."),
    ])?;
    let mut sources = ProvenanceStore::new();
    sources.add(
        "note-1",
        SourceReference::new("manual-p1")?
            .with_document("manual")?
            .with_page(1)?,
    )?;
    sources.add(
        "note-2",
        SourceReference::new("manual-p2")?
            .with_document("manual")?
            .with_page(2)?,
    )?;
    let context = ContextAssembler::new(&index)
        .with_provenance(&sources)
        .query("Rust", None, &ContextOptions::new(4, 256))?;
    assert_eq!(context.items().len(), 1);
    assert_eq!(context.items()[0].references().len(), 2);
    assert!(context.items()[0]
        .references()
        .iter()
        .all(|r| r.provenance_status() == ProvenanceStatus::Referenced));
    println!("{}", serde_json::to_string_pretty(&context)?);
    Ok(())
}
