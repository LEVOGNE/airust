# AIRust 0.2.26 API cookbook

AIRust finds knowledge, compares content and returns evidence. Your application owns its records, permissions and durable storage. Start with the smallest feature set that solves the task. All examples named below ship with the crate and are checked by the release process.

## 1. Choose features

```toml
[dependencies]
airust = { version = "=0.2.26", default-features = false }
```

| Feature | What it adds | Guide and executable example |
| --- | --- | --- |
| None | Shared lexical index, BM25/TF-IDF, matching, duplicates and lexical relation suggestions | [API](API.md), `core` |
| `embeddings` | Provider-neutral vectors and exact semantic retrieval | [SEMANTIC](SEMANTIC.md), `semantic` |
| `local-embeddings` | Explicit local E5/ONNX inference | [LOCAL-EMBEDDINGS](LOCAL-EMBEDDINGS.md), `semantic_quality` |
| `hybrid` | Weighted reciprocal-rank fusion of lexical and semantic candidates | [HYBRID](HYBRID.md), `hybrid` |
| `execution` | Optional cancellation, cooperative deadlines and real work-phase notifications | [EXECUTION](EXECUTION.md), `cancellation`, `time_budget`, `progress` |
| `shared-index` | Optional copy-on-write lexical payload sharing across index clones and snapshots | [SHARED-INDEX](SHARED-INDEX.md), existing `stress_index`/`stress_snapshots` |
| `snapshots` | One revision for immutable search/vector/source views and checked updates | [SNAPSHOTS](SNAPSHOTS.md), `snapshots`, `snapshot_benchmark` |
| `adaptive` | Optional source registry, policy-controlled query routing and bounded traces | [ADAPTIVE](ADAPTIVE.md), `adaptive`, `adaptive_quality`, `adaptive_benchmark` |
| `retrieval` | Compact owned excerpts/evidence and complete JSON budgets over interchangeable sources | [RETRIEVAL](RETRIEVAL.md), `retrieval`, `retrieval_benchmark` |
| `storage` / `sqlite-storage` | Revision-fenced atomic record storage and standalone SQLite | [STORAGE](STORAGE.md), `storage`, `storage_benchmark` |
| `advanced-search` | Explicit phrase queries, Unicode word boundaries and optional German stemming | [TEXT-SEARCH](TEXT-SEARCH.md), `advanced_search` |
| `reranking` | Bounded provider-neutral query/document rescoring | [RERANKING](RERANKING.md), `reranking` |
| `local-reranking` | Explicit verified multilingual cross-encoder/ONNX inference | [LOCAL-RERANKING](LOCAL-RERANKING.md), `reranking_quality` |
| `graph` | Explicit relationships, bounded BFS/DFS and cycle checks | [GRAPH](GRAPH.md), `graph` |
| `context-assembly` | Scoped ranked excerpts and optional graph neighbors | [CONTEXT](CONTEXT.md), `context` |
| `provenance` | Shared source definitions and entry bindings | [PROVENANCE](PROVENANCE.md), `provenance`, `provenance_context` |
| `cache` | Validated atomic derived-index/vector snapshots | [CACHE](CACHE.md), `cache` |
| `hnsw` | Explicit approximate semantic candidates and exact fallbacks | [ANN](ANN.md), `ann` |
| `versioning` | Expected-revision writes, tombstones and optional history | [VERSIONING](VERSIONING.md), `versioning` |
| `quality` | Retrieval metrics with explicit judgments and unknown-result policy | [QUALITY](QUALITY.md), `quality`, `retrieval_quality` |

Rust 1.89 supports the provider-independent features. The local ONNX adapters need Rust 1.92. The default library requires no model, network service, HTTP server or database. Existing CLI/PDF/web integrations remain separately opt-in. `reranking` does not enable embeddings; `local-reranking` and `local-embeddings` can be selected separately or together with the same runtime-library path.

## 2. Understand ownership before combining APIs

