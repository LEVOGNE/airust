use airust::{
    retrieval::{ExcerptOptions, LexicalSource, RetrievalOptions, Retriever},
    Engine, KnowledgeEntry,
};
use serde_json::json;
use std::{hint::black_box, time::Instant};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let count = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "10000".into())
        .parse::<usize>()?;
    if count == 0 || count > 100000 {
        return Err("record count must be 1..100000".into());
    }
    let start = Instant::now();
    let index=Engine::from_entries((0..count).map(|i|KnowledgeEntry::new(format!("record-{i:08}"),format!("{}\nRust ownership prevents use-after-free errors in key{i}.\nAdditional UTF-8 context: 🦀 cafe\u{301}.","Unrelated background information for the reader. ".repeat(18)))))?;
    let build_ms = start.elapsed().as_secs_f64() * 1000.0;
    let base = RetrievalOptions::new(10)
        .with_candidates(100)?
        .with_byte_budgets(1024, 2048, 160, 16384)?;
    let mut retriever = Retriever::new(LexicalSource::new(&index));
    let mut reports = Vec::new();
    for query in [
        "Rust ownership".to_string(),
        format!("key{}", count - 1),
        "absentword".to_string(),
    ] {
        let expected = index
            .search(&query, 10)
            .into_iter()
            .map(|h| (h.entry.id.clone(), h.score.to_bits()))
            .collect::<Vec<_>>();
        for (mode, options) in [
            ("prefix", base.clone()),
            (
                "relevant",
                base.clone()
                    .with_relevant_excerpts(ExcerptOptions::default()),
            ),
        ] {
            for _ in 0..5 {
                black_box(retriever.query(&query, &options)?);
            }
            let mut samples = Vec::new();
            let (mut bytes, mut covering, mut returned) = (0, 0, 0);
            for _ in 0..30 {
                let start = Instant::now();
                let response = retriever.query(&query, &options)?;
                samples.push(start.elapsed().as_secs_f64() * 1000.0);
                assert_eq!(
                    response
                        .hits()
                        .iter()
                        .map(|h| (h.id.clone(), h.evidence.score.to_bits()))
                        .collect::<Vec<_>>(),
                    expected
                );
                returned = response.hits().len();
                covering = response
                    .hits()
                    .iter()
                    .filter(|h| query.split_whitespace().all(|q| h.excerpt.contains(q)))
                    .count();
                for h in response.hits() {
                    assert_eq!(
                        h.span.slice(&index.get(&h.id).unwrap().text),
                        Some(h.excerpt.as_str())
                    );
                }
                if mode == "relevant" {
                    assert_eq!(covering, returned);
                    for (h, d) in response
                        .hits()
                        .iter()
                        .zip(response.excerpt_details().unwrap())
                    {
                        assert_eq!(h.id, d.id);
                        assert!(d
                            .matches
                            .iter()
                            .all(|m| m.span.relative_to(h.span).is_some()));
                    }
                }
                bytes = response.to_json().len();
                assert!(bytes <= options.max_output_bytes());
                black_box(response);
            }
            samples.sort_by(f64::total_cmp);
            reports.push(json!({"mode":mode,"query":query,"median_ms":(samples[14]+samples[15])/2.0,"samples_ms":samples,"output_bytes":bytes,"returned_hits":returned,"excerpts_containing_all_query_words":covering}));
        }
    }
    println!(
        "{}",
        json!({"entries":count,"build_ms":build_ms,"queries":reports,"method":"public synthetic long-prefix records; source build includes generation; five warmups and thirty end-to-end samples per query/mode; exact IDs/scores and original positions verified outside timing; literal coverage is a constructed localization check, not independent relevance evaluation"})
    );
    Ok(())
}
