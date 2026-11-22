use airust::{Engine, KnowledgeEntry, SearchAlgorithm, SearchOptions};
use serde_json::{json, Value};
use std::{
    error::Error,
    fs,
    io::Write,
    path::Path,
    time::{Duration, Instant},
};

type Signature = Vec<(String, u64, Vec<String>)>;
fn next(state: &mut u64) -> usize {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    (*state >> 24) as usize
}
fn old_entry(e: &KnowledgeEntry) -> previous::KnowledgeEntry {
    previous::KnowledgeEntry {
        id: e.id.clone(),
        text: e.text.clone(),
        metadata: e.metadata.clone(),
    }
}
fn current_signature(index: &Engine, q: &str, tfidf: bool, limit: usize) -> Signature {
    index
        .search_with_options(
            q,
            &SearchOptions {
                limit,
                algorithm: if tfidf {
                    SearchAlgorithm::Tfidf
                } else {
                    SearchAlgorithm::Bm25
                },
                ..Default::default()
            },
        )
        .into_iter()
        .map(|h| {
            assert!(h.score.is_finite());
            (h.entry.id.clone(), h.score.to_bits(), h.matched_terms)
        })
        .collect()
}
fn previous_signature(index: &previous::Engine, q: &str, tfidf: bool, limit: usize) -> Signature {
    index
        .search_with_options(
            q,
            &previous::SearchOptions {
                limit,
                algorithm: if tfidf {
                    previous::SearchAlgorithm::Tfidf
                } else {
                    previous::SearchAlgorithm::Bm25
                },
                ..Default::default()
            },
        )
        .into_iter()
        .map(|h| {
            assert!(h.score.is_finite());
            (h.entry.id.clone(), h.score.to_bits(), h.matched_terms)
        })
        .collect()
}
fn save_new(path: &Path, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    f.write_all(bytes)?;
    f.sync_all()?;
    Ok(())
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 6 {
        return Err(
            "usage: soak SOURCE_JSON QUERIES_JSON PREFIX SEED SECONDS NEW_OUTPUT_DIR".into(),
        );
    }
    let prefix: usize = args[2].parse()?;
    let mut state: u64 = args[3].parse()?;
    let seed = state;
    let seconds: u64 = args[4].parse()?;
    if !(1..=100000).contains(&prefix) || !(1..=28800).contains(&seconds) {
        return Err("prefix must be 1..100000 and seconds 1..28800".into());
    }
    let output = Path::new(&args[5]);
    fs::create_dir(output)?;
    let all: Vec<KnowledgeEntry> = serde_json::from_slice(&fs::read(&args[0])?)?;
    let source: Vec<_> = all.into_iter().take(prefix).collect();
    if source.is_empty() {
        return Err("empty corpus".into());
    }
    let mut queries: Vec<String> = serde_json::from_slice::<Vec<Value>>(&fs::read(&args[1])?)?
        .into_iter()
        .map(|v| {
            v.as_str()
                .or_else(|| v["query"].as_str())
                .unwrap()
                .to_owned()
        })
        .collect();
    queries.extend(
        [
            "",
            "--- 😀",
            "café 東京",
            "alpha alpha beta",
            "unfindableqqqq",
            "Straße Häuser",
            "a-2 123",
        ]
        .into_iter()
        .map(str::to_owned),
    );
    let mut index = Engine::from_entries(source.clone())?;
    let mut old = previous::Engine::from_entries(source.iter().map(old_entry))?;
    let start = Instant::now();
    let mut epoch = 0;
    let mut mutations = 0;
    let mut comparisons = 0;
    let mut concurrent_checks = 0;
    let mut cache_pairs = 0;
    while start.elapsed() < Duration::from_secs(seconds) || epoch == 0 {
        let retained = index.clone();
        let old_retained = old.clone();
        let retained_query = &queries[next(&mut state) % queries.len()];
        let retained_signature = current_signature(&retained, retained_query, true, 11);
        for op in 0..128 {
            let mut e = source[next(&mut state) % source.len()].clone();
            match next(&mut state) % 8 {
                0 => {
                    assert_eq!(
                        index.remove(&e.id).map(|e| json!(e)),
                        old.remove(&e.id).map(|e| json!(e))
                    );
                }
                1 => {
                    e.metadata = Some(json!({"epoch":epoch,"op":op,"seed":seed}));
                    assert_eq!(
                        index.upsert(e.clone())?.map(|e| json!(e)),
                        old.upsert(old_entry(&e))?.map(|e| json!(e))
                    );
                }
                2 => {
                    e.text = " \n\t".into();
                    assert!(index.upsert(e.clone()).is_err());
                    assert!(old.upsert(old_entry(&e)).is_err());
                }
                _ => {
                    if op % 11 == 0 {
                        e.id = format!("soak/{}\0/原", next(&mut state) % 128);
                        let count = [0, 1, 2, 1023, 1024, 1025, 2049][next(&mut state) % 7];
                        e.text = if count == 0 {
                            "😀 !!!".into()
                        } else {
                            (0..count)
                                .map(|i| {
                                    if i % 7 == 0 {
                                        format!("word{i}")
                                    } else {
                                        "ＣＡＦÉ".into()
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join(" ")
                        };
                    } else if op % 3 == 0 {
                        e.text.push_str("\nalpha beta Straße café 東京");
                    }
                    assert_eq!(
                        index.upsert(e.clone())?.map(|e| json!(e)),
                        old.upsert(old_entry(&e))?.map(|e| json!(e))
                    );
                }
            }
            mutations += 1;
        }
        assert_eq!(index.to_json()?, old.to_json()?);
        let rebuilt = Engine::from_entries(index.entries().cloned())?;
        assert_eq!(index.stats(), rebuilt.stats());
        for _ in 0..20 {
            let q = &queries[next(&mut state) % queries.len()];
            let limit = [0, 1, 7, 21, usize::MAX][next(&mut state) % 5];
            for tfidf in [false, true] {
                let actual = current_signature(&index, q, tfidf, limit);
                assert_eq!(
                    actual,
                    previous_signature(&old, q, tfidf, limit),
                    "registry divergence at epoch {epoch}"
                );
                assert_eq!(
                    actual,
                    current_signature(&rebuilt, q, tfidf, limit),
                    "rebuild divergence at epoch {epoch}"
                );
                comparisons += 2;
            }
        }
        assert_eq!(
            current_signature(&retained, retained_query, true, 11),
            retained_signature
        );
        assert_eq!(
            retained_signature,
            previous_signature(&old_retained, retained_query, true, 11)
        );
        std::thread::scope(|scope| {
            let handles: Vec<_> = (0..4)
                .map(|worker| {
                    let index = &index;
                    let rebuilt = &rebuilt;
                    let queries = &queries;
                    scope.spawn(move || {
                        for q in queries.iter().skip(worker).step_by(19) {
                            assert_eq!(
                                current_signature(index, q, true, 11),
                                current_signature(rebuilt, q, true, 11)
                            );
                        }
                    })
                })
                .collect();
            for handle in handles {
                handle.join().unwrap();
            }
        });
        concurrent_checks += (0..4)
            .map(|w| queries.iter().skip(w).step_by(19).count())
            .sum::<usize>();
        let control = airust::execution::WorkControl::default();
        control.token().cancel();
        assert!(matches!(
            index.search_controlled("alpha", &SearchOptions::default(), &control),
            Err(airust::execution::ExecutionError::Cancelled)
        ));
        let control = airust::execution::WorkControl::default().with_deadline(Instant::now());
        assert!(matches!(
            index.search_controlled("alpha", &SearchOptions::default(), &control),
            Err(airust::execution::ExecutionError::DeadlineExceeded)
        ));
        if epoch % 256 == 0 {
            let a = output.join(format!("current-{epoch}.cache"));
            let b = output.join(format!("previous-{epoch}.cache"));
            let cache = airust::cache::IndexCache::new(
                &a,
                airust::cache::CacheIdentity::new("overnight-soak")?,
            );
            let old_cache = previous::cache::IndexCache::new(
                &b,
                previous::cache::CacheIdentity::new("overnight-soak")?,
            );
            cache.save(&index)?;
            old_cache.save(&old)?;
            let mut originals = index.entries().cloned().collect();
            let mut old_originals = old.entries().cloned().collect();
            let loaded = airust::cache::IndexCache::new(
                &b,
                airust::cache::CacheIdentity::new("overnight-soak")?,
            )
            .load(&mut originals)?;
            let old_loaded = previous::cache::IndexCache::new(
                &a,
                previous::cache::CacheIdentity::new("overnight-soak")?,
            )
            .load(&mut old_originals)?;
            assert!(originals.is_empty() && old_originals.is_empty());
            for q in queries.iter().step_by(11) {
                assert_eq!(
                    current_signature(loaded.index(), q, false, 11),
                    previous_signature(old_loaded.index(), q, false, 11)
                );
            }
            let damaged = output.join(format!("truncated-{epoch}.cache"));
            let bytes = fs::read(a)?;
            save_new(&damaged, &bytes[..bytes.len() / 2])?;
            let mut originals: Vec<_> = index.entries().cloned().collect();
            let preserved = originals.clone();
            assert!(airust::cache::IndexCache::new(
                &damaged,
                airust::cache::CacheIdentity::new("overnight-soak")?
            )
            .load(&mut originals)
            .is_err());
            assert_eq!(originals, preserved);
            cache_pairs += 1;
        }
        epoch += 1;
        eprintln!(
            "epoch={epoch} mutations={mutations} comparisons={comparisons} elapsed_seconds={:.1}",
            start.elapsed().as_secs_f64()
        );
        let summary = json!({"version":airust::VERSION,"previous":"0.4.3","sharing":cfg!(feature="shared"),"seed":seed,"epochs":epoch,"mutations":mutations,"exact_ranking_comparisons":comparisons,"concurrent_reader_checks":concurrent_checks,"reader_threads":4,"cache_cross_load_pairs":cache_pairs,"cancelled_and_expired_calls_checked":epoch*2,"elapsed_seconds":start.elapsed().as_secs_f64(),"initial_entries":source.len(),"final_entries":index.len(),"status":"running","scope":"Fixed-seed differential/rebuild/retained-reader/cache/concurrency stress; no independent relevance or race-freedom proof."});
        fs::write(
            output.join("progress.json"),
            serde_json::to_vec_pretty(&summary)?,
        )?;
    }
    let completed_work_seconds = start.elapsed().as_secs_f64();
    save_new(
        &output.join("final-test-index.json"),
        index.to_json()?.as_bytes(),
    )?;
    let p = output.join("progress.json");
    let mut summary: Value = serde_json::from_slice(&fs::read(&p)?)?;
    summary["status"] = json!("passed");
    summary["elapsed_seconds"] = json!(completed_work_seconds);
    fs::write(&p, serde_json::to_vec_pretty(&summary)?)?;
    save_new(
        &output.join("summary.json"),
        &serde_json::to_vec_pretty(&summary)?,
    )?;
    println!("{}", summary);
    Ok(())
}