| Type | Owns | Useful access |
| --- | --- | --- |
| `Engine` | Current source entries and lexical analysis | Search, matching, updates and portable source JSON |
| `SemanticIndex` | One Engine and model-specific vector buffers | `lexical()` borrows; `into_lexical()` consumes |
| `HnswIndex` | One SemanticIndex and derived topology over shared buffers | `semantic()` borrows; `into_semantic()` consumes |
| `VersionedIndex` | One Engine, revision heads and optional snapshots | `index()` borrows; `into_index()` consumes and ends history bookkeeping |
| `DatasetManager` / `DatasetSnapshot` | Published immutable data generation / retained read view | Prepare off-lock, publish with CAS, read every component from one view |
| `TextIndex` | One privately analyzed lexical index and positions | Explicit analyzer and original-text phrase evidence |
| `MemoryStore` / `SqliteStore` | Authoritative generic records with a revision lineage | Atomic batches, pages and staged snapshots |
| `Retriever` / `RetrievalResponse` | Source/provider borrows / only selected owned output | Full encoded JSON budget; no second source corpus |
| `ContextAssembler` | Borrows an Engine or SemanticIndex and optional sidecars | No second source corpus |
| `RerankOutcome` | Borrows the supplied candidate slice and captured entry references | Retains component scores/ranks; no second corpus or index mutation |
| Graph/provenance | IDs, relationships or source definitions/bindings | Caller-owned sidecars |

VersionedIndex and SemanticIndex are alternative owners of an Engine. They are not two mutable wrappers over the same index. An application with its own database history normally keeps that authoritative history and chooses SemanticIndex when it needs vectors. Do not duplicate the full corpus simply to enable every feature. Consuming a wrapper discards the bookkeeping that is not part of its returned inner index.

Specialized search results borrow original entries; compact RetrievalResponse owns its selected excerpts. Serialize a borrowed bounded result while its read guard remains valid, then release the guard before network work. Copy only the small selected result when it must outlive the index. Update through the owning wrapper so vectors, revisions or ANN overlays cannot be bypassed.

## 3. Index general records

`KnowledgeEntry` still has three fields: `id`, `text`, and optional JSON `metadata`. Use stable IDs. Put access scope, source revision and application-specific labels in metadata; AIRust does not interpret them as permission grants.

`Engine::from_entries` validates and rejects duplicate IDs. `upsert` inserts/replaces; `remove` deletes one record; `clear` resets the lexical engine. `get`, `entries` and `stats` inspect it. Portable `to_json`/`from_json` store source entries and rebuild lexical analysis. They do not write a database or preserve versioning sidecars.

`search(query, limit)` uses BM25. `search_with_options` also accepts TF-IDF, BM25 settings and a Context. Results expose original entries, scores and matched terms. Scores are ranking values, not probabilities. Equal scores use entry IDs for stable ordering.

`find_matches` and `match_with_options` support exact, character-fuzzy and token-overlap matching. `duplicates` groups normalized exact duplicates. `matching::compare` exposes separate exact, character and token similarities. Meaningful synonyms can require embeddings; character similarity is not semantic understanding.

`relate(id, limit)` proposes connections explained by shared terms. It does not infer prerequisites, hierarchy or contradictions, and it never writes a graph edge automatically. Review suggestions in the application.

## 4. Apply access scope before retrieval

`Context` filters entry IDs and metadata. Search and semantic options accept it. `Context::intersection` returns `None` for incompatible filters: that means deny/empty, never replace it with a default Context.

An empty Context ID set means unrestricted under the existing contract. If the authorized set is empty, return no results before querying. Do not construct an empty ID set and assume it denies access. Graph traversal has a different explicit allowlist contract; see its guide.

Context assembly intersects its global scope with the selected ranker's scope. Missing/excluded entries cannot become graph bridges. HNSW uses exact retrieval for scoped requests, avoiding an unscoped top-K post-filter that could miss authorized answers. Matching, global duplicate grouping and lexical relation suggestions need an appropriately scoped index or caller-enforced authorized record set.

## 5. Add a provider and validated vectors

An `EmbeddingSpace` identifies the complete model recipe and dimensions. Include weights/revision, tokenizer, pooling, truncation, prefixes and normalization, not just a marketing model name. Query and document embeddings must share that recipe.

`EmbeddingProvider` accepts borrowed text batches and Query/Document purpose, then returns vectors. `embed_batch` checks count, identity stability, dimensions, finite values, nonzero norms and input validity. `Embedding::new` normalizes a validated vector; `cosine` rejects incompatible spaces. `cosine_similarity` also validates raw vectors.

