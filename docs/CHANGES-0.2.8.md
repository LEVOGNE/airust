# AIRust 0.2.8: reproducible quality and complete API guidance

This release completes the requested 0.2.0–0.2.8 sequence. The independent `quality` feature adds graded retrieval metrics without another dependency. Existing default features, source entries/JSON, retrieval behavior, graph, provenance, cache and revision formats are preserved. No DenkMaps or MCP integration is added to the core. Proposed 0.2.9–0.2.12 work remains a roadmap, not implemented functionality.

The [API cookbook](API-COOKBOOK.md) explains all new modules, public operations, ownership choices and practical examples. The [DenkMaps upgrade guide](DENKMAPS-UPGRADE.md) describes a future shadow-tested upgrade of its inspected existing 0.1.9 adapter while retaining SQLite authority. It performs none of those production changes. The compiled generic `knowledge_pipeline` demonstrates scoped retrieval, context, graph/source bindings, cache and metrics with explicitly handwritten vectors; actual model quality is evaluated separately.

## Quality API and review

`Judgments` validates IDs and grades 0..31. `evaluate_ranking` requires a positive cutoff and explicit unjudged-result policy, rejects duplicates in the evaluated window and exposes returned/judged counts. Precision divides by K, recall by all positive judgments, reciprocal rank uses the first relevant position, and nDCG uses exponential gain and an ideal ranking at K. No positive labels means undefined recall/nDCG, not perfect scores. `summarize` reports compatible macro means and defined query counts. Summary count arithmetic is checked. The [quality guide](QUALITY.md) states the formulas and primary references.

Eight quality tests include hand-computed graded values, empty/short lists, cutoff behavior, unknown/duplicate IDs, extreme grades, summary denominators and all small ranking permutations. Eight manually specified literal IDs in the public fixture have independently expected first hits. Two additional context contracts check authorized evidence selection, excluded graph bridges, source bindings, UTF-8 excerpts, exact-normalized duplicate sources, budgets and no-match behavior. Existing algorithm, cache-crash and version-conflict tests remain in the complete suite.

The new separately versioned corpus contains 30 public authored records and 36 requests. The earlier twelve-record/sixteen-query semantic fixture is unchanged. New labels include graded multiple sources, exact identifiers, multilingual paraphrases, distractors and two no-answer cases. Unlisted grades are explicitly zero within this closed fixture only. This is developer-authored smoke evaluation, not an independent relevance benchmark, training/test split, entailment proof or production quality guarantee. Parameters were not promoted as universally optimal from these examples.

## Actual local E5 retrieval

Five modes use the same corpus and cutoff three. The model is multilingual-e5-small, 384 dimensions, two CPU threads and a 512-token maximum, with the explicitly supplied verified recipe recorded in the raw result. No model/runtime downloads occur during Cargo builds. Each query/mode retains IDs, ranks, grades, scores, latency and metrics in [retrieval-e5.json](measurements/0.2.8/retrieval-e5.json).

| Mode | Precision@3 | Recall@3 | MRR@3 | nDCG@3 |
| --- | ---: | ---: | ---: | ---: |
| Bm25 | 0.185 | 0.426 | 0.412 | 0.394 |
| Tfidf | 0.194 | 0.456 | 0.421 | 0.409 |
| exact-semantic | 0.370 | 0.971 | 0.806 | 0.858 |
| hybrid-equal | 0.269 | 0.676 | 0.579 | 0.598 |
| hybrid-semantic-75 | 0.269 | 0.676 | 0.606 | 0.607 |

Precision and MRR average all 36 queries; recall/nDCG use the 34 with positive judgments. The two no-answer cases remain undefined for those denominators. Known positive records are often fewer than K, so perfect retrieval of one positive need not produce Precision@3 of one.

Reciprocal rank by query group makes strengths and failures visible:

| Group | Queries | BM25 | Exact E5 | Hybrid equal | Hybrid semantic 75% |
| --- | ---: | ---: | ---: | ---: | ---: |
| multilingual-paraphrase | 16 | 0.052 | 0.875 | 0.427 | 0.490 |
| literal-identifier | 8 | 1.000 | 0.938 | 1.000 | 1.000 |
| graded-multiple | 4 | 1.000 | 1.000 | 1.000 | 1.000 |
| negation-distractor | 1 | 1.000 | 1.000 | 1.000 | 1.000 |
| temporal-distractor | 1 | 0.000 | 0.500 | 0.000 | 0.000 |
| action-distractor | 3 | 0.000 | 0.333 | 0.000 | 0.000 |
| code-name | 1 | 1.000 | 1.000 | 1.000 | 1.000 |
| no-answer | 2 | 0.000 | 0.000 | 0.000 | 0.000 |

