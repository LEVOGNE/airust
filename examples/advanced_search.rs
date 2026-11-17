use airust::{
    text_search::{AnalyzerConfig, TextIndex, TextSearchOptions},
    KnowledgeEntry,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let index = TextIndex::from_entries(
        AnalyzerConfig::german(),
        [
            KnowledgeEntry::new("a", "Wir bauen große Häuser mit Gärten."),
            KnowledgeEntry::new("b", "Häuser sind groß. Wir bauen später."),
        ],
    )?;
    let hits = index.search("\"große Häuser\"", &TextSearchOptions::new(5))?;
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].entry.id, "a");
    println!("Analyzer: {}", index.analyzer().config().recipe());
    for hit in hits {
        println!("{}: {}", hit.entry.id, hit.score);
        for evidence in hit.phrases {
            for span in evidence.spans {
                println!(
                    "Phrase in original UTF-8: {:?}",
                    span.slice(&hit.entry.text)
                );
            }
        }
    }
    assert_eq!(index.search("Haus", &TextSearchOptions::new(5))?.len(), 2);
    Ok(())
}
