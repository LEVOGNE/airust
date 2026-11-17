use airust::{
    ann::{AnnSearchOptions, HnswConfig, HnswIndex},
    cache::{CacheIdentity, IndexCache},
    context_engine::{ContextAssembler, ContextOptions},
    graph::{KnowledgeGraph, Relation, RelationType},
    hybrid::HybridSearchOptions,
    provenance::{ProvenanceStore, SourceReference},
    quality::{evaluate_ranking, Judgments, UnjudgedPolicy},
    semantic::{
        EmbeddingError, EmbeddingProvider, EmbeddingPurpose, EmbeddingSpace, SemanticIndex,
    },
    Context, Engine, KnowledgeEntry,
};
use serde_json::json;
use std::error::Error;

struct ExampleVectors(EmbeddingSpace);

#[cfg(feature = "reranking")]
struct ExampleScorer;
#[cfg(feature = "reranking")]
impl airust::reranking::Reranker for ExampleScorer {
    fn recipe(&self) -> &str {
        "handwritten-pipeline-score-demo-v1"
    }
    fn score(
        &mut self,
        _: &str,
        texts: &[&str],
    ) -> Result<Vec<f64>, airust::reranking::RerankError> {
        Ok(texts
            .iter()
            .map(|text| if text.contains("Rust") { 1.0 } else { 0.0 })
            .collect())
    }
}
impl EmbeddingProvider for ExampleVectors {
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
            .map(|text| {
                if text.contains("Rust") || text.contains("Ownership") {
                    vec![1.0, 0.0]
                } else {
                    vec![0.0, 1.0]
                }
            })
            .collect())
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    let originals = vec![
        KnowledgeEntry::new("rust", "Rust memory safety").with_metadata(json!({"scope":"public"})),
        KnowledgeEntry::new("ownership", "Ownership and borrowing")
            .with_metadata(json!({"scope":"public"})),
        KnowledgeEntry::new("private", "Private gardening note")
            .with_metadata(json!({"scope":"private"})),
    ];
    let mut provider = ExampleVectors(EmbeddingSpace::new("handwritten-api-demo-v1", 2)?);
    let semantic =
        SemanticIndex::from_index(Engine::from_entries(originals.clone())?, &mut provider, 2)?;
    let ann = HnswIndex::from_semantic(semantic, HnswConfig::default());
    let scope = Context::new().with_metadata("scope", "public");
    let ann_options = AnnSearchOptions::new(2).with_semantic_options(
        airust::semantic::SemanticSearchOptions::new(2).with_context(scope.clone()),
    );
    let approximate = ann.search("Rust", &mut provider, &ann_options)?;
    let mut graph = KnowledgeGraph::new();
    for entry in ann.semantic().lexical().entries() {
        graph.add_node(&entry.id)?;
    }
    graph.upsert_relation(Relation::new(
        "rust-ownership",
        "rust",
        "ownership",
        RelationType::References,
    )?)?;
    let mut sources = ProvenanceStore::new();
    sources.add(
        "rust",
        SourceReference::new("rust-reference-v1")?
            .with_document("example-reference")?
            .with_revision("v1")?,
    )?;
    sources.add(
        "ownership",
        SourceReference::new("ownership-reference-v1")?
            .with_document("example-reference")?
            .with_revision("v1")?,
    )?;
    let options = ContextOptions::new(2, 128)
        .with_scope(scope.clone())
        .with_hybrid(HybridSearchOptions::new(2));
    let assembler = ContextAssembler::from_semantic(ann.semantic())
        .with_graph(&graph)
        .with_provenance(&sources);
    #[cfg(feature = "reranking")]
    let (bundle, rerank_trace) = {
        let candidates = ann.semantic().hybrid_search(
            "Rust",
            &mut provider,
            &HybridSearchOptions::new(2).with_context(scope.clone()),
        )?;
        let reranked = airust::reranking::rerank(
            "Rust",
            candidates.results(),
            &mut ExampleScorer,
            &airust::reranking::RerankOptions::new(2).with_context(scope),
        )?;
        let ids = reranked.results().iter().map(|r| r.entry().id.as_str());
        (assembler.assemble(ids, &options)?, json!(reranked))
    };
    #[cfg(not(feature = "reranking"))]
    let (bundle, rerank_trace) = (
        assembler.query("Rust", Some(&mut provider), &options)?,
        serde_json::Value::Null,
    );
    assert!(bundle
        .items()
        .iter()
        .flat_map(|item| item.references())
        .all(|r| r.id() != "private"));
    let selected: Vec<_> = bundle
        .items()
        .iter()
        .flat_map(|item| item.references().iter().map(|r| r.id()))
        .collect();
    let labels = Judgments::new([("rust", 3), ("ownership", 2)])?;
    let metrics = evaluate_ranking(&selected, &labels, 2, UnjudgedPolicy::RequireJudged)?;
    let root = std::env::temp_dir().join(format!("airust-pipeline-{}", std::process::id()));
    std::fs::create_dir(&root)?;
    let cache = IndexCache::new(
        root.join("derived.cache"),
        CacheIdentity::new("example-dataset")?.with_revision("committed-v1")?,
    );
    cache.save_semantic(ann.semantic())?;
    let mut records = originals;
    let loaded = cache.load_semantic(&mut records, provider.space())?;
    assert!(records.is_empty());
    assert_eq!(loaded.index().len(), 3);
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"demo":"Handwritten vectors/scores demonstrate APIs, not semantic understanding","ann_mode":approximate.mode(),"reranking":rerank_trace,"context":bundle,"source_binding_metrics":metrics,"cache_entries":loaded.index().len()})
        )?
    );
    std::fs::remove_dir_all(root)?;
    Ok(())
}
