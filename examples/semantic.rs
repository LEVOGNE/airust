use airust::{
    semantic::{Embedding, EmbeddingSpace, SemanticIndex, SemanticSearchOptions},
    KnowledgeEntry,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let space = EmbeddingSpace::new("demonstration-vectors-only:v1", 3)?;
    let mut index = SemanticIndex::new(space.clone());
    index.upsert_embedded(
        KnowledgeEntry::new("a", "A sample record."),
        Embedding::new(space.clone(), vec![1.0, 0.0, 0.0])?,
    )?;
    index.upsert_embedded(
        KnowledgeEntry::new("b", "Another sample record."),
        Embedding::new(space.clone(), vec![0.0, 1.0, 0.0])?,
    )?;
    let query = Embedding::new(space, vec![0.9, 0.1, 0.0])?;
    let hits = index.search_embedding(&query, &SemanticSearchOptions::new(2))?;
    assert_eq!(hits[0].entry.id, "a");
    println!(
        "{}: {:.3} cosine similarity",
        hits[0].entry.id, hits[0].score
    );
    println!("These hand-written vectors demonstrate the API, not semantic understanding.");
    assert!(!index.lexical().search("sample", 2).is_empty());
    Ok(())
}
