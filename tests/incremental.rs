#![cfg(all(feature = "snapshots", feature = "advanced-search"))]
use airust::{provenance::SourceReference, snapshots::*, text_search::*, KnowledgeEntry};
use std::collections::BTreeMap;
fn manager() -> DatasetManager {
    DatasetManager::new(
        "incremental-test",
        DatasetConfig::default().with_text_analyzer(AnalyzerConfig::german()),
    )
    .unwrap()
}
fn record(id: &str, text: &str, version: usize) -> DatasetRecord {
    DatasetRecord::new(
        KnowledgeEntry::new(id, text).with_metadata(serde_json::json!({"version":version})),
    )
}
#[test]
fn metadata_source_and_model_only_changes_reuse_analysis_and_keep_old_views() {
    let manager = manager();
    let empty = manager.snapshot().unwrap();
    let first = manager
        .publish(
            manager
                .prepare_replace(
                    empty.revision(),
                    vec![
                        record("a", "Häuser haben Türen", 1),
                        record("b", "Rust ownership", 1),
                    ],
                    None,
                    None,
                )
                .unwrap(),
        )
        .unwrap();
    let next = manager
        .publish(
            manager
                .prepare(
                    first.revision(),
                    DatasetUpdate::new(vec![DatasetChange::Put(record(
                        "a",
                        "Häuser haben Türen",
                        2,
                    ))]),
                    None,
                )
                .unwrap(),
        )
        .unwrap();
    assert_eq!(next.stats().text_entries_rebuilt, 0);
    assert_eq!(next.stats().text_entries_reused, 2);
    assert_eq!(next.stats().metadata_only_puts, 1);
    assert_eq!(
        next.text_index().unwrap().get("a").unwrap().metadata,
        Some(serde_json::json!({"version":2}))
    );
    assert_eq!(
        first.text_index().unwrap().get("a").unwrap().metadata,
        Some(serde_json::json!({"version":1}))
    );
    let source = record("a", "Häuser haben Türen", 2)
        .with_sources(vec![SourceReference::new("manual").unwrap()]);
    let third = manager
        .publish(
            manager
                .prepare(
                    next.revision(),
                    DatasetUpdate::new(vec![DatasetChange::Put(source)]),
                    None,
                )
                .unwrap(),
        )
        .unwrap();
    assert!(std::ptr::eq(
        next.text_index().unwrap(),
        third.text_index().unwrap()
    ));
    assert_eq!(third.stats().unchanged_puts, 1);
    assert!(next.provenance().source("manual").is_none());
    assert!(third.provenance().source("manual").is_some());
    let model = manager
        .publish(manager.prepare_model(third.revision(), None).unwrap())
        .unwrap();
    assert!(std::ptr::eq(
        model.text_index().unwrap(),
        third.text_index().unwrap()
    ));
    assert_eq!(model.stats().text_entries_reused, 2);
}
#[test]
fn normalized_equal_original_changes_recompute_positions_instead_of_reusing_stale_spans() {
    let manager = manager();
    let empty = manager.snapshot().unwrap();
    let first = manager
        .publish(
            manager
                .prepare_replace(
                    empty.revision(),
                    vec![record("a", "ＡＩ Ｒｕｓｔ", 1)],
                    None,
                    None,
                )
                .unwrap(),
        )
        .unwrap();
    let next = manager
        .publish(
            manager
                .prepare(
                    first.revision(),
                    DatasetUpdate::new(vec![DatasetChange::Put(record("a", "AI Rust", 1))]),
                    None,
                )
                .unwrap(),
        )
        .unwrap();
    assert_eq!(next.stats().text_entries_rebuilt, 1);
    assert_eq!(next.stats().text_entries_reused, 0);
    let options = TextSearchOptions::new(1);
    let before = first
        .text_index()
        .unwrap()
        .search("\"AI Rust\"", &options)
        .unwrap();
    let after = next
        .text_index()
        .unwrap()
        .search("\"AI Rust\"", &options)
        .unwrap();
    assert_eq!(before[0].phrases[0].spans[0].end, 19);
    assert_eq!(after[0].phrases[0].spans[0].end, 7);
    assert_eq!(after[0].entry.text, "AI Rust");
}
#[test]
fn signed_zero_metadata_is_updated_in_every_projection_without_text_reanalysis() {
    let manager = manager();
    let empty = manager.snapshot().unwrap();
    let negative: serde_json::Value = serde_json::from_str("-0.0").unwrap();
    let positive: serde_json::Value = serde_json::from_str("0.0").unwrap();
    let first = manager
        .publish(
            manager
                .prepare_replace(
                    empty.revision(),
                    vec![DatasetRecord::new(
                        KnowledgeEntry::new("a", "Rust").with_metadata(negative),
                    )],
                    None,
                    None,
                )
                .unwrap(),
        )
        .unwrap();
    let next = manager
        .publish(
            manager
                .prepare(
                    first.revision(),
                    DatasetUpdate::new(vec![DatasetChange::Put(DatasetRecord::new(
                        KnowledgeEntry::new("a", "Rust").with_metadata(positive),
                    ))]),
                    None,
                )
                .unwrap(),
        )
        .unwrap();
    assert_eq!(next.stats().metadata_only_puts, 1);
    assert_eq!(next.stats().unchanged_puts, 0);
    assert_eq!(next.stats().text_entries_rebuilt, 0);
    for entry in [
        next.lexical().get("a").unwrap(),
        next.text_index().unwrap().get("a").unwrap(),
    ] {
        assert_eq!(entry.metadata.as_ref().unwrap().to_string(), "0.0");
    }
    assert_eq!(
        first
            .lexical()
            .get("a")
            .unwrap()
            .metadata
            .as_ref()
            .unwrap()
            .to_string(),
        "-0.0"
    );
}
#[test]
fn seeded_incremental_updates_match_complete_rebuilds_with_metadata_and_phrases() {
    let manager = manager();
    let mut current = manager.snapshot().unwrap();
    let mut authoritative = BTreeMap::<String, KnowledgeEntry>::new();
    let mut seed = 79u64;
    let mut retained = Vec::new();
    for step in 0usize..600 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let id = format!("id{}", seed % 64);
        let before = authoritative.get(&id).cloned();
        let mut expected_rebuilt = 0;
        let change = if seed.is_multiple_of(5) {
            authoritative.remove(&id);
            DatasetChange::Delete(id.clone())
        } else {
            let text = if seed.is_multiple_of(3) {
                before
                    .as_ref()
                    .map(|e| e.text.clone())
                    .unwrap_or_else(|| "Häuser haben Türen".into())
            } else {
                format!("Häuser haben Türen Rust topic{}", seed % 17)
            };
            expected_rebuilt = usize::from(before.as_ref().is_none_or(|e| e.text != text));
            let entry =
                KnowledgeEntry::new(&id, text).with_metadata(serde_json::json!({"step":step}));
            authoritative.insert(id.clone(), entry.clone());
            DatasetChange::Put(DatasetRecord::new(entry))
        };
        if step.is_multiple_of(100) {
            retained.push((current.clone(), current.lexical().to_json().unwrap()));
        }
        current = manager
            .publish(
                manager
                    .prepare(current.revision(), DatasetUpdate::new(vec![change]), None)
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(current.stats().text_entries_rebuilt, expected_rebuilt);
        assert_eq!(
            current.stats().text_entries_reused,
            authoritative.len() - expected_rebuilt
        );
        let rebuilt =
            TextIndex::from_entries(AnalyzerConfig::german(), authoritative.values().cloned())
                .unwrap();
        assert_eq!(current.text_index().unwrap().stats(), rebuilt.stats());
        for query in [
            "Haus",
            "Rust topic3",
            "\"Haus haben\"",
            "topic16",
            "missing",
        ] {
            let options = TextSearchOptions::new(10);
            assert_eq!(
                serde_json::to_string(
                    &current
                        .text_index()
                        .unwrap()
                        .search(query, &options)
                        .unwrap()
                )
                .unwrap(),
                serde_json::to_string(&rebuilt.search(query, &options).unwrap()).unwrap()
            );
        }
    }
    for (view, expected) in retained {
        assert_eq!(view.lexical().to_json().unwrap(), expected);
    }
}
#[test]
fn full_replacement_reconciles_removal_and_metadata_without_reanalyzing_unchanged_text() {
    let manager = manager();
    let empty = manager.snapshot().unwrap();
    let first = manager
        .publish(
            manager
                .prepare_replace(
                    empty.revision(),
                    vec![
                        record("a", "Rust ownership", 1),
                        record("b", "Häuser Türen", 1),
                    ],
                    None,
                    None,
                )
                .unwrap(),
        )
        .unwrap();
    let next = manager
        .publish(
            manager
                .prepare_replace(
                    first.revision(),
                    vec![record("a", "Rust ownership", 2)],
                    None,
                    None,
                )
                .unwrap(),
        )
        .unwrap();
    assert_eq!(next.stats().text_entries_rebuilt, 0);
    assert_eq!(next.stats().text_entries_reused, 1);
    assert!(next.text_index().unwrap().get("b").is_none());
    assert!(next
        .text_index()
        .unwrap()
        .search("Haus", &TextSearchOptions::new(3))
        .unwrap()
        .is_empty());
    assert!(first.text_index().unwrap().get("b").is_some());
}
#[test]
fn failed_changed_text_analysis_cannot_corrupt_a_published_projection() {
    let manager = DatasetManager::new(
        "analysis-limit",
        DatasetConfig::default()
            .with_text_analyzer(AnalyzerConfig::german().with_limits(1024, 4096, 3).unwrap()),
    )
    .unwrap();
    let empty = manager.snapshot().unwrap();
    let first = manager
        .publish(
            manager
                .prepare_replace(
                    empty.revision(),
                    vec![record("a", "Häuser haben Türen", 1)],
                    None,
                    None,
                )
                .unwrap(),
        )
        .unwrap();
    let result = manager.prepare(
        first.revision(),
        DatasetUpdate::new(vec![DatasetChange::Put(record(
            "a",
            "Häuser haben sehr viele Türen",
            2,
        ))]),
        None,
    );
    assert!(matches!(
        result,
        Err(DatasetError::Text(TextSearchError::Limit(_)))
    ));
    assert_eq!(manager.snapshot().unwrap().revision(), first.revision());
    assert_eq!(
        first.text_index().unwrap().get("a").unwrap().text,
        "Häuser haben Türen"
    );
}
#[cfg(feature = "execution")]
#[test]
fn cancellation_from_incremental_analysis_discards_the_prepared_change() {
    use airust::execution::*;
    let manager = manager();
    let empty = manager.snapshot().unwrap();
    let first = manager
        .publish(
            manager
                .prepare_replace(
                    empty.revision(),
                    vec![record("a", "Häuser Türen", 1)],
                    None,
                    None,
                )
                .unwrap(),
        )
        .unwrap();
    let token = CancellationToken::new();
    let cancel = token.clone();
    let control = WorkControl::new(token).with_observer(move |event: &ProgressEvent| {
        if event.phase() == WorkPhase::AnalyzingText {
            cancel.cancel();
        }
    });
    let result = manager.prepare_controlled(
        first.revision(),
        DatasetUpdate::new(vec![DatasetChange::Put(record(
            "a",
            "Rust changed text",
            2,
        ))]),
        None,
        &control,
    );
    assert!(matches!(
        result,
        Err(DatasetError::Execution(ExecutionError::Cancelled))
    ));
    assert_eq!(manager.snapshot().unwrap().revision(), first.revision());
    assert_eq!(
        manager
            .snapshot()
            .unwrap()
            .text_index()
            .unwrap()
            .get("a")
            .unwrap()
            .text,
        "Häuser Türen"
    );
}
