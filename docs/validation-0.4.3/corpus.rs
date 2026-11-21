use airust::{
    execution::{CancellationToken, WorkControl},
    Engine, KnowledgeEntry, SearchAlgorithm, SearchOptions,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    error::Error,
    fs,
    sync::Arc,
    time::{Duration, Instant},
};
const QUERIES: &[&str] = &[
    "memory ordering",
    "vector matrix",
    "type inference",
    "permission",
    "array broadcasting",
    "object stream",
    "parallel thread",
    "function",
    "memory",
    "airustabsentmarker723981",
];
fn signature(e: &Engine, q: &str, a: SearchAlgorithm) -> Vec<(String, u64, Vec<String>)> {
    e.search_with_options(
        q,
        &SearchOptions {
            limit: 10,
            algorithm: a,
            ..Default::default()
        },
    )
    .into_iter()
    .map(|h| {
        assert!(h.score.is_finite() && h.score > 0.0);
        (h.entry.id.clone(), h.score.to_bits(), h.matched_terms)
    })
    .collect()
}
fn digest(s: &str) -> String {
    format!("{:x}", Sha256::digest(s.as_bytes()))
}
fn distribution(mut data: Vec<f64>) -> Value {
    assert!(!data.is_empty());
    data.sort_by(f64::total_cmp);
    let at = |f: f64| data[((data.len() - 1) as f64 * f).ceil() as usize];
    json!({"samples":data.len(),"p50_ms":at(0.5),"p95_ms":at(0.95),"p99_ms":at(0.99),"max_ms":data.last()})
}
fn by_algorithm(data: &[f64]) -> Value {
    json!({"bm25":distribution(data.iter().enumerate().filter(|(i,_)|i%(QUERIES.len()*2)<QUERIES.len()).map(|(_,x)|*x).collect()),"tfidf":distribution(data.iter().enumerate().filter(|(i,_)|i%(QUERIES.len()*2)>=QUERIES.len()).map(|(_,x)|*x).collect())})
}
fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 3 {
        return Err("usage: corpus_stability INDEX.json MAX_ENTRIES ROUNDS".into());
    }
    let count: usize = args[1].parse()?;
    let rounds: usize = args[2].parse()?;
    if !(1000..=200000).contains(&count) || !(1..=32).contains(&rounds) {
        return Err("MAX_ENTRIES must be1000..200000 and ROUNDS1..32".into());
    }
    let start = Instant::now();
    let input = fs::read_to_string(&args[0])?;
    let input_sha = digest(&input);
    let mut records: Vec<KnowledgeEntry> = serde_json::from_str(&input)?;
    let parsed_ms = start.elapsed().as_secs_f64() * 1000.;
    drop(input);
    records.truncate(count);
    let count = records.len();
    assert!(count >= 1000);
    let start = Instant::now();
    let mut engine = Engine::from_entries(records)?;
    let build_ms = start.elapsed().as_secs_f64() * 1000.;
    assert_eq!(engine.len(), count);
    let baseline = engine.to_json()?;
    let baseline_sha = digest(&baseline);
    let original = engine.entries().take(128).cloned().collect::<Vec<_>>();
    let mut expected = Vec::new();
    let mut warm = Vec::new();
    for algorithm in [SearchAlgorithm::Bm25, SearchAlgorithm::Tfidf] {
        for q in QUERIES {
            let start = Instant::now();
            expected.push(signature(&engine, q, algorithm));
            warm.push(start.elapsed().as_secs_f64() * 1000.);
        }
    }
    let mut observed = Vec::new();
    let mut updates = Vec::new();
    let mut rebuilds = Vec::new();
    let mut roundtrips = Vec::new();
    for epoch in 0..rounds {
        for (i, q) in QUERIES.iter().cycle().take(QUERIES.len() * 2).enumerate() {
            let a = if i < QUERIES.len() {
                SearchAlgorithm::Bm25
            } else {
                SearchAlgorithm::Tfidf
            };
            let start = Instant::now();
            assert_eq!(signature(&engine, q, a), expected[i]);
            observed.push(start.elapsed().as_secs_f64() * 1000.);
        }
        let start = Instant::now();
        for old in original.iter().take(64) {
            assert!(engine.remove(&old.id).is_some());
        }
        for old in original.iter().skip(64) {
            engine.upsert(KnowledgeEntry::new(
                &old.id,
                format!("validationmarker epoch{epoch} memory vector"),
            ))?;
        }
        assert_eq!(engine.len(), count - 64);
        updates.push(start.elapsed().as_secs_f64() * 1000.);
        let start = Instant::now();
        let rebuilt = Engine::from_entries(engine.entries().cloned())?;
        rebuilds.push(start.elapsed().as_secs_f64() * 1000.);
        assert_eq!(engine.stats(), rebuilt.stats());
        assert_eq!(engine.to_json()?, rebuilt.to_json()?);
        for a in [SearchAlgorithm::Bm25, SearchAlgorithm::Tfidf] {
            for q in QUERIES {
                assert_eq!(signature(&engine, q, a), signature(&rebuilt, q, a));
            }
        }
        drop(rebuilt);
        for old in &original {
            engine.upsert(old.clone())?;
        }
        assert_eq!(engine.len(), count);
        assert_eq!(digest(&engine.to_json()?), baseline_sha);
        let start = Instant::now();
        let restored = Engine::from_json(&baseline)?;
        roundtrips.push(start.elapsed().as_secs_f64() * 1000.);
        for (i, q) in QUERIES.iter().cycle().take(QUERIES.len() * 2).enumerate() {
            let a = if i < QUERIES.len() {
                SearchAlgorithm::Bm25
            } else {
                SearchAlgorithm::Tfidf
            };
            assert_eq!(signature(&restored, q, a), expected[i]);
        }
        eprintln!(
            "Completed real-corpus epoch {}/{} with {} entries",
            epoch + 1,
            rounds,
            count
        );
    }
    let token = CancellationToken::new();
    token.cancel();
    assert!(engine
        .search_controlled(
            "memory",
            &SearchOptions::default(),
            &WorkControl::new(token)
        )
        .is_err());
    let expired = WorkControl::default().with_timeout(Duration::ZERO)?;
    assert!(engine
        .search_controlled("memory", &SearchOptions::default(), &expired)
        .is_err());
    let engine = Arc::new(engine);
    let start = Instant::now();
    let concurrent = std::thread::scope(|scope| {
        let threads = (0..4)
            .map(|_| {
                let engine = engine.clone();
                let expected = &expected;
                scope.spawn(move || {
                    let mut samples = Vec::new();
                    for _ in 0..2 {
                        for (i, q) in QUERIES.iter().cycle().take(QUERIES.len() * 2).enumerate() {
                            let a = if i < QUERIES.len() {
                                SearchAlgorithm::Bm25
                            } else {
                                SearchAlgorithm::Tfidf
                            };
                            let start = Instant::now();
                            assert_eq!(signature(&engine, q, a), expected[i]);
                            samples.push(start.elapsed().as_secs_f64() * 1000.);
                        }
                    }
                    samples
                })
            })
            .collect::<Vec<_>>();
        threads
            .into_iter()
            .flat_map(|t| t.join().unwrap())
            .collect::<Vec<_>>()
    });
    let concurrent_ms = start.elapsed().as_secs_f64() * 1000.;
    let start = Instant::now();
    let retained = engine.clone();
    let mut changed = engine.as_ref().clone();
    changed.upsert(KnowledgeEntry::new(
        &original[0].id,
        "isolated snapshot mutation",
    ))?;
    assert_eq!(retained.get(&original[0].id), Some(&original[0]));
    assert_ne!(changed.get(&original[0].id), Some(&original[0]));
    let clone_and_detach_ms = start.elapsed().as_secs_f64() * 1000.;
    println!(
        "{}",
        json!({"version":airust::VERSION,"fixture":"real-corpus-readiness-v1","features":["execution","shared-index"],"source_export_sha256":input_sha,"entries":count,"rounds":rounds,"queries":QUERIES,"parse_ms":parsed_ms,"build_ms":build_ms,"initial_search":by_algorithm(&warm),"repeated_search":by_algorithm(&observed),"update_delete_128":distribution(updates),"rebuild":distribution(rebuilds),"roundtrip":distribution(roundtrips),"concurrent":{"readers":4,"queries_per_reader":40,"elapsed_ms":concurrent_ms,"latency":by_algorithm(&concurrent)},"clone_and_detach_ms":clone_and_detach_ms,"baseline_export_sha256":baseline_sha,"all_incremental_rebuilds_identical":true,"all_roundtrip_and_concurrent_signatures_identical":true,"cancelled_and_expired_searches_rejected":true,"retained_clone_unchanged":true,"scope":"Trusted real PDF-derived lexical entries, shared-index/execution. Fixed literal queries, not semantic relevance labels. Timings include validation checks where stated; no model, external DB transaction or capacity/security guarantee. Canonical source never written; mutations remain in memory."})
    );
    Ok(())
}
