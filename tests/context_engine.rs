#![cfg(feature = "context-assembly")]

use airust::{
    context_engine::{
        ContextAssembler, ContextError, ContextMode, ContextOptions, ContextOriginKind,
    },
    graph::{KnowledgeGraph, Relation, RelationType, TraversalDirection, TraversalOptions},
    hybrid::{HybridSearchOptions, SemanticFailurePolicy},
    semantic::{
        Embedding, EmbeddingError, EmbeddingProvider, EmbeddingPurpose, EmbeddingSpace,
        SemanticIndex, SemanticSearchOptions,
    },
    Context, Engine, KnowledgeEntry, SearchOptions,
};
use serde_json::json;

fn index(pairs: &[(&str, &str)]) -> Engine {
    let mut index = Engine::new();
    for (id, text) in pairs {
        index.upsert(KnowledgeEntry::new(*id, *text)).unwrap();
    }
    index
}
fn graph(ids: &[&str], edges: &[(&str, &str)]) -> KnowledgeGraph {
    let mut graph = KnowledgeGraph::new();
    for id in ids {
        graph.add_node(*id).unwrap();
    }
    for (n, (a, b)) in edges.iter().enumerate() {
        graph
            .upsert_relation(
                Relation::new(format!("e{n}"), *a, *b, RelationType::References).unwrap(),
            )
            .unwrap();
    }
    graph
}
fn ids<'a>(bundle: &'a airust::context_engine::ContextBundle<'a>) -> Vec<&'a str> {
    bundle
        .items()
        .iter()
        .flat_map(|item| item.references().iter().map(|r| r.id()))
        .collect()
}

#[test]
fn search_seeds_have_priority_and_graph_neighbors_preserve_path_evidence() {
    let index = index(&[
        ("a", "Rust memory safety"),
        ("b", "Rust borrow checking"),
        ("c", "Ownership details"),
    ]);
    let graph = graph(&["a", "b", "c"], &[("a", "c")]);
    let assembler = ContextAssembler::new(&index).with_graph(&graph);
    let bundle = assembler
        .query("Rust", None, &ContextOptions::new(3, 1000))
        .unwrap();
    assert_eq!(ids(&bundle), ["a", "b", "c"]);
    let origin = bundle.items()[2].references()[0].origin();
    assert_eq!(origin.kind(), ContextOriginKind::Graph);
    assert_eq!(origin.seed_id(), Some("a"));
    assert_eq!(origin.parent_id(), Some("a"));
    assert_eq!(origin.relation().unwrap().id(), "e0");
    assert_eq!(origin.depth(), 1);
    assert_eq!(
        bundle.items()[0].references()[0].origin().mode(),
        Some(ContextMode::Lexical)
    );
    let tight = assembler
        .query("Rust", None, &ContextOptions::new(2, 1000))
        .unwrap();
    assert_eq!(ids(&tight), ["a", "b"]);
}

#[test]
fn duplicate_content_retains_ids_metadata_and_respects_the_source_node_budget() {
    let mut index = index(&[
        ("a", "  CAFÉ   Rust  "),
        ("b", "cafe\u{301} rust"),
        ("c", "other"),
    ]);
    index
        .upsert(KnowledgeEntry::new("b", "cafe\u{301} rust").with_metadata(json!({"document":2})))
        .unwrap();
    let options = ContextOptions::new(2, 100);
    let bundle = ContextAssembler::new(&index)
        .assemble(["a", "a", "b", "c"], &options)
        .unwrap();
    assert_eq!(bundle.node_count(), 2);
    assert_eq!(bundle.items().len(), 1);
    assert_eq!(ids(&bundle), ["a", "b"]);
    assert_eq!(
        bundle.items()[0].references()[1].metadata().unwrap()["document"],
        2
    );
    assert_eq!(bundle.used_text_bytes(), index.get("a").unwrap().text.len());
    let separate = ContextAssembler::new(&index)
        .assemble(
            ["a", "b"],
            &options
                .clone()
                .with_deduplication(false)
                .with_metadata(false),
        )
        .unwrap();
    assert_eq!(separate.items().len(), 2);
    assert!(separate
        .items()
        .iter()
        .all(|item| item.references()[0].metadata().is_none()));
}

