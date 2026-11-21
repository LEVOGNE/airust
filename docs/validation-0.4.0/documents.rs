use airust::{documents::*, Engine, PdfImportLimits, PdfLoader};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::{collections::BTreeSet,error::Error,fs,io::Read,path::Path,time::Instant};
fn sha(bytes:&[u8])->String {format!("{:x}",Sha256::digest(bytes))}
fn load(path:&str,maximum:u64)->Result<String,Box<dyn Error>>{
 let mut data=String::new();fs::File::open(path)?.take(maximum+1).read_to_string(&mut data)?;assert!((data.len()as u64)<=maximum);Ok(data)
}
fn exercise(snapshot:&DocumentSnapshot,queries:&[Value],out:&Path)->Result<Value,Box<dyn Error>>{
 let start=Instant::now();let mut entries=Vec::new();let mut positions=0usize;let mut ids=BTreeSet::new();
 for page in snapshot.pages(){
  let p=snapshot.page_passage(page.page_number())?;let text=p.resolve(snapshot)?;let span=p.span();assert_eq!(snapshot.text().get(span.start_byte()..span.end_byte()),Some(text));
  assert_eq!(p.page_number(),Some(page.page_number()));p.source_reference()?.validate_source_text(snapshot.text())?;
  let count=text.chars().count();
  for character in [0,count/2,count.saturating_sub(1)]{
   let start=text.char_indices().nth(character).map(|(i,_)|i).unwrap_or(text.len());let end=text[start..].chars().next().map(|c|start+c.len_utf8()).unwrap_or(start);
   let sub=p.subspan(snapshot,start,end)?;assert_eq!(sub.resolve(snapshot)?,&text[start..end]);positions+=1;
  }
  let restored=snapshot.passage_from_json(&p.to_json(8192)?,&DocumentLimits::default())?;assert_eq!(p,restored);
  if !text.trim().is_empty(){let entry=p.to_entry(snapshot)?;p.validate_entry(snapshot,&entry)?;assert!(ids.insert(entry.id.clone()));entries.push(entry);}
 }
 let positions_ms=start.elapsed().as_secs_f64()*1000.;
 let start=Instant::now();let current=Engine::from_entries(entries.clone())?;let current_index_ms=start.elapsed().as_secs_f64()*1000.;
 let start=Instant::now();let baseline=previous::Engine::from_entries(entries.iter().map(|e|previous::KnowledgeEntry::new(&e.id,&e.text).with_metadata(e.metadata.clone().unwrap())))?;let previous_index_ms=start.elapsed().as_secs_f64()*1000.;
 let start=Instant::now();let mut probe_count=0;let mut query_results=Vec::new();
 for query in queries {
  let text=query["query"].as_str().unwrap();
  for tfidf in [false,true] {
   let a=current.search_with_options(text,&airust::SearchOptions{limit:10,algorithm:if tfidf{airust::SearchAlgorithm::Tfidf}else{airust::SearchAlgorithm::Bm25},..Default::default()});
   let b=baseline.search_with_options(text,&previous::SearchOptions{limit:10,algorithm:if tfidf{previous::SearchAlgorithm::Tfidf}else{previous::SearchAlgorithm::Bm25},..Default::default()});
   let old:Vec<_>=b.iter().map(|h|json!({"id":h.entry.id,"score_bits":h.score.to_bits(),"matched_terms":h.matched_terms})).collect();
   let new:Vec<_>=a.iter().map(|h|json!({"id":h.entry.id,"score_bits":h.score.to_bits(),"matched_terms":h.matched_terms})).collect();assert_eq!(old,new);probe_count+=1;
   for hit in &a{
    let descriptor=&hit.entry.metadata.as_ref().unwrap()["airust_document"];
    let p=snapshot.passage_from_json(&descriptor.to_string(),&DocumentLimits::default())?;p.validate_entry(snapshot,hit.entry)?;
    assert_eq!(p.resolve(snapshot)?,hit.entry.text);let sub=p.subspan(snapshot,0,hit.entry.text.len())?;assert_eq!(sub.resolve(snapshot)?,hit.entry.text);
   }
   query_results.push(json!({"case":probe_count,"tfidf":tfidf,"ranked":new}));
  }
 }
 let query_check_ms=start.elapsed().as_secs_f64()*1000.;
 let first=snapshot.page_passage(snapshot.pages()[0].page_number())?;let changed=DocumentSnapshot::new(snapshot.document_id(),SourceFingerprint::from_bytes(b"changed-original",&DocumentLimits::default())?,snapshot.recipe(),snapshot.text_space(),"changed text".into(),&DocumentLimits::default())?;
 assert!(matches!(first.resolve(&changed),Err(DocumentError::StalePassage)));assert!(first.resolve(snapshot).is_ok());
 let start=Instant::now();let clones=(0..10000).map(|_|snapshot.clone()).collect::<Vec<_>>();assert!(clones.iter().all(|s|s.text().as_ptr()==snapshot.text().as_ptr()));let clone_ms=start.elapsed().as_secs_f64()*1000.;drop(clones);
 let index=current.to_json()?;assert_eq!(index,Engine::from_json(&index)?.to_json()?);
 fs::write(out.join("entries.json"),index.as_bytes())?;fs::write(out.join("query-results.json"),serde_json::to_vec(&query_results)?)?;
 Ok(json!({"page_entries":entries.len(),"provided_pages":snapshot.pages().len(),"checked_subspans":positions,"all_position_and_reference_checks_passed":true,"query_probes":probe_count,"all_ranked_ids_score_bits_terms_equal_to_0_3_15":true,"all_returned_entry_positions_revalidated":true,"old_revision_rejected_after_source_change":true,"retained_old_snapshot_resolves":true,"roundtrip_entry_json_sha256":sha(index.as_bytes()),"clone_10000_ms":clone_ms,"clones_share_text":true,"positions_ms":positions_ms,"current_index_ms":current_index_ms,"previous_index_ms":previous_index_ms,"query_check_ms":query_check_ms}))
}
fn main()->Result<(),Box<dyn Error>>{
 let args=std::env::args().skip(1).collect::<Vec<_>>();let mode=&args[0];let input=&args[1];let queries:Vec<Value>=serde_json::from_str(&load(&args[2],16*1024*1024)?)?;let out=Path::new(&args[3]);assert!(!out.exists());fs::create_dir(out)?;
 let limits=DocumentLimits::default();let total=Instant::now();let mut timings=json!({});
 let snapshot=if mode=="pdf"{
  let start=Instant::now();let source=SourceFingerprint::from_reader(fs::File::open(input)?,&limits)?;timings["source_hash_ms"]=json!(start.elapsed().as_secs_f64()*1000.);
  let start=Instant::now();let pages=PdfLoader::new().with_limits(PdfImportLimits::new(128*1024*1024,50000,256*1024*1024,500000)?).extract_pages_from_pdf(Path::new(input))?;timings["native_extract_ms"]=json!(start.elapsed().as_secs_f64()*1000.);
  let start=Instant::now();let snapshot=DocumentSnapshot::from_pages(&args[4],source.clone(),"airust-pdf-extraction-0.3.15;backend=0.3.5;clean=true;remove-page-numbers=true;dehyphenate=true;join=LF2",TextSpace::Cleaned,pages.into_iter().map(|p|(p.page_number,p.text)),&limits)?;timings["snapshot_build_ms"]=json!(start.elapsed().as_secs_f64()*1000.);
  assert_eq!(source,SourceFingerprint::from_reader(fs::File::open(input)?,&limits)?);snapshot
 }else{
  assert_eq!(mode,"restore");let encoded=load(input,limits.max_json_bytes as u64)?;let start=Instant::now();let s=DocumentSnapshot::from_json(&encoded,&limits)?;timings["snapshot_restore_ms"]=json!(start.elapsed().as_secs_f64()*1000.);s
 };
 let start=Instant::now();let encoded=snapshot.to_json(limits.max_json_bytes)?;timings["snapshot_export_ms"]=json!(start.elapsed().as_secs_f64()*1000.);
 let start=Instant::now();let restored=DocumentSnapshot::from_json(&encoded,&limits)?;assert_eq!(snapshot.revision(),restored.revision());assert_eq!(snapshot.text(),restored.text());assert_eq!(snapshot.pages(),restored.pages());timings["roundtrip_restore_ms"]=json!(start.elapsed().as_secs_f64()*1000.);
 fs::write(out.join("snapshot.json"),encoded.as_bytes())?;let checks=exercise(&snapshot,&queries,out)?;
 let summary=json!({"version":airust::VERSION,"mode":mode,"document_id":snapshot.document_id(),"revision":snapshot.revision(),"source_sha256":snapshot.source().sha256(),"source_bytes":snapshot.source().byte_len(),"text_sha256":snapshot.text_sha256(),"text_bytes":snapshot.text().len(),"snapshot_json_bytes":encoded.len(),"snapshot_json_sha256":sha(encoded.as_bytes()),"first_page":snapshot.pages().first().map(|p|p.page_number()),"last_page":snapshot.pages().last().map(|p|p.page_number()),"checks":checks,"timings":timings,"elapsed_seconds":total.elapsed().as_secs_f64()});fs::write(out.join("summary.json"),serde_json::to_vec_pretty(&summary)?)?;println!("{}",json!({"mode":mode,"pages":snapshot.pages().len(),"revision":snapshot.revision(),"elapsed_seconds":summary["elapsed_seconds"]}));Ok(())
}
