use airust::{
    quality::{evaluate_ranking, summarize, Judgments, QueryMetrics, UnjudgedPolicy},
    Engine, KnowledgeEntry, SearchAlgorithm, SearchOptions,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    time::Instant,
};

#[derive(Deserialize)]
struct Fixture {
    version: String,
    description: String,
    documents: Vec<KnowledgeEntry>,
    queries: Vec<Query>,
}
#[derive(Deserialize)]
struct Query {
    id: String,
    text: String,
    kind: String,
    grades: BTreeMap<String, u8>,
    lexical_top1: Option<String>,
}
fn labels(query: &Query, ids: &BTreeSet<String>) -> Result<Judgments, Box<dyn Error>> {
    if query.grades.keys().any(|id| !ids.contains(id)) {
        return Err("Unknown judged document".into());
    }
    Ok(Judgments::new(ids.iter().map(|id| {
        (id.clone(), query.grades.get(id).copied().unwrap_or(0))
    }))?)
}
fn record(
    query: &Query,
    ids: Vec<String>,
    scores: Vec<f64>,
    elapsed_ms: f64,
    labels: &Judgments,
) -> Result<(Value, QueryMetrics), Box<dyn Error>> {
    let metrics = evaluate_ranking(&ids, labels, 3, UnjudgedPolicy::RequireJudged)?;
    Ok((
        json!({"query_id":query.id,"query":query.text,"kind":query.kind,"grades":query.grades,"ranked_ids":ids,"scores":scores,"elapsed_ms":elapsed_ms,"metrics":metrics}),
        metrics,
    ))
}
fn main() -> Result<(), Box<dyn Error>> {
    let fixture: Fixture =
        serde_json::from_str(include_str!("../tests/fixtures/quality-reference-v1.json"))?;
    let ids: BTreeSet<_> = fixture.documents.iter().map(|e| e.id.clone()).collect();
    let mut query_ids = BTreeSet::new();
    if fixture
        .queries
        .iter()
        .any(|q| q.text.trim().is_empty() || !query_ids.insert(q.id.as_str()))
    {
        return Err("Invalid fixture queries".into());
    }
    let index = Engine::from_entries(fixture.documents)?;
    let mut reports = Vec::new();
    for algorithm in [SearchAlgorithm::Bm25, SearchAlgorithm::Tfidf] {
        let mut cases = Vec::new();
        let mut metrics = Vec::new();
        for query in &fixture.queries {
            let judged = labels(query, &ids)?;
            let start = Instant::now();
            let results = index.search_with_options(
                &query.text,
                &SearchOptions {
                    algorithm,
                    limit: 3,
                    ..SearchOptions::default()
                },
            );
            let elapsed = start.elapsed().as_secs_f64() * 1000.0;
            if let Some(expected) = &query.lexical_top1 {
                assert_eq!(results.first().map(|r| &r.entry.id), Some(expected));
            }
            let (case, metric) = record(
                query,
                results.iter().map(|r| r.entry.id.clone()).collect(),
                results.iter().map(|r| r.score).collect(),
                elapsed,
                &judged,
            )?;
            cases.push(case);
            metrics.push(metric);
        }
        reports.push(
            json!({"mode":format!("{algorithm:?}"),"summary":summarize(&metrics)?,"cases":cases}),
        );
    }
    let args: Vec<_> = std::env::args().skip(1).collect();
    #[cfg(not(feature = "local-embeddings"))]
    let model = Value::Null;
    #[cfg(not(feature = "local-embeddings"))]
    if !args.is_empty() {
        return Err("Enable local-embeddings for explicit model evaluation".into());
    }
    #[cfg(feature = "local-embeddings")]
    let model = if !args.is_empty() {
        use airust::{
            hybrid::HybridSearchOptions,
            semantic::{
                local::FastEmbedProvider, EmbeddingProvider, SemanticIndex, SemanticSearchOptions,
            },
        };
        if args.len() != 2 {
            return Err("Usage: retrieval_quality [MODEL_DIRECTORY ONNX_RUNTIME_LIBRARY]".into());
        }
        let start = Instant::now();
        let mut provider = FastEmbedProvider::multilingual_e5_small(&args[0], &args[1], 2)?;
        let load_ms = start.elapsed().as_secs_f64() * 1000.0;
        let start = Instant::now();
        let semantic = SemanticIndex::from_index(index, &mut provider, 8)?;
        let model = json!({"recipe":provider.space().model(),"dimensions":provider.space().dimensions(),"load_ms":load_ms,"build_ms":start.elapsed().as_secs_f64()*1000.0,"threads":2,"maximum_tokens":512});
        for (name, weights) in [
            ("exact-semantic", None),
            ("hybrid-equal", Some((0.5, 0.5))),
            ("hybrid-semantic-75", Some((0.25, 0.75))),
        ] {
            let mut cases = Vec::new();
            let mut metrics = Vec::new();
            for query in &fixture.queries {
                let judged = labels(query, &ids)?;
                let start = Instant::now();
                let (ranked, scores) = if let Some((lexical_weight, semantic_weight)) = weights {
                    let options = HybridSearchOptions::new(3)
                        .with_weights(lexical_weight, semantic_weight)?;
                    let outcome = semantic.hybrid_search(&query.text, &mut provider, &options)?;
                    (
                        outcome
                            .results()
                            .iter()
                            .map(|r| r.entry.id.clone())
                            .collect(),
                        outcome.results().iter().map(|r| r.score).collect(),
                    )
                } else {
                    let results = semantic.search(
                        &query.text,
                        &mut provider,
                        &SemanticSearchOptions::new(3),
                    )?;
                    (
                        results.iter().map(|r| r.entry.id.clone()).collect(),
                        results.iter().map(|r| r.score).collect(),
                    )
                };
                let (case, metric) = record(
                    query,
                    ranked,
                    scores,
                    start.elapsed().as_secs_f64() * 1000.0,
                    &judged,
                )?;
                cases.push(case);
                metrics.push(metric);
            }
            reports.push(json!({"mode":name,"summary":summarize(&metrics)?,"cases":cases}));
        }
        model
    } else {
        Value::Null
    };
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"airust_version":airust::VERSION,"fixture":fixture.version,"limits":fixture.description,"documents":ids.len(),"queries":fixture.queries.len(),"cutoff":3,"model":model,"reports":reports})
        )?
    );
    Ok(())
}
