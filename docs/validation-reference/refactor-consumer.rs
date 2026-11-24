use airust::{Context, Engine, KnowledgeEntry, SearchAlgorithm, SearchOptions};
use serde_json::{json, Value};
use std::{fs, hint::black_box, path::Path, time::Instant};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn rows(path: &Path) -> Result<Vec<Value>> {
    fs::read_to_string(path)?.lines().map(|line| Ok(serde_json::from_str(line)?)).collect()
}

fn load(directory: &Path, copies: usize) -> Result<Engine> {
    let corpus = rows(&directory.join("corpus.jsonl"))?;
    let mut entries = Vec::new();
    for copy in 0..copies {
        for row in &corpus {
            entries.push(KnowledgeEntry::new(format!("{copy:02}:{}", row["id"].as_str().unwrap()), row["text"].as_str().unwrap())
                .with_metadata(json!({"collection": row["collection"], "copy": copy})));
        }
    }
    for n in 0..160 {
        let mut text = format!("alpha beta Café Grüße 東京 marker{n:04} ");
        for term in 0..320 { text.push_str(&format!("word{term:04} ")); }
        entries.push(KnowledgeEntry::new(format!("synthetic:{n:04}"), text)
            .with_metadata(json!({"collection":"airust-reference", "copy":0})));
    }
    Ok(Engine::from_entries(entries)?)
}

fn queries(directory: &Path) -> Result<Vec<String>> {
    let mut result: Vec<_> = rows(&directory.join("cases.jsonl"))?.into_iter()
        .map(|row| row["query"].as_str().unwrap().to_owned()).collect();
    result.extend([
        "rust".to_owned(), "rust ".repeat(128), "alpha beta".to_owned(),
        (0..320).map(|n|format!("word{n:04}")).collect::<Vec<_>>().join(" "),
        "Cafe\u{301} Grüße 東京".to_owned(), "unfindable987654321".to_owned(),
        "".to_owned(), "source snapshot document search query evidence model".to_owned(),
    ]);
    Ok(result)
}

fn signature(index: &Engine, queries: &[String], label: &str) -> Vec<Value> {
    let scopes = [None, Some(Context::new().with_metadata("collection", "airust-reference")),
        Some(Context::new().with_entry("00:ref-v1:ADAPTIVE.md:0:29").with_entry("synthetic:0001"))];
    let mut result = Vec::new();
    for algorithm in [SearchAlgorithm::Bm25, SearchAlgorithm::Tfidf] {
        for (scope, context) in scopes.iter().enumerate() {
            for (position, query) in queries.iter().enumerate() {
                let hits = index.search_with_options(query, &SearchOptions {limit: 12, algorithm, context: context.clone(), ..Default::default()});
                result.push(json!({"stage":label,"algorithm":format!("{algorithm:?}"),"scope":scope,"query":position,
                    "hits":hits.iter().map(|hit|json!([hit.entry.id,hit.score.to_bits(),hit.matched_terms])).collect::<Vec<_>>()}));
            }
        }
    }
    result
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len()!=3 {return Err("Usage: refactor-consumer signature|measure DATASET NEW_OUTPUT".into());}
    let directory = Path::new(&args[1]);
    let queries = queries(directory)?;
    let mut index = load(directory, if args[0]=="measure" {8} else {1})?;
    let report = if args[0]=="signature" {
        let mut records = signature(&index, &queries, "original");
        let earlier = index.clone();
        index.upsert(KnowledgeEntry::new("synthetic:0001","alpha replaced snapshot rust").with_metadata(json!({"collection":"airust-reference","copy":0})))?;
        index.remove("synthetic:0002");
        index.upsert(KnowledgeEntry::new("added","Café new source alpha").with_metadata(json!({"collection":"airust-reference","copy":0})))?;
        let before = index.to_json()?;
        assert!(index.upsert(KnowledgeEntry::new("added"," \t")).is_err());
        assert_eq!(before,index.to_json()?);
        records.extend(signature(&index,&queries,"edited"));
        records.extend(signature(&earlier,&queries,"retained"));
        records.extend(signature(&Engine::from_json(&index.to_json()?)?,&queries,"restored"));
        json!({"signatures":records,"portable_json":index.to_json()?,"retained_json":earlier.to_json()?})
    } else if args[0]=="measure" {
        let mut samples=Vec::new();
        for algorithm in [SearchAlgorithm::Bm25,SearchAlgorithm::Tfidf] {
            let options=SearchOptions{limit:12,algorithm,context:Some(Context::new().with_metadata("collection","airust-reference")),..Default::default()};
            for query in &queries {black_box(index.search_with_options(black_box(query),&options));}
            for repetition in 0..7 {
                for (position,query) in queries.iter().enumerate() {
                    let start=Instant::now();
                    let hits=index.search_with_options(black_box(query),&options);
                    black_box(&hits);
                    let micros=start.elapsed().as_secs_f64()*1_000_000.0;
                    samples.push(json!({"algorithm":format!("{algorithm:?}"),"query":position,"repetition":repetition,"micros":micros,"hits":hits.len()}));
                }
            }
        }
        json!({"entries":index.len(),"query_count":queries.len(),"samples":samples})
    } else {return Err("Unknown mode".into());};
    use std::io::Write;
    let mut file=fs::OpenOptions::new().create_new(true).write(true).open(&args[2])?;
    serde_json::to_writer(&mut file,&report)?;
    file.write_all(b"\n")?;
    Ok(())
}
