use airust::{
    text_search::{AnalyzerConfig, TextIndex, TextSearchOptions},
    KnowledgeEntry,
};
use serde_json::json;
use std::{hint::black_box, time::Instant};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let n = args
        .first()
        .map(|v| v.parse())
        .transpose()?
        .unwrap_or(10_000usize);
    let samples = args
        .get(1)
        .map(|v| v.parse())
        .transpose()?
        .unwrap_or(30usize);
    if n == 0 || samples == 0 {
        return Err("entry/sample counts must be positive".into());
    }
    let records: Vec<_> = (0..n)
        .map(|i| {
            KnowledgeEntry::new(
                format!("doc-{i:09}"),
                format!(
                    "key{i} Häuser Gärten {}",
                    if i % 3 == 0 {
                        "red car"
                    } else {
                        "red bright car"
                    }
                ),
            )
        })
        .collect();
    let start = Instant::now();
    let mut index = TextIndex::from_entries(AnalyzerConfig::german(), records)?;
    let build = start.elapsed().as_secs_f64() * 1000.0;
    let mut reports = Vec::new();
    for query in ["Haus", "\"red car\"", "\"red missing car\"", "key42"] {
        for _ in 0..5 {
            black_box(index.search(query, &TextSearchOptions::new(10))?);
        }
        let mut raw = Vec::new();
        let mut count = 0;
        for _ in 0..samples {
            let start = Instant::now();
            let hits = index.search(query, &TextSearchOptions::new(10))?;
            count = hits.len();
            raw.push(start.elapsed().as_secs_f64() * 1000.0);
            black_box(hits);
        }
        reports.push(json!({"query":query,"returned":count,"raw_ms":raw}));
    }
    let start = Instant::now();
    for i in 0..n.min(100) {
        index.upsert(KnowledgeEntry::new(
            format!("doc-{i:09}"),
            "Neue Häuser red car",
        ))?;
    }
    let updates = start.elapsed().as_secs_f64() * 1000.0;
    let fresh = TextIndex::from_entries(AnalyzerConfig::german(), index.entries().cloned())?;
    assert_eq!(fresh.stats(), index.stats());
    for q in ["Haus", "\"red car\"", "key42"] {
        assert_eq!(
            serde_json::to_value(index.search(q, &TextSearchOptions::new(20))?)?,
            serde_json::to_value(fresh.search(q, &TextSearchOptions::new(20))?)?
        );
    }
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"version":airust::VERSION,"entries":n,"samples":samples,"warmups":5,"analyzer":index.analyzer().config().recipe(),"build_ms_excluding_generation":build,"updates":n.min(100),"update_ms":updates,"reports":reports,"rebuild_agreement":true,"memory_scope":"source, positional index, returned evidence, updates and simultaneous fresh rebuild; not minimal steady-state index"})
        )?
    );
    Ok(())
}
