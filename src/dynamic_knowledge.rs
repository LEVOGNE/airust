use crate::agent::TrainingExample;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct KnowledgeBase {
    examples: Vec<TrainingExample>,
    file_path: Option<PathBuf>,
}

impl KnowledgeBase {
    pub fn new() -> Self {
        Self {
            examples: Vec::new(),
            file_path: None,
        }
    }

    // Load knowledge base from file
    pub fn load(path: PathBuf) -> Result<Self, String> {
        let data = fs::read_to_string(&path).map_err(|e| format!("Could not read file: {}", e))?;

        let examples: Vec<TrainingExample> = serde_json::from_str(&data)
            .map_err(|e| format!("Error during deserialization: {}", e))?;

        Ok(Self {
            examples,
            file_path: Some(path),
        })
    }

    // Save knowledge base to file
    pub fn save(&self, path: Option<PathBuf>) -> Result<(), String> {
        let path = path
            .or(self.file_path.clone())
            .ok_or_else(|| "No path specified".to_string())?;

        let json = serde_json::to_string_pretty(&self.examples)
            .map_err(|e| format!("Error during serialization: {}", e))?;

        let mut file = File::create(&path).map_err(|e| format!("Could not create file: {}", e))?;

        file.write_all(json.as_bytes())
            .map_err(|e| format!("Could not write to file: {}", e))?;

        Ok(())
    }

    // Add a new training example
    pub fn add_example(&mut self, input: String, output: String, weight: f32) {
        let example = TrainingExample {
            input,
            output,
            weight,
        };
        self.examples.push(example);
    }

    // Remove a training example
    pub fn remove_example(&mut self, index: usize) -> Result<TrainingExample, String> {
        if index < self.examples.len() {
            Ok(self.examples.remove(index))
        } else {
            Err(format!("Index {} out of valid range", index))
        }
    }

    // Get all examples
    pub fn get_examples(&self) -> &[TrainingExample] {
        &self.examples
    }
}