#[test]
fn text_budget_counts_only_borrowed_utf8_excerpts_and_never_merges_shared_prefixes() {
    let index = index(&[
        ("a", "Grüße 世界 Rust alpha"),
        ("b", "Grüße 世界 Rust beta"),
        ("c", "🙂🙂🙂🙂"),
    ]);
    for max_nodes in 0..6 {
        for bytes in 0..60 {
            for item_bytes in [0, 1, 2, 3, 4, 7, 15, 100] {
                let options = ContextOptions::new(max_nodes, bytes).with_item_bytes(item_bytes);
                let bundle = ContextAssembler::new(&index)
                    .assemble(["a", "b", "c", "a"], &options)
                    .unwrap();
                assert!(bundle.node_count() <= max_nodes);
                assert!(bundle.used_text_bytes() <= bytes);
                assert_eq!(
                    bundle.used_text_bytes(),
                    bundle.items().iter().map(|i| i.text().len()).sum::<usize>()
                );
                for item in bundle.items() {
                    assert!(item.text().len() <= item_bytes);
                    let original = &index.get(item.references()[0].id()).unwrap().text;
                    assert_eq!(item.text(), &original[item.start_byte()..item.end_byte()]);
                    assert_eq!(item.references().len(), 1);
                }
            }
        }
    }
}

#[test]
fn query_excerpts_find_deep_terms_and_keep_original_byte_offsets() {
    let text = format!("{} café TARGET information 世界", "unrelated ".repeat(1000));
    let index = index(&[("a", &text)]);
    let bundle = ContextAssembler::new(&index)
        .query("target", None, &ContextOptions::new(1, 32))
        .unwrap();
    let item = &bundle.items()[0];
    assert!(item.text().contains("TARGET"));
    assert!(item.start_byte() > 9000);
    assert_eq!(item.text(), &text[item.start_byte()..item.end_byte()]);
    assert!(item.truncated());
    let tiny = ContextAssembler::new(&index)
        .query("target", None, &ContextOptions::new(1, 1))
        .unwrap();
    assert_eq!(tiny.items()[0].text(), "T");
}

#[test]
fn skipped_multibyte_candidate_does_not_consume_node_budget_and_ids_stream_lazily() {
    let index = index(&[("wide", "世界"), ("a", "a")]);
    let bundle = ContextAssembler::new(&index)
        .assemble(["wide", "a"], &ContextOptions::new(1, 1))
        .unwrap();
    assert_eq!(ids(&bundle), ["a"]);
    assert_eq!(bundle.skipped_budget(), 1);
    let input = ["a"]
        .into_iter()
        .chain(std::iter::once_with(|| panic!("must stop at node budget")));
    let bundle = ContextAssembler::new(&index)
        .assemble(input, &ContextOptions::new(1, 50))
        .unwrap();
    assert_eq!(bundle.node_count(), 1);
}

#[test]
fn global_and_ranker_scopes_intersect_and_excluded_entries_are_not_graph_bridges() {
    let index = index(&[
        ("a", "Rust"),
        ("hidden", "private"),
        ("c", "related"),
        ("missing-bridge-target", "not reached"),
    ]);
    let graph = graph(
        &["a", "hidden", "c", "absent", "missing-bridge-target"],
        &[
            ("a", "hidden"),
            ("hidden", "c"),
            ("a", "absent"),
            ("absent", "missing-bridge-target"),
        ],
    );
    let assembler = ContextAssembler::new(&index).with_graph(&graph);
    let options = ContextOptions::new(10, 1000)
        .with_scope(
            Context::new()
                .with_entry("a")
                .with_entry("c")
                .with_entry("missing-bridge-target"),
        )
        .with_traversal(TraversalOptions::new(5, 100));
    let bundle = assembler.query("Rust", None, &options).unwrap();
    assert_eq!(ids(&bundle), ["a"]);
    assert_eq!(bundle.graph_visits(), 0);
    let disjoint = options.with_lexical(SearchOptions {
        context: Some(Context::new().with_entry("hidden")),
        ..SearchOptions::default()
    });
    assert!(assembler
        .query("Rust", None, &disjoint)
        .unwrap()
        .items()
        .is_empty());
}

