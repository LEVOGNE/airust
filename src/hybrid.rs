#![doc = include_str!("../docs/HYBRID.md")]

use crate::{
    ranking::{Ranked, TopK},
    semantic::{EmbeddingError, EmbeddingProvider, SemanticIndex, SemanticSearchOptions},
    Bm25Config, Context, KnowledgeEntry, SearchAlgorithm, SearchOptions,
};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum HybridError {
    #[error("invalid hybrid search configuration: {0}")]
    Configuration(&'static str),
    #[error(transparent)]
    Embedding(#[from] EmbeddingError),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum SemanticFailurePolicy {
    #[default]
    ReturnError,
    LexicalOnly,
}

#[derive(Debug, Clone)]
pub struct HybridSearchOptions {
    #[cfg(feature = "execution")]
    control: Option<crate::execution::WorkControl>,
    limit: usize,
    candidates: usize,
    algorithm: SearchAlgorithm,
    bm25: Bm25Config,
    rrf_k: f64,
    lexical_weight: f64,
    semantic_weight: f64,
    min_score: f64,
    context: Option<Context>,
    failure_policy: SemanticFailurePolicy,
}

impl Default for HybridSearchOptions {
    fn default() -> Self {
        Self {
            #[cfg(feature = "execution")]
            control: None,
            limit: 10,
            candidates: 100,
            algorithm: SearchAlgorithm::Bm25,
            bm25: Bm25Config::default(),
            rrf_k: 60.0,
            lexical_weight: 0.5,
            semantic_weight: 0.5,
            min_score: -1.0,
            context: None,
            failure_policy: SemanticFailurePolicy::ReturnError,
        }
    }
}

impl HybridSearchOptions {
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
    fn checkpoint(&self) -> Result<(), HybridError> {
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
            candidates: limit.max(100),
            ..Self::default()
        }
    }

    pub fn with_candidates(mut self, candidates: usize) -> Result<Self, HybridError> {
        if candidates == 0 || candidates < self.limit {
            return Err(HybridError::Configuration(
                "candidate limit must be positive and at least the result limit",
            ));
        }
        self.candidates = candidates;
        Ok(self)
    }

    pub fn with_rrf_k(mut self, k: f64) -> Result<Self, HybridError> {
        if !k.is_finite() || k < 0.0 {
            return Err(HybridError::Configuration(
                "RRF smoothing must be finite and nonnegative",
            ));
        }
        self.rrf_k = k;
        Ok(self)
    }

    pub fn with_weights(mut self, lexical: f64, semantic: f64) -> Result<Self, HybridError> {
        if !lexical.is_finite()
            || !semantic.is_finite()
            || lexical < 0.0
            || semantic < 0.0
            || lexical.max(semantic) == 0.0
        {
            return Err(HybridError::Configuration(
                "weights must be finite, nonnegative, and not both zero",
            ));
        }
        let scale = lexical.max(semantic);
        let a = lexical / scale;
        let b = semantic / scale;
        self.lexical_weight = a / (a + b);
        self.semantic_weight = b / (a + b);
        Ok(self)
    }

    pub fn with_lexical_algorithm(mut self, algorithm: SearchAlgorithm) -> Self {
        self.algorithm = algorithm;
        self
    }

    pub fn with_bm25(mut self, config: Bm25Config) -> Self {
        self.bm25 = config;
        self
    }

    pub fn with_min_score(mut self, score: f64) -> Result<Self, HybridError> {
        SemanticSearchOptions::new(self.limit).with_min_score(score)?;
        self.min_score = score;
        Ok(self)
    }

    pub fn with_context(mut self, context: Context) -> Self {
        self.context = Some(context);
        self
    }

    pub fn with_failure_policy(mut self, policy: SemanticFailurePolicy) -> Self {
        self.failure_policy = policy;
        self
    }

    pub fn limit(&self) -> usize {
        self.limit
    }
    pub fn candidates(&self) -> usize {
        self.candidates
    }
    pub fn rrf_k(&self) -> f64 {
        self.rrf_k
    }
    pub fn lexical_weight(&self) -> f64 {
        self.lexical_weight
    }
    pub fn semantic_weight(&self) -> f64 {
        self.semantic_weight
    }
    pub fn lexical_algorithm(&self) -> SearchAlgorithm {
        self.algorithm
    }
    pub fn bm25(&self) -> Bm25Config {
        self.bm25
    }
    pub fn min_score(&self) -> f64 {
        self.min_score
    }
    pub fn context(&self) -> Option<&Context> {
        self.context.as_ref()
    }
    pub fn failure_policy(&self) -> SemanticFailurePolicy {
        self.failure_policy
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[non_exhaustive]
pub struct HybridResult<'a> {
    pub entry: &'a KnowledgeEntry,
    pub score: f64,
    pub lexical_rank: Option<usize>,
    pub semantic_rank: Option<usize>,
    pub lexical_score: Option<f64>,
    pub semantic_score: Option<f64>,
    pub matched_terms: Vec<String>,
}

impl Ranked for HybridResult<'_> {
    fn score(&self) -> f64 {
        self.score
    }
    fn id(&self) -> &str {
        &self.entry.id
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct HybridOutcome<'a> {
    results: Vec<HybridResult<'a>>,
    semantic_used: bool,
    fallback_reason: Option<String>,
}

impl<'a> HybridOutcome<'a> {
    pub fn results(&self) -> &[HybridResult<'a>] {
        &self.results
    }
    pub fn into_results(self) -> Vec<HybridResult<'a>> {
        self.results
    }
    pub fn semantic_used(&self) -> bool {
        self.semantic_used
    }
    pub fn fallback_reason(&self) -> Option<&str> {
        self.fallback_reason.as_deref()
    }
}

impl SemanticIndex {
    pub fn hybrid_search(
        &self,
        query: &str,
        provider: &mut (impl EmbeddingProvider + ?Sized),
        options: &HybridSearchOptions,
    ) -> Result<HybridOutcome<'_>, HybridError> {
        options.checkpoint()?;
        if query.trim().is_empty() {
            return Err(EmbeddingError::EmptyInput.into());
        }
        if options.limit == 0 || self.is_empty() {
            return Ok(HybridOutcome {
                results: Vec::new(),
                semantic_used: false,
                fallback_reason: None,
            });
        }
        let mut semantic_used = false;
        let mut fallback_reason = None;
        let semantic = if options.semantic_weight > 0.0 {
            let mut semantic_options =
                SemanticSearchOptions::new(options.candidates).with_min_score(options.min_score)?;
            if let Some(context) = &options.context {
                semantic_options = semantic_options.with_context(context.clone());
            }
            #[cfg(feature = "execution")]
            if let Some(control) = options.control() {
                semantic_options = semantic_options.with_control(control.clone());
            }
            match self.search(query, provider, &semantic_options) {
                Ok(results) => {
                    semantic_used = true;
                    results
                }
                Err(error @ EmbeddingError::Provider(_))
                    if options.failure_policy == SemanticFailurePolicy::LexicalOnly =>
                {
                    fallback_reason = Some(error.to_string());
                    Vec::new()
                }
                Err(error) => return Err(error.into()),
            }
        } else {
            Vec::new()
        };
        let lexical_weight = if fallback_reason.is_some() {
            1.0
        } else {
            options.lexical_weight
        };
        let lexical = if lexical_weight > 0.0 {
            let search = SearchOptions {
                limit: options.candidates,
                algorithm: options.algorithm,
                bm25: options.bm25,
                context: options.context.clone(),
            };
            #[cfg(feature = "execution")]
            let hits = if let Some(control) = options.control() {
                self.lexical()
                    .search_controlled(query, &search, control)
                    .map_err(EmbeddingError::from)?
            } else {
                self.lexical().search_with_options(query, &search)
            };
            #[cfg(not(feature = "execution"))]
            let hits = self.lexical().search_with_options(query, &search);
            hits
        } else {
            Vec::new()
        };
        options.checkpoint()?;
        #[cfg(feature = "execution")]
        let contribution_count = lexical.len() + semantic.len();
        #[cfg(feature = "execution")]
        let mut contributions = 0usize;
        #[cfg(feature = "execution")]
        options.progress(
            crate::execution::WorkPhase::Fusing,
            0,
            Some(contribution_count),
        );
        let mut fused: BTreeMap<&str, HybridResult<'_>> = BTreeMap::new();
        for (rank, hit) in lexical.into_iter().enumerate() {
            #[cfg(feature = "execution")]
            {
                contributions += 1;
                if contributions.is_multiple_of(512) {
                    options.progress(
                        crate::execution::WorkPhase::Fusing,
                        contributions,
                        Some(contribution_count),
                    );
                }
            }
            options.checkpoint()?;
            let rank = rank + 1;
            fused.insert(
                &hit.entry.id,
                HybridResult {
                    entry: hit.entry,
                    score: lexical_weight / (options.rrf_k + rank as f64),
                    lexical_rank: Some(rank),
                    semantic_rank: None,
                    lexical_score: Some(hit.score),
                    semantic_score: None,
                    matched_terms: hit.matched_terms,
                },
            );
        }
        for (rank, hit) in semantic.into_iter().enumerate() {
            #[cfg(feature = "execution")]
            {
                contributions += 1;
                if contributions.is_multiple_of(512) {
                    options.progress(
                        crate::execution::WorkPhase::Fusing,
                        contributions,
                        Some(contribution_count),
                    );
                }
            }
            options.checkpoint()?;
            let rank = rank + 1;
            let result = fused.entry(&hit.entry.id).or_insert_with(|| HybridResult {
                entry: hit.entry,
                score: 0.0,
                lexical_rank: None,
                semantic_rank: None,
                lexical_score: None,
                semantic_score: None,
                matched_terms: Vec::new(),
            });
            result.semantic_rank = Some(rank);
            result.semantic_score = Some(hit.score);
            result.score += options.semantic_weight / (options.rrf_k + rank as f64);
        }
        #[cfg(feature = "execution")]
        options.progress(
            crate::execution::WorkPhase::Fusing,
            contribution_count,
            Some(contribution_count),
        );
        let mut results = TopK::new(options.limit);
        for hit in fused.into_values() {
            options.checkpoint()?;
            if hit.score > 0.0 && hit.score.is_finite() {
                results.push(hit);
            }
        }
        options.checkpoint()?;
        let results = results.finish();
        options.checkpoint()?;
        Ok(HybridOutcome {
            results,
            semantic_used,
            fallback_reason,
        })
    }
}