`SemanticIndex::from_index(engine, provider, batch_size)` consumes the lexical engine. Use `upsert`, error-atomic `upsert_batch`, or `upsert_embedded` with already validated vectors. `remove`/`clear` keep lexical and vector state aligned. `search` embeds a query; `search_embedding` uses an existing embedding. Configure limit, minimum cosine and Context with `SemanticSearchOptions`.

Provider calls are synchronous. Applications own cancellation, deadlines, service authentication and retries. Move blocking inference off an async request thread and use a bounded worker/pool. Do not instantiate a large model for each query. The local adapter's measured process memory is documented in its guide.

`FastEmbedProvider::multilingual_e5_small` needs explicit model-directory, runtime-library and thread-count arguments. It verifies its assets and recipe and does not download weights/runtime binaries during compilation. The handwritten-vector examples demonstrate API behavior only. Actual E5 quality is measured separately.

## 6. Combine lexical and semantic rankings

Enable `hybrid` and call `SemanticIndex::hybrid_search`. `HybridSearchOptions` controls result/candidate limits, lexical algorithm/BM25, normalized weights, RRF constant, cosine floor, scope and provider-failure policy. Fusion uses rank positions, not a sum of incompatible raw scores.

Results retain lexical/semantic ranks, component scores and matched terms. `HybridOutcome` reports semantic use and explicit fallback reason. Default provider failures return an error; `LexicalOnly` is opt-in for provider failures, not permission or invalid-vector bypass. Zero-weight components are skipped. Hybrid can be worse than semantic or lexical search on a particular query; evaluate the chosen weights.

## 7. Supply explicit relationships

Create `KnowledgeGraph`, call `add_node` for each known ID, then `upsert_relation(Relation::new(...))`. Choose ParentChild, DependsOn, References, Related or a custom type, directed/undirected semantics and optional metadata. Retargeting an existing relation ID validates before mutation.

Inspect `neighbors`, sorted node/relation accessors and counts. Remove relations or nodes explicitly; node removal removes incident relationships. `TraversalOptions` controls direction, relation types, depth, node budget, start inclusion and allowed IDs. Use BFS for shortest-hop neighborhoods; DFS is a bounded first-discovery traversal. `breadth_first_matching` applies a caller predicate before expansion. Cycle checks support mixed directed/undirected relationships and iterative long-chain handling.

These relationships come from the application, not an automatic claim of causality or truth. Dangling index/graph IDs are caller reconciliation work; context skips missing records.

## 8. Assemble context and preserve sources

Create `ContextAssembler::new(&engine)` or `from_semantic(&semantic)`. Attach `with_graph` and, when enabled, `with_provenance`. `ContextOptions` selects lexical/semantic/hybrid retrieval, scope, unique source-ID budget, total text bytes, per-excerpt bytes, graph traversal, deduplication and metadata/source-reference output.

`query` retrieves seeds; `assemble` accepts already ranked IDs. Seeds take priority over graph neighbors. The bundle retains source IDs, ranking/path origins and UTF-8 excerpt offsets. Normalized duplicates can share text while retaining separate entry/source references. Excerpt offsets refer to the canonical knowledge text, not automatically to a PDF or original source file.

The byte budget covers returned text only. Metadata, paths, citations, serialized JSON and actual LLM tokenization are outside it. Measure the full prompt with the target model's tokenizer. This release claims no measured token-saving percentage or answer-quality guarantee.

Register `SourceReference` definitions once in `ProvenanceStore`, then `bind` multiple entry IDs. `add` combines registration/binding. Checked replacements/merges reject conflicting definitions; removal/pruning and versioned JSON are available. Locations include optional document/page, zero-based UTF-8 text spans, one-based line/byte-column code spans, revision, URI and metadata. Validate positions against exact source text when available.

Sources are borrowed in context. Missing and intentionally Omitted references are separate. A location does not prove truth/freshness, and the engine opens no path or URI. Update/invalidate bindings when content changes. Never invent page numbers or convert excerpt offsets into source offsets without a verified mapping.

## 9. Restart from a derived cache

`IndexCache::new(path, CacheIdentity::new(dataset)?)` selects a trusted private location. Optionally add the committed source revision. `save`/`load` handle lexical analysis; `save_semantic`/`load_semantic` also preserve vectors and require the expected space.

