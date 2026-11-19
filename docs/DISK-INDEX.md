# Optional disk search index

Enable `disk-index`. It uses SQLite FTS5 through the existing optional bundled SQLite dependency, with native cancellation hooks. It adds no model, server or application integration. Source entries and the derived full-text index live in a dedicated AIRust database. Do not open an application's unrelated SQLite database with this adapter.

## Own format and atomic updates

DiskIndex::open(path, namespace, options) creates an empty database or checks its application ID, format, namespace, analysis recipe and declared limits. Unexpected tables/views/triggers or unrelated schemas are rejected. It does not load all entries into a Rust Engine. A compatible Unicode/SQLite analysis recipe is required; on mismatch rebuild from original records. An application-owned private directory and trusted database file are required. Format checks are not file authentication or protection against arbitrary hostile SQLite binaries.

revision returns a persistent StoreRevision. apply(expected, changes, work) stages bounded StoreChange::Put/Delete operations, checks the expected identity/generation and updates records, FTS rows and revision in one SQLite transaction. Stale writers fail without changes. Any batch failure rolls back the entire batch. Empty batches return the unchanged revision after checking it; nonempty accepted batches advance it even if a delete finds no record. Reopen preserves committed revisions. The adapter uses rollback-journal DELETE mode with FULL synchronous writes, a 500 ms busy timeout, a bounded SQLite page cache, no mmap and file-backed temporary work. WAL mode is not enabled. Lock contention is an explicit SQLite error; applications may retry from a current revision.

The revision covers this database's original entries and FTS index. Vectors, graphs, provenance and an external application's transactions are not included. DatasetManager RAM snapshots do not automatically cover a DiskIndex. Coordinate corresponding source revisions explicitly before combining different sources. The database is not a distributed transaction or a source watcher.

## Search and transport-neutral retrieval

search(query, DiskSearchOptions) returns owned original entries, scores, matched terms and the read transaction's revision. Rust core NFKC/lower normalization is followed by SQLite unicode61 tokenization with diacritic removal disabled. Query terms are safely quoted literal OR terms. Raw SQLite query syntax is never executed from user input. Phrase queries, German stemming and tokenization parity with TextIndex are not claimed. Empty/no-term queries return no hits.

Scores are negated native SQLite FTS5 BM25 values so larger is better. They are not interchangeable with Core BM25 scores. The distinct recipe includes Unicode and SQLite versions and must be used for calibrated score gates. Stable ties use entry ID. Returned matched terms use core literal normalization; an FTS match without such overlap must not be described as an invented supporting passage.

with_context filters eligible records before applying the top-K result limit. An empty Context ID set is unrestricted, as in Core. If authorization yields no IDs, stop before querying. Native ranking statistics still reflect the whole index; use separate authorized/tenant indexes where statistics or timing must be isolated. This is not an authorization service. A bounded scan may encounter excluded candidates; reaching the scan cap returns ScanLimit, never a misleading partial success. Internal scan counts are not returned.

DiskSource::new(&mut index) implements RetrievalSource and works with Retriever. The source retains only the bounded current candidate batch and exposes last_revision after a successful backend read. Later response validation/budgeting can still fail; use the query Result as the final outcome. Original text is retained for relevant excerpts and UTF-8 positions. Recipe, scope and optional WorkControl are forwarded; technical errors/interruptions do not become no-answer results.

## Limits, cancellation and original-record export

Defaults: one million entries, 4 GiB total encoded original-entry bytes and a 2 MiB SQLite page-cache target. DiskOptions::new supports up to ten million entries, 64 GiB encoded originals and 64 KiB..64 MiB page-cache targets. FTS content/postings, journals, temporary sorting and filesystem caches are extra disk/memory costs. These settings are not exact process-RSS or whole-machine memory quotas.

StorageLimits defaults bound individual encoded entries to 1 MiB, IDs to 4 KiB, batches to 1,000 operations/16 MiB and metadata structure. Normalized entry text caps at 4 MiB. Queries cap at 16 KiB and 64 distinct terms; results at 1,000. Scoped scans default to 100,000 candidate rows and can be configured up to one million. A limit error returns no partial successful batch/result.

With execution enabled, DiskWork and DiskSearchOptions accept WorkControl. Checkpoints and SQLite VM progress hooks support cooperative cancellation/deadlines through writes, queries and integrity checks. A successful commit is never reported as cancelled afterward. SQLite lock waits, filesystem I/O and individual bounded operations can overshoot a deadline; this is not hard preemption. Hooks only inspect cancellation/deadlines and do not invoke user observers while SQLite locks are held. Use outer retrieval/application progress events for visible start/end activity.

page(expected_revision, after_id, limit) exports original entries in stable ID order, with bounded page size and next_after. Pages cap at 1,000 entries/4 MiB encoded entry bytes, excluding the application envelope. A revision change fails the next page so an export cannot silently mix generations. Export requires authorization to the whole index; it is not a scoped search endpoint. Preserve these originals independently when the index is disposable derived state.

integrity_check(work) performs SQLite/FTS checks plus a bounded-row scan comparing original records, normalized FTS content, row membership and metadata counts. It can be costly. Opening the index validates metadata/schema, not every payload. Do not copy a database file during active writes as an uncoordinated backup; coordinate exports/snapshots and application ownership.

## Validation and measurements

Tests cover reopen, updates/deletes, scoped top-K, literal query syntax, Unicode, generation conflicts across connections, complete rollback, byte/count/scan limits, logical FTS damage, bounded exports, seeded membership against an authoritative set, cancellation and an abrupt writer exit with a hot rollback journal. The latter is an injected crash point, not exhaustive power-loss or filesystem certification.

Run disk_benchmark with `100000 memory` and `100000 disk`. Both modes use the same generator/executable in separate processes. Native scoring recipes differ; their top IDs agree on this fixture. Record process peak RSS, build/reopen/query times and disk bytes. Process RSS excludes much of the OS filesystem cache, and reopen is not a guaranteed cold-device test. These synthetic measurements do not establish real application capacity or universal performance gains. Full data and limits are in the release report. No DenkMaps/MCP integration.

## Complete example

The demonstration uses `tempfile = "3"` as a development dependency to clean up its temporary database. An application can pass its own controlled path directly.

```rust
use airust::{disk_index::*, retrieval::*, storage::StoreChange, KnowledgeEntry};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let folder = tempfile::tempdir()?;
    let path = folder.path().join("index.sqlite");
    let mut index = DiskIndex::open(&path, "demo", DiskOptions::default())?;
    let rev = index.revision()?;
    let committed = index.apply(
        &rev,
        &[StoreChange::Put(KnowledgeEntry::new(
            "a",
            "Rust ownership prevents memory errors",
        ))],
        &DiskWork::default(),
    )?;
    drop(index);
    let mut index = DiskIndex::open(&path, "demo", DiskOptions::default())?;
    assert_eq!(index.revision()?, committed);
    let mut retriever = Retriever::new(DiskSource::new(&mut index));
    let response = retriever.query("Rust ownership", &RetrievalOptions::new(3))?;
    assert_eq!(response.hits()[0].id, "a");
    println!("{}", response.to_json());
    Ok(())
}

```
