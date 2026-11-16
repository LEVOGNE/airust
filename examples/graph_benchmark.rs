use airust::graph::{KnowledgeGraph, Relation, RelationType, TraversalOptions};
use serde_json::json;
use std::{error::Error, hint::black_box, time::Instant};

fn main() -> Result<(), Box<dyn Error>> {
    let count = std::env::args()
        .nth(1)
        .map(|s| s.parse::<usize>())
        .transpose()?
        .unwrap_or(1000);
    if count == 0 {
        return Err("Node count must be positive".into());
    }
    let id = |n| format!("node-{n:09}");
    let mut graph = KnowledgeGraph::new();
    let started = Instant::now();
    for n in 0..count {
        graph.add_node(id(n))?;
    }
    for n in 1..count {
        graph.upsert_relation(Relation::new(
            format!("e{n}"),
            id(n - 1),
            id(n),
            RelationType::ParentChild,
        )?)?;
        if n > 1 && n % 13 == 0 {
            graph.upsert_relation(Relation::new(
                format!("x{n}"),
                id(n - 2),
                id(n),
                RelationType::References,
            )?)?;
        }
    }
    let build_ms = started.elapsed().as_secs_f64() * 1000.0;
    let relations = graph.relation_count();
    let started = Instant::now();
    assert!(!graph.has_cycle());
    let cycle_ms = started.elapsed().as_secs_f64() * 1000.0;
    graph.upsert_relation(Relation::new(
        "back",
        id(count - 1),
        id(0),
        RelationType::DependsOn,
    )?)?;
    assert!(graph.has_cycle());
    graph.remove_relation("back");
    let options = TraversalOptions::new(256, 100);
    let mut bfs = Vec::new();
    let mut dfs = Vec::new();
    for n in 0..50 {
        let start = id(n * 7919 % count.saturating_sub(100).max(1));
        let began = Instant::now();
        let result = graph.breadth_first(&start, &options)?;
        bfs.push(began.elapsed().as_secs_f64() * 1000.0);
        assert!(!result.is_empty());
        assert!(result.len() <= 100);
        black_box(result);
        let began = Instant::now();
        let result = graph.depth_first(&start, &options)?;
        dfs.push(began.elapsed().as_secs_f64() * 1000.0);
        assert!(!result.is_empty());
        assert!(result.len() <= 100);
        black_box(result);
    }
    bfs.sort_by(f64::total_cmp);
    dfs.sort_by(f64::total_cmp);
    let mutations = count.saturating_sub(1).min(100);
    let began = Instant::now();
    for n in 1..=mutations {
        assert!(graph
            .upsert_relation(Relation::new(
                format!("e{n}"),
                id(n - 1),
                id(n),
                RelationType::References
            )?)?
            .is_some());
    }
    let update_ms = began.elapsed().as_secs_f64() * 1000.0;
    let began = Instant::now();
    for n in 0..count.min(100) {
        assert!(graph.remove_node(&id(n)));
    }
    let delete_ms = began.elapsed().as_secs_f64() * 1000.0;
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"airust_version":airust::VERSION,"nodes":count,"relations":relations,"build_ms":build_ms,"cycle_check_ms":cycle_ms,"query_count":50,"depth_limit":256,"node_limit":100,"bfs_p50_ms":bfs[24],"bfs_p95_ms":bfs[46],"dfs_p50_ms":dfs[24],"dfs_p95_ms":dfs[46],"bfs_samples_ms":bfs,"dfs_samples_ms":dfs,"relation_updates":mutations,"update_ms":update_ms,"nodes_deleted":count.min(100),"delete_ms":delete_ms,"limits":"Single-process directed chain with deterministic forward shortcuts; bounded traversals; no claim for dense, mixed or application graphs."})
        )?
    );
    Ok(())
}
