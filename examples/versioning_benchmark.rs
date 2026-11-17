use airust::{
    versioning::{ChangeMetadata, ExpectedRevision, VersionedIndex},
    Engine, KnowledgeEntry, SearchAlgorithm, SearchOptions,
};
use serde_json::json;
use std::{error::Error, time::Instant};
fn signature(index: &Engine) -> Vec<(String, u64)> {
    let mut result = Vec::new();
    for algorithm in [SearchAlgorithm::Bm25, SearchAlgorithm::Tfidf] {
        for query in ["Rust", "changed", "key17"] {
            result.extend(
                index
                    .search_with_options(
                        query,
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
    result
}
fn main() -> Result<(), Box<dyn Error>> {
    let count = std::env::args()
        .nth(1)
        .map(|s| s.parse::<usize>())
        .transpose()?
        .unwrap_or(1000);
    let limit = std::env::args()
        .nth(2)
        .map(|s| s.parse::<usize>())
        .transpose()?
        .unwrap_or(128);
    if count == 0 {
        return Err("Entry count must be positive".into());
    }
    let start = Instant::now();
    let engine = Engine::from_entries((0..count).map(|n| {
        KnowledgeEntry::new(
            format!("entry-{n:09}"),
            format!(
                "Rust key{n} Grüße 世界 {}",
                "Knowledge search source text. ".repeat(8)
            ),
        )
        .with_metadata(json!({"group":n%17}))
    }))?;
    let lexical_build_ms = start.elapsed().as_secs_f64() * 1000.0;
    let start = Instant::now();
    let mut index = VersionedIndex::from_index(
        engine,
        ChangeMetadata::default().with_reason("Import originals")?,
    )?
    .with_history_limit(limit);
    let revision_setup_ms = start.elapsed().as_secs_f64() * 1000.0;
    let changes = count.min(1000);
    let start = Instant::now();
    for n in 0..changes {
        let id = format!("entry-{n:09}");
        let expected = ExpectedRevision::Exact(index.revision(&id).unwrap());
        if n % 5 == 0 {
            index.delete(&id, expected, ChangeMetadata::default())?;
        } else {
            index.upsert(
                KnowledgeEntry::new(id, format!("Rust changed key{n} Grüße 世界")),
                expected,
                ChangeMetadata::default(),
            )?;
        }
    }
    let changes_ms = start.elapsed().as_secs_f64() * 1000.0;
    let expected = signature(index.index());
    let sequence = index.sequence();
    let history_len = index.history_len();
    let start = Instant::now();
    let encoded = index.to_json()?;
    let export_ms = start.elapsed().as_secs_f64() * 1000.0;
    drop(index);
    let start = Instant::now();
    let restored = VersionedIndex::from_json_with_limit(&encoded, encoded.len())?;
    let import_ms = start.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(signature(restored.index()), expected);
    assert_eq!(restored.sequence(), sequence);
    assert_eq!(restored.history_len(), history_len);
    for n in 0..changes {
        let id = format!("entry-{n:09}");
        assert_eq!(restored.version(&id).unwrap().is_deleted(), n % 5 == 0);
    }
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"airust_version":airust::VERSION,"initial_entries":count,"current_entries":restored.index().len(),"tracked_ids":restored.tracked_ids(),"history_limit":limit,"retained_snapshots":history_len,"changes":changes,"lexical_build_ms":lexical_build_ms,"revision_setup_ms":revision_setup_ms,"changes_ms":changes_ms,"json_bytes":encoded.len(),"export_ms":export_ms,"import_ms":import_ms,"exact_query_roundtrip":true,"source_generation_timed":true})
        )?
    );
    Ok(())
}
