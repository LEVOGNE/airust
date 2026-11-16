#![cfg(feature = "provenance")]

use airust::provenance::{
    CodePosition, CodeSpan, ProvenanceError, ProvenanceStatus, ProvenanceStore, SourceReference,
    TextSpan,
};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};

fn source(id: &str, page: usize) -> SourceReference {
    SourceReference::new(id)
        .unwrap()
        .with_document("manual-v1")
        .unwrap()
        .with_page(page)
        .unwrap()
}
fn ids(store: &ProvenanceStore, entry: &str) -> Vec<String> {
    store
        .sources_for(entry)
        .map(|s| s.id().to_string())
        .collect()
}

#[test]
fn minimal_and_missing_sources_never_invent_locations_or_truth() {
    let source = SourceReference::new("opaque-source-id").unwrap();
    assert!(
        source.document_id().is_none() && source.page().is_none() && source.text_span().is_none()
    );
    assert!(
        source.code_span().is_none()
            && source.file_path().is_none()
            && source.revision().is_none()
            && source.uri().is_none()
    );
    let store = ProvenanceStore::new();
    assert_eq!(store.status("absent"), ProvenanceStatus::Missing);
    assert!(store.sources_for("absent").next().is_none());
    assert!(store.is_empty());
}

#[test]
fn identity_page_and_opaque_uri_revision_validation_is_consistent() {
    for id in ["", " \t", "a\nb", "x\0y"] {
        assert!(SourceReference::new(id).is_err());
    }
    assert!(source("a", 1).with_page(0).is_err());
    assert!(source("a", 1).with_document("").is_err());
    assert!(source("a", 1).with_file("\n").is_err());
    assert!(source("a", 1).with_uri(" ").is_err());
    assert!(source("a", 1).with_revision("").is_err());
    let reference = source("a", 9000)
        .with_uri("urn:source:manual")
        .unwrap()
        .with_revision("application-revision-7")
        .unwrap()
        .with_metadata(json!({"review":"unknown"}));
    assert_eq!(reference.uri(), Some("urn:source:manual"));
    assert_eq!(reference.revision(), Some("application-revision-7"));
    assert_eq!(reference.metadata().unwrap()["review"], "unknown");
    assert!(reference.validate_source_text("").is_ok());
}

#[test]
fn text_spans_validate_exact_utf8_byte_boundaries_including_empty_positions() {
    let text = "Grüße 世界🙂";
    assert!(TextSpan::new(9, 8).is_err());
    for start in 0..text.len() + 3 {
        for end in start..text.len() + 3 {
            let expected =
                end <= text.len() && text.is_char_boundary(start) && text.is_char_boundary(end);
            assert_eq!(
                TextSpan::new(start, end).unwrap().validate_in(text).is_ok(),
                expected
            );
        }
    }
    assert!(TextSpan::new(usize::MAX, usize::MAX)
        .unwrap()
        .validate_in(text)
        .is_err());
    assert!(TextSpan::new(text.len(), text.len())
        .unwrap()
        .validate_in(text)
        .is_ok());
}

#[test]
fn code_positions_use_one_based_lines_and_utf8_byte_columns_with_crlf() {
    let text = "αx\r\n世界\n";
    for position in [(0, 1), (1, 0)] {
        assert!(CodePosition::new(position.0, position.1).is_err());
    }
    for (line, column, valid) in [
        (1, 1, true),
        (1, 2, false),
        (1, 3, true),
        (1, 4, true),
        (1, 5, false),
        (2, 1, true),
        (2, 2, false),
        (2, 4, true),
        (2, 7, true),
        (3, 1, true),
        (3, 2, false),
        (4, 1, false),
    ] {
        assert_eq!(
            CodePosition::new(line, column)
                .unwrap()
                .validate_in(text)
                .is_ok(),
            valid
        );
    }
    let span = CodeSpan::new(
        CodePosition::new(1, 3).unwrap(),
        CodePosition::new(2, 7).unwrap(),
    )
    .unwrap();
    assert!(span.validate_in(text).is_ok());
    assert_eq!(span.start().line(), 1);
    assert_eq!(span.end().byte_column(), 7);
    assert!(CodeSpan::new(span.end(), span.start()).is_err());
    assert!(CodePosition::new(usize::MAX, 1)
        .unwrap()
        .validate_in(text)
        .is_err());
}