Load receives `&mut Vec<KnowledgeEntry>` from a consistent authoritative snapshot. A successful load consumes those records. Any returned error leaves them available for rebuilding without a corpus clone. Source fingerprints include text and metadata. Dataset, format, Unicode-analysis and model recipes must match.

Files use private temporary writes, sync and atomic replacement. Limits cap encoded file bytes, entries, dimensions and term bindings, not exact heap use. Checksums detect accidental damage, not a writer able to rehash a private cache. Log a miss/error, rebuild from originals and choose whether an optional cache-write failure should affect application availability. Multi-process source coordination remains application work.

Caches reconstruct posting/duplicate trees; they are not a second source database or an automatic SQLite watcher. Graph/source/history data are separate. Current semantic snapshots do not preserve HNSW topology.

## 10. Choose ANN only after measuring recall

`HnswIndex::from_semantic` takes a semantic index. `HnswConfig` controls construction, small-index threshold and pending-change policy. `AnnSearchOptions` selects semantic options and search width. `search`/`search_embedding` return results plus the actual ANN/exact mode and final candidates scored.

The topology shares vector buffers and reuses a maintained HNSW implementation. Candidates receive existing exact cosine scores. Approximation can miss other relevant vectors. Existing hybrid/context methods through `semantic()` remain exact. Feeding ANN IDs into `assemble` is an explicit application choice; retain the ANN outcome separately because Provided context origins do not invent ANN score evidence.

Update/delete through the wrapper. Live changed vectors are searched exactly and stale base IDs are ignored. Excessive changes switch to exact retrieval until `rebuild` compacts topology. Stale buffers remain until compaction; rebuild temporarily retains old/new topology. A cache restart rebuilds ANN separately.

Read the unfavorable measurements too: at 100k independent random vectors, default-width recall@10 was 0.145; width 1000 reached 0.685, and serial construction took about nine minutes. Small/wide searches can be slower than exact. These synthetic limits are not universal semantic performance. See [ANN](ANN.md).

## 11. Use generic revision history when needed

`VersionedIndex` is a lexical owner for callers needing generic revisions. `ExpectedRevision::Absent` creates a never-seen ID; `Exact(current)` updates, deletes or recreates it. Tombstones retain revision identities. No force-write default bypasses CAS.

`ChangeMetadata` holds optional checked actor/reason and JSON data. These are caller claims. History defaults to zero; `with_history_limit(n)` retains a bounded global journal of committed snapshots. It records future commits, not imported/pruned past states. `version`, `revision`, `history`, `history_for`, `at` and `compare` inspect available state. `restore` creates a new revision, including restoring deletion.

Versioned JSON preserves heads, tombstones and retained snapshots and validates consistency on load. Its input cap defaults to 16 MiB. History counts records rather than bytes; heads/tombstones remain after pruning. Whole-index rollback, independent clones or a rebuilt lineage require application coordination and discarded stale CAS tokens. Restoring knowledge does not automatically refresh vectors, sources or SQLite. An application already providing authoritative revisions need not add a second history owner.

## 12. Evaluate results before deployment

Create `Judgments` with explicit grades. Call `evaluate_ranking` with a positive K and an explicit policy for unjudged IDs. Review Precision@K, Recall@K, reciprocal rank and exponential-gain nDCG@K with returned/judged counts. `summarize` reports macro means and defined denominators. No positive judgments means undefined recall/nDCG, not perfect results.

Use fixed labeled application queries and preserve baseline outputs. Include literal identifiers, German text, paraphrases, no-answer cases, access filters and update/restart behavior. Record CPU, RAM, model load/inference and final context evidence. Descriptive p50/p95/p99 samples are not statistical confidence or production-tail guarantees. See [QUALITY](QUALITY.md) and [BENCHMARKS](BENCHMARKS.md).

## 13. Run the combined demonstration

```bash
cargo run --locked --features hnsw,cache,context-assembly,provenance,quality --example knowledge_pipeline
```

It uses three small generic records and explicitly handwritten vectors, then demonstrates scope, hybrid context, explicit graph/source bindings, quality metrics and semantic-cache restoration. Add `reranking` to that feature list to demonstrate the full retrieve → rerank → sourced-context pipeline with a handwritten API scorer. Both variants are compiled and run by the release checks. The separate `versioning` example demonstrates the alternative lexical history owner. Replace demo vectors/scorer with real providers and records with an authoritative application snapshot before evaluating a real integration.

