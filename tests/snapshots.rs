#![cfg(feature = "snapshots")]
use airust::{
    provenance::{ProvenanceStatus, SourceReference},
    semantic::{
        Embedding, EmbeddingError, EmbeddingProvider, EmbeddingPurpose, EmbeddingSpace,
        SemanticSearchOptions,
    },
    snapshots::*,
    storage::{StorageLimits, StoreRevision},
    KnowledgeEntry,
};
use serde_json::json;
use std::{
    collections::BTreeMap,
    sync::{mpsc, Arc, Barrier},
    thread,
    time::Duration,
};
fn space() -> EmbeddingSpace {
    EmbeddingSpace::new("handwritten-consistency-v1", 2).unwrap()
}
fn raw(text: &str) -> Vec<f32> {
    vec![
        1.0,
        text.split(':')
            .next_back()
            .and_then(|n| n.parse::<f32>().ok())
            .unwrap_or(1.0)
            + 1.0,
    ]
}
struct Provider {
    space: EmbeddingSpace,
    calls: usize,
    seen: Vec<String>,
    fail_at: usize,
    bad: bool,
    bad_count: bool,
    change_space: bool,
}
impl Provider {
    fn new() -> Self {
        Self {
            space: space(),
            calls: 0,
            seen: Vec::new(),
            fail_at: usize::MAX,
            bad: false,
            bad_count: false,
            change_space: false,
        }
    }
}
impl EmbeddingProvider for Provider {
    fn space(&self) -> &EmbeddingSpace {
        &self.space
    }
    fn embed(
        &mut self,
        texts: &[&str],
        purpose: EmbeddingPurpose,
    ) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        assert_eq!(purpose, EmbeddingPurpose::Document);
        self.calls += 1;
        self.seen.extend(texts.iter().map(|s| s.to_string()));
        if self.calls == self.fail_at {
            return Err(EmbeddingError::Provider("injected failure".into()));
        }
        if self.change_space {
            self.space = EmbeddingSpace::new("changed-mid-call", 2).unwrap();
        }
        if self.bad_count {
            return Ok(Vec::new());
        }
        if self.bad {
            return Ok(texts.iter().map(|_| vec![f32::NAN, 1.0]).collect());
        }
        Ok(texts.iter().map(|s| raw(s)).collect())
    }
}
fn reference(id: &str, revision: &str) -> SourceReference {
    SourceReference::new(id)
        .unwrap()
        .with_revision(revision)
        .unwrap()
}
fn record(id: &str, value: usize) -> DatasetRecord {
    let text = format!("record:{value}");
    DatasetRecord::new(KnowledgeEntry::new(id, &text))
        .with_sources(vec![reference(&format!("source-{id}"), &text)])
}
fn change(id: &str, value: usize) -> DatasetChange {
    DatasetChange::Put(record(id, value))
}
fn manager() -> DatasetManager {
    DatasetManager::new(
        "test",
        DatasetConfig::default()
            .with_embedding_space(space())
            .unwrap(),
    )
    .unwrap()
}
fn seed(manager: &DatasetManager) -> Arc<DatasetSnapshot> {
    let base = manager.snapshot().unwrap();
    let prepared = manager
        .prepare(
            base.revision(),
            DatasetUpdate::new(vec![change("a", 1), change("b", 2)]),
            Some(&mut Provider::new()),
        )
        .unwrap();
    manager.publish(prepared).unwrap()
}
fn assert_record(view: &DatasetSnapshot, id: &str, value: usize) {
    let expected = format!("record:{value}");
    let entry = view.lexical().get(id).unwrap();
    assert_eq!(entry.text, expected);
    let refs: Vec<_> = view.provenance().sources_for(id).collect();
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].revision(), Some(expected.as_str()));
    if let Some(semantic) = view.semantic() {
        let expected = Embedding::new(space(), raw(&expected)).unwrap();
        assert_eq!(semantic.embedding(id).unwrap(), expected.values());
        assert_eq!(semantic.lexical().get(id).unwrap().text, entry.text);
    }
    #[cfg(feature = "advanced-search")]
    if let Some(text) = view.text_index() {
        assert_eq!(text.get(id).unwrap().text, entry.text);
    }
}
#[test]
fn all_views_publish_together_and_older_readers_keep_a_complete_snapshot() {
    let m = manager();
    let empty = m.snapshot().unwrap();
    assert_eq!(empty.revision().generation(), 0);
    let first = seed(&m);
    assert_eq!(first.revision().generation(), 1);
    assert_record(&first, "a", 1);
    assert_record(&first, "b", 2);
    assert!(empty.lexical().is_empty());
    assert!(empty.provenance().is_empty());
    let staged = m
        .prepare(
            first.revision(),
            DatasetUpdate::new(vec![change("a", 3), DatasetChange::Delete("b".into())]),
            Some(&mut Provider::new()),
        )
        .unwrap();
    assert_eq!(staged.base_revision(), first.revision());
    assert_eq!(staged.stats().embedded_entries, 1);
    assert_eq!(staged.stats().deleted, 1);
    assert!(Arc::ptr_eq(&m.snapshot().unwrap(), &first));
    let second = m.publish(staged).unwrap();
    assert_record(&second, "a", 3);
    let query = Embedding::new(space(), raw("record:3")).unwrap();
    let semantic_hits = second
        .semantic()
        .unwrap()
        .search_embedding(&query, &SemanticSearchOptions::new(1))
        .unwrap();
    assert_eq!(semantic_hits[0].entry.id, "a");
    assert_eq!(semantic_hits[0].entry.text, "record:3");
    assert!(second.lexical().get("b").is_none());
    assert!(second.semantic().unwrap().embedding("b").is_none());
    assert_eq!(second.provenance().status("b"), ProvenanceStatus::Missing);
    assert!(second.provenance().source("source-b").is_none());
    assert_record(&first, "a", 1);
    assert_record(&first, "b", 2);
    assert!(matches!(
        m.require_current(first.revision()),
        Err(DatasetError::Conflict)
    ));
    m.require_current(second.revision()).unwrap();
}
#[test]
fn exact_text_metadata_updates_reuse_vector_buffers_and_clear_unprovided_sources() {
    let m = manager();
    let first = seed(&m);
    let a = first.semantic().unwrap().embedding("a").unwrap();
    let b = first.semantic().unwrap().embedding("b").unwrap();
    let record = DatasetRecord::new(
        KnowledgeEntry::new("a", "record:1").with_metadata(json!({"label":"updated"})),
    );
    let prepared = m
        .prepare(
            first.revision(),
            DatasetUpdate::new(vec![DatasetChange::Put(record)]),
            None,
        )
        .unwrap();
    assert_eq!(prepared.stats().embedded_entries, 0);
    assert_eq!(prepared.stats().reused_vectors, 2);
    let next = m.publish(prepared).unwrap();
    assert_eq!(
        next.semantic().unwrap().embedding("a").unwrap().as_ptr(),
        a.as_ptr()
    );
    assert_eq!(
        next.semantic().unwrap().embedding("b").unwrap().as_ptr(),
        b.as_ptr()
    );
    assert_eq!(next.semantic().unwrap().embedding("a").unwrap(), a);
    assert_eq!(next.provenance().status("a"), ProvenanceStatus::Missing);
    assert!(first.provenance().source("source-a").is_some());
    let changed = m.prepare(
        next.revision(),
        DatasetUpdate::new(vec![change("a", 8)]),
        None,
    );
    assert!(matches!(changed, Err(DatasetError::MissingProvider)));
    assert!(Arc::ptr_eq(&next, &m.snapshot().unwrap()));
}
#[test]
fn invalid_later_records_duplicate_ids_and_metadata_fail_before_inference() {
    let m = manager();
    let first = seed(&m);
    for changes in [
        vec![
            change("a", 5),
            DatasetChange::Put(DatasetRecord::new(KnowledgeEntry::new("bad", ""))),
        ],
        vec![change("a", 5), change("a", 6)],
        vec![DatasetChange::Delete(" ".into())],
        vec![DatasetChange::Put(
            DatasetRecord::new(KnowledgeEntry::new("a", "v"))
                .with_sources(vec![reference("s", "v"), reference("s", "v")]),
        )],
    ] {
        let mut p = Provider::new();
        assert!(m
            .prepare(first.revision(), DatasetUpdate::new(changes), Some(&mut p))
            .is_err());
        assert_eq!(p.calls, 0);
        assert!(Arc::ptr_eq(&first, &m.snapshot().unwrap()));
    }
    let mut deep = json!(0);
    for _ in 0..70 {
        deep = json!([deep]);
    }
    let r = DatasetRecord::new(KnowledgeEntry::new("a", "v"))
        .with_sources(vec![reference("s", "v").with_metadata(deep)]);
    let mut p = Provider::new();
    assert!(m
        .prepare(
            first.revision(),
            DatasetUpdate::new(vec![DatasetChange::Put(r)]),
            Some(&mut p)
        )
        .is_err());
    assert_eq!(p.calls, 0);
}
#[test]
fn shared_reference_redefinition_requires_replacing_every_affected_binding() {
    let m = DatasetManager::new("refs", DatasetConfig::default()).unwrap();
    let empty = m.snapshot().unwrap();
    let records = vec![
        DatasetRecord::new(KnowledgeEntry::new("a", "old"))
            .with_sources(vec![reference("shared", "old")]),
        DatasetRecord::new(KnowledgeEntry::new("b", "old"))
            .with_sources(vec![reference("shared", "old")]),
    ];
    let first = m
        .publish(
            m.prepare_replace(empty.revision(), records, None, None)
                .unwrap(),
        )
        .unwrap();
    let make = |id: &str| {
        DatasetChange::Put(
            DatasetRecord::new(KnowledgeEntry::new(id, "new"))
                .with_sources(vec![reference("shared", "new")]),
        )
    };
    assert!(matches!(
        m.prepare(first.revision(), DatasetUpdate::new(vec![make("a")]), None),
        Err(DatasetError::Provenance(_))
    ));
    let next = m
        .publish(
            m.prepare(
                first.revision(),
                DatasetUpdate::new(vec![make("a"), make("b")]),
                None,
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(
        next.provenance().source("shared").unwrap().revision(),
        Some("new")
    );
    assert_eq!(
        first.provenance().source("shared").unwrap().revision(),
        Some("old")
    );
}
#[test]
fn failed_partial_embedding_batches_wrong_spaces_and_nonfinite_vectors_never_publish() {
    let m = DatasetManager::new(
        "failure",
        DatasetConfig::default()
            .with_embedding_space(space())
            .unwrap()
            .with_batch_size(1)
            .unwrap(),
    )
    .unwrap();
    let base = m.snapshot().unwrap();
    for mode in 0..5 {
        let mut p = Provider::new();
        match mode {
            0 => p.fail_at = 2,
            1 => p.bad = true,
            2 => p.change_space = true,
            3 => p.space = EmbeddingSpace::new("wrong-space", 2).unwrap(),
            _ => p.bad_count = true,
        };
        assert!(m
            .prepare(
                base.revision(),
                DatasetUpdate::new(vec![change("a", 1), change("b", 2)]),
                Some(&mut p)
            )
            .is_err());
        assert!(Arc::ptr_eq(&base, &m.snapshot().unwrap()));
        assert!(base.lexical().is_empty());
    }
}
#[test]
fn stale_and_cross_dataset_preparations_fail_and_manager_clones_share_state() {
    let m = manager();
    let clone = m.clone();
    let base = m.snapshot().unwrap();
    let other = manager();
    assert_ne!(
        other.snapshot().unwrap().revision().identity(),
        base.revision().identity()
    );
    let a = m
        .prepare(
            base.revision(),
            DatasetUpdate::new(vec![change("a", 1)]),
            Some(&mut Provider::new()),
        )
        .unwrap();
    let b = m
        .prepare(
            base.revision(),
            DatasetUpdate::new(vec![change("b", 2)]),
            Some(&mut Provider::new()),
        )
        .unwrap();
    assert!(matches!(other.publish(a), Err(DatasetError::Conflict)));
    let winner = clone.publish(b).unwrap();
    assert_eq!(m.snapshot().unwrap().revision(), winner.revision());
    let mut p = Provider::new();
    assert!(matches!(
        m.prepare(
            base.revision(),
            DatasetUpdate::new(vec![change("a", 2)]),
            Some(&mut p)
        ),
        Err(DatasetError::Conflict)
    ));
    assert_eq!(p.calls, 0);
}
#[test]
fn full_reconciliation_removes_missing_records_and_is_not_limited_to_delta_batch_count() {
    let limits = StorageLimits::default().with_batch_limits(1, 100).unwrap();
    let m = DatasetManager::new(
        "replace",
        DatasetConfig::default()
            .with_record_limits(limits)
            .with_embedding_space(space())
            .unwrap(),
    )
    .unwrap();
    let base = m.snapshot().unwrap();
    assert!(m
        .prepare(
            base.revision(),
            DatasetUpdate::new(vec![change("a", 1), change("b", 2)]),
            Some(&mut Provider::new())
        )
        .is_err());
    let first = m
        .publish(
            m.prepare_replace(
                base.revision(),
                vec![record("a", 1), record("b", 2)],
                None,
                Some(&mut Provider::new()),
            )
            .unwrap(),
        )
        .unwrap();
    let a = first.semantic().unwrap().embedding("a").unwrap();
    let mut p = Provider::new();
    let next = m
        .publish(
            m.prepare_replace(
                first.revision(),
                vec![record("a", 1), record("c", 3)],
                None,
                Some(&mut p),
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(p.seen, vec!["record:3"]);
    assert_eq!(
        next.semantic().unwrap().embedding("a").unwrap().as_ptr(),
        a.as_ptr()
    );
    assert!(next.lexical().get("b").is_none());
    assert!(next.provenance().source("source-b").is_none());
    assert_record(&next, "a", 1);
    assert_record(&next, "c", 3);
    assert!(m
        .prepare_replace(
            next.revision(),
            vec![record("x", 1), record("x", 2)],
            None,
            Some(&mut p)
        )
        .is_err());
    let empty = m
        .publish(
            m.prepare_replace(next.revision(), Vec::new(), None, None)
                .unwrap(),
        )
        .unwrap();
    assert!(empty.lexical().is_empty());
    assert!(empty.provenance().is_empty());
    assert_eq!(empty.semantic().unwrap().len(), 0);
}
#[test]
fn source_stamps_advance_monotonically_and_model_rebuilds_keep_the_source_stamp() {
    let m = manager();
    let base = m.snapshot().unwrap();
    let source = StoreRevision::new("sqlite", "a".repeat(32), 5).unwrap();
    let first = m
        .publish(
            m.prepare(
                base.revision(),
                DatasetUpdate::new(vec![change("a", 1)]).with_source_revision(source.clone()),
                Some(&mut Provider::new()),
            )
            .unwrap(),
        )
        .unwrap();
    for incoming in [
        None,
        Some(source.clone()),
        Some(StoreRevision::new("sqlite", "a".repeat(32), 4).unwrap()),
        Some(StoreRevision::new("sqlite", "b".repeat(32), 6).unwrap()),
        Some(StoreRevision::new("other", "a".repeat(32), 6).unwrap()),
    ] {
        let mut u = DatasetUpdate::new(vec![change("a", 2)]);
        if let Some(s) = incoming {
            u = u.with_source_revision(s);
        }
        assert!(matches!(
            m.prepare(first.revision(), u, Some(&mut Provider::new())),
            Err(DatasetError::SourceRevision)
        ));
    }
    let model = m
        .publish(m.prepare_model(first.revision(), None).unwrap())
        .unwrap();
    assert!(model.semantic().is_none());
    assert_eq!(model.source_revision(), Some(&source));
    let source_next = StoreRevision::new("sqlite", "a".repeat(32), 6).unwrap();
    let advanced = m
        .publish(
            m.prepare(
                model.revision(),
                DatasetUpdate::new(vec![]).with_source_revision(source_next.clone()),
                None,
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(advanced.source_revision(), Some(&source_next));
    assert_eq!(advanced.lexical().get("a").unwrap().text, "record:1");
    assert!(m
        .prepare(advanced.revision(), DatasetUpdate::new(vec![]), None)
        .is_err());
}
#[test]
fn model_replacement_rebuilds_all_vectors_and_failure_keeps_the_previous_model() {
    let m = manager();
    let first = seed(&m);
    let mut p = Provider::new();
    p.space = EmbeddingSpace::new("next-model", 2).unwrap();
    let next = m
        .publish(m.prepare_model(first.revision(), Some(&mut p)).unwrap())
        .unwrap();
    assert_eq!(p.seen.len(), 2);
    assert_eq!(next.semantic().unwrap().space().model(), "next-model");
    assert_eq!(first.semantic().unwrap().space(), &space());
    let mut bad = Provider::new();
    bad.fail_at = 1;
    assert!(m.prepare_model(next.revision(), Some(&mut bad)).is_err());
    assert!(Arc::ptr_eq(&next, &m.snapshot().unwrap()));
    let lexical = m
        .publish(m.prepare_model(next.revision(), None).unwrap())
        .unwrap();
    assert!(lexical.semantic().is_none());
    assert_eq!(lexical.provenance(), first.provenance());
}
#[test]
fn final_batch_capacity_is_independent_of_change_order() {
    for reverse in [false, true] {
        let m = DatasetManager::new(
            "capacity",
            DatasetConfig::default().with_limits(1, 170, 100).unwrap(),
        )
        .unwrap();
        let base = m.snapshot().unwrap();
        let first = m
            .publish(
                m.prepare(
                    base.revision(),
                    DatasetUpdate::new(vec![DatasetChange::Put(DatasetRecord::new(
                        KnowledgeEntry::new("old", "x".repeat(70)),
                    ))]),
                    None,
                )
                .unwrap(),
            )
            .unwrap();
        let mut changes = vec![
            DatasetChange::Put(DatasetRecord::new(KnowledgeEntry::new(
                "new",
                "x".repeat(70),
            ))),
            DatasetChange::Delete("old".into()),
        ];
        if reverse {
            changes.reverse();
        }
        let next = m
            .publish(
                m.prepare(first.revision(), DatasetUpdate::new(changes), None)
                    .unwrap(),
            )
            .unwrap();
        assert!(next.lexical().get("old").is_none());
        assert!(next.lexical().get("new").is_some());
        assert!(next.encoded_record_bytes() <= 170);
    }
}
#[test]
fn entry_encoded_source_vector_and_dimension_budgets_fail_without_inference() {
    let configs = vec![
        DatasetConfig::default()
            .with_limits(1, 10000, 10000)
            .unwrap()
            .with_embedding_space(space())
            .unwrap(),
        DatasetConfig::default()
            .with_limits(10, 10000, 8)
            .unwrap()
            .with_embedding_space(space())
            .unwrap(),
        DatasetConfig::default()
            .with_limits(10, 30, 10000)
            .unwrap()
            .with_embedding_space(space())
            .unwrap(),
    ];
    for config in configs {
        let m = DatasetManager::new("limits", config).unwrap();
        let base = m.snapshot().unwrap();
        let mut p = Provider::new();
        assert!(m
            .prepare_replace(
                base.revision(),
                vec![record("a", 1), record("b", 2)],
                None,
                Some(&mut p)
            )
            .is_err());
        assert_eq!(p.calls, 0);
    }
    let m = DatasetManager::new(
        "source-limit",
        DatasetConfig::default().with_source_limit(1).unwrap(),
    )
    .unwrap();
    let base = m.snapshot().unwrap();
    let r = DatasetRecord::new(KnowledgeEntry::new("a", "v"))
        .with_sources(vec![reference("one", "v"), reference("two", "v")]);
    assert!(m
        .prepare_replace(base.revision(), vec![r], None, None)
        .is_err());
    assert!(DatasetConfig::default()
        .with_embedding_space(EmbeddingSpace::new("x", 9000).unwrap())
        .is_err());
    assert!(DatasetConfig::default().with_limits(0, 1, 1).is_err());
    assert!(DatasetConfig::default().with_batch_size(0).is_err());
    assert!(DatasetConfig::default().with_source_limit(0).is_err());
    assert!(DatasetConfig::default().with_max_dimensions(0).is_err());
}
#[test]
fn slow_preparation_does_not_block_readers_or_allow_a_stale_publish() {
    struct Blocking {
        inner: Provider,
        started: mpsc::Sender<()>,
        resume: mpsc::Receiver<()>,
    }
    impl EmbeddingProvider for Blocking {
        fn space(&self) -> &EmbeddingSpace {
            self.inner.space()
        }
        fn embed(
            &mut self,
            t: &[&str],
            p: EmbeddingPurpose,
        ) -> Result<Vec<Vec<f32>>, EmbeddingError> {
            self.started.send(()).unwrap();
            self.resume.recv_timeout(Duration::from_secs(5)).unwrap();
            self.inner.embed(t, p)
        }
    }
    let m = manager();
    let first = seed(&m);
    let expected = first.revision().clone();
    let worker = m.clone();
    let (started_tx, started_rx) = mpsc::channel();
    let (resume_tx, resume_rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let mut p = Blocking {
            inner: Provider::new(),
            started: started_tx,
            resume: resume_rx,
        };
        worker
            .prepare(
                &expected,
                DatasetUpdate::new(vec![change("a", 8)]),
                Some(&mut p),
            )
            .unwrap()
    });
    started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_record(&m.snapshot().unwrap(), "a", 1);
    let winner = m
        .publish(
            m.prepare(
                first.revision(),
                DatasetUpdate::new(vec![change("a", 9)]),
                Some(&mut Provider::new()),
            )
            .unwrap(),
        )
        .unwrap();
    resume_tx.send(()).unwrap();
    let stale = handle.join().unwrap();
    assert!(matches!(m.publish(stale), Err(DatasetError::Conflict)));
    assert!(Arc::ptr_eq(&winner, &m.snapshot().unwrap()));
    assert_record(&winner, "a", 9);
}
#[test]
fn concurrent_writers_with_the_same_base_have_exactly_one_winner() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<DatasetManager>();
    send_sync::<DatasetSnapshot>();
    let m = DatasetManager::new("race", DatasetConfig::default()).unwrap();
    let base = m.snapshot().unwrap();
    let barrier = Arc::new(Barrier::new(12));
    let mut handles = Vec::new();
    for i in 0..12 {
        let manager = m.clone();
        let revision = base.revision().clone();
        let b = barrier.clone();
        handles.push(thread::spawn(move || {
            let prepared = manager
                .prepare(
                    &revision,
                    DatasetUpdate::new(vec![change(&format!("id{i}"), i)]),
                    None,
                )
                .unwrap();
            b.wait();
            manager.publish(prepared)
        }));
    }
    let mut winners = 0;
    for handle in handles {
        match handle.join().unwrap() {
            Ok(_) => winners += 1,
            Err(e) => assert!(matches!(e, DatasetError::Conflict)),
        }
    }
    assert_eq!(winners, 1);
    let view = m.snapshot().unwrap();
    assert_eq!(view.revision().generation(), 1);
    assert_eq!(view.lexical().len(), 1);
    assert_eq!(view.provenance().entry_count(), 1);
    assert!(base.lexical().is_empty());
}
#[test]
fn readers_never_observe_mixed_text_vectors_and_sources_during_repeated_publication() {
    let m = manager();
    let initial = m.snapshot().unwrap();
    m.publish(
        m.prepare(
            initial.revision(),
            DatasetUpdate::new(vec![change("a", 0)]),
            Some(&mut Provider::new()),
        )
        .unwrap(),
    )
    .unwrap();
    let start = Arc::new(Barrier::new(5));
    let mut readers = Vec::new();
    for _ in 0..4 {
        let manager = m.clone();
        let b = start.clone();
        readers.push(thread::spawn(move || {
            b.wait();
            for _ in 0..600 {
                let view = manager.snapshot().unwrap();
                let value = view
                    .lexical()
                    .get("a")
                    .unwrap()
                    .text
                    .split(':')
                    .next_back()
                    .unwrap()
                    .parse()
                    .unwrap();
                assert_record(&view, "a", value);
            }
        }));
    }
    start.wait();
    for value in 1..60 {
        let base = m.snapshot().unwrap();
        m.publish(
            m.prepare(
                base.revision(),
                DatasetUpdate::new(vec![change("a", value)]),
                Some(&mut Provider::new()),
            )
            .unwrap(),
        )
        .unwrap();
    }
    for reader in readers {
        reader.join().unwrap();
    }
    assert_record(&m.snapshot().unwrap(), "a", 59);
}
#[test]
fn seeded_reconciliation_matches_an_independent_record_and_source_map() {
    let m = manager();
    let mut expected: BTreeMap<String, usize> = BTreeMap::new();
    let mut seed = 13u64;
    for step in 0..260 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let id = format!("id{}", (seed >> 32) % 20);
        let base = m.snapshot().unwrap();
        let old: Vec<_> = expected.iter().map(|(id, v)| (id.clone(), *v)).collect();
        let prepared = if step % 13 == 12 {
            m.prepare_replace(
                base.revision(),
                expected.iter().map(|(id, v)| record(id, *v)).collect(),
                None,
                Some(&mut Provider::new()),
            )
            .unwrap()
        } else if step % 5 == 0 {
            expected.remove(&id);
            m.prepare(
                base.revision(),
                DatasetUpdate::new(vec![DatasetChange::Delete(id)]),
                None,
            )
            .unwrap()
        } else {
            expected.insert(id.clone(), step);
            m.prepare(
                base.revision(),
                DatasetUpdate::new(vec![change(&id, step)]),
                Some(&mut Provider::new()),
            )
            .unwrap()
        };
        let next = m.publish(prepared).unwrap();
        assert_eq!(next.lexical().len(), expected.len());
        for (id, v) in &expected {
            assert_record(&next, id, *v);
        }
        for (id, v) in old {
            assert_record(&base, &id, v);
        }
    }
}
#[test]
fn panicking_external_providers_do_not_poison_the_publication_lock() {
    struct PanicProvider(EmbeddingSpace);
    impl EmbeddingProvider for PanicProvider {
        fn space(&self) -> &EmbeddingSpace {
            &self.0
        }
        fn embed(
            &mut self,
            _: &[&str],
            _: EmbeddingPurpose,
        ) -> Result<Vec<Vec<f32>>, EmbeddingError> {
            panic!("provider panic")
        }
    }
    let m = manager();
    let base = m.snapshot().unwrap();
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        m.prepare(
            base.revision(),
            DatasetUpdate::new(vec![change("a", 1)]),
            Some(&mut PanicProvider(space())),
        )
    }));
    assert!(caught.is_err());
    assert!(Arc::ptr_eq(&base, &m.snapshot().unwrap()));
    seed(&m);
}
#[cfg(feature = "advanced-search")]
#[test]
fn positional_index_updates_and_analysis_failures_share_the_same_publication_boundary() {
    use airust::text_search::{AnalyzerConfig, TextSearchOptions};
    let m = DatasetManager::new(
        "german",
        DatasetConfig::default()
            .with_text_analyzer(AnalyzerConfig::german().with_limits(100, 400, 100).unwrap()),
    )
    .unwrap();
    let base = m.snapshot().unwrap();
    let record = DatasetRecord::new(KnowledgeEntry::new("de", "Die Häuser haben Gärten."))
        .with_sources(vec![reference("de-source", "one")]);
    let first = m
        .publish(
            m.prepare_replace(base.revision(), vec![record], None, None)
                .unwrap(),
        )
        .unwrap();
    assert_eq!(
        first
            .text_index()
            .unwrap()
            .search("\"Haus haben\"", &TextSearchOptions::new(1))
            .unwrap()[0]
            .entry
            .id,
        "de"
    );
    assert_eq!(
        first.text_index().unwrap().get("de").unwrap().text,
        first.lexical().get("de").unwrap().text
    );
    assert_eq!(first.stats().text_entries_rebuilt, 1);
    let invalid = DatasetRecord::new(KnowledgeEntry::new("de", "x".repeat(101)));
    assert!(m
        .prepare(
            first.revision(),
            DatasetUpdate::new(vec![DatasetChange::Put(invalid)]),
            None
        )
        .is_err());
    assert!(Arc::ptr_eq(&first, &m.snapshot().unwrap()));
    let next = m
        .publish(
            m.prepare_model(first.revision(), Some(&mut Provider::new()))
                .unwrap(),
        )
        .unwrap();
    assert!(std::ptr::eq(
        next.text_index().unwrap(),
        first.text_index().unwrap()
    ));
    assert!(next.semantic().is_some());
}
#[test]
fn complete_authoritative_storage_reads_can_be_reconciled_without_forgetting_deletions() {
    use airust::storage::{
        read_snapshot, MemoryStore, SnapshotOptions, StorageBackend, StoreChange,
    };
    let mut store = MemoryStore::new("source").unwrap();
    let rev = store.revision().unwrap();
    store
        .apply(
            &rev,
            &[
                StoreChange::Put(KnowledgeEntry::new("a", "record:1")),
                StoreChange::Put(KnowledgeEntry::new("b", "record:2")),
            ],
        )
        .unwrap();
    let m = manager();
    let dataset = m.snapshot().unwrap();
    let (source, records) = read_snapshot(&mut store, &SnapshotOptions::default())
        .unwrap()
        .into_parts();
    let records = records
        .into_iter()
        .map(|e| {
            let r = reference(&format!("source-{}", e.id), &e.text);
            DatasetRecord::new(e).with_sources(vec![r])
        })
        .collect();
    let first = m
        .publish(
            m.prepare_replace(
                dataset.revision(),
                records,
                Some(source.clone()),
                Some(&mut Provider::new()),
            )
            .unwrap(),
        )
        .unwrap();
    assert_record(&first, "a", 1);
    assert_record(&first, "b", 2);
    store
        .apply(
            &source,
            &[
                StoreChange::Delete("b".into()),
                StoreChange::Put(KnowledgeEntry::new("a", "record:3")),
            ],
        )
        .unwrap();
    let (source, records) = read_snapshot(&mut store, &SnapshotOptions::default())
        .unwrap()
        .into_parts();
    let records = records
        .into_iter()
        .map(|e| {
            let r = reference(&format!("source-{}", e.id), &e.text);
            DatasetRecord::new(e).with_sources(vec![r])
        })
        .collect();
    let next = m
        .publish(
            m.prepare_replace(
                first.revision(),
                records,
                Some(source.clone()),
                Some(&mut Provider::new()),
            )
            .unwrap(),
        )
        .unwrap();
    assert_record(&next, "a", 3);
    assert!(next.lexical().get("b").is_none());
    assert!(next.semantic().unwrap().embedding("b").is_none());
    assert!(next.provenance().source("source-b").is_none());
    assert_eq!(next.source_revision(), Some(&source));
}

