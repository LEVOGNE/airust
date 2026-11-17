use airust::{Engine, KnowledgeEntry};
use serde_json::json;
use std::{env, error::Error, hint::black_box, time::Instant};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "--help" | "-h"))
    {
        println!("index_benchmark [ENTRY_COUNT=10000] [QUERY_COUNT=300]");
        return Ok(());
    }
    if args.len() > 2 {
        return Err("Expected at most entry count and query count".into());
    }
    let count = args
        .first()
        .map(|arg| arg.parse::<usize>())
        .transpose()?
        .unwrap_or(10_000);
    let iterations = args
        .get(1)
        .map(|arg| arg.parse::<usize>())
        .transpose()?
        .unwrap_or(300);
    if count == 0 || iterations == 0 {
        return Err("Entry and query counts must be positive".into());
    }
    let buckets = count.min(10_000);
    let started = Instant::now();
    let mut index = Engine::from_entries((0..count).map(|id| {
        KnowledgeEntry::new(
            format!("entry-{id:09}"),
            format!(
                "Knowledge entry bucket{} reference alpha{}.",
                id % buckets,
                id % 37
            ),
        )
    }))?;
    let build = started.elapsed();
    assert_eq!(index.len(), count);
    for id in 0..20.min(iterations) {
        black_box(index.search(&format!("bucket{}", id % buckets), 10));
    }
    let mut latencies = Vec::with_capacity(iterations);
    let mut total_hits = 0usize;
    for id in 0..iterations {
        let query = format!("bucket{}", id.wrapping_mul(7919) % buckets);
        let started = Instant::now();
        let hits = index.search(black_box(&query), 10);
        latencies.push(started.elapsed().as_nanos());
        assert!(!hits.is_empty());
        assert!(hits.iter().all(|hit| hit.score.is_finite()
            && hit.score > 0.0
            && hit.matched_terms.contains(&query)));
        total_hits += hits.len();
        black_box(hits);
    }
    let raw_latencies = latencies.clone();
    latencies.sort_unstable();
    let percentile =
        |percent: usize| latencies[(latencies.len() - 1) * percent / 100] as f64 / 1000.0;
    let mutations = count.min(1000);
    let started = Instant::now();
    for id in 0..mutations {
        assert!(index
            .upsert(KnowledgeEntry::new(
                format!("entry-{id:09}"),
                format!("Updated bucket{} reference alpha{}.", id % buckets, id % 37)
            ))?
            .is_some());
    }
    let update = started.elapsed();
    let started = Instant::now();
    for id in 0..mutations {
        assert!(index.remove(&format!("entry-{id:09}")).is_some());
    }
    let deletion = started.elapsed();
    assert_eq!(index.len(), count - mutations);
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "airust_version": airust::VERSION,
            "entries": count,
            "queries": iterations,
            "bucket_count": buckets,
            "result_limit": 10,
            "total_hits": total_hits,
            "build_ms": build.as_secs_f64() * 1000.0,
            "warmup_queries": 20.min(iterations),
            "raw_search_samples_ns": raw_latencies,
            "search_p50_us": percentile(50),
            "search_p95_us": percentile(95),
            "search_p99_us": percentile(99),
            "mutations": mutations,
            "update_total_ms": update.as_secs_f64() * 1000.0,
            "delete_total_ms": deletion.as_secs_f64() * 1000.0,
            "stats_after_delete": index.stats()
        }))?
    );
    Ok(())
}
