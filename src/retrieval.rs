#![doc = include_str!("../docs/RETRIEVAL.md")]

use crate::{Context, Engine, KnowledgeEntry, SearchAlgorithm, SearchOptions};
use serde::Serialize;
use serde_json::Value;
use std::{
    collections::BTreeSet,
    io::{self, Write},
};
use thiserror::Error;

pub mod assessment;
pub mod excerpts;
pub use assessment::{
    assess_response, AssessmentLimits, AssessmentReason, EvidenceAssessment, EvidencePolicy,
    EvidenceVerdict, HitAssessment, ScoreGate, ScoreScale,
};
pub use excerpts::{
    select_excerpt, ExcerptAnalyzer, ExcerptDetails, ExcerptMatch, ExcerptMatchKind,
    ExcerptOptions, ExcerptReason, SelectedExcerpt,
};

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum RetrievalError {
    #[cfg(feature = "execution")]
    #[error(transparent)]
    Execution(#[from] crate::execution::ExecutionError),
    #[error("invalid retrieval configuration: {0}")]
    Configuration(&'static str),
    #[error("retrieval budget exceeded: {0}")]
    Budget(&'static str),
    #[error("invalid retrieval source output: {0}")]
    Output(&'static str),
    #[error("retrieval source failed: {0}")]
    Source(String),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[cfg(feature = "reranking")]
    #[error(transparent)]
    Rerank(#[from] crate::reranking::RerankError),
}
#[derive(Debug, Clone)]
pub struct RetrievalOptions {
    #[cfg(feature = "execution")]
    control: Option<crate::execution::WorkControl>,
    limit: usize,
    candidates: usize,
    context: Option<Context>,
    max_query_bytes: usize,
    max_text_bytes: usize,
    max_excerpt_bytes: usize,
    max_output_bytes: usize,
    max_id_bytes: usize,
    max_evidence_bytes: usize,
    metadata: bool,
    max_metadata_bytes: usize,
    excerpt_options: Option<ExcerptOptions>,
    evidence_policy: Option<EvidencePolicy>,
}
impl Default for RetrievalOptions {
    fn default() -> Self {
        Self {
            #[cfg(feature = "execution")]
            control: None,
            limit: 10,
            candidates: 100,
            context: None,
            max_query_bytes: 16 * 1024,
            max_text_bytes: 16 * 1024,
            max_excerpt_bytes: 2048,
            max_output_bytes: 64 * 1024,
            max_id_bytes: 4096,
            max_evidence_bytes: 64 * 1024,
            metadata: false,
            max_metadata_bytes: 4096,
            excerpt_options: None,
            evidence_policy: None,
        }
    }
}
impl RetrievalOptions {
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
    fn checkpoint(&self) -> Result<(), RetrievalError> {
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
    pub fn with_candidates(mut self, n: usize) -> Result<Self, RetrievalError> {
        if n == 0 || n < self.limit {
            return Err(RetrievalError::Configuration(
                "candidates must be positive and at least the result limit",
            ));
        }
        self.candidates = n;
        Ok(self)
    }
    pub fn with_context(mut self, context: Context) -> Self {
        self.context = Some(context);
        self
    }
    pub fn with_byte_budgets(
        mut self,
        query: usize,
        total_text: usize,
        excerpt: usize,
        output: usize,
    ) -> Result<Self, RetrievalError> {
        if query == 0 || excerpt == 0 || output == 0 {
            return Err(RetrievalError::Configuration(
                "query, excerpt and output budgets must be positive",
            ));
        }
        self.max_query_bytes = query;
        self.max_text_bytes = total_text;
        self.max_excerpt_bytes = excerpt;
        self.max_output_bytes = output;
        Ok(self)
    }
    pub fn with_evidence_limits(
        mut self,
        id: usize,
        evidence: usize,
    ) -> Result<Self, RetrievalError> {
        if id == 0 || evidence == 0 {
            return Err(RetrievalError::Configuration(
                "ID and evidence budgets must be positive",
            ));
        }
        self.max_id_bytes = id;
        self.max_evidence_bytes = evidence;
        Ok(self)
    }
    pub fn with_metadata(mut self, max_bytes: usize) -> Result<Self, RetrievalError> {
        if max_bytes == 0 {
            return Err(RetrievalError::Configuration(
                "metadata budget must be positive",
            ));
        }
        self.metadata = true;
        self.max_metadata_bytes = max_bytes;
        Ok(self)
    }
    pub fn with_relevant_excerpts(mut self, options: ExcerptOptions) -> Self {
        self.excerpt_options = Some(options);
        self
    }
    pub fn with_prefix_excerpts(mut self) -> Self {
        self.excerpt_options = None;
        self
    }
    pub fn excerpt_options(&self) -> Option<&ExcerptOptions> {
        self.excerpt_options.as_ref()
    }
    pub fn with_evidence_policy(mut self, policy: EvidencePolicy) -> Self {
        self.evidence_policy = Some(policy);
        self
    }
    pub fn without_evidence_policy(mut self) -> Self {
        self.evidence_policy = None;
        self
    }
    pub fn evidence_policy(&self) -> Option<&EvidencePolicy> {
        self.evidence_policy.as_ref()
    }
    pub fn limit(&self) -> usize {
        self.limit
    }
    pub fn candidates(&self) -> usize {
        self.candidates
    }
    pub fn context(&self) -> Option<&Context> {
        self.context.as_ref()
    }
    pub fn max_query_bytes(&self) -> usize {
        self.max_query_bytes
    }
    pub fn max_text_bytes(&self) -> usize {
        self.max_text_bytes
    }
    pub fn max_excerpt_bytes(&self) -> usize {
        self.max_excerpt_bytes
    }
    pub fn max_output_bytes(&self) -> usize {
        self.max_output_bytes
    }
    pub fn max_id_bytes(&self) -> usize {
        self.max_id_bytes
    }
    pub fn max_evidence_bytes(&self) -> usize {
        self.max_evidence_bytes
    }
    pub fn includes_metadata(&self) -> bool {
        self.metadata
    }
    pub fn max_metadata_bytes(&self) -> usize {
        self.max_metadata_bytes
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum RankingKind {
    Bm25,
    Tfidf,
    TextBm25,
    TextTfidf,
    Cosine,
    ReciprocalRankFusion,
    Custom,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ExcerptSpan {
    pub start: usize,
    pub end: usize,
}
impl ExcerptSpan {
    pub fn slice<'a>(&self, text: &'a str) -> Option<&'a str> {
        text.get(self.start..self.end)
    }
    pub fn relative_to(self, outer: Self) -> Option<Self> {
        if self.start > self.end || outer.start > self.start || self.end > outer.end {
            return None;
        }
        Some(Self {
            start: self.start - outer.start,
            end: self.end - outer.start,
        })
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct RankingEvidence {
    pub kind: RankingKind,
    pub score: f64,
    pub matched_terms: Vec<String>,
    pub lexical_rank: Option<usize>,
    pub semantic_rank: Option<usize>,
    pub lexical_score: Option<f64>,
    pub semantic_score: Option<f64>,
    pub phrase_spans: Vec<ExcerptSpan>,
    pub phrase_evidence_truncated: bool,
}
impl RankingEvidence {
    pub fn new(kind: RankingKind, score: f64) -> Self {
        Self {
            kind,
            score,
            matched_terms: Vec::new(),
            lexical_rank: None,
            semantic_rank: None,
            lexical_score: None,
            semantic_score: None,
            phrase_spans: Vec::new(),
            phrase_evidence_truncated: false,
        }
    }
}
#[derive(Debug)]
pub struct RetrievalCandidate<'a> {
    pub entry: &'a KnowledgeEntry,
    pub evidence: RankingEvidence,
}
#[cfg(feature = "reranking")]
impl crate::reranking::Rerankable for RetrievalCandidate<'_> {
    fn entry(&self) -> &KnowledgeEntry {
        self.entry
    }
}
#[derive(Debug)]
pub struct RetrievalBatch<'a> {
    pub candidates: Vec<RetrievalCandidate<'a>>,
    pub recipe: String,
    pub fallback_reason: Option<String>,
}
impl<'a> RetrievalBatch<'a> {
    pub fn new(recipe: impl Into<String>, candidates: Vec<RetrievalCandidate<'a>>) -> Self {
        Self {
            recipe: recipe.into(),
            candidates,
            fallback_reason: None,
        }
    }
}
pub trait RetrievalSource {
    fn excerpt_analyzer(&self) -> ExcerptAnalyzer {
        ExcerptAnalyzer::Legacy
    }
    fn retrieve<'a>(
        &'a mut self,
        query: &str,
        options: &RetrievalOptions,
    ) -> Result<RetrievalBatch<'a>, RetrievalError>;
}
impl<T: RetrievalSource + ?Sized> RetrievalSource for Box<T> {
    fn excerpt_analyzer(&self) -> ExcerptAnalyzer {
        (**self).excerpt_analyzer()
    }
    fn retrieve<'a>(
        &'a mut self,
        query: &str,
        options: &RetrievalOptions,
    ) -> Result<RetrievalBatch<'a>, RetrievalError> {
        (**self).retrieve(query, options)
    }
}
impl<T: RetrievalSource + ?Sized> RetrievalSource for &mut T {
    fn excerpt_analyzer(&self) -> ExcerptAnalyzer {
        (**self).excerpt_analyzer()
    }
    fn retrieve<'a>(
        &'a mut self,
        query: &str,
        options: &RetrievalOptions,
    ) -> Result<RetrievalBatch<'a>, RetrievalError> {
        (**self).retrieve(query, options)
    }
}
fn scope(base: Option<&Context>, request: Option<&Context>) -> Option<Context> {
    match (base, request) {
        (Some(a), Some(b)) => a.intersection(b),
        (Some(a), None) | (None, Some(a)) => Some(a.clone()),
        (None, None) => Some(Context::default()),
    }
}
fn kind(algorithm: SearchAlgorithm, phrase: bool) -> RankingKind {
    match (algorithm, phrase) {
        (SearchAlgorithm::Bm25, false) => RankingKind::Bm25,
        (SearchAlgorithm::Tfidf, false) => RankingKind::Tfidf,
        (SearchAlgorithm::Bm25, true) => RankingKind::TextBm25,
        (SearchAlgorithm::Tfidf, true) => RankingKind::TextTfidf,
    }
}
pub struct LexicalSource<'a> {
    index: &'a Engine,
    options: SearchOptions,
}
impl<'a> LexicalSource<'a> {
    pub fn new(index: &'a Engine) -> Self {
        Self {
            index,
            options: SearchOptions::default(),
        }
    }
    pub fn with_options(mut self, options: SearchOptions) -> Self {
        self.options = options;
        self
    }
}
impl RetrievalSource for LexicalSource<'_> {
    fn retrieve<'a>(
        &'a mut self,
        query: &str,
        request: &RetrievalOptions,
    ) -> Result<RetrievalBatch<'a>, RetrievalError> {
        let mut options = self.options.clone();
        let Some(context) = scope(options.context.as_ref(), request.context()) else {
            return Ok(RetrievalBatch::new("airust-lexical-v1", Vec::new()));
        };
        options.context = Some(context);
        options.limit = request.candidates;
        #[cfg(feature = "execution")]
        let hits = if let Some(control) = request.control() {
            self.index.search_controlled(query, &options, control)?
        } else {
            self.index.search_with_options(query, &options)
        };
        #[cfg(not(feature = "execution"))]
        let hits = self.index.search_with_options(query, &options);
        Ok(RetrievalBatch::new(
            "airust-lexical-v1",
            hits.into_iter()
                .map(|hit| {
                    let mut evidence =
                        RankingEvidence::new(kind(options.algorithm, false), hit.score);
                    evidence.matched_terms = hit.matched_terms;
                    RetrievalCandidate {
                        entry: hit.entry,
                        evidence,
                    }
                })
                .collect(),
        ))
    }
}
#[cfg(feature = "advanced-search")]
pub struct TextSource<'a> {
    index: &'a crate::text_search::TextIndex,
    options: crate::text_search::TextSearchOptions,
}
#[cfg(feature = "advanced-search")]
impl<'a> TextSource<'a> {
    pub fn new(index: &'a crate::text_search::TextIndex) -> Self {
        Self {
            index,
            options: crate::text_search::TextSearchOptions::default(),
        }
    }
    pub fn with_options(mut self, options: crate::text_search::TextSearchOptions) -> Self {
        self.options = options;
        self
    }
}
#[cfg(feature = "advanced-search")]
impl RetrievalSource for TextSource<'_> {
    fn excerpt_analyzer(&self) -> ExcerptAnalyzer {
        ExcerptAnalyzer::Text(self.index.analyzer().config().clone())
    }
    fn retrieve<'a>(
        &'a mut self,
        query: &str,
        request: &RetrievalOptions,
    ) -> Result<RetrievalBatch<'a>, RetrievalError> {
        let mut search = self.options.search_options().clone();
        let Some(context) = scope(search.context.as_ref(), request.context()) else {
            return Ok(RetrievalBatch::new("airust-text-v1", Vec::new()));
        };
        search.context = Some(context);
        search.limit = request.candidates;
        let algorithm = search.algorithm;
        let options = self.options.clone().with_search(search);
        #[cfg(feature = "execution")]
        let hits = if let Some(control) = request.control() {
            self.index.search_controlled(query, &options, control)
        } else {
            self.index.search(query, &options)
        };
        #[cfg(not(feature = "execution"))]
        let hits = self.index.search(query, &options);
        request.checkpoint()?;
        let hits = hits.map_err(|e| RetrievalError::Source(e.to_string()))?;
        Ok(RetrievalBatch::new(
            self.index.analyzer().config().recipe(),
            hits.into_iter()
                .map(|hit| {
                    let mut evidence = RankingEvidence::new(kind(algorithm, true), hit.score);
                    evidence.matched_terms = hit.matched_terms;
                    for phrase in hit.phrases {
                        evidence.phrase_evidence_truncated |= phrase.truncated;
                        evidence
                            .phrase_spans
                            .extend(phrase.spans.into_iter().map(|s| ExcerptSpan {
                                start: s.start,
                                end: s.end,
                            }));
                    }
                    RetrievalCandidate {
                        entry: hit.entry,
                        evidence,
                    }
                })
                .collect(),
        ))
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataStatus {
    Omitted,
    Missing,
    Included,
    BudgetExceeded,
}
#[derive(Debug, Serialize)]
pub struct RetrievalHit {
    pub id: String,
    pub excerpt: String,
    pub span: ExcerptSpan,
    pub text_truncated: bool,
    pub source_rank: usize,
    pub evidence: RankingEvidence,
    pub rerank_score: Option<f64>,
    pub metadata_status: MetadataStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}
