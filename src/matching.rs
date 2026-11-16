//! Explainable lexical comparison. Scores do not measure semantic equivalence.

use crate::{knowledge::KnowledgeEntry, text};
use serde::Serialize;
use thiserror::Error;

pub use strsim::levenshtein as levenshtein_distance;

/// The lexical evidence for comparing two nonempty texts.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Similarity {
    pub exact: bool,
    /// Normalized Levenshtein similarity, using Unicode scalar values, in 0..=1.
    pub character: f64,
    /// Jaccard similarity of unique tokens, in 0..=1.
    pub token: f64,
}

/// Compare normalized text. Empty text has zero similarity, including to itself.
pub fn compare(a: &str, b: &str) -> Similarity {
    compare_normalized(&text::normalize(a), &text::normalize(b))
}

pub(crate) fn compare_normalized(a: &str, b: &str) -> Similarity {
    if a.is_empty() || b.is_empty() {
        return Similarity {
            exact: false,
            character: 0.0,
            token: 0.0,
        };
    }
    let max_len = a.chars().count().max(b.chars().count());
    let terms_a = text::unique_terms(a);
    let terms_b = text::unique_terms(b);
    let union = terms_a.union(&terms_b).count();
    Similarity {
        exact: a == b,
        character: 1.0 - levenshtein_distance(a, b) as f64 / max_len as f64,
        token: if union == 0 {
            0.0
        } else {
            terms_a.intersection(&terms_b).count() as f64 / union as f64
        },
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub enum MatchStrategy {
    Exact,
    #[default]
    Fuzzy,
    TokenOverlap,
}

#[derive(Debug, Clone)]
pub struct MatchOptions {
    pub strategy: MatchStrategy,
    pub min_similarity: f64,
    pub limit: usize,
}

impl Default for MatchOptions {
    fn default() -> Self {
        Self {
            strategy: MatchStrategy::Fuzzy,
            min_similarity: 0.8,
            limit: 10,
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("min_similarity must be finite and between 0 and 1")]
pub struct MatchError;

impl MatchOptions {
    pub(crate) fn validate(&self) -> Result<(), MatchError> {
        if !self.min_similarity.is_finite() || !(0.0..=1.0).contains(&self.min_similarity) {
            return Err(MatchError);
        }
        Ok(())
    }
}

/// A matching entry and the evidence used to rank it.
#[derive(Debug, Clone, Serialize)]
pub struct MatchResult<'a> {
    pub entry: &'a KnowledgeEntry,
    pub score: f64,
    pub similarity: Similarity,
}
