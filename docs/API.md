# Core API guide

This guide records the public behavior of AIRust's knowledge core. Use `cargo doc --locked --no-deps --open` for generated type signatures, and `cargo run --locked --example core` for a complete working example.

For the complete current feature map and integration recipes, see the [0.3.9 API cookbook](API-COOKBOOK.md). Existing core search methods retain their behavior; optional reranking is a separate explicit second step.

## Stability and upgrades

See the [0.3.0 migration guide](MIGRATION-0.3.0.md) for dependency changes, existing artifact compatibility and stricter model imports. The [stability contract](STABILITY.md) describes supported builds, result semantics and validation limits.

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

## Optional graph API

The independent `graph` feature exposes `airust::graph::{KnowledgeGraph, Relation, RelationType, RelationDirection, TraversalDirection, TraversalOptions, Neighbor, GraphVisit, GraphError}`. It stores stable IDs, typed explicit relationships, and relation metadata without storing knowledge text. The [graph guide](GRAPH.md) defines update validation, neighbors, deterministic bounded BFS/DFS, allowlists, mixed-cycle semantics, and memory limits.

## Optional structured context

Enable `context-assembly` and use `ContextAssembler::new(&Engine)` or `from_semantic(&SemanticIndex)`, optionally with `with_graph`. `query` selects a configured existing ranker; `assemble` accepts already ranked IDs. The borrowed `ContextBundle` preserves source references, metadata, rank/path evidence and UTF-8 excerpt offsets. Configure node/text/per-item budgets, deduplication, scope and graph traversal with `ContextOptions`. `Context::intersection` returns `None` when filters conflict; keep this fail-closed result. The complete contract and runnable example are in [CONTEXT.md](CONTEXT.md).

## Optional source provenance

Enable `provenance` for `SourceReference`, `TextSpan`, `CodePosition`, `CodeSpan` and `ProvenanceStore`. Register and bind shared references, preserve multiple sources on merges, reject conflicting definitions, and export/import versioned JSON. Locations have explicit UTF-8 byte units and optional exact-text validation. Combine with `context-assembly` and attach `with_provenance` to borrow source evidence in selected context; missing and omitted sources are explicit. The complete API and caller freshness responsibilities are in [PROVENANCE.md](PROVENANCE.md).

## Optional derived-index cache

Enable `cache` for `IndexCache`, `CacheIdentity`, `CacheLimits`, `CacheMetadata` and `CacheLoad<T>`. `save`/`load` handle the lexical engine; with `embeddings`, `save_semantic`/`load_semantic` preserve validated vectors. Load takes `&mut Vec<KnowledgeEntry>` and consumes it only on success. The [cache guide](CACHE.md) defines fingerprints, recipes, format, limits and rebuild behavior.

## Optional HNSW candidate retrieval

Enable `hnsw` for `HnswIndex`, `HnswConfig`, `AnnSearchOptions`, `AnnOutcome`, `AnnSearchMode` and `AnnError`. Move a `SemanticIndex` into the wrapper, query through an existing provider or validated embedding, inspect the reported mode, and compact changes explicitly. Existing exact APIs remain available through `semantic()`. See [ANN.md](ANN.md).

## Optional revision ownership

Enable `versioning` for `VersionedIndex`, `Revision`, `ExpectedRevision`, `ChangeMetadata`, `EntryVersion`, `RevisionRecord`, borrowed `RevisionView` and `RevisionComparison`. Writes require absent/exact expectations. History is explicitly enabled by record count; restore creates a new revision. The [revision guide](VERSIONING.md) defines tombstones, comparison, pruning, JSON and caller coordination.

## Optional retrieval-quality API

Enable `quality` for `Judgments`, `UnjudgedPolicy`, `evaluate_ranking`, `QueryMetrics`, `summarize` and `QualitySummary`. Metrics use a declared cutoff, graded relevance and explicit unknown-result policy, with undefined recall/nDCG denominators retained. See [QUALITY.md](QUALITY.md). The [complete cookbook](API-COOKBOOK.md) maps all optional APIs and ownership choices; [BENCHMARKS.md](BENCHMARKS.md) gives repeatable validation commands.

## Optional intelligent reranking

Enable `reranking` for `rerank`, `Reranker`, `Rerankable`, `RerankOptions`, `RerankResult`, `RerankOutcome`, `RerankError` and `RerankFailurePolicy`. Apply scope before bounded query/document scoring. Keep original candidate evidence and raw new scores separate. Wrong counts, invalid scores and recipe changes fail. Provider-error fallback is explicit and cannot bypass a configured score threshold. See [RERANKING.md](RERANKING.md).

Enable `local-reranking` separately for `FastEmbedReranker::multilingual_minilm_v2`, with explicit pinned local assets, a trusted dynamic runtime and a positive thread count. It uses 512-token pairs and raw logits, not relevance probabilities. The [local guide](LOCAL-RERANKING.md) documents downloads, resource cost and shared runtime behavior. Neither feature changes existing search results automatically.

