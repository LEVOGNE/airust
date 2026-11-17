use airust::{
    storage::{
        read_snapshot, sqlite::SqliteStore, MemoryStore, SnapshotOptions, StorageBackend,
        StoreChange,
    },
    KnowledgeEntry,
};
use serde_json::{json, Value};
use std::time::Instant;

fn measure(
    store: &mut dyn StorageBackend,
    count: usize,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut revision = store.revision()?;
    let start = Instant::now();
    for base in (0..count).step_by(1000) {
        let changes: Vec<_> = (base..count.min(base + 1000))
            .map(|i| {
                StoreChange::Put(KnowledgeEntry::new(
                    format!("record-{i:08}"),
                    format!("Rust knowledge search record {i}. Häuser und sichere Speicherung."),
                ))
            })
            .collect();
        revision = store.apply(&revision, &changes)?.revision;
    }
    let write_ms = start.elapsed().as_secs_f64() * 1000.0;
    let start = Instant::now();
    let mut after = None;
    let mut read = 0;
    loop {
        let page = store.page(&revision, after.as_deref(), 1000)?;
        read += page.entries.len();
        after = page.next_after;
        if after.is_none() {
            break;
        }
    }
    assert_eq!(read, count);
    let page_ms = start.elapsed().as_secs_f64() * 1000.0;
    let start = Instant::now();
    for i in 0..1000 {
        assert!(store
            .get(&revision, &format!("record-{:08}", i % count))?
            .is_some());
    }
    let get_1000_ms = start.elapsed().as_secs_f64() * 1000.0;
    let start = Instant::now();
    let loaded = read_snapshot(
        store,
        &SnapshotOptions::new(1000, count + 1, 128 * 1024 * 1024)?,
    )?
    .into_index()?;
    assert_eq!(loaded.revision(), &revision);
    assert!(!loaded.index().search("knowledge", 5).is_empty());
    let snapshot_and_index_ms = start.elapsed().as_secs_f64() * 1000.0;
    let changes: Vec<_> = (0..100.min(count))
        .map(|i| {
            StoreChange::Put(KnowledgeEntry::new(
                format!("record-{i:08}"),
                "Updated knowledge record",
            ))
        })
        .collect();
    let start = Instant::now();
    let committed = store.apply(&revision, &changes)?;
    let update_100_ms = start.elapsed().as_secs_f64() * 1000.0;
    assert_ne!(&committed.revision, loaded.revision());
    Ok(
        json!({"entries":count,"write_ms":write_ms,"page_all_ms":page_ms,"get_1000_ms":get_1000_ms,"snapshot_and_index_ms":snapshot_and_index_ms,"update_100_ms":update_100_ms,"generation":committed.revision.generation(),"durable":store.is_durable()}),
    )
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let count = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "10000".into())
        .parse::<usize>()?;
    if count == 0 || count > 1_000_000 {
        return Err("entry count must be 1..1000000".into());
    }
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("records.sqlite");
    let memory = measure(&mut MemoryStore::new("benchmark")?, count)?;
    let mut store = SqliteStore::open(&path, "benchmark")?;
    let sqlite = measure(&mut store, count)?;
    let revision = store.revision()?;
    drop(store);
    let start = Instant::now();
    let mut reopened = SqliteStore::open_read_only(&path, "benchmark")?;
    assert_eq!(reopened.revision()?, revision);
    assert_eq!(
        reopened.get(&revision, "record-00000000")?.unwrap().text,
        "Updated knowledge record"
    );
    println!(
        "{}",
        json!({"memory":memory,"sqlite":sqlite,"sqlite_reopen_get_ms":start.elapsed().as_secs_f64()*1000.0,"method":"one process, sequential adapters, synthetic records, SQLite WAL/FULL, timings include record generation/validation; peak RSS includes the derived RAM index"})
    );
    Ok(())
}
