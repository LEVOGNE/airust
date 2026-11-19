use airust::{
    disk_index::*, feedback::*, learned_ranking::*, storage::StoreChange, Engine, KnowledgeEntry,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::path::PathBuf::from(std::env::args().nth(1).unwrap());
    std::fs::create_dir_all(&output)?;
    let entries = vec![
        KnowledgeEntry::new(
            "de",
            "Die Häuser haben Türen. Rust ownership prevents errors.",
        ),
        KnowledgeEntry::new("en", "Search with Rust and explicit knowledge sources."),
    ];
    let index = Engine::from_entries(entries.clone())?;
    std::fs::write(output.join("engine.json"), index.to_json()?)?;
    let examples = [
        TrainingExample::new("Rust ownership", "Rust ownership prevents errors", true)?,
        TrainingExample::new("Rust ownership", "Unrelated archive", false)?,
    ];
    let model = train(&examples, &mut LexicalFeatures, &TrainingOptions::default())?;
    std::fs::write(output.join("model.json"), model.to_json()?)?;
    let mut ledger = FeedbackLedger::default();
    ledger.record(FeedbackEvent::new(
        "vote-1",
        "compat",
        "Rust ownership",
        &entries[0].text,
        Rating::Helpful,
    )?)?;
    std::fs::write(output.join("feedback.json"), ledger.to_json()?)?;
    let mut db = DiskIndex::open(output.join("legacy.db"), "compat", DiskOptions::default())?;
    db.apply(
        &db.revision()?,
        &entries
            .into_iter()
            .map(StoreChange::Put)
            .collect::<Vec<_>>(),
        &DiskWork::default(),
    )?;
    std::fs::write(
        output.join("identity.json"),
        serde_json::to_string_pretty(
            &serde_json::json!({"producer_version":airust::VERSION,"model_fingerprint":model.fingerprint()?,"feedback_profile":ledger.profile_fingerprint("compat")?,"search_ids":index.search("Rust ownership",2).iter().map(|h|h.entry.id.clone()).collect::<Vec<_>>(),"disk_revision":db.revision()?}),
        )?,
    )?;
    Ok(())
}
