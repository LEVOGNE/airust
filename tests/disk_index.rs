#![cfg(feature = "disk-index")]
use airust::{disk_index::*, retrieval::*, storage::StoreChange, Context, KnowledgeEntry};
fn put(id: &str, text: &str) -> StoreChange {
    StoreChange::Put(KnowledgeEntry::new(id, text))
}
#[test]
fn durable_search_updates_delete_and_reopen_keep_one_revision() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("index.db");
    let mut db = DiskIndex::open(&path, "n", DiskOptions::default()).unwrap();
    let r = db.revision().unwrap();
    let r = db
        .apply(
            &r,
            &[put("a", "Rust ownership"), put("b", "German Häuser")],
            &DiskWork::default(),
        )
        .unwrap();
    assert_eq!(
        db.search("Rust", &DiskSearchOptions::new(2).unwrap())
            .unwrap()
            .hits()[0]
            .entry
            .id,
        "a"
    );
    drop(db);
    let mut db = DiskIndex::open(&path, "n", DiskOptions::default()).unwrap();
    assert_eq!(r, db.revision().unwrap());
    let r = db
        .apply(
            &r,
            &[put("a", "changed content"), StoreChange::Delete("b".into())],
            &DiskWork::default(),
        )
        .unwrap();
    assert!(db
        .search("Rust Häuser", &DiskSearchOptions::new(10).unwrap())
        .unwrap()
        .hits()
        .is_empty());
    assert_eq!(db.len().unwrap(), 1);
    assert_eq!(
        db.search("changed", &DiskSearchOptions::new(1).unwrap())
            .unwrap()
            .revision(),
        &r
    );
    db.integrity_check(&DiskWork::default()).unwrap();
}
#[test]
fn scoped_results_are_filtered_before_top_k_and_retrieval_preserves_original_text() {
    let folder = tempfile::tempdir().unwrap();
    let mut db =
        DiskIndex::open(folder.path().join("index.db"), "n", DiskOptions::default()).unwrap();
    db.apply(
        &db.revision().unwrap(),
        &[
            put("private", "Rust"),
            put("public", "Rust ownership prevents errors"),
        ],
        &DiskWork::default(),
    )
    .unwrap();
    let options = RetrievalOptions::new(1).with_context(Context::new().with_entry("public"));
    let mut retriever = Retriever::new(DiskSource::new(&mut db));
    let result = retriever.query("Rust", &options).unwrap();
    assert_eq!(result.hits()[0].id, "public");
    assert!(retriever.source().last_revision().is_some());
    assert!(!result.to_json().contains("private"));
}
#[test]
fn stale_writers_and_failing_batches_never_partially_commit() {
    let folder = tempfile::tempdir().unwrap();
    let mut db =
        DiskIndex::open(folder.path().join("index.db"), "n", DiskOptions::default()).unwrap();
    let original = db.revision().unwrap();
    let newer = db
        .apply(&original, &[put("a", "one")], &DiskWork::default())
        .unwrap();
    assert!(matches!(
        db.apply(&original, &[put("b", "two")], &DiskWork::default()),
        Err(DiskError::Conflict)
    ));
    assert!(db
        .apply(
            &newer,
            &[put("b", "two"), put("invalid", "")],
            &DiskWork::default()
        )
        .is_err());
    assert_eq!(db.revision().unwrap(), newer);
    assert_eq!(db.len().unwrap(), 1);
}
#[test]
fn unrelated_database_and_namespace_are_rejected_without_rewriting_data() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("foreign.db");
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute("CREATE TABLE application(secret TEXT)", [])
        .unwrap();
    drop(connection);
    assert!(DiskIndex::open(&path, "n", DiskOptions::default()).is_err());
    let connection = rusqlite::Connection::open(&path).unwrap();
    assert!(connection.prepare("SELECT secret FROM application").is_ok());
    let path = folder.path().join("index.db");
    drop(DiskIndex::open(&path, "n", DiskOptions::default()).unwrap());
    assert!(DiskIndex::open(&path, "other", DiskOptions::default()).is_err());
}
#[test]
fn query_syntax_is_data_and_unicode_text_is_preserved() {
    let folder = tempfile::tempdir().unwrap();
    let mut db =
        DiskIndex::open(folder.path().join("index.db"), "n", DiskOptions::default()).unwrap();
    db.apply(
        &db.revision().unwrap(),
        &[
            put("de", "Die Häuser haben Türen"),
            put("literal", "AND OR NEAR"),
        ],
        &DiskWork::default(),
    )
    .unwrap();
    for query in ["Häuser", "AND", "\" OR * NOT :"] {
        let _ = db
            .search(query, &DiskSearchOptions::new(10).unwrap())
            .unwrap();
    }
    assert_eq!(
        db.search("Häuser", &DiskSearchOptions::new(1).unwrap())
            .unwrap()
            .hits()[0]
            .entry
            .text,
        "Die Häuser haben Türen"
    );
    assert!(db
        .search("", &DiskSearchOptions::new(1).unwrap())
        .unwrap()
        .hits()
        .is_empty());
}
#[test]
fn scan_and_output_limits_fail_explicitly() {
    let folder = tempfile::tempdir().unwrap();
    let mut db =
        DiskIndex::open(folder.path().join("index.db"), "n", DiskOptions::default()).unwrap();
    db.apply(
        &db.revision().unwrap(),
        &[put("a", "Rust"), put("b", "Rust"), put("c", "Rust")],
        &DiskWork::default(),
    )
    .unwrap();
    let o = DiskSearchOptions::new(1)
        .unwrap()
        .with_scan_limit(1)
        .unwrap()
        .with_context(Context::new().with_entry("z"));
    assert!(matches!(db.search("Rust", &o), Err(DiskError::ScanLimit)));
    assert!(DiskSearchOptions::new(1001).is_err());
}
#[test]
fn index_size_limits_rollback_an_entire_batch() {
    let folder = tempfile::tempdir().unwrap();
    let mut db = DiskIndex::open(
        folder.path().join("index.db"),
        "n",
        DiskOptions::new(1, 4096, 64).unwrap(),
    )
    .unwrap();
    let rev = db.revision().unwrap();
    assert!(db
        .apply(
            &rev,
            &[put("a", "one"), put("b", "two")],
            &DiskWork::default()
        )
        .is_err());
    assert_eq!(db.revision().unwrap(), rev);
    assert!(db.is_empty().unwrap());
    db.integrity_check(&DiskWork::default()).unwrap();
}
#[test]
fn independent_connections_use_generation_fencing() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("index.db");
    let mut a = DiskIndex::open(&path, "n", DiskOptions::default()).unwrap();
    let mut b = DiskIndex::open(&path, "n", DiskOptions::default()).unwrap();
    let old = a.revision().unwrap();
    a.apply(&old, &[put("a", "one")], &DiskWork::default())
        .unwrap();
    assert!(matches!(
        b.apply(&old, &[put("b", "two")], &DiskWork::default()),
        Err(DiskError::Conflict)
    ));
    assert_eq!(a.revision().unwrap(), b.revision().unwrap());
}
#[test]
fn seeded_batches_match_authoritative_membership_after_every_reopen() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("index.db");
    let mut db = DiskIndex::open(&path, "n", DiskOptions::default()).unwrap();
    let mut expected = std::collections::BTreeSet::new();
    let mut seed = 7u64;
    for step in 0..100 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let id = format!("id{}", seed % 30);
        let change = if seed & 4 == 0 {
            expected.remove(&id);
            StoreChange::Delete(id)
        } else {
            expected.insert(id.clone());
            put(&id, "shared term")
        };
        db.apply(&db.revision().unwrap(), &[change], &DiskWork::default())
            .unwrap();
        if step % 10 == 0 {
            drop(db);
            db = DiskIndex::open(&path, "n", DiskOptions::default()).unwrap();
            let actual = db
                .search("shared", &DiskSearchOptions::new(100).unwrap())
                .unwrap()
                .hits()
                .iter()
                .map(|h| h.entry.id.clone())
                .collect::<std::collections::BTreeSet<_>>();
            assert_eq!(expected, actual);
            db.integrity_check(&DiskWork::default()).unwrap();
        }
    }
}
#[test]
fn logical_fts_damage_is_detected() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("index.db");
    let mut db = DiskIndex::open(&path, "n", DiskOptions::default()).unwrap();
    db.apply(
        &db.revision().unwrap(),
        &[put("a", "Rust")],
        &DiskWork::default(),
    )
    .unwrap();
    let c = rusqlite::Connection::open(&path).unwrap();
    c.execute("UPDATE search SET terms='wrong'", []).unwrap();
    assert!(db.integrity_check(&DiskWork::default()).is_err());
}
#[cfg(feature = "execution")]
#[test]
fn cancelled_and_expired_operations_do_not_publish_changes() {
    use airust::execution::*;
    let folder = tempfile::tempdir().unwrap();
    let mut db =
        DiskIndex::open(folder.path().join("index.db"), "n", DiskOptions::default()).unwrap();
    let rev = db.revision().unwrap();
    let token = CancellationToken::new();
    token.cancel();
    let work = DiskWork::default().with_control(WorkControl::new(token));
    assert!(matches!(
        db.apply(&rev, &[put("a", "Rust")], &work),
        Err(DiskError::Execution(ExecutionError::Cancelled))
    ));
    assert_eq!(db.revision().unwrap(), rev);
    let c = WorkControl::default()
        .with_timeout(std::time::Duration::ZERO)
        .unwrap();
    let options = DiskSearchOptions::new(1).unwrap().with_control(c);
    assert!(matches!(
        db.search("Rust", &options),
        Err(DiskError::Execution(ExecutionError::DeadlineExceeded))
    ));
}

