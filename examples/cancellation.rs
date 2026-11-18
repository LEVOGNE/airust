use airust::{execution::*, retrieval::*, Engine, KnowledgeEntry};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = CancellationToken::new();
    let control = WorkControl::new(token.clone());
    let index = Engine::from_entries([KnowledgeEntry::new("a", "Rust ownership")])?;
    let options = RetrievalOptions::new(3).with_control(control);
    assert_eq!(
        Retriever::new(LexicalSource::new(&index))
            .query("Rust", &options)?
            .hits()
            .len(),
        1
    );
    token.cancel();
    assert!(matches!(
        Retriever::new(LexicalSource::new(&index)).query("Rust", &options),
        Err(RetrievalError::Execution(ExecutionError::Cancelled))
    ));
    println!("Cancellation returns a typed error without a partial response.");
    Ok(())
}
