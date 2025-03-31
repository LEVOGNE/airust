// src/match_agent.rs - Replaces simple_agent.rs and fuzzy_agent.rs
use crate::agent::{Agent, ResponseFormat, TrainableAgent, TrainingExample};
use strsim::levenshtein;

// Enum for different matching strategies
pub enum MatchingStrategy {
    Exact,               // Exact match (formerly SimpleAgent)
    Fuzzy(FuzzyOptions), // Fuzzy matching (formerly FuzzyAgent)
}

pub struct FuzzyOptions {
    pub max_distance: Option<usize>, // Maximum Levenshtein distance, None for no limit
    pub threshold_factor: Option<f32>, // Factor for dynamic threshold based on input length
}

impl Default for FuzzyOptions {
    fn default() -> Self {
        Self {
            max_distance: None,
            threshold_factor: Some(0.3), // Default: 30% of input length as max distance
        }
    }
}

impl Default for MatchingStrategy {
    fn default() -> Self {
        MatchingStrategy::Fuzzy(FuzzyOptions::default())
    }
}

// Unified agent for various matching strategies
pub struct MatchAgent {
    memory: Vec<TrainingExample>,
    strategy: MatchingStrategy,
}

impl MatchAgent {
    pub fn new(strategy: MatchingStrategy) -> Self {
        Self {
            memory: Vec::new(),
            strategy,
        }
    }

    // Factory methods for convenience
    pub fn new_exact() -> Self {
        Self::new(MatchingStrategy::Exact)
    }

    pub fn new_fuzzy() -> Self {
        Self::new(MatchingStrategy::Fuzzy(FuzzyOptions::default()))
    }

    // Sets a new matching strategy
    pub fn with_strategy(mut self, strategy: MatchingStrategy) -> Self {
        self.strategy = strategy;
        self
    }
}

impl Agent for MatchAgent {
    fn predict(&self, input: &str) -> ResponseFormat {
        if self.memory.is_empty() {
            return ResponseFormat::Text("No training data available.".to_string());
        }

        match &self.strategy {
            MatchingStrategy::Exact => {
                // Exact match (formerly SimpleAgent)
                for item in &self.memory {
                    if item.input.to_lowercase() == input.to_lowercase() {
                        return item.output.clone();
                    }
                }
                ResponseFormat::Text("No matching answer found.".to_string())
            }
            MatchingStrategy::Fuzzy(options) => {
                // Fuzzy matching using Levenshtein distance (formerly FuzzyAgent)
                let mut best_score = usize::MAX;
                let mut best_match = None;

                let input_lower = input.to_lowercase();

                // Calculate threshold based on input length if specified
                let threshold = match options.threshold_factor {
                    Some(factor) => (input_lower.len() as f32 * factor) as usize,
                    None => usize::MAX,
                };

                for item in &self.memory {
                    let score = levenshtein(&item.input.to_lowercase(), &input_lower);

                    // Check if score is within max distance
                    if let Some(max_dist) = options.max_distance {
                        if score > max_dist {
                            continue;
                        }
                    }

                    // Check if score is within dynamic threshold
                    if score > threshold {
                        continue;
                    }

                    if score < best_score {
                        best_score = score;
                        best_match = Some(item);
                    }
                }

                match best_match {
                    Some(item) => item.output.clone(),
                    None => ResponseFormat::Text("No matching answer found.".to_string()),
                }
            }
        }
    }
}

impl TrainableAgent for MatchAgent {
    fn train(&mut self, data: &[TrainingExample]) {
        self.memory = data.to_vec();
    }
}
