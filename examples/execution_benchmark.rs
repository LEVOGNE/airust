use airust::{execution::*, Engine, KnowledgeEntry, SearchOptions};
use serde_json::json;
use std::time::Instant;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let index = Engine::from_entries(
        (0..10000).map(|i| KnowledgeEntry::new(i.to_string(), format!("Rust ownership topic{i}"))),
    )?;
    let options = SearchOptions::default();
    let control = WorkControl::default();
    let mut rows = Vec::new();
    for controlled in [false, true, false, true] {
        let started = Instant::now();
        let mut signature = 0u64;
        for _ in 0..200 {
            let hits = if controlled {
                index.search_controlled("Rust", &options, &control)?
            } else {
                index.search_with_options("Rust", &options)
            };
            for hit in hits {
                signature = signature.wrapping_add(hit.score.to_bits());
            }
        }
        rows.push(json!({"controlled":controlled,"queries":200,"elapsed_ms":started.elapsed().as_secs_f64()*1000.0,"score_signature":signature}));
    }
    let token = CancellationToken::new();
    token.cancel();
    let cancelled = WorkControl::new(token);
    let started = Instant::now();
    let mut errors = 0;
    for _ in 0..10000 {
        errors += usize::from(
            index
                .search_controlled("Rust", &options, &cancelled)
                .is_err(),
        );
    }
    assert_eq!(errors, 10000);
    println!(
        "{}",
        json!({"entries":index.len(),"fixture":"execution-lexical-v1","rows":rows,"cancelled_calls":errors,"cancelled_calls_ms":started.elapsed().as_secs_f64()*1000.0,"limits":"Descriptive lexical microbenchmark; no hard interruption or provider latency guarantee."})
    );
    Ok(())
}
