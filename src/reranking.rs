#![doc = include_str!("../docs/RERANKING.md")]

use crate::{Context, KnowledgeEntry, SearchResult};
use serde::Serialize;
use std::collections::BTreeSet;
use thiserror::Error;

#[cfg(feature = "local-reranking")]
pub mod local;

#[derive(Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum RerankError {
    #[cfg(feature = "execution")]
    #[error(transparent)]
    Execution(#[from] crate::execution::ExecutionError),
    #[error("invalid reranking configuration: {0}")]
    Configuration(&'static str),
    #[error("reranking query must not be empty or whitespace-only")]
    EmptyQuery,
    #[error("selected reranking candidate at input rank {rank} has an empty {field}")]
    InvalidCandidate { rank: usize, field: &'static str },
    #[error("selected reranking candidates contain duplicate ID at input rank {0}")]
    DuplicateCandidate(usize),
    #[error("reranking {0} byte budget exceeded")]
    Budget(&'static str),
    #[error("reranker recipe must be nonempty and at most 1024 bytes")]
    InvalidRecipe,
    #[error("reranker recipe changed during scoring")]
    RecipeChanged,
    #[error("reranker returned {actual} scores for {expected} documents")]
    ScoreCount { expected: usize, actual: usize },
    #[error("reranker returned a non-finite score at input rank {0}")]
    NonFiniteScore(usize),
    #[error("reranking provider failed: {0}")]
    Provider(String),
    #[error("invalid reranker output: {0}")]
    Output(&'static str),
}

pub trait Reranker {
    fn recipe(&self) -> &str;
    fn score(&mut self, query: &str, documents: &[&str]) -> Result<Vec<f64>, RerankError>;
}

pub trait Rerankable {
    fn entry(&self) -> &KnowledgeEntry;
}

impl Rerankable for KnowledgeEntry {
    fn entry(&self) -> &KnowledgeEntry {
        self
    }
}
impl<T: Rerankable + ?Sized> Rerankable for &T {
    fn entry(&self) -> &KnowledgeEntry {
        (**self).entry()
    }
}
impl Rerankable for SearchResult<'_> {
    fn entry(&self) -> &KnowledgeEntry {
        self.entry
    }
}
#[cfg(feature = "embeddings")]
impl Rerankable for crate::semantic::SemanticResult<'_> {
    fn entry(&self) -> &KnowledgeEntry {
        self.entry
    }
}
#[cfg(feature = "hybrid")]
impl Rerankable for crate::hybrid::HybridResult<'_> {
    fn entry(&self) -> &KnowledgeEntry {
        self.entry
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum RerankFailurePolicy {
    #[default]
    ReturnError,
    KeepOriginalOrder,
}

#[derive(Debug, Clone)]
pub struct RerankOptions {
    #[cfg(feature = "execution")]
    control: Option<crate::execution::WorkControl>,
    limit: usize,
    candidates: usize,
    batch_size: usize,
    max_query_bytes: usize,
    max_document_bytes: usize,
    max_pair_bytes: usize,
    min_score: Option<f64>,
    context: Option<Context>,
    failure_policy: RerankFailurePolicy,
}
impl Default for RerankOptions {
    fn default() -> Self {
        Self {
            #[cfg(feature = "execution")]
            control: None,
            limit: 10,
            candidates: 100,
            batch_size: 16,
            max_query_bytes: 16 * 1024,
            max_document_bytes: 64 * 1024,
            max_pair_bytes: 2 * 1024 * 1024,
            min_score: None,
            context: None,
            failure_policy: RerankFailurePolicy::ReturnError,
        }
    }
}
impl RerankOptions {
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
    fn checkpoint(&self) -> Result<(), RerankError> {
        #[cfg(feature = "execution")]
        if let Some(control) = &self.control {
            control.checkpoint()?;
        }
        Ok(())
    }

    pub fn new(limit: usize) -> Self {
        Self {
            limit,
            candidates: limit.max(100),
            ..Self::default()
        }
    }
    pub fn with_candidates(mut self, candidates: usize) -> Result<Self, RerankError> {
        if candidates == 0 || candidates < self.limit {
            return Err(RerankError::Configuration(
                "candidate limit must be positive and at least the result limit",
            ));
        }
        self.candidates = candidates;
        Ok(self)
    }
    pub fn with_batch_size(mut self, batch_size: usize) -> Result<Self, RerankError> {
        if batch_size == 0 {
            return Err(RerankError::Configuration("batch size must be positive"));
        }
        self.batch_size = batch_size;
        Ok(self)
    }
    pub fn with_byte_budgets(
        mut self,
        query: usize,
        document: usize,
        pairs: usize,
    ) -> Result<Self, RerankError> {
        if query == 0 || document == 0 || pairs == 0 {
            return Err(RerankError::Configuration("byte budgets must be positive"));
        }
        self.max_query_bytes = query;
        self.max_document_bytes = document;
        self.max_pair_bytes = pairs;
        Ok(self)
    }
    pub fn with_min_score(mut self, score: f64) -> Result<Self, RerankError> {
        if !score.is_finite() {
            return Err(RerankError::Configuration("minimum score must be finite"));
        }
        self.min_score = Some(score);
        Ok(self)
    }
    pub fn with_context(mut self, context: Context) -> Self {
        self.context = Some(context);
        self
    }
    pub fn with_failure_policy(mut self, policy: RerankFailurePolicy) -> Self {
        self.failure_policy = policy;
        self
    }
    pub fn limit(&self) -> usize {
        self.limit
    }
    pub fn candidates(&self) -> usize {
        self.candidates
    }
    pub fn batch_size(&self) -> usize {
        self.batch_size
    }
    pub fn max_query_bytes(&self) -> usize {
        self.max_query_bytes
    }
    pub fn max_document_bytes(&self) -> usize {
        self.max_document_bytes
    }
    pub fn max_pair_bytes(&self) -> usize {
        self.max_pair_bytes
    }
    pub fn min_score(&self) -> Option<f64> {
        self.min_score
    }
    pub fn context(&self) -> Option<&Context> {
        self.context.as_ref()
    }
    pub fn failure_policy(&self) -> RerankFailurePolicy {
        self.failure_policy
    }
}

#[derive(Debug, Serialize)]
pub struct RerankResult<'a, T> {
    pub candidate: &'a T,
    pub original_rank: usize,
    pub score: Option<f64>,
    #[serde(skip)]
    entry: &'a KnowledgeEntry,
}
impl<T: Rerankable> RerankResult<'_, T> {
    pub fn entry(&self) -> &KnowledgeEntry {
        self.entry
    }
}

#[derive(Debug, Serialize)]
pub struct RerankOutcome<'a, T> {
    results: Vec<RerankResult<'a, T>>,
    recipe: Option<String>,
    reranker_used: bool,
    fallback_reason: Option<String>,
    received_candidates: usize,
    eligible_candidates: usize,
    selected_candidates: usize,
    scored_candidates: usize,
}
impl<'a, T> RerankOutcome<'a, T> {
    pub fn results(&self) -> &[RerankResult<'a, T>] {
        &self.results
    }
    pub fn into_results(self) -> Vec<RerankResult<'a, T>> {
        self.results
    }
    pub fn recipe(&self) -> Option<&str> {
        self.recipe.as_deref()
    }
    pub fn reranker_used(&self) -> bool {
        self.reranker_used
    }
    pub fn fallback_reason(&self) -> Option<&str> {
        self.fallback_reason.as_deref()
    }
    pub fn received_candidates(&self) -> usize {
        self.received_candidates
    }
    pub fn eligible_candidates(&self) -> usize {
        self.eligible_candidates
    }
    pub fn selected_candidates(&self) -> usize {
        self.selected_candidates
    }
    pub fn scored_candidates(&self) -> usize {
        self.scored_candidates
    }
}