#[test]
fn processed_missing_ids_scope_and_original_ranks_are_visible() {
    let index = index(&[("a", "visible"), ("b", "hidden")]);
    let bundle = ContextAssembler::new(&index)
        .assemble(
            ["absent", "b", "a"],
            &ContextOptions::default().with_scope(Context::new().with_entry("a")),
        )
        .unwrap();
    assert_eq!(bundle.skipped_missing(), 1);
    assert_eq!(bundle.skipped_scope(), 1);
    assert_eq!(bundle.items()[0].references()[0].origin().rank(), Some(3));
    assert_eq!(
        bundle.items()[0].references()[0].origin().kind(),
        ContextOriginKind::Provided
    );
}

#[test]
fn total_graph_visit_budget_is_shared_across_seeds_and_relation_filters_survive() {
    let index = index(&[
        ("a", "seed one"),
        ("b", "seed two"),
        ("c", "neighbor one"),
        ("d", "neighbor two"),
    ]);
    let graph = graph(&["a", "b", "c", "d"], &[("a", "c"), ("b", "d")]);
    let options = ContextOptions::new(10, 1000).with_traversal(
        TraversalOptions::new(3, 1)
            .with_direction(TraversalDirection::Both)
            .with_start(true),
    );
    let bundle = ContextAssembler::new(&index)
        .with_graph(&graph)
        .assemble(["a", "b"], &options)
        .unwrap();
    assert_eq!(bundle.graph_visits(), 1);
    assert_eq!(ids(&bundle), ["a", "b", "c"]);
    let options = options.with_traversal(
        TraversalOptions::new(3, 100).with_relation_types([RelationType::DependsOn]),
    );
    let bundle = ContextAssembler::new(&index)
        .with_graph(&graph)
        .assemble(["a"], &options)
        .unwrap();
    assert_eq!(ids(&bundle), ["a"]);
}

struct Provider {
    space: EmbeddingSpace,
    fail: bool,
    invalid: bool,
    calls: usize,
}
impl EmbeddingProvider for Provider {
    fn space(&self) -> &EmbeddingSpace {
        &self.space
    }
    fn embed(
        &mut self,
        texts: &[&str],
        _: EmbeddingPurpose,
    ) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        self.calls += 1;
        if self.fail {
            return Err(EmbeddingError::Provider("offline".into()));
        }
        Ok(texts
            .iter()
            .map(|_| {
                if self.invalid {
                    vec![0.0, 0.0]
                } else {
                    vec![1.0, 0.0]
                }
            })
            .collect())
    }
}
fn semantic() -> (SemanticIndex, Provider) {
    let space = EmbeddingSpace::new("context-test-v1", 2).unwrap();
    let mut index = SemanticIndex::new(space.clone());
    index
        .upsert_embedded(
            KnowledgeEntry::new("car", "Kfz-Zulassung durchführen"),
            Embedding::new(space.clone(), vec![1.0, 0.0]).unwrap(),
        )
        .unwrap();
    index
        .upsert_embedded(
            KnowledgeEntry::new("other", "Rust programming"),
            Embedding::new(space.clone(), vec![0.0, 1.0]).unwrap(),
        )
        .unwrap();
    (
        index,
        Provider {
            space,
            fail: false,
            invalid: false,
            calls: 0,
        },
    )
}

