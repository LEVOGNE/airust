use crate::agent::TrainingExample;
use once_cell::sync::Lazy;
use serde_json;

pub static TRAINING_DATA: Lazy<Vec<TrainingExample>> = Lazy::new(|| {
    let raw = include_str!(concat!(env!("OUT_DIR"), "/train.json"));
    serde_json::from_str(raw).expect("Trainingsdaten konnten nicht geladen werden")
});
