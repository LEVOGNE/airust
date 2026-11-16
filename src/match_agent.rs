use crate::agent::{
    Agent, ConfidenceAgent, PredictionResult, ResponseFormat, TrainableAgent, TrainingExample,
};

#[derive(Debug, Clone)]
pub enum MatchingStrategy {
    Exact,
    Fuzzy(FuzzyOptions),
}

#[derive(Debug, Clone)]
pub struct FuzzyOptions {
    pub max_distance: Option<usize>,

    pub threshold_factor: Option<f32>,
}

impl Default for FuzzyOptions {
    fn default() -> Self {
        Self {
            max_distance: None,
            threshold_factor: Some(0.3),
        }
    }
}

impl Default for MatchingStrategy {
    fn default() -> Self {
        Self::Fuzzy(FuzzyOptions::default())
    }
}

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
    pub fn new_exact() -> Self {
        Self::new(MatchingStrategy::Exact)
    }
    pub fn new_fuzzy() -> Self {
        Self::new(MatchingStrategy::default())
    }
    pub fn with_strategy(mut self, strategy: MatchingStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    fn ranked_matches(&self, input: &str) -> Vec<(usize, usize, f32)> {
        let input = input.to_lowercase();
        let input_len = input.chars().count();
        let mut matches = Vec::new();
        for (idx, example) in self.memory.iter().enumerate() {
            let candidate = example.input.to_lowercase();
            let (distance, confidence) = match &self.strategy {
                MatchingStrategy::Exact => {
                    if input != candidate {
                        continue;
                    }
                    (0, 1.0)
                }
                MatchingStrategy::Fuzzy(options) => {
                    let threshold = match options.threshold_factor {
                        Some(factor) if factor.is_finite() && factor >= 0.0 => {
                            (input_len as f32 * factor) as usize
                        }
                        Some(_) => continue,
                        None => usize::MAX,
                    };
                    let maximum = threshold.min(options.max_distance.unwrap_or(usize::MAX));
                    let Some(distance) =
                        crate::matching::bounded_distance(&input, &candidate, maximum)
                    else {
                        continue;
                    };
                    let max_len = input_len.max(candidate.chars().count()).max(1);
                    (distance, (1.0 - distance as f32 / max_len as f32).max(0.0))
                }
            };
            matches.push((idx, distance, confidence));
        }
        matches.sort_by_key(|&(idx, distance, _)| (distance, idx));
        matches
    }
}

impl Agent for MatchAgent {
    fn predict(&self, input: &str) -> ResponseFormat {
        self.predict_with_metadata(input).response
    }

    fn predict_with_metadata(&self, input: &str) -> PredictionResult {
        self.predict_top_n(input, 1)
            .into_iter()
            .next()
            .unwrap_or_else(|| PredictionResult {
                response: ResponseFormat::Text(if self.memory.is_empty() {
                    "No training data available.".to_owned()
                } else {
                    "No matching answer found.".to_owned()
                }),
                confidence: 0.0,
                metadata: None,
            })
    }

    fn confidence(&self, input: &str) -> f32 {
        self.calculate_confidence(input)
    }
}

impl TrainableAgent for MatchAgent {
    fn train(&mut self, data: &[TrainingExample]) {
        self.memory = data.to_vec();
    }
    fn append(&mut self, data: &[TrainingExample]) {
        self.memory.extend_from_slice(data);
    }
}

impl ConfidenceAgent for MatchAgent {
    fn calculate_confidence(&self, input: &str) -> f32 {
        self.ranked_matches(input)
            .first()
            .map_or(0.0, |&(_, _, confidence)| confidence)
    }

