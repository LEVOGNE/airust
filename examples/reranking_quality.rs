use airust::{
    quality::{evaluate_ranking, summarize, Judgments, QueryMetrics, UnjudgedPolicy},
    reranking::{local::FastEmbedReranker, rerank, RerankOptions, Rerankable, Reranker},
    Engine, KnowledgeEntry,
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
}

fn evaluate<T: Rerankable>(
    query: &Query,
    candidates: &[T],
    provider: &mut FastEmbedReranker,
    ids: &BTreeSet<String>,
    window: usize,
    retrieval_ms: f64,
) -> Result<(Value, QueryMetrics, QueryMetrics), Box<dyn Error>> {
    let labels = Judgments::new(
        ids.iter()
            .map(|id| (id.clone(), query.grades.get(id).copied().unwrap_or(0))),
    )?;
    let pool = &candidates[..candidates.len().min(window)];
    let candidate_ids: Vec<_> = pool.iter().map(|hit| hit.entry().id.clone()).collect();
    let original_ids: Vec<_> = candidate_ids.iter().take(3).cloned().collect();
    let before = evaluate_ranking(&original_ids, &labels, 3, UnjudgedPolicy::RequireJudged)?;
    let pool_recall = evaluate_ranking(
        &candidate_ids,
        &labels,
        window,
        UnjudgedPolicy::RequireJudged,
    )?
    .recall_at_k();
    let start = Instant::now();
    let outcome = rerank(
        &query.text,
        pool,
        provider,
        &RerankOptions::new(3).with_candidates(window)?,
    )?;
    let rerank_ms = start.elapsed().as_secs_f64() * 1000.0;
    assert!(outcome.fallback_reason().is_none());
    assert_eq!(outcome.scored_candidates(), pool.len());
    let ranked: Vec<_> = outcome
        .results()
        .iter()
        .map(|r| r.entry().id.clone())
        .collect();
    let after = evaluate_ranking(&ranked, &labels, 3, UnjudgedPolicy::RequireJudged)?;
    assert!(after
        .recall_at_k()
        .is_none_or(|v| v <= pool_recall.unwrap() + 1e-12));
    let scores: Vec<_> = outcome.results().iter().map(|r| r.score.unwrap()).collect();
    let input_ranks: Vec<_> = outcome.results().iter().map(|r| r.original_rank).collect();
    Ok((
        json!({"query_id":query.id,"query":query.text,"kind":query.kind,"grades":query.grades,"candidate_ids":candidate_ids,"candidate_recall":pool_recall,"baseline_ids":original_ids,"reranked_ids":ranked,"raw_logits":scores,"original_ranks":input_ranks,"baseline_metrics":before,"reranked_metrics":after,"retrieval_ms":retrieval_ms,"rerank_ms":rerank_ms,"end_to_end_ms":retrieval_ms+rerank_ms}),
        before,
        after,
    ))
}
fn report(
    mode: &str,
    window: usize,
    cases: Vec<Value>,
    before: Vec<QueryMetrics>,
    after: Vec<QueryMetrics>,
) -> Result<Value, Box<dyn Error>> {
    let deltas = cases
        .iter()
        .filter(|v| v["baseline_ids"] != v["reranked_ids"])
        .count();
    Ok(
        json!({"retriever":mode,"candidate_window":window,"cutoff":3,"baseline_summary":summarize(&before)?,"reranked_summary":summarize(&after)?,"changed_queries":deltas,"cases":cases}),
    )
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if !(2..=3).contains(&args.len()) {
        return Err("Usage: reranking_quality RERANK_MODEL_DIRECTORY ONNX_RUNTIME_LIBRARY [E5_MODEL_DIRECTORY]".into());
    }
    #[cfg(not(all(feature = "local-embeddings", feature = "hybrid")))]
    if args.len() == 3 {
        return Err("Enable local-embeddings,hybrid for E5 evaluation".into());
    }
    let fixture: Fixture =
        serde_json::from_str(include_str!("../tests/fixtures/quality-reference-v1.json"))?;
    let ids: BTreeSet<_> = fixture.documents.iter().map(|e| e.id.clone()).collect();
    assert_eq!(ids.len(), fixture.documents.len());
    let mut queries = BTreeSet::new();
    for q in &fixture.queries {
        assert!(
            !q.text.trim().is_empty()
                && queries.insert(&q.id)
                && q.grades.keys().all(|id| ids.contains(id))
        );
    }
    let index = Engine::from_entries(fixture.documents)?;
    let start = Instant::now();
    let mut provider = FastEmbedReranker::multilingual_minilm_v2(&args[0], &args[1], 2)?;
    let load_ms = start.elapsed().as_secs_f64() * 1000.0;
    let warmup = Instant::now();
    let probe = [
        "Mars is known as the Red Planet.",
        "A loaf of bread is baked in an oven.",
    ];
    let scores = provider.score("Which planet is called the Red Planet?", &probe)?;
    assert_eq!(scores.len(), 2);
    assert!(scores.iter().all(|s| s.is_finite()));
    assert!(scores[0] > scores[1]);
    let mut split = Vec::new();
    for text in probe {
        split.extend(provider.score("Which planet is called the Red Planet?", &[text])?);
    }
    assert!(scores.iter().zip(&split).all(|(a, b)| (a - b).abs() < 1e-3));
    let model = json!({"recipe":provider.recipe(),"load_ms":load_ms,"warmup_ms":warmup.elapsed().as_secs_f64()*1000.0,"native_probe_logits":scores,"batch_consistency_tolerance":1e-3,"threads":2,"maximum_pair_tokens":512,"score_kind":"raw_single_logit_not_probability","runtime":args[1]});
    let mut reports = Vec::new();
    for window in [5, 20] {
        let (mut cases, mut before, mut after) = (Vec::new(), Vec::new(), Vec::new());
        for query in &fixture.queries {
            let start = Instant::now();
            let candidates = index.search(&query.text, window);
            let retrieval_ms = start.elapsed().as_secs_f64() * 1000.0;
            let (case, a, b) = evaluate(
                query,
                &candidates,
                &mut provider,
                &ids,
                window,
                retrieval_ms,
            )?;
            cases.push(case);
            before.push(a);
            after.push(b);
        }
        reports.push(report("BM25", window, cases, before, after)?);
    }
    #[cfg(all(feature = "local-embeddings", feature = "hybrid"))]
    let embedding = if args.len() == 3 {
        use airust::{
            hybrid::HybridSearchOptions,
            semantic::{
                local::FastEmbedProvider, EmbeddingProvider, SemanticIndex, SemanticSearchOptions,
            },
        };
        let start = Instant::now();
        let mut embeddings = FastEmbedProvider::multilingual_e5_small(&args[2], &args[1], 2)?;
        let embedding_load_ms = start.elapsed().as_secs_f64() * 1000.0;
        let start = Instant::now();
        let semantic = SemanticIndex::from_index(index, &mut embeddings, 8)?;
        let embedding = json!({"recipe":embeddings.space().model(),"load_ms":embedding_load_ms,"index_build_ms":start.elapsed().as_secs_f64()*1000.0,"dimensions":embeddings.space().dimensions(),"threads":2});
        for mode in ["exact-semantic", "hybrid-equal"] {
            for window in [5, 20] {
                let (mut cases, mut before, mut after) = (Vec::new(), Vec::new(), Vec::new());
                for query in &fixture.queries {
                    let start = Instant::now();
                    let (case, a, b) = if mode == "exact-semantic" {
                        let candidates = semantic.search(
                            &query.text,
                            &mut embeddings,
                            &SemanticSearchOptions::new(window),
                        )?;
                        let ms = start.elapsed().as_secs_f64() * 1000.0;
                        evaluate(query, &candidates, &mut provider, &ids, window, ms)?
                    } else {
                        let candidates = semantic.hybrid_search(
                            &query.text,
                            &mut embeddings,
                            &HybridSearchOptions::new(window),
                        )?;
                        let ms = start.elapsed().as_secs_f64() * 1000.0;
                        evaluate(query, candidates.results(), &mut provider, &ids, window, ms)?
                    };
                    cases.push(case);
                    before.push(a);
                    after.push(b);
                }
                reports.push(report(mode, window, cases, before, after)?);
            }
        }
        embedding
    } else {
        Value::Null
    };
    #[cfg(not(all(feature = "local-embeddings", feature = "hybrid")))]
    let embedding = Value::Null;
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"airust_version":airust::VERSION,"fixture":fixture.version,"limits":fixture.description,"documents":ids.len(),"queries":fixture.queries.len(),"candidate_selection":"unchanged first-stage ranked prefix; no positive documents injected; candidate recall is the maximum possible reranked recall","minimum_logit":null,"model":model,"embedding_model":embedding,"reports":reports})
        )?
    );
    Ok(())
}
