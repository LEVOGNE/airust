use airust::{
    reranking::{rerank, RerankError, RerankOptions, Reranker},
    Engine, KnowledgeEntry,
};

struct DemonstrationScorer;
impl Reranker for DemonstrationScorer {
    fn recipe(&self) -> &str {
        "handwritten-ownership-demo-v1-not-a-semantic-model"
    }
    fn score(&mut self, _query: &str, documents: &[&str]) -> Result<Vec<f64>, RerankError> {
        Ok(documents
            .iter()
            .map(|text| if text.contains("ownership") { 1.0 } else { 0.0 })
            .collect())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let index = Engine::from_entries([
        KnowledgeEntry::new("safe", "Rust ownership helps prevent memory errors."),
        KnowledgeEntry::new(
            "manual",
            "Rust raw pointers require manual memory invariants.",
        ),
    ])?;
    let candidates = index.search("Rust memory", 20);
    let result = rerank(
        "Rust memory",
        &candidates,
        &mut DemonstrationScorer,
        &RerankOptions::new(2),
    )?;
    assert_eq!(result.results()[0].entry().id, "safe");
    println!("Handwritten API demonstration; not a model quality evaluation.");
    for hit in result.results() {
        println!(
            "{}: rerank={:?}, input_rank={}, retrieval={}",
            hit.entry().id,
            hit.score,
            hit.original_rank,
            hit.candidate.score
        );
    }
    Ok(())
}
