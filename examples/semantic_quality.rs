use airust::{
    semantic::{
        embed_batch, local::FastEmbedProvider, EmbeddingProvider, EmbeddingPurpose, SemanticIndex,
        SemanticSearchOptions,
    },
    Engine, KnowledgeEntry,
};
use serde::Deserialize;
use serde_json::json;
use std::{error::Error, time::Instant};

#[derive(Deserialize)]
struct Reference {
    version: String,
    documents: Vec<KnowledgeEntry>,
    queries: Vec<Query>,
}
#[derive(Deserialize)]
struct Query {
    id: String,
    text: String,
    relevant: Vec<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("Usage: semantic_quality MODEL_DIRECTORY ONNX_RUNTIME_LIBRARY".into());
    }
    let reference: Reference =
        serde_json::from_str(include_str!("../tests/fixtures/semantic-reference.json"))?;
    let started = Instant::now();
    let mut provider = FastEmbedProvider::multilingual_e5_small(&args[0], &args[1], 2)?;
    let load_ms = started.elapsed().as_secs_f64() * 1000.0;
    let space = provider.space().clone();
    let same = embed_batch(
        &mut provider,
        &["Identischer deutscher Text", "Identischer deutscher Text"],
        EmbeddingPurpose::Document,
    )?;
    let identical_cosine = same[0].cosine(&same[1])?;
    assert!(identical_cosine > 0.99999);
    let started = Instant::now();
    let index =
        SemanticIndex::from_index(Engine::from_entries(reference.documents)?, &mut provider, 4)?;
    let build_ms = started.elapsed().as_secs_f64() * 1000.0;
    let mut cases = Vec::new();
    let mut top1 = 0;
    let mut recall3 = 0.0;
    let mut reciprocal = 0.0;
    for query in &reference.queries {
        let started = Instant::now();
        let results = index.search(
            &query.text,
            &mut provider,
            &SemanticSearchOptions::new(index.len()),
        )?;
        let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
        let rank = results
            .iter()
            .position(|hit| query.relevant.contains(&hit.entry.id))
            .map(|r| r + 1);
        if rank == Some(1) {
            top1 += 1;
        }
        if let Some(rank) = rank {
            reciprocal += 1.0 / rank as f64;
        }
        let found = results
            .iter()
            .take(3)
            .filter(|hit| query.relevant.contains(&hit.entry.id))
            .count();
        recall3 += found as f64 / query.relevant.len() as f64;
        cases.push(json!({"id":query.id,"query":query.text,"relevant":query.relevant,"first_relevant_rank":rank,"elapsed_ms":elapsed_ms,"top3":results.iter().take(3).collect::<Vec<_>>()}));
    }
    let queries = reference.queries.len();
    let top1_accuracy = top1 as f64 / queries as f64;
    let recall_at_3 = recall3 / queries as f64;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "airust_version":airust::VERSION,"fixture":reference.version,"model_space":space.model(),"dimensions":space.dimensions(),
            "model_load_ms":load_ms,"index_build_ms":build_ms,"documents":index.len(),"queries":queries,"identical_document_cosine":identical_cosine,
            "top1_accuracy":top1_accuracy,"recall_at_3":recall_at_3,"mrr":reciprocal/queries as f64,
            "threads":2,"max_tokens":512,"cases":cases,"limits":"Small hand-labeled multilingual smoke corpus. Not an independent relevance benchmark, entailment test, or production guarantee."
        }))?
    );
    if top1_accuracy < 0.75 || recall_at_3 < 0.875 {
        return Err("The fixed semantic smoke-quality gate was not met".into());
    }
    let registration_rank = cases
        .iter()
        .find(|case| case["id"] == "de-registration")
        .and_then(|case| case["first_relevant_rank"].as_u64());
    if !registration_rank.is_some_and(|rank| rank <= 3) {
        return Err("The German registration paraphrase was not retrieved in the top three".into());
    }
    Ok(())
}