#[test]
fn serde_cannot_bypass_span_position_page_or_reference_validation() {
    for value in [
        json!({"start_byte":7,"end_byte":1}),
        json!({"start_byte":0,"end_byte":1,"unknown":true}),
    ] {
        assert!(serde_json::from_value::<TextSpan>(value).is_err());
    }
    assert!(serde_json::from_value::<CodePosition>(json!({"line":0,"byte_column":1})).is_err());
    for value in [
        json!({"id":""}),
        json!({"id":"a","page":0}),
        json!({"id":"a","document_id":""}),
        json!({"id":"a","unknown":true}),
        json!({"id":"a","code_span":{"start":{"line":1,"byte_column":1},"end":{"line":1,"byte_column":2}}}),
        json!({"id":"a","file_path":"a.rs","code_span":{"start":{"line":2,"byte_column":1},"end":{"line":1,"byte_column":1}}}),
    ] {
        assert!(serde_json::from_value::<SourceReference>(value).is_err());
    }
}

#[test]
fn a_reference_checks_declared_positions_against_supplied_source_only() {
    let good = SourceReference::new("code")
        .unwrap()
        .with_code(
            "src/main.rs",
            CodeSpan::new(
                CodePosition::new(1, 1).unwrap(),
                CodePosition::new(1, 3).unwrap(),
            )
            .unwrap(),
        )
        .unwrap()
        .with_text_span(TextSpan::new(0, 2).unwrap());
    assert!(good.validate_source_text("αx").is_ok());
    assert!(good.validate_source_text("x").is_err());
    assert_eq!(good.file_path(), Some("src/main.rs"));
    assert!(SourceReference::new("bad")
        .unwrap()
        .with_text_span(TextSpan::new(1, 2).unwrap())
        .validate_source_text("α")
        .is_err());
}

#[test]
fn shared_sources_are_interned_and_multiple_references_have_stable_order() {
    let mut store = ProvenanceStore::new();
    assert!(store.add("a", source("z", 2)).unwrap());
    assert!(!store.add("a", source("z", 2)).unwrap());
    store.add("a", source("b", 1)).unwrap();
    store.add("c", source("z", 2)).unwrap();
    assert_eq!(store.source_count(), 2);
    assert_eq!(store.entry_count(), 2);
    assert_eq!(ids(&store, "a"), ["b", "z"]);
    assert!(std::ptr::eq(
        store.source("z").unwrap(),
        store.sources_for("c").next().unwrap()
    ));
    assert_eq!(store.status("a"), ProvenanceStatus::Referenced);
}

#[test]
fn registered_sources_can_be_bound_without_cloning_and_unknown_bindings_fail_atomically() {
    let mut store = ProvenanceStore::new();
    assert!(store.insert_source(source("r", 1)).unwrap());
    assert!(!store.insert_source(source("r", 1)).unwrap());
    assert!(store.is_empty());
    assert_eq!(store.source_count(), 1);
    assert!(store.bind("a", "r").unwrap());
    assert!(!store.bind("a", "r").unwrap());
    let before = store.clone();
    assert!(store.bind("b", "missing").is_err());
    assert!(store.bind(" ", "r").is_err());
    assert_eq!(store, before);
    assert!(store.insert_source(source("r", 2)).is_err());
    assert_eq!(store, before);
}

#[test]
fn conflicting_add_and_late_replacement_errors_leave_every_binding_unchanged() {
    let mut store = ProvenanceStore::new();
    store.add("a", source("original", 1)).unwrap();
    let before = store.clone();
    assert!(matches!(
        store.add("a", source("original", 2)),
        Err(ProvenanceError::Conflict(_))
    ));
    assert_eq!(store, before);
    assert!(store
        .replace("a", [source("new", 1), source("original", 2)])
        .is_err());
    assert_eq!(store, before);
    assert!(store
        .replace("a", [source("new", 1), source("new", 2)])
        .is_err());
    assert_eq!(store, before);
    assert!(store.add(" ", source("new", 1)).is_err());
    assert_eq!(store, before);
}

#[test]
fn entry_merges_preserve_all_original_sources_and_missing_entries_add_nothing() {
    let mut store = ProvenanceStore::new();
    store.add("a", source("one", 1)).unwrap();
    store.add("b", source("two", 2)).unwrap();
    store.add("a", source("shared", 3)).unwrap();
    store.add("b", source("shared", 3)).unwrap();
    assert_eq!(
        store
            .merge_entries("merged", ["a", "b", "missing"])
            .unwrap(),
        3
    );
    assert_eq!(ids(&store, "merged"), ["one", "shared", "two"]);
    assert_eq!(ids(&store, "a"), ["one", "shared"]);
    assert_eq!(store.merge_entries("merged", ["a", "b"]).unwrap(), 0);
    let before = store.clone();
    assert!(store.merge_entries("new", ["a", ""]).is_err());
    assert_eq!(store, before);
    assert_eq!(store.merge_entries("unknown", ["missing"]).unwrap(), 0);
    assert_eq!(store.status("unknown"), ProvenanceStatus::Missing);
}

