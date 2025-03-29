use crate::agent::{TrainableAgent, TrainingExample};

pub struct SimpleAgent {
    memory: Vec<TrainingExample>,
}

impl SimpleAgent {
    pub fn new() -> Self {
        Self { memory: Vec::new() }
    }
}

impl TrainableAgent for SimpleAgent {
    fn train(&mut self, data: &[TrainingExample]) {
        self.memory = data.to_vec();
    }

    fn predict(&self, input: &str) -> String {
        for item in &self.memory {
            if item.input.to_lowercase() == input.to_lowercase() {
                return item.output.clone();
            }
        }
        "Keine Antwort gefunden.".to_string()
    }
}
