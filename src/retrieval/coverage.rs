#![doc = include_str!("../../docs/LEXICAL-COVERAGE.md")]

use super::{
    scope, LexicalQueryAnalysis, LexicalQueryOptions, RankingEvidence, RankingKind, RetrievalBatch,
    RetrievalCandidate, RetrievalError, RetrievalOptions, RetrievalSource,
};
use crate::{Engine, KnowledgeEntry, SearchOptions};
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct LexicalCoverageOptions {
    query: LexicalQueryOptions,
    weight: f64,
}
impl Default for LexicalCoverageOptions {
    fn default() -> Self {
        Self {
            query: LexicalQueryOptions::default(),
            weight: 0.5,
        }
    }
}
impl LexicalCoverageOptions {
    pub fn with_query_options(mut self, options: LexicalQueryOptions) -> Self {
        self.query = options;
        self
    }
    pub fn with_coverage_weight(mut self, weight: f64) -> Result<Self, RetrievalError> {
        if !weight.is_finite() || !(0.0..=1.0).contains(&weight) {
            return Err(RetrievalError::Configuration(
                "coverage weight must be finite and between zero and one",
            ));
        }
        self.weight = weight;
        Ok(self)
    }
    pub fn query_options(&self) -> &LexicalQueryOptions {
        &self.query
    }
    pub fn coverage_weight(&self) -> f64 {
        self.weight
    }
    pub fn recipe(&self) -> String {
        format!(
            "airust-lexical-coverage-v1:{}:{}",
            self.weight,
            self.query.recipe()
        )
    }
}

#[derive(Debug)]
pub struct LexicalCoverageHit<'a> {
    entry: &'a KnowledgeEntry,
    score: f64,
    lexical_rank: usize,
    lexical_score: f64,
    normalized_score: f64,
    matched_query_terms: usize,
    query_terms: usize,
    matched_terms: Vec<String>,
}
impl<'a> LexicalCoverageHit<'a> {
    pub fn entry(&self) -> &'a KnowledgeEntry {
        self.entry
    }
    pub fn score(&self) -> f64 {
        self.score
    }
    pub fn lexical_rank(&self) -> usize {
        self.lexical_rank
    }
    pub fn lexical_score(&self) -> f64 {
        self.lexical_score
    }
    pub fn normalized_score(&self) -> f64 {
        self.normalized_score
    }
    pub fn matched_query_terms(&self) -> usize {
        self.matched_query_terms
    }
    pub fn query_terms(&self) -> usize {
        self.query_terms
    }
    pub fn query_coverage(&self) -> f64 {
        self.matched_query_terms as f64 / self.query_terms.max(1) as f64
    }
    pub fn matched_terms(&self) -> &[String] {
        &self.matched_terms
    }
}

#[derive(Debug)]
pub struct LexicalCoverageBatch<'a> {
    hits: Vec<LexicalCoverageHit<'a>>,
    analysis: Option<LexicalQueryAnalysis>,
    recipe: String,
}
impl<'a> LexicalCoverageBatch<'a> {
    pub fn hits(&self) -> &[LexicalCoverageHit<'a>] {
        &self.hits
    }
    pub fn analysis(&self) -> Option<&LexicalQueryAnalysis> {
        self.analysis.as_ref()
    }
    pub fn recipe(&self) -> &str {
        &self.recipe
    }
}

