use airust::{execution::{CancellationToken, WorkControl}, retrieval::*, Engine, SearchAlgorithm, SearchOptions};
use serde_json::json;
use std::{hint::black_box, io::{self, BufRead, Write}, sync::{Arc, Mutex}, time::{Duration, Instant}};
const QUERIES: [&str; 10] = ["memory ordering", "vector matrix", "type inference", "permission", "array broadcasting", "object stream", "parallel thread", "function", "memory", "airustabsentmarker723981"];
fn query(engine: &Engine, text: &str, algorithm: SearchAlgorithm, mode: &str) -> Result<RetrievalResponse, Box<dyn std::error::Error>> {
    let mut options = RetrievalOptions::new(5).with_candidates(100)?.with_byte_budgets(4096,8192,512,16384)?.with_relevant_excerpts(ExcerptOptions::default()).with_evidence_policy(EvidencePolicy::default());
    if mode != "plain" {
        let mut control = WorkControl::new(CancellationToken::new()).with_timeout(Duration::from_secs(60))?;
        if mode == "progress" {
            let state = Arc::new(Mutex::new(None));
            control = control.with_observer(move |event: &airust::execution::ProgressEvent| { if let Ok(mut latest)=state.lock(){*latest=Some(*event);} });
        }
        options=options.with_control(control);
    }
    Ok(Retriever::new(LexicalSource::new(engine).with_options(SearchOptions{algorithm,..Default::default()})).query(text,&options)?)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args=std::env::args().collect::<Vec<_>>();let mode=&args[2];let count=args[3].parse::<usize>()?;
    assert!(["plain","deadline","progress","matrix"].contains(&mode.as_str()));assert!((1..=1000).contains(&count));
    let engine=Engine::from_json(&std::fs::read_to_string(&args[1])?)?;
    let selected_mode=mode.as_str();
    let modes: &[&str]=if mode=="matrix" {&["plain","deadline","progress"]}else{std::slice::from_ref(&selected_mode)};
    let mut responses=Vec::new();let mut cardinalities=Vec::new();
    for algorithm in [SearchAlgorithm::Bm25,SearchAlgorithm::Tfidf] {for text in QUERIES { let expected=query(&engine,text,algorithm,"plain")?.to_json().to_owned();for actual_mode in modes {assert_eq!(expected,query(&engine,text,algorithm,actual_mode)?.to_json());}responses.push(expected); }}
    for text in QUERIES {
        let total=Arc::new(std::sync::atomic::AtomicUsize::new(0));let captured=total.clone();
        let control=WorkControl::default().with_observer(move |event: &airust::execution::ProgressEvent|{if event.phase()==airust::execution::WorkPhase::Ranking {if let Some(value)=event.total(){captured.store(value,std::sync::atomic::Ordering::Relaxed);}}});
        let _ = engine.search_controlled(text,&SearchOptions{limit:1,..Default::default()},&control)?;
        cardinalities.push(total.load(std::sync::atomic::Ordering::Relaxed));
    }
    println!("{}",json!({"ready":true,"entries":engine.len(),"mode":mode,"version":airust::VERSION,"candidate_counts":cardinalities,"responses":responses}));io::stdout().flush()?;
    let mut line=String::new();io::stdin().lock().read_line(&mut line)?;
    let mut rows=Vec::new();
    for round in 0..count { for (number,text) in QUERIES.iter().enumerate() { for algorithm in [SearchAlgorithm::Bm25,SearchAlgorithm::Tfidf] { for offset in 0..modes.len() { let mode=modes[(offset+round)%modes.len()];
        let start=Instant::now();let response=query(&engine,black_box(text),algorithm,mode)?;let nanos=start.elapsed().as_nanos();
        let signature=response.hits().iter().fold(0u64,|a,h|a.wrapping_add(h.evidence.score.to_bits()).wrapping_add(h.id.len()as u64));black_box(&response);
        rows.push(json!({"round":round,"query_number":number,"algorithm":format!("{algorithm:?}"),"elapsed_ns":nanos,"signature":signature,"control":mode}));
    }}}}
    println!("{}",json!({"version":airust::VERSION,"mode":mode,"rows":rows}));Ok(())
}