#[test]
fn oversized_source_encoding_and_model_rebuild_budgets_fail_before_provider_work() {
    let limits = StorageLimits::default()
        .with_entry_limits(100, 120)
        .unwrap();
    let m = DatasetManager::new(
        "encoded",
        DatasetConfig::default().with_record_limits(limits),
    )
    .unwrap();
    let base = m.snapshot().unwrap();
    let source = reference("source", "one").with_metadata(json!({"escaped":"\n".repeat(50)}));
    let record = DatasetRecord::new(KnowledgeEntry::new("a", "v")).with_sources(vec![source]);
    assert!(matches!(
        m.prepare_replace(base.revision(), vec![record], None, None),
        Err(DatasetError::Limit("encoded record including sources"))
    ));
    let m = DatasetManager::new(
        "dimensions",
        DatasetConfig::default().with_max_dimensions(2).unwrap(),
    )
    .unwrap();
    let base = m.snapshot().unwrap();
    let mut p = Provider::new();
    p.space = EmbeddingSpace::new("too-wide", 3).unwrap();
    assert!(m.prepare_model(base.revision(), Some(&mut p)).is_err());
    assert_eq!(p.calls, 0);
}
#[cfg(feature = "retrieval")]
#[test]
fn compact_retrieval_and_source_lookup_use_one_retained_snapshot() {
    use airust::retrieval::{ExcerptOptions, LexicalSource, RetrievalOptions, Retriever};
    let m = manager();
    let base = m.snapshot().unwrap();
    let original = format!(
        "{} Rust ownership prevents memory errors:1",
        "Background. ".repeat(80)
    );
    let record = DatasetRecord::new(KnowledgeEntry::new("a", &original))
        .with_sources(vec![reference("source-a", "committed-one")]);
    let first = m
        .publish(
            m.prepare_replace(
                base.revision(),
                vec![record],
                None,
                Some(&mut Provider::new()),
            )
            .unwrap(),
        )
        .unwrap();
    let options = RetrievalOptions::new(1)
        .with_relevant_excerpts(ExcerptOptions::default())
        .with_byte_budgets(1024, 120, 120, 10000)
        .unwrap();
    let response = Retriever::new(LexicalSource::new(first.lexical()))
        .query("Rust ownership", &options)
        .unwrap();
    assert_eq!(
        response.hits()[0].span.slice(&original),
        Some(response.hits()[0].excerpt.as_str())
    );
    m.publish(
        m.prepare(
            first.revision(),
            DatasetUpdate::new(vec![change("a", 5)]),
            Some(&mut Provider::new()),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        first
            .provenance()
            .sources_for(&response.hits()[0].id)
            .next()
            .unwrap()
            .revision(),
        Some("committed-one")
    );
    assert!(matches!(
        m.require_current(first.revision()),
        Err(DatasetError::Conflict)
    ));
}
#[test]
fn source_locations_remain_coordinates_in_external_artifacts() {
    let m = DatasetManager::new("locations", DatasetConfig::default()).unwrap();
    let base = m.snapshot().unwrap();
    let source = reference("pdf", "v1")
        .with_text_span(airust::provenance::TextSpan::new(10000, 10020).unwrap());
    let record =
        DatasetRecord::new(KnowledgeEntry::new("a", "short excerpt")).with_sources(vec![source]);
    let view = m
        .publish(
            m.prepare_replace(base.revision(), vec![record], None, None)
                .unwrap(),
        )
        .unwrap();
    assert_eq!(
        view.provenance()
            .source("pdf")
            .unwrap()
            .text_span()
            .unwrap()
            .start_byte(),
        10000
    );
}

#[test]
fn full_payload_capacity_and_identifier_limits_fail_early_without_partial_state() {
    let manager = DatasetManager::new(
        "early-limits",
        DatasetConfig::default().with_limits(10, 200, 1000).unwrap(),
    )
    .unwrap();
    let base = manager.snapshot().unwrap();
    let records = vec![
        DatasetRecord::new(KnowledgeEntry::new("a", "x".repeat(100))),
        DatasetRecord::new(KnowledgeEntry::new("b", "x".repeat(100))),
        DatasetRecord::new(KnowledgeEntry::new("c", "")),
    ];
    assert!(matches!(
        manager.prepare_replace(base.revision(), records, None, None),
        Err(DatasetError::Limit("total record bytes"))
    ));
    let too_long = DatasetRecord::new(KnowledgeEntry::new(" ".repeat(5000), "text"));
    assert!(matches!(
        manager.prepare_replace(base.revision(), vec![too_long], None, None),
        Err(DatasetError::InvalidData("invalid change ID"))
    ));
    assert!(Arc::ptr_eq(&base, &manager.snapshot().unwrap()));
}
