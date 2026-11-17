use airust::{
    retrieval::{ExcerptOptions, LexicalSource, RetrievalOptions, Retriever},
    Engine, KnowledgeEntry,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let original = format!(
        "{}\nRust ownership prevents use-after-free errors.\nAdditional background.",
        "Background material. ".repeat(80)
    );
    let index = Engine::from_entries([KnowledgeEntry::new("manual", &original)])?;
    let options = RetrievalOptions::new(3)
        .with_byte_budgets(1024, 240, 120, 8192)?
        .with_relevant_excerpts(ExcerptOptions::default());
    let response = Retriever::new(LexicalSource::new(&index)).query("Rust ownership", &options)?;
    let hit = &response.hits()[0];
    assert!(hit.span.start > 0);
    assert!(hit.excerpt.contains("Rust ownership"));
    assert_eq!(hit.span.slice(&original), Some(hit.excerpt.as_str()));
    for matched in &response.excerpt_details().unwrap()[0].matches {
        let relative = matched
            .span
            .relative_to(hit.span)
            .ok_or("match is outside excerpt")?;
        assert_eq!(relative.slice(&hit.excerpt), matched.span.slice(&original));
    }
    println!("{}", response.to_json());
    #[cfg(feature = "advanced-search")]
    {
        use airust::{
            retrieval::TextSource,
            text_search::{AnalyzerConfig, TextIndex},
        };
        let german = format!("{} Die Häuser haben rote Türen.", "Einleitung. ".repeat(50));
        let index = TextIndex::from_entries(
            AnalyzerConfig::german(),
            [KnowledgeEntry::new("de", german)],
        )?;
        let response = Retriever::new(TextSource::new(&index)).query("\"Haus haben\"", &options)?;
        assert!(response.hits()[0].excerpt.contains("Häuser haben"));
        println!("{}", response.to_json());
    }
    Ok(())
}