#[derive(Debug, Default, Serialize)]
pub struct RerankSummary {
    pub used: bool,
    pub recipe: Option<String>,
    pub fallback_reason: Option<String>,
    pub selected: usize,
    pub scored: usize,
}
#[derive(Debug, Serialize)]
pub struct RetrievalResponse {
    schema_version: u32,
    recipe: String,
    fallback_reason: Option<String>,
    received_candidates: usize,
    reranking: Option<RerankSummary>,
    truncated: bool,
    hits: Vec<RetrievalHit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    excerpt_details: Option<Vec<ExcerptDetails>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    assessment: Option<EvidenceAssessment>,
    #[serde(skip)]
    requested_limit: usize,
    #[serde(skip)]
    before_output_budget: usize,
    #[serde(skip)]
    json: String,
}
impl RetrievalResponse {
    pub fn hits(&self) -> &[RetrievalHit] {
        &self.hits
    }
    pub fn recipe(&self) -> &str {
        &self.recipe
    }
    pub fn fallback_reason(&self) -> Option<&str> {
        self.fallback_reason.as_deref()
    }
    pub fn received_candidates(&self) -> usize {
        self.received_candidates
    }
    pub fn reranking(&self) -> Option<&RerankSummary> {
        self.reranking.as_ref()
    }
    pub fn truncated(&self) -> bool {
        self.truncated
    }
    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }
    pub fn to_json(&self) -> &str {
        &self.json
    }
    pub fn excerpt_details(&self) -> Option<&[ExcerptDetails]> {
        self.excerpt_details.as_deref()
    }
    pub fn assessment(&self) -> Option<&EvidenceAssessment> {
        self.assessment.as_ref()
    }
    pub fn requested_limit(&self) -> usize {
        self.requested_limit
    }
}
struct BoundedWriter {
    bytes: Vec<u8>,
    limit: usize,
    exceeded: bool,
}
impl Write for BoundedWriter {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        if b.len() > self.limit.saturating_sub(self.bytes.len()) {
            self.exceeded = true;
            return Err(io::Error::other("retrieval serialization limit"));
        }
        self.bytes.extend_from_slice(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn bounded_json(value: &impl Serialize, limit: usize) -> Result<Option<String>, RetrievalError> {
    let mut w = BoundedWriter {
        bytes: Vec::new(),
        limit,
        exceeded: false,
    };
    let result = serde_json::to_writer(&mut w, value);
    if w.exceeded {
        return Ok(None);
    }
    result?;
    Ok(Some(
        String::from_utf8(w.bytes).map_err(|_| RetrievalError::Output("non-UTF8 JSON"))?,
    ))
}
fn metadata_safe(value: &Value) -> bool {
    let mut stack = vec![(value, 1usize)];
    let mut count = 0usize;
    while let Some((v, depth)) = stack.pop() {
        count += 1;
        if count > 100_000 || depth > 64 {
            return false;
        }
        match v {
            Value::Array(a) => {
                if a.len() > 100_000usize.saturating_sub(count + stack.len()) {
                    return false;
                }
                stack.extend(a.iter().map(|v| (v, depth + 1)));
            }
            Value::Object(a) => {
                if a.len() > 100_000usize.saturating_sub(count + stack.len()) {
                    return false;
                }
                stack.extend(a.values().map(|v| (v, depth + 1)));
            }
            _ => {}
        }
    }
    true
}
fn candidate_valid(
    c: &RetrievalCandidate<'_>,
    options: &RetrievalOptions,
) -> Result<(), RetrievalError> {
    if c.entry.id.trim().is_empty() || c.entry.text.trim().is_empty() {
        return Err(RetrievalError::Output("blank ID/text"));
    }
    if c.entry.id.len() > options.max_id_bytes {
        return Err(RetrievalError::Budget("ID bytes"));
    }
    if options.context().is_some_and(|s| !s.includes(c.entry)) {
        return Err(RetrievalError::Output(
            "source returned an out-of-scope entry",
        ));
    }
    let e = &c.evidence;
    if !e.score.is_finite()
        || e.lexical_score.is_some_and(|s| !s.is_finite())
        || e.semantic_score.is_some_and(|s| !s.is_finite())
        || e.lexical_rank == Some(0)
        || e.semantic_rank == Some(0)
    {
        return Err(RetrievalError::Output("invalid ranking evidence"));
    }
    if e.phrase_spans
        .iter()
        .any(|s| s.start >= s.end || c.entry.text.get(s.start..s.end).is_none())
    {
        return Err(RetrievalError::Output("invalid phrase span"));
    }
    if bounded_json(e, options.max_evidence_bytes)?.is_none() {
        return Err(RetrievalError::Budget("candidate evidence bytes"));
    }
    Ok(())
}
fn hit(
    candidate: &RetrievalCandidate<'_>,
    rank: usize,
    score: Option<f64>,
    options: &RetrievalOptions,
    remaining: &mut usize,
    query: &str,
    analyzer: &ExcerptAnalyzer,
) -> Result<(RetrievalHit, Option<ExcerptDetails>), RetrievalError> {
    options.checkpoint()?;
    #[cfg(feature = "execution")]
    options.progress(crate::execution::WorkPhase::SelectingExcerpts, 0, None);
    let (span, details) = if let Some(settings) = options.excerpt_options() {
        let selected = select_excerpt(
            &candidate.entry.text,
            query,
            &candidate.evidence,
            analyzer,
            settings,
            options.max_excerpt_bytes.min(*remaining),
        )?;
        (
            selected.span,
            Some(ExcerptDetails {
                id: candidate.entry.id.clone(),
                reason: selected.reason,
                matches: selected.matches,
                matches_truncated: selected.matches_truncated,
            }),
        )
    } else {
        let mut end = candidate
            .entry
            .text
            .len()
            .min(options.max_excerpt_bytes)
            .min(*remaining);
        while !candidate.entry.text.is_char_boundary(end) {
            end -= 1;
        }
        (ExcerptSpan { start: 0, end }, None)
    };
    *remaining -= span.end - span.start;
    let (metadata_status, metadata) = if !options.metadata {
        (MetadataStatus::Omitted, None)
    } else if let Some(v) = &candidate.entry.metadata {
        if metadata_safe(v) && bounded_json(v, options.max_metadata_bytes)?.is_some() {
            (MetadataStatus::Included, Some(v.clone()))
        } else {
            (MetadataStatus::BudgetExceeded, None)
        }
    } else {
        (MetadataStatus::Missing, None)
    };
    Ok((
        RetrievalHit {
            id: candidate.entry.id.clone(),
            excerpt: candidate.entry.text[span.start..span.end].to_string(),
            span,
            text_truncated: span.start > 0 || span.end < candidate.entry.text.len(),
            source_rank: rank,
            evidence: candidate.evidence.clone(),
            rerank_score: score,
            metadata_status,
            metadata,
        },
        details,
    ))
}
pub struct Retriever<'a, S> {
    source: S,
    #[cfg(feature = "reranking")]
    reranker: Option<(
        &'a mut dyn crate::reranking::Reranker,
        crate::reranking::RerankOptions,
    )>,
    lifetime: std::marker::PhantomData<&'a ()>,
}
impl<'a, S: RetrievalSource> Retriever<'a, S> {
    pub fn new(source: S) -> Self {
        Self {
            source,
            #[cfg(feature = "reranking")]
            reranker: None,
            lifetime: std::marker::PhantomData,
        }
    }
    #[cfg(feature = "reranking")]
    pub fn with_reranker(
        mut self,
        provider: &'a mut dyn crate::reranking::Reranker,
        options: crate::reranking::RerankOptions,
    ) -> Self {
        self.reranker = Some((provider, options));
        self
    }
    pub fn query(
        &mut self,
        query: &str,
        options: &RetrievalOptions,
    ) -> Result<RetrievalResponse, RetrievalError> {
        options.checkpoint()?;
        #[cfg(feature = "execution")]
        options.progress(crate::execution::WorkPhase::Query, 0, Some(1));
        if query.trim().is_empty() {
            return Err(RetrievalError::Configuration("query is blank"));
        }
        if query.len() > options.max_query_bytes {
            return Err(RetrievalError::Budget("query bytes"));
        }
        if options.limit == 0 {
            return finish(
                RetrievalResponse {
                    schema_version: 1,
                    recipe: "airust-noop-v1".into(),
                    fallback_reason: None,
                    received_candidates: 0,
                    reranking: None,
                    truncated: false,
                    hits: Vec::new(),
                    excerpt_details: options.excerpt_options().map(|_| Vec::new()),
                    assessment: None,
                    requested_limit: 0,
                    before_output_budget: 0,
                    json: String::new(),
                },
                options,
                options
                    .evidence_policy()
                    .map(|p| assessment::AssessmentPlan::new(query, &ExcerptAnalyzer::Legacy, p, 0))
                    .transpose()?
                    .as_ref(),
            );
        }
        let analyzer = if options.excerpt_options().is_some() || options.evidence_policy().is_some()
        {
            self.source.excerpt_analyzer()
        } else {
            ExcerptAnalyzer::Legacy
        };
        let assessment_plan = options
            .evidence_policy()
            .map(|p| assessment::AssessmentPlan::new(query, &analyzer, p, options.limit))
            .transpose()?;
        #[cfg(feature = "execution")]
        options.progress(crate::execution::WorkPhase::Retrieving, 0, None);
        options.checkpoint()?;
        let batch = self.source.retrieve(query, options);
        options.checkpoint()?;
        let batch = batch?;
        #[cfg(feature = "execution")]
        options.progress(
            crate::execution::WorkPhase::Retrieving,
            batch.candidates.len(),
            Some(batch.candidates.len()),
        );
        if batch.candidates.len() > options.candidates {
            return Err(RetrievalError::Output("source exceeded candidate count"));
        }
        if batch.recipe.trim().is_empty() || batch.recipe.len() > 1024 {
            return Err(RetrievalError::Output("invalid recipe"));
        }
        if batch
            .fallback_reason
            .as_ref()
            .is_some_and(|s| s.len() > 4096)
        {
            return Err(RetrievalError::Budget("fallback reason bytes"));
        }
        let mut ids = BTreeSet::new();
        for c in &batch.candidates {
            options.checkpoint()?;
            candidate_valid(c, options)?;
            if !ids.insert(&c.entry.id) {
                return Err(RetrievalError::Output("duplicate candidate ID"));
            }
        }
        let mut remaining = options.max_text_bytes;
        #[allow(unused_mut)]
        let mut reranking = None;
        #[cfg(feature = "reranking")]
        let hits = if let Some((provider, settings)) = &mut self.reranker {
            #[cfg(feature = "execution")]
            let settings = &if let Some(control) = options.control() {
                settings.clone().with_control(
                    settings
                        .control()
                        .map_or_else(|| control.clone(), |own| own.combined(control)),
                )
            } else {
                settings.clone()
            };
            let outcome = crate::reranking::rerank(query, &batch.candidates, *provider, settings)?;
            options.checkpoint()?;
            reranking = Some(RerankSummary {
                used: outcome.reranker_used(),
                recipe: outcome.recipe().map(str::to_owned),
                fallback_reason: outcome.fallback_reason().map(str::to_owned),
                selected: outcome.selected_candidates(),
                scored: outcome.scored_candidates(),
            });
            outcome
                .results()
                .iter()
                .take(options.limit)
                .map(|r| {
                    hit(
                        r.candidate,
                        r.original_rank,
                        r.score,
                        options,
                        &mut remaining,
                        query,
                        &analyzer,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?
        } else {
            plain_hits(&batch, options, &mut remaining, query, &analyzer)?
        };
        #[cfg(not(feature = "reranking"))]
        let hits = plain_hits(&batch, options, &mut remaining, query, &analyzer)?;
        let (hits, details): (Vec<_>, Vec<_>) = hits.into_iter().unzip();
        let excerpt_details = options
            .excerpt_options()
            .map(|_| details.into_iter().flatten().collect());
        let truncated = hits.len() < batch.candidates.len()
            || excerpt_details
                .as_ref()
                .is_some_and(|details: &Vec<ExcerptDetails>| {
                    details.iter().any(|d| d.matches_truncated)
                })
            || hits
                .iter()
                .any(|h| h.text_truncated || h.metadata_status == MetadataStatus::BudgetExceeded);
        finish(
            RetrievalResponse {
                schema_version: 1,
                recipe: batch.recipe,
                fallback_reason: batch.fallback_reason,
                received_candidates: batch.candidates.len(),
                reranking,
                truncated,
                hits,
                excerpt_details,
                assessment: None,
                requested_limit: options.limit,
                before_output_budget: 0,
                json: String::new(),
            },
            options,
            assessment_plan.as_ref(),
        )
    }
}
fn plain_hits(
    batch: &RetrievalBatch<'_>,
    options: &RetrievalOptions,
    remaining: &mut usize,
    query: &str,
    analyzer: &ExcerptAnalyzer,
) -> Result<Vec<(RetrievalHit, Option<ExcerptDetails>)>, RetrievalError> {
    batch
        .candidates
        .iter()
        .take(options.limit)
        .enumerate()
        .map(|(i, c)| hit(c, i + 1, None, options, remaining, query, analyzer))
        .collect()
}
fn finish(
    mut response: RetrievalResponse,
    options: &RetrievalOptions,
    plan: Option<&assessment::AssessmentPlan>,
) -> Result<RetrievalResponse, RetrievalError> {
    options.checkpoint()?;
    response.before_output_budget = response.hits.len();
    #[cfg(feature = "execution")]
    options.progress(
        crate::execution::WorkPhase::SelectingExcerpts,
        response.hits.len(),
        Some(response.hits.len()),
    );
    #[cfg(feature = "execution")]
    if plan.is_some() {
        options.progress(
            crate::execution::WorkPhase::AssessingEvidence,
            0,
            Some(response.hits.len()),
        );
    }
    let observations = plan.map(|p| p.observe(&response)).transpose()?;
    #[cfg(feature = "execution")]
    if plan.is_some() {
        options.progress(
            crate::execution::WorkPhase::AssessingEvidence,
            response.hits.len(),
            Some(response.hits.len()),
        );
    }
    #[cfg(feature = "execution")]
    let mut attempts = 0usize;
    loop {
        options.checkpoint()?;
        #[cfg(feature = "execution")]
        {
            options.progress(crate::execution::WorkPhase::Serializing, attempts, None);
            attempts += 1;
        }
        response.assessment = plan
            .zip(observations.as_ref())
            .map(|(p, observations)| p.summarize(&response, observations));
        if let Some(json) = bounded_json(&response, options.max_output_bytes)? {
            options.checkpoint()?;
            response.json = json;
            #[cfg(feature = "execution")]
            {
                options.progress(
                    crate::execution::WorkPhase::Serializing,
                    attempts,
                    Some(attempts),
                );
                options.progress(crate::execution::WorkPhase::Query, 1, Some(1));
            }
            return Ok(response);
        }
        if response.hits.pop().is_none() {
            return Err(RetrievalError::Budget("output envelope bytes"));
        }
        if let Some(details) = &mut response.excerpt_details {
            details.pop();
        }
        response.truncated = true;
    }
}

#[cfg(feature = "embeddings")]
pub struct SemanticSource<'i, 'p> {
    index: &'i crate::semantic::SemanticIndex,
    provider: &'p mut dyn crate::semantic::EmbeddingProvider,
    options: crate::semantic::SemanticSearchOptions,
}
#[cfg(feature = "embeddings")]
impl<'i, 'p> SemanticSource<'i, 'p> {
    pub fn new(
        index: &'i crate::semantic::SemanticIndex,
        provider: &'p mut dyn crate::semantic::EmbeddingProvider,
    ) -> Self {
        Self {
            index,
            provider,
            options: crate::semantic::SemanticSearchOptions::default(),
        }
    }
    pub fn with_options(mut self, options: crate::semantic::SemanticSearchOptions) -> Self {
        self.options = options;
        self
    }
}
#[cfg(feature = "embeddings")]
impl RetrievalSource for SemanticSource<'_, '_> {
    fn retrieve<'a>(
        &'a mut self,
        query: &str,
        request: &RetrievalOptions,
    ) -> Result<RetrievalBatch<'a>, RetrievalError> {
        let Some(context) = scope(self.options.context(), request.context()) else {
            return Ok(RetrievalBatch::new("airust-cosine-v1", Vec::new()));
        };
        let options = crate::semantic::SemanticSearchOptions::new(request.candidates)
            .with_min_score(self.options.min_score())
            .map_err(|e| RetrievalError::Source(e.to_string()))?
            .with_context(context);
        #[cfg(feature = "execution")]
        let options = if let Some(control) = request.control() {
            options.with_control(
                self.options
                    .control()
                    .map_or_else(|| control.clone(), |own| own.combined(control)),
            )
        } else if let Some(control) = self.options.control() {
            options.with_control(control.clone())
        } else {
            options
        };
        let results = self.index.search(query, self.provider, &options);
        request.checkpoint()?;
        #[cfg(feature = "execution")]
        if let Err(crate::semantic::EmbeddingError::Execution(error)) = &results {
            return Err(RetrievalError::Execution(*error));
        }
        let results = results.map_err(|e| RetrievalError::Source(e.to_string()))?;
        Ok(RetrievalBatch::new(
            format!(
                "airust-cosine-v1/{}/{}",
                self.index.space().model(),
                self.index.space().dimensions()
            ),
            results
                .into_iter()
                .map(|hit| RetrievalCandidate {
                    entry: hit.entry,
                    evidence: RankingEvidence::new(RankingKind::Cosine, hit.score),
                })
                .collect(),
        ))
    }
}
#[cfg(feature = "hybrid")]
pub struct HybridSource<'i, 'p> {
    index: &'i crate::semantic::SemanticIndex,
    provider: &'p mut dyn crate::semantic::EmbeddingProvider,
    options: crate::hybrid::HybridSearchOptions,
}
#[cfg(feature = "hybrid")]
impl<'i, 'p> HybridSource<'i, 'p> {
    pub fn new(
        index: &'i crate::semantic::SemanticIndex,
        provider: &'p mut dyn crate::semantic::EmbeddingProvider,
    ) -> Self {
        Self {
            index,
            provider,
            options: crate::hybrid::HybridSearchOptions::default(),
        }
    }
    pub fn with_options(mut self, options: crate::hybrid::HybridSearchOptions) -> Self {
        self.options = options;
        self
    }
}
#[cfg(feature = "hybrid")]
impl RetrievalSource for HybridSource<'_, '_> {
    fn retrieve<'a>(
        &'a mut self,
        query: &str,
        request: &RetrievalOptions,
    ) -> Result<RetrievalBatch<'a>, RetrievalError> {
        let Some(context) = scope(self.options.context(), request.context()) else {
            return Ok(RetrievalBatch::new("airust-rrf-v1", Vec::new()));
        };
        let options = crate::hybrid::HybridSearchOptions::new(request.candidates)
            .with_candidates(request.candidates)
            .map_err(|e| RetrievalError::Source(e.to_string()))?
            .with_rrf_k(self.options.rrf_k())
            .map_err(|e| RetrievalError::Source(e.to_string()))?
            .with_weights(
                self.options.lexical_weight(),
                self.options.semantic_weight(),
            )
            .map_err(|e| RetrievalError::Source(e.to_string()))?
            .with_min_score(self.options.min_score())
            .map_err(|e| RetrievalError::Source(e.to_string()))?
            .with_lexical_algorithm(self.options.lexical_algorithm())
            .with_bm25(self.options.bm25())
            .with_context(context)
            .with_failure_policy(self.options.failure_policy());
        #[cfg(feature = "execution")]
        let options = if let Some(control) = request.control() {
            options.with_control(
                self.options
                    .control()
                    .map_or_else(|| control.clone(), |own| own.combined(control)),
            )
        } else if let Some(control) = self.options.control() {
            options.with_control(control.clone())
        } else {
            options
        };
        let outcome = self.index.hybrid_search(query, self.provider, &options);
        request.checkpoint()?;
        #[cfg(feature = "execution")]
        if let Err(crate::hybrid::HybridError::Embedding(
            crate::semantic::EmbeddingError::Execution(error),
        )) = &outcome
        {
            return Err(RetrievalError::Execution(*error));
        }
        let outcome = outcome.map_err(|e| RetrievalError::Source(e.to_string()))?;
        let fallback = outcome.fallback_reason().map(str::to_owned);
        let mut batch = RetrievalBatch::new(
            format!(
                "airust-rrf-v1/{}/{}",
                self.index.space().model(),
                self.index.space().dimensions()
            ),
            outcome
                .into_results()
                .into_iter()
                .map(|hit| {
                    let mut evidence =
                        RankingEvidence::new(RankingKind::ReciprocalRankFusion, hit.score);
                    evidence.lexical_rank = hit.lexical_rank;
                    evidence.semantic_rank = hit.semantic_rank;
                    evidence.lexical_score = hit.lexical_score;
                    evidence.semantic_score = hit.semantic_score;
                    evidence.matched_terms = hit.matched_terms;
                    RetrievalCandidate {
                        entry: hit.entry,
                        evidence,
                    }
                })
                .collect(),
        );
        batch.fallback_reason = fallback;
        Ok(batch)
    }
}