    fn predict_top_n(&self, input: &str, n: usize) -> Vec<PredictionResult> {
        if n == 0 {
            return Vec::new();
        }
        self.ranked_matches(input)
            .into_iter()
            .take(n)
            .map(|(idx, _, confidence)| PredictionResult {
                response: self.memory[idx].output.clone(),
                confidence,
                metadata: self.memory[idx].metadata.clone(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_example(input: &str, output: &str) -> TrainingExample {
        TrainingExample {
            input: input.to_string(),
            output: ResponseFormat::Text(output.to_string()),
            weight: 1.0,
            metadata: None,
        }
    }

    fn sample_data() -> Vec<TrainingExample> {
        vec![
            make_example("hello", "Hi there!"),
            make_example("goodbye", "See you later!"),
            make_example("how are you", "I am fine, thanks!"),
        ]
    }

    #[test]
    fn test_exact_basic_match() {
        let mut agent = MatchAgent::new_exact();
        agent.train(&sample_data());
        assert_eq!(String::from(agent.predict("hello")), "Hi there!");
    }

    #[test]
    fn test_exact_case_insensitive() {
        let mut agent = MatchAgent::new_exact();
        agent.train(&sample_data());
        assert_eq!(String::from(agent.predict("HELLO")), "Hi there!");
        assert_eq!(String::from(agent.predict("Hello")), "Hi there!");
    }

    #[test]
    fn test_exact_no_match_fallback() {
        let mut agent = MatchAgent::new_exact();
        agent.train(&sample_data());
        let result = String::from(agent.predict("unknown"));
        assert!(result.contains("No matching answer found"));
    }

    #[test]
    fn test_exact_empty_training() {
        let agent = MatchAgent::new_exact();
        let result = String::from(agent.predict("hello"));
        assert!(result.contains("No training data available"));
    }

    #[test]
    fn test_exact_whitespace_sensitivity() {
        let mut agent = MatchAgent::new_exact();
        agent.train(&sample_data());

        let result = String::from(agent.predict(" hello"));
        assert!(result.contains("No matching answer found"));
    }

    #[test]
    fn test_exact_unicode_input() {
        let mut agent = MatchAgent::new_exact();
        agent.train(&[make_example("café", "coffee shop")]);
        assert_eq!(String::from(agent.predict("café")), "coffee shop");
        assert_eq!(String::from(agent.predict("CAFÉ")), "coffee shop");
    }

    #[test]
    fn test_exact_first_match_wins() {
        let mut agent = MatchAgent::new_exact();
        agent.train(&[
            make_example("hello", "first"),
            make_example("hello", "second"),
        ]);
        assert_eq!(String::from(agent.predict("hello")), "first");
    }

    #[test]
    fn test_exact_empty_string() {
        let mut agent = MatchAgent::new_exact();
        agent.train(&[make_example("", "empty match")]);
        assert_eq!(String::from(agent.predict("")), "empty match");
    }

    #[test]
    fn test_exact_markdown_response() {
        let mut agent = MatchAgent::new_exact();
        agent.train(&[TrainingExample {
            input: "help".to_string(),
            output: ResponseFormat::Markdown("# Help\nUse /help".to_string()),
            weight: 1.0,
            metadata: None,
        }]);
        match agent.predict("help") {
            ResponseFormat::Markdown(md) => assert!(md.contains("# Help")),
            _ => panic!("Expected Markdown response"),
        }
    }

    #[test]
    fn test_exact_json_response() {
        let mut agent = MatchAgent::new_exact();
        agent.train(&[TrainingExample {
            input: "status".to_string(),
            output: ResponseFormat::Json(serde_json::json!({"status": "ok"})),
            weight: 1.0,
            metadata: None,
        }]);
        match agent.predict("status") {
            ResponseFormat::Json(val) => assert_eq!(val["status"], "ok"),
            _ => panic!("Expected JSON response"),
        }
    }

    #[test]
    fn test_fuzzy_exact_input() {
        let mut agent = MatchAgent::new_fuzzy();
        agent.train(&sample_data());
        assert_eq!(String::from(agent.predict("hello")), "Hi there!");
    }

    #[test]
    fn test_fuzzy_close_match() {
        let mut agent = MatchAgent::new_fuzzy();
        agent.train(&sample_data());

        assert_eq!(String::from(agent.predict("helo")), "Hi there!");
    }

    #[test]
    fn test_fuzzy_threshold_rejection() {
        let mut agent = MatchAgent::new(MatchingStrategy::Fuzzy(FuzzyOptions {
            max_distance: None,
            threshold_factor: Some(0.1),
        }));
        agent.train(&sample_data());

        let result = String::from(agent.predict("xxxxxxxxx"));
        assert!(result.contains("No matching answer found"));
    }

    #[test]
    fn test_fuzzy_max_distance_constraint() {
        let mut agent = MatchAgent::new(MatchingStrategy::Fuzzy(FuzzyOptions {
            max_distance: Some(1),
            threshold_factor: None,
        }));
        agent.train(&sample_data());

        assert_eq!(String::from(agent.predict("helo")), "Hi there!");

        let result = String::from(agent.predict("hxxxxxxx"));
        assert!(result.contains("No matching answer found"));
    }

    #[test]
    fn test_fuzzy_best_match_selection() {
        let mut agent = MatchAgent::new_fuzzy();
        agent.train(&[
            make_example("cat", "feline"),
            make_example("car", "vehicle"),
            make_example("bat", "animal"),
        ]);

        assert_eq!(String::from(agent.predict("cat")), "feline");
    }

    #[test]
    fn test_fuzzy_case_insensitive() {
        let mut agent = MatchAgent::new_fuzzy();
        agent.train(&sample_data());
        assert_eq!(String::from(agent.predict("HELLO")), "Hi there!");
    }

    #[test]
    fn test_fuzzy_default_options() {
        let options = FuzzyOptions::default();
        assert!(options.max_distance.is_none());
        assert_eq!(options.threshold_factor, Some(0.3));
    }

    #[test]
    fn test_fuzzy_custom_options() {
        let options = FuzzyOptions {
            max_distance: Some(5),
            threshold_factor: Some(0.5),
        };
        assert_eq!(options.max_distance, Some(5));
        assert_eq!(options.threshold_factor, Some(0.5));
    }

    #[test]
    fn test_new_exact_constructor() {
        let agent = MatchAgent::new_exact();
        let result = String::from(agent.predict("anything"));
        assert!(result.contains("No training data available"));
    }

    #[test]
    fn test_new_fuzzy_constructor() {
        let agent = MatchAgent::new_fuzzy();
        let result = String::from(agent.predict("anything"));
        assert!(result.contains("No training data available"));
    }

    #[test]
    fn test_with_strategy() {
        let agent = MatchAgent::new_fuzzy().with_strategy(MatchingStrategy::Exact);
        let result = String::from(agent.predict("anything"));
        assert!(result.contains("No training data available"));
    }

    #[test]
    fn test_default_strategy_is_fuzzy() {
        let strategy = MatchingStrategy::default();
        match strategy {
            MatchingStrategy::Fuzzy(_) => {}
            MatchingStrategy::Exact => panic!("Default should be Fuzzy"),
        }
    }

    #[test]
    fn test_train_replaces_all_data() {
        let mut agent = MatchAgent::new_exact();
        agent.train(&sample_data());
        assert_eq!(String::from(agent.predict("hello")), "Hi there!");

        agent.train(&[make_example("rust", "A language")]);
        let result = String::from(agent.predict("hello"));
        assert!(result.contains("No matching answer found"));
        assert_eq!(String::from(agent.predict("rust")), "A language");
    }

    #[test]
    fn test_train_single_appends() {
        let mut agent = MatchAgent::new_exact();
        agent.train(&sample_data());

        agent.train_single(&make_example("new", "added"));
        assert_eq!(String::from(agent.predict("hello")), "Hi there!");
        assert_eq!(String::from(agent.predict("new")), "added");
    }

    #[test]
    fn test_append_preserves_existing() {
        let mut agent = MatchAgent::new_exact();
        agent.train(&[make_example("hello", "Hi!")]);
        agent.append(&[make_example("bye", "Goodbye!")]);
        assert_eq!(String::from(agent.predict("hello")), "Hi!");
        assert_eq!(String::from(agent.predict("bye")), "Goodbye!");
    }

    #[test]
    fn test_add_example_appends() {
        let mut agent = MatchAgent::new_exact();
        agent.train(&sample_data());
        agent.add_example("rust", "A language", 1.0);

        assert_eq!(String::from(agent.predict("hello")), "Hi there!");
        assert_eq!(String::from(agent.predict("rust")), "A language");
    }

    #[test]
    fn test_confidence_match() {
        let mut agent = MatchAgent::new_exact();
        agent.train(&sample_data());
        assert_eq!(agent.confidence("hello"), 1.0);
    }

    #[test]
    fn test_confidence_no_match() {
        let mut agent = MatchAgent::new_exact();
        agent.train(&sample_data());
        assert_eq!(agent.confidence("unknown"), 0.0);
    }

    #[test]
    fn test_can_answer_true() {
        let mut agent = MatchAgent::new_exact();
        agent.train(&sample_data());
        assert!(agent.can_answer("hello"));
    }

    #[test]
    fn test_can_answer_false() {
        let mut agent = MatchAgent::new_exact();
        agent.train(&sample_data());
        assert!(!agent.can_answer("unknown"));
    }

    #[test]
    fn test_exact_calculate_confidence_match() {
        let mut agent = MatchAgent::new_exact();
        agent.train(&sample_data());
        assert_eq!(agent.calculate_confidence("hello"), 1.0);
    }

    #[test]
    fn test_exact_calculate_confidence_no_match() {
        let mut agent = MatchAgent::new_exact();
        agent.train(&sample_data());
        assert_eq!(agent.calculate_confidence("unknown"), 0.0);
    }

    #[test]
    fn test_exact_calculate_confidence_empty() {
        let agent = MatchAgent::new_exact();
        assert_eq!(agent.calculate_confidence("anything"), 0.0);
    }

    #[test]
    fn test_fuzzy_calculate_confidence_exact() {
        let mut agent = MatchAgent::new_fuzzy();
        agent.train(&sample_data());

        assert_eq!(agent.calculate_confidence("hello"), 1.0);
    }

    #[test]
    fn test_fuzzy_calculate_confidence_close() {
        let mut agent = MatchAgent::new_fuzzy();
        agent.train(&sample_data());

        let conf = agent.calculate_confidence("helo");
        assert!(conf > 0.5 && conf < 1.0);
    }

    #[test]
    fn test_fuzzy_calculate_confidence_no_match() {
        let mut agent = MatchAgent::new(MatchingStrategy::Fuzzy(FuzzyOptions {
            max_distance: None,
            threshold_factor: Some(0.1),
        }));
        agent.train(&sample_data());
        assert_eq!(agent.calculate_confidence("xxxxxxxxxxx"), 0.0);
    }

    #[test]
    fn test_exact_predict_top_n() {
        let mut agent = MatchAgent::new_exact();
        agent.train(&[
            make_example("hello", "first"),
            make_example("hello", "second"),
            make_example("bye", "later"),
        ]);
        let results = agent.predict_top_n("hello", 5);
        assert_eq!(results.len(), 2);
        assert_eq!(String::from(results[0].response.clone()), "first");
        assert_eq!(results[0].confidence, 1.0);
    }

    #[test]
    fn test_fuzzy_predict_top_n() {
        let mut agent = MatchAgent::new_fuzzy();
        agent.train(&sample_data());
        let results = agent.predict_top_n("hello", 2);
        assert!(!results.is_empty());
        assert!(results.len() <= 2);

        if results.len() > 1 {
            assert!(results[0].confidence >= results[1].confidence);
        }
    }

    #[test]
    fn test_predict_top_n_empty() {
        let agent = MatchAgent::new_exact();
        let results = agent.predict_top_n("hello", 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_predict_top_n_zero() {
        let mut agent = MatchAgent::new_exact();
        agent.train(&sample_data());
        let results = agent.predict_top_n("hello", 0);
        assert!(results.is_empty());
    }
}
