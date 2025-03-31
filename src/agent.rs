// src/agent.rs - New trait hierarchy
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ResponseFormat {
    Text(String),
    Markdown(String),
    Json(serde_json::Value),
}

impl Default for ResponseFormat {
    fn default() -> Self {
        ResponseFormat::Text(String::new())
    }
}

// Conversion from ResponseFormat to String for backward compatibility
impl From<ResponseFormat> for String {
    fn from(format: ResponseFormat) -> Self {
        match format {
            ResponseFormat::Text(text) => text,
            ResponseFormat::Markdown(md) => md,
            ResponseFormat::Json(json) => json.to_string(),
        }
    }
}

impl From<String> for ResponseFormat {
    fn from(text: String) -> Self {
        ResponseFormat::Text(text)
    }
}

impl From<&str> for ResponseFormat {
    fn from(text: &str) -> Self {
        ResponseFormat::Text(text.to_string())
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct TrainingExample {
    pub input: String,
    pub output: ResponseFormat,
    #[serde(default = "default_weight")]
    pub weight: f32,
}

// For backward compatibility
#[derive(Clone, Deserialize, Serialize)]
pub struct LegacyTrainingExample {
    pub input: String,
    pub output: String,
    #[serde(default = "default_weight")]
    pub weight: f32,
}

impl From<LegacyTrainingExample> for TrainingExample {
    fn from(legacy: LegacyTrainingExample) -> Self {
        Self {
            input: legacy.input,
            output: ResponseFormat::Text(legacy.output),
            weight: legacy.weight,
        }
    }
}

// Default weight for backward compatibility
pub fn default_weight() -> f32 {
    1.0
}

// Base agent trait
pub trait Agent {
    fn predict(&self, input: &str) -> ResponseFormat;

    // Default implementation for backward compatibility
    fn predict_text(&self, input: &str) -> String {
        self.predict(input).into()
    }
}

// Extended trait for trainable agents
pub trait TrainableAgent: Agent {
    fn train(&mut self, data: &[TrainingExample]);

    // Helper method for training with legacy data
    fn train_legacy(&mut self, data: &[LegacyTrainingExample]) {
        let converted: Vec<TrainingExample> = data
            .iter()
            .map(|ex| TrainingExample {
                input: ex.input.clone(),
                output: ResponseFormat::Text(ex.output.clone()),
                weight: ex.weight,
            })
            .collect();

        self.train(&converted);
    }
}

// Trait for agents with context support
pub trait ContextualAgent: Agent {
    fn add_context(&mut self, question: String, answer: ResponseFormat);

    // Helper method for text answers
    fn add_text_context(&mut self, question: String, answer: String) {
        self.add_context(question, ResponseFormat::Text(answer));
    }
}

// Common text processing functions
pub mod text_utils {
    use std::collections::HashSet;

    // Tokenizes text into words
    pub fn tokenize(text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }

    // Finds unique terms in a text
    pub fn unique_terms(text: &str) -> HashSet<String> {
        tokenize(text).into_iter().collect()
    }
}
