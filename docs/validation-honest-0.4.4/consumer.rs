use airust::{Context, KnowledgeEntry, SearchOptions};
use airust::semantic::{Embedding, EmbeddingSpace, EmbeddingProvider, EmbeddingPurpose, EmbeddingError, SemanticIndex, SemanticSearchOptions};
use airust::hybrid::HybridSearchOptions;
use serde_json::{json, Value};
use std::{io::{self, BufRead, Write}, time::Instant};

struct FixedQuery { space: EmbeddingSpace, vector: Vec<f32> }
impl EmbeddingProvider for FixedQuery {
    fn space(&self) -> &EmbeddingSpace { &self.space }
    fn embed(&mut self, texts: &[&str], purpose: EmbeddingPurpose) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        assert_eq!(purpose, EmbeddingPurpose::Query);
        assert_eq!(texts.len(), 1);
        Ok(vec![self.vector.clone()])
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let start = Instant::now();
    let chunks: Vec<Value> = serde_json::from_slice(&std::fs::read(&args[1])?)?;
    let raw = std::fs::read(&args[2])?;
    let dims: usize = args[3].parse()?;
    assert_eq!(raw.len(), chunks.len() * dims * 4);
    let space = EmbeddingSpace::new(&args[4], dims)?;
    let mut index = SemanticIndex::new(space.clone());
    for (i, c) in chunks.iter().enumerate() {
        let text = c["text"].as_str().unwrap();
        assert_eq!(airust::text::tokenize(text).join(" "), c["lexical"].as_str().unwrap());
        let entry = KnowledgeEntry::new(c["id"].as_str().unwrap(), text)
            .with_metadata(json!({"project": c["project"]}));
        let vector = raw[i*dims*4..(i+1)*dims*4].chunks_exact(4)
            .map(|b| f32::from_le_bytes(b.try_into().unwrap())).collect();
        index.upsert_embedded(entry, Embedding::new(space.clone(), vector)?)?;
    }
    drop(raw); drop(chunks);
    println!("{}", json!({"ready":true,"version":airust::VERSION,"entries":index.len(),"build_s":start.elapsed().as_secs_f64()}));
    io::stdout().flush()?;
    for line in io::stdin().lock().lines() {
        let r: Value = serde_json::from_str(&line?)?;
        let query = r["lexical"].as_str().unwrap();
        let project = r["project"].as_str().unwrap();
        assert!(["airust", "flasm", "vecasm"].contains(&project));
        let scope = Context::new().with_metadata("project", project);
        let limit = r["limit"].as_u64().unwrap_or(6) as usize;
        let mode = r["mode"].as_str().unwrap();
        let start = Instant::now();
        let hits: Vec<Value> = if mode == "D" {
            index.lexical().search_with_options(query, &SearchOptions { limit, context:Some(scope), ..Default::default() })
                .into_iter().map(|h|json!({"id":h.entry.id,"score":h.score})).collect()
        } else {
            let vector: Vec<f32> = serde_json::from_value(r["vector"].clone())?;
            let mut provider = FixedQuery {space:space.clone(),vector};
            if mode == "semantic" {
                let q = Embedding::new(space.clone(), provider.vector)?;
                index.search_embedding(&q, &SemanticSearchOptions::new(limit).with_context(scope))?
                    .into_iter().map(|h|json!({"id":h.entry.id,"score":h.score})).collect()
            } else {
                assert_eq!(mode,"C");
                let options = HybridSearchOptions::new(limit).with_candidates(100)?.with_rrf_k(60.0)?.with_weights(0.5,0.5)?.with_context(scope);
                index.hybrid_search(query,&mut provider,&options)?.results().iter()
                    .map(|h|json!({"id":h.entry.id,"score":h.score,"lexical_rank":h.lexical_rank,"semantic_rank":h.semantic_rank})).collect()
            }
        };
        println!("{}",json!({"hits":hits,"search_ms":start.elapsed().as_secs_f64()*1000.0}));
        io::stdout().flush()?;
    }
    Ok(())
}
