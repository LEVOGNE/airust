use airust::{
    ann::{AnnSearchOptions, HnswConfig, HnswIndex},
    semantic::{Embedding, EmbeddingSpace, SemanticIndex, SemanticSearchOptions},
    KnowledgeEntry,
};
use serde_json::json;
use std::{collections::BTreeSet, error::Error, time::Instant};

fn values(id: u64, dimensions: usize, clustered: bool) -> Vec<f32> {
    let mut state = id
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    (0..dimensions)
        .map(|d| {
            state ^= state >> 12;
            state ^= state << 25;
            state ^= state >> 27;
            let noise = ((state.wrapping_mul(2685821657736338717) >> 40) as f32 / 16777216.0) - 0.5;
            if clustered {
                let cluster = id % 64;
                let anchor = (((cluster * 47 + (d as u64) * 23) % 101) as f32 - 50.0) / 50.0;
                anchor + noise * 0.25
            } else {
                noise
            }
        })
        .collect()
}
fn percentile(samples: &[f64], percent: usize) -> f64 {
    samples[(samples.len() * percent).div_ceil(100).saturating_sub(1)]
}
fn main() -> Result<(), Box<dyn Error>> {
    let count = std::env::args()
        .nth(1)
        .map(|s| s.parse::<usize>())
        .transpose()?
        .unwrap_or(1000);
    let clustered = std::env::args().nth(2).as_deref() != Some("random");
    if count < 10 {
        return Err("At least ten entries required".into());
    }
    let dimensions = 384;
    let space = EmbeddingSpace::new(
        if clustered {
            "synthetic-clusters-v1"
        } else {
            "synthetic-random-v1"
        },
        dimensions,
    )?;
    let mut semantic = SemanticIndex::new(space.clone());
    let start = Instant::now();
    for n in 0..count {
        semantic.upsert_embedded(
            KnowledgeEntry::new(format!("entry-{n:09}"), format!("synthetic record key{n}")),
            Embedding::new(space.clone(), values(n as u64, dimensions, clustered))?,
        )?;
    }
    let vector_build_ms = start.elapsed().as_secs_f64() * 1000.0;
    let start = Instant::now();
    let ann = HnswIndex::from_semantic(semantic, HnswConfig::default().with_exact_threshold(0));
    let topology_build_ms = start.elapsed().as_secs_f64() * 1000.0;
    let exact_options = SemanticSearchOptions::new(10);
    let mut records = Vec::new();
    for width in [40, 100, 250, 1000] {
        let options = AnnSearchOptions::new(10).with_ef_search(width)?;
        let mut exact_samples = Vec::new();
        let mut ann_samples = Vec::new();
        let mut recalls = Vec::new();
        for n in 0..100 {
            let query = Embedding::new(
                space.clone(),
                values((count + n * 7919 + 1) as u64, dimensions, clustered),
            )?;
            let start = Instant::now();
            let exact = ann.semantic().search_embedding(&query, &exact_options)?;
            exact_samples.push(start.elapsed().as_secs_f64() * 1000.0);
            let start = Instant::now();
            let actual = ann.search_embedding(&query, &options)?;
            ann_samples.push(start.elapsed().as_secs_f64() * 1000.0);
            let expected: BTreeSet<_> = exact.iter().map(|r| r.entry.id.as_str()).collect();
            recalls.push(
                actual
                    .results()
                    .iter()
                    .filter(|r| expected.contains(r.entry.id.as_str()))
                    .count() as f64
                    / 10.0,
            );
            for hit in actual.results() {
                let score = airust::semantic::cosine_similarity(
                    query.values(),
                    ann.semantic().embedding(&hit.entry.id).unwrap(),
                )?;
                assert_eq!(score.to_bits(), hit.score.to_bits());
            }
        }
        exact_samples.sort_by(f64::total_cmp);
        ann_samples.sort_by(f64::total_cmp);
        records.push(json!({"ef_search":width,"recall_at_10":recalls.iter().sum::<f64>()/100.0,"query_recalls":recalls,"exact_p50_ms":percentile(&exact_samples,50),"exact_p95_ms":percentile(&exact_samples,95),"ann_p50_ms":percentile(&ann_samples,50),"ann_p95_ms":percentile(&ann_samples,95),"exact_samples_ms":exact_samples,"ann_samples_ms":ann_samples}));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"airust_version":airust::VERSION,"entries":count,"dimensions":dimensions,"distribution":if clustered{"64 deterministic clusters with independent query noise"}else{"independent pseudorandom unit vectors"},"queries_per_width":100,"vector_build_ms":vector_build_ms,"topology_build_ms":topology_build_ms,"connections":16,"layers":16,"ef_construction":200,"widths":records,"inference_included":false})
        )?
    );
    Ok(())
}
