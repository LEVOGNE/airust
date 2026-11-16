# Optional embeddings

Enable the `embeddings` Cargo feature for provider-neutral vector handling and exact semantic retrieval. This feature adds no dependencies or model downloads. The default lexical core remains available without it.

```toml
[dependencies]
airust = { version = "=0.2.0", default-features = false, features = ["embeddings"] }
```

## A minimal vector example

```rust
use airust::{
    semantic::{Embedding, EmbeddingSpace, SemanticIndex, SemanticSearchOptions},
    KnowledgeEntry,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let space = EmbeddingSpace::new("example-vectors:v1", 3)?;
    let mut index = SemanticIndex::new(space.clone());
    index.upsert_embedded(
        KnowledgeEntry::new("note-1", "An application-owned record."),
        Embedding::new(space.clone(), vec![1.0, 0.0, 0.0])?,
    )?;
    let query = Embedding::new(space, vec![0.9, 0.1, 0.0])?;
    let hits = index.search_embedding(&query, &SemanticSearchOptions::new(5))?;
    assert_eq!(hits[0].entry.id, "note-1");
    assert!(hits[0].score > 0.99);
    assert_eq!(index.lexical().get("note-1"), Some(hits[0].entry));
    Ok(())
}
```

Hand-written vectors demonstrate the API. They do not establish semantic understanding. Meaning comes from a suitable embedding model; the release report separately measures a real multilingual model.

## Model identity and vector validation

`EmbeddingSpace::new(model_recipe_id, dimensions)` requires a nonempty identifier and positive dimension. The identifier must cover the actual model revision, tokenizer, pooling, prefixes, truncation, and other settings that affect vector meaning. A provider name alone is insufficient. Getters expose `model()` and `dimensions()`.

`Embedding::new(space, values)` consumes `Vec<f32>`, validates its dimension, rejects NaN, infinity, and zero norms, and L2-normalizes it using a double-precision norm. Extreme finite and subnormal inputs are supported. Floating-point rounding still applies. `values()` and `space()` return immutable references.

`Embedding::cosine(other)` checks complete model-space equality before comparing vectors. Equal dimensions do not make different models compatible. The low-level `cosine_similarity(left, right)` validates raw slices and compares their numbers only; it has no model identity to verify.

Cosine scores range from -1 through 1. They are similarities, not probabilities, truth scores, or proof of an inferred relationship.

## External providers

Implement `EmbeddingProvider` to call an application-owned local model or external service. Its two methods are:

- `space()` returns the current model recipe and dimension.
- `embed(texts, purpose)` receives borrowed strings in order and returns one `Vec<f32>` per input, in the same order.

`EmbeddingPurpose` distinguishes `Query` and `Document`, allowing models with different retrieval prefixes. AIRust does not lowercase, trim, or tokenize model input. The provider must implement the appropriate preprocessing recipe and report it through its space identity.

`embed_batch(provider, texts, purpose)` is the checked boundary. It rejects whitespace-only inputs before calling the provider, handles an empty batch without inference, validates output count and vectors, and rejects model identity changes during the call. It returns validated `Embedding` values.

Provider implementations report failures through `EmbeddingError::Provider`. The synchronous contract deliberately leaves credentials, request limits, deadlines, retries, cancellation, and network access with the caller. AIRust cannot undo a provider's external side effects or guarantee timeout behavior for arbitrary provider code.

## Index ownership and updates

`SemanticIndex` owns one existing `Engine`, one embedding space, and a vector side table. Original entries are stored once. Search results borrow those entries. The index stores no second copy of the full knowledge corpus.

| Method | Purpose |
| --- | --- |
| `new(space)` | Create an empty semantic index. |
| `from_index(engine, provider, batch_size)` | Consume an existing lexical index and embed its entries in bounded inference batches. |
| `lexical()` | Borrow the existing lexical index for BM25, TF-IDF, matching, entries, statistics, or JSON export. |
| `into_lexical()` | Consume the semantic index and recover its lexical index without copying the entries. Useful when rebuilding for another model. |
| `space()` | Inspect the shared embedding recipe and dimension. |
| `embedding(id)` | Borrow the stored normalized vector for an existing entry. |
| `upsert(entry, provider)` | Embed and insert or replace one entry; return its previous source entry. |
| `upsert_embedded(entry, embedding)` | Insert an already validated, application-supplied document embedding. |
| `upsert_batch(entries, provider, batch_size)` | Consume a batch of source entries, stage validated vectors, then apply all updates. Return the processed entry count. |
| `remove(id)` | Remove the source entry, lexical postings, duplicate references, and vector together. |
| `clear()` | Clear records and vectors while retaining the configured model space. |
| `len()` / `is_empty()` | Describe the number of entries. |

`batch_size` must be positive. Repeated IDs within one batch fail before inference; existing IDs may be replaced. Invalid entries, provider failures, malformed output, and model mismatches leave the existing index unchanged. Batch atomicity applies to returned errors in this process. It is not a transaction across external services, a crash-recovery mechanism, or multi-process synchronization.

For atomic batch updates, staging memory includes the incoming records, normalized text needed by the lexical index, and incoming vectors. Provider calls use only the requested batch size. Existing source records are not cloned. `from_index` consumes its input; on failure that owned input is dropped. A caller's durable original data remains the recovery source.

The lexical view is immutable so callers cannot create missing or stale vectors through direct mutation. Precomputed vectors must use the document purpose and the declared recipe. AIRust verifies dimensions and recipe identity, but cannot prove that an arbitrary caller used the claimed model.

## Exact semantic retrieval

`search(query, provider, options)` embeds one query with the query purpose, then performs exact cosine ranking. `search_embedding(query_embedding, options)` accepts a precomputed query. Both enforce the index's space identity.

Build `SemanticSearchOptions` with `new(limit)`, `with_min_score(score)`, and `with_context(context)`. Getters expose `limit()`, `min_score()`, and `context()`. The default limit is ten and the default minimum is -1, allowing the full cosine range. Invalid thresholds fail during construction. Empty or whitespace-only text queries are errors. An empty index or zero limit skips provider inference after checking query and model identity.

Results contain the original `entry` and `score`. Equal scores use ascending entry IDs. Metadata and ID filters use the existing `Context` implementation and are applied before keeping the best results. A query has no token-overlap requirement.

Retrieval scans all eligible vectors: O(N × D) arithmetic for N entries and D dimensions, with O(K) retained results for limit K. This release adds no approximate index, hybrid ranking, graph traversal, or inferred logical relation type.

## Storage and operational limits

The semantic index is in memory. Source JSON can be exported through `lexical().to_json()`, but that export contains no vectors. Reloading source entries requires fresh embeddings. Application storage, authorization, synchronization, and provider configuration remain application responsibilities.

The core and provider-neutral feature support Rust 1.89. The optional `local-embeddings` adapter currently requires Rust 1.92 through ONNX Runtime's Rust bindings. See the local-provider guide for its explicit model and runtime requirements.
