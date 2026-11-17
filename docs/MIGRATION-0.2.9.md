# Migrating to AIRust 0.2.9

Existing 0.2.8 APIs, source-entry fields, search behavior and persistence formats remain compatible. Default features are still empty. Updating the dependency does not automatically enable reranking, models, a web server or an application integration.

```toml
[dependencies]
airust = { version = "=0.2.9", default-features = false }
```

Select `reranking` for the provider-independent API or `local-reranking` for the explicit multilingual cross-encoder. Use `rerank` on a bounded authorized candidate slice after search. Preserve its source slice/read guard until the outcome is consumed, and retain its original evidence. See the [API cookbook](API-COOKBOOK.md), [reranking contract](RERANKING.md) and [native setup](LOCAL-RERANKING.md).

Raw reranker scores have their own scale. Do not add them to BM25/cosine or present logits as probabilities. Equal reranker scores retain input order rather than reretrieving or using ID order. Original candidates outside the selected cap are not returned. A scorer cannot recover a missing first-stage candidate. No default threshold rejects no-answer queries.

Invalid inputs and output contracts fail before returning a ranking. Optional provider-error fallback restores the selected original prefix with no scores and an explicit reason, discarding partial batches. A configured score threshold disables unscored fallback; a failed provider then returns an error.

Both local model adapters need Rust 1.92; provider-independent features retain Rust 1.89. Local E5 and reranker convenience constructors now share a process-wide ONNX environment and require the same canonical runtime-library path. This fixes the second-environment failure when composing providers. The environment remains alive until process exit. Applications that already own an external ORT environment can implement the provider traits around it rather than asking the convenience loaders to adopt it.

No existing cache, vector, graph, provenance or version-history data needs conversion. Reranking is derived request work, not a new persistent index. No SQLite or DenkMaps changes are made. Measure application quality and added resource cost before enabling the extra stage; the [release report](CHANGES-0.2.9.md) preserves native regressions as well as improvements.
