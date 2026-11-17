use airust::{
    ann::{AnnSearchOptions, HnswConfig, HnswIndex},
    semantic::{Embedding, EmbeddingSpace, SemanticIndex},
    KnowledgeEntry,
};
use std::error::Error;
fn main() -> Result<(), Box<dyn Error>> {
    let space = EmbeddingSpace::new("example-v1", 2)?;
    let mut semantic = SemanticIndex::new(space.clone());
    semantic.upsert_embedded(
        KnowledgeEntry::new("a", "Rust memory"),
        Embedding::new(space.clone(), vec![1.0, 0.0])?,
    )?;
    let mut index = HnswIndex::from_semantic(semantic, HnswConfig::default());
    let query = Embedding::new(space.clone(), vec![1.0, 0.0])?;
    let result = index.search_embedding(&query, &AnnSearchOptions::new(1))?;
    assert_eq!(result.results()[0].entry.id, "a");
    println!("Search mode: {:?}", result.mode());
    index.upsert_embedded(
        KnowledgeEntry::new("b", "updated knowledge"),
        Embedding::new(space, vec![0.8, 0.2])?,
    )?;
    index.rebuild();
    assert_eq!(index.pending_changes(), 0);
    Ok(())
}
