#![doc = include_str!("../docs/QUALITY-GATES.md")]
use crate::quality::{
    evaluate_ranking, summarize, Judgments, QualityError, QualitySummary, QueryMetrics,
    UnjudgedPolicy,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, time::Duration};
use thiserror::Error;
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum GateError {
    #[error("invalid quality-gate input: {0}")]
    Invalid(&'static str),
    #[error("quality runs are not comparable: {0}")]
    Incomparable(&'static str),
    #[error(transparent)]
    Quality(#[from] QualityError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
#[derive(Debug, Clone, Serialize)]
pub struct CaseMeasurement {
    id: String,
    identity: String,
    metrics: QueryMetrics,
    ranked_ids: Vec<String>,
    latency_ms: Option<f64>,
}
impl CaseMeasurement {
    pub fn evaluate(
        id: impl Into<String>,
        query: &str,
        ranking: &[impl AsRef<str>],
        judgments: &Judgments,
        cutoff: usize,
        policy: UnjudgedPolicy,
        latency: Option<Duration>,
    ) -> Result<Self, GateError> {
        let id = id.into();
        if !label(&id)
            || query.trim().is_empty()
            || query.len() > 16384
            || !(1..=1000).contains(&cutoff)
            || judgments.len() > 100000
        {
            return Err(GateError::Invalid("case identity/query/cutoff/judgments"));
        }
        let metrics = evaluate_ranking(ranking, judgments, cutoff, policy)?;
        let mut encoded = Vec::new();
        for (id, grade) in judgments.iter() {
            if id.len() > 4096 || encoded.len() + id.len() + 9 > 16 * 1024 * 1024 {
                return Err(GateError::Invalid("judgment bytes"));
            }
            encoded.extend_from_slice(&(id.len() as u64).to_le_bytes());
            encoded.extend_from_slice(id.as_bytes());
            encoded.push(grade);
        }
        if ranking
            .iter()
            .take(cutoff)
            .any(|id| id.as_ref().len() > 4096)
        {
            return Err(GateError::Invalid("ranked ID bytes"));
        }
        let identity = digest(&[
            b"airust-eval-case-v1",
            query.as_bytes(),
            &encoded,
            &serde_json::to_vec(&(cutoff, policy))?,
        ]);
        let latency_ms = latency.map(|v| v.as_secs_f64() * 1000.);
        if latency_ms.is_some_and(|v| !v.is_finite()) {
            return Err(GateError::Invalid("latency"));
        }
        Ok(Self {
            id,
            identity,
            ranked_ids: ranking
                .iter()
                .take(cutoff)
                .map(|id| id.as_ref().to_owned())
                .collect(),
            metrics,
            latency_ms,
        })
    }
    pub fn ranked_ids(&self) -> &[String] {
        &self.ranked_ids
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn metrics(&self) -> &QueryMetrics {
        &self.metrics
    }
    pub fn latency_ms(&self) -> Option<f64> {
        self.latency_ms
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct EvaluationRun {
    schema_version: u32,
    dataset: String,
    corpus_fingerprint: String,
    recipe: String,
    cases: BTreeMap<String, CaseMeasurement>,
    summary: QualitySummary,
    peak_rss_bytes: Option<u64>,
}
impl EvaluationRun {
    pub fn new(
        dataset: impl Into<String>,
        corpus_fingerprint: impl Into<String>,
        recipe: impl Into<String>,
        cases: Vec<CaseMeasurement>,
        peak_rss_bytes: Option<u64>,
    ) -> Result<Self, GateError> {
        let (dataset, corpus_fingerprint, recipe) =
            (dataset.into(), corpus_fingerprint.into(), recipe.into());
        if !label(&dataset)
            || !hash_valid(&corpus_fingerprint)
            || !label(&recipe)
            || cases.is_empty()
            || cases.len() > 10000
        {
            return Err(GateError::Invalid("run identity/count"));
        }
        let total = cases
            .iter()
            .flat_map(|c| &c.ranked_ids)
            .try_fold(0usize, |n, id| n.checked_add(id.len()))
            .ok_or(GateError::Invalid("run bytes"))?;
        if total > 64 * 1024 * 1024 {
            return Err(GateError::Invalid("run ranked-ID bytes"));
        }
        let summary = summarize(&cases.iter().map(|c| c.metrics.clone()).collect::<Vec<_>>())?;
        let mut indexed = BTreeMap::new();
        for case in cases {
            if indexed.insert(case.id.clone(), case).is_some() {
                return Err(GateError::Invalid("duplicate case ID"));
            }
        }
        Ok(Self {
            schema_version: 1,
            dataset,
            corpus_fingerprint,
            recipe,
            cases: indexed,
            summary,
            peak_rss_bytes,
        })
    }
    pub fn cases(&self) -> &BTreeMap<String, CaseMeasurement> {
        &self.cases
    }
    pub fn summary(&self) -> &QualitySummary {
        &self.summary
    }
    pub fn recipe(&self) -> &str {
        &self.recipe
    }
    pub fn dataset(&self) -> &str {
        &self.dataset
    }
    pub fn fingerprint(&self) -> Result<String, GateError> {
        Ok(digest(&[
            b"airust-evaluation-run-v1",
            &serde_json::to_vec(self)?,
        ]))
    }
    pub fn to_json(&self) -> Result<String, GateError> {
        Ok(serde_json::to_string(self)?)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Metric {
    Precision,
    Recall,
    ReciprocalRank,
    Ndcg,
}
#[derive(Debug, Clone, Serialize)]
pub struct GatePolicy {
    metric: Metric,
    floor: Option<f64>,
    allowed_mean_drop: f64,
    allowed_query_drop: f64,
    max_regressed_queries: usize,
    max_latency_ratio: Option<f64>,
    max_peak_rss: Option<u64>,
}
impl Default for GatePolicy {
    fn default() -> Self {
        Self {
            metric: Metric::Ndcg,
            floor: None,
            allowed_mean_drop: 0.,
            allowed_query_drop: 0.,
            max_regressed_queries: 0,
            max_latency_ratio: None,
            max_peak_rss: None,
        }
    }
}
impl GatePolicy {
    pub fn new(metric: Metric) -> Self {
        Self {
            metric,
            ..Self::default()
        }
    }
    pub fn with_quality_limits(
        mut self,
        floor: Option<f64>,
        mean_drop: f64,
        query_drop: f64,
        max_regressions: usize,
    ) -> Result<Self, GateError> {
        if [floor.unwrap_or(0.), mean_drop, query_drop]
            .iter()
            .any(|v| !v.is_finite() || !(0.0..=1.).contains(v))
            || max_regressions > 10000
        {
            return Err(GateError::Invalid("quality limits"));
        }
        self.floor = floor;
        self.allowed_mean_drop = mean_drop;
        self.allowed_query_drop = query_drop;
        self.max_regressed_queries = max_regressions;
        Ok(self)
    }
    pub fn with_resource_limits(
        mut self,
        latency_ratio: Option<f64>,
        peak_rss_bytes: Option<u64>,
    ) -> Result<Self, GateError> {
        if latency_ratio.is_some_and(|v| !v.is_finite() || v < 1.) || peak_rss_bytes == Some(0) {
            return Err(GateError::Invalid("resource limits"));
        }
        self.max_latency_ratio = latency_ratio;
        self.max_peak_rss = peak_rss_bytes;
        Ok(self)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum GateVerdict {
    Passed,
    Failed,
    Undetermined,
}
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct GateFinding {
    pub check: String,
    pub case_id: Option<String>,
    pub verdict: GateVerdict,
    pub baseline: Option<f64>,
    pub candidate: Option<f64>,
}
#[derive(Debug, Clone, Serialize)]
pub struct GateReport {
    baseline: String,
    candidate: String,
    policy: GatePolicy,
    verdict: GateVerdict,
    regressed_queries: usize,
    findings: Vec<GateFinding>,
}
impl GateReport {
    pub fn verdict(&self) -> GateVerdict {
        self.verdict
    }
    pub fn findings(&self) -> &[GateFinding] {
        &self.findings
    }
    pub fn regressed_queries(&self) -> usize {
        self.regressed_queries
    }
    pub fn to_json(&self) -> Result<String, GateError> {
        Ok(serde_json::to_string(self)?)
    }
}
pub fn compare_runs(
    baseline: &EvaluationRun,
    candidate: &EvaluationRun,
    policy: &GatePolicy,
) -> Result<GateReport, GateError> {
    if baseline.dataset != candidate.dataset
        || baseline.corpus_fingerprint != candidate.corpus_fingerprint
        || baseline.cases.len() != candidate.cases.len()
    {
        return Err(GateError::Incomparable("dataset/corpus/case count"));
    }
    for (id, a) in &baseline.cases {
        let b = candidate
            .cases
            .get(id)
            .ok_or(GateError::Incomparable("case IDs"))?;
        if a.identity != b.identity {
            return Err(GateError::Incomparable("query/judgments/cutoff/policy"));
        }
    }
    let mut findings = Vec::new();
    let mut regressed = 0;
    let a = mean(&baseline.summary, policy.metric);
    let b = mean(&candidate.summary, policy.metric);
    add(
        &mut findings,
        "mean_quality",
        None,
        a,
        b,
        match (a, b) {
            (Some(a), Some(b)) => {
                if b + policy.allowed_mean_drop + 1e-12 < a {
                    GateVerdict::Failed
                } else {
                    GateVerdict::Passed
                }
            }
            _ => GateVerdict::Undetermined,
        },
    );
    if let Some(floor) = policy.floor {
        add(
            &mut findings,
            "quality_floor",
            None,
            Some(floor),
            b,
            b.map_or(GateVerdict::Undetermined, |b| {
                if b + 1e-12 < floor {
                    GateVerdict::Failed
                } else {
                    GateVerdict::Passed
                }
            }),
        );
    }
    for (id, a) in &baseline.cases {
        let b = &candidate.cases[id];
        match (
            value(&a.metrics, policy.metric),
            value(&b.metrics, policy.metric),
        ) {
            (Some(av), Some(bv)) if bv + policy.allowed_query_drop + 1e-12 < av => {
                regressed += 1;
                add(
                    &mut findings,
                    "query_regression",
                    Some(id.clone()),
                    Some(av),
                    Some(bv),
                    GateVerdict::Passed,
                );
            }
            (None, _) | (_, None) => add(
                &mut findings,
                "undefined_query_metric",
                Some(id.clone()),
                None,
                None,
                GateVerdict::Undetermined,
            ),
            _ => {}
        }
    }
    add(
        &mut findings,
        "regression_count",
        None,
        Some(policy.max_regressed_queries as f64),
        Some(regressed as f64),
        if regressed > policy.max_regressed_queries {
            GateVerdict::Failed
        } else {
            GateVerdict::Passed
        },
    );
    if let Some(ratio) = policy.max_latency_ratio {
        let a = latency(baseline);
        let b = latency(candidate);
        add(
            &mut findings,
            "mean_latency",
            None,
            a,
            b,
            match (a, b) {
                (Some(a), Some(b)) => {
                    if b > a * ratio + 1e-12 {
                        GateVerdict::Failed
                    } else {
                        GateVerdict::Passed
                    }
                }
                _ => GateVerdict::Undetermined,
            },
        );
    }
    if let Some(cap) = policy.max_peak_rss {
        add(
            &mut findings,
            "peak_rss",
            None,
            Some(cap as f64),
            candidate.peak_rss_bytes.map(|n| n as f64),
            candidate
                .peak_rss_bytes
                .map_or(GateVerdict::Undetermined, |n| {
                    if n > cap {
                        GateVerdict::Failed
                    } else {
                        GateVerdict::Passed
                    }
                }),
        );
    }
    let verdict = if findings.iter().any(|f| f.verdict == GateVerdict::Failed) {
        GateVerdict::Failed
    } else if findings
        .iter()
        .any(|f| f.verdict == GateVerdict::Undetermined)
    {
        GateVerdict::Undetermined
    } else {
        GateVerdict::Passed
    };
    Ok(GateReport {
        baseline: baseline.fingerprint()?,
        candidate: candidate.fingerprint()?,
        policy: policy.clone(),
        verdict,
        regressed_queries: regressed,
        findings,
    })
}
fn add(
    out: &mut Vec<GateFinding>,
    check: &str,
    case_id: Option<String>,
    baseline: Option<f64>,
    candidate: Option<f64>,
    verdict: GateVerdict,
) {
    out.push(GateFinding {
        check: check.into(),
        case_id,
        verdict,
        baseline,
        candidate,
    });
}
fn value(m: &QueryMetrics, metric: Metric) -> Option<f64> {
    match metric {
        Metric::Precision => Some(m.precision_at_k()),
        Metric::Recall => m.recall_at_k(),
        Metric::ReciprocalRank => Some(m.reciprocal_rank_at_k()),
        Metric::Ndcg => m.ndcg_at_k(),
    }
}
fn mean(m: &QualitySummary, metric: Metric) -> Option<f64> {
    match metric {
        Metric::Precision => Some(m.mean_precision_at_k()),
        Metric::Recall => m.mean_recall_at_k(),
        Metric::ReciprocalRank => Some(m.mean_reciprocal_rank_at_k()),
        Metric::Ndcg => m.mean_ndcg_at_k(),
    }
}
fn latency(run: &EvaluationRun) -> Option<f64> {
    let mut sum = 0.;
    for c in run.cases.values() {
        sum += c.latency_ms?;
    }
    Some(sum / run.cases.len() as f64)
}
fn label(s: &str) -> bool {
    !s.trim().is_empty() && s.len() <= 256 && !s.chars().any(char::is_control)
}
fn hash_valid(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn digest(parts: &[&[u8]]) -> String {
    let mut h = Sha256::new();
    for p in parts {
        h.update((p.len() as u64).to_le_bytes());
        h.update(p);
    }
    format!("{:x}", h.finalize())
}
