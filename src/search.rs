//! Lexical ranking shared by the core and the legacy BM25 agent.

use crate::{context::Context, knowledge::KnowledgeEntry};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SearchAlgorithm {
    #[default]
    Bm25,
    Tfidf,
}

/// Validated Okapi BM25 parameters. k1 must be positive; b is in 0..=1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bm25Config {
    k1: f64,
    b: f64,
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("BM25 requires finite k1 > 0 and finite b between 0 and 1")]
pub struct SearchError;

impl Default for Bm25Config {
    fn default() -> Self {
        Self { k1: 1.2, b: 0.75 }
    }
}

impl Bm25Config {
    pub fn new(k1: f64, b: f64) -> Result<Self, SearchError> {
        if !k1.is_finite() || k1 <= 0.0 || !b.is_finite() || !(0.0..=1.0).contains(&b) {
            return Err(SearchError);
        }
        Ok(Self { k1, b })
    }

    pub fn k1(self) -> f64 {
        self.k1
    }
    pub fn b(self) -> f64 {
        self.b
    }

    /// Score a single term. Empty documents and missing terms contribute zero.
    pub(crate) fn term_score(
        self,
        tf: usize,
        df: usize,
        count: usize,
        length: usize,
        average: f64,
    ) -> f64 {
        if tf == 0 || count == 0 || average <= 0.0 {
            return 0.0;
        }
        let idf = (1.0 + (count as f64 - df as f64 + 0.5) / (df as f64 + 0.5)).ln();
        let tf = tf as f64;
        // Divide first so even very large, valid k1 values do not overflow.
        let norm = 1.0 - self.b + self.b * length as f64 / average;
        if self.k1 >= 1.0 {
            idf * tf * (1.0 + 1.0 / self.k1) / (norm + tf / self.k1)
        } else {
            idf * tf * (self.k1 + 1.0) / (tf + self.k1 * norm)
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub limit: usize,
    pub algorithm: SearchAlgorithm,
    pub bm25: Bm25Config,
    /// Restrict eligible entries. Corpus statistics still refer to the full index.
    pub context: Option<Context>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            limit: 10,
            algorithm: SearchAlgorithm::default(),
            bm25: Bm25Config::default(),
            context: None,
        }
    }
}

/// A positive relevance score, its source entry, and the shared query terms.
/// BM25 scores are unbounded relevance values, not probabilities.
#[derive(Debug, Clone, Serialize)]
pub struct SearchResult<'a> {
    pub entry: &'a KnowledgeEntry,
    pub score: f64,
    pub matched_terms: Vec<String>,
}

pub(crate) fn tfidf_idf(count: usize, df: usize) -> f64 {
    ((count as f64 + 1.0) / (df as f64 + 1.0)).ln() + 1.0
}
