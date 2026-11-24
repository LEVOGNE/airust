# Optional hybrid retrieval

Enable `hybrid` to combine the existing BM25 or TF-IDF ranking with exact semantic retrieval. It also enables `embeddings` and adds no dependencies. Source entries and all existing algorithms are reused.

```toml
airust = { version = "=0.2.1", default-features = false, features = ["hybrid"] }
```

Run the model-independent API example with `cargo run --locked --features hybrid --example hybrid`. The explicit real-model comparison is `cargo run --locked --release --features hybrid,local-embeddings --example hybrid_quality -- MODEL_DIRECTORY ONNX_RUNTIME_LIBRARY`.

`SemanticIndex::hybrid_search(query, provider, options)` returns `HybridOutcome`. `results()` borrows its ranked hits; `into_results()` consumes the outcome. `semantic_used()` records successful semantic participation and `fallback_reason()` records an explicitly requested provider-error fallback.

Each hit borrows its source `entry` and contains the fused score, optional component ranks and raw scores, and any lexical matched terms. Duplicate IDs appear once. Equal fused scores use ascending IDs.

## Ranking and configuration

The score is `w_lexical / (k + rank_lexical) + w_semantic / (k + rank_semantic)`, omitting absent components. Ranks start at one. Weights are normalized to sum to one, including safely handling extreme finite values. BM25 and cosine values are retained as evidence but are not added together. The fused score is not a probability.

`HybridSearchOptions::new(limit)` defaults to at least 100 candidates per component, BM25, smoothing `k = 60`, equal weights, the full cosine range, no context filter, and `ReturnError` on provider failure.

Builders configure `with_candidates`, `with_rrf_k`, `with_weights`, `with_lexical_algorithm`, `with_bm25`, `with_min_score`, `with_context`, and `with_failure_policy`. Getters expose the accepted settings. Candidate limits must be positive and at least the result limit. Smoothing must be finite and nonnegative. Weights must be finite, nonnegative, and not both zero. Cosine thresholds use the existing semantic validation.

The cosine threshold affects the semantic component only. An entry with lexical evidence can still appear without an eligible semantic contribution.

A zero component weight skips that component. A zero result limit or empty index skips inference. Whitespace-only queries are errors. Metadata and ID filters apply within both component rankings before fusion, preserving scoped top results.

## Explicit fallback

`SemanticFailurePolicy::ReturnError` is the default. `LexicalOnly` permits a provider failure to return lexical results with effective lexical weight one and a visible fallback reason. It can therefore recover even when the normal lexical weight is zero. Invalid vectors, model-space mismatches, and other integrity errors still fail; they are not silently converted into successful semantic retrieval.

Provider errors originate in application-owned code. Applications control what is safe to display or log from the fallback message. No HTTP route or UI is added by this feature.

## Limits

Fusion considers only the configured candidates from each component. A record below both candidate cutoffs cannot appear in the output. Increasing the cutoff may change rankings and costs. Exact semantic scoring still scans the eligible vector index. Fusion retains at most twice the candidate count in its temporary ID map and keeps final results through the shared bounded heap.

Hybrid retrieval does not necessarily outperform either component on every corpus. Evaluate the weights, cutoff, cosine threshold, and query workload with labeled data. The fixed multilingual release fixture is a smoke test, not a universal quality claim.

Primary method reference: [Cormack, Clarke, and Buettcher, SIGIR 2009](https://plg.uwaterloo.ca/~gvcormac/cormacksigir09-rrf.pdf).
