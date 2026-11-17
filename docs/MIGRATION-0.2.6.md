# Migrating to AIRust 0.2.6

The optional `hnsw` feature is additive within Cargo's 0.2 compatibility range. Existing exact lexical/semantic/hybrid/context methods, source entries, cache and provenance formats remain unchanged. Default features stay empty. hnsw_rs 0.3.5 is optional, MIT/Apache-2.0, and checked on Rust 1.89. No model or external service is enabled.

```toml
airust = { version = "=0.2.6", default-features = false, features = ["hnsw"] }
```

Move a validated existing semantic index into `HnswIndex::from_semantic(index, HnswConfig::default())`. Select ANN explicitly with `search` or `search_embedding`. Outcomes report the actual ANN/exact mode. Candidate cosine scores retain the existing meaning; approximation changes candidate recall, not that score definition. Scope, small-index, oversized-window and excessive-pending cases use exact fallback. An explicit empty `Context` still means unrestricted under the existing context contract and triggers exact scoped fallback.

Update through the wrapper, not an independently mutated clone. Immutable `semantic()` permits existing exact hybrid/context/cache operations; `into_semantic()` ends ANN ownership. Updates and deletions mark stale base IDs and search live changes exactly. Schedule `rebuild()` to release stale buffers and reset the overlay. `max_pending` is a fallback trigger, not a strict memory ceiling; pending IDs remain until compaction. Temporary rebuild memory contains old/new topology. No automatic rebuild, native model, ANN persistence or application integration is added. Build ANN after restoring an existing semantic cache. See [ANN.md](ANN.md).
