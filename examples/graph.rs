use airust::graph::{KnowledgeGraph, Relation, RelationType, TraversalOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut graph = KnowledgeGraph::new();
    for id in ["rust", "memory", "ownership"] {
        graph.add_node(id)?;
    }
    graph.upsert_relation(Relation::new(
        "rust-memory",
        "rust",
        "memory",
        RelationType::ParentChild,
    )?)?;
    graph.upsert_relation(Relation::new(
        "rust-ownership",
        "rust",
        "ownership",
        RelationType::ParentChild,
    )?)?;
    graph.upsert_relation(Relation::new(
        "ownership-memory",
        "ownership",
        "memory",
        RelationType::DependsOn,
    )?)?;
    assert!(!graph.has_cycle());
    for visit in graph.breadth_first("rust", &TraversalOptions::new(2, 10))? {
        println!(
            "{}: depth {}, parent {:?}",
            visit.id, visit.depth, visit.parent
        );
    }
    println!(
        "The application supplies these relationships; the engine does not infer their truth."
    );
    Ok(())
}
