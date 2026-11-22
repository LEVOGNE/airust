use airust::{Engine, KnowledgeEntry, matching::{MatchOptions,MatchStrategy}, execution::WorkControl};
use serde_json::{Value,json};
use std::{error::Error,fs,time::Instant};
fn main()->Result<(),Box<dyn Error>>{
 let args=std::env::args().skip(1).collect::<Vec<_>>();
 let count:usize=args[1].parse()?;
 let all:Vec<KnowledgeEntry>=serde_json::from_str(&fs::read_to_string(&args[0])?)?;
 let parsed=all.len();let entries:Vec<_>=all.into_iter().take(count).collect();
 let queries:Vec<String>=entries.iter().take(12).map(|e|e.text.clone()).collect();
 let short_queries:Vec<String>=entries.iter().filter(|e|(24..=128).contains(&e.text.chars().count())).take(12).map(|e|e.text.clone()).collect();
 assert!(!short_queries.is_empty(),"Select a prefix with a 24..128 character record for bounded positive fuzzy probes");
 let current=Engine::from_entries(entries.clone())?;
 let previous=previous::Engine::from_entries(entries.iter().map(|e|previous::KnowledgeEntry{id:e.id.clone(),text:e.text.clone(),metadata:e.metadata.clone()}))?;
 let mut samples=Vec::new();let mut compared=0;
 for (q,full_query) in queries.iter().enumerate(){for strategy in [MatchStrategy::Exact,MatchStrategy::Fuzzy,MatchStrategy::TokenOverlap]{
 let edited=format!("{} {q}",short_queries[q%short_queries.len()]);
 let query=if strategy==MatchStrategy::Fuzzy {edited.as_str()}else{full_query.as_str()};
 for threshold in [0.5,0.8]{
  let options=MatchOptions{limit:10,strategy,min_similarity:threshold};
  let old_options=previous::matching::MatchOptions{limit:10,min_similarity:threshold,strategy:match strategy{MatchStrategy::Exact=>previous::matching::MatchStrategy::Exact,MatchStrategy::Fuzzy=>previous::matching::MatchStrategy::Fuzzy,MatchStrategy::TokenOverlap=>previous::matching::MatchStrategy::TokenOverlap}};
  let (old,new,old_us,new_us)=if q%2==0 {
   let start=Instant::now();let old=previous.match_with_options(query,&old_options)?;let old_us=start.elapsed().as_secs_f64()*1e6;
   let start=Instant::now();let new=current.match_with_options(query,&options)?;let new_us=start.elapsed().as_secs_f64()*1e6;(old,new,old_us,new_us)
  }else{
   let start=Instant::now();let new=current.match_with_options(query,&options)?;let new_us=start.elapsed().as_secs_f64()*1e6;
   let start=Instant::now();let old=previous.match_with_options(query,&old_options)?;let old_us=start.elapsed().as_secs_f64()*1e6;(old,new,old_us,new_us)
  };
  assert!(!new.is_empty(),"Positive source-derived probe returned no matches");
  let old:Value=serde_json::to_value(old)?;let new:Value=serde_json::to_value(new)?;assert_eq!(old,new);
  let controlled=current.match_controlled(query,&options,&WorkControl::default())?;assert_eq!(new,serde_json::to_value(controlled)?);
  compared+=1;samples.push(json!({"query_number":q,"strategy":format!("{strategy:?}"),"threshold":threshold,"previous_us":old_us,"current_us":new_us}));
 }}}
 for entry in entries.iter().take(12){assert_eq!(serde_json::to_value(previous.relate(&entry.id,10)?)?,serde_json::to_value(current.relate(&entry.id,10)?)?);assert_eq!(serde_json::to_value(current.relate(&entry.id,10)?)?,serde_json::to_value(current.relate_controlled(&entry.id,10,&WorkControl::default())?)?);}
 let result=json!({"version":airust::VERSION,"previous":"0.4.3","parsed_records":parsed,"indexed_records":current.len(),"matching_cases":compared,"all_matching_cases_returned_hits":true,"fuzzy_source_entries":short_queries.len(),"relation_cases":12,"old_new_and_controlled_results_equal":true,"samples":samples,"scope":"Paired warm affected-core calls; result serialization and controlled oracle calls excluded from timings. No relevance judgments."});
 let mut output=fs::OpenOptions::new().write(true).create_new(true).open(&args[2])?;serde_json::to_writer_pretty(&mut output,&result)?;println!("{}",json!({"parsed_records":parsed,"indexed_records":current.len(),"matching_cases":compared,"all_matching_cases_returned_hits":true,"fuzzy_source_entries":short_queries.len(),"relation_cases":12,"all_equal":true}));Ok(())
}
