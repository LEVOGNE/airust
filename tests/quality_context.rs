#![cfg(all(
    feature = "quality",
    feature = "context-assembly",
    feature = "provenance"
))]

use airust::{
    context_engine::{ContextAssembler, ContextOptions},
    graph::{KnowledgeGraph, Relation, RelationType, TraversalDirection, TraversalOptions},
    provenance::{ProvenanceStatus, ProvenanceStore, SourceReference},
    quality::{evaluate_ranking, Judgments, UnjudgedPolicy},
    Context, Engine, KnowledgeEntry, SearchOptions,
};
use serde_json::json;
use std::collections::BTreeSet;
#[test]
fn reference_context_preserves_authorized_evidence_without_excluded_graph_bridges() {
    let engine = Engine::from_entries([
        KnowledgeEntry::new("seed", "Grüße 世界 Rust ownership proof")
            .with_metadata(json!({"scope":"allowed"})),
        KnowledgeEntry::new("duplicate", "GRÜSSE 世界 RUST ownership proof")
            .with_metadata(json!({"scope":"allowed"})),
        KnowledgeEntry::new("neighbor", "Borrowing is the connected reference note.")
            .with_metadata(json!({"scope":"allowed"})),
        KnowledgeEntry::new("hidden", "Private Rust ownership note")
            .with_metadata(json!({"scope":"denied"})),
        KnowledgeEntry::new("behind-hidden", "Allowed content behind an excluded bridge")
            .with_metadata(json!({"scope":"allowed"})),
    ])
    .unwrap();
    let mut graph = KnowledgeGraph::new();
    for entry in engine.entries() {
        graph.add_node(&entry.id).unwrap();
    }
    for (id, from, to) in [
        ("a", "seed", "neighbor"),
        ("b", "seed", "hidden"),
        ("c", "hidden", "behind-hidden"),
        ("d", "seed", "duplicate"),
    ] {
        graph
            .upsert_relation(Relation::new(id, from, to, RelationType::References).unwrap())
            .unwrap();
    }
    let mut sources = ProvenanceStore::new();
    for id in ["seed", "duplicate", "neighbor"] {
        sources
            .add(
                id,
                SourceReference::new(format!("source-{id}"))
                    .unwrap()
                    .with_document(format!("document-{id}"))
                    .unwrap()
                    .with_page(1)
                    .unwrap(),
            )
            .unwrap();
    }
    let options = ContextOptions::new(5, 512)
        .with_item_bytes(128)
        .with_scope(Context::new().with_metadata("scope", "allowed"))
        .with_traversal(TraversalOptions::new(3, 30).with_direction(TraversalDirection::Both));
    let assembler = ContextAssembler::new(&engine)
        .with_graph(&graph)
        .with_provenance(&sources);
    let bundle = assembler.assemble(["seed"], &options).unwrap();
    let ids: BTreeSet<_> = bundle
        .items()
        .iter()
        .flat_map(|item| item.references().iter().map(|r| r.id()))
        .collect();
    assert_eq!(ids, BTreeSet::from(["seed", "duplicate", "neighbor"]));
    assert!(bundle.used_text_bytes() <= 512);
    assert_eq!(bundle.node_count(), 3);
    for item in bundle.items() {
        let original = engine.get(item.references()[0].id()).unwrap();
        assert_eq!(
            item.text(),
            &original.text[item.start_byte()..item.end_byte()]
        );
        for reference in item.references() {
            assert_eq!(reference.provenance_status(), ProvenanceStatus::Referenced);
            assert_eq!(reference.sources().len(), 1);
        }
    }
    let judged = Judgments::new([("seed", 3), ("duplicate", 3), ("neighbor", 2)]).unwrap();
    let metrics = evaluate_ranking(
        &ids.iter().copied().collect::<Vec<_>>(),
        &judged,
        3,
        UnjudgedPolicy::RequireJudged,
    )
    .unwrap();
    assert_eq!(metrics.recall_at_k(), Some(1.0));
    let clipped = assembler
        .assemble(["seed"], &options.clone().with_item_bytes(9))
        .unwrap();
    for item in clipped.items() {
        assert!(item.text().len() <= 9);
        assert!(std::str::from_utf8(item.text().as_bytes()).is_ok());
    }
    let empty = assembler
        .query(
            "unmatched_identifier_ZXQ123",
            None,
            &options.with_lexical(SearchOptions {
                limit: 3,
                ..SearchOptions::default()
            }),
        )
        .unwrap();
    assert!(empty.items().is_empty());
}

#[test]
fn exact_normalized_duplicates_share_text_but_keep_distinct_source_references() {
    let engine = Engine::from_entries([
        KnowledgeEntry::new("a", "Grüße 世界"),
        KnowledgeEntry::new("b", "GRÜßE 世界"),
    ])
    .unwrap();
    let mut sources = ProvenanceStore::new();
    sources
        .add("a", SourceReference::new("doc-a").unwrap())
        .unwrap();
    sources
        .add("b", SourceReference::new("doc-b").unwrap())
        .unwrap();
    let bundle = ContextAssembler::new(&engine)
        .with_provenance(&sources)
        .assemble(["a", "b"], &ContextOptions::default())
        .unwrap();
    assert_eq!(bundle.items().len(), 1);
    assert_eq!(bundle.node_count(), 2);
    let refs = bundle.items()[0].references();
    assert_eq!(refs[0].sources()[0].id(), "doc-a");
    assert_eq!(refs[1].sources()[0].id(), "doc-b");
}