## 14. Rerank retrieved candidates explicitly

Enable `reranking` alone for the provider-independent API. Implement `Reranker::recipe` and `score(query, documents)` with one finite score per input document, in document order. Higher ranks first; raw scores from different models or BM25/cosine are not directly comparable. `local-reranking` supplies a verified, explicitly loaded multilingual `FastEmbedReranker`. It works alongside E5 through a shared ONNX environment using the same runtime-library path. No model or runtime download happens during a build.

Call `rerank(query, &candidates, &mut scorer, &RerankOptions::new(k))`. Candidate slices can contain entries/references, lexical hits, semantic hits or hybrid hits. A custom wrapper implements `Rerankable`. Each result exposes `entry()`, its original `candidate`, the one-based `original_rank` in the input slice and an optional new `score`. Keep original component evidence through `candidate` and retain the candidate slice/read guard for the outcome's lifetime. Reranking borrows data; it owns no index and changes no source records.

`with_context` filters before selection and inference. Keep the existing empty-ID-set-means-unrestricted contract in mind. `with_candidates` caps the first eligible prefix, `with_batch_size` bounds provider calls and `with_byte_budgets` bounds original UTF-8 inputs. Defaults: ten returned results, one hundred candidates, batches of sixteen, query/document limits of 16/64 KiB and 2 MiB of complete query/document pairs. Total pair bytes repeat the query for each document. The outcome exposes received/eligible/selected/scored counts; omitted candidates are never presented as scored. Native 512-token pair truncation remains a separate model limit.

All selected inputs are validated before any inference. Wrong output counts, non-finite scores, duplicate selected IDs and changed recipes fail. Equal scores keep input order. Default provider failures return errors. Explicit `KeepOriginalOrder` fallback discards every partial score, keeps the selected original order, returns `score=None` and explains the failure. It never bypasses a configured finite `with_min_score` threshold. A successful score is not a calibrated relevance probability or a no-answer decision.

For context, feed the selected IDs into `ContextAssembler::assemble` and preserve rerank scores separately. Assembly still enforces its source/scope/excerpt budgets and does not invent ranking evidence. Reuse model instances in bounded workers, keep database/index locks out of expensive inference and evaluate latency/quality before rollout. See the complete [reranking contract](RERANKING.md), [native setup](LOCAL-RERANKING.md), `reranking` executable and `reranking_quality` evaluator. The [0.2.9 report](CHANGES-0.2.9.md) includes both improvements and regressions on the unchanged fixture. No DenkMaps/MCP integration is performed.

## 15. Search phrases and German word forms

Enable `advanced-search`. `TextIndex::new(AnalyzerConfig)` is an alternative positional lexical owner that reuses the shared Engine's storage/ranking privately. Use `from_entries`, `upsert`, `remove`, `clear`, `get`, `entries`, `stats` and `tokens`; no second source corpus is maintained. The default Engine and its saved/cache formats are unchanged. Analyzer changes require an explicit source rebuild.

`AnalyzerConfig::default()` selects Unicode word boundaries, typographic-apostrophe folding and no stemming. `german()` enables German Snowball stemming. Set tokenizer, stemming, apostrophe handling and positive input/normalized-byte/token limits explicitly. `TextAnalyzer::analyze` returns normalized/stemmed terms with sequential positions and original UTF-8 spans; expansions may share one original grapheme. Store the complete analyzer recipe/config with derived data.

Call `search(query, &TextSearchOptions::new(limit))`. Every double-quoted token phrase is required and matched before top-K; unquoted terms additionally obey Any/All. `with_search` selects the shared BM25/TF-IDF options, `with_context` scopes results, and query/evidence budgets fail explicitly. Invalid/empty/unclosed phrases do not silently become ordinary search. Results retain original entries, analyzed matched terms and bounded phrase evidence with truncation flags. With `reranking`, these results implement `Rerankable` without copying original content.

German stemming handles useful suffix forms but is not full morphology, irregular-verb lemmatization, decompounding or meaning recognition. Analyzed phrases can cross punctuation/whitespace; they are not byte-exact quotations or evidence of truth. The [full contract](TEXT-SEARCH.md), compiled `advanced_search` example and [measured release report](CHANGES-0.2.10.md) define limits and source-coordinate behavior. This feature does not automatically change an existing application or wrap a SemanticIndex.


