use airust::{Engine, retrieval::{Retriever,LexicalSource,RetrievalOptions,ExcerptOptions,EvidencePolicy}, SearchOptions, SearchAlgorithm};
use serde_json::{Value,json};
fn main()->Result<(),Box<dyn std::error::Error>>{
 let args=std::env::args().skip(1).collect::<Vec<_>>();
 let index=Engine::from_json(&std::fs::read_to_string(&args[0])?)?;
 let queries:Vec<Value>=serde_json::from_str(&std::fs::read_to_string(&args[1])?)?;
 let mut expected=Vec::new();
 for query in queries {
  let query=query.as_str().or_else(||query["query"].as_str()).unwrap();let mut row=json!({});
  for (name,algorithm) in [("bm25",SearchAlgorithm::Bm25),("tfidf",SearchAlgorithm::Tfidf)] {
   let options=RetrievalOptions::new(5).with_candidates(100)?.with_byte_budgets(4096,8192,512,16384)?.with_relevant_excerpts(ExcerptOptions::default()).with_evidence_policy(EvidencePolicy::default());
   let response=Retriever::new(LexicalSource::new(&index).with_options(SearchOptions{algorithm,..Default::default()})).query(query,&options)?;
   row[name]=serde_json::from_str(response.to_json())?;
  }
  expected.push(row);
 }
 let mut file=std::fs::OpenOptions::new().write(true).create_new(true).open(&args[2])?;
 serde_json::to_writer(&mut file,&expected)?;println!("checked {} entries, {} queries", index.len(),expected.len());Ok(())
}
