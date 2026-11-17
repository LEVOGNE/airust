# Optional approximate semantic retrieval

Enable `hnsw` for `HnswIndex`, `HnswConfig`, `AnnSearchOptions` and `AnnOutcome`. The exact `SemanticIndex` API remains unchanged. HNSW proposes candidates; AIRust computes their final cosine scores with its existing exact function and orders ties by ID. Approximation can miss relevant candidates. No semantic model is included by this feature.

`HnswIndex::from_semantic(index, config)` takes ownership of an existing semantic index and builds generic topology over shared vector buffers. `semantic()` borrows it immutably and `into_semantic()` returns ownership. Existing graph/hybrid/context/cache operations can use that borrowed semantic index and retain their exact behavior. ANN is explicitly selected only through this module.

`HnswConfig::new(connections, layers, ef_construction)` validates connections 2..64, layers 1..16, and construction width greater than twice the connections and at most 100000. Defaults are 16/16/200. `with_exact_threshold` defaults to 512 live entries; `with_max_pending` defaults to 1000 changed IDs. `AnnSearchOptions::new(limit)` defaults to search width 100. `with_ef_search` accepts 1..1000000. `with_semantic_options` supplies the existing threshold, limit and scope. Larger widths can improve recall at additional CPU/memory cost.

`search` embeds a query through an application provider; `search_embedding` uses a validated existing embedding. Query/model/dimension contracts match exact retrieval. Outcomes expose borrowed `results`, `mode` and `candidates_scored` (final exact cosine evaluations, not internal graph distance calls). Small indexes, any explicit context scope, excessive pending changes and limits exceeding search width use exact retrieval, with the reason visible in `AnnSearchMode`. Empty requests skip inference. Scoped fallback applies permissions before exact scoring and cannot lose authorized results by filtering only an unscoped ANN top-K.

Updates, batches and removals change the authoritative in-memory semantic index. An overlay marks stale base IDs and searches all live changed vectors exactly. Stale base candidates are excluded. `needs_rebuild` reports when pending IDs exceed the configured policy; searches then fall back to exact. `rebuild` creates fresh topology and clears stale vectors/IDs. There is no automatic expensive rebuild on a write. Peak memory during a rebuild includes old and new topology. Applications schedule compaction; repeated replacements retain old vector buffers until compaction. Never expose a mutable underlying index that could bypass this overlay.

Navigation reuses the existing anndists L2 function; final cosine re-scoring remains independent. The backend is pinned hnsw_rs 0.3.5, MIT/Apache-2.0. Its generic point type shares `Arc<Vec<f32>>` values with the semantic index; source text is not duplicated. Build uses sorted IDs and serial insertion with the pinned upstream fixed seed. Deterministic small-corpus and mutation tests are provided; cross-platform floating-point or future backend graph identity is not guaranteed. Cache format 1 stores semantic vectors, not HNSW topology: load a validated semantic cache, then rebuild this optional derived topology. Original application data, access control and durable revisions remain caller-owned.

## Minimal usage

```rust
use airust::{ann::{AnnSearchOptions, HnswConfig, HnswIndex},
    semantic::{Embedding, EmbeddingSpace, SemanticIndex}, KnowledgeEntry};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let space = EmbeddingSpace::new("example-recipe-v1", 2)?;
let mut semantic = SemanticIndex::new(space.clone());
semantic.upsert_embedded(KnowledgeEntry::new("note", "Rust memory safety"),
    Embedding::new(space.clone(), vec![1.0, 0.0])?)?;
let index = HnswIndex::from_semantic(semantic, HnswConfig::default());
let query = Embedding::new(space, vec![1.0, 0.0])?;
let result = index.search_embedding(&query, &AnnSearchOptions::new(1))?;
assert_eq!(result.results()[0].entry.id, "note");
# Ok(())
# }
```

This tiny example deliberately uses exact-small-index fallback. See `examples/ann_benchmark.rs` for explicit ANN/exact recall comparisons and recorded limits in [CHANGES-0.2.6.md](CHANGES-0.2.6.md).

The 100k independent-random-vector release test recovered 0.145 recall@10 at width 100 and 0.685 at width 1000, with roughly nine minutes for serial topology construction. The clustered workload behaved differently. These measured limits are part of the release contract: defaults are not a universal recall or startup guarantee. The full report preserves both favorable and unfavorable cases.
