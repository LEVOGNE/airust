use crate::agent::{default_weight, TrainableAgent, TrainingExample};
use crate::tfidf_agent::TfidfAgent;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ResponseFormat {
    Text(String),
    Markdown(String),
    Json(Value),
}

impl Default for ResponseFormat {
    fn default() -> Self {
        ResponseFormat::Text(String::new())
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct StructuredTrainingExample {
    pub input: String,
    pub output: ResponseFormat,
    #[serde(default = "default_weight")]
    pub weight: f32,
}

// Conversion between TrainingExample and StructuredTrainingExample
impl From<TrainingExample> for StructuredTrainingExample {
    fn from(example: TrainingExample) -> Self {
        Self {
            input: example.input,
            output: ResponseFormat::Text(example.output),
            weight: example.weight,
        }
    }
}

impl From<StructuredTrainingExample> for TrainingExample {
    fn from(example: StructuredTrainingExample) -> Self {
        let output = match &example.output {
            ResponseFormat::Text(text) => text.clone(),
            ResponseFormat::Markdown(md) => md.clone(),
            ResponseFormat::Json(json) => json.to_string(),
        };

        Self {
            input: example.input,
            output,
            weight: example.weight,
        }
    }
}

pub struct StructuredAgent {
    base_agent: TfidfAgent,
    structured_memory: Vec<StructuredTrainingExample>,
}

impl StructuredAgent {
    pub fn new() -> Self {
        Self {
            base_agent: TfidfAgent::new(),
            structured_memory: Vec::new(),
        }
    }

    // Additional method to get structured response
    pub fn predict_structured(&self, input: &str) -> ResponseFormat {
        let plain_answer = self.predict(input);

        // Try to find the corresponding structured answer
        for example in &self.structured_memory {
            let example_simple: TrainingExample = example.clone().into();
            if example_simple.output == plain_answer {
                return example.output.clone();
            }
        }

        // Fallback: Text answer
        ResponseFormat::Text(plain_answer)
    }
}

impl TrainableAgent for StructuredAgent {
    fn train(&mut self, data: &[TrainingExample]) {
        self.base_agent.train(data);

        // Convert to structured examples
        self.structured_memory = data
            .iter()
            .map(|ex| StructuredTrainingExample::from(ex.clone()))
            .collect();
    }

    fn predict(&self, input: &str) -> String {
        self.base_agent.predict(input)
    }
}