## 16. Store authoritative records behind one contract

Enable `storage` for `MemoryStore` and the object-safe `StorageBackend`; add `sqlite-storage` for an independently owned SQLite file. Both implement revision-checked get/page and atomic apply. Read the complete `StoreRevision`, then pass it to every operation. `StoreChange::Put/Delete` batches reject repeated IDs, invalid records and exceeded budgets before commit. Conflicts require rereading/reconciling, not force overwrites. Rotate identity after restoring independent database copies.

Use `page` with a pinned revision and exclusive `next_after`, or `read_snapshot` with explicit row/count/byte limits. A concurrent write fails the complete snapshot. `StoredSnapshot::into_index` consumes the source records and returns a `LoadedIndex` with immutable index/revision accessors; `into_parts` supports alternative indexing explicitly. A later source write does not update that view. Stage a fresh view, verify its source revision, then swap it under application coordination.

SQLite uses a standalone versioned AIRust schema, WAL/FULL, bounded busy waits and transactional record/revision commits. It rejects unrelated application databases; this release performs no DenkMaps integration. It is a record backend, not a persisted positional/vector index. See [STORAGE](STORAGE.md), `cargo run --features storage --example storage` and the reproducible `storage_benchmark`.


## 17. Return a compact response through any transport

Enable `retrieval`, construct a source and wrap it in `Retriever::new(source)`. `LexicalSource` borrows Engine; `TextSource` additionally needs `advanced-search`; `SemanticSource` needs `embeddings` and an explicit provider; `HybridSource` needs `hybrid`. Source options retain ranking/threshold/base scope, while `RetrievalOptions` controls request scope, candidates, hits and byte budgets. Base/request scopes intersect before top-K; a disjoint intersection stays empty. Boxed or borrowed custom RetrievalSource implementations use the same pipeline.

Call `query` to receive an owned immutable RetrievalResponse. Use `hits`, original `evidence`, `source_rank`, optional `rerank_score`, `reranking`, `fallback_reason`, `truncated` and `to_json`. The complete schema-version-1 JSON is checked against its output budget, including escaping, envelope and opted-in metadata. Tail hits are removed if needed; an envelope that cannot fit returns an error. Excerpts default to original UTF-8 prefixes with exact spans. The optional relevant-excerpt mode described below locates lexical evidence without synthesizing a summary or counting model tokens. Metadata is omitted by default and reports its status explicitly.

With `reranking`, attach `with_reranker` and explicit RerankOptions. Source validation/scope checks run first. Provider errors remain errors unless explicitly configured for original-order fallback; fallback never fabricates scores. Feed returned IDs into scoped ContextAssembler::assemble with graph/provenance when sourced structural context is needed, retaining original ranking evidence separately. No transport, MCP endpoint or database synchronization is created automatically.

Use the [full retrieval contract](RETRIEVAL.md), compiled `retrieval` example and [proposed DenkMaps guide](DENKMAPS-UPGRADE.md). Keep application permissions and committed source revisions authoritative. The exact JSON byte limit does not replace a model tokenizer, timeout or relevance evaluation.


## 18. Return the matching passage with original positions

Keep `retrieval` enabled and opt in with `RetrievalOptions::with_relevant_excerpts(ExcerptOptions::default())`. This preserves old prefix behavior for existing callers. A window containing a complete phrase hint has priority, followed by distinct matching terms and token occurrences. TextSource forwards its selected tokenizer/stemmer, so German forms retain original-text highlights. Other built-in sources locate legacy lexical overlap; semantic paraphrases without word overlap have an explicit `no_lexical_match` prefix fallback.

`hit.span` indexes the original KnowledgeEntry text in half-open UTF-8 bytes. `hit.span.slice(original)` equals the returned excerpt. `response.excerpt_details()` supplies aligned hit IDs, reasons, bounded matches and highlight-omission flags. For display, convert each `matched.span.relative_to(hit.span)` to excerpt-local bytes, then translate/escape for the target UI. Preserve source revision and original text; these are not automatic PDF/file coordinates.

Relevant mode respects per-item, total-text and full-JSON budgets, including details/highlights. ExcerptOptions additionally bounds analysis input/normalized bytes, tokens, query terms, occurrences and markers. Analysis overruns fail explicitly; marker truncation sets both its local flag and the response flag. Tail-hit removal also removes its details. Use `with_prefix_excerpts` to return to compatible prefix JSON.

