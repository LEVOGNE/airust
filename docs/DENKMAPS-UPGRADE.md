# Proposed DenkMaps upgrade to AIRust 0.2.9

This is an implementation guide, not a claim that DenkMaps uses the new release. The AIRust sequence keeps its core independent. The existing DenkMaps application was inspected read-only; the new library is not integrated automatically.

## What already exists

The inspected DenkMaps checkout depends on AIRust `=0.1.9`, with default features disabled. `src/knowledge_index.rs` already uses Engine/KnowledgeEntry. It reconciles committed SQLite revisions through `wissen_stand` and `wissen_karte`, caches per database identity/tenant, bounds nodes/bytes and falls back to the legacy scan when needed. Its inspected Unicode/non-ASCII query path deliberately preserves the legacy search behavior. Private-map exclusions are applied to results.

SQLite's `karte` data and existing application versions remain authoritative. The adapter maps node titles/notes into stable `map:<map-id>/node:<node-id>` IDs within its tenant index. Do not replace this synchronization with an uncoordinated second database or rewrite identifiers unnecessarily. If changing to a shared multi-tenant index, explicitly namespace tenant identity and test permissions first.

Other local UI/import/MCP work is separate from this library upgrade. Re-read current DenkMaps sources before implementing: concurrent application work can change these inspected paths/contracts.

## 1. Upgrade the dependency without enabling everything

Start with the existing lexical dependency and run all application tests:

```toml
airust = { version = "=0.2.9", default-features = false }
```

This uses the same generic entry shape and existing lexical API. Keep a configuration rollback to the current search. Compare result IDs, counts and authorization before changing ranking behavior. Do not remove the Unicode fallback merely because the new version compiles.

When semantic/context work is explicitly approved and tested, select the needed features, for example:

```toml
airust = { version = "=0.2.9", default-features = false,
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
