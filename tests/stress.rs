use airust::{Engine, KnowledgeEntry, SearchAlgorithm, SearchOptions};
use std::collections::{BTreeMap, BTreeSet};
#[test]
fn long_seeded_mutation_workload_matches_authoritative_records_and_fresh_rebuilds() {
    let mut index = Engine::new();
    let mut originals = BTreeMap::new();
    let mut seed = 71u64;
    for step in 0..4000 {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let id = format!("item{}", seed % 1500);
        if seed.is_multiple_of(5) {
            assert_eq!(index.remove(&id), originals.remove(&id));
        } else {
            let text = match seed % 4 {
                0 => format!("Rust ownership topic{}", seed % 31),
                1 => format!("Häuser Türen Café topic{}", seed % 31),
                2 => format!("ㄱㅏ 가 日本語 topic{}", seed % 31),
                _ => format!("ＡＩ e\u{301} memory topic{}", seed % 31),
            };
            let entry =
                KnowledgeEntry::new(&id, text).with_metadata(serde_json::json!({"step":step}));
            assert_eq!(
                index.upsert(entry.clone()).unwrap(),
                originals.insert(id, entry)
            );
        }
        if step % 100 == 0 {
            let rebuilt = Engine::from_entries(originals.values().cloned()).unwrap();
            assert_eq!(index.stats(), rebuilt.stats());
            let tokens: Vec<_> = originals
                .values()
                .flat_map(|e| airust::text::tokenize(&e.text))
                .collect();
            assert_eq!(index.stats().tokens, tokens.len());
            assert_eq!(
                index.stats().terms,
                tokens.into_iter().collect::<BTreeSet<_>>().len()
            );
            for query in ["Rust memory", "Häuser", "가", "topic7", "unavailable"] {
                for algorithm in [SearchAlgorithm::Bm25, SearchAlgorithm::Tfidf] {
                    let options = SearchOptions {
                        algorithm,
                        limit: 30,
                        ..SearchOptions::default()
                    };
                    let actual = index.search_with_options(query, &options);
                    let expected = rebuilt.search_with_options(query, &options);
                    assert_eq!(
                        actual
                            .iter()
                            .map(|h| (&h.entry.id, h.score.to_bits(), &h.matched_terms))
                            .collect::<Vec<_>>(),
                        expected
                            .iter()
                            .map(|h| (&h.entry.id, h.score.to_bits(), &h.matched_terms))
                            .collect::<Vec<_>>()
                    );
                }
            }
        }
    }
}
#[test]
fn shared_immutable_index_keeps_ranking_stable_across_concurrent_readers() {
    let index = std::sync::Arc::new(
        Engine::from_entries((0..5000).map(|i| {
            KnowledgeEntry::new(i.to_string(), format!("Rust ownership topic{}", i % 100))
        }))
        .unwrap(),
    );
    let expected = index
        .search("Rust topic7", 10)
        .iter()
        .map(|h| (h.entry.id.clone(), h.score.to_bits()))
        .collect::<Vec<_>>();
    std::thread::scope(|scope| {
        for _ in 0..8 {
            let index = index.clone();
            let expected = &expected;
            scope.spawn(move || {
                for _ in 0..25 {
                    assert_eq!(
                        index
                            .search("Rust topic7", 10)
                            .iter()
                            .map(|h| (h.entry.id.clone(), h.score.to_bits()))
                            .collect::<Vec<_>>(),
                        *expected
                    );
                }
            });
        }
    });
}
