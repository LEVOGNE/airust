#![doc = include_str!("../docs/QUALITY.md")]

use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum QualityError {
    #[error("judgment and ranked IDs must be nonempty")]
    EmptyId,
    #[error("duplicate judgment or ranked ID: {0}")]
    DuplicateId(String),
    #[error("relevance grade must be between 0 and 31, received {0}")]
    InvalidGrade(u8),
    #[error("ranking cutoff must be positive")]
    Cutoff,
    #[error("ranked ID has no relevance judgment: {0}")]
    Unjudged(String),
    #[error("cannot summarize an empty query set")]
    NoQueries,
    #[error("summary queries must use the same cutoff and unjudged policy")]
    MixedConfiguration,
    #[error("summary result count overflow")]
    CountOverflow,
}
#[derive(Debug, Clone, Serialize)]
pub struct Judgments {
    grades: BTreeMap<String, u8>,
    relevant: usize,
}
impl Judgments {
    pub fn new<I, S>(grades: I) -> Result<Self, QualityError>
    where
        I: IntoIterator<Item = (S, u8)>,
        S: Into<String>,
    {
        let mut result = Self {
            grades: BTreeMap::new(),
            relevant: 0,
        };
        for (id, grade) in grades {
            let id = id.into();
            if id.trim().is_empty() {
                return Err(QualityError::EmptyId);
            }
            if grade > 31 {
                return Err(QualityError::InvalidGrade(grade));
            }
            if result.grades.insert(id.clone(), grade).is_some() {
                return Err(QualityError::DuplicateId(id));
            }
            result.relevant += usize::from(grade > 0);
        }
        Ok(result)
    }
    pub fn grade(&self, id: &str) -> Option<u8> {
        self.grades.get(id).copied()
    }
    pub fn len(&self) -> usize {
        self.grades.len()
    }
    pub fn is_empty(&self) -> bool {
        self.grades.is_empty()
    }
    pub fn relevant_count(&self) -> usize {
        self.relevant
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub enum UnjudgedPolicy {
    NonRelevant,
    RequireJudged,
}
#[derive(Debug, Clone, Serialize)]
pub struct QueryMetrics {
    cutoff: usize,
    unjudged_policy: UnjudgedPolicy,
    returned: usize,
    relevant_retrieved: usize,
    judged_retrieved: usize,
    unjudged_retrieved: usize,
    precision_at_k: f64,
    recall_at_k: Option<f64>,
    reciprocal_rank_at_k: f64,
    ndcg_at_k: Option<f64>,
}
impl QueryMetrics {
    pub fn cutoff(&self) -> usize {
        self.cutoff
    }
    pub fn unjudged_policy(&self) -> UnjudgedPolicy {
        self.unjudged_policy
    }
    pub fn returned(&self) -> usize {
        self.returned
    }
    pub fn relevant_retrieved(&self) -> usize {
        self.relevant_retrieved
    }
    pub fn judged_retrieved(&self) -> usize {
        self.judged_retrieved
    }
    pub fn unjudged_retrieved(&self) -> usize {
        self.unjudged_retrieved
    }
    pub fn precision_at_k(&self) -> f64 {
        self.precision_at_k
    }
    pub fn recall_at_k(&self) -> Option<f64> {
        self.recall_at_k
    }
    pub fn reciprocal_rank_at_k(&self) -> f64 {
        self.reciprocal_rank_at_k
    }
    pub fn ndcg_at_k(&self) -> Option<f64> {
        self.ndcg_at_k
    }
}
fn gain(grade: u8) -> f64 {
    2f64.powi(i32::from(grade)) - 1.0
}
fn discount(rank: usize) -> f64 {
    (rank as f64 + 1.0).log2()
}
pub fn evaluate_ranking<S: AsRef<str>>(
    ranking: &[S],
    judgments: &Judgments,
    cutoff: usize,
    policy: UnjudgedPolicy,
) -> Result<QueryMetrics, QualityError> {
    if cutoff == 0 {
        return Err(QualityError::Cutoff);
    }
    let mut seen = BTreeSet::new();
    let mut relevant = 0;
    let mut judged = 0;
    let mut reciprocal = 0.0;
    let mut dcg = 0.0;
    for (position, id) in ranking.iter().take(cutoff).enumerate() {
        let id = id.as_ref();
        if id.trim().is_empty() {
            return Err(QualityError::EmptyId);
        }
        if !seen.insert(id) {
            return Err(QualityError::DuplicateId(id.to_owned()));
        }
        let grade = match judgments.grade(id) {
            Some(grade) => {
                judged += 1;
                grade
            }
            None => match policy {
                UnjudgedPolicy::NonRelevant => 0,
                UnjudgedPolicy::RequireJudged => return Err(QualityError::Unjudged(id.to_owned())),
            },
        };
        if grade > 0 {
            relevant += 1;
            if reciprocal == 0.0 {
                reciprocal = 1.0 / (position + 1) as f64;
            }
        }
        dcg += gain(grade) / discount(position + 1);
    }
    let mut ideal: Vec<_> = judgments
        .grades
        .values()
        .copied()
        .filter(|&g| g > 0)
        .collect();
    ideal.sort_unstable_by(|a, b| b.cmp(a));
    let ideal_dcg: f64 = ideal
        .into_iter()
        .take(cutoff)
        .enumerate()
        .map(|(n, g)| gain(g) / discount(n + 1))
        .sum();
    let returned = ranking.len().min(cutoff);
    Ok(QueryMetrics {
        cutoff,
        unjudged_policy: policy,
        returned,
        relevant_retrieved: relevant,
        judged_retrieved: judged,
        unjudged_retrieved: returned - judged,
        precision_at_k: relevant as f64 / cutoff as f64,
        recall_at_k: (judgments.relevant > 0).then(|| relevant as f64 / judgments.relevant as f64),
        reciprocal_rank_at_k: reciprocal,
        ndcg_at_k: (ideal_dcg > 0.0).then(|| (dcg / ideal_dcg).clamp(0.0, 1.0)),
    })
}
#[derive(Debug, Clone, Serialize)]
pub struct QualitySummary {
    queries: usize,
    cutoff: usize,
    unjudged_policy: UnjudgedPolicy,
    recall_queries: usize,
    ndcg_queries: usize,
    unjudged_retrieved: usize,
    mean_precision_at_k: f64,
    mean_recall_at_k: Option<f64>,
    mean_reciprocal_rank_at_k: f64,
    mean_ndcg_at_k: Option<f64>,
}
impl QualitySummary {
    pub fn queries(&self) -> usize {
        self.queries
    }
    pub fn cutoff(&self) -> usize {
        self.cutoff
    }
    pub fn unjudged_policy(&self) -> UnjudgedPolicy {
        self.unjudged_policy
    }
    pub fn recall_queries(&self) -> usize {
        self.recall_queries
    }
    pub fn ndcg_queries(&self) -> usize {
        self.ndcg_queries
    }
    pub fn unjudged_retrieved(&self) -> usize {
        self.unjudged_retrieved
    }
    pub fn mean_precision_at_k(&self) -> f64 {
        self.mean_precision_at_k
    }
    pub fn mean_recall_at_k(&self) -> Option<f64> {
        self.mean_recall_at_k
    }
    pub fn mean_reciprocal_rank_at_k(&self) -> f64 {
        self.mean_reciprocal_rank_at_k
    }
    pub fn mean_ndcg_at_k(&self) -> Option<f64> {
        self.mean_ndcg_at_k
    }
}
pub fn summarize(metrics: &[QueryMetrics]) -> Result<QualitySummary, QualityError> {
    let first = metrics.first().ok_or(QualityError::NoQueries)?;
    if metrics
        .iter()
        .any(|m| m.cutoff != first.cutoff || m.unjudged_policy != first.unjudged_policy)
    {
        return Err(QualityError::MixedConfiguration);
    }
    let queries = metrics.len();
    let recall_queries = metrics.iter().filter(|m| m.recall_at_k.is_some()).count();
    let ndcg_queries = metrics.iter().filter(|m| m.ndcg_at_k.is_some()).count();
    let unjudged_retrieved = metrics.iter().try_fold(0usize, |total, metric| {
        total
            .checked_add(metric.unjudged_retrieved)
            .ok_or(QualityError::CountOverflow)
    })?;
    Ok(QualitySummary {
        queries,
        cutoff: first.cutoff,
        unjudged_policy: first.unjudged_policy,
        recall_queries,
        ndcg_queries,
        unjudged_retrieved,
        mean_precision_at_k: metrics.iter().map(|m| m.precision_at_k).sum::<f64>() / queries as f64,
        mean_recall_at_k: (recall_queries > 0).then(|| {
            metrics.iter().filter_map(|m| m.recall_at_k).sum::<f64>() / recall_queries as f64
        }),
        mean_reciprocal_rank_at_k: metrics.iter().map(|m| m.reciprocal_rank_at_k).sum::<f64>()
            / queries as f64,
        mean_ndcg_at_k: (ndcg_queries > 0)
            .then(|| metrics.iter().filter_map(|m| m.ndcg_at_k).sum::<f64>() / ndcg_queries as f64),
    })
}
