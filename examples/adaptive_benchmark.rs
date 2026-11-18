use airust::{adaptive::*, retrieval::*, semantic::*, Engine, KnowledgeEntry};
use serde_json::json;
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Instant,
};
struct Provider {
    space: EmbeddingSpace,
    calls: Arc<AtomicUsize>,
}
impl EmbeddingProvider for Provider {
    fn space(&self) -> &EmbeddingSpace {
        &self.space
    }
    fn embed(
        &mut self,
        texts: &[&str],
        purpose: EmbeddingPurpose,
    ) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        if purpose == EmbeddingPurpose::Query {
            self.calls.fetch_add(1, Ordering::Relaxed);
        }
        Ok(vec![vec![1.0, 0.0]; texts.len()])
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let space = EmbeddingSpace::new("handwritten-routing-counter", 2)?;
    let mut build = Provider {
        space: space.clone(),
        calls: Arc::new(AtomicUsize::new(0)),
    };
    let index = SemanticIndex::from_index(
        Engine::from_entries([KnowledgeEntry::new(
            "a",
            "Rust ownership prevents memory errors",
        )])?,
        &mut build,
        8,
    )?;
    let mut rows = Vec::new();
    for adaptive in [false, true] {
        let calls = Arc::new(AtomicUsize::new(0));
        let mut provider = Provider {
            space: space.clone(),
            calls: calls.clone(),
        };
        let started = Instant::now();
        if adaptive {
            let source = AdaptiveSource::default()
                .with_source(
                    "words",
                    SourceRole::Lexical,
                    LexicalSource::new(index.lexical()),
                )?
                .with_source(
                    "configured-hybrid",
                    SourceRole::Hybrid,
                    HybridSource::new(&index, &mut provider),
                )?;
            let mut retriever = Retriever::new(source);
            for i in 0usize..1000 {
                let query = if i.is_multiple_of(5) {
                    "How does Rust ownership prevent memory errors"
                } else {
                    "Rust ownership"
                };
                assert_eq!(
                    retriever.query(query, &RetrievalOptions::new(1))?.hits()[0].id,
                    "a"
                );
            }
        } else {
            let mut retriever = Retriever::new(HybridSource::new(&index, &mut provider));
            for i in 0usize..1000 {
                let query = if i.is_multiple_of(5) {
                    "How does Rust ownership prevent memory errors"
                } else {
                    "Rust ownership"
                };
                assert_eq!(
                    retriever.query(query, &RetrievalOptions::new(1))?.hits()[0].id,
                    "a"
                );
            }
        }
        let count = calls.load(Ordering::Relaxed);
        assert_eq!(count, if adaptive { 200 } else { 1000 });
        rows.push(json!({"adaptive":adaptive,"queries":1000,"query_embedding_calls":count,"elapsed_ms":started.elapsed().as_secs_f64()*1000.0}));
    }
    println!(
        "{}",
        json!({"fixture":"routing-provider-count-v1","rows":rows,"limits":"One document and constant handwritten 2D vectors: checks orchestration/call avoidance, not real model quality or inference latency. Router overhead may dominate this deliberately cheap provider."})
    );
    Ok(())
}
