#![cfg(feature = "storage")]
use airust::{
    storage::{
        read_snapshot, MemoryStore, SnapshotOptions, StorageBackend, StorageError, StorageLimits,
        StoragePage, StoreChange, StoreRevision,
    },
    KnowledgeEntry,
};
use serde_json::json;
fn stores(limits: StorageLimits) -> Vec<Box<dyn StorageBackend>> {
    let list: Vec<Box<dyn StorageBackend>> = vec![Box::new(
        MemoryStore::with_limits("test", limits.clone()).unwrap(),
    )];
    #[cfg(feature = "sqlite-storage")]
    let list = {
        let mut list = list;
        list.push(Box::new(
            airust::storage::sqlite::SqliteStore::in_memory_with_limits("test", limits).unwrap(),
        ));
        list
    };
    list
}
fn put(id: &str, text: &str) -> StoreChange {
    StoreChange::Put(KnowledgeEntry::new(id, text))
}
#[test]
fn adapters_share_atomic_crud_revision_and_identity_contracts() {
    for mut b in stores(StorageLimits::default()) {
        let initial = b.revision().unwrap();
        assert_eq!(initial.generation(), 0);
        assert_eq!(b.apply(&initial, &[]).unwrap().revision, initial);
        assert!(b.get(&initial, "absent").unwrap().is_none());
        let written = b
            .apply(
                &initial,
                &[
                    put("β", "Rust memory"),
                    put("a", "English"),
                    put("東京", "日本語"),
                ],
            )
            .unwrap();
        assert_eq!(written.upserts, 3);
        assert_eq!(written.revision.generation(), 1);
        assert!(matches!(b.get(&initial, "a"), Err(StorageError::Conflict)));
        assert!(matches!(
            b.apply(&initial, &[StoreChange::Delete("a".into())]),
            Err(StorageError::Conflict)
        ));
        let page = b.page(&written.revision, None, 2).unwrap();
        assert_eq!(
            page.entries
                .iter()
                .map(|e| e.id.as_str())
                .collect::<Vec<_>>(),
            ["a", "β"]
        );
        assert_eq!(page.next_after.as_deref(), Some("β"));
        let tail = b
            .page(&written.revision, page.next_after.as_deref(), 2)
            .unwrap();
        assert_eq!(tail.entries[0].id, "東京");
        assert!(tail.next_after.is_none());
        let changed = b
            .apply(
                &written.revision,
                &[
                    put("a", "changed"),
                    StoreChange::Delete("β".into()),
                    StoreChange::Delete("missing".into()),
                ],
            )
            .unwrap();
        assert_eq!(changed.deleted, 1);
        assert_eq!(
            b.get(&changed.revision, "a").unwrap().unwrap().text,
            "changed"
        );
        let snapshot =
            read_snapshot(&mut *b, &SnapshotOptions::new(1, 10, 10000).unwrap()).unwrap();
        assert_eq!(snapshot.entries().len(), 2);
        let loaded = snapshot.into_index().unwrap();
        assert_eq!(loaded.index().search("changed", 1)[0].entry.id, "a");
        assert_eq!(loaded.revision(), &changed.revision);
        let fresh = b.renew_identity(&changed.revision).unwrap();
        assert_ne!(fresh.identity(), changed.revision.identity());
        assert_eq!(fresh.generation(), 0);
        assert!(matches!(
            b.page(&changed.revision, None, 1),
            Err(StorageError::Conflict)
        ));
        assert_eq!(b.get(&fresh, "a").unwrap().unwrap().text, "changed");
    }
}
#[test]
fn invalid_later_operations_duplicate_ids_and_limits_never_partially_commit() {
    for mut b in stores(
        StorageLimits::default()
            .with_entry_limits(8, 150)
            .unwrap()
            .with_batch_limits(3, 300)
            .unwrap(),
    ) {
        let rev = b.revision().unwrap();
        for changes in [
            vec![put("a", "valid"), put("b", "")],
            vec![put("a", "valid"), put("a", "duplicate")],
            vec![put("toolongid", "valid")],
            vec![put("a", &"x".repeat(151))],
            vec![put("1", "v"), put("2", "v"), put("3", "v"), put("4", "v")],
        ] {
            assert!(b.apply(&rev, &changes).is_err());
            assert_eq!(b.revision().unwrap(), rev);
            assert!(b.page(&rev, None, 10).unwrap().entries.is_empty());
        }
    }
}
#[test]
fn metadata_depth_node_count_and_encoded_escaping_are_bounded() {
    for mut b in stores(
        StorageLimits::default()
            .with_entry_limits(50, 120)
            .unwrap()
            .with_metadata_limits(3, 5)
            .unwrap(),
    ) {
        let rev = b.revision().unwrap();
        for value in [
            json!([[[[1]]]]),
            json!([1, 2, 3, 4, 5, 6]),
            json!({"key":"x".repeat(150)}),
        ] {
            let change = StoreChange::Put(KnowledgeEntry::new("a", "v").with_metadata(value));
            assert!(matches!(
                b.apply(&rev, &[change]),
                Err(StorageError::Limit(_))
            ));
            assert_eq!(b.revision().unwrap(), rev);
        }
        assert!(b.apply(&rev, &[put("a", &"\n".repeat(60))]).is_err());
        assert_eq!(b.revision().unwrap(), rev);
    }
}
#[test]
fn page_byte_budgets_never_skip_a_deferred_record() {
    for mut b in stores(StorageLimits::default().with_page_limits(10, 60).unwrap()) {
        let rev = b.revision().unwrap();
        let rev = b
            .apply(
                &rev,
                &[put("a", "x"), put("b", &"y".repeat(30)), put("c", "z")],
            )
            .unwrap()
            .revision;
        let first = b.page(&rev, None, 10).unwrap();
        assert_eq!(first.entries.len(), 1);
        assert_eq!(first.next_after.as_deref(), Some("a"));
        let second = b.page(&rev, first.next_after.as_deref(), 10).unwrap();
        assert_eq!(second.entries[0].id, "b");
        let all = read_snapshot(&mut *b, &SnapshotOptions::new(10, 10, 10000).unwrap()).unwrap();
        assert_eq!(
            all.entries()
                .iter()
                .map(|e| e.id.as_str())
                .collect::<Vec<_>>(),
            ["a", "b", "c"]
        );
    }
}
#[test]
fn single_record_too_large_for_a_page_and_snapshot_limits_are_errors() {
    for mut b in stores(StorageLimits::default().with_page_limits(10, 10).unwrap()) {
        let rev = b.revision().unwrap();
        let rev = b.apply(&rev, &[put("a", "text")]).unwrap().revision;
        assert!(matches!(b.page(&rev, None, 1), Err(StorageError::Limit(_))));
    }
    for mut b in stores(StorageLimits::default()) {
        let rev = b.revision().unwrap();
        b.apply(&rev, &[put("a", "text"), put("b", "text")])
            .unwrap();
        for opt in [
            SnapshotOptions::new(1, 1, 1000).unwrap(),
            SnapshotOptions::new(1, 10, 10).unwrap(),
        ] {
            assert!(matches!(
                read_snapshot(&mut *b, &opt),
                Err(StorageError::Limit(_))
            ));
        }
    }
}
#[test]
fn revision_tokens_reject_invalid_or_cross_store_values() {
    for raw in [
        json!({"namespace":"","identity":"a".repeat(32),"generation":0}),
        json!({"namespace":"test","identity":"bad","generation":0}),
        json!({"namespace":"test","identity":"a".repeat(32),"generation":u64::MAX}),
    ] {
        assert!(serde_json::from_value::<StoreRevision>(raw).is_err());
    }
    let mut a = MemoryStore::new("test").unwrap();
    let mut b = MemoryStore::new("test").unwrap();
    assert!(matches!(
        b.apply(&a.revision().unwrap(), &[]),
        Err(StorageError::Conflict)
    ));
}
#[test]
fn invalid_configuration_and_cursors_are_rejected() {
    assert!(MemoryStore::new("").is_err());
    assert!(MemoryStore::new("bad\nname").is_err());
    assert!(StorageLimits::default().with_entry_limits(0, 1).is_err());
    assert!(StorageLimits::default().with_batch_limits(0, 1).is_err());
    assert!(StorageLimits::default().with_page_limits(1, 0).is_err());
    assert!(StorageLimits::default()
        .with_metadata_limits(65, 1)
        .is_err());
    assert!(SnapshotOptions::new(0, 1, 1).is_err());
    for mut b in stores(StorageLimits::default()) {
        let rev = b.revision().unwrap();
        assert!(b.page(&rev, None, 0).is_err());
        assert!(b.page(&rev, Some(""), 1).is_err());
        assert!(b.page(&rev, None, 1001).is_err());
    }
}
#[test]
fn randomized_commits_match_an_independent_map_and_paged_snapshot() {
    use std::collections::BTreeMap;
    for mut b in stores(StorageLimits::default()) {
        let mut seed = 89171u64;
        let mut reference = BTreeMap::new();
        let mut rev = b.revision().unwrap();
        for step in 0..400 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let id = format!("key-{}", (seed >> 32) % 31);
            let change = if seed & 3 == 0 {
                reference.remove(&id);
                StoreChange::Delete(id)
            } else {
                let text = format!("text-{step}");
                reference.insert(id.clone(), text.clone());
                put(&id, &text)
            };
            rev = b.apply(&rev, &[change]).unwrap().revision;
            if step % 17 == 0 {
                let snap =
                    read_snapshot(&mut *b, &SnapshotOptions::new(3, 100, 100000).unwrap()).unwrap();
                assert_eq!(
                    snap.entries()
                        .iter()
                        .map(|e| (e.id.clone(), e.text.clone()))
                        .collect::<BTreeMap<_, _>>(),
                    reference
                );
                assert_eq!(snap.revision(), &rev);
            }
        }
    }
}

