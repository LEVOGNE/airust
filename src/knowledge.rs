// src/knowledge.rs - Unified Knowledge Base
use crate::agent::{LegacyTrainingExample, ResponseFormat, TrainingExample};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

// Contains both data types for backward compatibility
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum TrainingData {
    Legacy(Vec<LegacyTrainingExample>),
    Modern(Vec<TrainingExample>),
}

#[derive(Clone)]
pub struct KnowledgeBase {
    examples: Vec<TrainingExample>,
    file_path: Option<PathBuf>,
}

// Compile-time data (static)
pub static EMBEDDED_DATA: Lazy<Arc<Vec<TrainingExample>>> = Lazy::new(|| {
    let raw = include_str!(concat!(env!("OUT_DIR"), "/train.json"));

    match serde_json::from_str::<TrainingData>(raw) {
        Ok(TrainingData::Modern(examples)) => Arc::new(examples),
        Ok(TrainingData::Legacy(legacy)) => {
            // Converts legacy data into the modern format
            Arc::new(legacy.into_iter().map(|ex| ex.into()).collect())
        }
        Err(e) => {
            eprintln!("Error loading embedded training data: {}", e);
            Arc::new(Vec::new())
        }
    }
});

impl KnowledgeBase {
    pub fn new() -> Self {
        Self {
            examples: Vec::new(),
            file_path: None,
        }
    }

    // Creates a knowledge base from embedded data
    pub fn from_embedded() -> Self {
        Self {
            examples: EMBEDDED_DATA.to_vec(),
            file_path: None,
        }
    }

    // Loads knowledge base from file
    pub fn load(path: PathBuf) -> Result<Self, String> {
        let data = fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))?;

        match serde_json::from_str::<TrainingData>(&data) {
            Ok(TrainingData::Modern(examples)) => Ok(Self {
                examples,
                file_path: Some(path),
            }),
            Ok(TrainingData::Legacy(legacy)) => {
                // Converts legacy data
                Ok(Self {
                    examples: legacy.into_iter().map(|ex| ex.into()).collect(),
                    file_path: Some(path),
                })
            }
            Err(e) => Err(format!("Deserialization error: {}", e)),
        }
    }

    // Saves knowledge base to file
    pub fn save(&self, path: Option<PathBuf>) -> Result<(), String> {
        let path = path
            .or_else(|| self.file_path.clone())
            .ok_or_else(|| "No path provided".to_string())?;

        let json = serde_json::to_string_pretty(&self.examples)
            .map_err(|e| format!("Serialization error: {}", e))?;

        let mut file = File::create(&path).map_err(|e| format!("Failed to create file: {}", e))?;

        file.write_all(json.as_bytes())
            .map_err(|e| format!("Failed to write to file: {}", e))?;

        Ok(())
    }

    // Adds a new training example
    pub fn add_example(&mut self, input: String, output: impl Into<ResponseFormat>, weight: f32) {
        let example = TrainingExample {
            input,
            output: output.into(),
            weight,
        };
        self.examples.push(example);
    }

    // Removes a training example
    pub fn remove_example(&mut self, index: usize) -> Result<TrainingExample, String> {
        if index < self.examples.len() {
            Ok(self.examples.remove(index))
        } else {
            Err(format!("Index {} out of bounds", index))
        }
    }

    // Returns all examples
    pub fn get_examples(&self) -> &[TrainingExample] {
        &self.examples
    }

    // Merges two knowledge bases
    pub fn merge(&mut self, other: &KnowledgeBase) {
        self.examples.extend_from_slice(&other.examples);
    }

    // Merges embedded data into the current knowledge base
    pub fn merge_embedded(&mut self) {
        self.examples.extend_from_slice(&EMBEDDED_DATA);
    }
}
