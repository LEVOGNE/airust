# Migrating to AIRust 0.2.1

This release is additive within Cargo's 0.2 compatibility range. Existing public APIs, source formats, embeddings, model fingerprints, and default features are preserved.

```toml
airust = { version = "=0.2.1", default-features = false, features = ["hybrid"] }
```

`hybrid` also enables `embeddings` and adds no dependencies. It supports Rust 1.89. A local model remains separately enabled through `local-embeddings`, with Rust 1.92 and explicit model/runtime files.

Call `SemanticIndex::hybrid_search()` with `HybridSearchOptions`. The outcome exposes borrowed source results, lexical and semantic evidence, and any provider fallback reason. Existing `search()` remains exact semantic retrieval and `lexical().search()` remains BM25.

The default failure policy returns an error. Opt into `SemanticFailurePolicy::LexicalOnly` when a provider failure should fall back to lexical ranking. This policy does not suppress invalid vectors, output-count errors, or model mismatches.

RRF uses rank positions and normalized relative weights. Fused scores cannot be compared directly with BM25, TF-IDF, character similarity, or cosine. Candidate limits and metadata scope influence component ranks; choose settings using labeled data for the intended workload. The semantic minimum-score setting affects semantic contributions only, so lexical-only evidence may still return an entry below that cosine threshold.

No graph, context assembly, persistent index, ANN, or application integration is added. See the [hybrid guide](HYBRID.md) and [release report](CHANGES-0.2.1.md).
