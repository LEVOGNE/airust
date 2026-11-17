# Migrating to AIRust 0.2.7

The independent `versioning` feature is additive within Cargo's 0.2 compatibility range. It adds no dependency and enables shared Serde references. Default features, original knowledge-entry fields, portable source JSON, exact/ANN retrieval, derived cache and provenance formats remain unchanged. Rust 1.89 is supported.

```toml
airust = { version = "=0.2.7", default-features = false, features = ["versioning"] }
```

Create `VersionedIndex::new()` or consume an existing engine with `from_index(engine, change_metadata)`. Supply `ExpectedRevision::Absent` for a never-seen ID and `Exact(current)` for changes or recreating a deleted ID. There is no implicit force-write option. Tombstones prevent normal delete/recreate ABA conflicts. Current revisions are monotone u64 numbers within this index lineage; they are not globally unique. Applications provide their own synchronization and database transaction checks.

History is disabled by default. Opt in with `with_history_limit(n)`; this retains at most n committed snapshots globally, including deletion states. It intentionally copies retained snapshot text and counts records rather than bytes. Changing the limit cannot reconstruct past unrecorded/pruned states. `compare` reports field/existence changes, and `restore` creates a new revision from an available state, never an old counter value. Actor/reason/metadata are caller claims, not authenticated audit evidence.

`to_json`/`from_json` preserve current records, tombstones and retained history in independent format 1. The default input cap is 16 MiB, with an explicit override; generic Serde uses the caller's resource policy. Imports validate source and revision consistency. Loading an old entire snapshot or rebuilding an unrelated wrapper needs a new application coordination epoch and discarded outstanding CAS values. Consuming `into_index` explicitly ends revision bookkeeping. Existing engine source JSON remains a separate format.

A restored entry does not automatically update SQLite, refresh an embedding, restore graph links or prove old provenance still supports it. Reconcile those components after an authorized application commit. No external integration is performed. See [VERSIONING.md](VERSIONING.md).
