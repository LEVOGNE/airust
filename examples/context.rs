use airust::{
    context_engine::{ContextAssembler, ContextOptions},
    graph::{KnowledgeGraph, Relation, RelationType},
    Context, Engine, KnowledgeEntry,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut index = Engine::new();
    index.upsert(KnowledgeEntry::new("rust", "Rust provides memory safety."))?;
    index.upsert(KnowledgeEntry::new(
        "ownership",
        "Ownership manages resource lifetimes.",
    ))?;
    let mut graph = KnowledgeGraph::new();
    graph.add_node("rust")?;
    graph.add_node("ownership")?;
    graph.upsert_relation(Relation::new(
        "ref-1",
        "rust",
        "ownership",
        RelationType::References,
    )?)?;
    let scope = Context::new().with_entry("rust").with_entry("ownership");
    let options = ContextOptions::new(5, 256)
        .with_item_bytes(128)
        .with_scope(scope);
    let bundle = ContextAssembler::new(&index)
        .with_graph(&graph)
        .query("Rust", None, &options)?;
    assert_eq!(bundle.node_count(), 2);
    assert!(bundle.used_text_bytes() <= options.max_text_bytes());
    println!("{}", serde_json::to_string_pretty(&bundle)?);
    Ok(())
}
