#![doc = include_str!("../docs/TRAINING-PIPELINE.md")]
use crate::{
    feedback::{FeedbackEvent, FeedbackLedger, Rating},
    learned_ranking::{LinearModel, TrainingExample},
    quality_gates::{EvaluationRun, GateReport, GateVerdict},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum PipelineError {
    #[error("invalid training pipeline data/configuration: {0}")]
    Invalid(&'static str),
    #[error("training pipeline budget exceeded")]
    Budget,
    #[error("model registry revision conflict")]
    Conflict,
    #[error("model candidate was not approved by the matching quality gate")]
    NotApproved,
    #[error("no previous model is retained")]
    NoPrevious,
    #[error("registry generation exhausted")]
    Exhausted,
    #[error(transparent)]
    Learning(#[from] crate::learned_ranking::LearningError),
    #[error(transparent)]
    Feedback(#[from] crate::feedback::FeedbackError),
    #[error(transparent)]
    Gate(#[from] crate::quality_gates::GateError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
#[derive(Debug, Clone)]
pub struct TrainingItem {
    id: String,
    example: TrainingExample,
}
impl TrainingItem {
    pub fn new(
        id: impl Into<String>,
        query: impl Into<String>,
        document: impl Into<String>,
    ) -> Result<Self, PipelineError> {
        let id = id.into();
        if !label(&id, 128) {
            return Err(PipelineError::Invalid("item ID"));
        }
        Ok(Self {
            id,
            example: TrainingExample::new(query, document, false)?,
        })
    }
    pub fn with_group(mut self, group: impl Into<String>) -> Result<Self, PipelineError> {
        self.example = self.example.with_group(group)?;
        Ok(self)
    }
}
#[derive(Debug, Clone)]
pub struct CurationPolicy {
    trusted: BTreeSet<String>,
    min_votes: usize,
    min_margin: usize,
}
impl CurationPolicy {
    pub fn new(trusted: impl IntoIterator<Item = String>) -> Result<Self, PipelineError> {
        let p = Self {
            trusted: trusted.into_iter().collect(),
            min_votes: 2,
            min_margin: 1,
        };
        if p.trusted.len() > 100000 || p.trusted.iter().any(|s| !label(s, 128)) {
            return Err(PipelineError::Invalid("trusted event IDs"));
        }
        Ok(p)
    }
    pub fn with_minimums(mut self, votes: usize, margin: usize) -> Result<Self, PipelineError> {
        if !(1..=100000).contains(&votes) || !(1..=100000).contains(&margin) {
            return Err(PipelineError::Invalid("vote policy"));
        }
        self.min_votes = votes;
        self.min_margin = margin;
        Ok(self)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Row {
    id: String,
    query: String,
    document: String,
    group: String,
    relevant: bool,
    helpful: usize,
    unhelpful: usize,
    events: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DatasetData {
    schema_version: u32,
    namespace: String,
    feedback_profile: String,
    min_votes: usize,
    min_margin: usize,
    considered: usize,
    rows: Vec<Row>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DatasetEnvelope {
    checksum: String,
    data: DatasetData,
}
#[derive(Debug, Clone)]
pub struct TrainingDataset {
    data: DatasetData,
    examples: Vec<TrainingExample>,
}
impl TrainingDataset {
    pub fn examples(&self) -> &[TrainingExample] {
        &self.examples
    }
    pub fn len(&self) -> usize {
        self.examples.len()
    }
    pub fn is_empty(&self) -> bool {
        self.examples.is_empty()
    }
    pub fn considered(&self) -> usize {
        self.data.considered
    }
    pub fn fingerprint(&self) -> Result<String, PipelineError> {
        Ok(digest(&serde_json::to_vec(&self.data)?))
    }
    pub fn is_current(&self, ledger: &FeedbackLedger) -> Result<bool, PipelineError> {
        Ok(ledger.profile_fingerprint(&self.data.namespace)? == self.data.feedback_profile)
    }
    pub fn to_json(&self) -> Result<String, PipelineError> {
        let json = serde_json::to_string(&DatasetEnvelope {
            checksum: self.fingerprint()?,
            data: self.data.clone(),
        })?;
        if json.len() > 16 * 1024 * 1024 {
            return Err(PipelineError::Budget);
        }
        Ok(json)
    }
    pub fn from_json(json: &str) -> Result<Self, PipelineError> {
        if json.len() > 16 * 1024 * 1024 {
            return Err(PipelineError::Budget);
        }
        let wire: DatasetEnvelope = serde_json::from_str(json).map_err(crate::input::json_error)?;
        if digest(&serde_json::to_vec(&wire.data)?) != wire.checksum {
            return Err(PipelineError::Invalid("dataset checksum"));
        }
        let examples = validate_dataset(&wire.data)?;
        Ok(Self {
            data: wire.data,
            examples,
        })
    }
}
pub fn curate_feedback(
    namespace: &str,
    ledger: &FeedbackLedger,
    items: &[TrainingItem],
    policy: &CurationPolicy,
) -> Result<TrainingDataset, PipelineError> {
    if !label(namespace, 128) || items.len() > 10000 {
        return Err(PipelineError::Invalid("namespace/items"));
    }
    let mut bytes = 0usize;
    let mut ids = BTreeSet::new();
    let mut unique = BTreeMap::<(String, String), &TrainingItem>::new();
    for item in items {
        bytes = bytes
            .checked_add(item.example.query().len() + item.example.document().len())
            .ok_or(PipelineError::Budget)?;
        if bytes > 64 * 1024 * 1024 {
            return Err(PipelineError::Budget);
        }
        if !ids.insert(&item.id) {
            return Err(PipelineError::Invalid("duplicate item ID"));
        }
        let probe = FeedbackEvent::new(
            "probe",
            namespace,
            item.example.query(),
            item.example.document(),
            Rating::Helpful,
        )?;
        let key = (
            probe.query_fingerprint().to_string(),
            probe.document_fingerprint().to_string(),
        );
        if let Some(old) = unique.get(&key) {
            let old: &TrainingItem = old;
            if old.example.group() != item.example.group() {
                return Err(PipelineError::Invalid(
                    "duplicate content has inconsistent group",
                ));
            }
            if old.id > item.id {
                unique.insert(key, item);
            }
        } else {
            unique.insert(key, item);
        }
    }
    let mut votes = BTreeMap::<(&str, &str), (usize, usize, Vec<String>)>::new();
    for event in ledger.events() {
        if event.namespace() != namespace || !policy.trusted.contains(event.id()) {
            continue;
        }
        let v = votes
            .entry((event.query_fingerprint(), event.document_fingerprint()))
            .or_default();
        match event.rating() {
            Rating::Helpful => v.0 += 1,
            Rating::Unhelpful => v.1 += 1,
        }
        v.2.push(event.id().into());
    }
    let mut rows = Vec::new();
    for ((q, d), item) in unique {
        let Some((helpful, unhelpful, events)) = votes.get(&(q.as_str(), d.as_str())) else {
            continue;
        };
        if helpful + unhelpful < policy.min_votes
            || helpful.abs_diff(*unhelpful) < policy.min_margin
        {
            continue;
        }
        rows.push(Row {
            id: item.id.clone(),
            query: item.example.query().into(),
            document: item.example.document().into(),
            group: item.example.group().into(),
            relevant: helpful > unhelpful,
            helpful: *helpful,
            unhelpful: *unhelpful,
            events: events.clone(),
        });
    }
    let data = DatasetData {
        schema_version: 1,
        namespace: namespace.into(),
        feedback_profile: ledger.profile_fingerprint(namespace)?,
        min_votes: policy.min_votes,
        min_margin: policy.min_margin,
        considered: items.len(),
        rows,
    };
    let examples = validate_dataset(&data)?;
    let dataset = TrainingDataset { data, examples };
    dataset.to_json()?;
    Ok(dataset)
}
fn validate_dataset(d: &DatasetData) -> Result<Vec<TrainingExample>, PipelineError> {
    if d.schema_version != 1
        || !label(&d.namespace, 128)
        || !hash_valid(&d.feedback_profile)
        || d.rows.len() > 10000
        || d.considered > 10000
        || d.rows.len() > d.considered
        || !(1..=100000).contains(&d.min_votes)
        || !(1..=100000).contains(&d.min_margin)
    {
        return Err(PipelineError::Invalid("dataset metadata"));
    }
    let mut ids = BTreeSet::new();
    let mut pairs = BTreeSet::new();
    let mut events = BTreeSet::new();
    let mut examples = Vec::new();
    for r in &d.rows {
        if !label(&r.id, 128)
            || !ids.insert(&r.id)
            || r.events.len() > 100000
            || r.helpful.checked_add(r.unhelpful) != Some(r.events.len())
            || r.events.len() < d.min_votes
            || r.helpful.abs_diff(r.unhelpful) < d.min_margin
            || r.relevant != (r.helpful > r.unhelpful)
        {
            return Err(PipelineError::Invalid("training label provenance"));
        }
        for id in &r.events {
            if !label(id, 128) || !events.insert(id) {
                return Err(PipelineError::Invalid("duplicate/invalid event provenance"));
            }
        }
        let probe = FeedbackEvent::new(
            "probe",
            &d.namespace,
            &r.query,
            &r.document,
            Rating::Helpful,
        )?;
        if !pairs.insert((
            probe.query_fingerprint().to_owned(),
            probe.document_fingerprint().to_owned(),
        )) {
            return Err(PipelineError::Invalid("duplicate training pair"));
        }
        examples
            .push(TrainingExample::new(&r.query, &r.document, r.relevant)?.with_group(&r.group)?);
    }
    Ok(examples)
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RegistryRevision {
    identity: String,
    generation: u64,
    model: Option<String>,
}
impl RegistryRevision {
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn model_fingerprint(&self) -> Option<&str> {
        self.model.as_deref()
    }
}
#[derive(Debug, Clone)]
pub struct ModelRegistry {
    identity: String,
    generation: u64,
    capacity: usize,
    models: Vec<LinearModel>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RegistryData {
    schema_version: u32,
    identity: String,
    generation: u64,
    capacity: usize,
    models: Vec<String>,
}
impl ModelRegistry {
    pub fn new(identity: impl Into<String>, capacity: usize) -> Result<Self, PipelineError> {
        let identity = identity.into();
        if !label(&identity, 128) || !(2..=32).contains(&capacity) {
            return Err(PipelineError::Invalid("registry identity/capacity"));
        }
        Ok(Self {
            identity,
            generation: 0,
            capacity,
            models: Vec::new(),
        })
    }
    pub fn current(&self) -> Option<&LinearModel> {
        self.models.last()
    }
    pub fn revision(&self) -> Result<RegistryRevision, PipelineError> {
        Ok(RegistryRevision {
            identity: self.identity.clone(),
            generation: self.generation,
            model: self.current().map(LinearModel::fingerprint).transpose()?,
        })
    }
    pub fn promote(
        &mut self,
        expected: &RegistryRevision,
        model: LinearModel,
        baseline: &EvaluationRun,
        candidate: &EvaluationRun,
        gate: &GateReport,
    ) -> Result<RegistryRevision, PipelineError> {
        if &self.revision()? != expected {
            return Err(PipelineError::Conflict);
        }
        let model_fp = model.fingerprint()?;
        let recipe = format!("airust-linear-v1/{model_fp}");
        let current_fp = self.current().map(LinearModel::fingerprint).transpose()?;
        if gate.verdict() != GateVerdict::Passed
            || gate.baseline_fingerprint() != baseline.fingerprint()?
            || gate.candidate_fingerprint() != candidate.fingerprint()?
            || candidate.recipe() != recipe
            || current_fp
                .as_ref()
                .is_some_and(|fp| baseline.recipe() != format!("airust-linear-v1/{fp}"))
        {
            return Err(PipelineError::NotApproved);
        }
        if current_fp.as_deref() == Some(model_fp.as_str()) {
            return self.revision();
        }
        let next = self
            .generation
            .checked_add(1)
            .ok_or(PipelineError::Exhausted)?;
        self.models.push(model);
        if self.models.len() > self.capacity {
            self.models.remove(0);
        }
        self.generation = next;
        self.revision()
    }
    pub fn rollback(
        &mut self,
        expected: &RegistryRevision,
    ) -> Result<RegistryRevision, PipelineError> {
        if &self.revision()? != expected {
            return Err(PipelineError::Conflict);
        }
        if self.models.len() < 2 {
            return Err(PipelineError::NoPrevious);
        }
        let next = self
            .generation
            .checked_add(1)
            .ok_or(PipelineError::Exhausted)?;
        self.models.pop();
        self.generation = next;
        self.revision()
    }
    pub fn to_json(&self) -> Result<String, PipelineError> {
        let models = self
            .models
            .iter()
            .map(LinearModel::to_json)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(serde_json::to_string(&RegistryData {
            schema_version: 1,
            identity: self.identity.clone(),
            generation: self.generation,
            capacity: self.capacity,
            models,
        })?)
    }
    pub fn from_json(json: &str) -> Result<Self, PipelineError> {
        if json.len() > 32 * 1024 * 1024 {
            return Err(PipelineError::Budget);
        }
        let d: RegistryData = serde_json::from_str(json).map_err(crate::input::json_error)?;
        let mut registry = Self::new(d.identity, d.capacity)?;
        if d.schema_version != 1
            || d.models.len() > d.capacity
            || d.generation < (d.models.len() as u64)
        {
            return Err(PipelineError::Invalid("registry state"));
        }
        registry.models = d
            .models
            .iter()
            .map(|m| LinearModel::from_json(m))
            .collect::<Result<Vec<_>, _>>()?;
        registry.generation = d.generation;
        Ok(registry)
    }
}
fn digest(data: &[u8]) -> String {
    let mut hash = Sha256::new();
    hash.update(b"airust-training-pipeline-v1");
    hash.update(data);
    format!("{:x}", hash.finalize())
}
fn label(s: &str, n: usize) -> bool {
    !s.trim().is_empty() && s.len() <= n && !s.chars().any(char::is_control)
}
fn hash_valid(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

#[derive(Debug, Clone)]
pub struct GroupedTraining {
    model: LinearModel,
    dataset: String,
    training_indices: Vec<usize>,
    holdout_indices: Vec<usize>,
    evaluation: crate::learned_ranking::Evaluation,
}
impl GroupedTraining {
    pub fn model(&self) -> &LinearModel {
        &self.model
    }
    pub fn into_model(self) -> LinearModel {
        self.model
    }
    pub fn dataset_fingerprint(&self) -> &str {
        &self.dataset
    }
    pub fn training_indices(&self) -> &[usize] {
        &self.training_indices
    }
    pub fn holdout_indices(&self) -> &[usize] {
        &self.holdout_indices
    }
    pub fn evaluation(&self) -> &crate::learned_ranking::Evaluation {
        &self.evaluation
    }
}
pub fn train_grouped(
    dataset: &TrainingDataset,
    features: &mut dyn crate::learned_ranking::FeatureExtractor,
    options: &crate::learned_ranking::TrainingOptions,
    holdout_percent: u8,
    seed: u64,
) -> Result<GroupedTraining, PipelineError> {
    use crate::learned_ranking::{evaluate, split_grouped, train};
    let (training_indices, holdout_indices) =
        split_grouped(dataset.examples(), holdout_percent, seed)?;
    let training = training_indices
        .iter()
        .map(|i| dataset.examples[*i].clone())
        .collect::<Vec<_>>();
    let holdout = holdout_indices
        .iter()
        .map(|i| dataset.examples[*i].clone())
        .collect::<Vec<_>>();
    let model = train(&training, features, options)?;
    let evaluation = evaluate(&model, &holdout, features)?;
    Ok(GroupedTraining {
        model,
        dataset: dataset.fingerprint()?,
        training_indices,
        holdout_indices,
        evaluation,
    })
}
