# One coherent dataset revision

Enable `snapshots` for DatasetManager, immutable DatasetSnapshot readers and prepared updates. It reuses the existing `storage`, `embeddings` and `provenance` contracts. It enables no SQLite, web server, model download or native inference. Default crate features stay empty. Rust 1.89 supports the provider-independent API.

A published snapshot contains one lexical search owner, optional vectors and source references under one complete revision. With `advanced-search`, an explicitly configured positional TextIndex can join the same publication boundary. Read **one snapshot per request**, then borrow every component from that same snapshot.

```rust
use airust::{KnowledgeEntry, snapshots::{DatasetConfig, DatasetManager,
    DatasetRecord, DatasetUpdate, DatasetChange}, provenance::SourceReference};
let manager = DatasetManager::new("notes", DatasetConfig::default())?;
let empty = manager.snapshot()?;
let record = DatasetRecord::new(KnowledgeEntry::new("a", "Rust memory safety"))
    .with_sources(vec![SourceReference::new("manual-v1")?.with_revision("commit-1")?]);
let view = manager.publish(manager.prepare_replace(empty.revision(), vec![record], None, None)?)?;
assert_eq!(view.lexical().search("Rust", 1)[0].entry.id, "a");
assert_eq!(view.provenance().sources_for("a").next().unwrap().revision(), Some("commit-1"));
let changed = DatasetRecord::new(KnowledgeEntry::new("a", "Rust ownership safety"));
manager.publish(manager.prepare(view.revision(),
    DatasetUpdate::new(vec![DatasetChange::Put(changed)]), None)?)?;
assert!(manager.require_current(view.revision()).is_err());
assert!(view.provenance().source("manual-v1").is_some());
assert!(manager.snapshot()?.provenance().source("manual-v1").is_none());
# Ok::<(), Box<dyn std::error::Error>>(())
```

Run `cargo run --features snapshots,retrieval --example snapshots` for the compiled vector/relevant-excerpt example. Add `advanced-search,context-assembly` to exercise an optional positional projection and sourced context from the retained view. The example uses handwritten vectors, not a neural relevance evaluation.

## Ownership and publication

`DatasetManager::new(namespace, config)` starts an empty generation zero with a random 128-bit identity. Revisions reuse the validated StoreRevision format: namespace, identity and generation in 0..=i64::MAX. Compare the **complete** value. A manager clone shares the same publication slot; a separately created manager gets another identity. The revision describes this in-process dataset lineage, not an implicit storage backend or an authentication capability.

`snapshot()` clones an Arc through a short read lock. DatasetSnapshot exposes immutable `lexical`, optional `semantic`, `provenance`, configuration, encoded-record byte count, preparation statistics and revision accessors. There is no mutable index/source escape hatch. Semantic mode owns one existing SemanticIndex and borrows its lexical Engine; records are not stored in a second canonical Engine. Lexical-only mode owns an Engine directly.

`prepare(expected, update, provider)` checks the starting revision, validates bounded changes and builds a candidate outside the publication lock. `publish(prepared)` takes a short write lock, checks the expected revision again and swaps the complete Arc only if it still matches. Losing concurrent work returns Conflict without publishing any part. The old snapshot is released outside the write lock. Generation exhaustion fails without publication. Prepared data has private fields and cannot be deserialized or edited into an unchecked candidate.

Old readers deliberately retain the old complete snapshot while new readers acquire the new one. `require_current(view.revision())` checks whether a retained view is still current at that instant. It does not guarantee that no future publication occurs after the check. Capture the revision alongside results, preserve the Arc when original text or sources are needed later, and recheck freshness/access under application policy before sending externally. Calling snapshot separately for every component can mix different generations; reuse one acquired view.

Provider work, index cloning, analysis and source staging happen outside publication locks. Provider errors or unwinding panics leave the published state intact; a panic is not caught by the API, and abort-on-panic builds still abort. A poisoned publication lock fails closed. An application can move blocking preparation to a bounded worker. This API imposes no async executor or model worker/pool policy.

## Changes, complete reconciliation and vectors

DatasetUpdate contains Put(DatasetRecord) and Delete(ID) changes, with unique IDs. A DatasetRecord contains an entire KnowledgeEntry and the complete desired source-reference list. Every accepted update publishes one new generation, including an equal-value put or deletion of an absent ID. An empty update needs an advancing external source marker; otherwise use no operation. `PreparationStats` reports puts, actual deletes, newly embedded entries, reused vectors and positional entries rebuilt.

`prepare_replace(expected, records, source_revision, provider)` reconciles a complete authoritative record list. Missing old IDs are deleted from search, vectors and sources together. Duplicate IDs fail. It uses total dataset limits instead of delta batch operation/byte caps, allowing an initial load or a full reconciliation larger than 1,000 entries. It still obeys per-record, source, vector and total-byte limits, and embedding calls remain bounded batches.

Select `DatasetConfig::with_embedding_space` to enable semantic mode. New or byte-changed text then requires a matching EmbeddingProvider. Vector dimensions, finite values, counts and model recipe are validated through the existing semantic API. No record with pending vectors is published. Exact unchanged text reuses its existing Arc-backed vector even when metadata or source bindings change; metadata is not part of the provider's text input. Deletes remove vectors. A wrong provider supplied for an update requiring inference fails; an unused provider is not invoked.