Standalone callers can use `select_excerpt` with original text, RankingEvidence and an explicit ExcerptAnalyzer. Custom RetrievalSource implementations can override the default excerpt analyzer; boxed/borrowed forwarding preserves it. Read the [complete excerpt contract](EXCERPTS.md) and run `cargo run --features retrieval --example relevant_excerpts`; adding `advanced-search` also runs its German phrase example. No application integration is performed.


## 19. Keep search, vectors and sources on one dataset revision

Enable `snapshots`. Create DatasetManager with DatasetConfig; use `with_embedding_space` for provider-neutral semantic mode and, with `advanced-search`, `with_text_analyzer` for an optional positional projection. Acquire one `snapshot()` per request and borrow lexical(), semantic(), provenance() and text_index() from that same view. Carry revision() alongside results. Older Arcs remain coherent after publication; require_current checks freshness only at the instant called.

Build DatasetRecords with complete entries and source lists. prepare() stages unique-ID Put/Delete changes against an expected revision, outside the publication lock. prepare_replace() reconciles an entire source image and removes missing IDs automatically. publish() swaps the complete result only if that base is still current. Invalid records, provider failures, changed model recipes, source conflicts, stale work and capacity overruns publish nothing. No automatic retry repeats model work.

Exact unchanged text shares its existing vector buffer for metadata/source changes; changed text needs a matching provider. prepare_model explicitly rebuilds every vector for an enabled/replaced model, or switches to lexical-only with None. Every Put replaces sources; omitting them clears old bindings. Changing a shared source definition requires replacing all affected bindings together. Optional positional search is rebuilt on record updates and published in the same boundary.

Dataset revisions reuse StoreRevision's complete namespace/identity/generation token. An optional separate source_revision can record a consistent external committed generation; later markers must advance within that lineage. This is a caller assertion, not a database watcher or cross-store transaction. Preserve external source/access rechecks. A full image mapped from storage::read_snapshot can be reconciled without changing that storage backend.

Index/source staging copies bookkeeping and can retain multiple generations in RAM; unchanged vectors are shared. Default limits bound encoded records, source lists, vector components and batches, not exact heap. No automatic graph/HNSW/cache/history synchronization is claimed. See [the complete snapshot contract](SNAPSHOTS.md) and run `cargo run --features snapshots,retrieval --example snapshots`. Add `advanced-search,context-assembly` for positional/sourced-context composition. No DenkMaps integration is performed.

## 20. Decide whether delivered evidence meets a policy

Enable `retrieval`; set `RetrievalOptions::new(10).with_evidence_policy(EvidencePolicy::default())`. Inspect `response.assessment()` and match its verdict. `MeetsPolicy` means configured criteria passed; `Insufficient` asks for more evidence; `Undetermined` preserves missing calibration or fallback; `NotRequested` represents a zero result limit. Use qualified returned ranks to inspect supporting excerpts. Pair relevant excerpts with assessment so a useful late passage can be examined.

Coverage compares distinct informative query terms with each delivered excerpt. It never merges support across unrelated records or assumes an uncalibrated semantic score is enough. Explicit recipe-bound score gates permit evaluated thresholds without hiding original scores. Complete JSON bounds include the assessment and reassess after hit removal. See [full assessment API](ASSESSMENT.md), `evidence_assessment` and `assessment_quality` examples. Preserve the source revision and evaluation recipe in your application. No automatic integration or answer generation is performed.

## 21. Cancel supported work cooperatively

Enable `execution` alongside the operation's feature. Create `CancellationToken::new()` and `WorkControl::new(token.clone())`. Pass control through RetrievalOptions, RerankOptions or semantic/hybrid search options; use the explicit controlled Engine/TextIndex/DatasetManager methods. Another thread can call token.cancel(). Handle the typed execution error without presenting an empty answer or retrying a fallback automatically. Dataset publication has a final pre-commit checkpoint; cancellation after that point can still return a successful committed revision. See [execution guide](EXECUTION.md) and `cancellation` example. No transport integration is introduced.

## 22. Set a cooperative time budget

