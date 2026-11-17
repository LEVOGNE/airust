use airust::{
    cache::{CacheIdentity, IndexCache},
    Engine, KnowledgeEntry, SearchAlgorithm, SearchOptions,
};
use serde_json::json;
use std::{error::Error, time::Instant};

fn records(count: usize) -> Vec<KnowledgeEntry> {
    (0..count)
        .map(|n| {
            KnowledgeEntry::new(
                format!("entry-{n:09}"),
                format!(
                    "key{n} Rust memory safety Grüße 世界 {}",
                    "Search knowledge with original metadata. ".repeat(10)
                ),
            )
            .with_metadata(json!({"group":n%17}))
        })
        .collect()
}
fn signature(index: &Engine, count: usize) -> Vec<(String, u64)> {
    let mut output = Vec::new();
    for query in [
        "Rust memory".to_owned(),
        format!("key{}", count / 2),
        "Grüße 世界".to_owned(),
    ] {
        for algorithm in [SearchAlgorithm::Bm25, SearchAlgorithm::Tfidf] {
            output.extend(
                index
                    .search_with_options(
                        &query,
                        &SearchOptions {
                            algorithm,
                            limit: 10,
                            ..SearchOptions::default()
                        },
                    )
                    .into_iter()
                    .map(|r| (r.entry.id.clone(), r.score.to_bits())),
            );
        }
    }
    output
}
fn main() -> Result<(), Box<dyn Error>> {
    let count = std::env::args()
        .nth(1)
        .map(|s| s.parse::<usize>())
        .transpose()?
        .unwrap_or(1000);
    if count == 0 {
        return Err("Entry count must be positive".into());
    }
    let path = std::env::temp_dir().join(format!("airust-cache-bench-{}", std::process::id()));
    std::fs::create_dir(&path)?;
    let cache = IndexCache::new(path.join("derived.cache"), CacheIdentity::new("synthetic")?);
    let originals = records(count);
    let began = Instant::now();
    let index = Engine::from_entries(originals)?;
    let build_ms = began.elapsed().as_secs_f64() * 1000.0;
    let expected = signature(&index, count);
    let began = Instant::now();
    cache.save(&index)?;
    let save_ms = began.elapsed().as_secs_f64() * 1000.0;
    let bytes = std::fs::metadata(cache.path())?.len();
    drop(index);
    let mut originals = records(count);
    let began = Instant::now();
    let loaded = cache.load(&mut originals)?.into_index();
    let load_ms = began.elapsed().as_secs_f64() * 1000.0;
    assert!(originals.is_empty());
    assert_eq!(expected, signature(&loaded, count));
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"airust_version":airust::VERSION,"entries":count,"build_ms":build_ms,"save_ms":save_ms,"load_ms":load_ms,"cache_bytes":bytes,"exact_query_score_roundtrip":true,"source_generation_timed":false})
        )?
    );
    std::fs::remove_dir_all(path)?;
    Ok(())
}
