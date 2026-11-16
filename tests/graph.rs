#![cfg(feature = "graph")]

use airust::graph::{
    GraphError, KnowledgeGraph, Relation, RelationDirection, RelationType, TraversalDirection,
    TraversalOptions,
};
use serde_json::json;

fn graph(ids: &[&str]) -> KnowledgeGraph {
    let mut graph = KnowledgeGraph::new();
    for id in ids {
        graph.add_node(*id).unwrap();
    }
    graph
}
fn add(
    graph: &mut KnowledgeGraph,
    id: &str,
    source: &str,
    target: &str,
    kind: RelationType,
    undirected: bool,
) {
    let mut relation = Relation::new(id, source, target, kind).unwrap();
    if undirected {
        relation = relation.with_direction(RelationDirection::Undirected);
    }
    graph.upsert_relation(relation).unwrap();
}

#[test]
fn empty_graph_ids_missing_nodes_and_empty_budgets_are_explicit() {
    let mut g = KnowledgeGraph::new();
    assert!(g.is_empty());
    assert!(!g.has_cycle());
    assert!(g.add_node(" ").is_err());
    assert_eq!(g.node_count(), 0);
    assert!(matches!(
        g.breadth_first("absent", &TraversalOptions::new(0, 0)),
        Err(GraphError::UnknownNode(_))
    ));
    assert!(g.neighbors("absent", TraversalDirection::Both).is_err());
    assert!(g.add_node("A").unwrap());
    assert!(!g.add_node("A").unwrap());
    assert!(g
        .breadth_first("A", &TraversalOptions::new(9, 0))
        .unwrap()
        .is_empty());
    assert!(g
        .depth_first("A", &TraversalOptions::new(9, 0))
        .unwrap()
        .is_empty());
    assert!(Relation::new("", "A", "A", RelationType::Related).is_err());
    assert!(Relation::new("e", "", "A", RelationType::Related).is_err());
    assert!(Relation::new("e", "A", "", RelationType::Related).is_err());
    assert!(Relation::new("e", "A", "A", RelationType::Custom(" ".into())).is_err());
}

#[test]
fn directed_undirected_parallel_and_self_neighbors_preserve_evidence() {
    let mut g = graph(&["a", "b", "c"]);
    add(&mut g, "z", "a", "b", RelationType::ParentChild, false);
    add(&mut g, "a", "a", "b", RelationType::References, false);
    add(&mut g, "u", "c", "a", RelationType::Related, true);
    add(&mut g, "self", "a", "a", RelationType::Related, false);
    let out = g.neighbors("a", TraversalDirection::Outgoing).unwrap();
    assert_eq!(
        out.iter()
            .map(|n| (n.id, n.relation.id()))
            .collect::<Vec<_>>(),
        [("a", "self"), ("b", "a"), ("b", "z"), ("c", "u")]
    );
    let incoming = g.neighbors("a", TraversalDirection::Incoming).unwrap();
    assert_eq!(
        incoming.iter().map(|n| n.id).collect::<Vec<_>>(),
        ["a", "c"]
    );
    assert_eq!(
        g.neighbors("b", TraversalDirection::Incoming)
            .unwrap()
            .len(),
        2
    );
    assert!(g
        .neighbors("b", TraversalDirection::Outgoing)
        .unwrap()
        .is_empty());
    assert_eq!(
        g.neighbors("c", TraversalDirection::Incoming).unwrap()[0].id,
        "a"
    );
}

