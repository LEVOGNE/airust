use airust::{retrieval::*, Context, Engine, KnowledgeEntry};
use serde_json::{json, Value};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let chunks: Vec<Value> = serde_json::from_slice(&std::fs::read(&args[1])?)?;
    let queries: Vec<Value> = serde_json::from_slice(&std::fs::read(&args[2])?)?;
    let index = Engine::from_entries(chunks.iter().map(|c| {
        KnowledgeEntry::new(c["id"].as_str().unwrap(), c["text"].as_str().unwrap())
            .with_metadata(json!({"project": c["project"]}))
    }))?;
    for query in queries {
        for prepared in [false, true] {
            for mode in ["prefix", "leading", "compact"] {
                for distinct in [false, true] {
                    let mut source = LexicalSource::new(&index);
                    if prepared {
                        source = source.with_query_analysis(LexicalQueryOptions::default());
                    }
                    let mut options = RetrievalOptions::new(6)
                        .with_candidates(100)?
                        .with_context(Context::new().with_metadata("project", query["project"].as_str().unwrap()))
                        .with_byte_budgets(16384, 12000, 2000, 128 * 1024)?;
                    if distinct { options = options.with_distinct_evidence(); }
                    match mode {
                        "leading" => options = options.with_relevant_excerpts(ExcerptOptions::default().with_leading_context()),
                        "compact" => options = options.with_relevant_excerpts(ExcerptOptions::default().with_paragraph_boundaries()),
                        _ => {},
                    }
                    let response = Retriever::new(source).query(query["question"].as_str().unwrap(), &options)?;
                    let mut hits = Vec::new();
                    for hit in response.hits() {
                        assert_eq!(hit.span.slice(&index.get(&hit.id).unwrap().text), Some(hit.excerpt.as_str()));
                        assert!(hit.excerpt.len() <= 2000);
                        hits.push(json!({"id":hit.id,"span":hit.span,"bytes":hit.excerpt.len(),"source_rank":hit.source_rank}));
                    }
                    assert!(hits.len() <= 6);
                    let trace = if prepared && mode == "prefix" && !distinct {
                        let a = LexicalQueryOptions::default().analyze(query["question"].as_str().unwrap())?;
                        Some(json!({"effective_query":a.effective_query(),"added_terms":a.added_terms(),"removed_terms":a.removed_terms(),"original_fallback":a.original_fallback()}))
                    } else { None };
                    println!("{}", json!({"query_id":query["id"],"prepared":prepared,"mode":mode,"distinct":distinct,"hits":hits,"query_analysis":trace}));
                }
            }
        }
    }
    Ok(())
}
