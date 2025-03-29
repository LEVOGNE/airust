use crate::agent::{TrainableAgent, TrainingExample};
use strsim::levenshtein;

pub struct FuzzyAgent {
    memory: Vec<TrainingExample>,
}

impl FuzzyAgent {
    pub fn new() -> Self {
        Self { memory: Vec::new() }
    }
}

impl TrainableAgent for FuzzyAgent {
    fn train(&mut self, data: &[TrainingExample]) {
        self.memory = data.to_vec();
    }

    fn predict(&self, input: &str) -> String {
        let mut best_score = usize::MAX;
        let mut best_output = "Keine Antwort gefunden.".to_string();

        for item in &self.memory {
            let score = levenshtein(&item.input.to_lowercase(), &input.to_lowercase());
            if score < best_score {
                best_score = score;
                best_output = item.output.clone();
            }
        }

        best_output
    }
}
