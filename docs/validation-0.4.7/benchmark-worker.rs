use airust::{retrieval::*, Context, Engine, KnowledgeEntry};
use serde_json::{json, Value};
use std::{
    io::{self, BufRead, Write},
    time::Instant,
};

#[cfg(feature = "candidate")]
struct Delivered(Vec<KnowledgeEntry>);
#[cfg(feature = "candidate")]
impl RetrievalSource for Delivered {
    fn retrieve<'a>(
        &'a mut self,
        _query: &str,
        _options: &RetrievalOptions,
    ) -> Result<RetrievalBatch<'a>, RetrievalError> {
        Ok(RetrievalBatch::new(
            "benchmark-delivered-v1",
            self.0
                .iter()
                .map(|entry| RetrievalCandidate {
                    entry,
                    evidence: RankingEvidence::new(RankingKind::Custom, 1.0),
                })
                .collect(),
        ))
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    let path = std::env::args().nth(1).expect("chunk input path");
    let chunks: Vec<Value> = serde_json::from_slice(&std::fs::read(path)?)?;
    let entries: Vec<_> = chunks
        .iter()
        .map(|c| {
            let text = c["text"].as_str().unwrap();
            assert_eq!(
                airust::text::tokenize(text).join(" "),
                c["lexical"].as_str().unwrap()
            );
            KnowledgeEntry::new(c["id"].as_str().unwrap(), text)
                .with_metadata(json!({"project":c["project"]}))
        })
        .collect();
    let index = Engine::from_entries(entries)?;
    drop(chunks);
    println!(
        "{}",
        json!({"ready":true,"version":airust::VERSION,"entries":index.len(),"build_s":start.elapsed().as_secs_f64()})
    );
    io::stdout().flush()?;
    for line in io::stdin().lock().lines() {
        let line = line?;
        assert!(line.len() <= 256 * 1024);
        let request: Value = serde_json::from_str(&line)?;
        let scope = request["project"].as_str().unwrap();
        assert!(["airust", "flasm", "vecasm"].contains(&scope));
        let mode = request["mode"].as_str().unwrap();
        if mode == "check" {
            #[cfg(feature = "candidate")]
            {
                use airust::retrieval::citations::*;
                let limits = CitationLimits::default();
                let admission_start = Instant::now();
                let parsed =
                    CitedAnswer::from_json(request["answer_json"].as_str().unwrap(), &limits);
                let admission_ms = admission_start.elapsed().as_secs_f64() * 1000.0;
                let mut entries = Vec::new();
                let mut positions = std::collections::BTreeMap::new();
                let passages = request["passages"].as_array().unwrap();
                assert!(passages.len() <= 6);
                let mut total = 0;
                for p in passages {
                    let id = p["id"].as_str().unwrap();
                    let entry = index.get(id).expect("known source ID");
                    assert_eq!(entry.metadata.as_ref().unwrap()["project"], scope);
                    let from = p["start_byte"].as_u64().unwrap() as usize;
                    let to = p["end_byte"].as_u64().unwrap() as usize;
                    let text = p["text"].as_str().unwrap();
                    assert_eq!(entry.text.get(from..to), Some(text));
                    total += text.len();
                    assert!(total <= 12000);
                    assert!(positions.insert(id.to_owned(), from).is_none());
                    entries.push(KnowledgeEntry::new(id, text));
                }
                let catalog = Retriever::new(Delivered(entries)).query(
                    "citation validation",
                    &RetrievalOptions::new(6).with_byte_budgets(16384, 12000, 12000, 128 * 1024)?,
                )?;
                let check_start = Instant::now();
                let output = match parsed {
                    Ok(answer) => {
                        let report = CitationRegistry::new(&catalog).check(&answer, &limits)?;
                        let mut checks = serde_json::to_value(&report)?;
                        for check in checks["checks"].as_array_mut().unwrap() {
                            let id = check["id"].as_str().unwrap().to_owned();
                            if let Some(span) = check["span"].as_object_mut() {
                                let offset = positions[&id];
                                let from = span["start"].as_u64().unwrap() as usize + offset;
                                let to = span["end"].as_u64().unwrap() as usize + offset;
                                assert!(index.get(&id).unwrap().text.get(from..to).is_some());
                                check["span"] = json!({"start":from,"end":to});
                            }
                        }
                        json!({"admitted":true,"report":checks})
                    }
                    Err(error) => json!({"admitted":false,"error":error.to_string()}),
                };
                println!(
                    "{}",
                    json!({"validation":output,"admission_ms":admission_ms,"check_ms":check_start.elapsed().as_secs_f64()*1000.0})
                );
                io::stdout().flush()?;
                continue;
            }
            #[cfg(not(feature = "candidate"))]
            panic!("quote checking requires the candidate build");
        }
        let query = request["query"].as_str().unwrap();
        #[allow(unused_mut)]
        let mut options = RetrievalOptions::new(6)
            .with_candidates(100)?
            .with_context(Context::new().with_metadata("project", scope))
            .with_byte_budgets(16384, 12000, 2000, 128 * 1024)?;
        #[cfg(feature = "coverage")]
        if mode == "trace" {
            let source = LexicalCoverageSource::new(&index);
            let trace = source.inspect(query, &options)?;
            println!("{}", json!({"recipe":trace.recipe(),"effective":trace.analysis().map(|a|a.effective_query()),"hits":trace.hits().iter().map(|h|json!({"id":h.entry().id,"score":h.score(),"lexical_rank":h.lexical_rank(),"lexical_score":h.lexical_score(),"normalized":h.normalized_score(),"matched_query_terms":h.matched_query_terms(),"query_terms":h.query_terms(),"coverage":h.query_coverage()})).collect::<Vec<_>>()}));
            io::stdout().flush()?;
            continue;
        }
        if mode == "evidence" || mode == "prepared" {
            #[cfg(feature = "candidate")]
            {
                options = options
                    .with_distinct_evidence()
                    .with_relevant_excerpts(ExcerptOptions::default().with_leading_context());
            }
            #[cfg(not(feature = "candidate"))]
            panic!("evidence selection requires the candidate build");
        } else {
            assert_eq!(mode, "baseline");
        }
        let start = Instant::now();
        #[allow(unused_mut)]
        let mut source = LexicalSource::new(&index);
        #[cfg(feature = "candidate")]
        if mode == "evidence" || mode == "prepared" {
            source = source.with_query_analysis(LexicalQueryOptions::default());
        }
        #[cfg(feature = "coverage")]
        let response = if mode == "evidence" {
            Retriever::new(LexicalCoverageSource::new(&index)).query(query, &options)?
        } else {
            Retriever::new(source).query(query, &options)?
        };
        #[cfg(not(feature = "coverage"))]
        let response = Retriever::new(source).query(query, &options)?;
        println!(
            "{}",
            json!({"response":serde_json::from_str::<Value>(response.to_json())?,"search_ms":start.elapsed().as_secs_f64()*1000.0})
        );
        io::stdout().flush()?;
    }
    Ok(())
}
