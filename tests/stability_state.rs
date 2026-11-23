use airust::{Context, Engine, KnowledgeEntry, SearchAlgorithm, SearchOptions};
use serde_json::json;
use std::collections::BTreeMap;

fn verify(index: &Engine, records: &BTreeMap<String, KnowledgeEntry>) {
    let rebuilt = Engine::from_entries(records.values().cloned()).unwrap();
    assert_eq!(
        index.entries().cloned().collect::<Vec<_>>(),
        records.values().cloned().collect::<Vec<_>>()
    );
    assert_eq!(index.stats(), rebuilt.stats());
    let queries: &[&str] = if cfg!(miri) {
        &[
            "shared",
            "shared shared shared",
            "CAFÉ cafe\u{301}",
            "absent",
        ]
    } else {
        &[
            "shared",
            "shared shared shared",
            "CAFÉ cafe\u{301}",
            "中文",
            "Слово",
            "السَّلَامُ",
            "a_b identifier",
            "0 1",
            "absent",
            "... 🦀",
        ]
    };
    let scopes: &[Option<i32>] = if cfg!(miri) {
        &[None, Some(1)]
    } else {
        &[None, Some(1), Some(99)]
    };
    let limits: &[usize] = if cfg!(miri) { &[1] } else { &[0, 1, 50] };
    for algorithm in [SearchAlgorithm::Bm25, SearchAlgorithm::Tfidf] {
        for &scope in scopes {
            for &limit in limits {
                let options = SearchOptions {
                    algorithm,
                    limit,
                    context: scope.map(|s| Context::new().with_metadata("scope", s)),
                    ..Default::default()
                };
                for query in queries {
                    let signature = |engine: &Engine| {
                        engine
                            .search_with_options(query, &options)
                            .into_iter()
                            .map(|h| (h.entry.clone(), h.score.to_bits(), h.matched_terms))
                            .collect::<Vec<_>>()
                    };
                    assert_eq!(
                        signature(index),
                        signature(&rebuilt),
                        "{query:?} {algorithm:?} {scope:?} {limit}"
                    );
                }
            }
        }
    }
}

#[test]
fn seeded_mutations_keep_independent_records_caches_and_retained_clones_consistent() {
    let mut index = Engine::new();
    let mut records = BTreeMap::new();
    let mut seed = 0x5a17_2026_u64;
    let mut snapshots = Vec::new();
    let steps = if cfg!(miri) { 8 } else { 250 };
    for step in 0..steps {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let id = format!("entry/é/{:02}", (seed >> 24) % 31);
        if step % 7 == 0 {
            verify(&index, &records);
            snapshots.push((index.clone(), records.clone(), index.to_json().unwrap()));
        }
        match (seed >> 8) % 5 {
            0 => assert_eq!(index.remove(&id), records.remove(&id)),
            1..=3 => {
                let text = format!(
                    "shared {} café 中文 Слово السَّلَامُ a_b identifier topic{}",
                    "shared ".repeat((seed % 4) as usize),
                    seed % 7
                );
                let entry = KnowledgeEntry::new(id.clone(), text)
                    .with_metadata(json!({"scope": step % 3, "generation": step}));
                assert_eq!(
                    index.upsert(entry.clone()).unwrap(),
                    records.insert(id, entry)
                );
            }
            _ => {
                let before = index.to_json().unwrap();
                assert!(index.upsert(KnowledgeEntry::new(id, " \t\r\n")).is_err());
                assert_eq!(index.to_json().unwrap(), before);
            }
        }
        verify(&index, &records);
        if step % 11 == 0 {
            let restored = Engine::from_json(&index.to_json().unwrap()).unwrap();
            verify(&restored, &records);
        }
    }
    #[cfg(not(miri))]
    std::thread::scope(|scope| {
        let handles: Vec<_> = snapshots
            .iter()
            .map(|(snapshot, expected, before)| {
                scope.spawn(move || {
                    verify(snapshot, expected);
                    assert_eq!(&snapshot.to_json().unwrap(), before);
                })
            })
            .collect();
        for handle in handles {
            handle.join().unwrap();
        }
    });
    #[cfg(miri)]
    for (snapshot, expected, before) in &snapshots {
        verify(snapshot, expected);
        assert_eq!(&snapshot.to_json().unwrap(), before);
    }
    verify(&index, &records);
}
