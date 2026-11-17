#![doc = include_str!("../docs/ANN.md")]

use crate::{
    ranking::TopK,
    semantic::{
        cosine_similarity, embed_batch, Embedding, EmbeddingError, EmbeddingProvider,
        EmbeddingPurpose, SemanticIndex, SemanticResult, SemanticSearchOptions,
    },
    KnowledgeEntry,
};
use hnsw_rs::prelude::{DistL2, Distance, Hnsw};
use std::{collections::BTreeSet, fmt, sync::Arc};
use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum AnnError {
    #[error("invalid HNSW configuration: {0}")]
    Configuration(&'static str),
    #[error(transparent)]
    Embedding(#[from] EmbeddingError),
}

#[derive(Debug, Clone)]
pub struct HnswConfig {
    connections: usize,
    layers: usize,
    ef_construction: usize,
    exact_threshold: usize,
    max_pending: usize,
}
impl Default for HnswConfig {
    fn default() -> Self {
        Self {
            connections: 16,
            layers: 16,
            ef_construction: 200,
            exact_threshold: 512,
            max_pending: 1000,
        }
    }
}
impl HnswConfig {
    pub fn new(
        connections: usize,
        layers: usize,
        ef_construction: usize,
    ) -> Result<Self, AnnError> {
        if !(2..=64).contains(&connections)
            || !(1..=16).contains(&layers)
            || ef_construction <= 2 * connections
            || ef_construction > 100_000
        {
            return Err(AnnError::Configuration("connections 2..64, layers 1..16, construction width > 2*connections and <= 100000 required"));
        }
        Ok(Self {
            connections,
            layers,
            ef_construction,
            ..Self::default()
        })
    }
    pub fn with_exact_threshold(mut self, entries: usize) -> Self {
        self.exact_threshold = entries;
        self
    }
    pub fn with_max_pending(mut self, entries: usize) -> Self {
        self.max_pending = entries;
        self
    }
    pub fn connections(&self) -> usize {
        self.connections
    }
    pub fn layers(&self) -> usize {
        self.layers
    }
    pub fn ef_construction(&self) -> usize {
        self.ef_construction
    }
    pub fn exact_threshold(&self) -> usize {
        self.exact_threshold
    }
    pub fn max_pending(&self) -> usize {
        self.max_pending
    }
}

#[derive(Debug, Clone)]
pub struct AnnSearchOptions {
    semantic: SemanticSearchOptions,
    ef_search: usize,
}
impl Default for AnnSearchOptions {
    fn default() -> Self {
        Self {
            semantic: SemanticSearchOptions::default(),
            ef_search: 100,
        }
    }
}
impl AnnSearchOptions {
    pub fn new(limit: usize) -> Self {
        Self {
            semantic: SemanticSearchOptions::new(limit),
            ..Self::default()
        }
    }
    pub fn with_semantic_options(mut self, options: SemanticSearchOptions) -> Self {
        self.semantic = options;
        self
    }
    pub fn with_ef_search(mut self, candidates: usize) -> Result<Self, AnnError> {
        if candidates == 0 || candidates > 1_000_000 {
            return Err(AnnError::Configuration("search width must be 1..1000000"));
        }
        self.ef_search = candidates;
        Ok(self)
    }
    pub fn semantic_options(&self) -> &SemanticSearchOptions {
        &self.semantic
    }
    pub fn ef_search(&self) -> usize {
        self.ef_search
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[non_exhaustive]
pub enum AnnSearchMode {
    Hnsw,
    ExactSmallIndex,
    ExactScope,
    ExactPendingChanges,
    ExactResultWindow,
    Empty,
}
#[derive(Debug, Clone, serde::Serialize)]
pub struct AnnOutcome<'a> {
    results: Vec<SemanticResult<'a>>,
    mode: AnnSearchMode,
    candidates_scored: usize,
}
impl<'a> AnnOutcome<'a> {
    pub fn results(&self) -> &[SemanticResult<'a>] {
        &self.results
    }
    pub fn into_results(self) -> Vec<SemanticResult<'a>> {
        self.results
    }
    pub fn mode(&self) -> AnnSearchMode {
        self.mode
    }
    pub fn candidates_scored(&self) -> usize {
        self.candidates_scored
    }
}

#[derive(Debug)]
struct SharedDistance;
impl Distance<Arc<Vec<f32>>> for SharedDistance {
    fn eval(&self, left: &[Arc<Vec<f32>>], right: &[Arc<Vec<f32>>]) -> f32 {
        DistL2.eval(left[0].as_slice(), right[0].as_slice())
    }
}
type Topology = Hnsw<'static, Arc<Vec<f32>>, SharedDistance>;
pub struct HnswIndex {
    semantic: SemanticIndex,
    topology: Topology,
    ids: Vec<String>,
    pending: BTreeSet<String>,
    config: HnswConfig,
}
impl fmt::Debug for HnswIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HnswIndex")
            .field("entries", &self.semantic.len())
            .field("indexed_points", &self.ids.len())
            .field("pending_changes", &self.pending.len())
            .field("config", &self.config)
            .finish()
    }
}
impl HnswIndex {
    pub fn from_semantic(semantic: SemanticIndex, config: HnswConfig) -> Self {
        let (topology, ids) = Self::build(&semantic, &config);
        Self {
            semantic,
            topology,
            ids,
            pending: BTreeSet::new(),
            config,
        }
    }
    fn build(semantic: &SemanticIndex, config: &HnswConfig) -> (Topology, Vec<String>) {
        let mut topology = Hnsw::new(
            config.connections,
            semantic.len().max(1),
            config.layers,
            config.ef_construction,
            SharedDistance,
        );
        let mut ids = Vec::with_capacity(semantic.len());
        for (id, vector) in semantic.shared_embeddings() {
            topology.insert((&[vector], ids.len()));
            ids.push(id.to_owned());
        }
        topology.set_searching_mode(true);
        (topology, ids)
    }
    pub fn semantic(&self) -> &SemanticIndex {
        &self.semantic
    }
    pub fn into_semantic(self) -> SemanticIndex {
        self.semantic
    }
    pub fn config(&self) -> &HnswConfig {
        &self.config
    }
    pub fn len(&self) -> usize {
        self.semantic.len()
    }
    pub fn is_empty(&self) -> bool {
        self.semantic.is_empty()
    }
    pub fn indexed_points(&self) -> usize {
        self.ids.len()
    }
    pub fn pending_changes(&self) -> usize {
        self.pending.len()
    }
    pub fn needs_rebuild(&self) -> bool {
        self.pending.len() > self.config.max_pending
    }
    pub fn rebuild(&mut self) {
        let (topology, ids) = Self::build(&self.semantic, &self.config);
        self.topology = topology;
        self.ids = ids;
        self.pending.clear();
    }
    pub fn upsert_embedded(
        &mut self,
        entry: KnowledgeEntry,
        embedding: Embedding,
    ) -> Result<Option<KnowledgeEntry>, AnnError> {
        let id = entry.id.clone();
        let previous = self.semantic.upsert_embedded(entry, embedding)?;
        self.pending.insert(id);
        Ok(previous)
    }
    pub fn upsert(
        &mut self,
        entry: KnowledgeEntry,
        provider: &mut (impl EmbeddingProvider + ?Sized),
    ) -> Result<Option<KnowledgeEntry>, AnnError> {
        let id = entry.id.clone();
        let previous = self.semantic.upsert(entry, provider)?;
        self.pending.insert(id);
        Ok(previous)
    }
    pub fn upsert_batch(
        &mut self,
        entries: Vec<KnowledgeEntry>,
        provider: &mut (impl EmbeddingProvider + ?Sized),
        batch_size: usize,
    ) -> Result<usize, AnnError> {
        let ids: Vec<_> = entries.iter().map(|e| e.id.clone()).collect();
        let count = self.semantic.upsert_batch(entries, provider, batch_size)?;
        self.pending.extend(ids);
        Ok(count)
    }
    pub fn remove(&mut self, id: &str) -> Option<KnowledgeEntry> {
        let previous = self.semantic.remove(id);
        if previous.is_some() {
            self.pending.insert(id.to_owned());
        }
        previous
    }
    pub fn clear(&mut self) {
        self.semantic.clear();
        self.rebuild();
    }
    pub fn search(
        &self,
        query: &str,
        provider: &mut (impl EmbeddingProvider + ?Sized),
        options: &AnnSearchOptions,
    ) -> Result<AnnOutcome<'_>, AnnError> {
        if query.trim().is_empty() {
            return Err(EmbeddingError::EmptyInput.into());
        }
        self.semantic.space().require(provider.space())?;
        if options.semantic.limit() == 0 || self.is_empty() {
            return Ok(AnnOutcome {
                results: Vec::new(),
                mode: AnnSearchMode::Empty,
                candidates_scored: 0,
            });
        }
        let mut embeddings = embed_batch(provider, &[query], EmbeddingPurpose::Query)?;
        self.semantic.space().require(provider.space())?;
        self.search_embedding(
            &embeddings.pop().expect("validated single embedding"),
            options,
        )
    }
    pub fn search_embedding(
        &self,
        query: &Embedding,
        options: &AnnSearchOptions,
    ) -> Result<AnnOutcome<'_>, AnnError> {
        self.semantic.space().require(query.space())?;
        let mode = if options.semantic.limit() == 0 || self.is_empty() {
            AnnSearchMode::Empty
        } else if options.semantic.context().is_some() {
            AnnSearchMode::ExactScope
        } else if self.len() <= self.config.exact_threshold {
            AnnSearchMode::ExactSmallIndex
        } else if self.needs_rebuild() {
            AnnSearchMode::ExactPendingChanges
        } else if options.semantic.limit() > options.ef_search {
            AnnSearchMode::ExactResultWindow
        } else {
            AnnSearchMode::Hnsw
        };
        if mode != AnnSearchMode::Hnsw {
            let results = self.semantic.search_embedding(query, &options.semantic)?;
            let candidates_scored = if mode == AnnSearchMode::Empty {
                0
            } else {
                self.semantic
                    .lexical()
                    .entries()
                    .filter(|e| options.semantic.context().is_none_or(|c| c.includes(e)))
                    .count()
            };
            return Ok(AnnOutcome {
                results,
                mode,
                candidates_scored,
            });
        }
        let query_vector = Arc::new(query.values().to_vec());
        let candidates = options
            .ef_search
            .saturating_add(self.pending.len())
            .min(self.ids.len());
        let found = self.topology.search(
            &[query_vector],
            candidates,
            candidates.max(options.ef_search),
        );
        let mut selected = TopK::new(options.semantic.limit());
        let mut seen = BTreeSet::new();
        for neighbour in found {
            if let Some(id) = self.ids.get(neighbour.d_id) {
                if !self.pending.contains(id) {
                    seen.insert(id.as_str());
                }
            }
        }
        for id in &self.pending {
            if self.semantic.lexical().get(id).is_some() {
                seen.insert(id.as_str());
            }
        }
        let candidates_scored = seen.len();
        for id in seen {
            let entry = self.semantic.lexical().get(id).expect("live ANN candidate");
            let vector = self.semantic.embedding(id).expect("semantic vector exists");
            let score = cosine_similarity(query.values(), vector)?;
            if score >= options.semantic.min_score() {
                selected.push(SemanticResult { entry, score });
            }
        }
        Ok(AnnOutcome {
            results: selected.finish(),
            mode,
            candidates_scored,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::EmbeddingSpace;

    #[test]
    fn topology_shares_original_vector_buffers_and_rebuild_drops_stale_points() {
        let space = EmbeddingSpace::new("sharing", 2).unwrap();
        let mut semantic = SemanticIndex::new(space.clone());
        semantic
            .upsert_embedded(
                KnowledgeEntry::new("a", "original"),
                Embedding::new(space.clone(), vec![1.0, 0.0]).unwrap(),
            )
            .unwrap();
        let pointer = semantic.embedding("a").unwrap().as_ptr();
        let mut ann = HnswIndex::from_semantic(semantic, HnswConfig::default());
        for point in ann.topology.get_point_indexation() {
            assert_eq!(point.get_v()[0].as_ptr(), pointer);
        }
        ann.upsert_embedded(
            KnowledgeEntry::new("a", "replacement"),
            Embedding::new(space, vec![0.0, 1.0]).unwrap(),
        )
        .unwrap();
        assert_ne!(ann.semantic().embedding("a").unwrap().as_ptr(), pointer);
        ann.rebuild();
        for point in ann.topology.get_point_indexation() {
            assert_eq!(
                point.get_v()[0].as_ptr(),
                ann.semantic().embedding("a").unwrap().as_ptr()
            );
        }
    }
}