E5's model load was 1575.560 ms and thirty-document embedding/index construction 86.233 ms. Peak process memory was 1587.05 MiB. The recipe ID is `intfloat/multilingual-e5-small:sha256:bdeaf1db1a21c209f990af1bd3faa13d439d214ebc80e4e3dc73d76c9fa79456`. Native query timings include inference; raw samples retain cold/warm ordering rather than claiming production-tail percentiles from thirty-six questions.

The unchanged original semantic smoke test passed again with top-one accuracy, Recall@3 and MRR all 1.0. Its smaller/easier corpus is not directly comparable with the new distractor set; see [semantic-original.json](measurements/0.2.8/semantic-original.json).

Material failures are deliberately preserved. For “create a new backup, do not delete existing copies,” E5's top three omitted the judged backup note and preferred the deletion distractor. Archived weather, account recovery and bread-storage notes were first for other contrasting requests, ahead of the judged answer. Semantic/hybrid modes returned three candidates for each no-answer request under their default thresholds, while lexical modes returned none. Hybrid improved exact-identifier behavior here but was worse overall than exact semantic retrieval. These observations support explicit corpus evaluation and calibrated fallback/abstention; they do not establish reliable logical understanding or contradiction detection.

## Standalone scale/resource matrix

Host: Apple M5 Max, 18 logical CPUs, 64 GiB RAM, macOS 27.0.1 ARM64, Rust 1.98.0, release/thin-LTO build. Timed processes ran without concurrent compilation. Source generation is outside the main lexical-build timer. A four-ary graph and shared source bindings are timed separately. Most operations use five warmups and one hundred measured observations; fuzzy matching uses five observations and duplicate grouping thirty. Source revisions, scopes and native inference are separate from these synthetic lexical cases.

| Entries | Lexical build (ms) | Graph/source build (ms) | Up to 1000 changes (ms) | Peak RSS (MiB) |
| ---: | ---: | ---: | ---: | ---: |
| 1,000 | 4.255 | 2.129 | 2.788 | 8.17 |
| 10,000 | 25.344 | 9.468 | 3.132 | 52.41 |
| 100,000 | 284.237 | 95.955 | 3.488 | 486.72 |

Query latency in milliseconds:

| Entries | Operation | Samples | p50 | p95 | p99 |
| ---: | --- | ---: | ---: | ---: | ---: |
| 1,000 | context_with_sources | 100 | 0.005625 | 0.013167 | 0.016875 |
| 1,000 | duplicate_groups | 30 | 0.002458 | 0.006584 | 0.009083 |
| 1,000 | graph_bfs | 100 | 0.004542 | 0.005625 | 0.005833 |
| 1,000 | graph_dfs | 100 | 0.004542 | 0.005209 | 0.006458 |
| 1,000 | matching_exact | 100 | 0.001542 | 0.002334 | 0.003000 |
| 1,000 | matching_fuzzy | 5 | 0.709334 | 0.771792 | 0.771792 |
| 1,000 | search_common | 100 | 0.372334 | 0.463542 | 0.520917 |
| 1,000 | search_miss | 100 | 0.000333 | 0.000334 | 0.000334 |
| 1,000 | search_rare | 100 | 0.000750 | 0.000875 | 0.000875 |
| 10,000 | context_with_sources | 100 | 0.009833 | 0.019084 | 0.022166 |
| 10,000 | duplicate_groups | 30 | 0.028708 | 0.035041 | 0.044792 |
| 10,000 | graph_bfs | 100 | 0.004584 | 0.004875 | 0.006000 |
| 10,000 | graph_dfs | 100 | 0.004500 | 0.004667 | 0.005542 |
| 10,000 | matching_exact | 100 | 0.003291 | 0.007666 | 0.010458 |
| 10,000 | matching_fuzzy | 5 | 9.297458 | 9.690416 | 9.690416 |
| 10,000 | search_common | 100 | 3.920125 | 4.278875 | 4.469416 |
| 10,000 | search_miss | 100 | 0.000375 | 0.000417 | 0.000458 |
| 10,000 | search_rare | 100 | 0.000542 | 0.000625 | 0.000667 |
| 100,000 | context_with_sources | 100 | 0.017416 | 0.029250 | 0.033291 |
| 100,000 | duplicate_groups | 30 | 0.594458 | 0.820833 | 1.056709 |
| 100,000 | graph_bfs | 100 | 0.004916 | 0.006042 | 0.010542 |
| 100,000 | graph_dfs | 100 | 0.004750 | 0.004792 | 0.004833 |
| 100,000 | matching_exact | 100 | 0.006375 | 0.011000 | 0.011958 |
| 100,000 | matching_fuzzy | 5 | 72.286791 | 73.449250 | 73.449250 |
| 100,000 | search_common | 100 | 48.904541 | 51.608666 | 52.469167 |
| 100,000 | search_miss | 100 | 0.000416 | 0.000417 | 0.000458 |
| 100,000 | search_rare | 100 | 0.000708 | 0.000875 | 0.001000 |

