# AIRust 0.2.6: optional HNSW candidates

The optional `hnsw` feature adds configurable approximate semantic candidate retrieval. Existing semantic, hybrid and context APIs retain exact retrieval; default features remain empty. No model, application or MCP integration is added. The wrapper owns a semantic index and a derived topology, shares its vector buffers, and returns borrowed original entries with exact final cosine scores.

Official-source comparison rejected the archived instant-distance upstream and reviewed the older rust-cv hnsw implementation. The selected pinned hnsw_rs 0.3.5 has an active upstream, a generic distance interface and MIT/Apache-2.0 licensing. AIRust's adapter stores one shared vector handle per point and reuses the backend's existing anndists L2 function over normalized embeddings for navigation. Existing cosine scoring orders the selected live candidates; no third-party implementation is copied. Its optional dependency tree includes threading and memory-map support, but this adapter opens no mapped file and enables no model download.

Private validated configuration prevents invalid connection/layer/beam values reaching upstream exit or invalid-range paths. Serial sorted-ID insertion uses the pinned backend's fixed layer-generator seed. Small-corpus results and stable final tie ordering are regression-tested; cross-platform topology identity is not promised. Approximate candidate recall is separate from correct final scores.

Updates and deletions mark old base IDs stale and search live changed vectors exactly. Explicit compaction replaces topology and resets the overlay. Pending-ID policy switches to exact search when exceeded; it is not a memory ceiling. Old vector buffers can remain until compaction, whose temporary memory includes both topologies. Scoped requests use exact filtering rather than dropping authorized answers from an unscoped ANN top-K. Small-index and oversized-result-window fallbacks also report their modes. No automatic rebuild or mutation of a separately borrowed semantic index is permitted.

## Validation and review

Nine integration tests check configuration boundaries, complete candidate/exact score agreement, thresholds, stable ties, empty/zero-limit/provider behavior, identity errors, failed update/batch atomicity, scopes, pending policy, 150 mutations with repeated compaction, and a 700-entry narrow-window scenario with replacements, deletions and a new best result. A private test checks vector pointer identity in backend points and replacement/compaction, confirming shared buffers rather than a second vector copy. The compiled ANN guide exercises the default small-index path.

The complete script passes 189 default tests. The final all-feature suite passes 325 tests, with the existing crash-helper entry point intentionally ignored and exercised by its parent. Strict Clippy, formatting, rustdoc, isolated features and examples pass. Provider-independent features including HNSW pass Rust 1.89; complete features pass Rust 1.92. An independent Rust 1.89 consumer uses old APIs, cached semantic vectors, explicit ANN search, deletion and compaction. The audit reports zero known vulnerabilities and the same optional unmaintained `paste` and `ttf-parser` warnings.

## Measurement method

`examples/ann_benchmark.rs` builds normalized 384-dimensional vectors with two deterministic generators: 64 clusters with independent query noise, and independent pseudorandom unit vectors. It builds lexical/vector state, constructs serial topology (16 connections, 16 layers, construction width 200), and compares four search widths against exact top ten for one hundred queries each. It records recall@10, p50/p95 query time, all samples, topology/vector construction time and OS peak RSS. Query vectors are generated independently from indexed IDs. Every returned score is compared bit-for-bit to the existing exact cosine function.

These are synthetic navigation tests, not labeled semantic relevance judgments or production application measurements. Random high-dimensional neighbors can be difficult to approximate. Wider search can improve recall with additional cost; approximation is not selected automatically by any existing retrieval API. A result's cosine score does not certify that other relevant candidates were found. Measure the target corpus and retain exact retrieval when recall is inadequate. Default construction/search parameters are starting values, not a quality guarantee.

Final measurements are recorded below. Raw samples and resource counters are in [measurements/0.2.6](measurements/0.2.6/). Host: macOS ARM64, 64 GiB memory, Rust 1.98.0, release build. No embedding inference, network or model is timed. Peak memory includes source entries, lexical structures, vector buffers, graph, allocator retention and query scratch; it is not vector-only storage.

## Compatibility and recovery

