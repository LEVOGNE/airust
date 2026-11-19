#![doc = include_str!("../docs/RANK-EXPLANATIONS.md")]
use crate::{
    feedback::FeedbackReranker,
    reranking::{RerankError, Reranker},
};
use serde::Serialize;
use sha2::{Digest, Sha256};
#[derive(Debug, Clone)]
pub struct ExplanationOptions {
    max_documents: usize,
    max_json_bytes: usize,
}
impl Default for ExplanationOptions {
    fn default() -> Self {
        Self {
            max_documents: 1000,
            max_json_bytes: 1024 * 1024,
        }
    }
}
impl ExplanationOptions {
    pub fn new(documents: usize, json_bytes: usize) -> Result<Self, RerankError> {
        if !(1..=1000).contains(&documents) || !(1024..=16 * 1024 * 1024).contains(&json_bytes) {
            return Err(RerankError::Configuration("explanation limits"));
        }
        Ok(Self {
            max_documents: documents,
            max_json_bytes: json_bytes,
        })
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct ScoreComponent {
    name: String,
    value: f64,
    weight: f64,
    contribution: f64,
}
impl ScoreComponent {
    pub fn new(name: impl Into<String>, value: f64, weight: f64) -> Result<Self, RerankError> {
        let name = name.into();
        let contribution = value * weight;
        if name.trim().is_empty()
            || name.len() > 128
            || name.chars().any(char::is_control)
            || ![value, weight, contribution].iter().all(|v| v.is_finite())
        {
            return Err(RerankError::Output("invalid explanation component"));
        }
        Ok(Self {
            name,
            value,
            weight,
            contribution,
        })
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn value(&self) -> f64 {
        self.value
    }
    pub fn weight(&self) -> f64 {
        self.weight
    }
    pub fn contribution(&self) -> f64 {
        self.contribution
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct DocumentExplanation {
    input_position: usize,
    document_fingerprint: String,
    score: f64,
    components: Vec<ScoreComponent>,
}
impl DocumentExplanation {
    pub fn input_position(&self) -> usize {
        self.input_position
    }
    pub fn document_fingerprint(&self) -> &str {
        &self.document_fingerprint
    }
    pub fn score(&self) -> f64 {
        self.score
    }
    pub fn components(&self) -> &[ScoreComponent] {
        &self.components
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct ExplainedScores {
    schema_version: u32,
    recipe: String,
    query_fingerprint: String,
    documents: Vec<DocumentExplanation>,
}
impl ExplainedScores {
    pub fn from_components(
        recipe: impl Into<String>,
        query: &str,
        documents: &[&str],
        components: Vec<Vec<ScoreComponent>>,
        options: &ExplanationOptions,
    ) -> Result<Self, RerankError> {
        validate_inputs(query, documents, options)?;
        let recipe = recipe.into();
        if recipe.trim().is_empty() || recipe.len() > 1024 || recipe.chars().any(char::is_control) {
            return Err(RerankError::InvalidRecipe);
        }
        if documents.len() != components.len() {
            return Err(RerankError::ScoreCount {
                expected: documents.len(),
                actual: components.len(),
            });
        }
        let mut output = Vec::with_capacity(documents.len());
        for (i, (document, components)) in documents.iter().zip(components).enumerate() {
            if components.is_empty() || components.len() > 64 {
                return Err(RerankError::Output("explanation component count"));
            }
            let mut names = std::collections::BTreeSet::new();
            if components.iter().any(|c| !names.insert(&c.name)) {
                return Err(RerankError::Output("duplicate explanation component"));
            }
            let score = components.iter().map(|c| c.contribution).sum::<f64>();
            if !score.is_finite() {
                return Err(RerankError::NonFiniteScore(i + 1));
            }
            output.push(DocumentExplanation {
                input_position: i + 1,
                document_fingerprint: fingerprint(b"document", document),
                score,
                components,
            });
        }
        let result = Self {
            schema_version: 1,
            recipe,
            query_fingerprint: fingerprint(b"query", query),
            documents: output,
        };
        if result.to_json()?.len() > options.max_json_bytes {
            return Err(RerankError::Budget("explanation JSON"));
        }
        Ok(result)
    }
    pub fn documents(&self) -> &[DocumentExplanation] {
        &self.documents
    }
    pub fn recipe(&self) -> &str {
        &self.recipe
    }
    pub fn scores(&self) -> Vec<f64> {
        self.documents.iter().map(|d| d.score).collect()
    }
    pub fn to_json(&self) -> Result<String, RerankError> {
        serde_json::to_string(self).map_err(|_| RerankError::Output("explanation JSON"))
    }
}
pub trait ExplainableReranker: Reranker {
    fn score_explained(
        &mut self,
        query: &str,
        documents: &[&str],
        options: &ExplanationOptions,
    ) -> Result<ExplainedScores, RerankError>;
}
pub(crate) fn validate_inputs(
    query: &str,
    documents: &[&str],
    options: &ExplanationOptions,
) -> Result<(), RerankError> {
    if query.trim().is_empty() {
        return Err(RerankError::EmptyQuery);
    }
    if query.len() > 16384 || documents.len() > options.max_documents {
        return Err(RerankError::Budget("explanation query/documents"));
    }
    let mut bytes = 0usize;
    for d in documents {
        if d.trim().is_empty() {
            return Err(RerankError::Output("empty explanation document"));
        }
        if d.len() > 65536 {
            return Err(RerankError::Budget("explanation document"));
        }
        bytes = bytes
            .checked_add(d.len())
            .filter(|n| *n <= 16 * 1024 * 1024)
            .ok_or(RerankError::Budget("explanation input bytes"))?;
    }
    Ok(())
}
impl ExplainableReranker for FeedbackReranker<'_> {
    fn score_explained(
        &mut self,
        query: &str,
        documents: &[&str],
        options: &ExplanationOptions,
    ) -> Result<ExplainedScores, RerankError> {
        validate_inputs(query, documents, options)?;
        let scores = self.score(query, documents)?;
        let components = self
            .last_adjustments()
            .iter()
            .map(|a| {
                Ok(vec![
                    ScoreComponent::new("base_score", a.base_score, 1.)?,
                    ScoreComponent::new("feedback_boost", a.boost, 1.)?,
                ])
            })
            .collect::<Result<Vec<_>, RerankError>>()?;
        let result =
            ExplainedScores::from_components(self.recipe(), query, documents, components, options)?;
        if result.scores() != scores {
            return Err(RerankError::Output("feedback explanation score mismatch"));
        }
        Ok(result)
    }
}
fn fingerprint(domain: &[u8], text: &str) -> String {
    let mut hash = Sha256::new();
    hash.update(b"airust-score-explanation-v1");
    hash.update((domain.len() as u64).to_le_bytes());
    hash.update(domain);
    hash.update((text.len() as u64).to_le_bytes());
    hash.update(text.as_bytes());
    format!("{:x}", hash.finalize())
}
