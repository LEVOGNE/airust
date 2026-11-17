# Optional revisions and retained history

Enable `versioning` for `VersionedIndex`, positive `Revision` values, `ExpectedRevision`, `ChangeMetadata`, version records and comparisons. It adds no dependency and enables Serde's shared-reference support. Existing knowledge entries and source JSON are unchanged. Current records live in one existing lexical index; change metadata can share immutable storage.

`VersionedIndex::new()` starts empty. `from_index(engine, change)` consumes an existing index and assigns revisions in sorted ID order without copying source text. `index()` borrows it for existing retrieval; `into_index()` returns ownership and ends this wrapper's revision bookkeeping. There is no mutable escape. Revision numbers are monotone u64 values within this particular index lineage, not globally unique IDs or authenticated timestamps.

Every write requires `ExpectedRevision::Absent` for a never-seen ID or `Exact(current_revision)`. A deleted ID has a tombstone and is not absent. `upsert` inserts/replaces/recreates content, `delete` records a deletion, and `restore` copies an available prior or current state into a new revision. Restore can restore deletion and never rewinds the sequence. Invalid entries, stale expectations, missing/pruned revisions and sequence exhaustion return errors without changing current records, revision heads or history. Applications coordinate their own locks and database transactions.

`ChangeMetadata::default()` is empty. Optional checked actor/reason builders accept 1..1024 plain-text characters, trim surrounding spaces and reject controls. Arbitrary JSON metadata is optional. Labels are caller claims, not authenticated identities or a tamper-proof audit log. The engine invents no wall-clock time.

The default history limit is zero: revisions/conflicts work without retaining historical text. `with_history_limit(n)` opts into at most n committed snapshots globally, including deletion states and possibly current snapshots. Reducing the limit prunes immediately; increasing it does not recover previously unrecorded history. Snapshot text is copied only when this journal is enabled. The limit counts records, not bytes; choose it for actual text sizes. This is a bounded helper, not full event sourcing or permanent application history.

`version(id)` and `revision(id)` inspect the latest live/deleted head. `history()` and `history_for(id)` borrow retained committed snapshots in sequence order. `at(id, revision)` borrows a current or retained state; unavailable versions fail explicitly. `compare(id, from, to)` reports existence, text and metadata-field changes without a generated prose diff. Missing entries differ from present fields, even when metadata itself is absent. `RevisionView` preserves the original state; restoring it creates a new `Restore` action with caller-supplied change metadata.

Versioned JSON format 1 preserves current records, heads/tombstones, the sequence and retained journal. Imports validate entry IDs/text, unique heads, exact live/deleted agreement, positive/ordered revision identities, action/state consistency, current-snapshot agreement and journal capacity. `from_json` defaults to a 16 MiB input cap; `from_json_with_limit` sets a caller policy. Generic Serde validates the schema under the caller's input-resource policy. Numeric u64 revisions require consumers that preserve integer precision. Loading rebuilds the lexical index; portable `Engine` source JSON and derived cache format remain separate.

Rebuilding an unrelated wrapper from entries creates a new revision lineage; do not reuse old expected revisions across it. Applications retain the versioned snapshot or maintain their own authoritative revision mapping. SQLite commits, map permissions, semantic-vector refreshes, graph edges and provenance invalidation remain caller-owned. A restored entry does not automatically restore those sidecars or an application's database.

Loading an older whole-index snapshot is an application-level rollback, not the `restore` operation. Start a new coordination epoch and discard outstanding expected revisions when performing such a rollback. This library cannot detect a caller replacing its entire authoritative snapshot with an older one.

## Minimal usage

```rust
use airust::{versioning::{ChangeMetadata, ExpectedRevision, VersionedIndex}, KnowledgeEntry};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let mut index = VersionedIndex::new().with_history_limit(8);
let first = index.upsert(KnowledgeEntry::new("a", "Original note"),
    ExpectedRevision::Absent, ChangeMetadata::default())?;
let second = index.upsert(KnowledgeEntry::new("a", "Edited note"),
    ExpectedRevision::Exact(first), ChangeMetadata::default())?;
assert!(index.compare("a", first, second)?.text_changed());
let restored = index.restore("a", first, ExpectedRevision::Exact(second),
    ChangeMetadata::default().with_reason("Restore original")?)?;
assert!(restored > second);
assert_eq!(index.index().get("a").unwrap().text, "Original note");
# Ok(())
# }
```

Run `cargo run --locked --features versioning --example versioning`. Reproducible measurements are provided by `versioning_benchmark` and the release report.

`from_index` assigns current heads without retaining imported snapshots. Enabling history afterwards records future commits; it does not retroactively checkpoint the imports. Cloning a versioned index creates independent mutable state, not a shared concurrency coordinator.

History pruning does not remove revision heads or tombstones. Their memory cost follows all tracked IDs, including deleted ones. Reclaiming those identities requires an application-controlled new coordination epoch so old expected revisions cannot be reused.
