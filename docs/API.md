# Core API guide

This guide records the public behavior of AIRust's knowledge core. Use `cargo doc --locked --no-deps --open` for generated type signatures, and `cargo run --locked --example core` for a complete working example.

## Source entries

`KnowledgeEntry` contains:

| Field | Meaning |
| --- | --- |
| `id: String` | Stable identifier owned by the calling application. |
| `text: String` | Searchable source text. |
| `metadata: Option<serde_json::Value>` | Optional application data retained with the entry. |

`KnowledgeEntry::new(id, text)` constructs an entry. `with_metadata(value)` attaches metadata. Construction alone does not validate an entry; the index validates before accepting it.

Original content remains unchanged. Normalized text and term counts are separate derived records. Nonempty text without searchable words can be stored and compared even when it produces no search tokens.

## Creating and maintaining an index

`Engine` is an alias for `KnowledgeIndex`.

| Method | Contract |
| --- | --- |
| `new()` | Create an empty index. |
| `from_entries(entries)` | Validate all entries into a new index; repeated IDs are errors. |
| `upsert(entry)` | Insert or replace one entry; return its previous value when present. |
| `remove(id)` | Remove an entry and all associated postings and duplicate references; unknown IDs return `None`. |
| `clear()` | Remove all entries and derived data. |
| `get(id)` | Return an immutable entry reference when present. |
| `entries()` | Iterate immutable entries in ascending ID order. |
| `len()` / `is_empty()` | Describe the number of source entries. |
| `stats()` | Return source-entry, vocabulary-term, and total-token counts. |

Empty or whitespace-only IDs and normalized-empty text are rejected. Validation of a replacement occurs before removing its prior value. IDs are not automatically rewritten or generated.

## Search

`search(query, limit)` returns BM25 results. `search_with_options(query, options)` selects the algorithm and configuration. `search_in_context(query, context, limit)` applies an explicit scope.

Each `SearchResult` contains a borrowed `entry`, `score`, and sorted `matched_terms`. A query with no overlapping searchable terms returns an empty vector. `limit = 0` returns an empty vector. Ties use ascending entry IDs.

`SearchOptions` fields are:

| Field | Default and meaning |
| --- | --- |
| `limit` | At most ten results. |
| `algorithm` | `SearchAlgorithm::Bm25`; `Tfidf` is also available. |
| `bm25` | Validated BM25 settings, `k1 = 1.2`, `b = 0.75`. |
| `context` | No filter by default. |

`Bm25Config::new(k1, b)` rejects nonfinite values, nonpositive `k1`, and `b` outside zero through one. Getter methods expose the accepted values. BM25 uses positive IDF and length-normalized term-frequency saturation. TF-IDF uses smoothed IDF and cosine similarity over indexed vocabulary.

BM25 relevance can exceed one. TF-IDF cosine similarity is bounded to zero through one. Positive finite scores are returned. These values are not confidence probabilities.

## Text comparison

`compare(a, b)` returns `Similarity` with:

| Field | Meaning |
| --- | --- |
| `exact` | The nonempty normalized strings are equal. |
| `character` | One minus edit distance divided by the longer Unicode-scalar length. |
| `token` | Jaccard similarity of unique normalized tokens. |

Empty text has zero character and token similarity and is not an exact match. Comparison is lexical and symmetric.

`find_matches(text, limit)` uses fuzzy comparison with a minimum similarity of 0.8. `r#match(text, limit)` is an equivalent raw-identifier spelling. `match_with_options()` accepts `MatchOptions`:

| Field | Meaning |
| --- | --- |
| `strategy` | `Exact`, `Fuzzy`, or `TokenOverlap`. |
| `min_similarity` | Finite value between zero and one; default 0.8. |
| `limit` | Maximum number of results; default ten. |

The result contains a borrowed entry, the selected score, and complete comparison evidence. Zero-score results are excluded even when the minimum threshold is zero. Ties use ascending IDs. An invalid threshold returns `MatchError`.

`duplicates()` returns groups sharing normalized text. The original entries, including metadata, remain in the index.

## Filters

