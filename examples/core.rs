//! Run with: cargo run --example core
use airust::{
    Context, Engine, KnowledgeEntry, MatchOptions, MatchStrategy, SearchAlgorithm, SearchOptions,
};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = Engine::from_entries([
        KnowledgeEntry::new(
            "note-1",
            "Rust schützt den Speicher ohne Garbage Collector.",
        )
        .with_metadata(json!({"project": "DenkMaps", "source": "Meine Notizen"})),
        KnowledgeEntry::new("note-2", "Rust eignet sich für schnelle Suchwerkzeuge."),
        KnowledgeEntry::new("note-3", "Python eignet sich für Skripte."),
        KnowledgeEntry::new("note-4", "  RUST eignet sich für schnelle Suchwerkzeuge.  "),
    ])?;

    println!("Wissen finden:");
    for hit in engine.search("Rust Speicher", 3) {
        println!(
            "  {} ({:.3}): {} — Begriffe: {:?}",
            hit.entry.id, hit.score, hit.entry.text, hit.matched_terms
        );
    }

    println!("Ähnliche Texte:");
    for hit in engine.find_matches("Rust eignet sich für schnelle Suchwerkzeug.", 3) {
        println!(
            "  {}: {:.1}% Zeichenähnlichkeit",
            hit.entry.id,
            hit.score * 100.0
        );
    }

    let exact = MatchOptions {
        strategy: MatchStrategy::Exact,
        ..Default::default()
    };
    println!(
        "Exakte Treffer: {}",
        engine
            .match_with_options("Rust eignet sich für schnelle Suchwerkzeuge.", &exact)?
            .len()
    );
    println!("Duplikatgruppen: {}", engine.duplicates().len());

    println!("Mögliche Verbindungen:");
    for proposal in engine.relate("note-1", 3)? {
        println!(
            "  {} → {}: gemeinsame Begriffe {:?}",
            proposal.source_id, proposal.target_id, proposal.shared_terms
        );
    }

    let context = Context::new().with_metadata("project", "DenkMaps");
    println!(
        "Treffer im Projekt: {}",
        engine.search_in_context("Rust", &context, 10).len()
    );
    let options = SearchOptions {
        algorithm: SearchAlgorithm::Tfidf,
        ..Default::default()
    };
    println!(
        "TF-IDF-Treffer: {}",
        engine.search_with_options("Rust", &options).len()
    );

    engine.upsert(KnowledgeEntry::new(
        "note-3",
        "Python eignet sich für Automatisierung.",
    ))?;
    engine.remove("note-4");
    let restored = Engine::from_json(&engine.to_json()?)?;
    println!(
        "Gespeichert und wieder geladen: {} Einträge",
        restored.len()
    );
    Ok(())
}
