#![cfg(feature = "versioning")]

use airust::{
    versioning::{
        ChangeMetadata, EntryVersion, ExpectedRevision as Expected, Revision, RevisionAction,
        RevisionRecord, VersionError, VersionedIndex,
    },
    Engine, KnowledgeEntry,
};
use serde_json::{json, Value};

fn change(reason: &str) -> ChangeMetadata {
    ChangeMetadata::default()
        .with_actor("tester")
        .unwrap()
        .with_reason(reason)
        .unwrap()
}
fn entry(text: &str) -> KnowledgeEntry {
    KnowledgeEntry::new("a", text)
}

#[test]
fn default_revision_tracking_retains_no_text_history_and_preserves_source_json() {
    let mut index = VersionedIndex::new();
    let first = index
        .upsert(entry("Rust Grüße 世界"), Expected::Absent, change("create"))
        .unwrap();
    let second = index
        .upsert(
            entry("Rust updated"),
            Expected::Exact(first),
            change("update"),
        )
        .unwrap();
    assert_eq!(first.get(), 1);
    assert_eq!(second.get(), 2);
    assert_eq!(index.history_len(), 0);
    assert!(matches!(
        index.at("a", first),
        Err(VersionError::Unavailable { .. })
    ));
    assert_eq!(
        index.version("a").unwrap().action(),
        RevisionAction::Replace
    );
    assert_eq!(
        index.version("a").unwrap().change().reason(),
        Some("update")
    );
    assert_eq!(index.index().search("updated", 1)[0].entry.id, "a");
    assert_eq!(
        index.index().to_json().unwrap(),
        Engine::from_entries([entry("Rust updated")])
            .unwrap()
            .to_json()
            .unwrap()
    );
    assert!(VersionedIndex::from_json(&index.index().to_json().unwrap()).is_err());
}
#[test]
fn importing_an_engine_moves_original_text_and_shares_bulk_change_metadata() {
    let engine = Engine::from_entries([entry("Rust"), KnowledgeEntry::new("b", "Python")]).unwrap();
    let pointer = engine.get("a").unwrap().text.as_ptr();
    let index = VersionedIndex::from_index(engine, change("import")).unwrap();
    assert_eq!(index.index().get("a").unwrap().text.as_ptr(), pointer);
    assert!(std::ptr::eq(
        index.version("a").unwrap().change(),
        index.version("b").unwrap().change()
    ));
    assert_eq!(index.revision("a").unwrap().get(), 1);
    assert_eq!(index.revision("b").unwrap().get(), 2);
    assert_eq!(index.history_len(), 0);
}
#[test]
fn stale_writes_and_invalid_inputs_leave_every_observable_state_unchanged() {
    let mut index = VersionedIndex::new().with_history_limit(10);
    let revision = index
        .upsert(entry("original"), Expected::Absent, change("create"))
        .unwrap();
    let before = index.to_json().unwrap();
    assert!(matches!(
        index.upsert(entry("wrong"), Expected::Absent, change("stale")),
        Err(VersionError::Conflict { .. })
    ));
    assert!(index
        .upsert(entry(" "), Expected::Exact(revision), change("invalid"))
        .is_err());
    assert!(index
        .delete("missing", Expected::Absent, change("delete"))
        .is_err());
    assert!(index
        .restore(
            "a",
            Revision::new(99).unwrap(),
            Expected::Exact(revision),
            change("restore")
        )
        .is_err());
    assert_eq!(index.to_json().unwrap(), before);
}
#[test]
fn deleting_and_recreating_ids_preserves_tombstone_conflict_checks() {
    let mut index = VersionedIndex::new().with_history_limit(10);
    let first = index
        .upsert(entry("before"), Expected::Absent, change("create"))
        .unwrap();
    let deleted = index
        .delete("a", Expected::Exact(first), change("delete"))
        .unwrap();
    assert!(index.index().get("a").is_none());
    assert!(index.version("a").unwrap().is_deleted());
    let before = index.to_json().unwrap();
    assert!(matches!(
        index.upsert(entry("stale"), Expected::Exact(first), change("stale")),
        Err(VersionError::Conflict { .. })
    ));
    assert!(matches!(
        index.upsert(entry("absent"), Expected::Absent, change("stale")),
        Err(VersionError::Conflict { .. })
    ));
    assert!(matches!(
        index.delete("a", Expected::Exact(deleted), change("again")),
        Err(VersionError::AlreadyDeleted(_))
    ));
    assert_eq!(index.to_json().unwrap(), before);
    let recreated = index
        .upsert(entry("new"), Expected::Exact(deleted), change("recreate"))
        .unwrap();
    assert!(recreated > deleted);
    assert_eq!(index.version("a").unwrap().action(), RevisionAction::Insert);
    assert_eq!(index.tracked_ids(), 1);
}
#[test]
fn restores_create_new_revisions_for_live_and_deleted_states_with_distinct_metadata() {
    let mut index = VersionedIndex::new().with_history_limit(10);
    let first = index
        .upsert(entry("Grüße 世界"), Expected::Absent, change("create"))
        .unwrap();
    let second = index
        .upsert(entry("changed"), Expected::Exact(first), change("replace"))
        .unwrap();
    let restored = index
        .restore("a", first, Expected::Exact(second), change("restore first"))
        .unwrap();
    assert!(restored > second);
    assert_eq!(index.index().get("a").unwrap().text, "Grüße 世界");
    assert_eq!(
        index.version("a").unwrap().action(),
        RevisionAction::Restore
    );
    assert_eq!(
        index.at("a", first).unwrap().version().change().reason(),
        Some("create")
    );
    assert_eq!(
        index.version("a").unwrap().change().reason(),
        Some("restore first")
    );
    let deleted = index
        .delete("a", Expected::Exact(restored), change("delete"))
        .unwrap();
    let live = index
        .restore("a", first, Expected::Exact(deleted), change("restore live"))
        .unwrap();
    let tombstone = index
        .restore(
            "a",
            deleted,
            Expected::Exact(live),
            change("restore deletion"),
        )
        .unwrap();
    assert!(tombstone > live);
    assert!(index.at("a", tombstone).unwrap().entry().is_none());
    assert_eq!(
        index.version("a").unwrap().action(),
        RevisionAction::Restore
    );
}
#[test]
fn global_history_quota_prunes_oldest_commits_and_never_invents_unrecorded_history() {
    let mut index = VersionedIndex::new();
    let first = index
        .upsert(entry("unrecorded"), Expected::Absent, change("create"))
        .unwrap();
    let mut index = index.with_history_limit(2);
    let second = index
        .upsert(entry("recorded"), Expected::Exact(first), change("update"))
        .unwrap();
    let other = index
        .upsert(
            KnowledgeEntry::new("b", "other"),
            Expected::Absent,
            change("other"),
        )
        .unwrap();
    let latest = index
        .upsert(entry("latest"), Expected::Exact(second), change("update"))
        .unwrap();
    assert_eq!(
        index
            .history()
            .map(|r| r.version().revision())
            .collect::<Vec<_>>(),
        [other, latest]
    );
    assert!(index.at("a", first).is_err());
    assert!(index.at("a", second).is_err());
    assert_eq!(index.history_for("a").count(), 1);
    let index = index.with_history_limit(0);
    assert_eq!(index.history_len(), 0);
    assert!(index.at("a", latest).is_ok());
}
#[test]
fn comparisons_distinguish_text_metadata_and_entry_existence() {
    let mut index = VersionedIndex::new().with_history_limit(10);
    let a = index
        .upsert(entry("same"), Expected::Absent, change("a"))
        .unwrap();
    let b = index
        .upsert(
            entry("same").with_metadata(json!({"tag":"b"})),
            Expected::Exact(a),
            change("b"),
        )
        .unwrap();
    let c = index
        .upsert(
            entry("different").with_metadata(json!({"tag":"b"})),
            Expected::Exact(b),
            change("c"),
        )
        .unwrap();
    let d = index.delete("a", Expected::Exact(c), change("d")).unwrap();
    let ab = index.compare("a", a, b).unwrap();
    assert!(!ab.text_changed() && ab.metadata_changed() && !ab.existence_changed());
    let bc = index.compare("a", b, c).unwrap();
    assert!(bc.text_changed() && !bc.metadata_changed());
    let cd = index.compare("a", c, d).unwrap();
    assert!(cd.existence_changed() && cd.text_changed() && cd.metadata_changed());
    let aa = index.compare("a", a, a).unwrap();
    assert!(!aa.text_changed() && !aa.metadata_changed() && !aa.existence_changed());
    assert_eq!(ab.id(), "a");
    assert_eq!(ab.from(), a);
    assert_eq!(ab.to(), b);
}
#[test]
fn two_concurrent_callers_with_the_same_expected_revision_have_one_winner() {
    use std::sync::{Arc, Barrier, Mutex};
    let mut index = VersionedIndex::new();
    let revision = index
        .upsert(entry("start"), Expected::Absent, change("start"))
        .unwrap();
    let index = Arc::new(Mutex::new(index));
    let barrier = Arc::new(Barrier::new(2));
    let threads: Vec<_> = (0..2)
        .map(|n| {
            let index = Arc::clone(&index);
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                index.lock().unwrap().upsert(
                    entry(&format!("writer {n}")),
                    Expected::Exact(revision),
                    change("concurrent"),
                )
            })
        })
        .collect();
    let results: Vec<_> = threads.into_iter().map(|t| t.join().unwrap()).collect();
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|r| matches!(r, Err(VersionError::Conflict { .. })))
            .count(),
        1
    );
    assert_eq!(index.lock().unwrap().sequence(), 2);
}
#[test]
fn repeated_changes_serialization_and_restores_match_an_independent_current_state_map() {
    use std::collections::BTreeMap;
    let mut index = VersionedIndex::new().with_history_limit(19);
    let mut oracle = BTreeMap::new();
    for step in 0..300 {
        let id = format!("entry-{}", step * 17 % 37);
        if step % 5 == 0 {
            if oracle.remove(&id).is_some() {
                index
                    .delete(
                        &id,
                        Expected::Exact(index.revision(&id).unwrap()),
                        change("delete"),
                    )
                    .unwrap();
            }
        } else {
            let expected = index
                .revision(&id)
                .map_or(Expected::Absent, Expected::Exact);
            let entry =
                KnowledgeEntry::new(&id, format!("Rust Grüße bucket{} revision{step}", step % 7))
                    .with_metadata(json!({"step":step}));
            index
                .upsert(entry.clone(), expected, change("update"))
                .unwrap();
            oracle.insert(id, entry);
        }
        assert!(index.history_len() <= 19);
        if step % 11 == 0 {
            let encoded = index.to_json().unwrap();
            index = VersionedIndex::from_json(&encoded).unwrap();
            assert_eq!(index.to_json().unwrap(), encoded);
            let fresh = Engine::from_entries(oracle.values().cloned()).unwrap();
            assert_eq!(fresh.to_json().unwrap(), index.index().to_json().unwrap());
            for query in ["Rust", "Grüße", "bucket3", "missing"] {
                assert_eq!(
                    serde_json::to_value(fresh.search(query, 10)).unwrap(),
                    serde_json::to_value(index.index().search(query, 10)).unwrap()
                );
            }
        }
    }
}
#[test]
fn source_ownership_history_and_tombstones_survive_versioned_json_and_generic_serde() {
    let mut index = VersionedIndex::new().with_history_limit(10);
    let first = index
        .upsert(entry("original"), Expected::Absent, change("create"))
        .unwrap();
    index
        .delete("a", Expected::Exact(first), change("delete"))
        .unwrap();
    let encoded = index.to_json().unwrap();
    for restored in [
        VersionedIndex::from_json(&encoded).unwrap(),
        serde_json::from_str::<VersionedIndex>(&encoded).unwrap(),
    ] {
        assert_eq!(restored.to_json().unwrap(), encoded);
        assert!(restored.version("a").unwrap().is_deleted());
        assert_eq!(
            restored.at("a", first).unwrap().entry().unwrap().text,
            "original"
        );
    }
    assert!(matches!(
        VersionedIndex::from_json_with_limit(&encoded, encoded.len() - 1),
        Err(VersionError::InputLimit)
    ));
    assert!(VersionedIndex::from_json_with_limit(&encoded, encoded.len()).is_ok());
}
#[test]
fn malformed_snapshots_and_standalone_records_cannot_bypass_invariants() {
    let mut index = VersionedIndex::new().with_history_limit(10);
    let first = index
        .upsert(entry("one"), Expected::Absent, change("first"))
        .unwrap();
    index
        .upsert(entry("two"), Expected::Exact(first), change("second"))
        .unwrap();
    let base: Value = serde_json::from_str(&index.to_json().unwrap()).unwrap();
    let mutations: &[fn(&mut Value)] = &[
        |v| v["format_version"] = json!(99),
        |v| v["sequence"] = json!(0),
        |v| v["history_limit"] = json!(0),
        |v| v["extra"] = json!(true),
        |v| v["versions"][0]["version"]["revision"] = json!(0),
        |v| v["versions"][0]["version"]["deleted"] = json!(true),
        |v| v["versions"][0]["version"]["action"] = json!("Delete"),
        |v| v["versions"][0]["version"]["change"]["actor"] = json!("bad\nlabel"),
        |v| {
            let head = v["versions"][0].clone();
            v["versions"].as_array_mut().unwrap().push(head);
        },
        |v| v["versions"][0]["id"] = json!("unknown"),
        |v| v["entries"] = json!([]),
        |v| v["journal"][1]["entry"]["text"] = json!("inconsistent latest"),
        |v| v["journal"][0]["id"] = json!("unknown"),
        |v| v["journal"][0]["entry"]["id"] = json!("wrong"),
        |v| v["journal"][0]["entry"]["text"] = json!(" "),
        |v| v["journal"][0]["version"]["revision"] = json!(3),
        |v| v["journal"].as_array_mut().unwrap().reverse(),
        |v| v["versions"][0]["extra"] = json!(true),
    ];
    for mutate in mutations {
        let mut value = base.clone();
        mutate(&mut value);
        let encoded = value.to_string();
        assert!(
            VersionedIndex::from_json(&encoded).is_err(),
            "accepted {encoded}"
        );
        assert!(serde_json::from_str::<VersionedIndex>(&encoded).is_err());
    }
    let mut version = base["versions"][0]["version"].clone();
    version["deleted"] = json!(true);
    assert!(serde_json::from_value::<EntryVersion>(version).is_err());
    let mut record = base["journal"][0].clone();
    record["entry"] = Value::Null;
    assert!(serde_json::from_value::<RevisionRecord>(record).is_err());
}
#[test]
fn revision_exhaustion_is_checked_before_writes_and_numeric_boundaries_are_explicit() {
    assert!(Revision::new(0).is_err());
    assert!(serde_json::from_str::<Revision>("0").is_err());
    assert!(serde_json::from_str::<Revision>("-1").is_err());
    assert!(serde_json::from_str::<Revision>("18446744073709551616").is_err());
    let mut index = VersionedIndex::new();
    index
        .upsert(entry("last"), Expected::Absent, change("create"))
        .unwrap();
    let mut wire: Value = serde_json::from_str(&index.to_json().unwrap()).unwrap();
    wire["sequence"] = json!(u64::MAX);
    wire["versions"][0]["version"]["revision"] = json!(u64::MAX);
    let mut index = VersionedIndex::from_json(&wire.to_string()).unwrap();
    let before = index.to_json().unwrap();
    assert!(matches!(
        index.upsert(
            entry("overflow"),
            Expected::Exact(Revision::new(u64::MAX).unwrap()),
            change("overflow")
        ),
        Err(VersionError::Exhausted)
    ));
    assert_eq!(index.to_json().unwrap(), before);
}
#[test]
fn change_labels_validate_unicode_controls_limits_and_optional_metadata() {
    for label in [
        "".to_owned(),
        " ".to_owned(),
        "bad\nactor".to_owned(),
        "a".repeat(1025),
    ] {
        assert!(ChangeMetadata::default().with_actor(&label).is_err());
        assert!(ChangeMetadata::default().with_reason(&label).is_err());
    }
    let metadata = ChangeMetadata::default()
        .with_actor(" Grüße ")
        .unwrap()
        .with_reason("ä".repeat(1024))
        .unwrap()
        .with_metadata(json!({"custom":true}));
    assert_eq!(metadata.actor(), Some("Grüße"));
    assert_eq!(metadata.metadata(), Some(&json!({"custom":true})));
    assert_eq!(
        serde_json::from_str::<ChangeMetadata>(&serde_json::to_string(&metadata).unwrap()).unwrap(),
        metadata
    );
}
