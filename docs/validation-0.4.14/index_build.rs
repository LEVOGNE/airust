use airust::{Engine, KnowledgeEntry};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{hint::black_box, time::Instant};
#[cfg(feature = "heap")]
#[global_allocator]
static ALLOCATOR: dhat::Alloc = dhat::Alloc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let mode = args.get(1).ok_or("mode")?;
    let fixture = args.get(2).ok_or("fixture")?;
    let entries: Vec<KnowledgeEntry> = match fixture.as_str() {
        "canonical" => serde_json::from_reader(std::io::BufReader::new(std::fs::File::open(args.get(3).ok_or("source")?)?))?,
        "ascii" => (0..5000).map(|n|KnowledgeEntry::new(n.to_string(),format!("  HEADER\tRust memory\u{b}ordering  document{n} source quote\r\n{}", "Alpha beta gamma delta. ".repeat(12)))).collect(),
        "mixed" => (0..5000).map(|n|KnowledgeEntry::new(n.to_string(),format!(" ＲＵＳＴ café cafe\u{301}\t東京 source{n} {}", "Memory Straße ﬃ. ".repeat(12)))).collect(),
        "long_tokens" => (0..500).map(|n|KnowledgeEntry::new(n.to_string(),format!("{}{} {}", "A".repeat(1024), n, "word ".repeat(8)))).collect(),
        _ => return Err("fixture".into()),
    };
    let source_bytes: usize = entries.iter().map(|e|e.text.len()).sum();
    #[cfg(feature = "heap")]
    let profiler = if mode == "heap" {Some(dhat::Profiler::builder().file_name(args.get(4).ok_or("profile")?).build())} else {None};
    #[cfg(feature = "heap")]
    let before = dhat::HeapStats::get();
    let start = Instant::now();
    let engine = Engine::from_entries(black_box(entries))?;
    let build_ms = start.elapsed().as_secs_f64()*1000.;
    #[cfg(feature = "heap")]
    let after = dhat::HeapStats::get();
    #[cfg(feature = "heap")]
    drop(profiler);
    let checks: Vec<_> = ["memory ordering", "vector matrix", "function", "source quote", "café ﬃ", "missingtoken978923"]
        .iter().map(|q|engine.search(q,10).into_iter().map(|h|(&h.entry.id,h.score.to_bits(),h.matched_terms)).collect::<Vec<_>>()).collect();
    let digest = format!("{:x}",Sha256::digest(serde_json::to_vec(&checks)?));
    let stats = engine.stats();
    let mut result = json!({"version":airust::VERSION,"fixture":fixture,"source_text_bytes":source_bytes,"stats":stats,"query_signature":digest,"build_ms":build_ms,"instrumented":cfg!(feature="heap"),"input_parse_excluded":true});
    #[cfg(feature = "heap")]
    {
        result["allocation_blocks"]=json!(after.total_blocks-before.total_blocks);
        result["allocated_bytes"]=json!(after.total_bytes-before.total_bytes);
        result["live_bytes_after_build"]=json!(after.curr_bytes as i64-before.curr_bytes as i64);
        result["live_blocks_after_build"]=json!(after.curr_blocks as i64-before.curr_blocks as i64);
    }
    #[cfg(not(feature = "heap"))]
    {assert_eq!(mode,"time");let _=&mut result;}
    println!("{result}");
    Ok(())
}
