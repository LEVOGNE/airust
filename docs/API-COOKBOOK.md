# AIRust 0.2.9 API cookbook

AIRust finds knowledge, compares content and returns evidence. Your application owns its records, permissions and durable storage. Start with the smallest feature set that solves the task. All examples named below ship with the crate and are checked by the release process.

## 1. Choose features

```toml
[dependencies]
airust = { version = "=0.2.9", default-features = false }
```

| Feature | What it adds | Guide and executable example |
| --- | --- | --- |
| None | Shared lexical index, BM25/TF-IDF, matching, duplicates and lexical relation suggestions | [API](API.md), `core` |
| `embeddings` | Provider-neutral vectors and exact semantic retrieval | [SEMANTIC](SEMANTIC.md), `semantic` |
| `local-embeddings` | Explicit local E5/ONNX inference | [LOCAL-EMBEDDINGS](LOCAL-EMBEDDINGS.md), `semantic_quality` |
| `hybrid` | Weighted reciprocal-rank fusion of lexical and semantic candidates | [HYBRID](HYBRID.md), `hybrid` |
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
| `ContextAssembler` | Borrows an Engine or SemanticIndex and optional sidecars | No second source corpus |
| `RerankOutcome` | Borrows the supplied candidate slice and captured entry references | Retains component scores/ranks; no second corpus or index mutation |
| Graph/provenance | IDs, relationships or source definitions/bindings | Caller-owned sidecars |

VersionedIndex and SemanticIndex are alternative owners of an Engine. They are not two mutable wrappers over the same index. An application with its own database history normally keeps that authoritative history and chooses SemanticIndex when it needs vectors. Do not duplicate the full corpus simply to enable every feature. Consuming a wrapper discards the bookkeeping that is not part of its returned inner index.

Results borrow original entries. Serialize the bounded result while its read guard remains valid, then release the guard before network work. Copy only the small selected result when it must outlive the index. Update through the owning wrapper so vectors, revisions or ANN overlays cannot be bypassed.

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
