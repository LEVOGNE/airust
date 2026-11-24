# Reproducible performance and quality checks

The current [reference-v1 report](measurements/reference-v1/README.md) compares nine profiles on 150 frozen, explicitly authored technical-document questions. Its development-selected optional profile, exact model identities, per-query losses, timing boundaries and failed lexical quality targets are retained. This is a separate corpus and method from the historical campaigns below.

The [completed local LLM/file search/SQLite/AIRust comparison](HONEST-COMPARISON-0.4.4.md) reports 400 answers, 100 authored project-document questions, actual token counts, four warm retrieval series, RAM observations and an exploratory 80-answer audit. It does **not** establish an AIRust hybrid quality advantage over SQLite, production accuracy or a general speed claim. Its [method and limitations](validation-honest-0.4.4/README.md) distinguish reference hits, answer proxies and manually reviewed evidence.

Build before measuring. Run timed binaries without concurrent compiler work, retain the lockfile and record host, compiler, feature set, corpus, query counts and model recipe. Separate source generation, indexing, model load/inference, query execution and application/network latency. Raw samples and failures matter as much as summaries.

## Tool choice

Criterion 0.8.2 and its warmup/measurement/bootstrap/comparison process were reviewed using its [official API](https://docs.rs/criterion/0.8.2/criterion/) and [analysis guide](https://bheisler.github.io/criterion.rs/book/analysis.html). It is appropriate for statistically controlled microbenchmark comparisons. AIRust keeps dependency-free integration harnesses using Instant/black_box, fixed warmups/samples and descriptive percentiles, together with OS CPU/RSS counters. These tools do not supply Criterion-style confidence intervals, significance tests or proof of production p99 latency. No Criterion implementation is copied.

## Scale checks

```bash
cargo build --locked --release --features quality,context-assembly,provenance --example quality_benchmark
python3 scripts/measure.py /tmp/quality-1000.json target/release/examples/quality_benchmark 1000 100
python3 scripts/measure.py /tmp/quality-10000.json target/release/examples/quality_benchmark 10000 100
python3 scripts/measure.py /tmp/quality-100000.json target/release/examples/quality_benchmark 100000 100
```

The harness creates multilingual short records, controlled duplicate pairs and unique query keys. Source generation is outside indexing time. It measures rare/common/missing searches, exact/fuzzy matching, duplicate grouping, bounded BFS/DFS and context with source bindings. Most operations use five warmups and one hundred samples; fuzzy uses five measured samples and duplicates at most thirty. The raw arrays preserve observation order. Verify entry/text counts and inspect this method before comparing it with another workload.

It also applies up to one thousand updates/deletes, removes stale source bindings and checks exact query score/statistic agreement after a fresh source-JSON rebuild. Peak RSS includes lexical data, graph, sources, result buffers, JSON and reconstruction staging, not only minimal steady-state index storage. `scripts/measure.py` runs one child and records wall, user/system CPU and peak RSS with Darwin/Linux unit normalization. The Python wrapper is a development tool, not a runtime library dependency.

An optional million-entry lexical-only check uses the existing independent `index_benchmark`. Its workload/method differs and must be reported separately:

```bash
cargo build --locked --release --example index_benchmark
python3 scripts/measure.py /tmp/index-million.json target/release/examples/index_benchmark 1000000 100
```

Earlier release reports contain separate semantic/ANN, graph, context, provenance, cache and revision measurements. In particular, retain the [ANN recall limits](CHANGES-0.2.6.md); a fast approximate query is not a complete relevance ranking.

## Labeled retrieval checks

```bash
cargo run --locked --release --features quality,hybrid --example retrieval_quality
cargo build --locked --release --features quality,hybrid,local-embeddings --example retrieval_quality --example semantic_quality
python3 scripts/measure.py /tmp/retrieval-quality.json target/release/examples/retrieval_quality MODEL_DIRECTORY ONNX_RUNTIME_LIBRARY
python3 scripts/measure.py /tmp/semantic-smoke.json target/release/examples/semantic_quality MODEL_DIRECTORY ONNX_RUNTIME_LIBRARY
```

The new `airust-quality-reference-v1` is separate from the unchanged earlier twelve-document/sixteen-query semantic fixture. It has authored public judgments, including multilingual paraphrases, literal IDs, graded multiple sources, distractors and no-answer cases. It is not an independent benchmark, a logical entailment test or a production guarantee. Unlisted grades are explicit zero only within this closed fixture. Metrics expose the two no-positive queries as undefined recall/nDCG rather than perfect scores. Report query groups and individual failures before choosing weights.

No native weights/runtime are downloaded by Cargo. Supply explicit previously obtained assets. Actual E5 runs record the recipe and inference latency; handwritten-vector examples are API demonstrations only. Graph/context/source reference contracts run in the normal tests and verify scopes, UTF-8 offsets, deduplication, bindings and budgets. Source-binding coverage is not proof that every claim is true.

## Regression policy

Correctness gates cover exact literal IDs, metric reference calculations, scope/source/context contracts, update/restart consistency and the existing semantic smoke expectations. Performance samples are recorded rather than converted into flaky fixed wall-time unit tests. Before a production rollout, compare fixed authorized application queries, their judgments, actual selected evidence and end-to-end latency. Token savings need the target LLM's complete prompt/tokenizer and answer/evidence checks; none are inferred from a smaller node count.

## Native reranking comparison

```bash
python3 scripts/download_reranker.py MODEL_DIRECTORY
cargo build --locked --release --features local-reranking,local-embeddings,hybrid,quality --example reranking_quality
python3 scripts/measure.py /tmp/reranking-reference.json target/release/examples/reranking_quality MODEL_DIRECTORY ONNX_RUNTIME_LIBRARY E5_DIRECTORY
```

The 0.2.9 evaluator uses the unchanged 30-record/36-query reference from 0.2.8. Each comparison reranks an actual top-5 or top-20 candidate prefix from BM25, exact E5 or equal-weight hybrid retrieval. Hybrid component candidate limits remain at their existing default of one hundred. Missing judged positives are never injected. Candidate Recall@N bounds the possible final recall. Cutoff remains three, with explicitly judged zero grades for unlisted records in this closed corpus only.

The model scores query/document pairs with raw logits. It warms once with an obvious pair and checks separate-versus-batched native scores within a declared tolerance. Report model load, warmup, per-query retrieval/rerank/end-to-end latency, query-level before/after metrics and OS CPU/peak RSS. No default score threshold is applied; no-answer failures remain visible. One 36-query series is descriptive, not a confidence interval or production p95 guarantee. The [0.2.9 report](CHANGES-0.2.9.md) links raw data, adverse cases and the exact pinned asset manifest.
