# Storage Backend API

Enable `storage` for the provider-neutral contract and `MemoryStore`. Add `sqlite-storage` for a standalone `SqliteStore`. Neither feature enables the web server. Default features remain empty. Rust 1.89 is supported.

A backend owns **original knowledge records**, not a persistent inverted index. `Engine`, `TextIndex`, vectors, graph edges, provenance and caches remain separately derived or application-owned data. Loading a snapshot builds a RAM index; this is not out-of-core search or automatic database synchronization.

```rust
use airust::{KnowledgeEntry, storage::{MemoryStore, StorageBackend, StoreChange,
    read_snapshot, SnapshotOptions}};
let mut store = MemoryStore::new("notes")?;
let initial = store.revision()?;
let committed = store.apply(&initial, &[
    StoreChange::Put(KnowledgeEntry::new("a", "Rust memory safety"))
])?;
let loaded = read_snapshot(&mut store, &SnapshotOptions::default())?.into_index()?;
assert_eq!(loaded.revision(), &committed.revision);
assert_eq!(loaded.index().search("Rust", 1)[0].entry.id, "a");
# Ok::<(), Box<dyn std::error::Error>>(())
```

## One explicit contract

`StorageBackend` is object-safe. Implement `limits`, `revision`, `get`, `page` and `apply`; optionally override durability/read-only flags and `renew_identity`. Calls take `&mut self`; concurrent callers use separate connections or application synchronization. No asynchronous executor or connection pool is imposed.

Each `StoreRevision` contains a namespace, random 128-bit lowercase-hex identity and generation in 0..=i64::MAX. Compare the **complete** token. Every read/page/write receives the expected token and fails with `Conflict` if it differs. Nonempty accepted batches increment the generation once, including equal-value puts or deleting an absent ID. Empty batches check the token and leave the generation unchanged. Exhaustion fails without mutation. `renew_identity` keeps records, generates a new identity and resets generation to zero.

Tokens detect stale writes within a coordinated lineage. They are not authentication, per-entry history, timestamps or a distributed consensus protocol. Full database restoration or independent file copies can recreate an old token. Rotate identity before accepting clients against restored/copied data and coordinate independent copies externally. Namespace isolation is not authorization.

`StoreChange::Put` inserts/replaces an entire entry; `Delete` removes an ID. A batch is atomic: validate every entry, duplicate ID and budget before changing records; commit every operation plus its revision or return an error with no partial commit. Commit counts distinguish accepted puts from actual deletions. A custom adapter must uphold the same contract; AIRust cannot repair a dishonest adapter that mutates without advancing its token.

`page(expected, after, limit)` returns strictly increasing UTF-8-byte-ordered IDs, exclusively after the supplied ID. It respects both row and encoded-record byte budgets. `next_after` equals the last returned ID only when more records exist. A record that does not fit the remaining page is deferred, never skipped. If one record cannot fit an empty page, return a limit error. A cursor is meaningful only with its namespace/revision; restarting at a newer revision is an explicit caller decision.

## Limits and consistent snapshots

`StorageLimits::default()` permits 4,096-byte IDs, 1 MiB encoded entries, 1,000 operations / 16 MiB per batch, and 1,000 rows / 4 MiB per page. Metadata is capped at depth 64 and 100,000 JSON nodes. Builders validate positive limits; depth cannot exceed 64. Limits count UTF-8/encoded JSON bytes, including escapes, not characters or total heap. Put/delete IDs within a batch must be unique. Invalid IDs, blank text, malformed stored JSON and ID/record disagreement fail explicitly.

`read_snapshot` pins one complete revision across every page and checks it again after the final page. It validates ordering, cursors, entries and page budgets even for custom adapters. A concurrent write or malformed page discards the staged load; it returns no partially usable snapshot. The default snapshot permits 128 rows per page, one million entries and 128 MiB encoded records. Configure `SnapshotOptions` within the backend page limits; retry conflicts with bounded application policy rather than an endless internal loop.

`StoredSnapshot` exposes immutable records and its revision. `into_index` consumes records and creates `LoadedIndex`, whose immutable `index()` stays associated with that loaded revision. `into_parts` explicitly transfers ownership for custom indexing, including `TextIndex::from_entries`; the caller then owns freshness tracking. Subsequent writes do not update previously loaded indexes. Rebuild/stage and swap a derived view only after a successful authoritative commit/snapshot. A crash between source commit and RAM update requires a reload, not rollback of already committed data.

## SQLite adapter

```text
use airust::storage::sqlite::SqliteStore;
let mut store = SqliteStore::open("/private/application/knowledge.sqlite", "notes")?;
let read_only = SqliteStore::open_read_only("/private/application/knowledge.sqlite", "notes")?;
```

The executable `storage_benchmark` exercises this API against real temporary files. The parent directory must already exist. New Unix files use mode 0600. Symlink/nonregular paths are rejected; use a trusted private directory because filesystem replacement races and hostile SQLite files are outside the contract. No URI path syntax is enabled. Existing permissions are not silently changed.

The adapter owns a separate AIRust database with three versioned tables. It rejects unrelated databases, additional schema objects and unknown format versions instead of altering an application's schema. Do not point it at DenkMaps or another application's database. Namespaces share a file but have independent records and revision identities. A missing namespace is initialized only on a writable connection.

File-backed writes use SQLite WAL and `synchronous=FULL`, immediate transactions, parameterized values and a default five-second busy timeout. Reads fence revision and rows in one read transaction. Set a bounded alternative with `set_busy_timeout`; a busy/SQL/commit error propagates and the transaction rolls back. Separate connections racing with the same token produce one successful writer and a conflict (or a lock timeout if the writer cannot acquire the lock). Read-only writes/identity renewal fail. `in_memory` follows the same data contract but reports non-durable storage.

Durability depends on SQLite, filesystem and hardware guarantees. Copying only a live main file may omit WAL data: use a coordinated SQLite backup or shut down/checkpoint writers first. This API does not add encryption, automatic backup, history, migrations of unrelated schemas or automatic cache invalidation. The [release report](CHANGES-0.2.11.md) describes measured behavior and fault tests.
