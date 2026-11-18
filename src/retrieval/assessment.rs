#![doc = include_str!("../../docs/ASSESSMENT.md")]

use super::{
    excerpts::analyzed_terms, ExcerptAnalyzer, ExcerptOptions, RankingKind, RetrievalError,
    RetrievalResponse,
};
use crate::text;
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct AssessmentLimits {
    query_bytes: usize,
    terms: usize,
    hits: usize,
    text_bytes: usize,
    normalized_bytes: usize,
    tokens: usize,
}
impl Default for AssessmentLimits {
    fn default() -> Self {
        Self {
            query_bytes: 16 * 1024,
            terms: 256,
            hits: 100,
            text_bytes: 64 * 1024,
            normalized_bytes: 256 * 1024,
            tokens: 8192,
        }
    }
}
impl AssessmentLimits {
    pub fn with_query_limits(mut self, bytes: usize, terms: usize) -> Result<Self, RetrievalError> {
        if bytes == 0 || terms == 0 {
            return Err(RetrievalError::Configuration(
                "assessment query limits must be positive",
            ));
        }
        self.query_bytes = bytes;
        self.terms = terms;
        Ok(self)
    }
    pub fn with_hit_limit(mut self, hits: usize) -> Result<Self, RetrievalError> {
        if hits == 0 {
            return Err(RetrievalError::Configuration(
                "assessment hit limit must be positive",
            ));
        }
        self.hits = hits;
        Ok(self)
    }
    pub fn with_text_limits(
        mut self,
        bytes: usize,
        normalized: usize,
        tokens: usize,
    ) -> Result<Self, RetrievalError> {
        if [bytes, normalized, tokens].contains(&0) {
            return Err(RetrievalError::Configuration(
                "assessment text limits must be positive",
            ));
        }
        self.text_bytes = bytes;
        self.normalized_bytes = normalized;
        self.tokens = tokens;
        Ok(self)
    }
    pub fn query_bytes(&self) -> usize {
        self.query_bytes
    }
    pub fn terms(&self) -> usize {
        self.terms
    }
    pub fn hits(&self) -> usize {
        self.hits
    }
    pub fn text_bytes(&self) -> usize {
        self.text_bytes
    }
    pub fn normalized_bytes(&self) -> usize {
        self.normalized_bytes
    }
    pub fn tokens(&self) -> usize {
        self.tokens
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ScoreScale {
    Retrieval(RankingKind),
    Rerank,
}
#[derive(Debug, Clone, Serialize)]
pub struct ScoreGate {
    scale: ScoreScale,
    recipe: String,
    minimum: f64,
}
impl ScoreGate {
    pub fn new(
        scale: ScoreScale,
        recipe: impl Into<String>,
        minimum: f64,
    ) -> Result<Self, RetrievalError> {
        let recipe = recipe.into();
        if recipe.trim().is_empty() || recipe.len() > 1024 || !minimum.is_finite() {
            return Err(RetrievalError::Configuration(
                "score gate needs a bounded recipe and finite threshold",
            ));
        }
        let valid = valid_score(scale, minimum);
        if !valid {
            return Err(RetrievalError::Configuration(
                "threshold is outside the selected score scale",
            ));
        }
        Ok(Self {
            scale,
            recipe,
            minimum,
        })
    }
    pub fn scale(&self) -> ScoreScale {
        self.scale
    }
    pub fn recipe(&self) -> &str {
        &self.recipe
    }
    pub fn minimum(&self) -> f64 {
        self.minimum
    }
}
fn valid_score(scale: ScoreScale, value: f64) -> bool {
    if !value.is_finite() {
        return false;
    }
    match scale {
        ScoreScale::Retrieval(RankingKind::Cosine) => (-1.0..=1.0).contains(&value),
        ScoreScale::Retrieval(RankingKind::Tfidf | RankingKind::TextTfidf) => {
            (0.0..=1.0).contains(&value)
        }
        ScoreScale::Retrieval(
            RankingKind::Bm25 | RankingKind::TextBm25 | RankingKind::ReciprocalRankFusion,
        ) => value >= 0.0,
        _ => true,
    }
}

#[derive(Debug, Clone)]
pub struct EvidencePolicy {
    label: String,
    coverage: Option<f64>,
    gate: Option<ScoreGate>,
    min_hits: usize,
    min_text_bytes: usize,
    ignore_stopwords: bool,
    allow_nonlexical_coverage: bool,
    limits: AssessmentLimits,
}
impl Default for EvidencePolicy {
    fn default() -> Self {
        Self {
            label: "lexical-coverage-v1".into(),
            coverage: Some(1.0),
            gate: None,
            min_hits: 1,
            min_text_bytes: 1,
            ignore_stopwords: true,
            allow_nonlexical_coverage: false,
            limits: AssessmentLimits::default(),
        }
    }
}
impl EvidencePolicy {
    pub fn lexical(minimum: f64) -> Result<Self, RetrievalError> {
        if !minimum.is_finite() || minimum <= 0.0 || minimum > 1.0 {
            return Err(RetrievalError::Configuration(
                "query coverage must be in (0,1]",
            ));
        }
        Ok(Self {
            coverage: Some(minimum),
            ..Self::default()
        })
    }
    pub fn scored(gate: ScoreGate) -> Self {
        Self {
            label: "score-gate-v1".into(),
            coverage: None,
            gate: Some(gate),
            ..Self::default()
        }
    }
    pub fn scored_and_lexical(gate: ScoreGate, minimum: f64) -> Result<Self, RetrievalError> {
        let mut p = Self::lexical(minimum)?;
        p.gate = Some(gate);
        p.label = "score-and-coverage-v1".into();
        p.allow_nonlexical_coverage = true;
        Ok(p)
    }
    pub fn with_label(mut self, label: impl Into<String>) -> Result<Self, RetrievalError> {
        let label = label.into();
        if label.trim().is_empty() || label.len() > 128 || label.chars().any(char::is_control) {
            return Err(RetrievalError::Configuration(
                "assessment label must be bounded and contain no controls",
            ));
        }
        self.label = label;
        Ok(self)
    }
    pub fn with_min_hits(mut self, count: usize) -> Result<Self, RetrievalError> {
        if count == 0 {
            return Err(RetrievalError::Configuration(
                "minimum qualified hits must be positive",
            ));
        }
        self.min_hits = count;
        Ok(self)
    }
    pub fn with_min_text_bytes(mut self, count: usize) -> Result<Self, RetrievalError> {
        if count == 0 {
            return Err(RetrievalError::Configuration(
                "minimum excerpt bytes must be positive",
            ));
        }
        self.min_text_bytes = count;
        Ok(self)
    }
    pub fn with_ignore_stopwords(mut self, enabled: bool) -> Self {
        self.ignore_stopwords = enabled;
        self
    }
    pub fn with_nonlexical_coverage(mut self, enabled: bool) -> Self {
        self.allow_nonlexical_coverage = enabled;
        self
    }
    pub fn with_limits(mut self, limits: AssessmentLimits) -> Self {
        self.limits = limits;
        self
    }
    pub fn label(&self) -> &str {
        &self.label
    }
    pub fn minimum_coverage(&self) -> Option<f64> {
        self.coverage
    }
    pub fn score_gate(&self) -> Option<&ScoreGate> {
        self.gate.as_ref()
    }
    pub fn minimum_hits(&self) -> usize {
        self.min_hits
    }
    pub fn minimum_text_bytes(&self) -> usize {
        self.min_text_bytes
    }
    pub fn limits(&self) -> &AssessmentLimits {
        &self.limits
    }
    pub fn ignores_stopwords(&self) -> bool {
        self.ignore_stopwords
    }
    pub fn allows_nonlexical_coverage(&self) -> bool {
        self.allow_nonlexical_coverage
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum EvidenceVerdict {
    MeetsPolicy,
    Insufficient,
    Undetermined,
    NotRequested,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum AssessmentReason {
    ZeroLimit,
    NoResults,
    NoDeliveredHits,
    OutputBudget,
    TooFewQualifiedHits,
    InsufficientText,
    LowTermCoverage,
    NoInformativeTerms,
    UncalibratedScoreScale,
    ScoreRecipeMismatch,
    ScoreUnavailable,
    BelowScoreThreshold,
    ProviderFallback,
}
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct HitAssessment {
    pub rank: usize,
    pub qualifies: Option<bool>,
    pub query_terms: usize,
    pub matched_terms: usize,
    pub coverage: Option<f64>,
    pub observed_score: Option<f64>,
    pub reasons: Vec<AssessmentReason>,
}
#[derive(Debug, Serialize)]
pub struct EvidenceAssessment {
    policy: String,
    verdict: EvidenceVerdict,
    reasons: Vec<AssessmentReason>,
    returned_hits: usize,
    qualified_hits: usize,
    undetermined_hits: usize,
    query_terms: usize,
    minimum_hits: usize,
    minimum_coverage: Option<f64>,
    score_gate: Option<ScoreGate>,
    observations: Vec<HitAssessment>,
}
impl EvidenceAssessment {
    pub fn policy(&self) -> &str {
        &self.policy
    }
    pub fn verdict(&self) -> EvidenceVerdict {
        self.verdict
    }
    pub fn reasons(&self) -> &[AssessmentReason] {
        &self.reasons
    }
    pub fn returned_hits(&self) -> usize {
        self.returned_hits
    }
    pub fn qualified_hits(&self) -> usize {
        self.qualified_hits
    }
    pub fn undetermined_hits(&self) -> usize {
        self.undetermined_hits
    }
    pub fn query_terms(&self) -> usize {
        self.query_terms
    }
    pub fn observations(&self) -> &[HitAssessment] {
        &self.observations
    }
    pub fn minimum_hits(&self) -> usize {
        self.minimum_hits
    }
    pub fn minimum_coverage(&self) -> Option<f64> {
        self.minimum_coverage
    }
    pub fn score_gate(&self) -> Option<&ScoreGate> {
        self.score_gate.as_ref()
    }
    pub fn qualified_ranks(&self) -> impl Iterator<Item = usize> + '_ {
        self.observations
            .iter()
            .filter(|h| h.qualifies == Some(true))
            .map(|h| h.rank)
    }
}
pub(super) struct AssessmentPlan {
    policy: EvidencePolicy,
    terms: BTreeSet<String>,
    analyzer: ExcerptAnalyzer,
    analysis: ExcerptOptions,
}
impl AssessmentPlan {
    pub(super) fn new(
        query: &str,
        analyzer: &ExcerptAnalyzer,
        policy: &EvidencePolicy,
        limit: usize,
    ) -> Result<Self, RetrievalError> {
        if limit > policy.limits.hits {
            return Err(RetrievalError::Budget("assessment requested hit count"));
        }
        let analysis = ExcerptOptions::default().with_analysis_limits(
            policy.limits.text_bytes,
            policy.limits.normalized_bytes,
            policy.limits.tokens,
        )?;
        let terms = if limit == 0 {
            BTreeSet::new()
        } else {
            if query.len() > policy.limits.query_bytes {
                return Err(RetrievalError::Budget("assessment query bytes"));
            }
            let terms: BTreeSet<_> = analyzed_terms(query, analyzer, &analysis)?
                .into_iter()
                .filter(|t| !policy.ignore_stopwords || !text::is_stopword(t))
                .collect();
            if terms.len() > policy.limits.terms {
                return Err(RetrievalError::Budget("assessment query terms"));
            }
            terms
        };
        Ok(Self {
            policy: policy.clone(),
            terms,
            analyzer: analyzer.clone(),
            analysis,
        })
    }
    pub(super) fn observe(
        &self,
        response: &RetrievalResponse,
    ) -> Result<Vec<HitAssessment>, RetrievalError> {
        if response.hits.len() > self.policy.limits.hits {
            return Err(RetrievalError::Budget("assessment hit count"));
        }
        let degraded = response.fallback_reason.is_some()
            || response
                .reranking
                .as_ref()
                .is_some_and(|r| r.fallback_reason.is_some());
        let mut result = Vec::new();
        for (i, hit) in response.hits.iter().enumerate() {
            if hit.excerpt.len() > self.policy.limits.text_bytes {
                return Err(RetrievalError::Budget("assessment excerpt bytes"));
            }
            let mut reasons = Vec::new();
            let mut failed = false;
            let mut unknown = false;
            if hit.excerpt.trim().is_empty() || hit.excerpt.len() < self.policy.min_text_bytes {
                failed = true;
                reasons.push(AssessmentReason::InsufficientText);
            }
            let mut matched = 0usize;
            let mut coverage = None;
            let mut score = None;
            if let Some(minimum) = self.policy.coverage {
                let lexical = matches!(
                    hit.evidence.kind,
                    RankingKind::Bm25
                        | RankingKind::Tfidf
                        | RankingKind::TextBm25
                        | RankingKind::TextTfidf
                );
                if !lexical && !self.policy.allow_nonlexical_coverage {
                    unknown = true;
                    reasons.push(AssessmentReason::UncalibratedScoreScale);
                } else if self.terms.is_empty() {
                    unknown = true;
                    reasons.push(AssessmentReason::NoInformativeTerms);
                } else {
                    let words: BTreeSet<_> =
                        analyzed_terms(&hit.excerpt, &self.analyzer, &self.analysis)?
                            .into_iter()
                            .collect();
                    matched = self.terms.intersection(&words).count();
                    let value = matched as f64 / self.terms.len() as f64;
                    coverage = Some(value);
                    if value < minimum {
                        failed = true;
                        reasons.push(AssessmentReason::LowTermCoverage);
                    }
                }
            }
            if let Some(gate) = &self.policy.gate {
                let observed = match gate.scale {
                    ScoreScale::Retrieval(kind) => {
                        if hit.evidence.kind != kind {
                            unknown = true;
                            reasons.push(AssessmentReason::UncalibratedScoreScale);
                            None
                        } else if response.recipe != gate.recipe {
                            unknown = true;
                            reasons.push(AssessmentReason::ScoreRecipeMismatch);
                            None
                        } else {
                            Some(hit.evidence.score)
                        }
                    }
                    ScoreScale::Rerank => match &response.reranking {
                        Some(r) if r.used && r.recipe.as_deref() == Some(gate.recipe.as_str()) => {
                            hit.rerank_score
                        }
                        Some(r) if r.used => {
                            unknown = true;
                            reasons.push(AssessmentReason::ScoreRecipeMismatch);
                            None
                        }
                        _ => {
                            unknown = true;
                            reasons.push(AssessmentReason::ScoreUnavailable);
                            None
                        }
                    },
                };
                if let Some(value) = observed {
                    if !valid_score(gate.scale, value) {
                        return Err(RetrievalError::Output(
                            "assessment score is outside the declared scale",
                        ));
                    }
                    score = Some(value);
                    if value < gate.minimum {
                        failed = true;
                        reasons.push(AssessmentReason::BelowScoreThreshold);
                    }
                } else {
                    unknown = true;
                    if !reasons.contains(&AssessmentReason::ScoreUnavailable) {
                        reasons.push(AssessmentReason::ScoreUnavailable);
                    }
                }
            }
            let qualifies = if degraded {
                reasons.push(AssessmentReason::ProviderFallback);
                None
            } else if failed {
                Some(false)
            } else if unknown {
                None
            } else {
                Some(true)
            };
            result.push(HitAssessment {
                rank: i + 1,
                qualifies,
                query_terms: self.terms.len(),
                matched_terms: matched,
                coverage,
                observed_score: score,
                reasons,
            });
        }
        Ok(result)
    }
    pub(super) fn summarize(
        &self,
        response: &RetrievalResponse,
        observed: &[HitAssessment],
    ) -> EvidenceAssessment {
        let observations: Vec<_> = observed.iter().take(response.hits.len()).cloned().collect();
        let qualified = observations
            .iter()
            .filter(|o| o.qualifies == Some(true))
            .count();
        let unknown = observations
            .iter()
            .filter(|o| o.qualifies.is_none())
            .count();
        let mut reasons: BTreeSet<_> = observations
            .iter()
            .flat_map(|o| o.reasons.iter().copied())
            .collect();
        let degraded = response.fallback_reason.is_some()
            || response
                .reranking
                .as_ref()
                .is_some_and(|r| r.fallback_reason.is_some());
        let verdict = if response.requested_limit == 0 {
            reasons.insert(AssessmentReason::ZeroLimit);
            EvidenceVerdict::NotRequested
        } else if degraded {
            reasons.insert(AssessmentReason::ProviderFallback);
            EvidenceVerdict::Undetermined
        } else if response.hits.is_empty() {
            reasons.insert(if response.received_candidates == 0 {
                AssessmentReason::NoResults
            } else {
                AssessmentReason::NoDeliveredHits
            });
            EvidenceVerdict::Insufficient
        } else if qualified >= self.policy.min_hits {
            EvidenceVerdict::MeetsPolicy
        } else if qualified.saturating_add(unknown) >= self.policy.min_hits {
            EvidenceVerdict::Undetermined
        } else {
            reasons.insert(AssessmentReason::TooFewQualifiedHits);
            EvidenceVerdict::Insufficient
        };
        if response.hits.len() < response.before_output_budget {
            reasons.insert(AssessmentReason::OutputBudget);
        }
        EvidenceAssessment {
            policy: self.policy.label.clone(),
            verdict,
            reasons: reasons.into_iter().collect(),
            returned_hits: response.hits.len(),
            qualified_hits: qualified,
            undetermined_hits: unknown,
            query_terms: self.terms.len(),
            minimum_hits: self.policy.min_hits,
            minimum_coverage: self.policy.coverage,
            score_gate: self.policy.gate.clone(),
            observations,
        }
    }
}
pub fn assess_response(
    query: &str,
    response: &RetrievalResponse,
    analyzer: &ExcerptAnalyzer,
    policy: &EvidencePolicy,
) -> Result<EvidenceAssessment, RetrievalError> {
    if query.trim().is_empty() {
        return Err(RetrievalError::Configuration("assessment query is blank"));
    }
    let plan = AssessmentPlan::new(query, analyzer, policy, response.requested_limit)?;
    let observations = plan.observe(response)?;
    Ok(plan.summarize(response, &observations))
}