`Context::new()` accepts every entry. `with_entry(id)` adds an allowed ID. Multiple IDs are alternatives within that set. `with_metadata(key, value)` adds an exact top-level JSON condition. All metadata conditions must hold.

An entry must satisfy both the ID selection and metadata conditions. Missing metadata fields do not match. Numeric JSON values and text containing digits are different values. A filter does not rewrite the query or recompute corpus statistics for a smaller collection.

## Relationships

`relate(id, limit)` returns `RelationSuggestion` values. Each includes `source_id`, `target_id`, `kind`, `score`, and sorted `shared_terms`.

The only inferred kind is `Related`. Its score is Jaccard similarity after applying the small English and German stopword list. No shared eligible terms means no suggestion. A source entry never suggests itself. An unknown source ID returns `IndexError::UnknownId`, including when the requested limit is zero.

The method does not mutate the index or save a relationship. Review suggestions in the calling application before using them as links.

## JSON and errors

`to_json()` serializes source entries in deterministic ID order. `from_json()` parses and validates an entry array, then rebuilds the index. Duplicate IDs fail instead of silently replacing content.

`IndexError` distinguishes empty IDs, empty text, repeated import IDs, unknown relation-source IDs, and malformed JSON. `SearchError` reports invalid BM25 configuration. `MatchError` reports invalid similarity thresholds.

Result entries borrow the index. Finish using those references before modifying the index. Export owned data or clone an entry when a caller needs to keep it through later mutations.

## Existing agents and training collections

The compatibility layer remains available for `input`/`output` training examples. `ResponseFormat` supports text, Markdown, and JSON. Training examples also have weights and optional metadata. Existing plain-string responses can be loaded alongside structured response variants.

`KnowledgeBase` is distinct from the generic index. The application must choose how a stored question-answer pair maps to stable IDs and searchable document text. `from_embedded()` returns an empty collection unless `embedded-knowledge` is enabled. `cli` and `web` enable that sample data automatically.

`TfidfAgent` retains its historical name while using BM25. Its confidence score is a heuristic. `MatchAgent` preserves its older exact whitespace rule. `ContextAgent` appends formatted history. The default generic `Agent` confidence behavior exists for compatibility; callers needing typed no-match handling should prefer the core result API.

## Training persistence

`KnowledgeBase::load()` and `save()` validate that weights are finite and nonnegative. Invalid data produces an error. `save()` rejects symlink and non-file destinations. It validates and serializes before replacing an existing file, writes through an exclusively created private temporary file, synchronizes content, and preserves existing file permissions.

Atomic replacement keeps complete snapshots. It does not merge simultaneous writers or provide transaction isolation; callers requiring coordinated updates must supply that policy. Generic `Engine::to_json()` remains a serialization operation and does not write files.

## Optional semantic API

The `embeddings` feature exposes `airust::semantic`: `EmbeddingProvider`, `EmbeddingPurpose`, `EmbeddingSpace`, validated `Embedding` values, `embed_batch`, `cosine_similarity`, `SemanticIndex`, `SemanticSearchOptions`, and borrowed `SemanticResult` values. It adds no dependencies and preserves the existing lexical API. See [SEMANTIC.md](SEMANTIC.md) for complete contracts and a compiled example.

The `local-embeddings` feature adds `semantic::local::FastEmbedProvider`, which accepts explicit E5 model and ONNX Runtime library paths. It enables no model-hub access or compile-time binary downloads. See [LOCAL-EMBEDDINGS.md](LOCAL-EMBEDDINGS.md) for its Rust 1.92 requirement, model recipe, memory costs, and explicit evaluation command.

## Optional hybrid API

The `hybrid` feature adds `SemanticIndex::hybrid_search()` and `airust::hybrid::{HybridSearchOptions, HybridOutcome, HybridResult, HybridError, SemanticFailurePolicy}`. It reuses both lexical modes, exact semantic retrieval, source entries, scope filters, and result ordering. The outcome explains weighted reciprocal ranks and explicit provider-error fallback. See [HYBRID.md](HYBRID.md) for configuration, evidence, candidate budgets, and limits.
