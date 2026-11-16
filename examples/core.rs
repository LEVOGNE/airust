use airust::{
    Context, Engine, KnowledgeEntry, MatchOptions, MatchStrategy, SearchAlgorithm, SearchOptions,
};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = Engine::from_entries([
        KnowledgeEntry::new(
            "note-1",
            "Rust provides memory safety without a garbage collector.",
        )
        .with_metadata(json!({"project": "research", "source": "Personal notes"})),
        KnowledgeEntry::new("note-2", "Rust is useful for fast search tools."),
        KnowledgeEntry::new("note-3", "Python is useful for scripts."),
        KnowledgeEntry::new("note-4", "  RUST is useful for fast search tools.  "),
    ])?;

    println!("Find knowledge:");
    for hit in engine.search("Rust memory", 3) {
        println!(
            "  {} ({:.3}): {} — terms: {:?}",
            hit.entry.id, hit.score, hit.entry.text, hit.matched_terms
        );
    }

    println!("Similar text:");
    for hit in engine.find_matches("Rust is useful for fast search tool.", 3) {
        println!(
            "  {}: {:.1}% character similarity",
            hit.entry.id,
            hit.score * 100.0
        );
    }

    let exact = MatchOptions {
        strategy: MatchStrategy::Exact,
        ..Default::default()
    };
    println!(
        "Exact matches: {}",
        engine
            .match_with_options("Rust is useful for fast search tools.", &exact)?
            .len()
    );
    println!("Duplicate groups: {}", engine.duplicates().len());

    println!("Suggested connections:");
    for proposal in engine.relate("note-1", 3)? {
        println!(
            "  {} → {}: shared terms {:?}",
            proposal.source_id, proposal.target_id, proposal.shared_terms
        );
    }

    let context = Context::new().with_metadata("project", "research");
    println!(
        "Matches in project: {}",
        engine.search_in_context("Rust", &context, 10).len()
    );
    let options = SearchOptions {
        algorithm: SearchAlgorithm::Tfidf,
        ..Default::default()
    };
    println!(
        "TF-IDF matches: {}",
        engine.search_with_options("Rust", &options).len()
    );

    engine.upsert(KnowledgeEntry::new(
        "note-3",
        "Python is useful for automation.",
    ))?;
    engine.remove("note-4");
    let restored = Engine::from_json(&engine.to_json()?)?;
    println!("Saved and restored: {} entries", restored.len());
    Ok(())
}
