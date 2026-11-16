use airust::{
    hybrid::HybridSearchOptions,
    semantic::{local::FastEmbedProvider, EmbeddingProvider, SemanticIndex, SemanticSearchOptions},
    Engine, SearchAlgorithm, SearchOptions,
};
use serde_json::{json, Value};
use std::{error::Error, time::Instant};
mod support;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("Usage: hybrid_quality MODEL_DIRECTORY ONNX_RUNTIME_LIBRARY".into());
    }
    let reference = support::reference()?;
    let mut provider = FastEmbedProvider::multilingual_e5_small(&args[0], &args[1], 2)?;
    let model = provider.space().model().to_owned();
    let index =
        SemanticIndex::from_index(Engine::from_entries(reference.documents)?, &mut provider, 4)?;
    let mut systems = Vec::new();
    for system in [
        "bm25",
        "tfidf",
        "semantic",
        "hybrid_equal",
        "hybrid_semantic75",
    ] {
        let mut cases = Vec::new();
        let mut top1 = 0;
        let mut recall3 = 0.0;
        let mut mrr = 0.0;
        for query in &reference.queries {
            let started = Instant::now();
            let (ids, evidence): (Vec<String>, Vec<Value>) = match system {
                "bm25" | "tfidf" => {
                    let hits = index.lexical().search_with_options(
                        &query.text,
                        &SearchOptions {
                            limit: index.len(),
                            algorithm: if system == "bm25" {
                                SearchAlgorithm::Bm25
                            } else {
                                SearchAlgorithm::Tfidf
                            },
                            ..Default::default()
                        },
                    );
                    (hits.iter().map(|h|h.entry.id.clone()).collect(),hits.iter().take(3).map(|h|json!({"id":h.entry.id,"score":h.score,"terms":h.matched_terms})).collect())
                }
                "semantic" => {
                    let hits = index.search(
                        &query.text,
                        &mut provider,
                        &SemanticSearchOptions::new(index.len()),
                    )?;
                    (
                        hits.iter().map(|h| h.entry.id.clone()).collect(),
                        hits.iter()
                            .take(3)
                            .map(|h| json!({"id":h.entry.id,"score":h.score}))
                            .collect(),
                    )
                }
                _ => {
                    let mut options = HybridSearchOptions::new(index.len());
                    if system == "hybrid_semantic75" {
                        options = options.with_weights(1.0, 3.0)?;
                    }
                    let outcome = index.hybrid_search(&query.text, &mut provider, &options)?;
                    let hits = outcome.results();
                    (hits.iter().map(|h|h.entry.id.clone()).collect(),hits.iter().take(3).map(|h|json!({"id":h.entry.id,"score":h.score,"lexical_rank":h.lexical_rank,"semantic_rank":h.semantic_rank})).collect())
                }
            };
            let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
            let rank = ids
                .iter()
                .position(|id| query.relevant.contains(id))
                .map(|r| r + 1);
            if rank == Some(1) {
                top1 += 1;
            }
            if let Some(rank) = rank {
                mrr += 1.0 / rank as f64;
            }
            recall3 += ids
                .iter()
                .take(3)
                .filter(|id| query.relevant.contains(id))
                .count() as f64
                / query.relevant.len() as f64;
            cases.push(json!({"id":query.id,"query":query.text,"relevant":query.relevant,"first_relevant_rank":rank,"elapsed_ms":elapsed_ms,"top3":evidence}));
        }
        let n = reference.queries.len() as f64;
        systems.push(json!({"system":system,"top1_accuracy":top1 as f64/n,"recall_at_3":recall3/n,"mrr":mrr/n,"cases":cases}));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"airust_version":airust::VERSION,"fixture":reference.version,"model_space":model,"documents":index.len(),"queries":reference.queries.len(),"systems":systems,"limits":"Fixed hand-labeled smoke corpus. Single-pass timings exclude model load and index build. No general quality improvement or latency guarantee."})
        )?
    );
    Ok(())
}
