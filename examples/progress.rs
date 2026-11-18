use airust::{execution::*, retrieval::*, Engine, KnowledgeEntry};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let index = Engine::from_entries(
        (0..5000)
            .map(|i| KnowledgeEntry::new(i.to_string(), "Rust ownership prevents memory errors")),
    )?;
    let control = WorkControl::default().with_observer(|event: &ProgressEvent| {
        eprintln!(
            "{} {:?}: {} / {:?}",
            event.sequence(),
            event.phase(),
            event.completed(),
            event.total()
        );
    });
    let options = RetrievalOptions::new(3)
        .with_control(control.clone())
        .with_evidence_policy(EvidencePolicy::default());
    let response = Retriever::new(LexicalSource::new(&index)).query("Rust ownership", &options)?;
    assert_eq!(control.observer_failures(), 0);
    println!("{}", response.to_json());
    Ok(())
}
