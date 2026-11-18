# Proposed DenkMaps upgrade to AIRust 0.2.18

This is an implementation guide, not a claim that DenkMaps uses the new release. The AIRust sequence keeps its core independent. The existing DenkMaps application was previously inspected read-only and was not re-audited for this library-only release; the new library is not integrated automatically.

## What already exists

The inspected DenkMaps checkout depends on AIRust `=0.1.9`, with default features disabled. `src/knowledge_index.rs` already uses Engine/KnowledgeEntry. It reconciles committed SQLite revisions through `wissen_stand` and `wissen_karte`, caches per database identity/tenant, bounds nodes/bytes and falls back to the legacy scan when needed. Its inspected Unicode/non-ASCII query path deliberately preserves the legacy search behavior. Private-map exclusions are applied to results.

SQLite's `karte` data and existing application versions remain authoritative. The adapter maps node titles/notes into stable `map:<map-id>/node:<node-id>` IDs within its tenant index. Do not replace this synchronization with an uncoordinated second database or rewrite identifiers unnecessarily. If changing to a shared multi-tenant index, explicitly namespace tenant identity and test permissions first.

Other local UI/import/MCP work is separate from this library upgrade. Re-read current DenkMaps sources before implementing: concurrent application work can change these inspected paths/contracts.

## 1. Upgrade the dependency without enabling everything

Start with the existing lexical dependency and run all application tests:

```toml
airust = { version = "=0.2.18", default-features = false }
```

This uses the same generic entry shape and existing lexical API. Keep a configuration rollback to the current search. Compare result IDs, counts and authorization before changing ranking behavior. Do not remove the Unicode fallback merely because the new version compiles.

When semantic/context work is explicitly approved and tested, select the needed features, for example:

```toml
airust = { version = "=0.2.18", default-features = false,
           features = ["context-assembly", "provenance", "cache", "quality"] }
```

This enables a provider-neutral semantic interface through context/hybrid features, not a model. Add `local-embeddings` only with explicit assets, its Rust 1.92 requirement and a measured inference budget. Add HNSW only after target-corpus recall and build/update costs justify it.

## 2. Keep one retrieval owner and application history

For lexical retrieval, keep Engine. For vectors, let SemanticIndex own that engine. HnswIndex can later own the semantic index while sharing vector buffers. Graph and provenance stores remain ID-based sidecars; ContextAssembler borrows them.

DenkMaps already has authoritative map/SQLite revisions. Keep using those for application writes and rollback. VersionedIndex is an alternative lexical owner for generic callers, not another mutable layer over the same semantic engine. Do not duplicate every node and vector merely to use every Cargo feature.

## 3. Build from a consistent authorized snapshot

Map each source node to its existing stable ID, searchable title/note text and needed metadata: tenant/map/node identifiers, source revision, visibility and structural references. Keep private fields out of displayed context unless needed and allowed.

Start with per-tenant/authorized indexes or explicit validated scopes. An empty authorized set returns no results before querying; an empty Context ID set means unrestricted. Intersections returning None remain denied. Matching, duplicate and relation APIs need the same authorized record boundary, not only search filtering. Graph neighbors and provenance must not reintroduce excluded maps or reveal private paths.

## 4. Reconcile commits and model work correctly

Preserve the existing SQLite revision projection and consistent read snapshot. Prepare embeddings outside a long SQL/index lock, using an identified source generation. Before publishing prepared vectors, check that the source text, committed revision and permissions still match. Discard/retry stale work.

A safe application flow is:

1. Read an authorized committed snapshot and its generation.
2. Prepare generic records and optional vectors off the async request thread.
3. Recheck the generation/access state before publishing the derived result.
4. For edits, let the existing authorized SQLite CAS transaction commit first.
5. Apply the corresponding validated index/graph/source changes for that generation.
6. If derived synchronization fails, invalidate/rebuild from committed SQLite state.

These are coordinated application steps, not one distributed atomic transaction provided by AIRust. An outbox can be considered if later measurements require durable asynchronous replay; it is not mandatory for the current revision-reconciliation design. Never report an index update as a successful database commit.

Use one bounded model worker/pool, not one multi-gigabyte model per query or tenant. Measure Send/Sync and runtime constraints for the chosen provider rather than assuming them. Move blocking inference/search work away from the async executor when it would block other requests. Retain timeouts/cancellation and an explicit lexical fallback policy.

## 5. Use caches as disposable derived state

Use database identity plus committed source generation in CacheIdentity. Supply the original consistent records and exact expected model recipe on load. Full text/metadata fingerprints, Unicode recipe and model checks must match. A miss/error leaves those records available for a normal rebuild.

Keep cache files in an application-controlled private directory. A checksum detects damage, not a hostile writer. Cache write failure can be logged while serving from a valid rebuilt index if application policy permits. Cache snapshots do not automatically preserve graph/source/history sidecars or ANN topology. Rebuild/reconcile those from authoritative data. Coordinate multiple processes through source generations; last-writer-wins files are not SQLite transactions.

