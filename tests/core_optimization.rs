use airust::{Engine, KnowledgeEntry, SearchAlgorithm, SearchOptions};
use serde_json::json;

fn signature(index: &Engine, query: &str) -> Vec<(String, u64, Vec<String>)> {
    index
        .search_with_options(
            query,
            &SearchOptions {
                algorithm: SearchAlgorithm::Tfidf,
                limit: 100,
                ..Default::default()
            },
        )
        .into_iter()
        .map(|h| (h.entry.id.clone(), h.score.to_bits(), h.matched_terms))
        .collect()
}
fn source() -> Vec<KnowledgeEntry> {
    vec![
        KnowledgeEntry::new("a", "alpha alpha beta Straße"),
        KnowledgeEntry::new("b", "beta gamma Grüße 🌍"),
        KnowledgeEntry::new("c", "東京 alpha delta"),
        KnowledgeEntry::new("d", "🧠"),
    ]
}

#[test]
fn warm_norms_follow_document_count_frequencies_deletion_and_clear() {
    let mut index = Engine::from_entries(source()).unwrap();
    let queries = ["alpha", "beta gamma", "Straße 東京", "missing"];
    for step in 0..80 {
        for query in queries {
            let _ = signature(&index, query);
        }
        match step % 5 {
            0 => {
                index
                    .upsert(KnowledgeEntry::new(
                        format!("extra-{step}"),
                        "beta alpha alpha gamma",
                    ))
                    .unwrap();
            }
            1 => {
                index
                    .upsert(KnowledgeEntry::new(
                        "a",
                        format!("alpha {} Straße", "beta ".repeat(step + 1)),
                    ))
                    .unwrap();
            }
            2 => {
                index.remove(&format!("extra-{}", step - 2));
            }
            3 => {
                let mut entry = index.get("b").unwrap().clone();
                entry.metadata = Some(json!({"changed":step}));
                index.upsert(entry).unwrap();
            }
            _ => {
                assert!(index.upsert(KnowledgeEntry::new("a", "   ")).is_err());
            }
        }
        let rebuilt = Engine::from_entries(index.entries().cloned()).unwrap();
        for query in queries {
            assert_eq!(signature(&index, query), signature(&rebuilt, query));
        }
    }
    index.clear();
    index
        .upsert(KnowledgeEntry::new("a", "alpha gamma"))
        .unwrap();
    assert_eq!(
        signature(&index, "alpha"),
        signature(
            &Engine::from_entries(index.entries().cloned()).unwrap(),
            "alpha"
        )
    );
}

#[test]
fn cached_readers_and_diverging_clones_keep_their_own_statistics() {
    let original = Engine::from_entries(source()).unwrap();
    let expected = signature(&original, "alpha beta");
    let mut left = original.clone();
    let mut right = original.clone();
    left.upsert(KnowledgeEntry::new("new", "alpha alpha gamma"))
        .unwrap();
    right.remove("a");
    for index in [&original, &left, &right] {
        assert_eq!(
            signature(index, "alpha beta"),
            signature(
                &Engine::from_entries(index.entries().cloned()).unwrap(),
                "alpha beta"
            )
        );
    }
    assert_eq!(signature(&original, "alpha beta"), expected);
    left.clear();
    assert_eq!(signature(&original, "alpha beta"), expected);
}

#[test]
fn concurrent_first_and_warm_queries_have_identical_score_bits() {
    let index = Engine::from_entries((0..250).map(|i| {
        KnowledgeEntry::new(
            i.to_string(),
            format!("alpha beta {} gamma", "beta ".repeat(i % 11)),
        )
    }))
    .unwrap();
    let expected = signature(
        &Engine::from_entries(index.entries().cloned()).unwrap(),
        "alpha gamma",
    );
    std::thread::scope(|scope| {
        for _ in 0..8 {
            let index = &index;
            let expected = &expected;
            scope.spawn(move || {
                for _ in 0..20 {
                    assert_eq!(&signature(index, "alpha gamma"), expected);
                }
            });
        }
    });
}

#[test]
fn interning_preserves_original_unicode_ids_duplicates_and_roundtrip_bytes() {
    let entries = vec![
        KnowledgeEntry::new("原", "ＣＡＦÉ alpha"),
        KnowledgeEntry::new("a\nraw", "cafe\u{301} alpha"),
        KnowledgeEntry::new("a", "café alpha"),
    ];
    let mut index = Engine::from_entries(entries.clone()).unwrap();
    let before = index.to_json().unwrap();
    let _ = signature(&index, "café");
    assert_eq!(index.to_json().unwrap(), before);
    assert_eq!(index.duplicates()[0].len(), 3);
    index.remove("原");
    assert_eq!(index.duplicates()[0].len(), 2);
    assert_eq!(index.get("a\nraw").unwrap().text, entries[1].text);
    let restored = Engine::from_json(&before).unwrap();
    assert_eq!(restored.to_json().unwrap(), before);
    assert_eq!(
        signature(&restored, "café"),
        signature(&Engine::from_entries(entries).unwrap(), "café")
    );
}
