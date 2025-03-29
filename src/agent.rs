use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct TrainingExample {
    pub input: String,
    pub output: String,
}

pub trait TrainableAgent {
    fn train(&mut self, data: &[TrainingExample]);
    fn predict(&self, input: &str) -> String;
}
