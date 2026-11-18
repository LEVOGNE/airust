use airust::{feedback::*, reranking::*, KnowledgeEntry};
use std::time::Instant;
struct Neutral;
impl Reranker for Neutral {
    fn recipe(&self) -> &str {
        "neutral-v1"
    }
    fn score(&mut self, _: &str, d: &[&str]) -> Result<Vec<f64>, RerankError> {
        Ok(vec![0.; d.len()])
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let records = [
        KnowledgeEntry::new("a", "glossary headings"),
        KnowledgeEntry::new("b", "specific answer"),
    ];
    let mut base = Neutral;
    let mut ledger = FeedbackLedger::default();
    let baseline = rerank("query", &records, &mut base, &RerankOptions::new(2))?.results()[0]
        .entry()
        .id
        .clone();
    ledger.record(FeedbackEvent::new(
        "vote",
        "fixture",
        "query",
        &records[1].text,
        Rating::Helpful,
    )?)?;
    let preferred = {
        let mut s =
            FeedbackReranker::new(&mut base, &ledger, "fixture", FeedbackOptions::default())?;
        rerank("query", &records, &mut s, &RerankOptions::new(2))?.results()[0]
            .entry()
            .id
            .clone()
    };
    ledger.retract("vote")?;
    let reverted = {
        let mut s =
            FeedbackReranker::new(&mut base, &ledger, "fixture", FeedbackOptions::default())?;
        rerank("query", &records, &mut s, &RerankOptions::new(2))?.results()[0]
            .entry()
            .id
            .clone()
    };
    assert_eq!((&*baseline, &*preferred, &*reverted), ("a", "b", "a"));
    let start = Instant::now();
    for i in 0..10000 {
        ledger.record(FeedbackEvent::new(
            format!("event-{i}"),
            "fixture",
            &format!("query {}", i % 100),
            &format!("document {}", i % 200),
            if i % 3 == 0 {
                Rating::Unhelpful
            } else {
                Rating::Helpful
            },
        )?)?;
    }
    let record_ms = start.elapsed().as_secs_f64() * 1000.;
    let start = Instant::now();
    let json = ledger.to_json()?;
    let restored = FeedbackLedger::from_json(&json, FeedbackLimits::default())?;
    let roundtrip_ms = start.elapsed().as_secs_f64() * 1000.;
    assert_eq!(
        ledger.profile_fingerprint("fixture")?,
        restored.profile_fingerprint("fixture")?
    );
    println!(
        "{}",
        serde_json::json!({"schema":1,"fixture":"authored explicit preference, not held-out generalization","baseline_first":baseline,"feedback_first":preferred,"after_retraction_first":reverted,"events":ledger.len(),"json_bytes":json.len(),"record_ms":record_ms,"roundtrip_ms":roundtrip_ms})
    );
    Ok(())
}