#[test]
fn merging_stores_is_atomic_even_when_an_earlier_new_source_precedes_a_conflict() {
    let mut store = ProvenanceStore::new();
    store.add("a", source("z", 1)).unwrap();
    let mut other = ProvenanceStore::new();
    other.add("b", source("a-new", 1)).unwrap();
    other.add("b", source("z", 2)).unwrap();
    let before = store.clone();
    assert!(store.merge(&other).is_err());
    assert_eq!(store, before);
    other.replace("b", [source("a-new", 1)]).unwrap();
    other.prune_unreferenced();
    store.merge(&other).unwrap();
    assert_eq!(store.source_count(), 2);
    assert_eq!(ids(&store, "b"), ["a-new"]);
}

#[test]
fn source_removal_pruning_replacement_and_clear_preserve_referential_integrity() {
    let mut store = ProvenanceStore::new();
    store.add("a", source("one", 1)).unwrap();
    store.add("b", source("one", 1)).unwrap();
    store.add("a", source("two", 2)).unwrap();
    assert!(store.remove_source("one"));
    assert!(!store.remove_source("one"));
    assert_eq!(ids(&store, "a"), ["two"]);
    assert_eq!(store.status("b"), ProvenanceStatus::Missing);
    assert!(store.remove_entry("a"));
    assert!(!store.remove_entry("a"));
    assert!(store.is_empty());
    assert_eq!(store.source_count(), 1);
    assert_eq!(store.prune_unreferenced(), 1);
    store.add("a", source("two", 2)).unwrap();
    assert_eq!(store.replace("a", []).unwrap(), 0);
    assert!(store.is_empty());
    assert_eq!(store.source_count(), 1);
    store.clear();
    assert_eq!(store.source_count(), 0);
}

#[test]
fn versioned_json_and_generic_serde_round_trip_without_changing_source_definitions() {
    let mut store = ProvenanceStore::new();
    store
        .add("b", source("one", 1).with_revision("rev-1").unwrap())
        .unwrap();
    store.add("a", source("two", 2)).unwrap();
    let encoded = store.to_json().unwrap();
    assert_eq!(ProvenanceStore::from_json(&encoded).unwrap(), store);
    assert_eq!(
        serde_json::from_str::<ProvenanceStore>(&encoded).unwrap(),
        store
    );
    let mut reversed = ProvenanceStore::new();
    reversed.add("a", source("two", 2)).unwrap();
    reversed
        .add("b", source("one", 1).with_revision("rev-1").unwrap())
        .unwrap();
    assert_eq!(encoded, reversed.to_json().unwrap());
}

#[test]
fn malformed_imports_duplicate_ids_and_dangling_bindings_are_rejected() {
    let base =
        json!({"version":1,"sources":[{"id":"r"}],"entries":[{"entry_id":"a","source_ids":["r"]}]});
    for mutate in 0..7 {
        let mut value = base.clone();
        match mutate {
            0 => value["version"] = json!(99),
            1 => value["sources"] = json!([{"id":"r"},{"id":"r"}]),
            2 => {
                value["entries"] =
                    json!([{"entry_id":"a","source_ids":["r"]},{"entry_id":"a","source_ids":["r"]}])
            }
            3 => value["entries"][0]["source_ids"] = json!(["missing"]),
            4 => value["entries"][0]["source_ids"] = json!(["r", "r"]),
            5 => value["entries"][0]["entry_id"] = json!(" "),
            _ => value["extra"] = json!(true),
        }
        assert!(ProvenanceStore::from_json(&value.to_string()).is_err());
        assert!(serde_json::from_value::<ProvenanceStore>(value).is_err());
    }
    assert!(matches!(
        ProvenanceStore::from_json(&json!({"version":9,"sources":[],"entries":[]}).to_string()),
        Err(ProvenanceError::UnsupportedVersion(9))
    ));
    let empty = json!({"version":1,"sources":[],"entries":[{"entry_id":"a","source_ids":[]}]});
    assert_eq!(
        ProvenanceStore::from_json(&empty.to_string())
            .unwrap()
            .status("a"),
        ProvenanceStatus::Missing
    );
}

#[test]
fn explicit_json_byte_limits_are_checked_before_parsing() {
    assert!(matches!(
        ProvenanceStore::from_json_with_limit("not json", 0),
        Err(ProvenanceError::InputTooLarge)
    ));
    let text = ProvenanceStore::new().to_json().unwrap();
    assert!(ProvenanceStore::from_json_with_limit(&text, text.len()).is_ok());
    assert!(ProvenanceStore::from_json_with_limit(&text, text.len() - 1).is_err());
}

