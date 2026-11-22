use airust::{Engine, KnowledgeEntry, SearchAlgorithm, SearchOptions};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{error::Error, fs, path::Path, time::Instant};
fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn save(path: &str, data: &Value) -> Result<(), Box<dyn Error>> {
    let f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    serde_json::to_writer_pretty(f, data)?;
    Ok(())
}
fn new_sig(index: &Engine, q: &str, tfidf: bool) -> Value {
    json!(index
        .search_with_options(
            q,
            &SearchOptions {
                limit: 10,
                algorithm: if tfidf {
                    SearchAlgorithm::Tfidf
                } else {
                    SearchAlgorithm::Bm25
                },
                ..Default::default()
            }
        )
        .iter()
        .map(|h| json!({"id":h.entry.id,"score_bits":h.score.to_bits(),"terms":h.matched_terms}))
        .collect::<Vec<_>>())
}
fn old_sig(index: &previous::Engine, q: &str, tfidf: bool) -> Value {
    json!(index
        .search_with_options(
            q,
            &previous::SearchOptions {
                limit: 10,
                algorithm: if tfidf {
                    previous::SearchAlgorithm::Tfidf
                } else {
                    previous::SearchAlgorithm::Bm25
                },
                ..Default::default()
            }
        )
        .iter()
        .map(|h| json!({"id":h.entry.id,"score_bits":h.score.to_bits(),"terms":h.matched_terms}))
        .collect::<Vec<_>>())
}
fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let mode = &args[0];
    let encoded = fs::read(&args[1])?;
    let source_sha = sha(&encoded);
    let all: Vec<KnowledgeEntry> = serde_json::from_slice(&encoded)?;
    let parsed = all.len();
    drop(encoded);
    let count: usize = args[2].parse()?;
    let mut entries = all.into_iter().take(count).collect::<Vec<_>>();
    entries.shrink_to_fit();
    let query_data = fs::read(&args[3])?;
    let queries: Vec<Value> = serde_json::from_slice(&query_data)?;
    let query_sha = sha(&query_data);
    if mode == "compare" {
        let old_entries = || {
            entries
                .iter()
                .map(|e| previous::KnowledgeEntry {
                    id: e.id.clone(),
                    text: e.text.clone(),
                    metadata: e.metadata.clone(),
                })
                .collect::<Vec<_>>()
        };
        let mut current = Engine::from_entries(entries.clone())?;
        let mut old = previous::Engine::from_entries(old_entries())?;
        let mut checked = 0;
        for _ in 0..2 {
            for q in &queries {
                let q = q.as_str().or_else(|| q["query"].as_str()).unwrap();
                for tfidf in [false, true] {
                    assert_eq!(new_sig(&current, q, tfidf), old_sig(&old, q, tfidf));
                    checked += 1;
                }
            }
        }
        let retained = current.clone();
        let old_retained = old.clone();
        let mut mutations = 0;
        for (n, entry) in entries.iter().take(32).enumerate() {
            let mut changed = entry.clone();
            if n % 3 == 0 {
                changed.metadata = Some(json!({"verified_update":n}));
            } else {
                changed.text.push_str(" airustvalidationchangedterm");
            }
            current.upsert(changed.clone())?;
            old.upsert(previous::KnowledgeEntry {
                id: changed.id,
                text: changed.text,
                metadata: changed.metadata,
            })?;
            mutations += 1;
        }
        for entry in entries.iter().skip(32).take(16) {
            current.remove(&entry.id);
            old.remove(&entry.id);
            mutations += 1;
        }
        current.upsert(KnowledgeEntry::new(
            "airust-validation-new",
            "memory vector alpha beta",
        ))?;
        old.upsert(previous::KnowledgeEntry::new(
            "airust-validation-new",
            "memory vector alpha beta",
        ))?;
        mutations += 1;
        for q in &queries {
            let q = q.as_str().or_else(|| q["query"].as_str()).unwrap();
            for tfidf in [false, true] {
                assert_eq!(new_sig(&current, q, tfidf), old_sig(&old, q, tfidf));
                assert_eq!(
                    new_sig(&retained, q, tfidf),
                    old_sig(&old_retained, q, tfidf)
                );
                checked += 2;
            }
        }
        let path = Path::new(&args[4]);
        assert!(!path.exists());
        fs::create_dir(path)?;
        let a = path.join("current.cache");
        let b = path.join("previous.cache");
        airust::cache::IndexCache::new(&a, airust::cache::CacheIdentity::new("core-validation")?)
            .with_limits(airust::cache::CacheLimits::default().with_max_term_bindings(100_000_000))
            .save(&retained)?;
        previous::cache::IndexCache::new(
            &b,
            previous::cache::CacheIdentity::new("core-validation")?,
        )
        .with_limits(previous::cache::CacheLimits::default().with_max_term_bindings(100_000_000))
        .save(&old_retained)?;
        let first = fs::read(&a)?;
        let second = fs::read(&b)?;
        assert!(
            first.get(..8) == Some(b"AIRUSTC1".as_slice())
                && second.get(..8) == Some(b"AIRUSTC1".as_slice())
        );
        let (producer_new, payload_new) = postcard::take_from_bytes::<String>(&first[64..])?;
        let (producer_old, payload_old) = postcard::take_from_bytes::<String>(&second[64..])?;
        assert_eq!(producer_new, format!("airust/{}", airust::VERSION));
        assert_eq!(producer_old, "airust/0.4.3");
        assert!(
            payload_new == payload_old,
            "Cache metadata/analysis differs after the producer label"
        );
        let mut new_originals = entries.clone();
        let loaded_new = airust::cache::IndexCache::new(
            &b,
            airust::cache::CacheIdentity::new("core-validation")?,
        )
        .with_limits(airust::cache::CacheLimits::default().with_max_term_bindings(100_000_000))
        .load(&mut new_originals)?;
        let mut old_originals = old_entries();
        let loaded_old = previous::cache::IndexCache::new(
            &a,
            previous::cache::CacheIdentity::new("core-validation")?,
        )
        .with_limits(previous::cache::CacheLimits::default().with_max_term_bindings(100_000_000))
        .load(&mut old_originals)?;
        assert!(new_originals.is_empty() && old_originals.is_empty());
        for q in &queries {
            let q = q.as_str().or_else(|| q["query"].as_str()).unwrap();
            for tfidf in [false, true] {
                assert_eq!(
                    new_sig(loaded_new.index(), q, tfidf),
                    old_sig(&old_retained, q, tfidf)
                );
                assert_eq!(
                    old_sig(loaded_old.index(), q, tfidf),
                    new_sig(&retained, q, tfidf)
                );
                checked += 2;
            }
        }
        let result = json!({"mode":mode,"version":airust::VERSION,"previous":"0.4.3","parsed_records":parsed,"indexed_records":entries.len(),"source_sha256":source_sha,"query_sha256":query_sha,"query_comparisons":checked,"mutations":mutations,"all_ranked_ids_score_bits_and_terms_equal":true,"retained_readers_equal":true,"cache_analysis_and_metadata_equal_except_producer":true,"both_cache_versions_cross_load":true,"cache_analysis_payload_sha256":sha(payload_new),"cache_sha256":sha(&first)});
        save(path.join("summary.json").to_str().unwrap(), &result)?;
        println!("{result}");
        return Ok(());
    }
    let mut samples = Vec::new();
    let (build_ms, indexed) = if mode == "previous" {
        let source = entries
            .into_iter()
            .map(|e| previous::KnowledgeEntry {
                id: e.id,
                text: e.text,
                metadata: e.metadata,
            })
            .collect::<Vec<_>>();
        let build = Instant::now();
        let index = previous::Engine::from_entries(source)?;
        let elapsed = build.elapsed().as_secs_f64() * 1000.;
        for pass in 0..3 {
            for (number, q) in queries.iter().enumerate() {
                let q = q.as_str().or_else(|| q["query"].as_str()).unwrap();
                for tfidf in [false, true] {
                    let start = Instant::now();
                    let hits = index.search_with_options(
                        q,
                        &previous::SearchOptions {
                            limit: 10,
                            algorithm: if tfidf {
                                previous::SearchAlgorithm::Tfidf
                            } else {
                                previous::SearchAlgorithm::Bm25
                            },
                            ..Default::default()
                        },
                    );
                    let micros = start.elapsed().as_secs_f64() * 1e6;
                    std::hint::black_box(&hits);
                    samples.push(json!({"pass":pass,"query":number,"algorithm":if tfidf{"tfidf"}else{"bm25"},"microseconds":micros,"hits":hits.len()}));
                }
            }
        }
        (elapsed, index.len())
    } else {
        assert_eq!(mode, "current");
        let build = Instant::now();
        let index = Engine::from_entries(entries)?;
        let elapsed = build.elapsed().as_secs_f64() * 1000.;
        for pass in 0..3 {
            for (number, q) in queries.iter().enumerate() {
                let q = q.as_str().or_else(|| q["query"].as_str()).unwrap();
                for tfidf in [false, true] {
                    let start = Instant::now();
                    let hits = index.search_with_options(
                        q,
                        &SearchOptions {
                            limit: 10,
                            algorithm: if tfidf {
                                SearchAlgorithm::Tfidf
                            } else {
                                SearchAlgorithm::Bm25
                            },
                            ..Default::default()
                        },
                    );
                    let micros = start.elapsed().as_secs_f64() * 1e6;
                    std::hint::black_box(&hits);
                    samples.push(json!({"pass":pass,"query":number,"algorithm":if tfidf{"tfidf"}else{"bm25"},"microseconds":micros,"hits":hits.len()}));
                }
            }
        }
        (elapsed, index.len())
    };
    save(
        &args[4],
        &json!({"mode":mode,"version":if mode=="current"{airust::VERSION}else{"0.4.3"},"parsed_records":parsed,"indexed_records":indexed,"source_sha256":source_sha,"query_sha256":query_sha,"build_ms":build_ms,"samples":samples,"scope":"Release ordinary calls, first pass and two warm passes over a fixed literal query sequence; original source retained. Timings exclude result serialization. Full source parsing happens before build timer."}),
    )?;
    Ok(())
}
