# AIRust 0.2.1: optional hybrid retrieval

This release combines the existing lexical and semantic rankings without replacing either implementation or copying source entries. The [plan](PLAN-0.2.1.md) records scope and acceptance. Graphs and the remaining roadmap stages are not part of this release.

The optional `hybrid` feature enables `embeddings` and adds no dependencies. `HybridSearchOptions` configures per-component candidates, BM25 or TF-IDF, cosine eligibility, metadata scope, weighted RRF, and an explicit provider-failure policy. `HybridOutcome` exposes component ranks and raw scores, unique source IDs, matched lexical terms, semantic participation, and any fallback reason.

Provider errors may use an explicitly configured lexical fallback. Model mismatches and malformed embeddings remain errors. Extreme finite weights are scaled before normalization to avoid overflow. Metadata filters apply before ranking and fusion, preserving scoped top results.

## Fixed multilingual comparison

The same twelve-document, sixteen-query public fixture and pinned E5 model used in 0.2.0 compare the existing rankers and two predefined hybrid configurations. Model fingerprint equality with 0.2.0 was verified. No query, document, or relevance label was changed after inspecting these results.

| Method | Correct first result | Recall at three | MRR |
| --- | ---: | ---: | ---: |
| BM25 | 1/16 | 0.1875 | 0.114583 |
| TF-IDF | 1/16 | 0.1875 | 0.114583 |
| Semantic | 16/16 | 1.0000 | 1.000000 |
| Hybrid, equal weights | 9/16 | 0.8125 | 0.717708 |
| Hybrid, 25% lexical and 75% semantic | 11/16 | 0.8125 | 0.780208 |

This fixture focuses on paraphrases and cross-language questions, so literal token retrieval is weak. Pure semantic retrieval performed best here. RRF can promote a lexical match on an unhelpful common word above a semantic-only answer. Those results are retained rather than tuning the fixture or claiming that hybrid always improves retrieval. Exact terminology, identifiers, and other application workloads need their own labeled evaluation.

Raw cases, ranks, scores, and single-pass query timings are in [hybrid-quality.json](measurements/0.2.1/hybrid-quality.json). Query timings exclude model load and document embedding. The model, runtime, license, and artifact fingerprints are recorded in the [0.2.0 measurements](measurements/0.2.0/).

## Synthetic cost measurements

Fifty repetitions of one query used 384-dimensional generated vectors, ten final results, and at most 100 candidates per component. A precomputed provider excludes real-model inference. Measurements use release builds on macOS ARM64 with 64 GiB memory and Rust 1.98.0.

| Entries | Build (ms) | Query p50 (ms) | Query p95 (ms) | Peak process RSS (MiB) |
| ---: | ---: | ---: | ---: | ---: |
| 1,000 | 4.149 | 0.660 | 1.024 | 5.59 |
| 10,000 | 18.257 | 6.667 | 6.857 | 31.47 |

The exact semantic scan dominates this workload. These single-process repeated-query measurements do not establish production tail latency or application speedups. Raw samples and macOS process resource measurements are in [measurements/0.2.1](measurements/0.2.1/). Build `hybrid_benchmark` with the `hybrid` feature before timing its executable.

## Validation and review

The complete check script passed 187 default tests and 252 all-feature tests. The twelve new hybrid tests cover hand-calculated RRF ranks and scores, ID deduplication, stable ties, thresholds, filters before ranking, zero-weight components, visible provider fallback, integrity failures, extreme finite settings, TF-IDF evidence, and 5,000 candidates with 3,333 eligible records. Bounded results matched full-ranking prefixes at the same candidate budget.

Formatting, Clippy with warnings denied, rustdoc, isolated hybrid builds, the unchanged lexical/semantic examples, and the new fusion example passed. Minimum Rust checks retain the 1.89 core/provider boundary and the 1.92 local-model boundary. No dependency was added. The audit retains zero known vulnerabilities and the same two documented optional maintenance warnings.

The boundary review checked that zero weights skip their component, invalid model/vector output cannot trigger an offline fallback, failed searches do not mutate the index, and the temporary fusion map borrows stable source IDs. Candidate-cutoff sensitivity and common-word promotion are documented limitations. Existing 0.2.0 APIs and source formats remain unchanged.

The package contains 117 files and is approximately 240 KiB compressed. Required hybrid modules, examples, shared reference/vector helpers, fixtures, and reports are present; private data and external model/runtime assets are absent. All 252 all-feature tests passed from the extracted crate. An independent Rust 1.89 consumer passed the existing lexical/semantic API and the new hybrid API. The final clean package and publish dry run are verified before upload, followed by registry checksum, tag, asset, and independent-install checks. Release artifacts: [GitHub v0.2.1](https://github.com/LEVOGNE/airust/releases/tag/v0.2.1), [crates.io 0.2.1](https://crates.io/crates/airust/0.2.1).

See [HYBRID.md](HYBRID.md) for the API and cost model and [MIGRATION-0.2.1.md](MIGRATION-0.2.1.md) for compatibility details.