pub struct LexicalCoverageSource<'a> {
    index: &'a Engine,
    search: SearchOptions,
    coverage: LexicalCoverageOptions,
}
impl<'a> LexicalCoverageSource<'a> {
    pub fn new(index: &'a Engine) -> Self {
        Self {
            index,
            search: SearchOptions::default(),
            coverage: LexicalCoverageOptions::default(),
        }
    }
    pub fn with_options(mut self, options: SearchOptions) -> Self {
        self.search = options;
        self
    }
    pub fn with_coverage_options(mut self, options: LexicalCoverageOptions) -> Self {
        self.coverage = options;
        self
    }
    pub fn inspect(
        &self,
        query: &str,
        request: &RetrievalOptions,
    ) -> Result<LexicalCoverageBatch<'_>, RetrievalError> {
        request.checkpoint()?;
        if query.trim().is_empty() {
            return Err(RetrievalError::Configuration("lexical query is blank"));
        }
        if query.len() > request.max_query_bytes() {
            return Err(RetrievalError::Budget("query bytes"));
        }
        let recipe = format!(
            "{}:{:?}:{}:{}",
            self.coverage.recipe(),
            self.search.algorithm,
            self.search.bm25.k1(),
            self.search.bm25.b()
        );
        let Some(context) = scope(self.search.context.as_ref(), request.context()) else {
            return Ok(LexicalCoverageBatch {
                hits: Vec::new(),
                analysis: None,
                recipe,
            });
        };
        let analysis = self.coverage.query.analyze(query)?;
        request.checkpoint()?;
        let options = SearchOptions {
            limit: request.candidates(),
            context: Some(context),
            ..self.search.clone()
        };
        #[cfg(feature = "execution")]
        let lexical = if let Some(control) = request.control() {
            self.index
                .search_controlled(analysis.effective_query(), &options, control)?
        } else {
            self.index
                .search_with_options(analysis.effective_query(), &options)
        };
        #[cfg(not(feature = "execution"))]
        let lexical = self
            .index
            .search_with_options(analysis.effective_query(), &options);
        request.checkpoint()?;
        let terms: BTreeSet<_> = analysis.effective_query().split_whitespace().collect();
        let maximum = lexical.first().map_or(0.0, |h| h.score);
        let mut hits = Vec::with_capacity(lexical.len());
        for (offset, hit) in lexical.into_iter().enumerate() {
            request.checkpoint()?;
            if !hit.score.is_finite()
                || hit.score < 0.0
                || !maximum.is_finite()
                || hit.score > maximum
            {
                return Err(RetrievalError::Output(
                    "invalid lexical coverage component score",
                ));
            }
            let matched: BTreeSet<_> = hit
                .matched_terms
                .iter()
                .filter(|t| terms.contains(t.as_str()))
                .collect();
            let matched_query_terms = matched.len();
            let normalized_score = if maximum > 0.0 {
                hit.score / maximum
            } else {
                0.0
            };
            let fraction = matched_query_terms as f64 / terms.len().max(1) as f64;
            let score =
                (1.0 - self.coverage.weight) * normalized_score + self.coverage.weight * fraction;
            hits.push(LexicalCoverageHit {
                entry: hit.entry,
                score,
                lexical_rank: offset + 1,
                lexical_score: hit.score,
                normalized_score,
                matched_query_terms,
                query_terms: terms.len(),
                matched_terms: hit.matched_terms,
            });
        }
        hits.sort_by(|a, b| {
            b.score
                .total_cmp(&a.score)
                .then_with(|| a.entry.id.cmp(&b.entry.id))
        });
        hits.truncate(request.candidates());
        request.checkpoint()?;
        Ok(LexicalCoverageBatch {
            hits,
            analysis: Some(analysis),
            recipe,
        })
    }
}
impl RetrievalSource for LexicalCoverageSource<'_> {
    fn retrieve<'a>(
        &'a mut self,
        query: &str,
        request: &RetrievalOptions,
    ) -> Result<RetrievalBatch<'a>, RetrievalError> {
        let batch = self.inspect(query, request)?;
        Ok(RetrievalBatch::new(
            batch.recipe,
            batch
                .hits
                .into_iter()
                .map(|hit| {
                    let mut evidence = RankingEvidence::new(RankingKind::Custom, hit.score);
                    evidence.matched_terms = hit.matched_terms;
                    evidence.lexical_rank = Some(hit.lexical_rank);
                    evidence.lexical_score = Some(hit.lexical_score);
                    RetrievalCandidate {
                        entry: hit.entry,
                        evidence,
                    }
                })
                .collect(),
        ))
    }
}
