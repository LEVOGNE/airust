use serde_json::{json, Value};
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;
fn stats(s: dhat::HeapStats) -> Value {
    json!({"live_bytes":s.curr_bytes,"live_blocks":s.curr_blocks,"peak_bytes":s.max_bytes,"total_bytes":s.total_bytes,"total_blocks":s.total_blocks})
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let mode = &args[0];
    let count: usize = args[2].parse()?;
    let mut raw: Vec<airust::KnowledgeEntry> =
        serde_json::from_reader(std::fs::File::open(&args[1])?)?;
    raw.truncate(count);
    raw.shrink_to_fit();
    let queries: Vec<String> = raw
        .iter()
        .map(|e| {
            airust::text::term_frequencies(&e.text)
                .into_keys()
                .filter(|term| !airust::text::is_stopword(term))
                .take(3)
                .collect::<Vec<_>>()
                .join(" ")
        })
        .filter(|q| !q.is_empty())
        .take(10)
        .collect();
    assert_eq!(queries.len(), 10);

    let profiler = dhat::Profiler::builder()
        .file_name(&args[3])
        .trim_backtraces(Some(16))
        .build();
    let (build, warmed, cloned, released_clone, released) = if mode == "previous" {
        let entries = raw
            .iter()
            .map(|e| previous::KnowledgeEntry {
                id: e.id.clone(),
                text: e.text.clone(),
                metadata: e.metadata.clone(),
            })
            .collect::<Vec<_>>();
        let index = previous::Engine::from_entries(entries)?;
        let build = dhat::HeapStats::get();
        for _ in 0..2 {
            for query in &queries {
                let hits = index.search_with_options(
                    query,
                    &previous::SearchOptions {
                        algorithm: previous::SearchAlgorithm::Tfidf,
                        limit: 10,
                        ..Default::default()
                    },
                );
                assert!(
                    !hits.is_empty(),
                    "Positive profiling probe must return hits"
                );
                std::hint::black_box(&hits);
            }
        }
        let warmed = dhat::HeapStats::get();
        let clone = index.clone();
        let cloned = dhat::HeapStats::get();
        drop(clone);
        let released_clone = dhat::HeapStats::get();
        drop(index);
        (
            build,
            warmed,
            cloned,
            released_clone,
            dhat::HeapStats::get(),
        )
    } else {
        assert_eq!(mode, "current");
        let entries = raw.clone();
        let index = airust::Engine::from_entries(entries)?;
        let build = dhat::HeapStats::get();
        for _ in 0..2 {
            for query in &queries {
                let hits = index.search_with_options(
                    query,
                    &airust::SearchOptions {
                        algorithm: airust::SearchAlgorithm::Tfidf,
                        limit: 10,
                        ..Default::default()
                    },
                );
                assert!(
                    !hits.is_empty(),
                    "Positive profiling probe must return hits"
                );
                std::hint::black_box(&hits);
            }
        }
        let warmed = dhat::HeapStats::get();
        let clone = index.clone();
        let cloned = dhat::HeapStats::get();
        drop(clone);
        let released_clone = dhat::HeapStats::get();
        drop(index);
        (
            build,
            warmed,
            cloned,
            released_clone,
            dhat::HeapStats::get(),
        )
    };
    drop(profiler);
    let data = json!({"mode":mode,"entries":raw.len(),"build":stats(build),"warmed":stats(warmed),"cloned":stats(cloned),"released_clone":stats(released_clone),"released_all":stats(released),"scope":"DHAT-instrumented tracked heap for owned index clones of preloaded records; excludes preloaded source and profiler overhead. Timings under instrumentation are not performance measurements."});
    let output = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[4])?;
    serde_json::to_writer_pretty(output, &data)?;
    println!("{data}");
    Ok(())
}
