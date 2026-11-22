use airust::{retrieval::*, Engine, KnowledgeEntry, SearchAlgorithm, SearchOptions};
use serde_json::{json, Value};
use std::{hint::black_box, time::Instant};
#[cfg(feature = "heap")]
#[global_allocator]
static ALLOCATOR: dhat::Alloc = dhat::Alloc;
fn work(mode: &str, query: &str, index: &Engine, wide: &Engine, broad: &Engine) -> u64 {
    match mode {
        "frequencies" => airust::text::term_frequencies(black_box(query)).into_iter().fold(0u64, |s,(t,n)|s.wrapping_add((t.len() as u64).wrapping_mul(n as u64))),
        "search" => index.search(black_box(query),10).iter().fold(0u64,|s,h|s.wrapping_add(h.score.to_bits()).wrapping_add(h.entry.id.len()as u64)),
        "wide" => wide.search(black_box(query),10).iter().fold(0u64,|s,h|s.wrapping_add(h.score.to_bits()).wrapping_add(h.entry.id.len()as u64)),
        "tfidf" => index.search_with_options(black_box(query),&SearchOptions{algorithm:SearchAlgorithm::Tfidf,..Default::default()}).iter().fold(0u64,|s,h|s.wrapping_add(h.score.to_bits())),
        "wide_tfidf" => wide.search_with_options(black_box(query),&SearchOptions{algorithm:SearchAlgorithm::Tfidf,..Default::default()}).iter().fold(0u64,|s,h|s.wrapping_add(h.score.to_bits())),
        "broad" | "broad_tfidf" => broad.search_with_options(black_box(query),&SearchOptions{algorithm:if mode=="broad"{SearchAlgorithm::Bm25}else{SearchAlgorithm::Tfidf},..Default::default()}).iter().fold(0u64,|s,h|s.wrapping_add(h.score.to_bits())),
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
    #[cfg(feature="heap")]
    if mode=="corpus_norm_heap" {
        let records:Vec<KnowledgeEntry>=serde_json::from_reader(std::fs::File::open(&args[4])?)?;
        let corpus=Engine::from_entries(records)?;let count=corpus.len();
        let queries=["memory ordering","vector matrix","type inference","permission","array broadcasting","object stream","parallel thread","function function function function","memory memory memory memory","airustabsentmarker723981"];
        let profiler=dhat::Profiler::builder().file_name(&args[3]).build();let mut rows=Vec::new();
        for (number,query) in queries.into_iter().enumerate() {
            let before=dhat::HeapStats::get();
            let result=corpus.search_with_options(query,&SearchOptions{algorithm:SearchAlgorithm::Tfidf,..Default::default()});
            let signature=result.iter().fold(0u64,|s,h|s.wrapping_add(h.score.to_bits()));drop(result);
            let after=dhat::HeapStats::get();
            rows.push(json!({"query_number":number,"signature":signature,"cache_live_bytes_delta":after.curr_bytes as i64-before.curr_bytes as i64,"cache_live_blocks_delta":after.curr_blocks as i64-before.curr_blocks as i64,"allocation_blocks":after.total_blocks-before.total_blocks,"allocated_bytes":after.total_bytes-before.total_bytes}));
        }
        drop(profiler);println!("{}",json!({"version":airust::VERSION,"entries":count,"rows":rows,"scope":"Cold canonical first pass, query allocation traffic and hint deltas after dropping results; source/index setup excluded; no instrumented latency claim"}));return Ok(());
    }
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
        ("wide","union_32",(0..32).map(|n|format!("term{n} ")).collect()),
        ("wide","union_33",(0..33).map(|n|format!("term{n} ")).collect()),
        ("tfidf","tfidf_warm","common marker provenance store".into()),
        ("wide_tfidf","tfidf_union_32",(0..32).map(|n|format!("term{n} ")).collect()),
        ("wide_tfidf","tfidf_union_33",(0..33).map(|n|format!("term{n} ")).collect()),
        ("broad","sparse_bm25_33",(0..33).map(|n|format!("term{n} ")).collect()),
        ("broad","sparse_bm25_128",(0..128).map(|n|format!("term{n} ")).collect()),
        ("broad","sparse_bm25_512",(0..512).map(|n|format!("term{n} ")).collect()),
        ("broad_tfidf","sparse_tfidf_33",(0..33).map(|n|format!("term{n} ")).collect()),
        ("broad_tfidf","sparse_tfidf_128",(0..128).map(|n|format!("term{n} ")).collect()),
        ("broad_tfidf","sparse_tfidf_512",(0..512).map(|n|format!("term{n} ")).collect()),
    ];
    let index=Engine::from_entries((0..1000).map(|n|KnowledgeEntry::new(n.to_string(),format!("ProvenanceStore provenance store common marker topic{}",n%17))))?;
    let wide=Engine::from_entries((0..256).map(|n|KnowledgeEntry::new(n.to_string(),(0..64).filter(|t|(n+t)%4<2).map(|t|format!("term{t} ")).collect::<String>())))?;
    let broad=Engine::from_entries((0..256).map(|n|KnowledgeEntry::new(format!("sparse{n}"),(0..16).map(|j|format!("term{} ",(n*2+j)%512)).collect::<String>())).chain(std::iter::once(KnowledgeEntry::new("dense",(0..512).map(|n|format!("term{n} ")).collect::<String>()))))?;
    #[cfg(feature="heap")]
    if mode=="norm_heap" {
        let full=(0..512).map(|n|format!("term{n} ")).collect::<String>();
        let profiler=dhat::Profiler::builder().file_name(&args[3]).build();let mut rows=Vec::new();
        for (label,engine,query) in [("index",&index,"common"),("wide",&wide,"term0 term1"),("sparse",&broad,full.as_str())] {
            let before=dhat::HeapStats::get();
            let result=engine.search_with_options(query,&SearchOptions{algorithm:SearchAlgorithm::Tfidf,..Default::default()});
            let signature=result.iter().fold(0u64,|s,h|s.wrapping_add(h.score.to_bits()));drop(result);
            let after=dhat::HeapStats::get();
            rows.push(json!({"fixture":label,"signature":signature,"cache_live_bytes_delta":after.curr_bytes as i64-before.curr_bytes as i64,"cache_live_blocks_delta":after.curr_blocks as i64-before.curr_blocks as i64,"allocation_blocks":after.total_blocks-before.total_blocks,"allocated_bytes":after.total_bytes-before.total_bytes}));
        }
        drop(profiler);println!("{}",json!({"version":airust::VERSION,"scope":"Cold norm-hint retained heap after dropping result; source/index setup excluded","rows":rows}));return Ok(());
    }
    if mode=="cold_time" {
        assert!(!cfg!(feature="heap"));let mut rows=Vec::new();
        let full=(0..512).map(|n|format!("term{n} ")).collect::<String>();
        for repeat in 0..20 {
            for (label,engine,query) in [("index",&index,"common"),("wide",&wide,"term0 term1"),("sparse",&broad,full.as_str())] {
                let fresh=engine.clone();
                let start=Instant::now();let signature=black_box(work("tfidf",query,&fresh,&fresh,&fresh));let nanos=start.elapsed().as_nanos();
                rows.push(json!({"fixture":label,"repeat":repeat,"signature":signature,"elapsed_ns":nanos}));
            }
        }
        println!("{}",json!({"version":airust::VERSION,"rows":rows,"scope":"Fresh cold norm cache on cloned prebuilt index; clone/setup outside timer; normal process and filesystem caches, no DHAT"}));return Ok(());
    }
    for (kind,_,query) in &cases {black_box(work(kind,query,&index,&wide,&broad));}
    let mut rows:Vec<Value>=Vec::with_capacity(cases.len());
    #[cfg(feature="heap")]
    let profiler=if mode=="heap" {Some(dhat::Profiler::builder().file_name(&args[3]).build())}else{None};
    for (kind,label,query) in cases {
        let iterations=if kind.starts_with("broad"){iterations.min(200)}else{iterations};
        #[cfg(feature="heap")]
        let before=dhat::HeapStats::get();
        let start=Instant::now();let mut signature=0u64;
        for _ in 0..iterations {signature=signature.wrapping_add(black_box(work(kind,&query,&index,&wide,&broad)));}
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