#[test]
fn mutation_sequence_matches_an_independent_binding_reference_and_round_trips() {
    let mut store = ProvenanceStore::new();
    let mut bindings = BTreeMap::<String, BTreeSet<String>>::new();
    let mut known = BTreeSet::new();
    for step in 0..600 {
        let entry = format!("entry-{}", step * 17 % 31);
        let reference = format!("ref-{}", step * 13 % 47);
        match step % 5 {
            0 | 1 => {
                store.add(&entry, source(&reference, 1)).unwrap();
                known.insert(reference.clone());
                bindings.entry(entry.clone()).or_default().insert(reference);
            }
            2 => {
                store.remove_entry(&entry);
                bindings.remove(&entry);
            }
            3 => {
                store.remove_source(&reference);
                known.remove(&reference);
                bindings.retain(|_, ids| {
                    ids.remove(&reference);
                    !ids.is_empty()
                });
            }
            _ => {
                store.replace(&entry, [source(&reference, 1)]).unwrap();
                known.insert(reference.clone());
                bindings.insert(entry.clone(), BTreeSet::from([reference]));
            }
        }
        assert_eq!(store.source_count(), known.len());
        assert_eq!(store.entry_count(), bindings.len());
        for n in 0..31 {
            let entry = format!("entry-{n}");
            assert_eq!(
                ids(&store, &entry),
                bindings
                    .get(&entry)
                    .into_iter()
                    .flat_map(|v| v.iter().cloned())
                    .collect::<Vec<_>>()
            );
        }
        if step % 50 == 0 {
            assert_eq!(
                ProvenanceStore::from_json(&store.to_json().unwrap()).unwrap(),
                store
            );
        }
    }
}

#[cfg(feature = "context-assembly")]
#[test]
fn context_duplicate_sources_remain_distinct_missing_and_omitted_are_explicit() {
    use airust::{
        context_engine::{ContextAssembler, ContextOptions},
        Engine, KnowledgeEntry,
    };
    let index = Engine::from_entries([
        KnowledgeEntry::new("a", "Rust source"),
        KnowledgeEntry::new("b", "RUST SOURCE"),
        KnowledgeEntry::new("c", "no known provenance"),
    ])
    .unwrap();
    let before = index.to_json().unwrap();
    let mut sources = ProvenanceStore::new();
    sources.add("a", source("page1", 1)).unwrap();
    sources.add("b", source("page2", 2)).unwrap();
    let context = ContextAssembler::new(&index)
        .with_provenance(&sources)
        .assemble(["a", "b", "c"], &ContextOptions::default())
        .unwrap();
    assert_eq!(context.items().len(), 2);
    assert_eq!(context.items()[0].references().len(), 2);
    let refs = context.items()[0].references();
    assert_eq!(refs[0].sources()[0].page(), Some(1));
    assert_eq!(refs[1].sources()[0].page(), Some(2));
    assert!(std::ptr::eq(
        refs[0].sources()[0],
        sources.source("page1").unwrap()
    ));
    assert_eq!(
        context.items()[1].references()[0].provenance_status(),
        ProvenanceStatus::Missing
    );
    assert!(context.items()[1].references()[0].sources().is_empty());
    let omitted = ContextAssembler::new(&index)
        .with_provenance(&sources)
        .assemble(
            ["a", "c"],
            &ContextOptions::default().with_source_references(false),
        )
        .unwrap();
    assert!(omitted
        .items()
        .iter()
        .flat_map(|i| i.references())
        .all(|r| r.provenance_status() == ProvenanceStatus::Omitted && r.sources().is_empty()));
    assert_eq!(before, index.to_json().unwrap());
}

#[cfg(feature = "context-assembly")]
#[test]
fn context_never_extrapolates_external_source_offsets_from_knowledge_excerpts() {
    use airust::{
        context_engine::{ContextAssembler, ContextOptions},
        Context, Engine, KnowledgeEntry,
    };
    let index = Engine::from_entries([
        KnowledgeEntry::new(
            "a",
            format!("{}target source passage", "prefix ".repeat(200)),
        ),
        KnowledgeEntry::new("hidden", "private"),
    ])
    .unwrap();
    let mut sources = ProvenanceStore::new();
    sources
        .add(
            "a",
            source("r", 3).with_text_span(TextSpan::new(1000, 1100).unwrap()),
        )
        .unwrap();
    sources.add("hidden", source("private-source", 9)).unwrap();
    let options = ContextOptions::new(2, 32).with_scope(Context::new().with_entry("a"));
    let context = ContextAssembler::new(&index)
        .with_provenance(&sources)
        .query("target", None, &options)
        .unwrap();
    let item = &context.items()[0];
    assert!(item.start_byte() > 1000);
    assert_eq!(
        item.references()[0].sources()[0].text_span().unwrap(),
        TextSpan::new(1000, 1100).unwrap()
    );
    assert!(!serde_json::to_string(&context)
        .unwrap()
        .contains("private-source"));
}
