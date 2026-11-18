use airust::{feedback::*, reranking::*, KnowledgeEntry};
struct Base;
impl Reranker for Base {
    fn recipe(&self) -> &str {
        "demo-neutral-v1"
    }
    fn score(&mut self, _: &str, documents: &[&str]) -> Result<Vec<f64>, RerankError> {
        Ok(vec![0.0; documents.len()])
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let query = "Rust ownership";
    let entries = [
        KnowledgeEntry::new("glossary", "Rust ownership glossary headings"),
        KnowledgeEntry::new("answer", "Rust ownership prevents memory errors"),
    ];
    let mut ledger = FeedbackLedger::default();
    ledger.record(FeedbackEvent::new(
        "rating-1",
        "demo",
        query,
        &entries[1].text,
        Rating::Helpful,
    )?)?;
    ledger.record(FeedbackEvent::new(
        "rating-2",
        "demo",
        query,
        &entries[0].text,
        Rating::Unhelpful,
    )?)?;
    let mut base = Base;
    let mut scorer = FeedbackReranker::new(&mut base, &ledger, "demo", FeedbackOptions::default())?;
    let result = rerank(query, &entries, &mut scorer, &RerankOptions::new(2))?;
    assert_eq!(result.results()[0].entry().id, "answer");
    println!("{}", serde_json::to_string(scorer.last_adjustments())?);
    println!("Feedback events: {}; namespace: demo", ledger.len());
    Ok(())
}