pub fn rerank<'a, T: Rerankable>(
    query: &str,
    candidates: &'a [T],
    provider: &mut (impl Reranker + ?Sized),
    options: &RerankOptions,
) -> Result<RerankOutcome<'a, T>, RerankError> {
    options.checkpoint()?;
    if query.trim().is_empty() {
        return Err(RerankError::EmptyQuery);
    }
    if query.len() > options.max_query_bytes {
        return Err(RerankError::Budget("query"));
    }
    let mut outcome = RerankOutcome {
        results: Vec::new(),
        recipe: None,
        reranker_used: false,
        fallback_reason: None,
        received_candidates: candidates.len(),
        eligible_candidates: 0,
        selected_candidates: 0,
        scored_candidates: 0,
    };
    let mut selected = Vec::new();
    let mut ids = BTreeSet::new();
    let mut bytes = 0usize;
    for (position, candidate) in candidates.iter().enumerate() {
        options.checkpoint()?;
        let entry = candidate.entry();
        if options.context.as_ref().is_some_and(|c| !c.includes(entry)) {
            continue;
        }
        outcome.eligible_candidates += 1;
        if options.limit == 0 || selected.len() >= options.candidates {
            continue;
        }
        let rank = position + 1;
        for (field, value) in [("ID", entry.id.as_str()), ("text", entry.text.as_str())] {
            if value.trim().is_empty() {
                return Err(RerankError::InvalidCandidate { rank, field });
            }
        }
        if !ids.insert(entry.id.as_str()) {
            return Err(RerankError::DuplicateCandidate(rank));
        }
        if entry.text.len() > options.max_document_bytes {
            return Err(RerankError::Budget("document"));
        }
        bytes = bytes
            .checked_add(query.len())
            .and_then(|n| n.checked_add(entry.text.len()))
            .filter(|&n| n <= options.max_pair_bytes)
            .ok_or(RerankError::Budget("query/document pairs"))?;
        selected.push(RerankResult {
            candidate,
            original_rank: rank,
            score: None,
            entry,
        });
    }
    outcome.selected_candidates = selected.len();
    if selected.is_empty() {
        return Ok(outcome);
    }
    if provider.recipe().trim().is_empty() || provider.recipe().len() > 1024 {
        return Err(RerankError::InvalidRecipe);
    }
    let recipe = provider.recipe().to_owned();
    outcome.recipe = Some(recipe.clone());
    #[cfg(feature = "execution")]
    options.progress(
        crate::execution::WorkPhase::Reranking,
        0,
        Some(selected.len()),
    );
    let mut scores = Vec::with_capacity(selected.len());
    for batch in selected.chunks(options.batch_size) {
        options.checkpoint()?;
        let texts: Vec<_> = batch.iter().map(|hit| hit.entry().text.as_str()).collect();
        let result = provider.score(query, &texts);
        options.checkpoint()?;
        if provider.recipe() != recipe {
            return Err(RerankError::RecipeChanged);
        }
        let batch_scores = match result {
            Ok(values) => values,
            Err(error @ RerankError::Provider(_))
                if options.failure_policy == RerankFailurePolicy::KeepOriginalOrder
                    && options.min_score.is_none() =>
            {
                outcome.fallback_reason = Some(error.to_string());
                selected.truncate(options.limit);
                outcome.results = selected;
                return Ok(outcome);
            }
            Err(error) => return Err(error),
        };
        if batch_scores.len() != batch.len() {
            return Err(RerankError::ScoreCount {
                expected: batch.len(),
                actual: batch_scores.len(),
            });
        }
        for (hit, score) in batch.iter().zip(&batch_scores) {
            if !score.is_finite() {
                return Err(RerankError::NonFiniteScore(hit.original_rank));
            }
        }
        outcome.scored_candidates += batch_scores.len();
        #[cfg(feature = "execution")]
        options.progress(
            crate::execution::WorkPhase::Reranking,
            outcome.scored_candidates,
            Some(selected.len()),
        );
        scores.extend(batch_scores);
    }
    for (hit, score) in selected.iter_mut().zip(scores) {
        hit.score = Some(score);
    }
    selected.retain(|hit| {
        options
            .min_score
            .is_none_or(|min| hit.score.unwrap() >= min)
    });
    selected.sort_unstable_by(|a, b| {
        b.score
            .unwrap()
            .partial_cmp(&a.score.unwrap())
            .unwrap()
            .then_with(|| a.original_rank.cmp(&b.original_rank))
    });
    selected.truncate(options.limit);
    options.checkpoint()?;
    outcome.reranker_used = true;
    outcome.results = selected;
    Ok(outcome)
}