#[test]
fn crash_writer_child() {
    let Ok(path) = std::env::var("AIRUST_DISK_CRASH_PATH") else {
        return;
    };
    let c = rusqlite::Connection::open(path).unwrap();
    c.execute_batch("PRAGMA cache_size=1; PRAGMA cache_spill=ON; BEGIN IMMEDIATE")
        .unwrap();
    for i in 0..500 {
        let id = format!("uncommitted-{i}");
        let entry = KnowledgeEntry::new(&id, format!("unfinished Rust {}", "padding ".repeat(100)));
        let body = serde_json::to_string(&entry).unwrap();
        c.execute(
            "INSERT INTO records(id,body)VALUES(?1,?2)",
            rusqlite::params![id, body],
        )
        .unwrap();
        let rowid = c.last_insert_rowid();
        c.execute(
            "INSERT INTO search(rowid,terms)VALUES(?1,?2)",
            rusqlite::params![rowid, airust::text::normalize(&entry.text)],
        )
        .unwrap();
    }
    std::process::exit(91);
}
#[test]
fn abrupt_writer_exit_recovers_without_exposing_uncommitted_rows() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("index.db");
    let mut db = DiskIndex::open(&path, "n", DiskOptions::default()).unwrap();
    let revision = db
        .apply(
            &db.revision().unwrap(),
            &[put("committed", "original Rust")],
            &DiskWork::default(),
        )
        .unwrap();
    drop(db);
    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "crash_writer_child", "--nocapture"])
        .env("AIRUST_DISK_CRASH_PATH", &path)
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(91));
    assert!(
        std::fs::metadata(path.with_extension("db-journal"))
            .unwrap()
            .len()
            > 512
    );
    let mut db = DiskIndex::open(&path, "n", DiskOptions::default()).unwrap();
    assert_eq!(db.revision().unwrap(), revision);
    assert_eq!(db.len().unwrap(), 1);
    assert!(db
        .search("unfinished", &DiskSearchOptions::new(10).unwrap())
        .unwrap()
        .hits()
        .is_empty());
    db.integrity_check(&DiskWork::default()).unwrap();
}

#[test]
fn bounded_export_pages_preserve_records_and_reject_mixed_generations() {
    let folder = tempfile::tempdir().unwrap();
    let mut db =
        DiskIndex::open(folder.path().join("index.db"), "n", DiskOptions::default()).unwrap();
    let changes = (0..7)
        .rev()
        .map(|i| put(&format!("id{i}"), "original Häuser"))
        .collect::<Vec<_>>();
    let revision = db
        .apply(&db.revision().unwrap(), &changes, &DiskWork::default())
        .unwrap();
    let mut cursor = None;
    let mut ids = Vec::new();
    loop {
        let page = db.page(&revision, cursor.as_deref(), 2).unwrap();
        assert!(page.entries.len() <= 2);
        ids.extend(page.entries.iter().map(|e| e.id.clone()));
        cursor = page.next_after;
        if cursor.is_none() {
            break;
        }
    }
    assert_eq!(ids, (0..7).map(|i| format!("id{i}")).collect::<Vec<_>>());
    db.apply(&revision, &[put("new", "changed")], &DiskWork::default())
        .unwrap();
    assert!(matches!(
        db.page(&revision, None, 2),
        Err(DiskError::Conflict)
    ));
}