Use `WorkControl::default().with_timeout(Duration::from_secs(2))?` or attach a monotonic Instant with `with_deadline`. Pass it through the same controlled APIs used for cancellation. Inspect `remaining()` when allocating a child's budget; cloned/combined controls retain the earlier parent deadline. Handle ExecutionError::DeadlineExceeded distinctly. Build the budget just before the operation if queueing time should not consume it. Blocking provider calls need their own timeout support; AIRust checks after they return. The `time_budget` example and [execution contract](EXECUTION.md) show this limit explicitly.

## 23. Show progress without inventing an answer ETA

Attach `.with_observer(|event: &ProgressEvent| ...)` to a WorkControl and pass it through supported APIs. Render its phase and completed count. Use fraction only for a known phase total; otherwise show an indeterminate state. Forward updates through a bounded nonblocking application channel to keep UI/network work outside the computation. Treat returned Result as the final status, since busy observers can drop events. Inspect dropped_events() and observer_failures() for diagnostics. Never label ranking percentage as answer correctness or arrival time. The [execution guide](EXECUTION.md) and `progress` example provide the contract. No MCP/DenkMaps component is changed.

## 24. Validate scale before selecting an optimization

Build and run `stress_index` in scale/clone modes and `stress_snapshots` with explicit record counts. Keep category-separated latency, OS RSS, conflicts and correctness assertions. Run small CI profiles and record large runs separately. Compare the same generator, retained readers and mutation pattern before/after changes. There is no new runtime API in 0.2.19. See [stress methodology](STRESS.md) and the versioned release evidence; a synthetic million-record success does not replace real application testing.

## 25. Retain index copies with shared immutable data

Enable `shared-index` explicitly alongside your existing features. Continue using Engine::clone and the DatasetManager API. Edits detach changed private buffers; earlier views keep their original records and scores. Clones still copy outer maps, and changing a common posting set can be costly. Measure one-owner builds/queries as well as retained-copy memory before selecting the flag. The default mode is unchanged. See [sharing contract](SHARED-INDEX.md) and the fixed before/after profiles in the release report.

## 26. Recompute changed text instead of the whole projection

Use existing Engine/TextIndex upserts and DatasetManager prepared updates. Exact unchanged text keeps its terms/positions while metadata remains current. Source-only payloads can reuse the whole positional projection. Inspect PreparationStats.text_entries_rebuilt/text_entries_reused and metadata_only_puts/unchanged_puts. Treat receipts as additive diagnostics. TextIndex::clone allows independent views; shared-index optionally shares token buffers. A normalized-equal but byte-different original still refreshes positions. See [contract](INCREMENTAL.md) and the `incremental` example. Model refresh and external storage synchronization retain their explicit separate policies.

## 27. Route queries through explicitly configured sources

Build AdaptiveSource::default() or new(your_policy), register sources with with_source(name, role, source), and pass it to Retriever. Inspect retriever.source().last_route() after querying. The default policy prefers configured lexical/text sources for short queries and configured hybrid/semantic sources for longer ones; phrases require a declared text backend. It never loads models or retries another backend on errors. For consistent data, register views from the same DatasetSnapshot. The metadata-only prepare_query hook freezes route/analyzer selection, including boxed/borrowed sources. Keep traces and any outer transport envelope within application budgets. See [contract](ADAPTIVE.md); evaluate routing on your own labeled queries before treating it as an improvement.

## Optional explicit feedback learning (0.2.23)

a bounded local feedback ledger and inspectable smoothed ranking adjustments. Enable `feedback` explicitly. See [API and limits](FEEDBACK.md).

## Optional inspectable trainable ranking (0.2.24)

an optional small linear model with replaceable bounded features, explicit training, grouped holdout evaluation and versioned artifacts. Enable `learned-ranking` explicitly. See [API and limits](LEARNED_RANKING.md).

## Optional evidence-backed rule proposals (0.2.25)

an optional bounded rule interpreter that proposes conclusions with explicit premises, sources, revision, conflicts and incomplete-result status. Enable `reasoning` explicitly. See [API and limits](REASONING.md).

The [complete application guide](APPLICATION-INTEGRATION.md) connects these APIs into a future adapter with explicit revision, permission, feedback and review boundaries.

## Comparable quality gates (0.2.26)

comparisons of identified retrieval runs, visible per-query regressions and explicit quality/resource criteria. Enable `quality-gates` explicitly. See [contract](QUALITY-GATES.md) and [release report](CHANGES-0.2.26.md).