## Optional advanced text search

Enable `advanced-search` for `TextIndex`, `TextAnalyzer`, `AnalyzerConfig`, `WordTokenizer`, `Stemming`, `TextSearchOptions`, `TermOperator`, `Token`, `TextSpan`, `PhraseEvidence`, `TextSearchResult` and `TextSearchError`. The [text-search guide](TEXT-SEARCH.md) defines analysis recipes, required quoted phrases, original UTF-8 evidence, word-form limits and update atomicity. Legacy Engine methods remain unchanged.

Storage adapters are optional and own original records, not automatic search-index synchronization. See [the storage contract](STORAGE.md).

The optional compact Retrieval API reuses existing ranking, scopes results and bounds complete JSON. See [the retrieval contract](RETRIEVAL.md).

Relevant excerpts are explicitly enabled through RetrievalOptions, preserve original UTF-8 positions and impose separate analysis/highlight limits. See [the excerpt contract](EXCERPTS.md).

The optional snapshot coordinator publishes search, vectors and source bindings together. Acquire one immutable view per request; its revision is a consistency token, not authorization or an external database transaction. See [the snapshot contract](SNAPSHOTS.md).

Optional retrieval evidence assessment distinguishes policy success, insufficient evidence and uncertainty. It evaluates delivered text and explicit score gates, not source truth or calibrated answer probability. See [assessment contract](ASSESSMENT.md).

The optional execution feature adds shared cooperative cancellation. Supported controlled APIs return typed interruption errors and preserve the dataset publication boundary. Blocking providers/lock waits are not forcibly interrupted. See [execution contract](EXECUTION.md).

Execution controls also support monotonic cooperative deadlines, saturating remaining budgets and parent limits that cannot be extended by a derived control. A late provider response is discarded; blocked work is not forcibly stopped. See [time-budget contract](EXECUTION.md).

Optional execution observers expose actual work phases and counts. Notifications contain no query text, use best-effort delivery, isolate callback panics and run outside publication locks. Results/revisions remain authoritative. See [progress contract](EXECUTION.md).

Reproducible opt-in synthetic scale/concurrency workloads have explicit count guards and correctness assertions. They measure lexical RAM/source snapshots, not universal capacity or production security. See [stress methodology](STRESS.md).

The optional shared-index feature shares immutable record/posting/duplicate payloads across clones, detaches changed buffers and preserves old readers. Default storage and wire formats remain unchanged. See [sharing contract](SHARED-INDEX.md).

Incremental updates retain unchanged term/positional analysis, update metadata in each projection and refresh coordinates whenever original text bytes change. Diagnostic preparation receipts distinguish rebuilt/reused analysis. See [incremental contract](INCREMENTAL.md).

Optional adaptive routing selects only explicitly registered sources, freezes analyzer selection during metadata-only preflight and preserves actual recipes/scopes. Source failures never trigger hidden retries. Route traces are separate diagnostics. See [routing contract](ADAPTIVE.md).

## Optional explicit feedback learning (0.2.23)

a bounded local feedback ledger and inspectable smoothed ranking adjustments. Enable `feedback` explicitly. See [API and limits](FEEDBACK.md).

## Optional inspectable trainable ranking (0.2.24)

an optional small linear model with replaceable bounded features, explicit training, grouped holdout evaluation and versioned artifacts. Enable `learned-ranking` explicitly. See [API and limits](LEARNED_RANKING.md).

## Optional evidence-backed rule proposals (0.2.25)

an optional bounded rule interpreter that proposes conclusions with explicit premises, sources, revision, conflicts and incomplete-result status. Enable `reasoning` explicitly. See [API and limits](REASONING.md).

## Comparable quality gates (0.2.26)

comparisons of identified retrieval runs, visible per-query regressions and explicit quality/resource criteria. Enable `quality-gates` explicitly. See [contract](QUALITY-GATES.md) and [release report](CHANGES-0.2.26.md).

## Evidence freshness (0.2.27)

verification of captured proof dependencies against current facts, rules, source versions and allowed scope. Enable `evidence-freshness` explicitly. See [contract](EVIDENCE-FRESHNESS.md) and [release report](CHANGES-0.2.27.md).

## Exact ranking explanations (0.2.28)

per-document score explanations using the actual learned features and feedback adjustments. Enable `rank-explanations` explicitly. See [contract](RANK-EXPLANATIONS.md) and [release report](CHANGES-0.2.28.md).

## Controlled training pipeline (0.2.29)

trusted feedback curation, versioned datasets, grouped training and explicit quality-gated model promotion with rollback. Enable `training-pipeline` explicitly. See [contract](TRAINING-PIPELINE.md) and [release report](CHANGES-0.2.29.md).

## Optional disk search index (0.2.30)

