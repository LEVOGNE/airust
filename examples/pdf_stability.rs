use airust::{Engine, KnowledgeEntry, PdfLoader, SearchAlgorithm, SearchOptions};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, error::Error, fs, time::Instant};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if !(2..=3).contains(&args.len()) {
        return Err("usage: pdf_stability INPUT.pdf QUERIES.json [PRIVATE_INDEX_EXPORT.json]; stdout contains IDs and hashes, not document text".into());
    }
    let queries: Vec<Value> = serde_json::from_str(&fs::read_to_string(&args[1])?)?;
    let start = Instant::now();
    let examples = PdfLoader::new().pdf_to_training_examples(&args[0])?;
    let import_ms = start.elapsed().as_secs_f64() * 1000.0;
    let content_sha256 = format!("{:x}", Sha256::digest(serde_json::to_vec(&examples)?));
    let mut pages = BTreeSet::new();
    let mut chars = 0usize;
    let entries = examples
        .into_iter()
        .map(|example| {
            let metadata = example.metadata.unwrap();
            let page = metadata["page_number"].as_u64().unwrap();
            let chunk = metadata["chunk_index"].as_u64().unwrap();
            pages.insert(page);
            chars += example.input.chars().count();
            assert!(example.input.chars().count() <= 1000);
            KnowledgeEntry::new(format!("p{page:05}-c{chunk:05}"), example.input)
                .with_metadata(metadata)
        })
        .collect::<Vec<_>>();
    let start = Instant::now();
    let engine = Engine::from_entries(entries)?;
    let build_ms = start.elapsed().as_secs_f64() * 1000.0;
    if let Some(path) = args.get(2) {
        use std::io::Write;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        file.write_all(engine.to_json()?.as_bytes())?;
    }
    let mut cases = Vec::new();
    for (i, query) in queries.iter().enumerate() {
        let text = query["query"].as_str().ok_or("missing query text")?;
        let expected_page = query["page"].as_u64().ok_or("missing expected page")?;
        for algorithm in [SearchAlgorithm::Bm25, SearchAlgorithm::Tfidf] {
            let mut canonical = None;
            let mut times = Vec::new();
            for _ in 0..5 {
                let start = Instant::now();
                let hits = engine.search_with_options(
                    text,
                    &SearchOptions {
                        algorithm,
                        limit: 10,
                        ..Default::default()
                    },
                );
                times.push(start.elapsed().as_secs_f64() * 1000.0);
                let ids = hits.iter().map(|h| h.entry.id.clone()).collect::<Vec<_>>();
                assert!(hits.windows(2).all(|p| p[0].score >= p[1].score));
                assert!(hits.iter().all(|h| h.score.is_finite()));
                if let Some(previous) = &canonical {
                    assert_eq!(previous, &ids);
                } else {
                    canonical = Some(ids);
                }
            }
            let ids = canonical.unwrap();
            let page_rank = ids
                .iter()
                .position(|id| id.starts_with(&format!("p{expected_page:05}-")))
                .map(|p| p + 1);
            cases.push(json!({"case":i,"page":expected_page,"algorithm":format!("{algorithm:?}"),"expected_page_rank":page_rank,"ranked_ids":ids,"latency_ms":times}));
        }
    }
    assert!(engine.search("airustmissingterm9f8301bd", 10).is_empty());
    for term in ["Flash", "the", "und", "ActionScript"] {
        for limit in [1, 10, 100] {
            let all = engine.search(term, engine.len());
            let top = engine.search(term, limit);
            assert_eq!(
                top.iter().map(|h| &h.entry.id).collect::<Vec<_>>(),
                all.iter()
                    .take(limit)
                    .map(|h| &h.entry.id)
                    .collect::<Vec<_>>()
            );
        }
    }
    println!(
        "{}",
        serde_json::to_string(
            &json!({"version":airust::VERSION,"import_ms":import_ms,"index_build_ms":build_ms,"chunks":engine.len(),"chunk_characters":chars,"pages_with_chunks":pages,"content_sha256":content_sha256,"queries":cases,"repeated_ranking_stable":true,"top_k_matches_full_order":true,"limits":"Literal page-retrieval checks from an independent PDF parser; not a semantic relevance benchmark. Hashes are identities, not anonymization."})
        )?
    );
    Ok(())
}