#[test]
fn retargeting_removal_and_failed_replacements_keep_maps_consistent() {
    let mut g = graph(&["a", "b", "c"]);
    let relation = Relation::new("edge", "a", "b", RelationType::DependsOn)
        .unwrap()
        .with_metadata(json!({"source":"manual"}));
    g.upsert_relation(relation.clone()).unwrap();
    let invalid = Relation::new("edge", "a", "missing", RelationType::References).unwrap();
    assert!(g.upsert_relation(invalid).is_err());
    assert_eq!(g.get_relation("edge"), Some(&relation));
    let old = g
        .upsert_relation(Relation::new("edge", "c", "a", RelationType::References).unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(old, relation);
    assert_eq!(old.metadata().unwrap()["source"], "manual");
    assert!(g
        .neighbors("b", TraversalDirection::Both)
        .unwrap()
        .is_empty());
    assert_eq!(
        g.neighbors("c", TraversalDirection::Outgoing).unwrap()[0].id,
        "a"
    );
    add(&mut g, "loop", "a", "a", RelationType::Related, true);
    add(&mut g, "parallel", "c", "a", RelationType::Related, true);
    assert!(g.remove_node("a"));
    assert_eq!(g.relation_count(), 0);
    assert!(!g.contains_node("a"));
    assert!(!g.remove_node("absent"));
    assert!(g.remove_relation("edge").is_none());
    assert!(g.add_node("d").unwrap());
    add(
        &mut g,
        "new",
        "c",
        "d",
        RelationType::Custom("supports".into()),
        false,
    );
    assert_eq!(g.get_relation("new").unwrap().target(), "d");
    assert_eq!(g.nodes().collect::<Vec<_>>(), ["b", "c", "d"]);
    let owned = g.remove_relation("new").unwrap();
    g.clear();
    assert!(g.is_empty());
    assert_eq!(owned.source(), "c");
    assert_eq!(owned.target(), "d");
}

fn tree(reverse: bool) -> KnowledgeGraph {
    let mut g = graph(&["root", "a", "b", "c", "d", "e"]);
    let mut relations = vec![
        ("ra", "root", "a"),
        ("rb", "root", "b"),
        ("ac", "a", "c"),
        ("ad", "a", "d"),
        ("be", "b", "e"),
    ];
    if reverse {
        relations.reverse();
    }
    for (id, source, target) in relations {
        add(&mut g, id, source, target, RelationType::ParentChild, false);
    }
    g
}

#[test]
fn breadth_and_depth_orders_budgets_and_parent_paths_are_deterministic() {
    for reverse in [false, true] {
        let g = tree(reverse);
        let options = TraversalOptions::new(9, 99);
        let bfs = g.breadth_first("root", &options).unwrap();
        assert_eq!(
            bfs.iter().map(|v| v.id).collect::<Vec<_>>(),
            ["root", "a", "b", "c", "d", "e"]
        );
        let dfs = g.depth_first("root", &options).unwrap();
        assert_eq!(
            dfs.iter().map(|v| v.id).collect::<Vec<_>>(),
            ["root", "a", "c", "d", "b", "e"]
        );
        assert_eq!(dfs[2].parent, Some("a"));
        assert_eq!(dfs[2].relation.unwrap().id(), "ac");
        assert_eq!(dfs[2].depth, 2);
        for limit in 0..=6 {
            let options = TraversalOptions::new(9, limit);
            assert_eq!(
                g.breadth_first("root", &options)
                    .unwrap()
                    .iter()
                    .map(|v| v.id)
                    .collect::<Vec<_>>(),
                bfs.iter().take(limit).map(|v| v.id).collect::<Vec<_>>()
            );
            assert_eq!(
                g.depth_first("root", &options)
                    .unwrap()
                    .iter()
                    .map(|v| v.id)
                    .collect::<Vec<_>>(),
                dfs.iter().take(limit).map(|v| v.id).collect::<Vec<_>>()
            );
        }
        assert_eq!(
            g.breadth_first("root", &TraversalOptions::new(0, 10))
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            g.depth_first("root", &TraversalOptions::new(1, 10))
                .unwrap()
                .len(),
            3
        );
        let options = TraversalOptions::new(9, 3).with_start(false);
        assert_eq!(
            g.breadth_first("root", &options)
                .unwrap()
                .iter()
                .map(|v| v.id)
                .collect::<Vec<_>>(),
            ["a", "b", "c"]
        );
        assert_eq!(
            g.depth_first("root", &options)
                .unwrap()
                .iter()
                .map(|v| v.id)
                .collect::<Vec<_>>(),
            ["a", "c", "d"]
        );
    }
}

#[test]
fn allowlists_and_relation_filters_cannot_use_hidden_bridges() {
    let g = tree(false);
    let options = TraversalOptions::new(10, 20).with_allowed_ids(["root", "c", "b", "e"]);
    for visits in [
        g.breadth_first("root", &options).unwrap(),
        g.depth_first("root", &options).unwrap(),
    ] {
        assert_eq!(
            visits.iter().map(|v| v.id).collect::<Vec<_>>(),
            ["root", "b", "e"]
        );
    }
    let empty = TraversalOptions::default().with_allowed_ids(Vec::<String>::new());
    assert!(g.breadth_first("root", &empty).unwrap().is_empty());
    let filtered = TraversalOptions::default().with_relation_types([RelationType::DependsOn]);
    assert_eq!(g.depth_first("root", &filtered).unwrap().len(), 1);
    let incoming = TraversalOptions::new(10, 20).with_direction(TraversalDirection::Incoming);
    assert_eq!(
        g.breadth_first("c", &incoming)
            .unwrap()
            .iter()
            .map(|v| v.id)
            .collect::<Vec<_>>(),
        ["c", "a", "root"]
    );
}

#[test]
fn traversal_deduplicates_parallel_targets_and_cycles_without_losing_budget() {
    let mut g = graph(&["a", "b", "c", "d", "e"]);
    for (id, source, target) in [
        ("abz", "a", "b"),
        ("aba", "a", "b"),
        ("ac", "a", "c"),
        ("ad", "a", "d"),
        ("bc", "b", "c"),
        ("ce", "c", "e"),
        ("ea", "e", "a"),
    ] {
        add(&mut g, id, source, target, RelationType::References, false);
    }
    let options = TraversalOptions::new(10, 5);
    let dfs = g.depth_first("a", &options).unwrap();
    assert_eq!(
        dfs.iter().map(|v| v.id).collect::<Vec<_>>(),
        ["a", "b", "c", "e", "d"]
    );
    assert_eq!(dfs[1].relation.unwrap().id(), "aba");
    assert_eq!(g.breadth_first("a", &options).unwrap().len(), 5);
    assert!(g.has_cycle());
}

#[test]
fn mixed_cycles_self_loops_parallel_edges_and_type_selection_are_correct() {
    let mut g = graph(&["a", "b", "c", "d"]);
    add(&mut g, "ab", "a", "b", RelationType::ParentChild, true);
    assert!(!g.has_cycle());
    add(&mut g, "bc", "b", "c", RelationType::DependsOn, false);
    assert!(!g.has_cycle());
    add(&mut g, "ca", "c", "a", RelationType::DependsOn, false);
    assert!(g.has_cycle());
    assert!(!g.has_cycle_for(&[RelationType::DependsOn]));
    assert!(!g.has_cycle_for(&[]));
    g.remove_relation("ca");
    assert!(!g.has_cycle());
    add(&mut g, "ba", "b", "a", RelationType::References, false);
    assert!(g.has_cycle());
    g.remove_relation("ba");
    add(&mut g, "ab2", "a", "b", RelationType::Related, true);
    assert!(g.has_cycle());
    g.remove_relation("ab2");
    for undirected in [false, true] {
        add(&mut g, "self", "d", "d", RelationType::Related, undirected);
        assert!(g.has_cycle());
        g.remove_relation("self");
        assert!(!g.has_cycle());
    }
}

#[test]
fn breadth_first_provides_shortest_hops_while_depth_first_keeps_first_discovery() {
    let mut g = graph(&["r", "a", "b", "x", "y"]);
    for (id, source, target) in [
        ("ra", "r", "a"),
        ("ab", "a", "b"),
        ("bx", "b", "x"),
        ("rx", "r", "x"),
        ("xy", "x", "y"),
    ] {
        add(&mut g, id, source, target, RelationType::References, false);
    }
    let options = TraversalOptions::new(3, 100);
    let bfs = g.breadth_first("r", &options).unwrap();
    assert_eq!(bfs.iter().find(|v| v.id == "x").unwrap().depth, 1);
    assert_eq!(bfs.iter().find(|v| v.id == "y").unwrap().depth, 2);
    let dfs = g.depth_first("r", &options).unwrap();
    assert_eq!(dfs.iter().find(|v| v.id == "x").unwrap().depth, 3);
    assert!(dfs.iter().all(|v| v.id != "y"));
}

fn reference_cycle(nodes: usize, edges: &[(usize, usize, bool)]) -> bool {
    fn walk(
        current: usize,
        start: usize,
        edges: &[(usize, usize, bool)],
        seen: &mut Vec<usize>,
        used: &mut Vec<usize>,
    ) -> bool {
        for (id, &(source, target, undirected)) in edges.iter().enumerate() {
            if used.contains(&id) {
                continue;
            }
            let next = if source == current {
                Some(target)
            } else if undirected && target == current {
                Some(source)
            } else {
                None
            };
            let Some(next) = next else {
                continue;
            };
            if next == start {
                return true;
            }
            if seen.contains(&next) {
                continue;
            }
            seen.push(next);
            used.push(id);
            if walk(next, start, edges, seen, used) {
                return true;
            }
            seen.pop();
            used.pop();
        }
        false
    }
    (0..nodes).any(|start| walk(start, start, edges, &mut vec![start], &mut Vec::new()))
}

#[test]
fn mixed_cycle_detection_matches_exhaustive_small_graph_reference() {
    let pairs = [(0, 1), (0, 2), (1, 2)];
    for mask in 0..512 {
        let mut g = graph(&["0", "1", "2"]);
        let mut edges = Vec::new();
        for (pair, &(a, b)) in pairs.iter().enumerate() {
            for (bit, (source, target, undirected)) in [(a, b, false), (b, a, false), (a, b, true)]
                .into_iter()
                .enumerate()
            {
                if mask & (1 << (pair * 3 + bit)) != 0 {
                    edges.push((source, target, undirected));
                }
            }
        }
        for (n, &(a, b, u)) in edges.iter().enumerate() {
            add(
                &mut g,
                &n.to_string(),
                &a.to_string(),
                &b.to_string(),
                RelationType::Related,
                u,
            );
        }
        assert_eq!(g.has_cycle(), reference_cycle(3, &edges), "mask {mask}");
    }
}

#[test]
fn long_chains_and_high_degree_nodes_remain_iterative_and_bounded() {
    let mut chain = KnowledgeGraph::new();
    let count = 20_000;
    for n in 0..count {
        chain.add_node(format!("{n:06}")).unwrap();
    }
    for n in 1..count {
        add(
            &mut chain,
            &format!("e{n}"),
            &format!("{:06}", n - 1),
            &format!("{n:06}"),
            RelationType::DependsOn,
            false,
        );
    }
    assert!(!chain.has_cycle());
    let options = TraversalOptions::new(usize::MAX, count);
    assert_eq!(
        chain.breadth_first("000000", &options).unwrap().len(),
        count
    );
    assert_eq!(chain.depth_first("000000", &options).unwrap().len(), count);
    add(
        &mut chain,
        "back",
        &format!("{:06}", count - 1),
        "000000",
        RelationType::DependsOn,
        false,
    );
    assert!(chain.has_cycle());
    let mut star = KnowledgeGraph::new();
    star.add_node("root").unwrap();
    for n in (0..10_000).rev() {
        let id = format!("{n:06}");
        star.add_node(&id).unwrap();
        add(
            &mut star,
            &format!("e{n}"),
            "root",
            &id,
            RelationType::ParentChild,
            false,
        );
    }
    let options = TraversalOptions::new(1, 4);
    for visits in [
        star.breadth_first("root", &options).unwrap(),
        star.depth_first("root", &options).unwrap(),
    ] {
        assert_eq!(
            visits.iter().map(|v| v.id).collect::<Vec<_>>(),
            ["root", "000000", "000001", "000002"]
        );
    }
}

#[test]
fn repeated_node_and_edge_updates_match_public_source_relationships() {
    let mut g = KnowledgeGraph::new();
    for n in 0..30 {
        g.add_node(n.to_string()).unwrap();
    }
    let mut expected = std::collections::BTreeMap::<String, (String, String)>::new();
    for step in 0..600 {
        let id = format!("e{}", step % 47);
        let source = (step * 13 % 30).to_string();
        let target = (step * 7 % 30).to_string();
        if step % 11 == 0 {
            g.remove_node(&source);
            expected.retain(|_, (a, b)| a != &source && b != &source);
            g.add_node(&source).unwrap();
        } else if step % 5 == 0 {
            assert_eq!(
                g.remove_relation(&id).is_some(),
                expected.remove(&id).is_some()
            );
        } else {
            let previous = g
                .upsert_relation(
                    Relation::new(&id, &source, &target, RelationType::Related).unwrap(),
                )
                .unwrap();
            assert_eq!(
                previous.is_some(),
                expected.insert(id, (source, target)).is_some()
            );
        }
        assert_eq!(g.relation_count(), expected.len());
        for relation in g.relations() {
            let (source, target) = &expected[relation.id()];
            assert_eq!(relation.source(), source);
            assert_eq!(relation.target(), target);
            let encoded = serde_json::to_value(relation).unwrap();
            assert_eq!(encoded["source"], *source);
            assert_eq!(encoded["target"], *target);
        }
    }
}