Rare queries match a unique key; the common query has a matching term in every record. Exact and fuzzy matching are distinct operations. Graph traversals are bounded at 100 visits; context is bounded at twenty source IDs, 4096 text bytes and 256 bytes per excerpt. Context/source preflight checks validate returned text against original UTF-8 spans. After mutations/deletions, stale source bindings are removed and current source JSON is rebuilt; query-score bits and index statistics agree.

Peak RSS includes lexical state, graph, sources, buffers, JSON and fresh-rebuild staging. It is not minimal index-only memory. Single runs and small p99/fuzzy sample counts do not establish statistical significance or production tail latency. Criterion 0.8.2's statistical process was reviewed; this dependency-free integration harness reports descriptive samples rather than claiming equivalent bootstrap confidence. See [BENCHMARKS.md](BENCHMARKS.md) for exact commands and limits.

## Optional million-entry lexical pass

A separate existing lexical-only workload indexed one million five-token records, with ten thousand query buckets and approximately one hundred candidates per query. It does not include the graph/context matrix or a query matching every record.

- Build including source generation: 2698.727 ms.
- One hundred query observations after twenty warmups: p50 0.236541 ms, p95 0.370250 ms, p99 0.445584 ms.
- One thousand updates: 3.320 ms; deletions: 0.906 ms.
- Peak RSS: 1530.73 MiB; end count 999,000 entries.

[Raw latency observations](measurements/0.2.8/lexical-million.json) and OS counters retain this separate method. The result is not a universal million-node application-capacity or common-query claim. Earlier [HNSW measurements](CHANGES-0.2.6.md), including substantial random-vector recall loss and expensive construction, remain applicable; this release does not silently select ANN.

## Release checks and compatibility

The complete script passes **189 default tests** and **350 all-feature tests**, plus executable examples and the small debug harness smoke run. The existing intentionally ignored crash-child entry point is exercised by its parent test. Formatting, strict Clippy, rustdoc, isolated features, Rust 1.89 provider-independent checks and Rust 1.92 complete-feature checks pass. An independent Rust 1.89 consumer checks old APIs through the new metric interface. The optional million-entry harness now also retains raw latency order and warmup counts.

The official checkout action was updated from v4 to verified v7.0.1, pinned at `3d3c42e5aac5ba805825da76410c181273ba90b1`, using Node 24. CI disables persisted checkout credentials and limits its token to contents:read. The existing unsafe-code prohibition remains enabled.

A fresh RustSec fetch/audit on 17 November found zero known vulnerabilities against 1509 advisories, commit `02e2517f457a359fc6d3440ea4560c8ea188fee8` (latest database update 2026-11-15T10:59:21+01:00). Two optional unmaintained warnings remain: [paste](https://rustsec.org/advisories/RUSTSEC-2024-0436.html), via tokenizers/local embeddings, and [ttf-parser](https://rustsec.org/advisories/RUSTSEC-2026-0192.html), via PDF handling. These are not suppressed or represented as a completed production security audit. [Audit metadata](measurements/0.2.8/audit.json) is preserved.

Clean packaging, extracted-crate tests, candidate CI, publish dry run and registry/tag/asset verification gate publication. Existing 0.2 APIs/defaults/formats are preserved; see [migration notes](MIGRATION-0.2.8.md). Application integration and the separately requested existing DenkMaps MCP progress work remain separate from this core release. Artifacts: [GitHub v0.2.8](https://github.com/LEVOGNE/airust/releases/tag/v0.2.8), [crates.io 0.2.8](https://crates.io/crates/airust/0.2.8).
