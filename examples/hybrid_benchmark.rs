use airust::{
    hybrid::HybridSearchOptions,
    semantic::{
        Embedding, EmbeddingError, EmbeddingProvider, EmbeddingPurpose, EmbeddingSpace,
        SemanticIndex,
    },
    KnowledgeEntry,
};
use serde_json::json;
use std::{error::Error, hint::black_box, time::Instant};
#[path = "support/vectors.rs"]
mod vectors;

struct Provider(EmbeddingSpace, Vec<f32>);
impl EmbeddingProvider for Provider {
    fn space(&self) -> &EmbeddingSpace {
        &self.0
    }
    fn embed(
        &mut self,
        texts: &[&str],
        _: EmbeddingPurpose,
    ) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        Ok(texts.iter().map(|_| self.1.clone()).collect())
    }
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
    let dimensions = 384;
    let space = EmbeddingSpace::new("synthetic-benchmark-v1:seed7", dimensions)?;
    let started = Instant::now();
    let mut index = SemanticIndex::new(space.clone());
    for id in 0..count {
        index.upsert_embedded(
            KnowledgeEntry::new(
                format!("{id:09}"),
                format!("Knowledge record bucket{}.", id % 100),
            ),
            Embedding::new(space.clone(), vectors::values(id, dimensions))?,
        )?;
    }
    let build_ms = started.elapsed().as_secs_f64() * 1000.0;
    let mut provider = Provider(space, vectors::values(17, dimensions));
    let options = HybridSearchOptions::new(10);
    black_box(index.hybrid_search("bucket17", &mut provider, &options)?);
    let mut samples = Vec::new();
    for _ in 0..50 {
        let started = Instant::now();
        let outcome = index.hybrid_search(black_box("bucket17"), &mut provider, &options)?;
        samples.push(started.elapsed().as_secs_f64() * 1000.0);
        assert!(!outcome.results().is_empty());
        assert!(outcome.semantic_used());
        let ids: std::collections::BTreeSet<_> =
            outcome.results().iter().map(|h| &h.entry.id).collect();
        assert_eq!(ids.len(), outcome.results().len());
        black_box(outcome);
    }
    samples.sort_by(f64::total_cmp);
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"airust_version":airust::VERSION,"entries":count,"dimensions":dimensions,"queries":50,"candidate_limit":100,"result_limit":10,"build_ms":build_ms,"samples_ms":samples,"p50_ms":samples[24],"p95_ms":samples[46],"limits":"Repeated synthetic query, exact vector scan, precomputed provider, no real-model inference or general quality claim."})
        )?
    );
    Ok(())
}
