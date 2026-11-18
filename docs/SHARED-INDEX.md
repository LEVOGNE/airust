# Optional shared index storage

Enable `shared-index` explicitly. This feature adds no dependency, model, server or persistent format. Defaults remain unchanged. It changes private in-process storage used by the existing lexical Engine; public methods and ownership rules stay the same. Semantic indices and dataset snapshots containing that engine benefit when the flag is enabled too. The snapshots feature does not enable it automatically.

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use airust::{Engine, KnowledgeEntry};
let mut current = Engine::from_entries([KnowledgeEntry::new("a", "Original Rust note")])?;
let earlier = current.clone();
current.upsert(KnowledgeEntry::new("a", "Changed Rust note"))?;
assert_eq!(earlier.get("a").unwrap().text, "Original Rust note");
assert_eq!(current.get("a").unwrap().text, "Changed Rust note");
# Ok(()) }
```

## What is shared

Private immutable analyzed records, per-term posting maps and normalized duplicate-ID sets use Arc only under this flag. Cloning still copies outer ordered maps and their keys; it is not an O(1) operation. Buffers remain shared until an edit needs to change them. Arc::make_mut detaches changed posting/duplicate bindings; removing an owned record reuses it when unique or clones that record when an earlier view still holds it. Unchanged documents retain their buffers. No mutable buffer is shared between independent indices.

Returned records, metadata, ranks, scores, duplicate behavior, portable JSON and derived-cache formats remain unchanged. A previous index copy or DatasetSnapshot remains immutable after later upserts/removal/clear. Clones can be read or independently edited from different threads without adding a global index lock. The application's dataset/permission model remains its responsibility; sharing does not merge unrelated datasets or synchronize external storage.

## Cost and selection

Choose the flag for workloads with several retained copies, prepared updates or old read generations. Each shared record adds an allocation/reference-counting boundary; a single-owner build or lookup may have a cost. Editing a common term can still copy a large posting map, and each clone copies outer maps. Provenance definitions and positional analysis have their existing separate costs. This is neither an on-disk index nor operating-system/process-shared memory.

Compare default and flagged builds on the same workload. The unchanged `stress_index` generator measures three retained copies at 500k records, single-index scale at two million records and subsequent mutations. `stress_snapshots` measures concurrent old/new views. Raw binary/source hashes, feature choices and OS resources ship with the [0.2.20 report](CHANGES-0.2.20.md). No universal speed or capacity improvement is claimed.

```bash
cargo build --release --locked --features shared-index --example stress_index
python3 scripts/measure.py /tmp/shared-clones.json target/release/examples/stress_index clone 500000
cargo build --release --locked --features shared-index,snapshots,execution --example stress_snapshots
```

All-feature and shared-only regressions check unchanged old records/metadata, returned removed values, scores, normalized duplicate sets, retained generations and concurrent independent writers. Private boundary tests verify sharing before edits and detachment afterward. Existing cache/semantic/provenance/retrieval tests remain enabled. No application integration is performed.