The feature and wrapper are additive in Cargo's 0.2 range. Existing cache format 1 preserves lexical/vector data; HNSW topology is rebuilt after loading a semantic cache. Graph/context/provenance data and source permissions remain application-owned. Stable IDs, source JSON, exact scores and existing methods are preserved. See [ANN.md](ANN.md), [migration notes](MIGRATION-0.2.6.md) and [implementation plan](PLAN-0.2.6.md).

Clean package/extracted tests, candidate CI, dry run and registry/tag/asset verification gate publication and the next stage. Artifacts: [GitHub v0.2.6](https://github.com/LEVOGNE/airust/releases/tag/v0.2.6), [crates.io 0.2.6](https://crates.io/crates/airust/0.2.6).

## Final standalone measurements

Construction uses one serial build per case. Query columns show one hundred samples at each width; CPU user/system and wall counters for the full process are included in the raw `.time.txt` files. Exact and ANN queries use the same source/vector state. The process runs without concurrent compiler work; normal background applications remain active.

| Distribution | Entries | Vector/lexical build (ms) | HNSW build (ms) | Peak RSS (MiB) |
| --- | ---: | ---: | ---: | ---: |
| clustered | 1,000 | 4.151 | 158.609 | 8.83 |
| clustered | 10,000 | 24.407 | 2412.616 | 55.50 |
| clustered | 100,000 | 304.351 | 65875.831 | 577.73 |
| random | 1,000 | 5.133 | 653.039 | 10.38 |
| random | 10,000 | 53.711 | 32419.762 | 65.16 |
| random | 100,000 | 583.727 | 535468.533 | 602.44 |

Recall@10 is set overlap with the exact ten results, averaged across the hundred queries. Correct cosine scores do not remove candidate-recall loss.

| Distribution | Entries | Width 40 | Width 100 | Width 250 | Width 1000 |
| --- | ---: | ---: | ---: | ---: | ---: |
| clustered | 1,000 | 1.000 | 1.000 | 1.000 | 1.000 |
| clustered | 10,000 | 0.999 | 1.000 | 1.000 | 1.000 |
| clustered | 100,000 | 0.602 | 0.701 | 0.769 | 0.890 |
| random | 1,000 | 0.909 | 0.998 | 1.000 | 1.000 |
| random | 10,000 | 0.395 | 0.632 | 0.872 | 0.998 |
| random | 100,000 | 0.072 | 0.145 | 0.308 | 0.685 |

Selected default-width and wider-width latency comparisons (milliseconds):

| Distribution | Entries | Width | Exact p50 | Exact p95 | ANN p50 | ANN p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| clustered | 1,000 | 100 | 0.639 | 0.665 | 0.148 | 0.168 |
| clustered | 1,000 | 1000 | 0.638 | 0.660 | 1.156 | 1.199 |
| clustered | 10,000 | 100 | 7.297 | 7.386 | 0.366 | 0.448 |
| clustered | 10,000 | 1000 | 7.636 | 7.751 | 2.474 | 2.649 |
| clustered | 100,000 | 100 | 123.105 | 143.254 | 1.873 | 2.399 |
| clustered | 100,000 | 1000 | 141.431 | 149.011 | 6.150 | 8.383 |
| random | 1,000 | 100 | 1.394 | 1.496 | 0.758 | 0.965 |
| random | 1,000 | 1000 | 1.396 | 1.488 | 2.803 | 3.136 |
| random | 10,000 | 100 | 13.629 | 14.228 | 3.475 | 4.328 |
| random | 10,000 | 1000 | 13.552 | 14.406 | 13.814 | 16.385 |
| random | 100,000 | 100 | 113.310 | 128.078 | 3.664 | 4.666 |
| random | 100,000 | 1000 | 102.908 | 128.094 | 25.243 | 27.679 |

Wider search is not automatically worthwhile: it can approach or exceed exact-search latency, while difficult large workloads can still lose recall. Construction can take minutes. Treat the defaults as configurable starting points and evaluate actual authorized application queries before enabling ANN. Keep exact search for workloads that require complete candidate ranking. These samples do not establish production throughput, a universal recall target or long-tail latency.
