// src/match_agent.rs - Unified matching agent replacing simple and fuzzy agents
use crate::agent::{Agent, ResponseFormat, TrainableAgent, TrainingExample};
use strsim::levenshtein;

/// Defines different matching strategies for finding relevant training examples
pub enum MatchingStrategy {
    /// Exact match requiring full equality (case-insensitive)
    Exact,
    /// Fuzzy matching with configurable options
    Fuzzy(FuzzyOptions),
}

/// Configuration options for fuzzy matching
pub struct FuzzyOptions {
    /// Maximum allowed Levenshtein distance between input and training example
    /// None means no hard limit on distance
    pub max_distance: Option<usize>,

    /// Dynamic threshold factor based on input length
    /// Scales the maximum allowed distance as a fraction of input length
    pub threshold_factor: Option<f32>,
}

/// Default configuration for fuzzy matching
impl Default for FuzzyOptions {
    fn default() -> Self {
        Self {
            max_distance: None,
            threshold_factor: Some(0.3), // Default: 30% of input length as max distance
        }
    }
}

/// Default matching strategy (fuzzy with default options)
impl Default for MatchingStrategy {
    fn default() -> Self {
        MatchingStrategy::Fuzzy(FuzzyOptions::default())
    }
}

/// Unified agent capable of exact and fuzzy matching
pub struct MatchAgent {
    /// Stored training examples
    memory: Vec<TrainingExample>,

    /// Current matching strategy
    strategy: MatchingStrategy,
}

impl MatchAgent {
    /// Creates a new MatchAgent with a specific matching strategy
    pub fn new(strategy: MatchingStrategy) -> Self {
        Self {
            memory: Vec::new(),
            strategy,
        }
    }

    /// Creates an agent with exact matching strategy
    pub fn new_exact() -> Self {
        Self::new(MatchingStrategy::Exact)
    }

    /// Creates an agent with fuzzy matching strategy
    pub fn new_fuzzy() -> Self {
        Self::new(MatchingStrategy::Fuzzy(FuzzyOptions::default()))
    }

    /// Allows changing the matching strategy after agent creation
    pub fn with_strategy(mut self, strategy: MatchingStrategy) -> Self {
        self.strategy = strategy;
        self
    }
}

impl Agent for MatchAgent {
    /// Predicts the best matching response based on the current strategy
    fn predict(&self, input: &str) -> ResponseFormat {
        if self.memory.is_empty() {
            return ResponseFormat::Text("No training data available.".to_string());
        }

        match &self.strategy {
            MatchingStrategy::Exact => {
                // Exact match strategy
                for item in &self.memory {
                    if item.input.to_lowercase() == input.to_lowercase() {
                        return item.output.clone();
                    }
                }
                ResponseFormat::Text("No matching answer found.".to_string())
            }
            MatchingStrategy::Fuzzy(options) => {
                // Fuzzy matching strategy using Levenshtein distance
                let mut best_score = usize::MAX;
                let mut best_match = None;

                let input_lower = input.to_lowercase();

                // Calculate dynamic threshold based on input length
                let threshold = match options.threshold_factor {
                    Some(factor) => (input_lower.len() as f32 * factor) as usize,
                    None => usize::MAX,
                };

                for item in &self.memory {
                    let score = levenshtein(&item.input.to_lowercase(), &input_lower);

                    // Check max distance constraint
                    if let Some(max_dist) = options.max_distance {
                        if score > max_dist {
                            continue;
                        }
                    }

                    // Check dynamic threshold
                    if score > threshold {
                        continue;
                    }

                    // Find best match
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
    /// Trains the agent by storing training examples
    fn train(&mut self, data: &[TrainingExample]) {
        self.memory = data.to_vec();
    }
}
