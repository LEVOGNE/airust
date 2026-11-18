use airust::{adaptive::*, retrieval::*, text_search::*, Engine, KnowledgeEntry};
use serde_json::json;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let entries = [
        KnowledgeEntry::new("a", "Häuser haben rote Türen"),
        KnowledgeEntry::new("b", "Gärten mit grünen Bäumen"),
        KnowledgeEntry::new("c", "Rust ownership prevents memory errors"),
        KnowledgeEntry::new("d", "Türen haben große Häuser"),
    ];
    let lexical = Engine::from_entries(entries.clone())?;
    let text = TextIndex::from_entries(AnalyzerConfig::german(), entries)?;
    let source = AdaptiveSource::default()
        .with_source("legacy", SourceRole::Lexical, LexicalSource::new(&lexical))?
        .with_source("german-text", SourceRole::Text, TextSource::new(&text))?;
    let mut adaptive = Retriever::new(source);
    let mut baseline = Retriever::new(LexicalSource::new(&lexical));
    let cases = [
        ("Haus", Some("a")),
        ("Garten", Some("b")),
        ("Baum", Some("b")),
        ("Tür", Some("a")),
        ("Rust ownership", Some("c")),
        ("\"Haus haben\"", Some("a")),
        ("How ownership prevents memory errors", Some("c")),
        ("Vulkan Eruption", None),
    ];
    let mut rows = Vec::new();
    let mut baseline_correct = 0;
    let mut adaptive_correct = 0;
    for (query, expected) in cases {
        let options = RetrievalOptions::new(1);
        let before = baseline.query(query, &options)?;
        let after = adaptive.query(query, &options)?;
        let before_id = before.hits().first().map(|h| h.id.as_str());
        let after_id = after.hits().first().map(|h| h.id.as_str());
        baseline_correct += usize::from(before_id == expected);
        adaptive_correct += usize::from(after_id == expected);
        rows.push(json!({"query":query,"expected_authored":expected,"baseline":before_id,"adaptive":after_id,"route":adaptive.source().last_route()}));
    }
    assert_eq!(adaptive_correct, 8);
    println!(
        "{}",
        json!({"fixture":"authored-routing-v1","cases":8,"baseline_top1_or_empty_correct":baseline_correct,"adaptive_top1_or_empty_correct":adaptive_correct,"rows":rows,"limits":"Small authored German/phrase fixture; reuses existing TextIndex, not independent semantic/model validation or universal routing improvement."})
    );
    Ok(())
}
