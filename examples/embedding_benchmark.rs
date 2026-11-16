use airust::{
    semantic::{Embedding, EmbeddingSpace, SemanticIndex, SemanticSearchOptions},
    KnowledgeEntry,
};
use serde_json::json;
use std::{error::Error, hint::black_box, time::Instant};

#[path = "support/vectors.rs"]
mod vectors;

fn main() -> Result<(), Box<dyn Error>> {
    let count = std::env::args()
        .nth(1)
        .map(|s| s.parse::<usize>())
        .transpose()?
        .unwrap_or(1000);
    if count == 0 {
        return Err("Entry count must be positive".into());
    }
    let dimensions = 384;
    let space = EmbeddingSpace::new("synthetic-benchmark-v1:seed7", dimensions)?;
    let values = |id: usize| vectors::values(id, dimensions);
    let mut index = SemanticIndex::new(space.clone());
    let started = Instant::now();
    for id in 0..count {
        index.upsert_embedded(
            KnowledgeEntry::new(format!("{id:09}"), format!("Synthetic record {id}")),
            Embedding::new(space.clone(), values(id))?,
        )?;
    }
    let build_ms = started.elapsed().as_secs_f64() * 1000.0;
    let options = SemanticSearchOptions::new(10);
    let queries: Vec<_> = (0..50)
        .map(|n| Embedding::new(space.clone(), values(n * 7919 % count)))
        .collect::<Result<_, _>>()?;
    black_box(index.search_embedding(&queries[0], &options)?);
    let mut samples = Vec::new();
    for query in &queries {
        let started = Instant::now();
        let hits = index.search_embedding(black_box(query), &options)?;
        samples.push(started.elapsed().as_secs_f64() * 1000.0);
        assert!(!hits.is_empty());
        assert!(hits[0].score > 0.99999);
        black_box(hits);
    }
    samples.sort_by(f64::total_cmp);
    let mutations = count.min(100);
    let started = Instant::now();
    for id in 0..mutations {
        index.upsert_embedded(
            KnowledgeEntry::new(format!("{id:09}"), "Updated record"),
            Embedding::new(space.clone(), values(id + count))?,
        )?;
    }
    let update_ms = started.elapsed().as_secs_f64() * 1000.0;
    let started = Instant::now();
    for id in 0..mutations {
        assert!(index.remove(&format!("{id:09}")).is_some());
    }
    let delete_ms = started.elapsed().as_secs_f64() * 1000.0;
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"airust_version":airust::VERSION,"entries":count,"dimensions":dimensions,"queries":queries.len(),"build_ms":build_ms,"samples_ms":samples,"p50_ms":samples[24],"p95_ms":samples[46],"mutations":mutations,"update_ms":update_ms,"delete_ms":delete_ms,"limits":"Deterministic synthetic vectors, exact linear search, no semantic-quality or ANN claim. Inference time excluded."})
        )?
    );
    Ok(())
}
