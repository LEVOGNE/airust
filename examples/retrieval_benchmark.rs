use airust::{
    retrieval::{LexicalSource, RetrievalOptions, Retriever},
    Engine, KnowledgeEntry,
};
use serde_json::json;
use std::{hint::black_box, time::Instant};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let count = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "10000".into())
        .parse::<usize>()?;
    if count == 0 || count > 1_000_000 {
        return Err("record count must be 1..1000000".into());
    }
    let start = Instant::now();
    let index = Engine::from_entries(
        (0..count)
            .map(|i| {
                KnowledgeEntry::new(
                    format!("record-{i:08}"),
                    format!("Rust memory safety knowledge record {i}. 🦀 Original UTF-8 text."),
                )
            })
            .collect::<Vec<_>>(),
    )?;
    let build_ms = start.elapsed().as_secs_f64() * 1000.0;
    let options = RetrievalOptions::new(10)
        .with_candidates(100)?
        .with_byte_budgets(1024, 4096, 512, 16384)?;
    let mut retriever = Retriever::new(LexicalSource::new(&index));
    let mut reports = Vec::new();
    for query in [
        "Rust memory".to_string(),
        format!("{}", count - 1),
        "absentword".to_string(),
    ] {
        let expected = index
            .search(&query, 10)
            .iter()
            .map(|h| h.entry.id.clone())
            .collect::<Vec<_>>();
        for _ in 0..5 {
            black_box(retriever.query(&query, &options)?);
        }
        let mut samples = Vec::new();
        let mut bytes = 0;
        for _ in 0..30 {
            let start = Instant::now();
            let response = retriever.query(&query, &options)?;
            samples.push(start.elapsed().as_secs_f64() * 1000.0);
            assert_eq!(
                response
                    .hits()
                    .iter()
                    .map(|h| h.id.clone())
                    .collect::<Vec<_>>(),
                expected
            );
            bytes = response.to_json().len();
            assert!(bytes <= options.max_output_bytes());
            black_box(response);
        }
        samples.sort_by(f64::total_cmp);
        reports.push(json!({"query":query,"median_ms":(samples[14]+samples[15])/2.0,"samples_ms":samples,"output_bytes":bytes}));
    }
    println!(
        "{}",
        json!({"entries":count,"build_ms":build_ms,"queries":reports,"method":"public synthetic records; source build includes generation; five warmups and thirty end-to-end search/evidence/excerpt/JSON samples per query; exact IDs compared to Engine"})
    );
    Ok(())
}