`prepare_model(expected, Some(provider))` explicitly enables, refreshes or replaces the model for every current entry, publishing only after the entire rebuild succeeds. `None` deliberately switches to lexical-only mode. Model changes preserve records, sources and any external source marker, while advancing the dataset revision. Existing snapshots keep their previous model and vector space. Query providers must still match the chosen snapshot's EmbeddingSpace. Complete model recipe identity, including preprocessing and immutable model revision, remains the provider/caller's responsibility.

This is copy-on-write publication, not an O(1) update of an arbitrary-size index. A draft clones the existing index/source bookkeeping; unchanged vector buffers are shared. Retained readers, concurrent candidates and model rebuilds increase peak memory. Limit retained views and work queues. Preparation statistics describe vector reuse, not a promise that no source/index copying occurred.

## Sources and optional positional search

Every Put **replaces** the entry's source bindings. DatasetRecord::new starts with no sources; omitting sources clears previous bindings, including for metadata-only puts. Supply a checked complete list when preserving them is intended. Nothing silently retains old citations after a text replacement. Missing references are allowed and remain Missing; the system does not invent evidence.

A source ID identifies one immutable SourceReference definition. When changing a shared definition, update/delete every entry still bound to the old definition in the same batch; otherwise the conflicting definition is rejected. Staging removes changed bindings, prunes unused definitions, validates replacements and publishes all links together. Unused definitions are pruned. Source locations remain caller claims about external artifacts, not offsets automatically validated against the shorter KnowledgeEntry text. This mechanism enforces structural consistency, not source truth, file existence, permissions or semantic entailment.

With `advanced-search`, `DatasetConfig::with_text_analyzer` adds a positional projection with the selected German/tokenizer configuration. A record update rebuilds this projection from the complete staged source state before inference/publication; analysis failures publish nothing. It carries original text/metadata identical to the canonical index, but the current TextIndex design stores another source copy. Budget and measure that optional cost. Model-only changes share the unchanged positional Arc. `text_index()` is None unless explicitly configured. HNSW topology, KnowledgeGraph, derived cache files and entry-history journals are not part of this publication boundary.

## External committed revisions

DatasetUpdate::with_source_revision and prepare_replace's source_revision argument can carry an authoritative StoreRevision from a consistent external read. It is distinct from the dataset's own revision. Once bound, later data updates must supply a strictly newer generation from the same source namespace/identity. Missing, equal, older and foreign-lineage markers fail. Model-only rebuilds preserve the marker. Restored/independently copied source lineages require a separately coordinated fresh manager rather than silently accepting old prepared tokens.

A usual read/reconcile flow is: obtain a consistent committed source snapshot → map its entries and same-generation sources to DatasetRecords → prepare_replace against the acquired dataset revision → recheck the authoritative source/access state as required → publish with dataset CAS. Storage read_snapshot already supplies a consistent record list and its StoreRevision; its contents can be mapped explicitly without changing that backend.

The marker is a caller assertion. AIRust does not open/watch an application database, validate that every supplied delta matches that database, or make its external transaction atomic with RAM publication. In particular, record storage does not automatically store the separately supplied provenance list. Applications must read/persist those references consistently and coordinate source rechecks or retries. Complete reconciliation prevents forgotten local deletions when the supplied full source image is correct. There is no DenkMaps, MCP, database migration or automatic integration in this release.

## Limits and wire composition

DatasetConfig defaults to 100,000 entries, 128 MiB encoded records including source lists, 256 MiB of published f32 vector components, 8,192 dimensions, 64 source bindings per entry and embedding batches of 32. Reused StorageLimits default to 4,096-byte IDs, 1 MiB encoded records, 1,000 delta operations / 16 MiB delta bytes, metadata depth 64 and 100,000 nodes. Source metadata is checked before cloning/serialization. These limits exclude index/tree/Arc overhead, simultaneous generations, provider allocations and exact peak heap use.

Builders configure positive dataset limits, StorageLimits, source count, dimension and batch size. Capacity checks consider the final batch state rather than temporary operation order. Source/model/record failures return typed DatasetError values; invalid configuration, conflicts, missing providers and poisoned state are distinct. There is no automatic retry that can repeatedly run inference or overwrite concurrent work.

Borrow existing retrieval or ContextAssembler APIs from one view, and use its provenance for returned IDs. Compact RetrievalResponse remains a separate bounded output object. Adding revision/source/context fields to a transport envelope adds bytes outside that response's JSON cap; bound the combined envelope explicitly. A snapshot revision does not grant access: preserve existing request scopes and return no results for an empty authorized set before constructing an unrestricted empty Context. The snapshot itself is a RAM read view, not a new persistent backup/export format or a completed independent security audit.

See [API cookbook](API-COOKBOOK.md), [migration](MIGRATION-0.2.14.md) and [release report](CHANGES-0.2.14.md) for executed validation and measured cloning/rebuild costs.
