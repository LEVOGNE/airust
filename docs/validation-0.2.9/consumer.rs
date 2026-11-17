use airust::semantic::{
    EmbeddingError, EmbeddingProvider, EmbeddingPurpose, EmbeddingSpace, SemanticIndex,
    SemanticSearchOptions,
};
use airust::{Context, Engine, KnowledgeEntry};
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
            .map(|text| {
                if text.contains("Rust") {
                    vec![1.0, 0.0]
                } else {
                    vec![0.0, 1.0]
                }
            })
            .collect())
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    {
        use airust::reranking::{rerank, RerankError, RerankOptions, Reranker};
        struct Ranker;
        impl Reranker for Ranker {
            fn recipe(&self) -> &str {
                "independent-consumer-v1"
            }
            fn score(&mut self, _: &str, documents: &[&str]) -> Result<Vec<f64>, RerankError> {
                Ok(documents
                    .iter()
                    .map(|t| if t.contains("ownership") { 2.0 } else { -1.0 })
                    .collect())
            }
        }
        let source = Engine::from_entries([
            KnowledgeEntry::new("safe", "Rust ownership memory"),
            KnowledgeEntry::new("manual", "Rust raw memory"),
        ])?;
        let candidates = source.search("Rust memory", 20);
        let options = RerankOptions::new(2)
            .with_candidates(20)?
            .with_batch_size(1)?
            .with_min_score(0.0)?;
        let outcome = rerank("Rust memory", &candidates, &mut Ranker, &options)?;
        assert_eq!(outcome.results().len(), 1);
        assert_eq!(outcome.results()[0].entry().id, "safe");
        assert_eq!(outcome.results()[0].score, Some(2.0));
        assert!(outcome.reranker_used());
        assert_eq!(outcome.scored_candidates(), 2);
        assert!(outcome.results()[0].candidate.score > 0.0);
    }

    let mut core = Engine::from_entries([
        KnowledgeEntry {
            id: "a".into(),
            text: "Rust memory".into(),
            metadata: None,
        },
        KnowledgeEntry::new("b", "Python scripts"),
    ])?;
    assert_eq!(core.search("Rust", 1)[0].entry.id, "a");
    assert_eq!(core.find_matches("Rust memory", 1)[0].score, 1.0);
    core.upsert(KnowledgeEntry::new("c", "Rust search"))?;
    assert_eq!(core.relate("a", 1)?[0].target_id, "c");
    let mut provider = Provider(EmbeddingSpace::new("consumer-fixture-v1", 2)?);
    let mut index = SemanticIndex::from_index(core, &mut provider, 2)?;
    let options = SemanticSearchOptions::new(1).with_context(Context::new().with_entry("c"));
    assert_eq!(
        index.search("Rust", &mut provider, &options)?[0].entry.id,
        "c"
    );
    index.upsert(KnowledgeEntry::new("c", "Python changed"), &mut provider)?;
    assert_eq!(index.lexical().search("Rust", 9).len(), 1);
    index.remove("a");
    assert!(index.lexical().search("Rust", 9).is_empty());
    let outcome = index.hybrid_search(
        "Python",
        &mut provider,
        &airust::hybrid::HybridSearchOptions::new(2),
    )?;
    assert_eq!(outcome.results().len(), 2);
    assert!(outcome.semantic_used());
    assert!(outcome.fallback_reason().is_none());
    let mut graph = airust::graph::KnowledgeGraph::new();
    for entry in index.lexical().entries() {
        graph.add_node(&entry.id)?;
    }
    graph.upsert_relation(airust::graph::Relation::new(
        "bc",
        "b",
        "c",
        airust::graph::RelationType::DependsOn,
    )?)?;
    assert_eq!(
        graph
            .breadth_first("b", &airust::graph::TraversalOptions::new(1, 10))?
            .len(),
        2
    );
    assert!(!graph.has_cycle());
    graph.upsert_relation(airust::graph::Relation::new(
        "cb",
        "c",
        "b",
        airust::graph::RelationType::References,
    )?)?;
    assert!(graph.has_cycle());
    assert!(!graph.has_cycle_for(&[airust::graph::RelationType::DependsOn]));
    assert!(graph.remove_node("c"));
    assert_eq!(graph.relation_count(), 0);
    let context = airust::context_engine::ContextAssembler::from_semantic(&index)
        .with_graph(&graph)
        .query(
            "Python",
            Some(&mut provider),
            &airust::context_engine::ContextOptions::new(4, 12)
                .with_semantic(SemanticSearchOptions::new(2)),
        )?;
    assert!(context.node_count() > 0 && context.node_count() <= 4);
    assert!(context.used_text_bytes() <= 12);
    assert!(context
        .items()
        .iter()
        .all(|item| !item.references().is_empty()));
    let denied = airust::context_engine::ContextOptions::default()
        .with_scope(Context::new().with_entry("b"))
        .with_semantic(SemanticSearchOptions::new(2).with_context(Context::new().with_entry("c")));
    assert!(
        airust::context_engine::ContextAssembler::from_semantic(&index)
            .query("Python", None, &denied)?
            .items()
            .is_empty()
    );
    assert!(graph
        .breadth_first_matching("b", &airust::graph::TraversalOptions::default(), |_| false)?
        .is_empty());
    let mut sources = airust::provenance::ProvenanceStore::new();
    let source = airust::provenance::SourceReference::new("consumer-doc-v1")?
        .with_document("consumer-document")?
        .with_page(1)?
        .with_text_span(airust::provenance::TextSpan::new(0, 6)?);
    source.validate_source_text("Python source")?;
    sources.insert_source(source)?;
    sources.bind("b", "consumer-doc-v1")?;
    sources.bind("c", "consumer-doc-v1")?;
    assert_eq!(sources.source_count(), 1);
    assert_eq!(
        airust::provenance::ProvenanceStore::from_json(&sources.to_json()?)?,
        sources
    );
    let cited = airust::context_engine::ContextAssembler::from_semantic(&index)
        .with_provenance(&sources)
        .assemble(["b"], &airust::context_engine::ContextOptions::default())?;
    assert_eq!(
        cited.items()[0].references()[0].sources()[0].page(),
        Some(1)
    );
    assert_eq!(
        cited.items()[0].references()[0].provenance_status(),
        airust::provenance::ProvenanceStatus::Referenced
    );
    let path = std::env::temp_dir().join(format!("airust-cache-consumer-{}", std::process::id()));
    std::fs::create_dir_all(&path)?;
    let cache = airust::cache::IndexCache::new(
        path.join("derived.cache"),
        airust::cache::CacheIdentity::new("consumer")?.with_revision("revision-1")?,
    );
    cache.save_semantic(&index)?;
    let mut originals = index.lexical().entries().cloned().collect::<Vec<_>>();
    let restored = cache
        .load_semantic(&mut originals, index.space())?
        .into_index();
    assert!(originals.is_empty());
    assert_eq!(restored.lexical().to_json()?, index.lexical().to_json()?);
    assert_eq!(restored.embedding("b"), index.embedding("b"));
    std::fs::remove_dir_all(path)?;
    let mut ann = airust::ann::HnswIndex::from_semantic(
        restored,
        airust::ann::HnswConfig::default().with_exact_threshold(0),
    );
    let outcome = ann.search(
        "Python",
        &mut provider,
        &airust::ann::AnnSearchOptions::new(2),
    )?;
    assert_eq!(outcome.mode(), airust::ann::AnnSearchMode::Hnsw);
    assert_eq!(outcome.results().len(), 2);
    ann.remove("b");
    assert_eq!(ann.pending_changes(), 1);
    ann.rebuild();
    assert_eq!(ann.pending_changes(), 0);
    assert_eq!(ann.into_semantic().len(), 1);
    assert_eq!(index.into_lexical().len(), 2);
    let mut versions = airust::versioning::VersionedIndex::new().with_history_limit(4);
    let first = versions.upsert(
        KnowledgeEntry::new("versioned", "before"),
        airust::versioning::ExpectedRevision::Absent,
        airust::versioning::ChangeMetadata::default(),
    )?;
    let second = versions.upsert(
        KnowledgeEntry::new("versioned", "after"),
        airust::versioning::ExpectedRevision::Exact(first),
        airust::versioning::ChangeMetadata::default(),
    )?;
    assert!(versions.compare("versioned", first, second)?.text_changed());
    let restored = versions.restore(
        "versioned",
        first,
        airust::versioning::ExpectedRevision::Exact(second),
        airust::versioning::ChangeMetadata::default(),
    )?;
    assert!(restored > second);
    assert_eq!(versions.index().get("versioned").unwrap().text, "before");
    let versions = airust::versioning::VersionedIndex::from_json(&versions.to_json()?)?;
    assert_eq!(versions.revision("versioned"), Some(restored));
    let judgments = airust::quality::Judgments::new([("a", 3), ("b", 1), ("c", 0)])?;
    let metrics = airust::quality::evaluate_ranking(
        &["a", "c", "b"],
        &judgments,
        3,
        airust::quality::UnjudgedPolicy::RequireJudged,
    )?;
    assert_eq!(metrics.recall_at_k(), Some(1.0));
    assert_eq!(airust::quality::summarize(&[metrics])?.queries(), 1);
    println!("Independent consumer: existing APIs and optional reranking passed");
    Ok(())
}
