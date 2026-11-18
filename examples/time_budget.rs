use airust::{execution::*, retrieval::*, Engine, KnowledgeEntry, SearchOptions};
use serde_json::json;
use std::time::{Duration, Instant};
struct Slow;
impl RetrievalSource for Slow {
    fn retrieve<'a>(
        &'a mut self,
        _: &str,
        _: &RetrievalOptions,
    ) -> Result<RetrievalBatch<'a>, RetrievalError> {
        std::thread::sleep(Duration::from_millis(25));
        Ok(RetrievalBatch::new("blocking-demo", vec![]))
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let index = Engine::from_entries(
        (0..10000).map(|i| KnowledgeEntry::new(i.to_string(), "Rust ownership")),
    )?;
    let mut rows = Vec::new();
    for timed in [false, true, false, true] {
        let control = if timed {
            WorkControl::default().with_timeout(Duration::from_secs(120))?
        } else {
            WorkControl::default()
        };
        let start = Instant::now();
        let mut signature = 0u64;
        for _ in 0..200 {
            for hit in index.search_controlled("Rust", &SearchOptions::default(), &control)? {
                signature = signature.wrapping_add(hit.score.to_bits());
            }
        }
        rows.push(json!({"deadline_enabled":timed,"queries":200,"elapsed_ms":start.elapsed().as_secs_f64()*1000.0,"score_signature":signature}));
    }
    let start = Instant::now();
    let options = RetrievalOptions::new(1)
        .with_control(WorkControl::default().with_timeout(Duration::from_millis(5))?);
    let result = Retriever::new(Slow).query("Rust", &options);
    assert!(matches!(
        result,
        Err(RetrievalError::Execution(ExecutionError::DeadlineExceeded))
    ));
    println!(
        "{}",
        json!({"fixture":"time-budget-v1","entries":index.len(),"rows":rows,"blocking_provider":{"budget_ms":5,"provider_sleep_ms":25,"observed_elapsed_ms":start.elapsed().as_secs_f64()*1000.0,"result":"deadline_exceeded"},"limits":"Cooperative: deadline does not forcibly stop a blocked provider. Descriptive timings, no hard response bound."})
    );
    Ok(())
}