a dedicated disk-backed full-text index with atomic revisions, scoped retrieval, bounded original-record export and integrity checks. Enable `disk-index` explicitly. See [contract](DISK-INDEX.md) and [release report](CHANGES-0.2.30.md).

## PDF import stability (0.3.1)

The optional `pdf` importer refuses conversion when no usable text was extracted. `PdfLoader::save_knowledge_base_if_absent` publishes complete output only if no destination exists, including concurrent writers. The CLI uses it unless `--overwrite` is explicit. Parser unwinding panics become `PdfError::PdfError` at the adapter boundary; aborts, OOM and hard per-page deadlines are not covered. See [PDF API, tests and retained-knowledge guide](PDF-STABILITY.md).


## PDF import budgets (0.3.2)

With `pdf`, `PdfLoader::new().with_limits(PdfImportLimits::default())` selects bounded input, physical-page, extracted-text and chunk policies. `PdfImportLimits::new(file_bytes, pages, text_bytes, chunks)?` validates a custom policy, and getters expose it. Limits are opt-in and preserve successful imports in the measured corpus. A violation returns an error without a partial result. See [the complete API and CLI contract](PDF-LIMITS.md) for overlap counting, defaults, terminal handling and native execution limits.


## Supervised PDF conversion (0.3.3)

The optional `pdf2kb` CLI accepts `--timeout-seconds N` for a supervised child job and `--max-result-mib N` for bounded result acceptance. It enables bounded importer defaults, validates complete output in the parent and preserves old destinations on failure. Library signatures remain unchanged. This does not turn synchronous `PdfLoader` calls into interruptible `WorkControl` operations. See [commands, lifecycle and limits](PDF-WORKERS.md).


## PDF object-loading optimization (0.3.4)

The optional importer selects the maintained backend without changing public loader signatures, error variants, CLI flags or knowledge formats. Source bytes, complete structure ordering and original fallback are preserved. The malformed UTF-8 object-keyword check no longer unwinds; unexpected parser panics still have the existing guard. See [performance evidence and maintained-backend contract](PDF-PARSER.md).


## Pagewise PDF budget checks (0.3.5)

The existing optional `PdfLoader` and `PdfImportLimits` API is unchanged. Conversion checks chunk count and overlap-expanded chunk bytes after each page, before extracting later pages. It stages one copy of validated chunks, releases the parser and then creates training records. `extract_pages_from_pdf` still returns complete page contents and applies file/page/raw/cleaned-text limits; it does not split chunks. If several budgets could fail, the first encountered failure may differ from older processing order. See [examples, measured behavior and limits](PDF-IMPORT-PIPELINE.md). The separately versioned backend stays at 0.3.4.


## Corrected PDF cleaning (0.3.6)

The existing `PdfLoaderConfig` fields and `PdfLoader` methods keep their signatures. Digit boundaries and blank paragraphs are protected during line joining; chained wraps are handled in one pass; Unicode is normalized after transformations. Page-number filtering is restricted to first/last nonempty extracted lines. Set `remove_page_numbers = false` for literal numeric edge data and `dehyphenate = false` for meaningful hyphens. `clean_text = false` bypasses AIRust cleaning in PDF import. Corrected imports can change chunks, offsets and ranking: preserve old exports, reimport explicitly and rebuild derived data under a new source/configuration revision. See [full behavior, examples, measurements and limits](PDF-TEXT-FIDELITY.md). The parser backend remains exactly 0.3.4.


## Bounded PDF preview (0.3.7)

With the optional `pdf` feature, `PdfLoader::preview_pages_from_pdf(path, maximum)` returns up to `maximum` usable pages with original physical page numbers and configured cleaning. It stops extracting after that prefix; zero is invalid. Whole-file and total-page limits still apply; visited raw/cleaned text counts toward text limits. It does not split chunks, validate later pages or impose a native-call time/RAM bound. The verbose CLI now uses it for three raw pages. See [the complete API, commands, upgrade contract and limits](PDF-PREVIEW.md).


## Faithful PDF chunks (0.3.8)

The existing optional PDF API now preserves source characters when selecting sentence and paragraph cut positions. Every chunk is a contiguous excerpt of the extracted/cleaned page; outer chunk whitespace is still trimmed. No new method, flag or JSON field is added. Long-page chunks, IDs and rankings can change, so retain old exports and explicitly rebuild derived indexes under a new source/configuration revision. See [API, commands, tests and limits](PDF-CHUNKS.md).


## PDF overlap processing (0.3.9)

Existing `PdfLoader` configuration and methods are unchanged. Overlapping windows now track trimmed edges, skip empty windows without temporary strings and check chunk/UTF-8 quotas before retaining output. Source-excerpt fidelity, boundary choices, budget semantics and JSON fields remain compatible. Existing 0.3.8 exports need no reimport for this optimization. High overlap still repeats nonempty content. See [API, commands, measurements and limits](PDF-OVERLAP.md).
