#![cfg(feature = "shared-index")]
use airust::{Engine, KnowledgeEntry, SearchAlgorithm, SearchOptions};
#[test]
fn retained_copies_preserve_records_metadata_duplicates_and_scores_after_changes() {
    let original = Engine::from_entries([
        KnowledgeEntry::new("a", "Rust ownership").with_metadata(serde_json::json!({"version":1})),
        KnowledgeEntry::new("b", "Rust ownership"),
        KnowledgeEntry::new("c", "Python scripts"),
    ])
    .unwrap();
    let mut changed = original.clone();
    let before = original.to_json().unwrap();
    assert_eq!(
        changed
            .upsert(KnowledgeEntry::new("a", "Rust memory"))
            .unwrap()
            .unwrap()
            .metadata,
        Some(serde_json::json!({"version":1}))
    );
    assert!(changed.remove("b").is_some());
    changed
        .upsert(KnowledgeEntry::new("new", "Rust memory"))
        .unwrap();
    assert_eq!(original.to_json().unwrap(), before);
    assert_eq!(original.get("a").unwrap().text, "Rust ownership");
    assert!(original.get("new").is_none());
    let rebuilt = Engine::from_entries(changed.entries().cloned()).unwrap();
    assert_eq!(changed.stats(), rebuilt.stats());
    for algorithm in [SearchAlgorithm::Bm25, SearchAlgorithm::Tfidf] {
        let options = SearchOptions {
            algorithm,
            ..SearchOptions::default()
        };
        assert_eq!(
            changed
                .search_with_options("Rust memory", &options)
                .iter()
                .map(|h| (&h.entry.id, h.score.to_bits()))
                .collect::<Vec<_>>(),
            rebuilt
                .search_with_options("Rust memory", &options)
                .iter()
                .map(|h| (&h.entry.id, h.score.to_bits()))
                .collect::<Vec<_>>()
        );
    }
    changed.clear();
    assert!(changed.is_empty());
    assert_eq!(original.len(), 3);
}
#[test]
fn multiple_retained_generations_match_independent_authoritative_records() {
    let mut index = Engine::from_entries(
        (0..1000).map(|i| KnowledgeEntry::new(i.to_string(), format!("Rust topic{}", i % 31))),
    )
    .unwrap();
    let mut records = index
        .entries()
        .cloned()
        .map(|e| (e.id.clone(), e))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut retained = Vec::new();
    for step in 0usize..300 {
        if step.is_multiple_of(30) {
            retained.push((index.clone(), records.clone()));
        }
        let id = (step % 130).to_string();
        if step.is_multiple_of(7) {
            assert_eq!(index.remove(&id), records.remove(&id));
        } else {
            let entry =
                KnowledgeEntry::new(&id, format!("changed topic{} version{step}", step % 31))
                    .with_metadata(serde_json::json!({"step":step}));
            assert_eq!(
                index.upsert(entry.clone()).unwrap(),
                records.insert(id, entry)
            );
        }
    }
    for (view, records) in retained {
        assert_eq!(
            view.entries()
                .cloned()
                .map(|e| (e.id.clone(), e))
                .collect::<std::collections::BTreeMap<_, _>>(),
            records
        );
        let rebuilt = Engine::from_entries(records.into_values()).unwrap();
        assert_eq!(view.to_json().unwrap(), rebuilt.to_json().unwrap());
        assert_eq!(view.stats(), rebuilt.stats());
    }
}
#[test]
fn concurrent_copy_writers_do_not_change_their_base_or_siblings() {
    let base = Engine::from_entries(
        (0..1000).map(|i| KnowledgeEntry::new(i.to_string(), "Rust original")),
    )
    .unwrap();
    let before = base.to_json().unwrap();
    let results = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..8)
            .map(|worker| {
                let mut index = base.clone();
                scope.spawn(move || {
                    for i in 0..128 {
                        index
                            .upsert(KnowledgeEntry::new(
                                i.to_string(),
                                format!("marker{worker} {i}"),
                            ))
                            .unwrap();
                    }
                    assert_eq!(index.search(&format!("marker{worker}"), 200).len(), 128);
                    for other in 0..8 {
                        if other != worker {
                            assert!(index.search(&format!("marker{other}"), 10).is_empty());
                        }
                    }
                    index
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert_eq!(results.len(), 8);
    assert_eq!(base.to_json().unwrap(), before);
}