## 6. Add explicit structure and honest evidence

Create graph nodes before relationships. Map actual tree parent/child edges and verified application links to typed relations. Lexical/semantic similarity is a suggestion, not a stored prerequisite or contradiction.

Bind sources to immutable material/snapshot/file revisions when available. Use stable source IDs that identify that artifact/location. Retain exact hashes/byte coordinates and verify actual source text before citation claims. Knowledge-excerpt offsets are not automatically PDF/file offsets. Missing sources remain missing. Update/invalidate source bindings when a node's text or supporting artifact changes.

ContextAssembler selects ranked seeds and bounded permitted neighbors. Configure unique-ID, text-byte and excerpt budgets, and include sources only when authorized. Serialize bounded results while read guards remain valid; release them before external network/model work. Metadata/citations/JSON overhead are outside the text budget.

## 7. Run a shadow comparison with actual maps

Freeze a query set before tuning. Include German spelling/umlauts, literal paths/IDs, paraphrases, common/rare words, no-match queries and private-map cases. Have authorized people judge relevant nodes. Keep the existing search output alongside candidate output without changing the displayed result until accepted.

Measure per query: relevant IDs/ranks, Precision@K, Recall@K, MRR/nDCG, unjudged counts, search/inference/end-to-end latency and selected source evidence. Include model load/RAM, cache startup, changes/deletes, rollback, concurrent revision conflicts and restart. Test at realistic map sizes and across the application's configured index budget/fallback. A public synthetic benchmark does not replace these tests.

Suggested acceptance checks:

- No foreign/private node, neighbor, metadata or source path appears.
- Committed edits/deletes/restores are visible; rolled-back changes are not.
- Stale embedding batches cannot overwrite a newer committed generation.
- Cache corruption/model changes recover from SQLite without data loss.
- Relevant evidence survives context limits; no invented source locations appear.
- Latency/RAM and retrieval quality meet targets chosen before rollout.
- Legacy fallback and dependency rollback remain tested.

Do not enable HNSW based on speed alone. The published difficult random-vector test has substantial recall loss; small/wide ANN queries can be slower, and serial construction can take minutes. For typical small maps, exact retrieval is often the first implementation to evaluate.

## Optional reranking after shadow validation

This release does not enable reranking in DenkMaps. If a later application change is authorized, first retrieve a bounded authorized lexical/semantic/hybrid pool, then explicitly call `reranking::rerank` before context assembly. Keep SQLite revisions and original retrieval evidence. Add `reranking` for a caller scorer or `local-reranking` for the verified local cross-encoder; local E5 and reranker convenience adapters must use the same runtime-library path.

Measure the full extra CPU/RAM/latency and query-level relevance changes on real authorized application judgments. The public reference contains E5 regressions despite a better average. Reranking cannot recover a candidate absent from the first stage, and its raw logits do not establish a no-answer threshold or logical truth. Keep the existing retrieval path as an explicit rollback; do not silently enable reranking with a dependency update.

## 8. Expose bounded retrieval through existing MCP separately

The existing `wissen_suchen` transport does not automatically gain semantic/context behavior after a dependency update. Design any new response fields and tools only after the shadow tests. Return selected IDs, excerpts, source revisions, ranks/evidence and an explicit truncation/budget summary. Preserve the application's permission checks.

MCP/TOON are transport choices, not required AIRust dependencies. Measure the complete final prompt with the target LLM tokenizer before claiming token savings. Fewer selected nodes alone do not establish token counts or answer quality.

The separately requested MCP progress improvement should report real visible actions, elapsed time and completed units when known. It must distinguish a completed server tool request from the external agent's still-unobserved final answer. Start/progress/end communication rules belong to the agent/server instructions; no fabricated percentages, finish time or hidden thinking trace are needed.

The [API cookbook](API-COOKBOOK.md) gives compiled generic examples and exact module contracts. Implement the application upgrade in its own branch/change set after the above checks; this guide performs none of those production changes itself.

## Optional advanced-text trial

This release does not integrate the new TextIndex into DenkMaps. It is an alternative positional lexical owner with an immutable analyzer, not a drop-in replacement for SemanticIndex ownership. A later explicitly authorized trial must rebuild from committed authorized source records, preserve stable IDs/revisions, compare literal identifiers and phrase/German stem behavior against existing search, and measure positional memory/query costs before changing displayed results.


## 9. Trial the three new APIs in a separate application change

**Advanced text (0.2.10):** enable `advanced-search`, construct `TextIndex::from_entries(AnalyzerConfig::german(), records)` from the same committed authorized records, and use TextSearchOptions. Keep this as a measured alternative lexical projection. German stemming is suffix-based; phrases match adjacent analyzed tokens rather than exact bytes. Rebuild on analyzer/recipe changes and compare German/literal queries before selecting it. It is not an automatic semantic-index conversion.

