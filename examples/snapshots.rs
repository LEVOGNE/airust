use airust::{
    provenance::SourceReference,
    retrieval::{ExcerptOptions, LexicalSource, RetrievalOptions, Retriever},
    semantic::{EmbeddingError, EmbeddingProvider, EmbeddingPurpose, EmbeddingSpace},
    snapshots::{DatasetChange, DatasetConfig, DatasetManager, DatasetRecord, DatasetUpdate},
    KnowledgeEntry,
};
struct Provider(EmbeddingSpace);
impl EmbeddingProvider for Provider {
    fn space(&self) -> &EmbeddingSpace {
        &self.0
    }
    fn embed(
        &mut self,
        texts: &[&str],
        _: EmbeddingPurpose,
    ) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        Ok(texts
            .iter()
            .map(|t| {
                if t.contains("Rust") {
                    vec![1.0, 0.0]
                } else {
                    vec![0.0, 1.0]
                }
            })
            .collect())
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut provider = Provider(EmbeddingSpace::new("handwritten-snapshot-example", 2)?);
    let config = DatasetConfig::default().with_embedding_space(provider.0.clone())?;
    #[cfg(feature = "advanced-search")]
    let config = config.with_text_analyzer(airust::text_search::AnalyzerConfig::german());
    let manager = DatasetManager::new("knowledge", config)?;
    let initial = manager.snapshot()?;
    let record = DatasetRecord::new(KnowledgeEntry::new(
        "manual",
        format!(
            "{} Rust ownership prevents memory errors.",
            "Background. ".repeat(80)
        ),
    ))
    .with_sources(vec![
        SourceReference::new("manual-v1")?.with_revision("source-commit-1")?
    ]);
    let prepared =
        manager.prepare_replace(initial.revision(), vec![record], None, Some(&mut provider))?;
    let view = manager.publish(prepared)?;
    let options = RetrievalOptions::new(5)
        .with_relevant_excerpts(ExcerptOptions::default())
        .with_byte_budgets(1024, 1024, 160, 16384)?;
    let response =
        Retriever::new(LexicalSource::new(view.lexical())).query("Rust ownership", &options)?;
    assert!(response.hits()[0].excerpt.contains("Rust ownership"));
    assert_eq!(
        view.provenance()
            .sources_for(&response.hits()[0].id)
            .next()
            .unwrap()
            .revision(),
        Some("source-commit-1")
    );
    println!(
        "Snapshot generation {}: {}",
        view.revision().generation(),
        response.to_json()
    );
    let changed = DatasetRecord::new(KnowledgeEntry::new(
        "manual",
        "Rust changed text needs new vectors and explicitly supplied sources.",
    ));
    let next = manager.publish(manager.prepare(
        view.revision(),
        DatasetUpdate::new(vec![DatasetChange::Put(changed)]),
        Some(&mut provider),
    )?)?;
    assert!(manager.require_current(view.revision()).is_err());
    assert!(view.provenance().source("manual-v1").is_some());
    assert!(next.provenance().source("manual-v1").is_none());
    #[cfg(feature = "context-assembly")]
    {
        use airust::context_engine::{ContextAssembler, ContextOptions};
        let context = ContextAssembler::new(view.lexical())
            .with_provenance(view.provenance())
            .query("Rust", None, &ContextOptions::new(3, 1024))?;
        assert!(!context.items().is_empty());
    }
    println!(
        "The old reader stays coherent; the new generation is {}.",
        next.revision().generation()
    );
    println!("Handwritten vectors demonstrate consistency, not semantic quality.");
    Ok(())
}