struct BadPages {
    base: MemoryStore,
    mode: u8,
    calls: usize,
}
impl StorageBackend for BadPages {
    fn limits(&self) -> &StorageLimits {
        self.base.limits()
    }
    fn revision(&mut self) -> Result<StoreRevision, StorageError> {
        self.base.revision()
    }
    fn get(&mut self, r: &StoreRevision, id: &str) -> Result<Option<KnowledgeEntry>, StorageError> {
        self.base.get(r, id)
    }
    fn apply(
        &mut self,
        r: &StoreRevision,
        c: &[StoreChange],
    ) -> Result<airust::storage::StoreCommit, StorageError> {
        self.base.apply(r, c)
    }
    fn page(
        &mut self,
        r: &StoreRevision,
        after: Option<&str>,
        limit: usize,
    ) -> Result<StoragePage, StorageError> {
        self.calls += 1;
        let mut page = self.base.page(r, after, limit)?;
        match self.mode {
            1 => page.next_after = Some("not-last-entry".into()),
            2 => page.entries.reverse(),
            3 => {
                let now = self.base.revision()?;
                self.base.apply(&now, &[put("z", "changed")])?;
            }
            4 => page.entries.push(KnowledgeEntry::new("z", "extra")),
            _ => {}
        }
        Ok(page)
    }
}
#[test]
fn snapshot_loading_rejects_malformed_pages_and_writes_during_the_read() {
    for mode in 1..=4 {
        let mut base = MemoryStore::new("test").unwrap();
        let rev = base.revision().unwrap();
        base.apply(&rev, &[put("a", "one"), put("b", "two"), put("c", "three")])
            .unwrap();
        let mut backend = BadPages {
            base,
            mode,
            calls: 0,
        };
        assert!(
            read_snapshot(&mut backend, &SnapshotOptions::new(2, 100, 10000).unwrap()).is_err()
        );
    }
}

