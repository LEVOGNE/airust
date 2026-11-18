use airust::{
    adaptive::{AdaptiveSource, SourceRole},
    retrieval::*,
    Engine, KnowledgeEntry,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let index = Engine::from_entries([KnowledgeEntry::new(
        "a",
        "Rust ownership prevents memory errors",
    )])?;
    let source = AdaptiveSource::default().with_source(
        "local-words",
        SourceRole::Lexical,
        LexicalSource::new(&index),
    )?;
    let mut retriever = Retriever::new(source);
    let response = retriever.query(
        "Rust ownership",
        &RetrievalOptions::new(3).with_evidence_policy(EvidencePolicy::default()),
    )?;
    assert_eq!(response.hits()[0].id, "a");
    assert_eq!(response.recipe(), "airust-lexical-v1");
    println!(
        "Route: {}",
        serde_json::to_string(retriever.source().last_route().unwrap())?
    );
    println!("{}", response.to_json());
    Ok(())
}
