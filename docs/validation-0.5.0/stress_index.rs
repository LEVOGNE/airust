use airust::{Engine, KnowledgeEntry, SearchOptions};
use serde_json::{json, Value};
use std::time::Instant;
fn entry(i: usize) -> KnowledgeEntry {
    KnowledgeEntry::new(
        format!("item-{i:08}"),
        format!(
            "Rust ownership memory topic{} group{} indexing retrieval records",
            i % 1000,
            i % 100
        ),
    )
}
fn summary(mut values: Vec<f64>) -> Value {
    values.sort_by(f64::total_cmp);
    let at = |fraction: f64| values[((values.len() - 1) as f64 * fraction).ceil() as usize];
    json!({"samples":values.len(),"min_ms":values[0],"p50_ms":at(0.5),"p95_ms":at(0.95),"p99_ms":at(0.99),"max_ms":values[values.len()-1]})
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let mode = args.first().map(String::as_str).unwrap_or("scale");
    let count = args
        .get(1)
        .map(|s| s.parse())
        .transpose()?
        .unwrap_or(1000usize);
    if !["scale", "clone"].contains(&mode)
        || !(1000..=2000000).contains(&count)
        || (mode == "clone" && count > 500000)
    {
        return Err("usage: stress_index scale 1000..2000000 OR clone 1000..500000".into());
    }
    let start = Instant::now();
    let mut index = Engine::from_entries((0..count).map(entry))?;
    let build_ms = start.elapsed().as_secs_f64() * 1000.0;
    if mode == "clone" {
        let mut retained = Vec::new();
        let mut clone_ms = Vec::new();
        for _ in 0..3 {
            let start = Instant::now();
            retained.push(index.clone());
            clone_ms.push(start.elapsed().as_secs_f64() * 1000.0);
        }
        let start = Instant::now();
        retained[0].upsert(KnowledgeEntry::new(
            "item-00000000",
            "Changed isolated snapshot marker",
        ))?;
        let mutate_ms = start.elapsed().as_secs_f64() * 1000.0;
        assert_eq!(index.get("item-00000000").unwrap().text, entry(0).text);
        assert_eq!(
            retained[1].get("item-00000000").unwrap().text,
            entry(0).text
        );
        assert_ne!(
            retained[0].get("item-00000000").unwrap().text,
            entry(0).text
        );
        assert_eq!(retained.iter().map(Engine::len).sum::<usize>(), 3 * count);
        println!(
            "{}",
            json!({"version":airust::VERSION,"fixture":"stress-index-v1","mode":mode,"entries":count,"build_ms":build_ms,"retained_clones":3,"clone_ms":clone_ms,"one_clone_mutation_ms":mutate_ms,"old_views_unchanged":true})
        );
        return Ok(());
    }
    let mut selective = Vec::new();
    let mut common = Vec::new();
    let mut absent = Vec::new();
    let mut signature = 0u64;
    for i in 0..100 {
        let start = Instant::now();
        for hit in index.search(&format!("topic{}", i * 37 % 1000), 10) {
            signature = signature.wrapping_add(hit.score.to_bits());
        }
        selective.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    for _ in 0..8 {
        let start = Instant::now();
        let hits = index.search("Rust", 10);
        assert_eq!(hits.len(), 10);
        common.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    for _ in 0..10 {
        let start = Instant::now();
        assert!(index.search("unfindableterm", 10).is_empty());
        absent.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    let start = Instant::now();
    for i in 0..100 {
        index.upsert(KnowledgeEntry::new(
            format!("item-{i:08}"),
            format!("updatedmarker topic{}", i % 1000),
        ))?;
    }
    let update_ms = start.elapsed().as_secs_f64() * 1000.0;
    let start = Instant::now();
    for i in 0..20 {
        assert!(index.remove(&format!("item-{i:08}")).is_some());
    }
    let delete_ms = start.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(index.len(), count - 20);
    assert_eq!(index.search("updatedmarker", 100).len(), 80);
    let index = std::sync::Arc::new(index);
    let concurrent = std::thread::scope(|scope| {
        let mut threads = Vec::new();
        for worker in 0..4 {
            let index = index.clone();
            threads.push(scope.spawn(move || {
                let mut times = Vec::new();
                for i in 0..20 {
                    let start = Instant::now();
                    let hits = index.search_with_options(
                        &format!("topic{}", (worker * 31 + i * 17) % 1000),
                        &SearchOptions::default(),
                    );
                    for hit in hits {
                        assert!(index.get(&hit.entry.id).is_some());
                    }
                    times.push(start.elapsed().as_secs_f64() * 1000.0);
                }
                times
            }));
        }
        threads
            .into_iter()
            .flat_map(|h| h.join().unwrap())
            .collect::<Vec<_>>()
    });
    println!(
        "{}",
        json!({"version":airust::VERSION,"fixture":"stress-index-v1","mode":mode,"entries":count,"final_entries":index.len(),"build_ms":build_ms,"selective":summary(selective),"common":summary(common),"absent":summary(absent),"concurrent_readers":4,"concurrent_selective":summary(concurrent),"update_100_ms":update_ms,"delete_20_ms":delete_ms,"score_signature":signature,"limits":"Synthetic lexical RAM workload; separate latency categories; no model, storage or real-corpus capacity guarantee."})
    );
    Ok(())
}
