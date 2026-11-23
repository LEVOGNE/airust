use airust::{Context, Engine, KnowledgeEntry, SearchAlgorithm, SearchOptions};
use serde_json::json;
use std::{hint::black_box, time::Instant};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let iterations: usize = std::env::args().nth(1).unwrap().parse()?;
    assert!((1..=10000).contains(&iterations));
    let mut rows = Vec::new();
    let mut full = Vec::new();
    for count in [1usize, 4, 16, 33] {
        let engine = Engine::from_entries((0..count).map(|n| {
            KnowledgeEntry::new(format!("item{n:03}"), format!("common marker café {}", (0..33).filter(|t| t % 3 == n % 3 || n == 0).map(|t| format!("term{t} ")).collect::<String>()))
                .with_metadata(json!({"scope": if n % 2 == 0 {"even"} else {"odd"}}))
        }))?;
        let wide = (0..33).map(|t| format!("term{t} ")).collect::<String>();
        for algorithm in [SearchAlgorithm::Bm25, SearchAlgorithm::Tfidf] {
            for (label, query, limit, context) in [
                ("single", "common", 10, None),
                ("repeated", "common common common common", 10, None),
                ("unknown_added", "common absent", 10, None),
                ("multi", "common marker", 10, None),
                ("unicode", "CAFÉ cafe\u{301}", 10, None),
                ("unknown", "absent", 10, None),
                ("punctuation", "... 🦀", 10, None),
                ("scope", "common", 10, Some(Context::new().with_metadata("scope", "even"))),
                ("disjoint", "common", 10, Some(Context::new().with_metadata("scope", "none"))),
                ("limit_one", "common", 1, None),
                ("limit_zero", "common", 0, None),
                ("wide", wide.as_str(), 10, None),
            ] {
                let options = SearchOptions { algorithm, limit, context, ..Default::default() };
                let expected = engine.search_with_options(query, &options);
                let original = serde_json::to_string(&expected)?;
                full.push(json!({"entries":count,"case":label,"algorithm":format!("{algorithm:?}"),"result":original}));
                let expected_signature = expected.iter().fold(0u64, |s,h| s.wrapping_add(h.score.to_bits()).wrapping_add(h.entry.id.len() as u64).wrapping_add(h.matched_terms.len() as u64));
                drop(expected);
                let start = Instant::now();
                let mut signature = 0u64;
                for _ in 0..iterations {
                    let hits = engine.search_with_options(black_box(query), black_box(&options));
                    let one = black_box(hits.iter().fold(0u64, |s,h| s.wrapping_add(h.score.to_bits()).wrapping_add(h.entry.id.len() as u64).wrapping_add(h.matched_terms.len() as u64)));
                    signature = signature.wrapping_add(one);
                }
                let elapsed_ns = start.elapsed().as_nanos();
                assert_eq!(signature, expected_signature.wrapping_mul(iterations as u64));
                rows.push(json!({"fixture":format!("{count}_{algorithm:?}_{label}"),"entries":count,"case":label,"algorithm":format!("{algorithm:?}"),"iterations":iterations,"signature":signature,"elapsed_ns":elapsed_ns,"ns_per_call":elapsed_ns as f64 / iterations as f64}));
            }
        }
    }
    println!("{}", json!({"version":airust::VERSION,"rows":rows,"complete_results":full,"scope":"Fixed synthetic small Core controls; setup/warmup/complete JSON checks outside timer. Repeated normal process cache state; not independent relevance judgments."}));
    Ok(())
}
