use airust::{retrieval::*, Engine, KnowledgeEntry};
use serde_json::{json, Value};
use std::{hint::black_box, time::Instant};
#[cfg(feature = "heap")]
#[global_allocator]
static ALLOCATOR: dhat::Alloc = dhat::Alloc;
fn work(mode: &str, query: &str, index: &Engine) -> u64 {
    match mode {
        "frequencies" => airust::text::term_frequencies(black_box(query)).into_iter().fold(0u64, |s,(t,n)|s.wrapping_add((t.len() as u64).wrapping_mul(n as u64))),
        "search" => index.search(black_box(query),10).iter().fold(0u64,|s,h|s.wrapping_add(h.score.to_bits()).wrapping_add(h.entry.id.len()as u64)),
        "coverage" => {
            let source=LexicalCoverageSource::new(index);let opts=RetrievalOptions::new(6).with_candidates(100).unwrap();
            let result=source.inspect(black_box(query),&opts).unwrap();
            result.hits().iter().fold(0u64,|s,h|s.wrapping_add(h.score().to_bits()).wrapping_add(h.matched_query_terms()as u64))
        },
        "excerpt" => {
            let result=select_excerpt(black_box(query),"rust memory café ffi",&RankingEvidence::new(RankingKind::Bm25,1.0),&ExcerptAnalyzer::Legacy,&ExcerptOptions::default(),96).unwrap();
            result.matches.iter().fold(result.span.start as u64 ^ result.span.end as u64,|s,m|s.wrapping_add(m.span.start as u64).wrapping_add(m.span.end as u64))
        },
        "retrieval" => {
            let options=RetrievalOptions::new(6).with_candidates(100).unwrap().with_relevant_excerpts(ExcerptOptions::default());
            let result=Retriever::new(LexicalSource::new(index)).query(black_box(query),&options).unwrap();
            result.hits().iter().fold(result.to_json().len() as u64,|s,h|s.wrapping_add(h.evidence.score.to_bits()))
        },
        _=>panic!("unknown mode"),
    }
}
fn main()->Result<(),Box<dyn std::error::Error>> {
    let args:Vec<_>=std::env::args().collect();let mode=&args[1];let iterations:usize=args[2].parse()?;assert!((1..=10000).contains(&iterations));
    let cases:Vec<(&str,&str,String)>=vec![
        ("frequencies","repeated_short","rust ".repeat(256)),
        ("frequencies","mixed_unicode","cafe\u{301} हिंदी 中文 ＲＵＳＴ ".repeat(64)),
        ("frequencies","unique_short",(0..256).map(|n|format!("word{n} ")).collect()),
        ("frequencies","long_repeated",format!("{} ","x".repeat(1023)).repeat(16)),
        ("frequencies","unique_long",(0..256).map(|n|format!("prefix{n:04}abcdefghijklmnopqrstuv ")).collect()),
        ("frequencies","punctuation","... 🦀 \u{301} ".repeat(64)),
        ("search","repeated_search","common ".repeat(128)),
        ("search","unique_search","common marker provenance store unknown".into()),
        ("coverage","coverage_small","Which ProvenanceStore common marker?".into()),
        ("coverage","coverage_repeated","Which ProvenanceStore common marker? ".repeat(16)),
        ("excerpt","excerpt_ascii","Section HEADER Rust memory safety source context.\r\n".repeat(100)),
        ("excerpt","excerpt_mixed","Section café Rust memory safety 中文.\r\n".repeat(80)),
        ("excerpt","excerpt_unicode","👩‍🚀 café ﬃ 中文 cafe\u{301}\r\n".repeat(30)),
        ("retrieval","relevant_retrieval","common provenance store marker".into()),
    ];
    let index=Engine::from_entries((0..1000).map(|n|KnowledgeEntry::new(n.to_string(),format!("ProvenanceStore provenance store common marker topic{}",n%17))))?;
    for (kind,_,query) in &cases {black_box(work(kind,query,&index));}
    let mut rows:Vec<Value>=Vec::with_capacity(cases.len());
    #[cfg(feature="heap")]
    let profiler=if mode=="heap" {Some(dhat::Profiler::builder().file_name(&args[3]).build())}else{None};
    for (kind,label,query) in cases {
        #[cfg(feature="heap")]
        let before=dhat::HeapStats::get();
        let start=Instant::now();let mut signature=0u64;
        for _ in 0..iterations {signature=signature.wrapping_add(black_box(work(kind,&query,&index)));}
        let nanos=start.elapsed().as_nanos();
        #[cfg(feature="heap")]
        let after=dhat::HeapStats::get();
        let mut row=json!({"kind":kind,"fixture":label,"iterations":iterations,"query_bytes":query.len(),"signature":signature,"elapsed_ns":nanos,"ns_per_call":nanos as f64/iterations as f64,"instrumented":cfg!(feature="heap")});
        #[cfg(feature="heap")]
        {row["allocation_blocks"]=json!(after.total_blocks-before.total_blocks);row["allocated_bytes"]=json!(after.total_bytes-before.total_bytes);row["live_blocks_delta"]=json!(after.curr_blocks as i64-before.curr_blocks as i64);row["live_bytes_delta"]=json!(after.curr_bytes as i64-before.curr_bytes as i64);}
        #[cfg(not(feature="heap"))]
        {assert_eq!(mode,"time");let _=&mut row;}
        rows.push(row);
    }
    #[cfg(feature="heap")]
    drop(profiler);
    println!("{}",json!({"version":airust::VERSION,"rows":rows,"scope":"Complete call on a prebuilt index; input/corpus setup excluded. Heap instrumentation is not used for timing comparisons."}));
    Ok(())
}
