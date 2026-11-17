use airust::{
    context_engine::{ContextAssembler, ContextOptions},
    graph::{KnowledgeGraph, Relation, RelationType, TraversalDirection, TraversalOptions},
    provenance::{ProvenanceStore, SourceReference},
    Engine, KnowledgeEntry, MatchOptions, MatchStrategy, SearchOptions,
};
use serde_json::{json, Value};
use std::{error::Error, hint::black_box, time::Instant};
fn samples<R>(count: usize, mut action: impl FnMut(usize) -> R) -> Value {
    for n in 0..5 {
        black_box(action(n));
    }
    let mut raw = Vec::new();
    for n in 0..count {
        let start = Instant::now();
        let result = action(n);
        raw.push(start.elapsed().as_secs_f64() * 1000.0);
        black_box(result);
    }
    let mut sorted = raw.clone();
    sorted.sort_by(f64::total_cmp);
    let percentile = |p: usize| sorted[(count * p).div_ceil(100).saturating_sub(1)];
    json!({"samples":count,"warmups":5,"p50_ms":percentile(50),"p95_ms":percentile(95),"p99_ms":percentile(99),"raw_samples_ms":raw})
}
fn signature(index: &Engine) -> Vec<(String, u64)> {
    ["Rust", "key17", "changed", "missing_ZXQJ"]
        .into_iter()
        .flat_map(|query| {
            index
                .search(query, 10)
                .into_iter()
                .map(|r| (r.entry.id.clone(), r.score.to_bits()))
        })
        .collect()
}
fn main() -> Result<(), Box<dyn Error>> {
    let count = std::env::args()
        .nth(1)
        .map(|s| s.parse::<usize>())
        .transpose()?
        .unwrap_or(1000);
    let repetitions = std::env::args()
        .nth(2)
        .map(|s| s.parse::<usize>())
        .transpose()?
        .unwrap_or(100);
    if count < 1000 || repetitions == 0 {
        return Err("At least 1000 entries and one sample required".into());
    }
    let id = |n| format!("entry-{n:09}");
    let original: Vec<_> = (0..count)
        .map(|n| {
            KnowledgeEntry::new(
                id(n),
                if n % 101 < 2 {
                    format!("Rust duplicate group{} Grüße 世界", n / 101)
                } else {
                    format!("Rust common knowledge key{n} ownership Grüße 世界")
                },
            )
            .with_metadata(json!({"group":n%19}))
        })
        .collect();
    let start = Instant::now();
    let mut index = Engine::from_entries(original)?;
    let index_build_ms = start.elapsed().as_secs_f64() * 1000.0;
    let start = Instant::now();
    let mut graph = KnowledgeGraph::new();
    let mut sources = ProvenanceStore::new();
    for n in 0..count.min(1000) {
        sources.insert_source(
            SourceReference::new(format!("source-{n}"))?
                .with_document(format!("synthetic-document-{n}"))?,
        )?;
    }
    for n in 0..count {
        graph.add_node(id(n))?;
        sources.bind(id(n), &format!("source-{}", n % 1000))?;
    }
    for n in 1..count {
        graph.upsert_relation(Relation::new(
            format!("edge-{n}"),
            id((n - 1) / 4),
            id(n),
            RelationType::ParentChild,
        )?)?;
    }
    let graph_source_build_ms = start.elapsed().as_secs_f64() * 1000.0;
    let rare = |n: usize| {
        let mut value = n.wrapping_mul(7919) % count;
        while value % 101 < 2 {
            value = (value + 1) % count;
        }
        value
    };
    for n in 0..repetitions {
        let value = rare(n);
        assert_eq!(
            index.search(&format!("key{value}"), 1)[0].entry.id,
            id(value)
        );
    }
    let mut measurements = serde_json::Map::new();
    measurements.insert(
        "search_rare".into(),
        samples(repetitions, |n| {
            index.search(&format!("key{}", rare(n)), 10)
        }),
    );
    measurements.insert(
        "search_common".into(),
        samples(repetitions, |_| index.search("Rust common knowledge", 10)),
    );
    measurements.insert(
        "search_miss".into(),
        samples(repetitions, |_| index.search("missing_ZXQJ", 10)),
    );
    let exact = MatchOptions {
        strategy: MatchStrategy::Exact,
        min_similarity: 1.0,
        limit: 10,
    };
    measurements.insert(
        "matching_exact".into(),
        samples(repetitions, |n| {
            index
                .match_with_options(&index.get(&id(rare(n))).unwrap().text, &exact)
                .unwrap()
        }),
    );
    let fuzzy = MatchOptions {
        strategy: MatchStrategy::Fuzzy,
        min_similarity: 0.9,
        limit: 10,
    };
    measurements.insert(
        "matching_fuzzy".into(),
        samples(5, |_| {
            index
                .match_with_options("Rust common knowledge key17 ownership Grüße 世界", &fuzzy)
                .unwrap()
        }),
    );
    measurements.insert(
        "duplicate_groups".into(),
        samples(repetitions.min(30), |_| index.duplicates()),
    );
    let traversal = TraversalOptions::new(4, 100).with_direction(TraversalDirection::Both);
    assert_eq!(graph.breadth_first(&id(0), &traversal)?.len(), 100);
    measurements.insert(
        "graph_bfs".into(),
        samples(repetitions, |_| {
            graph.breadth_first(&id(0), &traversal).unwrap()
        }),
    );
    measurements.insert(
        "graph_dfs".into(),
        samples(repetitions, |_| {
            graph.depth_first(&id(0), &traversal).unwrap()
        }),
    );
    {
        let assembler = ContextAssembler::new(&index)
            .with_graph(&graph)
            .with_provenance(&sources);
        let options = ContextOptions::new(20, 4096)
            .with_item_bytes(256)
            .with_lexical(SearchOptions {
                limit: 1,
                ..SearchOptions::default()
            })
            .with_traversal(TraversalOptions::new(2, 100).with_direction(TraversalDirection::Both));
        let checked = assembler.query("key17", None, &options)?;
        assert!(checked.node_count() <= 20 && checked.used_text_bytes() <= 4096);
        assert_eq!(checked.items()[0].references()[0].id(), id(17));
        for item in checked.items() {
            let source = index.get(item.references()[0].id()).unwrap();
            assert_eq!(
                item.text(),
                &source.text[item.start_byte()..item.end_byte()]
            );
            assert!(item.references().iter().all(|r| !r.sources().is_empty()));
        }
        measurements.insert(
            "context_with_sources".into(),
            samples(repetitions, |n| {
                assembler
                    .query(&format!("key{}", rare(n)), None, &options)
                    .unwrap()
            }),
        );
    }
    let mutations = count.min(1000);
    let start = Instant::now();
    for n in count - mutations..count {
        let id = id(n);
        sources.remove_entry(&id);
        if n % 5 == 0 {
            index.remove(&id);
            graph.remove_node(&id);
        } else {
            index.upsert(KnowledgeEntry::new(
                id,
                format!("Rust changed key{n} Grüße 世界"),
            ))?;
        }
    }
    let update_ms = start.elapsed().as_secs_f64() * 1000.0;
    let expected = signature(&index);
    let stats = index.stats();
    let encoded = index.to_json()?;
    drop(index);
    let rebuilt = Engine::from_json(&encoded)?;
    assert_eq!(rebuilt.stats(), stats);
    assert_eq!(signature(&rebuilt), expected);
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"airust_version":airust::VERSION,"fixture":"synthetic-quality-scale-v1","initial_entries":count,"index_build_ms":index_build_ms,"source_generation_timed":false,"graph_source_build_ms":graph_source_build_ms,"mutations":mutations,"updates_and_deletes_ms":update_ms,"fresh_rebuild_agrees":true,"measurements":measurements,"limits":"Single process; descriptive samples, not confidence intervals or production tail latency. Peak RSS includes graph, sources, JSON and rebuild staging. No model inference."})
        )?
    );
    Ok(())
}
