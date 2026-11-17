# Migrating to AIRust 0.2.5

`cache` is an additive optional feature within the 0.2 Cargo compatibility range. Default features remain empty; existing entries, source JSON, retrieval, graph, context and provenance APIs are preserved. The optional cache uses Postcard and SHA-256 and supports Rust 1.89. No native model dependency is enabled.

```toml
airust = { version = "=0.2.5", default-features = false, features = ["cache"] }
```

Use `IndexCache::save` and `load` with a dataset `CacheIdentity` and an application-controlled path. Supply consistent original records on load; the vector is consumed only after validation succeeds. A miss or incompatibility leaves those records available for rebuilding. Add `embeddings` separately for `save_semantic` and `load_semantic`, supplying the exact expected model recipe/dimension. No inference happens during restoration.

Cache format version 1 is separate from the crate version. Analysis signatures include Unicode table versions, so a toolchain change can require a rebuild. Changed text or metadata, source revisions or model recipes invalidate reuse. Complete snapshots replace old snapshots atomically; no append log, automatic SQLite monitoring or multi-process transaction guarantee is added. Cache replacement uses Unix mode 0600 even if an older cache had broader permissions; existing source-JSON persistence keeps its prior behavior. Cache and graph/provenance lifetimes remain caller-owned. See [CACHE.md](CACHE.md).
