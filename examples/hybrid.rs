use airust::{
    hybrid::HybridSearchOptions,
    semantic::{
        Embedding, EmbeddingError, EmbeddingProvider, EmbeddingPurpose, EmbeddingSpace,
        SemanticIndex,
    },
    KnowledgeEntry,
};

struct ExampleProvider(EmbeddingSpace);
impl EmbeddingProvider for ExampleProvider {
    fn space(&self) -> &EmbeddingSpace {
        &self.0
    }
    fn embed(
        &mut self,
        texts: &[&str],
        _: EmbeddingPurpose,
    ) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        Ok(texts.iter().map(|_| vec![1.0, 0.0]).collect())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let space = EmbeddingSpace::new("hybrid-api-demonstration-only:v1", 2)?;
    let mut provider = ExampleProvider(space.clone());
    let mut index = SemanticIndex::new(space.clone());
    for (id, text, values) in [
        ("a", "Rust memory safety", vec![0.0, 1.0]),
        ("b", "Python scripts", vec![1.0, 0.0]),
        ("c", "Rust search", vec![0.8, 0.6]),
    ] {
        index.upsert_embedded(
            KnowledgeEntry::new(id, text),
            Embedding::new(space.clone(), values)?,
        )?;
    }
    let outcome = index.hybrid_search("Rust", &mut provider, &HybridSearchOptions::new(3))?;
    for hit in outcome.results() {
        println!(
            "{}: {:.6}, lexical rank {:?}, semantic rank {:?}",
            hit.entry.id, hit.score, hit.lexical_rank, hit.semantic_rank
        );
    }
    assert_eq!(outcome.results()[0].entry.id, "c");
    println!(
        "Hand-written vectors demonstrate fusion only; they do not establish semantic quality."
    );
    Ok(())
}