#[test]
fn semantic_mode_uses_existing_provider_contract_and_exposes_rank_and_score() {
    let (index, mut provider) = semantic();
    let options = ContextOptions::new(1, 100).with_semantic(SemanticSearchOptions::new(1));
    let bundle = ContextAssembler::from_semantic(&index)
        .query("Register a car", Some(&mut provider), &options)
        .unwrap();
    assert_eq!(ids(&bundle), ["car"]);
    assert!(bundle.semantic_used());
    assert_eq!(provider.calls, 1);
    assert_eq!(
        bundle.items()[0].references()[0].origin().score(),
        Some(1.0)
    );
    assert!(matches!(
        ContextAssembler::new(index.lexical()).query("car", Some(&mut provider), &options),
        Err(ContextError::MissingSemanticIndex)
    ));
    assert!(matches!(
        ContextAssembler::from_semantic(&index).query("car", None, &options),
        Err(ContextError::MissingProvider)
    ));
}

#[test]
fn hybrid_fallback_is_visible_but_invalid_vectors_still_fail() {
    let (index, mut provider) = semantic();
    provider.fail = true;
    let options = ContextOptions::default().with_hybrid(
        HybridSearchOptions::new(2).with_failure_policy(SemanticFailurePolicy::LexicalOnly),
    );
    let bundle = ContextAssembler::from_semantic(&index)
        .query("Rust", Some(&mut provider), &options)
        .unwrap();
    assert_eq!(ids(&bundle), ["other"]);
    assert!(!bundle.semantic_used());
    assert!(bundle.fallback_reason().unwrap().contains("offline"));
    provider.fail = false;
    provider.invalid = true;
    assert!(ContextAssembler::from_semantic(&index)
        .query("Rust", Some(&mut provider), &options)
        .is_err());
}

#[test]
fn denied_empty_and_zero_budgets_do_not_call_providers() {
    let (index, mut provider) = semantic();
    provider.fail = true;
    let options = ContextOptions::default()
        .with_semantic(
            SemanticSearchOptions::new(10).with_context(Context::new().with_entry("car")),
        )
        .with_scope(Context::new().with_entry("other"));
    let assembler = ContextAssembler::from_semantic(&index);
    assert!(assembler
        .query("car", None, &options)
        .unwrap()
        .items()
        .is_empty());
    for options in [
        ContextOptions::new(0, 100),
        ContextOptions::new(10, 0),
        ContextOptions::new(10, 100).with_item_bytes(0),
    ] {
        let options = options.with_semantic(SemanticSearchOptions::default());
        assert!(assembler
            .query("car", Some(&mut provider), &options)
            .unwrap()
            .items()
            .is_empty());
    }
    assert!(assembler
        .query(
            " \n ",
            Some(&mut provider),
            &ContextOptions::default().with_semantic(SemanticSearchOptions::default())
        )
        .unwrap()
        .items()
        .is_empty());
    assert_eq!(provider.calls, 0);
}

#[test]
fn serialization_contains_only_selected_excerpts_and_is_deterministic() {
    let text = format!("Rust {}", "x".repeat(10000));
    let index = index(&[("a", &text), ("b", "hidden")]);
    let options = ContextOptions::new(1, 10).with_metadata(false);
    let assembler = ContextAssembler::new(&index);
    let a = assembler.query("Rust", None, &options).unwrap();
    let b = assembler.query("Rust", None, &options).unwrap();
    let serialized = serde_json::to_string(&a).unwrap();
    assert_eq!(serialized, serde_json::to_string(&b).unwrap());
    assert!(serialized.len() < 1000);
    assert!(!serialized.contains(&"x".repeat(11)));
}

#[test]
fn graph_predicate_filters_start_and_hidden_bridges_before_queueing() {
    let graph = graph(&["a", "b", "c"], &[("a", "b"), ("b", "c")]);
    assert!(graph
        .breadth_first_matching("a", &TraversalOptions::new(4, 10), |id| id != "a")
        .unwrap()
        .is_empty());
    let visits = graph
        .breadth_first_matching("a", &TraversalOptions::new(4, 10), |id| id != "b")
        .unwrap();
    assert_eq!(visits.iter().map(|v| v.id).collect::<Vec<_>>(), ["a"]);
}