**Storage (0.2.11):** DenkMaps already owns SQLite transactions/revisions. Keep that owner. Do not point SqliteStore at the application database: it owns a separate AIRust schema and rejects unrelated schemas. A future adapter may implement StorageBackend over an appropriately coordinated source, but duplicating every application node into another authoritative database adds reconciliation work. For a standalone Rust tool without an existing store, use MemoryStore or a separate SqliteStore directly. Neither makes the derived search index persistent or automatically current.

**Compact retrieval (0.2.12):** enable `retrieval` and borrow the chosen index via LexicalSource, TextSource, SemanticSource or HybridSource. Construct `Retriever`, optionally attach an explicit reranker, then call `query` with scoped RetrievalOptions. Existing application-owned semantic providers remain explicitly supplied. Metadata stays omitted unless authorized and opted in. Response hit IDs, excerpts, source ranks, component evidence, rerank diagnostics and truncation are owned and can be serialized after releasing the index guard.

A future dependency selection for an isolated lexical/phrase trial could be:

```toml
airust = { version = "=0.2.18", default-features = false,
           features = ["retrieval", "advanced-search", "quality"] }
```

Application-side sequence: authorize the request → obtain/reconcile the committed source revision → borrow the matching derived index → intersect the permitted scope → retrieve a bounded owned response → release guards → recheck freshness/access if required → return through the existing application transport. An empty permitted ID set must stop before constructing an unrestricted empty Context. Carry the application's committed revision alongside the response; AIRust does not infer it.

`with_byte_budgets(query, total_text, excerpt, output)` caps complete output JSON as well as excerpts. If adding a surrounding MCP envelope or ContextAssembler graph/source bundle, measure/cap that additional encoded payload separately. Keep the original RetrievalResponse evidence when using hit IDs for sourced context assembly. The compatible default uses UTF-8 prefixes. The separately described 0.2.13 relevant mode locates lexical evidence with explicit fallbacks. Choose and test sufficient budgets.

The generic examples `advanced_search`, `storage`, `retrieval` and the API cookbook compile without any DenkMaps dependency. Run real application permissions, synchronization, German duplicate, update/restart, query relevance and large-map tests in a separate integration branch before rollout. This guide changes no application files, dependency or data.


## 10. Trial relevant excerpts after the retrieval comparison

With 0.2.13, an explicitly authorized application trial can add `with_relevant_excerpts(ExcerptOptions::default())` to the existing scoped retrieval options. This locates words/phrase hints inside long original node text. It preserves original retrieval/rerank scores and adds aligned excerpt_details. Keep the source generation and exact node text; spans use UTF-8 bytes, not JavaScript character offsets or original PDF coordinates. Convert positions for display and escape text.

TextSource forwards German stemming and phrase analysis. Semantic-only paraphrases without matching words expose a localization fallback; they do not acquire an invented supporting quotation. Set analysis/highlight limits as well as text/JSON budgets, handle budget errors explicitly and preserve the existing application fallback. Compare selected evidence with manually judged real queries before changing displayed context.

ContextAssembler retains its existing behavior. If feeding selected hit IDs to it for graph/source expansion, keep the RetrievalResponse's new excerpts, positions and ranking evidence separately; passing IDs alone does not carry the selected windows into another API. Cap any combined transport envelope independently. This documentation changes no DenkMaps source, dependency, database or MCP tool.


## 11. Consider coordinated snapshots in a separate application trial

The new optional snapshots feature offers DatasetManager as an alternative coordinated owner for lexical search, optional vectors and provenance. Acquire one DatasetSnapshot for each request, query its views and resolve sources from that same Arc. Carry its revision with selected evidence; an older retained view stays coherent but may no longer be current. An optional configured TextIndex joins the same publication boundary, with explicit extra rebuilding/memory cost.

Keep DenkMaps SQLite transactions, permissions and committed generations authoritative. Map a consistent authorized complete source image and its same-generation references into DatasetRecords, then use prepare_replace so missing IDs are removed. SourceRevision is a separate caller-provided marker, not a database watcher or distributed transaction. Preserve the application's source/access recheck before publication and handle stale prepared work by discarding/reconciling it. Do not infer matching freshness merely from the same numeric generation in unrelated lineages.

Exact unchanged text can reuse vectors; changed text needs a matching provider before anything is published. Every Put replaces its source list, so explicitly provide references intended to survive metadata edits. Model changes use prepare_model and rebuild all vectors against the retained source snapshot. Measure staging clones, retained-reader RAM, optional text-index rebuilding and model latency on real maps before adopting the new owner. HNSW/graph/cache/history remain separate projections requiring their own generation coordination.

A later integration must test simultaneous readers, concurrent commits, interrupted inference, deleted/private nodes, source-binding replacement, stale publication, model changes, restart reconciliation and transport envelope budgets. This guide performs no DenkMaps code, dependency, database or MCP integration.
