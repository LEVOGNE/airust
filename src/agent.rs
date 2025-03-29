use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct TrainingExample {
    pub input: String,
    pub output: String,
    #[serde(default = "default_weight")]
    pub weight: f32,
}

// Standardgewicht für abwärtskompatibilität
// Änderung: Funktion als pub markieren
pub fn default_weight() -> f32 {
    1.0
}

pub trait TrainableAgent {
    fn train(&mut self, data: &[TrainingExample]);
    fn predict(&self, input: &str) -> String;
}
