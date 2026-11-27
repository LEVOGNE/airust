#![doc = include_str!("../docs/LEARNED_RANKING.md")]
use crate::reranking::{RerankError, Reranker};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use thiserror::Error;
macro_rules! validated_deserialize {
    ($name:ident { $($field:ident: $kind:ty),* $(,)? }) => {
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                #[derive(Deserialize)]
                #[serde(deny_unknown_fields)]
                struct Fields { $($field: $kind),* }
                let fields = Fields::deserialize(deserializer)?;
                let value = Self { $($field: fields.$field),* };
                value.validate().map_err(serde::de::Error::custom)?;
                Ok(value)
            }
        }
    };
}
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum LearningError {
    #[error("invalid learning configuration/data: {0}")]
    Invalid(&'static str),
    #[error("learning budget exceeded: {0}")]
    Budget(&'static str),
    #[cfg(feature = "execution")]
    #[error(transparent)]
    Execution(#[from] crate::execution::ExecutionError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureSchema {
    recipe: String,
    names: Vec<String>,
}
validated_deserialize!(FeatureSchema { recipe: String, names: Vec<String> });
impl FeatureSchema {
    pub fn new(recipe: impl Into<String>, names: Vec<String>) -> Result<Self, LearningError> {
        let value = Self {
            recipe: recipe.into(),
            names,
        };
        value.validate()?;
        Ok(value)
    }
    pub fn recipe(&self) -> &str {
        &self.recipe
    }
    pub fn names(&self) -> &[String] {
        &self.names
    }
    fn validate(&self) -> Result<(), LearningError> {
        if !crate::text::Identifier::label(&self.recipe, 256)
            || self.names.is_empty()
            || self.names.len() > 64
            || self
                .names
                .iter()
                .any(|s| !crate::text::Identifier::label(s, 64))
            || self.names.iter().collect::<BTreeSet<_>>().len() != self.names.len()
        {
            return Err(LearningError::Invalid("feature schema"));
        }
        Ok(())
    }
}
pub trait FeatureExtractor {
    fn schema(&self) -> FeatureSchema;
    fn extract(&mut self, query: &str, document: &str) -> Result<Vec<f64>, LearningError>;
}
#[derive(Debug, Default, Clone, Copy)]
pub struct LexicalFeatures;
impl FeatureExtractor for LexicalFeatures {
    fn schema(&self) -> FeatureSchema {
        FeatureSchema {
            recipe: "airust-lexical-features-v1".into(),
            names: [
                "bias",
                "query_coverage",
                "document_density",
                "exact_phrase",
                "length_balance",
                "ordered_bigram_coverage",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        }
    }
    fn extract(&mut self, query: &str, document: &str) -> Result<Vec<f64>, LearningError> {
        validate_pair(query, document)?;
        let q = crate::text::normalize(query);
        let d = crate::text::normalize(document);
        if q.len() > 65536 || d.len() > 262144 {
            return Err(LearningError::Budget("normalized text"));
        }
        let qt = crate::text::tokenize(&q);
        let dt = crate::text::tokenize(&d);
        let qs = qt.iter().collect::<BTreeSet<_>>();
        let ds = dt.iter().collect::<BTreeSet<_>>();
        let common = qs.intersection(&ds).count() as f64;
        let pairs = qt.windows(2).collect::<Vec<_>>();
        let document_pairs = dt.windows(2).collect::<BTreeSet<_>>();
        let found = pairs
            .iter()
            .filter(|p| document_pairs.contains(**p))
            .count();
        Ok(vec![
            1.,
            common / qs.len().max(1) as f64,
            common / ds.len().max(1) as f64,
            f64::from(!q.is_empty() && d.contains(&q)),
            qt.len().min(dt.len()) as f64 / qt.len().max(dt.len()).max(1) as f64,
            if pairs.is_empty() {
                0.
            } else {
                found as f64 / pairs.len() as f64
            },
        ])
    }
}
#[derive(Debug, Clone)]
pub struct TrainingExample {
    query: String,
    document: String,
    relevant: bool,
    group: String,
}
impl TrainingExample {
    pub fn new(
        query: impl Into<String>,
        document: impl Into<String>,
        relevant: bool,
    ) -> Result<Self, LearningError> {
        let query = query.into();
        let document = document.into();
        validate_pair(&query, &document)?;
        let group = hash(&[b"query-group-v1", crate::text::normalize(&query).as_bytes()]);
        Ok(Self {
            query,
            document,
            relevant,
            group,
        })
    }
    pub fn with_group(mut self, group: impl Into<String>) -> Result<Self, LearningError> {
        self.group = group.into();
        if !crate::text::Identifier::label(&self.group, 256) {
            return Err(LearningError::Invalid("example group"));
        }
        Ok(self)
    }
    pub fn query(&self) -> &str {
        &self.query
    }
    pub fn document(&self) -> &str {
        &self.document
    }
    pub fn relevant(&self) -> bool {
        self.relevant
    }
    pub fn group(&self) -> &str {
        &self.group
    }
}
#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TrainingConfig {
    epochs: usize,
    learning_rate: f64,
    l2: f64,
    gradient_clip: f64,
}
validated_deserialize!(TrainingConfig {
    epochs: usize,
    learning_rate: f64,
    l2: f64,
    gradient_clip: f64
});
impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            epochs: 100,
            learning_rate: 0.2,
            l2: 0.01,
            gradient_clip: 5.,
        }
    }
}
impl TrainingConfig {
    pub fn new(
        epochs: usize,
        learning_rate: f64,
        l2: f64,
        gradient_clip: f64,
    ) -> Result<Self, LearningError> {
        let c = Self {
            epochs,
            learning_rate,
            l2,
            gradient_clip,
        };
        c.validate()?;
        Ok(c)
    }
    pub fn epochs(&self) -> usize {
        self.epochs
    }
    pub fn learning_rate(&self) -> f64 {
        self.learning_rate
    }
    pub fn l2(&self) -> f64 {
        self.l2
    }
    pub fn gradient_clip(&self) -> f64 {
        self.gradient_clip
    }
    fn validate(&self) -> Result<(), LearningError> {
        if !(1..=200).contains(&self.epochs)
            || !self.learning_rate.is_finite()
            || self.learning_rate <= 0.
            || self.learning_rate > 1.
            || !self.l2.is_finite()
            || !(0.0..=10.0).contains(&self.l2)
            || !self.gradient_clip.is_finite()
            || self.gradient_clip <= 0.
            || self.gradient_clip > 100.
        {
            return Err(LearningError::Invalid("training parameters"));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Default)]
pub struct TrainingOptions {
    config: TrainingConfig,
    #[cfg(feature = "execution")]
    control: Option<crate::execution::WorkControl>,
}
impl TrainingOptions {
    pub fn new(config: TrainingConfig) -> Self {
        Self {
            config,
            #[cfg(feature = "execution")]
            control: None,
        }
    }
    #[cfg(feature = "execution")]
    pub fn with_control(mut self, control: crate::execution::WorkControl) -> Self {
        self.control = Some(control);
        self
    }
    fn check(&self) -> Result<(), LearningError> {
        #[cfg(feature = "execution")]
        if let Some(c) = &self.control {
            c.checkpoint()?;
        }
        Ok(())
    }
    fn progress(&self, training: bool, completed: usize, total: usize) {
        #[cfg(feature = "execution")]
        if let Some(c) = &self.control {
            use crate::execution::{ProgressEvent, WorkPhase};
            c.report(
                ProgressEvent::new(
                    if training {
                        WorkPhase::Training
                    } else {
                        WorkPhase::Features
                    },
                    completed,
                    Some(total),
                )
                .expect("bounded phase count"),
            );
        }
        #[cfg(not(feature = "execution"))]
        let _ = (training, completed, total);
    }
}
#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TrainingReport {
    examples: usize,
    positive: usize,
    manifest: String,
    initial_loss: f64,
    final_loss: f64,
    config: TrainingConfig,
}
validated_deserialize!(TrainingReport {
    examples: usize,
    positive: usize,
    manifest: String,
    initial_loss: f64,
    final_loss: f64,
    config: TrainingConfig
});
impl TrainingReport {
    fn validate(&self) -> Result<(), LearningError> {
        self.config.validate()?;
        if self.examples == 0
            || self.examples > 100000
            || self.positive == 0
            || self.positive >= self.examples
            || !crate::text::Identifier::sha256(&self.manifest)
            || ![self.initial_loss, self.final_loss]
                .iter()
                .all(|v| v.is_finite() && *v >= 0.)
        {
            return Err(LearningError::Invalid("training report"));
        }
        Ok(())
    }
    pub fn examples(&self) -> usize {
        self.examples
    }
    pub fn positive(&self) -> usize {
        self.positive
    }
    pub fn manifest(&self) -> &str {
        &self.manifest
    }
    pub fn initial_loss(&self) -> f64 {
        self.initial_loss
    }
    pub fn final_loss(&self) -> f64 {
        self.final_loss
    }
    pub fn config(&self) -> &TrainingConfig {
        &self.config
    }
}
#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LinearModel {
    schema_version: u32,
    schema: FeatureSchema,
    weights: Vec<f64>,
    report: TrainingReport,
}
validated_deserialize!(LinearModel { schema_version: u32, schema: FeatureSchema, weights: Vec<f64>, report: TrainingReport });
impl LinearModel {
    pub fn schema(&self) -> &FeatureSchema {
        &self.schema
    }
    pub fn weights(&self) -> &[f64] {
        &self.weights
    }
    pub fn report(&self) -> &TrainingReport {
        &self.report
    }
    pub fn to_json(&self) -> Result<String, LearningError> {
        self.validate()?;
        Ok(serde_json::to_string(self)?)
    }
    pub fn from_json(json: &str) -> Result<Self, LearningError> {
        if json.len() > 1024 * 1024 {
            return Err(LearningError::Budget("model JSON"));
        }
        let model: Self = serde_json::from_str(json).map_err(crate::input::json_error)?;
        model.validate()?;
        Ok(model)
    }
    pub fn fingerprint(&self) -> Result<String, LearningError> {
        Ok(hash(&[
            b"airust-linear-model-v1",
            self.to_json()?.as_bytes(),
        ]))
    }
    fn validate(&self) -> Result<(), LearningError> {
        self.schema.validate()?;
        self.report.validate()?;
        if self.schema_version != 1
            || self.weights.len() != self.schema.names.len()
            || self.weights.iter().any(|w| !w.is_finite() || w.abs() > 1e6)
        {
            return Err(LearningError::Invalid("model artifact"));
        }
        Ok(())
    }
}
pub fn train(
    examples: &[TrainingExample],
    extractor: &mut dyn FeatureExtractor,
    options: &TrainingOptions,
) -> Result<LinearModel, LearningError> {
    options.config.validate()?;
    options.check()?;
    validate_examples(examples)?;
    let positive = examples.iter().filter(|e| e.relevant).count();
    if positive == 0 || positive == examples.len() {
        return Err(LearningError::Invalid("training requires both labels"));
    }
    let schema = extractor.schema();
    schema.validate()?;
    let mut rows = Vec::with_capacity(examples.len());
    let mut manifest = Sha256::new();
    manifest.update(b"airust-training-v1");
    options.progress(false, 0, examples.len());
    for (i, e) in examples.iter().enumerate() {
        options.check()?;
        let values = features(extractor, &schema, &e.query, &e.document)?;
        for p in [
            e.query.as_bytes(),
            e.document.as_bytes(),
            e.group.as_bytes(),
            &[u8::from(e.relevant)],
        ] {
            manifest.update((p.len() as u64).to_le_bytes());
            manifest.update(p);
        }
        rows.push((values, f64::from(e.relevant)));
        if i % 64 == 0 || i + 1 == examples.len() {
            options.progress(false, i + 1, examples.len());
        }
    }
    let mut weights = vec![0.; schema.names.len()];
    let initial_loss = mean_loss(&rows, &weights);
    let cfg = &options.config;
    options.progress(true, 0, cfg.epochs);
    for epoch in 0..cfg.epochs {
        options.check()?;
        let mut grad = vec![0.; weights.len()];
        for (i, (x, y)) in rows.iter().enumerate() {
            if i % 256 == 0 {
                options.check()?;
            }
            let diff = sigmoid(dot(x, &weights)) - *y;
            for (g, v) in grad.iter_mut().zip(x) {
                *g += diff * v;
            }
        }
        for (g, w) in grad.iter_mut().zip(&weights) {
            *g = *g / rows.len() as f64 + cfg.l2 * w;
            *g = g.clamp(-cfg.gradient_clip, cfg.gradient_clip);
        }
        for (w, g) in weights.iter_mut().zip(grad) {
            *w -= cfg.learning_rate * g;
            if !w.is_finite() || w.abs() > 1e6 {
                return Err(LearningError::Invalid("divergent weights"));
            }
        }
        options.progress(true, epoch + 1, cfg.epochs);
    }
    options.check()?;
    let final_loss = mean_loss(&rows, &weights);
    let model = LinearModel {
        schema_version: 1,
        schema,
        weights,
        report: TrainingReport {
            examples: examples.len(),
            positive,
            manifest: format!("{:x}", manifest.finalize()),
            initial_loss,
            final_loss,
            config: cfg.clone(),
        },
    };
    model.validate()?;
    Ok(model)
}
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct Evaluation {
    pub examples: usize,
    pub loss: f64,
    pub correct_at_zero: usize,
}
pub fn evaluate(
    model: &LinearModel,
    examples: &[TrainingExample],
    extractor: &mut dyn FeatureExtractor,
) -> Result<Evaluation, LearningError> {
    model.validate()?;
    validate_examples(examples)?;
    let mut loss = 0.;
    let mut correct = 0;
    for e in examples {
        let x = features(extractor, &model.schema, &e.query, &e.document)?;
        let z = dot(&x, &model.weights);
        loss += logistic_loss(z, f64::from(e.relevant));
        correct += usize::from((z >= 0.) == e.relevant);
    }
    Ok(Evaluation {
        examples: examples.len(),
        loss: loss / examples.len() as f64,
        correct_at_zero: correct,
    })
}
pub fn split_grouped(
    examples: &[TrainingExample],
    holdout_percent: u8,
    seed: u64,
) -> Result<(Vec<usize>, Vec<usize>), LearningError> {
    validate_examples(examples)?;
    if !(1..=99).contains(&holdout_percent) {
        return Err(LearningError::Invalid("holdout percentage"));
    }
    let mut train = Vec::new();
    let mut hold = Vec::new();
    for (i, e) in examples.iter().enumerate() {
        let d = hash(&[
            b"airust-holdout-v1",
            &seed.to_le_bytes(),
            e.group.as_bytes(),
        ]);
        let bucket = u64::from_str_radix(&d[..16], 16).expect("valid digest") % 100;
        if bucket < u64::from(holdout_percent) {
            hold.push(i)
        } else {
            train.push(i)
        }
    }
    if train.is_empty() || hold.is_empty() {
        return Err(LearningError::Invalid("split has an empty partition"));
    }
    Ok((train, hold))
}
pub struct LinearReranker<'a> {
    model: &'a LinearModel,
    extractor: &'a mut dyn FeatureExtractor,
    recipe: String,
}
impl<'a> LinearReranker<'a> {
    pub fn new(
        model: &'a LinearModel,
        extractor: &'a mut dyn FeatureExtractor,
    ) -> Result<Self, LearningError> {
        model.validate()?;
        if extractor.schema() != model.schema {
            return Err(LearningError::Invalid("feature schema mismatch"));
        }
        Ok(Self {
            model,
            extractor,
            recipe: format!("airust-linear-v1/{}", model.fingerprint()?),
        })
    }
}
impl Reranker for LinearReranker<'_> {
    fn recipe(&self) -> &str {
        &self.recipe
    }
    fn score(&mut self, q: &str, docs: &[&str]) -> Result<Vec<f64>, RerankError> {
        if docs.len() > 1000 {
            return Err(RerankError::Budget("learned document count"));
        }
        let mut result = Vec::with_capacity(docs.len());
        for d in docs {
            let x = features(self.extractor, &self.model.schema, q, d)
                .map_err(|_| RerankError::Output("invalid learned features/schema/input"))?;
            result.push(dot(&x, &self.model.weights));
        }
        Ok(result)
    }
}
fn features(
    extractor: &mut dyn FeatureExtractor,
    schema: &FeatureSchema,
    q: &str,
    d: &str,
) -> Result<Vec<f64>, LearningError> {
    validate_pair(q, d)?;
    if extractor.schema() != *schema {
        return Err(LearningError::Invalid("feature schema changed"));
    }
    let values = extractor.extract(q, d);
    if extractor.schema() != *schema {
        return Err(LearningError::Invalid("feature schema changed"));
    }
    let values = values?;
    if values.len() != schema.names.len() || values.iter().any(|v| !v.is_finite() || v.abs() > 1.) {
        return Err(LearningError::Invalid("feature count/range"));
    }
    Ok(values)
}
fn validate_examples(examples: &[TrainingExample]) -> Result<(), LearningError> {
    if examples.is_empty() || examples.len() > 100000 {
        return Err(LearningError::Budget("example count"));
    }
    let mut bytes = 0usize;
    for e in examples {
        validate_pair(&e.query, &e.document)?;
        bytes = bytes
            .checked_add(e.query.len() + e.document.len() + e.group.len())
            .ok_or(LearningError::Budget("training bytes"))?;
        if bytes > 64 * 1024 * 1024 {
            return Err(LearningError::Budget("training bytes"));
        }
    }
    Ok(())
}
fn validate_pair(q: &str, d: &str) -> Result<(), LearningError> {
    if q.trim().is_empty() || d.trim().is_empty() {
        return Err(LearningError::Invalid("blank query/document"));
    }
    if q.len() > 16384 || d.len() > 65536 {
        return Err(LearningError::Budget("query/document bytes"));
    }
    Ok(())
}
fn dot(x: &[f64], w: &[f64]) -> f64 {
    x.iter().zip(w).map(|(x, w)| x * w).sum()
}
fn sigmoid(z: f64) -> f64 {
    if z >= 0. {
        1. / (1. + (-z).exp())
    } else {
        let e = z.exp();
        e / (1. + e)
    }
}
fn logistic_loss(z: f64, y: f64) -> f64 {
    z.max(0.) - y * z + (-z.abs()).exp().ln_1p()
}
fn mean_loss(rows: &[(Vec<f64>, f64)], w: &[f64]) -> f64 {
    rows.iter()
        .map(|(x, y)| logistic_loss(dot(x, w), *y))
        .sum::<f64>()
        / rows.len() as f64
}
fn hash(parts: &[&[u8]]) -> String {
    let mut h = Sha256::new();
    for p in parts {
        h.update((p.len() as u64).to_le_bytes());
        h.update(p);
    }
    format!("{:x}", h.finalize())
}
#[cfg(feature = "rank-explanations")]
impl crate::rank_explanations::ExplainableReranker for LinearReranker<'_> {
    fn score_explained(
        &mut self,
        query: &str,
        documents: &[&str],
        options: &crate::rank_explanations::ExplanationOptions,
    ) -> Result<crate::rank_explanations::ExplainedScores, RerankError> {
        use crate::rank_explanations::*;
        validate_inputs(query, documents, options)?;
        let mut all = Vec::with_capacity(documents.len());
        for document in documents {
            let values = features(self.extractor, &self.model.schema, query, document)
                .map_err(|_| RerankError::Output("invalid explained learned features"))?;
            let components = self
                .model
                .schema
                .names
                .iter()
                .zip(values)
                .zip(&self.model.weights)
                .map(|((name, value), weight)| ScoreComponent::new(name, value, *weight))
                .collect::<Result<Vec<_>, _>>()?;
            all.push(components);
        }
        ExplainedScores::from_components(self.recipe(), query, documents, all, options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stable_math_and_gradient_match_finite_differences() {
        for z in [-1000., -5., 0., 5., 1000.] {
            for y in [0., 1.] {
                assert!(logistic_loss(z, y).is_finite());
                let eps = 1e-4;
                let finite = (logistic_loss(z + eps, y) - logistic_loss(z - eps, y)) / (2. * eps);
                assert!((finite - (sigmoid(z) - y)).abs() < 1e-6);
            }
        }
    }
}
