fn snapshot_check() -> Result<(), Box<dyn std::error::Error>> {
    use airust::snapshots::*;
    use airust::provenance::SourceReference;
    let mut provider = Provider(EmbeddingSpace::new("consumer-snapshot-v1", 2)?);
    let config = DatasetConfig::default().with_embedding_space(provider.0.clone())?
        .with_text_analyzer(airust::text_search::AnalyzerConfig::german());
    let manager = DatasetManager::new("consumer", config)?;
    let empty = manager.snapshot()?;
    let record = DatasetRecord::new(KnowledgeEntry::new("a", "Rust Häuser und Gärten"))
        .with_sources(vec![SourceReference::new("source-a")?.with_revision("one")?]);
    let old = manager.publish(manager.prepare_replace(empty.revision(), vec![record], None, Some(&mut provider))?)?;
    assert_eq!(old.lexical().search("Rust", 1)[0].entry.id, "a");
    assert!(old.semantic().unwrap().embedding("a").is_some());
    assert_eq!(old.text_index().unwrap().search("Haus", &airust::text_search::TextSearchOptions::new(1))?[0].entry.id, "a");
    assert_eq!(old.provenance().sources_for("a").next().unwrap().revision(), Some("one"));
    let stale = manager.prepare(old.revision(), DatasetUpdate::new(vec![DatasetChange::Delete("missing".into())]), None)?;
    let replacement = DatasetRecord::new(KnowledgeEntry::new("a", "Rust changed source"));
    let current = manager.publish(manager.prepare(old.revision(), DatasetUpdate::new(vec![DatasetChange::Put(replacement)]), Some(&mut provider))?)?;
    assert!(matches!(manager.publish(stale), Err(DatasetError::Conflict)));
    assert!(manager.require_current(old.revision()).is_err());
    assert!(old.provenance().source("source-a").is_some());
    assert!(current.provenance().source("source-a").is_none());
    assert_eq!(current.semantic().unwrap().lexical().get("a").unwrap().text, current.lexical().get("a").unwrap().text);
    Ok(())
}
fn excerpt_check() -> Result<(), Box<dyn std::error::Error>> {
    use airust::retrieval::*;
    let original = format!("{} Rust ownership prevents memory errors.", "Background. ".repeat(100));
    let index = Engine::from_entries([KnowledgeEntry::new("long", &original)])?;
    let options = RetrievalOptions::new(1).with_byte_budgets(1024, 120, 120, 8192)?
        .with_relevant_excerpts(ExcerptOptions::default());
    let response = Retriever::new(LexicalSource::new(&index)).query("Rust ownership", &options)?;
    let hit = &response.hits()[0];
    assert!(hit.span.start > 0 && hit.excerpt.contains("Rust ownership"));
    assert_eq!(hit.span.slice(&original), Some(hit.excerpt.as_str()));
    assert_eq!(response.excerpt_details().unwrap()[0].reason, ExcerptReason::Terms);
    for matched in &response.excerpt_details().unwrap()[0].matches {
        assert_eq!(matched.span.relative_to(hit.span).unwrap().slice(&hit.excerpt), matched.span.slice(&original));
    }
    let reset = Retriever::new(LexicalSource::new(&index)).query("Rust ownership", &options.clone().with_prefix_excerpts())?;
    assert_eq!(reset.hits()[0].span.start, 0);
    assert!(reset.excerpt_details().is_none());
    let evidence = RankingEvidence { kind: RankingKind::Bm25, score: 1.0, matched_terms: vec![], lexical_rank: None,
        semantic_rank: None, lexical_score: None, semantic_score: None, phrase_spans: vec![], phrase_evidence_truncated: false };
    let selected = select_excerpt(&original, "Rust ownership", &evidence, &ExcerptAnalyzer::Legacy, &ExcerptOptions::default(), 120)?;
    assert!(selected.text.contains("Rust ownership"));
    let korean_text = "Background. ㄱㅏ is here.";
    let korean_index = Engine::from_entries([KnowledgeEntry::new("ko", korean_text)])?;
    let korean = Retriever::new(LexicalSource::new(&korean_index)).query("가", &options)?;
    let korean_details = &korean.excerpt_details().unwrap()[0];
    assert_eq!(korean_details.reason, ExcerptReason::Terms);
    assert_eq!(korean_details.matches[0].span.slice(korean_text), Some("ㄱㅏ"));
    let _old_literal = RetrievalHit { id: "literal".into(), excerpt: "Rust".into(), span: ExcerptSpan { start: 0, end: 4 },
        text_truncated: false, source_rank: 1, evidence, rerank_score: None, metadata_status: MetadataStatus::Omitted, metadata: None };
    let text = airust::text_search::TextIndex::from_entries(airust::text_search::AnalyzerConfig::german(), [KnowledgeEntry::new("de", format!("{} Häuser und Gärten.", "Einleitung. ".repeat(50)))])?;
    let german = Retriever::new(TextSource::new(&text)).query("Haus Garten", &options)?;
    assert!(german.hits()[0].excerpt.contains("Häuser"));
    assert!(german.hits()[0].excerpt.contains("Gärten"));
    Ok(())
}
fn retrieval_check() -> Result<(), Box<dyn std::error::Error>> {
    use airust::retrieval::*;
    let index = Engine::from_entries([KnowledgeEntry::new("a", "Rust memory safety"), KnowledgeEntry::new("b", "Python scripts")])?;
    let options = RetrievalOptions::new(2).with_context(Context::new().with_entry("a"));
    let response = Retriever::new(LexicalSource::new(&index)).query("Rust", &options)?;
    assert_eq!(response.hits()[0].id, "a");
    assert!(response.to_json().len() <= options.max_output_bytes());
    let text = airust::text_search::TextIndex::from_entries(airust::text_search::AnalyzerConfig::german(), [KnowledgeEntry::new("de", "Häuser mit roten Türen")])?;
    assert_eq!(Retriever::new(TextSource::new(&text)).query("\"Haus mit\"", &RetrievalOptions::new(1))?.hits()[0].id, "de");
    let mut provider = Provider(EmbeddingSpace::new("consumer-v1", 2)?);
    let semantic = SemanticIndex::from_index(index, &mut provider, 8)?;
    assert_eq!(Retriever::new(SemanticSource::new(&semantic, &mut provider)).query("Rust", &options)?.hits()[0].id, "a");
    assert_eq!(Retriever::new(HybridSource::new(&semantic, &mut provider)).query("Rust", &options)?.hits()[0].id, "a");
    Ok(())
}
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
fn storage_check() {
    use airust::storage::{StorageBackend, StoreChange, MemoryStore, read_snapshot, SnapshotOptions};
    for mut store in [Box::new(MemoryStore::new("consumer").unwrap()) as Box<dyn StorageBackend>, Box::new(airust::storage::sqlite::SqliteStore::in_memory("consumer").unwrap())] {
        let before = store.revision().unwrap();
        let after = store.apply(&before, &[StoreChange::Put(KnowledgeEntry::new("stored", "Rust storage"))]).unwrap();
        let loaded = read_snapshot(&mut *store, &SnapshotOptions::default()).unwrap().into_index().unwrap();
        assert_eq!(loaded.revision(), &after.revision);
        assert_eq!(loaded.index().search("Rust", 1)[0].entry.id, "stored");
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    freshness_registry_check()?;
    disk_check()?;
    pipeline_check()?;
    explanations_check()?;
    freshness_check()?;
    gates_check()?;
    reasoning_check()?;
    learned_check()?;
    feedback_check()?;
    adaptive_check()?;
    incremental_check()?;
    shared_check()?;
    progress_check()?;
    budget_check()?;
    execution_check()?;
    assessment_check()?;
    snapshot_check()?;
    excerpt_check()?;
    retrieval_check()?;
    storage_check();
    check_text_search()?;
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
    std::fs::create_dir(&path)?;
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

fn check_text_search() -> Result<(), Box<dyn std::error::Error>> {
    use airust::text_search::{AnalyzerConfig, TextIndex, TextSearchOptions};
    let mut index = TextIndex::from_entries(
        AnalyzerConfig::german(),
        [
            KnowledgeEntry::new("a", "Große Häuser stehen hier."),
            KnowledgeEntry::new("b", "Häuser sind groß."),
        ],
    )?;
    assert_eq!(index.search("Haus", &TextSearchOptions::new(10))?.len(), 2);
    let hits = index.search("\"große Häuser\"", &TextSearchOptions::new(1))?;
    assert_eq!(hits[0].entry.id, "a");
    assert_eq!(
        hits[0].phrases[0].spans[0].slice(&hits[0].entry.text),
        Some("Große Häuser")
    );
    index.remove("a");
    assert!(index
        .search("\"große Häuser\"", &TextSearchOptions::new(1))?
        .is_empty());
    Ok(())
}

fn assessment_check() -> Result<(), Box<dyn std::error::Error>> {
    use airust::retrieval::*;
    let engine = Engine::from_entries([KnowledgeEntry::new("a", "Rust ownership memory")])?;
    let options = RetrievalOptions::new(2).with_evidence_policy(EvidencePolicy::default());
    let mut retriever = Retriever::new(LexicalSource::new(&engine));
    assert_eq!(retriever.query("Rust ownership", &options)?.assessment().unwrap().verdict(), EvidenceVerdict::MeetsPolicy);
    assert_eq!(retriever.query("Rust volcano", &options)?.assessment().unwrap().verdict(), EvidenceVerdict::Insufficient);
    Ok(())
}

fn execution_check() -> Result<(), Box<dyn std::error::Error>> {
    use airust::execution::*;
    let token=CancellationToken::new();let control=WorkControl::new(token.clone());
    let engine=Engine::from_entries([KnowledgeEntry::new("a","Rust")])?;
    assert_eq!(engine.search_controlled("Rust",&airust::SearchOptions::default(),&control)?.len(),1);
    token.cancel();assert!(matches!(engine.search_controlled("Rust",&airust::SearchOptions::default(),&control),Err(ExecutionError::Cancelled)));
    Ok(())
}

fn budget_check() -> Result<(), Box<dyn std::error::Error>> {
    use airust::execution::*;
    use std::time::Duration;
    let control=WorkControl::default().with_timeout(Duration::ZERO)?;
    assert_eq!(control.checkpoint(),Err(ExecutionError::DeadlineExceeded));
    assert_eq!(control.remaining(),Some(Duration::ZERO));
    assert_eq!(control.clone().with_timeout(Duration::from_secs(10))?.deadline(),control.deadline());
    Ok(())
}

fn progress_check() -> Result<(), Box<dyn std::error::Error>> {
    use airust::execution::*;
    use std::sync::{Arc,Mutex};
    let events=Arc::new(Mutex::new(Vec::new()));let captured=events.clone();
    let control=WorkControl::default().with_observer(move|event:&ProgressEvent|captured.lock().unwrap().push(*event));
    control.report(ProgressEvent::new(WorkPhase::Ranking,1,Some(2))?);
    assert_eq!(events.lock().unwrap()[0].fraction(),Some(0.5));assert_eq!(control.dropped_events(),0);
    Ok(())
}

fn shared_check() -> Result<(), Box<dyn std::error::Error>> {
    let old=Engine::from_entries([KnowledgeEntry::new("a","Rust original")])?;
    let mut next=old.clone();next.upsert(KnowledgeEntry::new("a","Rust changed"))?;
    assert_eq!(old.get("a").unwrap().text,"Rust original");assert_eq!(next.get("a").unwrap().text,"Rust changed");
    Ok(())
}

fn incremental_check() -> Result<(), Box<dyn std::error::Error>> {
    use airust::{snapshots::*,text_search::*};
    let manager=DatasetManager::new("consumer-incremental",DatasetConfig::default().with_text_analyzer(AnalyzerConfig::german()))?;
    let empty=manager.snapshot()?;let first=manager.publish(manager.prepare_replace(empty.revision(),vec![DatasetRecord::new(KnowledgeEntry::new("a","Häuser Türen"))],None,None)?)?;
    let next=manager.publish(manager.prepare(first.revision(),DatasetUpdate::new(vec![DatasetChange::Put(DatasetRecord::new(KnowledgeEntry::new("a","Häuser Türen").with_metadata("updated".into())))]),None)?)?;
    assert_eq!(next.stats().text_entries_rebuilt,0);assert_eq!(next.stats().text_entries_reused,1);assert_eq!(next.stats().metadata_only_puts,1);
    let copied=next.text_index().unwrap().clone();assert_eq!(copied.get("a").unwrap().metadata,next.lexical().get("a").unwrap().metadata);Ok(())
}

fn adaptive_check() -> Result<(), Box<dyn std::error::Error>> {
    use airust::{adaptive::*,retrieval::*};
    let index=Engine::from_entries([KnowledgeEntry::new("a","Rust ownership")])?;
    let source=AdaptiveSource::default().with_source("words",SourceRole::Lexical,LexicalSource::new(&index))?;
    let mut retriever=Retriever::new(Box::new(source));let response=retriever.query("Rust ownership",&RetrievalOptions::new(1).with_evidence_policy(EvidencePolicy::default()))?;
    assert_eq!(response.assessment().unwrap().verdict(),EvidenceVerdict::MeetsPolicy);assert_eq!(retriever.source().last_route().unwrap().source(),"words");Ok(())
}

fn feedback_check()->Result<(),Box<dyn std::error::Error>> {use airust::feedback::*;let mut ledger=FeedbackLedger::default();ledger.record(FeedbackEvent::new("vote","consumer","q","d",Rating::Helpful)?)?;let restored=FeedbackLedger::from_json(&ledger.to_json()?,FeedbackLimits::default())?;assert_eq!(restored.evidence("consumer","q","d",10)?.helpful,1);Ok(())}

fn learned_check()->Result<(),Box<dyn std::error::Error>> {use airust::learned_ranking::*;use airust::reranking::Reranker;let examples=[TrainingExample::new("q","q answer",true)?,TrainingExample::new("q","other",false)?];let mut features=LexicalFeatures;let model=train(&examples,&mut features,&TrainingOptions::default())?;let restored=LinearModel::from_json(&model.to_json()?)?;let mut scorer=LinearReranker::new(&restored,&mut features)?;let scores=scorer.score("q",&["q answer","other"])?;assert!(scores[0]>scores[1]);Ok(())}

fn reasoning_check()->Result<(),Box<dyn std::error::Error>> {use airust::reasoning::*;let facts=[Fact::new("ab",Statement::new("a","edge","b")?)?.with_sources(["s".into()])?,Fact::new("bc",Statement::new("b","edge","c")?)?.with_sources(["s".into()])?];let rule=Rule::new("transitive",vec![Pattern::new(Term::variable("a"),"edge",Term::variable("b"))?,Pattern::new(Term::variable("b"),"edge",Term::variable("c"))?],Pattern::new(Term::variable("a"),"edge",Term::variable("c"))?)?;let result=infer(&facts,&[rule],&ReasoningOptions::new("v1",["s".into()])?)?;assert!(result.complete());assert_eq!(result.proposals().len(),1);Ok(())}

fn gates_check()->Result<(),Box<dyn std::error::Error>>{use airust::{quality::*,quality_gates::*};let j=Judgments::new([("a",1)])?;let c=CaseMeasurement::evaluate("q","q",&["a"],&j,1,UnjudgedPolicy::NonRelevant,None)?;let r=EvaluationRun::new("test","0".repeat(64),"r",vec![c],None)?;assert_eq!(compare_runs(&r,&r,&GatePolicy::default())?.verdict(),GateVerdict::Passed);Ok(())}

fn freshness_check()->Result<(),Box<dyn std::error::Error>>{use airust::evidence_freshness::*;let o=FreshnessOptions::default();assert!(EvidenceReceipt::from_json("{}",&o).is_err());SourceVersion::new("source","v1")?;Ok(())}

fn freshness_registry_check() -> Result<(), Box<dyn std::error::Error>> {
use airust::{reasoning::*, evidence_freshness::*};

    let facts = [
        Fact::new("ab", Statement::new("a", "edge", "b")?)?.with_sources(["s".into()])?,
        Fact::new("bc", Statement::new("b", "edge", "c")?)?.with_sources(["s".into()])?,
    ];
    let rules = [Rule::new(
        "transitive",
        vec![
            Pattern::new(Term::variable("a"), "edge", Term::variable("b"))?,
            Pattern::new(Term::variable("b"), "edge", Term::variable("c"))?,
        ],
        Pattern::new(Term::variable("a"), "edge", Term::variable("c"))?,
    )?];
    let result = infer(&facts, &rules, &ReasoningOptions::new("r1", ["s".into()])?)?;
    let sources = [SourceVersion::new("s", "version-1")?];
    let view = EvidenceView::new("r1", &facts, &rules, &sources)?;
    let options = FreshnessOptions::default();
    let receipt = capture_evidence(&result, &view, &options)?;
    assert!(verify_evidence(&receipt, &view, &options)?.all_current());
    let changed = [SourceVersion::new("s", "version-2")?];
    let report = verify_evidence(
        &receipt,
        &EvidenceView::new("r2", &facts, &rules, &changed)?,
        &options,
    )?;
    assert_eq!(report.proofs()[0].state, FreshnessState::Stale);
    println!("{}", report.to_json()?);
    Ok(())
}

fn explanations_check() -> Result<(), Box<dyn std::error::Error>> {
use airust::{learned_ranking::*,rank_explanations::*};

    let data = [
        TrainingExample::new("Rust ownership", "Rust ownership prevents errors", true)?,
        TrainingExample::new("Rust ownership", "unrelated archive", false)?,
    ];
    let mut features = LexicalFeatures;
    let model = train(&data, &mut features, &TrainingOptions::default())?;
    let mut scorer = LinearReranker::new(&model, &mut features)?;
    let result = scorer.score_explained(
        "Rust ownership",
        &["Rust ownership prevents errors", "unrelated archive"],
        &ExplanationOptions::default(),
    )?;
    for doc in result.documents() {
        assert_eq!(
            doc.score(),
            doc.components().iter().map(|c| c.contribution()).sum::<f64>()
        );
    }
    println!("{}", result.to_json()?);
    Ok(())
}

fn pipeline_check() -> Result<(), Box<dyn std::error::Error>> {
use airust::{feedback::*,training_pipeline::*,learned_ranking::*};

    let mut ledger = FeedbackLedger::default();
    for (id, doc, rating) in [
        ("yes", "q answer", Rating::Helpful),
        ("no", "unrelated", Rating::Unhelpful),
    ] {
        ledger.record(FeedbackEvent::new(id, "demo", "q", doc, rating)?)?;
    }
    let policy = CurationPolicy::new(["yes".into(), "no".into()])?.with_minimums(1, 1)?;
    let items = [
        TrainingItem::new("answer", "q", "q answer")?,
        TrainingItem::new("other", "q", "unrelated")?,
    ];
    let data = curate_feedback("demo", &ledger, &items, &policy)?;
    let restored = TrainingDataset::from_json(&data.to_json()?)?;
    assert_eq!(restored.fingerprint()?, data.fingerprint()?);
    let model = train(
        data.examples(),
        &mut LexicalFeatures,
        &TrainingOptions::default(),
    )?;
    println!("{} examples; dataset {}; model {}",data.len(),data.fingerprint()?,model.fingerprint()?);
    Ok(())
}

fn disk_check()->Result<(),Box<dyn std::error::Error>>{use airust::{disk_index::*,storage::StoreChange};let path=std::env::temp_dir().join(format!("airust-registry-disk-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_nanos()));std::fs::create_dir(&path)?;let file=path.join("index.sqlite");let result=(||->Result<(),Box<dyn std::error::Error>>{let mut index=DiskIndex::open(&file,"consumer",DiskOptions::default())?;let r=index.revision()?;let r=index.apply(&r,&[StoreChange::Put(KnowledgeEntry::new("a","Rust ownership"))],&DiskWork::default())?;assert_eq!(index.search("Rust",&DiskSearchOptions::new(1)?)?.hits()[0].entry.id,"a");assert_eq!(index.page(&r,None,10)?.entries.len(),1);index.integrity_check(&DiskWork::default())?;Ok(())})();let _=std::fs::remove_dir_all(path);result}