#[cfg(feature = "sqlite-storage")]
mod sqlite_tests {
    use super::*;
    use airust::storage::sqlite::SqliteStore;
    use rusqlite::Connection;
    #[test]
    fn durable_namespaces_reopen_read_only_and_stale_connections_are_checked() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("store.db");
        let mut a = SqliteStore::open(&path, "tenant-a").unwrap();
        assert!(a.is_durable());
        let first = a.revision().unwrap();
        let written = a
            .apply(&first, &[put("id", "private to namespace a")])
            .unwrap()
            .revision;
        let mut other = SqliteStore::open(&path, "tenant-b").unwrap();
        let r = other.revision().unwrap();
        assert!(other.get(&r, "id").unwrap().is_none());
        assert!(matches!(
            other.get(&written, "id"),
            Err(StorageError::Conflict)
        ));
        drop(a);
        let mut reopened = SqliteStore::open(&path, "tenant-a").unwrap();
        assert_eq!(reopened.revision().unwrap(), written);
        let mut read = SqliteStore::open_read_only(&path, "tenant-a").unwrap();
        assert!(read.is_read_only());
        assert_eq!(
            read.get(&written, "id").unwrap().unwrap().text,
            "private to namespace a"
        );
        assert!(matches!(
            read.apply(&written, &[put("x", "v")]),
            Err(StorageError::ReadOnly)
        ));
        assert!(matches!(
            read.renew_identity(&written),
            Err(StorageError::ReadOnly)
        ));
        let now = reopened
            .apply(&written, &[put("id", "new")])
            .unwrap()
            .revision;
        assert!(matches!(
            read.get(&written, "id"),
            Err(StorageError::Conflict)
        ));
        assert_eq!(read.get(&now, "id").unwrap().unwrap().text, "new");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(path).unwrap().permissions().mode() & 0o077,
                0
            );
        }
    }
    #[test]
    fn an_sql_failure_in_a_later_operation_rolls_back_earlier_rows_and_revision() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("store.db");
        let mut store = SqliteStore::open(&path, "test").unwrap();
        let rev = store.revision().unwrap();
        let raw = Connection::open(&path).unwrap();
        raw.execute_batch("CREATE TRIGGER fail_late BEFORE INSERT ON airust_store_entries WHEN NEW.id='b' BEGIN SELECT RAISE(ABORT,'synthetic failure');END;").unwrap();
        assert!(store
            .apply(&rev, &[put("a", "first"), put("b", "fail")])
            .is_err());
        assert_eq!(store.revision().unwrap(), rev);
        assert!(store.get(&rev, "a").unwrap().is_none());
    }
    #[test]
    fn unrelated_databases_newer_formats_and_bad_rows_are_never_silently_accepted() {
        let dir = tempfile::tempdir().unwrap();
        let unrelated = dir.path().join("unrelated.db");
        let raw = Connection::open(&unrelated).unwrap();
        raw.execute_batch(
            "CREATE TABLE user_data(value TEXT);INSERT INTO user_data VALUES('keep');",
        )
        .unwrap();
        assert!(SqliteStore::open(&unrelated, "test").is_err());
        assert_eq!(
            raw.query_row("SELECT value FROM user_data", [], |r| r.get::<_, String>(0))
                .unwrap(),
            "keep"
        );
        let path = dir.path().join("store.db");
        let mut store = SqliteStore::open(&path, "test").unwrap();
        let rev = store.revision().unwrap();
        let raw = Connection::open(&path).unwrap();
        raw.execute(
            "INSERT INTO airust_store_entries VALUES('test','a','not json')",
            [],
        )
        .unwrap();
        assert!(store.get(&rev, "a").is_err());
        raw.execute(
            "UPDATE airust_store_entries SET record=?1",
            [r#"{"id":"wrong","text":"value"}"#],
        )
        .unwrap();
        assert!(store.page(&rev, None, 10).is_err());
        raw.execute("UPDATE airust_store_format SET version=2", [])
            .unwrap();
        assert!(SqliteStore::open(&path, "test").is_err());
    }
    #[test]
    fn concurrent_same_revision_writers_have_one_winner() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("store.db");
        let mut a = SqliteStore::open(&path, "test").unwrap();
        let mut b = SqliteStore::open(&path, "test").unwrap();
        let revision = a.revision().unwrap();
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let r = revision.clone();
        let gate = barrier.clone();
        let thread = std::thread::spawn(move || {
            gate.wait();
            a.apply(&r, &[put("a", "one")])
        });
        barrier.wait();
        let second = b.apply(&revision, &[put("b", "two")]);
        let first = thread.join().unwrap();
        assert_eq!(usize::from(first.is_ok()) + usize::from(second.is_ok()), 1);
        let error = first.err().or_else(|| second.err()).unwrap();
        assert!(matches!(error, StorageError::Conflict));
    }
    #[test]
    fn exhausted_generations_never_mutate_and_identity_can_be_renewed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("store.db");
        let mut store = SqliteStore::open(&path, "test").unwrap();
        let raw = Connection::open(&path).unwrap();
        raw.execute("UPDATE airust_store_state SET generation=?1", [i64::MAX])
            .unwrap();
        let rev = store.revision().unwrap();
        assert!(matches!(
            store.apply(&rev, &[put("a", "no")]),
            Err(StorageError::GenerationExhausted)
        ));
        assert!(store.get(&rev, "a").unwrap().is_none());
        let next = store.renew_identity(&rev).unwrap();
        assert_eq!(next.generation(), 0);
        assert_ne!(next.identity(), rev.identity());
    }
    #[cfg(unix)]
    #[test]
    fn symlink_paths_are_rejected_without_modifying_the_target() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("original");
        std::fs::write(&target, b"keep").unwrap();
        let link = dir.path().join("link");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        assert!(SqliteStore::open(&link, "test").is_err());
        assert_eq!(std::fs::read(target).unwrap(), b"keep");
    }
}
