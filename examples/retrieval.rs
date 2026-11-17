use airust::{
    retrieval::{LexicalSource, RetrievalOptions, Retriever},
    Context, Engine, KnowledgeEntry,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let index = Engine::from_entries(vec![
        KnowledgeEntry::new(
            "note-1",
            "Rust provides memory safety without garbage collection.",
        ),
        KnowledgeEntry::new("note-2", "Python is useful for scripting."),
    ])?;
    let options = RetrievalOptions::new(5)
        .with_candidates(20)?
        .with_context(Context::new().with_entry("note-1"))
        .with_byte_budgets(1024, 1024, 512, 8192)?;
    let response = Retriever::new(LexicalSource::new(&index)).query("memory safety", &options)?;
    assert_eq!(response.hits()[0].id, "note-1");
    println!("{}", response.to_json());
    Ok(())
}
