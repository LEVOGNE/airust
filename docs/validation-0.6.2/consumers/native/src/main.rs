#![forbid(unsafe_code)]
use airust::{KnowledgeEntry,reranking::{rerank,RerankOptions,process::{ProcessOptions,ProcessReranker}}};
use serde::Deserialize;
#[derive(Deserialize)]struct Case{query:String,candidates:Vec<KnowledgeEntry>}
fn main()->Result<(),Box<dyn std::error::Error>>{
 let args:Vec<_>=std::env::args().skip(1).collect();assert_eq!(args.len(),4);
 let cases:Vec<Case>=serde_json::from_str(include_str!("../cases.json"))?;
 let expected:Vec<Vec<(String,u64)>>=serde_json::from_str(include_str!("../oracle.json"))?;
 let copies=tempfile::tempdir_in(&args[3])?;
 for cycle in 0..8{
  let case=&cases[cycle%4];
  let mut scorer=ProcessReranker::multilingual_minilm_v2_from_snapshot(ProcessOptions::new(&args[0]),&args[1],&args[2],2,copies.path())?;
  let pid=scorer.worker_id();
  let options=RerankOptions::new(20).with_candidates(20)?.with_batch_size(1)?;
  let outcome=rerank(&case.query,&case.candidates,&mut scorer,&options)?;
  let actual:Vec<_>=outcome.results().iter().map(|r|(r.entry().id.clone(),r.score.unwrap().to_bits())).collect();
  assert_eq!(actual,expected[cycle%4]);
  drop(scorer);
  assert_eq!(std::fs::read_dir(copies.path())?.count(),0);
  assert!(!std::process::Command::new("/bin/kill").args(["-0",&pid.to_string()]).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status()?.success());
 }
 println!("8 real standalone-worker cycles; all160 ID/order/score bits match baseline; all workers reaped and private copies removed");
 Ok(())
}
