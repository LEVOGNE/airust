# AIRust 0.2.0: optional embeddings

This release adds provider-neutral embeddings and exact semantic retrieval while preserving the existing lexical engine. The [implementation plan](PLAN-0.2.0.md) records scope, reuse, and acceptance criteria. No DenkMaps or MCP integration is included.

## Implementation

The `embeddings` feature adds no dependencies. It provides model-space identities, finite and nonzero vector validation, stable cosine arithmetic, checked provider batches, exact semantic ranking, and atomic incremental updates. It reuses `KnowledgeIndex`, `Context`, and `TopK` rather than duplicating source entries or lexical algorithms.

The `local-embeddings` feature adds FastEmbed 7.7.1, dynamically loaded ONNX Runtime bindings, and SHA-256 fingerprints for explicit local E5 model files. Upstream hub access and native binary downloads are disabled. Core support remains Rust 1.89; the optional ONNX path requires Rust 1.92. No models or runtime binaries are included in the crate.

The [semantic API guide](SEMANTIC.md), [local provider guide](LOCAL-EMBEDDINGS.md), and [migration guide](MIGRATION-0.2.0.md) describe the contracts and requirements.

## Semantic reference evaluation

The fixed fixture has twelve public documents and sixteen labeled queries in German, English, French, and Spanish. It includes vehicle-registration paraphrases with no lexical token overlap, cross-language queries, and unrelated distractors. Release smoke gates were set before inspecting results: top-one accuracy at least 0.75, recall at three at least 0.875, and the German registration case in the first three.

The evaluated model is `intfloat/multilingual-e5-small`, revision `614241f622f53c4eeff9890bdc4f31cfecc418b3`, using 384 dimensions, mean pooling, 512-token truncation, and E5 query/document prefixes. The external ONNX Runtime is 1.32.0 on macOS ARM64, with two CPU threads.

All sixteen queries retrieved their labeled record first: top-one accuracy, recall at three, and MRR were each 1.0 on this fixture. Identical document embeddings had cosine 1.0. This does not establish comparable accuracy on other corpora.

The measured model load was 1,353.975 ms, including file reads, hashing, and native initialization. Embedding and indexing the twelve documents took 21.479 ms. Peak process RSS was 1,586.36 MiB, including model-loading and native-runtime overhead. Reusing a loaded provider avoids repeating that initialization.

Raw cases, query timings, model/tokenizer hashes, runtime archive hash, and resource output are in [measurements/0.2.0](measurements/0.2.0/). The public fixture was fixed before the quality results were inspected.

## Exact-vector measurements

The release example generated deterministic 384-dimensional vectors, built a source and vector index, and executed fifty exact queries with result limit ten. Inference is excluded. These numbers describe this synthetic workload on the same ARM64 machine with 64 GiB memory; they are not application speedups or ANN measurements.

| Vectors | Build (ms) | Query p50 (ms) | Query p95 (ms) | Peak RSS (MiB) | 100 replacements (ms) | 100 deletions (ms) |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1,000 | 2.432 | 0.742 | 0.817 | 6.28 | 0.289 | 0.060 |
| 10,000 | 19.255 | 6.203 | 6.400 | 39.31 | 0.333 | 0.048 |

Build before timing to exclude compilation:

```bash
cargo build --locked --release --features embeddings --example embedding_benchmark
/usr/bin/time -l target/release/examples/embedding_benchmark 1000
/usr/bin/time -l target/release/examples/embedding_benchmark 10000
```

The example records all sorted latency samples. Its p50 and p95 use indices 24 and 46 of fifty values. The macOS resource command measures the complete process. Queries are deterministic self-retrieval cases rather than a realistic independent query distribution.

## Validation and boundary review

The complete validation script passed: 187 default-feature tests and 240 all-feature tests, including documentation examples. Formatting, default/all-feature/CLI Clippy with warnings denied, isolated feature builds, rustdoc with warnings denied, and the runnable examples passed. The original CLI tests need local process-inspection permissions; the final successful run used them.

Rust 1.89 checked the provider-neutral feature and existing optional interfaces. Rust 1.92 checked all features and targets, including local inference. Feature-tree inspection confirmed that the local adapter enables neither model-hub access, HTTP clients, nor native binary-download features.

Fourteen semantic tests exercise dimension errors, NaN/infinity/zero vectors, extreme finite values, model mismatches, provider output counts, model changes, empty inputs, batch failures after an earlier successful chunk, duplicate IDs, invalid replacements, metadata filtering before top-result selection, and coordinated removal. Exact rankings were compared against a full cosine reference for 143 generated vectors and seven result limits. The real E5 evaluation is separate from deterministic provider fixtures.

The boundary review checked that failed provider batches do not change source entries or vectors, immutable lexical access cannot bypass vector maintenance, and source text is borrowed during inference. No additional defect was found in these checked cases. This is not an independent security audit or an exhaustive correctness proof.

The dependency audit reported zero known vulnerabilities. It retains two maintenance warnings: `paste 1.0.15` through the optional FastEmbed/tokenizers path (`RUSTSEC-2024-0436`) and `ttf-parser 0.25.1` through optional PDF support (`RUSTSEC-2026-0192`). These warnings are not suppressed. The advisory database and warning summary are recorded with the measurements.

Existing public lexical methods, entry fields, formats, and empty default features are preserved. New semantic errors and purpose/result types allow future extension without requiring exhaustive external matches or construction. Cargo's 0.1.x range does not select 0.2.x; future 0.2.x additions must preserve the established API.

The verified package contains 100 files and is approximately 224 KiB compressed. Required semantic modules, model-independent tests, public quality fixtures, documentation, and measurement summaries are present. Private corpora, databases, model weights, runtime libraries, and backups are absent. All 240 all-feature tests also passed from an independently extracted package.

An independent Rust 1.89 consumer compiled and ran the existing lexical APIs and the new provider-neutral semantic API, including updates, filters, removal, and ownership recovery. A missing native runtime returned a provider error rather than panicking. The final publish dry run checks the clean release source separately. No later roadmap feature is included in this release.

Release artifacts: [GitHub v0.2.0](https://github.com/LEVOGNE/airust/releases/tag/v0.2.0), [crates.io 0.2.0](https://crates.io/crates/airust/0.2.0).

## Known limits

The reference corpus is small and hand-labeled. It is a smoke test, not an independent relevance study or a production guarantee. An embedding model can rank related text but does not establish truth, negation, prerequisites, or contradictions.

Exact semantic retrieval is linear in the number of vectors and their dimension. The semantic index is in memory; loading source JSON requires recomputing vectors. Providers own deadlines, cancellation, credentials, and external side effects. Batch atomicity covers returned errors within this process, not crashes or distributed transactions.

The optional local path loads native code and a separately obtained model. Use trusted assets and observe their licenses. Model and tokenizer hashes protect space identity but do not authenticate an arbitrary download. FastEmbed is Apache-2.0-licensed, E5 is MIT-licensed, and ONNX Runtime is MIT-licensed. ONNX bindings are pinned to the selected prerelease API version; future binding changes require separate review.

Primary references: [FastEmbed](https://github.com/Anush008/fastembed-rs), [the E5 model card](https://huggingface.co/intfloat/multilingual-e5-small), [ONNX Runtime](https://github.com/microsoft/onnxruntime), and [Cargo compatibility rules](https://doc.rust-lang.org/cargo/reference/semver.html).
