#![doc = include_str!("../docs/SEMANTIC.md")]

use crate::{
    index::validate_entry,
    ranking::{Ranked, TopK},
    Context, Engine, IndexError, KnowledgeEntry,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
use thiserror::Error;

#[cfg(feature = "local-embeddings")]
pub mod local;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum EmbeddingError {
    #[cfg(feature = "execution")]
    #[error(transparent)]
    Execution(#[from] crate::execution::ExecutionError),
    #[error("embedding space requires a nonempty model recipe ID and positive dimension")]
    InvalidSpace,
    #[error("expected {expected} vector components, received {actual}")]
    Dimension { expected: usize, actual: usize },
    #[error("vector component {0} is not finite")]
    NonFinite(usize),
    #[error("embedding vector must have a nonzero norm")]
    ZeroVector,
    #[error("embedding spaces do not match: expected {expected:?}, received {actual:?}")]
    SpaceMismatch { expected: String, actual: String },
    #[error("embedding input must not be empty or whitespace-only")]
    EmptyInput,
    #[error("embedding batch size must be positive")]
    BatchSize,
    #[error("provider returned {actual} vectors for {expected} inputs")]
    BatchCount { expected: usize, actual: usize },
    #[error("minimum cosine score must be finite and between -1 and 1")]
    InvalidThreshold,
    #[error("embedding provider failed: {0}")]
    Provider(String),
    #[error(transparent)]
    Index(#[from] IndexError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddingSpace {
    model: String,
    dimensions: usize,
}

impl EmbeddingSpace {
    pub fn new(model: impl Into<String>, dimensions: usize) -> Result<Self, EmbeddingError> {
        let model = model.into();
        if model.trim().is_empty() || dimensions == 0 {
            return Err(EmbeddingError::InvalidSpace);
        }
        Ok(Self { model, dimensions })
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn dimensions(&self) -> usize {
        self.dimensions
    }

    pub(crate) fn require(&self, other: &Self) -> Result<(), EmbeddingError> {
        if self == other {
            Ok(())
        } else {
            Err(EmbeddingError::SpaceMismatch {
                expected: format!("{}:{}", self.model, self.dimensions),
                actual: format!("{}:{}", other.model, other.dimensions),
            })
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Embedding {
    space: EmbeddingSpace,
    values: Vec<f32>,
}

impl Embedding {
    pub fn new(space: EmbeddingSpace, mut values: Vec<f32>) -> Result<Self, EmbeddingError> {
        let norm = vector_norm(&values, space.dimensions)?;
        for value in &mut values {
            *value = (f64::from(*value) / norm) as f32;
        }
        Ok(Self { space, values })
    }

    pub fn space(&self) -> &EmbeddingSpace {
        &self.space
    }

    pub fn values(&self) -> &[f32] {
        &self.values
    }

    pub fn cosine(&self, other: &Self) -> Result<f64, EmbeddingError> {
        self.space.require(&other.space)?;
        cosine_similarity(&self.values, &other.values)
    }
}

pub(crate) fn vector_norm(values: &[f32], dimensions: usize) -> Result<f64, EmbeddingError> {
    if values.len() != dimensions || dimensions == 0 {
        return Err(EmbeddingError::Dimension {
            expected: dimensions,
            actual: values.len(),
        });
    }
    let mut squared = 0.0;
    for (index, &value) in values.iter().enumerate() {
        if !value.is_finite() {
            return Err(EmbeddingError::NonFinite(index));
        }
        squared += f64::from(value) * f64::from(value);
    }
    if squared == 0.0 {
        return Err(EmbeddingError::ZeroVector);
    }
    Ok(squared.sqrt())
}

pub fn cosine_similarity(left: &[f32], right: &[f32]) -> Result<f64, EmbeddingError> {
    let left_norm = vector_norm(left, left.len())?;
    let right_norm = vector_norm(right, left.len())?;
    let dot: f64 = left
        .iter()
        .zip(right)
        .map(|(&a, &b)| f64::from(a) * f64::from(b))
        .sum();
    Ok((dot / left_norm / right_norm).clamp(-1.0, 1.0))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum EmbeddingPurpose {
    Query,
    Document,
}

pub trait EmbeddingProvider {
    fn space(&self) -> &EmbeddingSpace;

    fn embed(
        &mut self,
        texts: &[&str],
        purpose: EmbeddingPurpose,
    ) -> Result<Vec<Vec<f32>>, EmbeddingError>;
}

pub fn embed_batch(
    provider: &mut (impl EmbeddingProvider + ?Sized),
    texts: &[&str],
    purpose: EmbeddingPurpose,
) -> Result<Vec<Embedding>, EmbeddingError> {
    if texts.iter().any(|text| text.trim().is_empty()) {
        return Err(EmbeddingError::EmptyInput);
    }
    if texts.is_empty() {
        return Ok(Vec::new());
    }
    let space = provider.space().clone();
    let vectors = provider.embed(texts, purpose)?;
    space.require(provider.space())?;
    if vectors.len() != texts.len() {
        return Err(EmbeddingError::BatchCount {
            expected: texts.len(),
            actual: vectors.len(),
        });
    }
    vectors
        .into_iter()
        .map(|vector| Embedding::new(space.clone(), vector))
        .collect()
}

#[derive(Debug, Clone)]
pub struct SemanticSearchOptions {
    #[cfg(feature = "execution")]
    control: Option<crate::execution::WorkControl>,
    limit: usize,
    min_score: f64,
    context: Option<Context>,
}

impl SemanticSearchOptions {
    #[cfg(feature = "execution")]
    fn progress(&self, phase: crate::execution::WorkPhase, completed: usize, total: Option<usize>) {
        if let Some(control) = self.control() {
            control.progress(phase, completed, total);
        }
    }

    #[cfg(feature = "execution")]
    pub fn with_control(mut self, control: crate::execution::WorkControl) -> Self {
        self.control = Some(control);
        self
    }
    #[cfg(feature = "execution")]
    pub fn control(&self) -> Option<&crate::execution::WorkControl> {
        self.control.as_ref()
    }
    fn checkpoint(&self) -> Result<(), EmbeddingError> {
        #[cfg(feature = "execution")]
        if let Some(control) = &self.control {
            control
                .checkpoint()
                .map_err(crate::semantic::EmbeddingError::from)?;
        }
        Ok(())
    }

    pub fn new(limit: usize) -> Self {
        Self {
            limit,
            ..Self::default()
        }
    }

    pub fn with_min_score(mut self, min_score: f64) -> Result<Self, EmbeddingError> {
        if !min_score.is_finite() || !(-1.0..=1.0).contains(&min_score) {
            return Err(EmbeddingError::InvalidThreshold);
        }
        self.min_score = min_score;
        Ok(self)
    }

    pub fn with_context(mut self, context: Context) -> Self {
        self.context = Some(context);
        self
    }

    pub fn limit(&self) -> usize {
        self.limit
    }

    pub fn min_score(&self) -> f64 {
        self.min_score
    }

    pub fn context(&self) -> Option<&Context> {
        self.context.as_ref()
    }
}

impl Default for SemanticSearchOptions {
    fn default() -> Self {
        Self {
            #[cfg(feature = "execution")]
            control: None,
            limit: 10,
            min_score: -1.0,
            context: None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[non_exhaustive]
pub struct SemanticResult<'a> {
    pub entry: &'a KnowledgeEntry,
    pub score: f64,
}

impl Ranked for SemanticResult<'_> {
    fn score(&self) -> f64 {
        self.score
    }
    fn id(&self) -> &str {
        &self.entry.id
    }
}

#[derive(Debug, Clone)]
pub struct SemanticIndex {
    lexical: Engine,
    space: EmbeddingSpace,
    vectors: BTreeMap<String, Arc<Vec<f32>>>,
}

impl SemanticIndex {
    #[cfg(feature = "cache")]
    pub(crate) fn from_cached_parts(
        lexical: Engine,
        space: EmbeddingSpace,
        vectors: BTreeMap<String, Vec<f32>>,
    ) -> Self {
        Self {
            lexical,
            space,
            vectors: vectors
                .into_iter()
                .map(|(id, vector)| (id, Arc::new(vector)))
                .collect(),
        }
    }
    pub fn new(space: EmbeddingSpace) -> Self {
        Self {
            lexical: Engine::new(),
            space,
            vectors: BTreeMap::new(),
        }
    }

    pub fn from_index(
        lexical: Engine,
        provider: &mut (impl EmbeddingProvider + ?Sized),
        batch_size: usize,
    ) -> Result<Self, EmbeddingError> {
        if batch_size == 0 {
            return Err(EmbeddingError::BatchSize);
        }
        let space = provider.space().clone();
        let mut vectors = BTreeMap::new();
        let mut entries = lexical.entries();
        loop {
            let batch: Vec<_> = entries.by_ref().take(batch_size).collect();
            if batch.is_empty() {
                break;
            }
            space.require(provider.space())?;
            let texts: Vec<_> = batch.iter().map(|entry| entry.text.as_str()).collect();
            let embedded = embed_batch(provider, &texts, EmbeddingPurpose::Document)?;
            space.require(provider.space())?;
            for (entry, embedding) in batch.into_iter().zip(embedded) {
                vectors.insert(entry.id.clone(), Arc::new(embedding.values));
            }
        }
        drop(entries);
        Ok(Self {
            lexical,
            space,
            vectors,
        })
    }

    #[cfg(feature = "hnsw")]
    pub(crate) fn shared_embeddings(&self) -> impl Iterator<Item = (&str, Arc<Vec<f32>>)> {
        self.vectors
            .iter()
            .map(|(id, vector)| (id.as_str(), Arc::clone(vector)))
    }

    pub fn space(&self) -> &EmbeddingSpace {
        &self.space
    }
    pub fn lexical(&self) -> &Engine {
        &self.lexical
    }

    pub fn into_lexical(self) -> Engine {
        self.lexical
    }
    pub fn len(&self) -> usize {
        self.lexical.len()
    }
    pub fn is_empty(&self) -> bool {
        self.lexical.is_empty()
    }
    pub fn embedding(&self, id: &str) -> Option<&[f32]> {
        self.vectors.get(id).map(|vector| vector.as_slice())
    }
    #[cfg(feature = "snapshots")]
    pub(crate) fn replace_unchanged_text(
        &mut self,
        entry: KnowledgeEntry,
    ) -> Result<Option<KnowledgeEntry>, EmbeddingError> {
        if self
            .lexical
            .get(&entry.id)
            .is_none_or(|old| old.text != entry.text)
        {
            return Ok(None);
        }
        Ok(self.lexical.upsert(entry)?)
    }

    pub fn upsert_embedded(
        &mut self,
        entry: KnowledgeEntry,
        embedding: Embedding,
    ) -> Result<Option<KnowledgeEntry>, EmbeddingError> {
        self.space.require(embedding.space())?;
        let normalized = validate_entry(&entry)?;
        self.vectors
            .insert(entry.id.clone(), Arc::new(embedding.values));
        Ok(self.lexical.insert_validated(entry, normalized))
    }

    pub fn upsert(
        &mut self,
        entry: KnowledgeEntry,
        provider: &mut (impl EmbeddingProvider + ?Sized),
    ) -> Result<Option<KnowledgeEntry>, EmbeddingError> {
        let normalized = validate_entry(&entry)?;
        self.space.require(provider.space())?;
        let mut embedded = embed_batch(provider, &[&entry.text], EmbeddingPurpose::Document)?;
        self.space.require(provider.space())?;
        let embedding = embedded.pop().expect("validated single embedding");
        self.vectors
            .insert(entry.id.clone(), Arc::new(embedding.values));
        Ok(self.lexical.insert_validated(entry, normalized))
    }

    pub fn upsert_batch(
        &mut self,
        entries: Vec<KnowledgeEntry>,
        provider: &mut (impl EmbeddingProvider + ?Sized),
        batch_size: usize,
    ) -> Result<usize, EmbeddingError> {
        if batch_size == 0 {
            return Err(EmbeddingError::BatchSize);
        }
        self.space.require(provider.space())?;
        let mut ids = BTreeSet::new();
        let mut prepared = Vec::with_capacity(entries.len());
        for entry in entries {
            if !ids.insert(entry.id.clone()) {
                return Err(IndexError::DuplicateId(entry.id).into());
            }
            let normalized = validate_entry(&entry)?;
            prepared.push((entry, normalized));
        }
        let mut embeddings = Vec::with_capacity(prepared.len());
        for batch in prepared.chunks(batch_size) {
            self.space.require(provider.space())?;
            let texts: Vec<_> = batch.iter().map(|(entry, _)| entry.text.as_str()).collect();
            embeddings.extend(embed_batch(provider, &texts, EmbeddingPurpose::Document)?);
            self.space.require(provider.space())?;
        }
        let count = prepared.len();
        for ((entry, normalized), embedding) in prepared.into_iter().zip(embeddings) {
            self.vectors
                .insert(entry.id.clone(), Arc::new(embedding.values));
            self.lexical.insert_validated(entry, normalized);
        }
        Ok(count)
    }

    pub fn remove(&mut self, id: &str) -> Option<KnowledgeEntry> {
        self.vectors.remove(id);
        self.lexical.remove(id)
    }

    pub fn clear(&mut self) {
        self.vectors.clear();
        self.lexical.clear();
    }

    pub fn search(
        &self,
        query: &str,
        provider: &mut (impl EmbeddingProvider + ?Sized),
        options: &SemanticSearchOptions,
    ) -> Result<Vec<SemanticResult<'_>>, EmbeddingError> {
        options.checkpoint()?;
        if query.trim().is_empty() {
            return Err(EmbeddingError::EmptyInput);
        }
        self.space.require(provider.space())?;
        if options.limit == 0 || self.is_empty() {
            return Ok(Vec::new());
        }
        #[cfg(feature = "execution")]
        options.progress(crate::execution::WorkPhase::Embedding, 0, Some(1));
        options.checkpoint()?;
        let embeddings = embed_batch(provider, &[query], EmbeddingPurpose::Query);
        options.checkpoint()?;
        let mut embeddings = embeddings?;
        #[cfg(feature = "execution")]
        options.progress(crate::execution::WorkPhase::Embedding, 1, Some(1));
        self.space.require(provider.space())?;
        self.search_embedding(
            &embeddings.pop().expect("validated single embedding"),
            options,
        )
    }

    pub fn search_embedding(
        &self,
        query: &Embedding,
        options: &SemanticSearchOptions,
    ) -> Result<Vec<SemanticResult<'_>>, EmbeddingError> {
        options.checkpoint()?;
        self.space.require(query.space())?;
        let mut results = TopK::new(options.limit);
        if options.limit == 0 {
            return Ok(Vec::new());
        }
        #[cfg(feature = "execution")]
        options.progress(
            crate::execution::WorkPhase::Ranking,
            0,
            Some(self.vectors.len()),
        );
        for (position, (id, vector)) in self.vectors.iter().enumerate() {
            #[cfg(not(feature = "execution"))]
            let _ = position;
            #[cfg(feature = "execution")]
            if position.is_multiple_of(512) {
                options.progress(
                    crate::execution::WorkPhase::Ranking,
                    position,
                    Some(self.vectors.len()),
                );
            }
            options.checkpoint()?;
            let entry = self
                .lexical
                .get(id)
                .expect("semantic and lexical IDs agree");
            if options
                .context
                .as_ref()
                .is_some_and(|context| !context.includes(entry))
            {
                continue;
            }
            let score = cosine_similarity(query.values(), vector)?;
            if score >= options.min_score {
                results.push(SemanticResult { entry, score });
            }
        }
        options.checkpoint()?;
        let results = results.finish();
        #[cfg(feature = "execution")]
        options.progress(
            crate::execution::WorkPhase::Ranking,
            self.vectors.len(),
            Some(self.vectors.len()),
        );
        options.checkpoint()?;
        Ok(results)
    }
}

#[cfg(all(feature = "execution", feature = "snapshots"))]
pub(crate) struct ControlledProvider<'a> {
    pub(crate) provider: &'a mut dyn EmbeddingProvider,
    pub(crate) control: &'a crate::execution::WorkControl,
}
#[cfg(all(feature = "execution", feature = "snapshots"))]
impl EmbeddingProvider for ControlledProvider<'_> {
    fn space(&self) -> &EmbeddingSpace {
        self.provider.space()
    }
    fn embed(
        &mut self,
        texts: &[&str],
        purpose: EmbeddingPurpose,
    ) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        self.control
            .progress(crate::execution::WorkPhase::Embedding, 0, Some(texts.len()));
        self.control.checkpoint()?;
        let result = self.provider.embed(texts, purpose);
        self.control.checkpoint()?;
        if result.is_ok() {
            self.control.progress(
                crate::execution::WorkPhase::Embedding,
                texts.len(),
                Some(texts.len()),
            );
        }
        result
    }
}
