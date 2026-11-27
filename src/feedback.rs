#![doc = include_str!("../docs/FEEDBACK.md")]
use crate::reranking::{RerankError, Reranker};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use thiserror::Error;
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum FeedbackError {
    #[error("invalid feedback configuration: {0}")]
    Configuration(&'static str),
    #[error("feedback limit exceeded: {0}")]
    Limit(&'static str),
    #[error("invalid feedback data: {0}")]
    InvalidData(&'static str),
    #[error("feedback revision exhausted")]
    Exhausted,
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
#[derive(Debug, Clone, Copy)]
pub struct FeedbackLimits {
    events: usize,
    json_bytes: usize,
}
impl Default for FeedbackLimits {
    fn default() -> Self {
        Self {
            events: 10000,
            json_bytes: 8 * 1024 * 1024,
        }
    }
}
impl FeedbackLimits {
    pub fn new(events: usize, json_bytes: usize) -> Result<Self, FeedbackError> {
        if events == 0 || events > 100000 || json_bytes == 0 || json_bytes > 64 * 1024 * 1024 {
            return Err(FeedbackError::Configuration(
                "event/JSON limits are outside supported bounds",
            ));
        }
        Ok(Self { events, json_bytes })
    }
    pub fn events(&self) -> usize {
        self.events
    }
    pub fn json_bytes(&self) -> usize {
        self.json_bytes
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Rating {
    Helpful,
    Unhelpful,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FeedbackKey {
    namespace: String,
    query: String,
    document: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeedbackEvent {
    id: String,
    key: FeedbackKey,
    rating: Rating,
}
impl FeedbackEvent {
    pub fn new(
        id: impl Into<String>,
        namespace: impl Into<String>,
        query: &str,
        document: &str,
        rating: Rating,
    ) -> Result<Self, FeedbackError> {
        let event = Self {
            id: id.into(),
            key: key(namespace.into(), query, document)?,
            rating,
        };
        validate_event(&event)?;
        Ok(event)
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn namespace(&self) -> &str {
        &self.key.namespace
    }
    pub fn query_fingerprint(&self) -> &str {
        &self.key.query
    }
    pub fn document_fingerprint(&self) -> &str {
        &self.key.document
    }
    pub fn rating(&self) -> Rating {
        self.rating
    }
}
#[derive(Debug, Clone, Copy, Default, Serialize)]
struct Votes {
    helpful: usize,
    unhelpful: usize,
}
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct FeedbackReceipt {
    pub changed: bool,
    pub revision: u64,
    pub events: usize,
}
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct FeedbackEvidence {
    pub helpful: usize,
    pub unhelpful: usize,
    pub event_ids: Vec<String>,
    pub truncated: bool,
}
#[derive(Debug, Clone, Default)]
pub struct FeedbackLedger {
    events: BTreeMap<String, FeedbackEvent>,
    counts: BTreeMap<FeedbackKey, Votes>,
    revision: u64,
    limits: FeedbackLimits,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LedgerData {
    schema_version: u32,
    revision: u64,
    events: Vec<FeedbackEvent>,
}
impl FeedbackLedger {
    pub fn with_limits(limits: FeedbackLimits) -> Self {
        Self {
            limits,
            ..Self::default()
        }
    }
    pub fn len(&self) -> usize {
        self.events.len()
    }
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn events(&self) -> impl Iterator<Item = &FeedbackEvent> {
        self.events.values()
    }
    pub fn event(&self, id: &str) -> Option<&FeedbackEvent> {
        self.events.get(id)
    }
    pub fn record(&mut self, event: FeedbackEvent) -> Result<FeedbackReceipt, FeedbackError> {
        validate_event(&event)?;
        if self.events.get(&event.id) == Some(&event) {
            return Ok(self.receipt(false));
        }
        if !self.events.contains_key(&event.id) && self.events.len() >= self.limits.events {
            return Err(FeedbackError::Limit("event count"));
        }
        let next = self
            .revision
            .checked_add(1)
            .ok_or(FeedbackError::Exhausted)?;
        if let Some(old) = self.events.remove(&event.id) {
            self.remove_vote(&old);
        }
        self.add_vote(&event);
        self.events.insert(event.id.clone(), event);
        self.revision = next;
        Ok(self.receipt(true))
    }
    pub fn retract(&mut self, id: &str) -> Result<FeedbackReceipt, FeedbackError> {
        if !self.events.contains_key(id) {
            return Ok(self.receipt(false));
        }
        let next = self
            .revision
            .checked_add(1)
            .ok_or(FeedbackError::Exhausted)?;
        let old = self.events.remove(id).expect("checked event exists");
        self.remove_vote(&old);
        self.revision = next;
        Ok(self.receipt(true))
    }
    pub fn evidence(
        &self,
        namespace: &str,
        query: &str,
        document: &str,
        max_ids: usize,
    ) -> Result<FeedbackEvidence, FeedbackError> {
        let key = key(namespace.to_owned(), query, document)?;
        let votes = self.counts.get(&key).copied().unwrap_or_default();
        let event_ids = self
            .events
            .values()
            .filter(|event| event.key == key)
            .take(max_ids.min(1000))
            .map(|event| event.id.clone())
            .collect::<Vec<_>>();
        Ok(FeedbackEvidence {
            helpful: votes.helpful,
            unhelpful: votes.unhelpful,
            truncated: event_ids.len() < votes.helpful + votes.unhelpful,
            event_ids,
        })
    }
    pub fn profile_fingerprint(&self, namespace: &str) -> Result<String, FeedbackError> {
        if !crate::text::Identifier::label(namespace, 128) {
            return Err(FeedbackError::InvalidData("namespace"));
        }
        let events = self
            .events
            .values()
            .filter(|event| event.key.namespace == namespace)
            .collect::<Vec<_>>();
        Ok(digest_parts(&[
            b"airust-feedback-profile-v1",
            namespace.as_bytes(),
            &serde_json::to_vec(&events)?,
        ]))
    }
    pub fn to_json(&self) -> Result<String, FeedbackError> {
        let data = LedgerData {
            schema_version: 1,
            revision: self.revision,
            events: self.events.values().cloned().collect(),
        };
        let json = serde_json::to_string(&data)?;
        if json.len() > self.limits.json_bytes {
            return Err(FeedbackError::Limit("JSON bytes"));
        }
        Ok(json)
    }
    pub fn from_json(json: &str, limits: FeedbackLimits) -> Result<Self, FeedbackError> {
        if json.len() > limits.json_bytes {
            return Err(FeedbackError::Limit("JSON bytes"));
        }
        let data: LedgerData = serde_json::from_str(json).map_err(crate::input::json_error)?;
        if data.schema_version != 1
            || data.events.len() > limits.events
            || data.revision < (data.events.len() as u64)
        {
            return Err(FeedbackError::InvalidData("schema/count/revision"));
        }
        let mut ledger = Self::with_limits(limits);
        for event in data.events {
            validate_event(&event)?;
            if ledger.events.contains_key(&event.id) {
                return Err(FeedbackError::InvalidData("duplicate event ID"));
            }
            ledger.add_vote(&event);
            ledger.events.insert(event.id.clone(), event);
        }
        ledger.revision = data.revision;
        Ok(ledger)
    }
    fn receipt(&self, changed: bool) -> FeedbackReceipt {
        FeedbackReceipt {
            changed,
            revision: self.revision,
            events: self.len(),
        }
    }
    fn add_vote(&mut self, event: &FeedbackEvent) {
        let votes = self.counts.entry(event.key.clone()).or_default();
        match event.rating {
            Rating::Helpful => votes.helpful += 1,
            Rating::Unhelpful => votes.unhelpful += 1,
        }
    }
    fn remove_vote(&mut self, event: &FeedbackEvent) {
        if let Some(votes) = self.counts.get_mut(&event.key) {
            match event.rating {
                Rating::Helpful => votes.helpful -= 1,
                Rating::Unhelpful => votes.unhelpful -= 1,
            }
            if votes.helpful + votes.unhelpful == 0 {
                self.counts.remove(&event.key);
            }
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct FeedbackOptions {
    weight: f64,
    prior: usize,
}
impl Default for FeedbackOptions {
    fn default() -> Self {
        Self {
            weight: 1.0,
            prior: 2,
        }
    }
}
impl FeedbackOptions {
    pub fn new(weight: f64, prior: usize) -> Result<Self, FeedbackError> {
        if !weight.is_finite()
            || !(0.0..=100.0).contains(&weight)
            || !(1..=1000000).contains(&prior)
        {
            return Err(FeedbackError::Configuration("feedback weight/prior"));
        }
        Ok(Self { weight, prior })
    }
    pub fn weight(&self) -> f64 {
        self.weight
    }
    pub fn prior(&self) -> usize {
        self.prior
    }
}
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct FeedbackAdjustment {
    pub batch_position: usize,
    pub document_fingerprint: String,
    pub helpful: usize,
    pub unhelpful: usize,
    pub base_score: f64,
    pub boost: f64,
    pub score: f64,
}
pub struct FeedbackReranker<'a> {
    base: &'a mut dyn Reranker,
    ledger: &'a FeedbackLedger,
    namespace: String,
    options: FeedbackOptions,
    base_recipe: String,
    recipe: String,
    last: Vec<FeedbackAdjustment>,
}
impl<'a> FeedbackReranker<'a> {
    pub fn new(
        base: &'a mut dyn Reranker,
        ledger: &'a FeedbackLedger,
        namespace: impl Into<String>,
        options: FeedbackOptions,
    ) -> Result<Self, FeedbackError> {
        let namespace = namespace.into();
        if base.recipe().trim().is_empty() || base.recipe().len() > 1024 {
            return Err(FeedbackError::Configuration("base scorer recipe"));
        }
        let base_recipe = base.recipe().to_owned();
        let fingerprint = ledger.profile_fingerprint(&namespace)?;
        let weight = options.weight.to_bits().to_le_bytes();
        let prior = (options.prior as u64).to_le_bytes();
        let recipe = format!(
            "airust-feedback-v1/{}",
            digest_parts(&[
                base_recipe.as_bytes(),
                fingerprint.as_bytes(),
                &weight,
                &prior
            ])
        );
        Ok(Self {
            base,
            ledger,
            namespace,
            options,
            base_recipe,
            recipe,
            last: Vec::new(),
        })
    }
    pub fn last_adjustments(&self) -> &[FeedbackAdjustment] {
        &self.last
    }
}
impl Reranker for FeedbackReranker<'_> {
    fn recipe(&self) -> &str {
        &self.recipe
    }
    fn score(&mut self, query: &str, documents: &[&str]) -> Result<Vec<f64>, RerankError> {
        self.last.clear();
        if documents.is_empty() {
            return Ok(Vec::new());
        }
        if documents.len() > 1000 {
            return Err(RerankError::Budget("feedback document count"));
        }
        let keys = documents
            .iter()
            .map(|document| {
                key(self.namespace.clone(), query, document).map_err(|error| match error {
                    FeedbackError::Limit(_) => RerankError::Budget("feedback query/document"),
                    _ => RerankError::Output("invalid feedback query/document"),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        if self.base.recipe() != self.base_recipe {
            return Err(RerankError::RecipeChanged);
        }
        let result = self.base.score(query, documents);
        if self.base.recipe() != self.base_recipe {
            return Err(RerankError::RecipeChanged);
        }
        let scores = result?;
        if scores.len() != documents.len() {
            return Err(RerankError::ScoreCount {
                expected: documents.len(),
                actual: scores.len(),
            });
        }
        let mut adjusted = Vec::with_capacity(scores.len());
        let mut adjustments = Vec::with_capacity(scores.len());
        for (index, (key, base_score)) in keys.into_iter().zip(scores).enumerate() {
            if !base_score.is_finite() {
                return Err(RerankError::NonFiniteScore(index + 1));
            }
            let votes = self.ledger.counts.get(&key).copied().unwrap_or_default();
            let count = votes.helpful + votes.unhelpful;
            let boost = self.options.weight * (votes.helpful as f64 - votes.unhelpful as f64)
                / (count + self.options.prior) as f64;
            let score = base_score + boost;
            if !score.is_finite() {
                return Err(RerankError::Output("nonfinite adjusted feedback score"));
            }
            adjustments.push(FeedbackAdjustment {
                batch_position: index + 1,
                document_fingerprint: key.document,
                helpful: votes.helpful,
                unhelpful: votes.unhelpful,
                base_score,
                boost,
                score,
            });
            adjusted.push(score);
        }
        self.last = adjustments;
        Ok(adjusted)
    }
}
fn key(namespace: String, query: &str, document: &str) -> Result<FeedbackKey, FeedbackError> {
    if !crate::text::Identifier::label(&namespace, 128) {
        return Err(FeedbackError::InvalidData("namespace"));
    }
    if query.trim().is_empty() || document.trim().is_empty() {
        return Err(FeedbackError::InvalidData("blank query/document"));
    }
    if query.len() > 16384 || document.len() > 65536 {
        return Err(FeedbackError::Limit("query/document bytes"));
    }
    let normalized = crate::text::normalize(query);
    if normalized.len() > 65536 {
        return Err(FeedbackError::Limit("normalized query bytes"));
    }
    Ok(FeedbackKey {
        namespace,
        query: digest_parts(&[b"query-nfkc-v1", normalized.as_bytes()]),
        document: digest_parts(&[b"document-raw-v1", document.as_bytes()]),
    })
}
fn validate_event(event: &FeedbackEvent) -> Result<(), FeedbackError> {
    if !crate::text::Identifier::label(&event.id, 128)
        || !crate::text::Identifier::label(&event.key.namespace, 128)
        || !crate::text::Identifier::sha256(&event.key.query)
        || !crate::text::Identifier::sha256(&event.key.document)
    {
        return Err(FeedbackError::InvalidData("event identity/fingerprints"));
    }
    Ok(())
}
fn digest_parts(parts: &[&[u8]]) -> String {
    let mut hash = Sha256::new();
    for part in parts {
        hash.update((part.len() as u64).to_le_bytes());
        hash.update(part);
    }
    format!("{:x}", hash.finalize())
}
