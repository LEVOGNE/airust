use airust::{retrieval::*, Engine, KnowledgeEntry};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let index = Engine::from_entries([KnowledgeEntry::new(
        "manual",
        "Rust ownership prevents memory errors.",
    )])?;
    let options = RetrievalOptions::new(3)
        .with_relevant_excerpts(ExcerptOptions::default())
        .with_evidence_policy(EvidencePolicy::default());
    let mut retriever = Retriever::new(LexicalSource::new(&index));
    let response = retriever.query("Rust ownership", &options)?;
    assert_eq!(
        response.assessment().unwrap().verdict(),
        EvidenceVerdict::MeetsPolicy
    );
    println!("{}", response.to_json());
    let partial = retriever.query("Rust volcano", &options)?;
    assert_eq!(
        partial.assessment().unwrap().verdict(),
        EvidenceVerdict::Insufficient
    );
    println!("{}", partial.to_json());
    Ok(())
}
