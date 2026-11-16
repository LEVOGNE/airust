use airust::{
    context_engine::{ContextAssembler, ContextOptions},
    graph::{KnowledgeGraph, Relation, RelationType, TraversalDirection, TraversalOptions},
    Engine, KnowledgeEntry, SearchOptions,
};
use serde_json::json;
use std::{error::Error, hint::black_box, time::Instant};

fn main() -> Result<(), Box<dyn Error>> {
    let count = std::env::args()
        .nth(1)
        .map(|s| s.parse::<usize>())
        .transpose()?
        .unwrap_or(1000);
    if count == 0 {
        return Err("Entry count must be positive".into());
    }
    let id = |n| format!("node-{n:09}");
    let mut index = Engine::new();
    let mut graph = KnowledgeGraph::new();
    let began = Instant::now();
    for n in 0..count {
        index.upsert(KnowledgeEntry::new(
            id(n),
            format!(
                "Identifier key{n} describes knowledge and context. {}",
                "Multilingual Grüße 世界 knowledge. ".repeat(20)
            ),
        ))?;
        graph.add_node(id(n))?;
        if n > 0 {
            graph.upsert_relation(Relation::new(
                format!("edge-{n}"),
                id(n - 1),
                id(n),
                RelationType::ParentChild,
            )?)?;
        }
    }
    let build_ms = began.elapsed().as_secs_f64() * 1000.0;
    let assembler = ContextAssembler::new(&index).with_graph(&graph);
    let options = ContextOptions::new(20, 2048)
        .with_item_bytes(256)
        .with_lexical(SearchOptions {
            limit: 1,
            ..SearchOptions::default()
        })
        .with_traversal(TraversalOptions::new(10, 100).with_direction(TraversalDirection::Both));
    let mut samples = Vec::new();
    for n in 0..100 {
        let query = format!("key{}", n * 7919 % count);
        let began = Instant::now();
        let bundle = assembler.query(&query, None, &options)?;
        samples.push(began.elapsed().as_secs_f64() * 1000.0);
        assert!(bundle.node_count() <= 20 && bundle.used_text_bytes() <= 2048);
        assert_eq!(bundle.items()[0].references()[0].id(), id(n * 7919 % count));
        for item in bundle.items() {
            let source = index.get(item.references()[0].id()).unwrap();
            assert_eq!(
                item.text(),
                &source.text[item.start_byte()..item.end_byte()]
            );
        }
        black_box(bundle);
    }
    samples.sort_by(f64::total_cmp);
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"airust_version":airust::VERSION,"entries":count,"build_ms":build_ms,"queries":100,"node_budget":20,"text_byte_budget":2048,"per_item_bytes":256,"p50_ms":samples[49],"p95_ms":samples[94],"samples_ms":samples})
        )?
    );
    Ok(())
}
