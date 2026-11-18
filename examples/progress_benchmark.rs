use airust::{execution::*, Engine, KnowledgeEntry, SearchOptions};
use serde_json::json;
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Instant,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let index = Engine::from_entries(
        (0..10000).map(|i| KnowledgeEntry::new(i.to_string(), "Rust ownership")),
    )?;
    let mut rows = Vec::new();
    for observed in [false, true, false, true] {
        let events = Arc::new(AtomicUsize::new(0));
        let count = events.clone();
        let control = if observed {
            WorkControl::default().with_observer(move |_: &ProgressEvent| {
                count.fetch_add(1, Ordering::Relaxed);
            })
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
        rows.push(json!({"observer":observed,"queries":200,"elapsed_ms":start.elapsed().as_secs_f64()*1000.0,"events":events.load(Ordering::Relaxed),"dropped_events":control.dropped_events(),"observer_failures":control.observer_failures(),"score_signature":signature}));
    }
    println!(
        "{}",
        json!({"fixture":"progress-lexical-v1","entries":index.len(),"rows":rows,"limits":"Synchronous in-memory counter observer only; no transport, UI or callback blocking latency guarantee."})
    );
    Ok(())
}
