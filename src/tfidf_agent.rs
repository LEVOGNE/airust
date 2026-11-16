//! Compatibility adapter: the historical TfidfAgent name uses the shared BM25 index.
use crate::agent::{
    Agent, ConfidenceAgent, PredictionResult, ResponseFormat, TrainableAgent, TrainingExample,
};
use crate::{text, Bm25Config, Engine, KnowledgeEntry, SearchError, SearchOptions};

pub struct TfidfAgent {
    docs: Vec<TrainingExample>,
    index: Engine,
    bm25_k1: f32,
    bm25_b: f32,
}

impl TfidfAgent {
    pub fn new() -> Self {
        Self {
            docs: Vec::new(),
            index: Engine::new(),
            bm25_k1: 1.2,
            bm25_b: 0.75,
        }
    }

    /// Set valid parameters. Invalid values leave the existing configuration unchanged.
    /// Use try_with_bm25_params when invalid configuration must be reported.
    pub fn with_bm25_params(mut self, k1: f32, b: f32) -> Self {
        if Bm25Config::new(k1 as f64, b as f64).is_ok() {
            self.bm25_k1 = k1;
            self.bm25_b = b;
        }
        self
    }

    pub fn try_with_bm25_params(self, k1: f32, b: f32) -> Result<Self, SearchError> {
        Bm25Config::new(k1 as f64, b as f64)?;
        Ok(self.with_bm25_params(k1, b))
    }

    fn scored_docs(&self, input: &str) -> Vec<(usize, f32)> {
        let options = SearchOptions {
            limit: self.docs.len(),
            bm25: Bm25Config::new(self.bm25_k1 as f64, self.bm25_b as f64).unwrap_or_default(),
            ..SearchOptions::default()
        };
        let mut scored: Vec<(usize, f32)> = self
            .index
            .search_with_options(input, &options)
            .into_iter()
            .filter_map(|hit| {
                let idx = hit.entry.id.parse::<usize>().ok()?;
                let weight = self.docs[idx].weight;
                let score = (hit.score * weight as f64) as f32;
                (score.is_finite() && score > 0.0).then_some((idx, score))
            })
            .collect();
        scored.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        scored
    }
}

impl Agent for TfidfAgent {
    fn predict(&self, input: &str) -> ResponseFormat {
        self.predict_with_metadata(input).response
    }

    fn predict_with_metadata(&self, input: &str) -> PredictionResult {
        self.predict_top_n(input, 1)
            .into_iter()
            .next()
            .unwrap_or_else(|| PredictionResult {
                response: ResponseFormat::Text(if self.docs.is_empty() {
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

impl ConfidenceAgent for TfidfAgent {
    /// Historical confidence heuristic, not a calibrated probability.
    fn calculate_confidence(&self, input: &str) -> f32 {
        self.predict_top_n(input, 1)
            .first()
            .map_or(0.0, |result| result.confidence)
    }

    fn predict_top_n(&self, input: &str, n: usize) -> Vec<PredictionResult> {
        if n == 0 {
            return Vec::new();
        }
        let normalizer = text::tokenize(input).len().max(1) as f32;
        self.scored_docs(input)
            .into_iter()
            .take(n)
            .map(|(idx, score)| PredictionResult {
                response: self.docs[idx].output.clone(),
                confidence: (score / normalizer).clamp(0.0, 1.0),
                metadata: self.docs[idx].metadata.clone(),
            })
            .collect()
    }
}

impl TrainableAgent for TfidfAgent {
    fn train(&mut self, data: &[TrainingExample]) {
        self.docs.clear();
        self.index.clear();
        self.append(data);
    }

    fn append(&mut self, data: &[TrainingExample]) {
        for example in data {
            let id = self.docs.len().to_string();
            // Empty historical training inputs remain stored but cannot match terms.
            if !example.input.trim().is_empty() {
                self.index
                    .upsert(KnowledgeEntry::new(id, &example.input))
                    .expect("nonempty historical input and generated ID are valid");
            }
            self.docs.push(example.clone());
        }
    }
}

impl Default for TfidfAgent {
    fn default() -> Self {
        Self::new()
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

    fn tech_data() -> Vec<TrainingExample> {
        vec![
            make_example(
                "rust programming language",
                "Rust is a systems programming language",
            ),
            make_example("python scripting language", "Python is great for scripting"),
            make_example(
                "javascript web development",
                "JavaScript runs in the browser",
            ),
        ]
    }

    // === Basic Query Tests ===

    #[test]
    fn test_exact_query() {
        let mut agent = TfidfAgent::new();
        agent.train(&tech_data());
        let result = String::from(agent.predict("rust programming language"));
        assert!(result.contains("Rust"));
    }

    #[test]
    fn test_partial_query() {
        let mut agent = TfidfAgent::new();
        agent.train(&tech_data());
        let result = String::from(agent.predict("rust"));
        assert!(result.contains("Rust"));
    }

    #[test]
    fn test_no_match() {
        let mut agent = TfidfAgent::new();
        agent.train(&tech_data());
        let result = String::from(agent.predict("quantum physics"));
        assert!(result.contains("No matching answer found"));
    }

    #[test]
    fn test_empty_training() {
        let agent = TfidfAgent::new();
        let result = String::from(agent.predict("hello"));
        assert!(result.contains("No training data available"));
    }

    #[test]
    fn test_single_term() {
        let mut agent = TfidfAgent::new();
        agent.train(&tech_data());
        let result = String::from(agent.predict("python"));
        assert!(result.contains("Python"));
    }

    #[test]
    fn test_stopwords_in_query() {
        let mut agent = TfidfAgent::new();
        agent.train(&tech_data());
        // "the" and "is" and "a" are filtered by tokenizer (alphabetic only)
        // but "rust" should still match
        let result = String::from(agent.predict("the rust language"));
        assert!(result.contains("Rust"));
    }

    #[test]
    fn test_case_insensitive() {
        let mut agent = TfidfAgent::new();
        agent.train(&tech_data());
        let result = String::from(agent.predict("RUST PROGRAMMING"));
        assert!(result.contains("Rust"));
    }

    #[test]
    fn test_empty_query() {
        let mut agent = TfidfAgent::new();
        agent.train(&tech_data());
        let result = String::from(agent.predict(""));
        assert!(result.contains("No matching answer found"));
    }

    #[test]
    fn test_punctuation_handling() {
        let mut agent = TfidfAgent::new();
        agent.train(&tech_data());
        // Punctuation gets stripped by tokenizer
        let result = String::from(agent.predict("rust!!! programming???"));
        assert!(result.contains("Rust"));
    }

    // === BM25 Parameter Tests ===

    #[test]
    fn test_bm25_default_params() {
        let agent = TfidfAgent::new();
        assert_eq!(agent.bm25_k1, 1.2);
        assert_eq!(agent.bm25_b, 0.75);
    }

    #[test]
    fn test_bm25_custom_params() {
        let agent = TfidfAgent::new().with_bm25_params(2.0, 0.5);
        assert_eq!(agent.bm25_k1, 2.0);
        assert_eq!(agent.bm25_b, 0.5);
    }

    #[test]
    fn test_bm25_b_zero_no_length_norm() {
        let mut agent = TfidfAgent::new().with_bm25_params(1.2, 0.0);
        agent.train(&tech_data());
        // Should still produce results (no length normalization)
        let result = String::from(agent.predict("rust"));
        assert!(result.contains("Rust"));
    }

    // === Weight Tests ===

    #[test]
    fn test_weight_affects_ranking() {
        let mut agent = TfidfAgent::new();
        let data = vec![
            TrainingExample {
                input: "programming language".to_string(),
                output: ResponseFormat::Text("low weight".to_string()),
                weight: 0.1,
                metadata: None,
            },
            TrainingExample {
                input: "programming language".to_string(),
                output: ResponseFormat::Text("high weight".to_string()),
                weight: 10.0,
                metadata: None,
            },
        ];
        agent.train(&data);
        let result = String::from(agent.predict("programming language"));
        assert_eq!(result, "high weight");
    }

    #[test]
    fn test_zero_weight_suppresses() {
        let mut agent = TfidfAgent::new();
        let data = vec![
            TrainingExample {
                input: "rust systems".to_string(),
                output: ResponseFormat::Text("suppressed".to_string()),
                weight: 0.0,
                metadata: None,
            },
            make_example("python scripting", "Python answer"),
        ];
        agent.train(&data);
        // "rust systems" has weight 0.0, so score * 0.0 = 0.0
        // "python scripting" shares no terms with query, so score = 0.0
        let result = String::from(agent.predict("rust systems"));
        assert!(result.contains("No matching answer found"));
    }

    // === Train Behavior Tests ===

    #[test]
    fn test_train_replaces_data() {
        let mut agent = TfidfAgent::new();
        agent.train(&tech_data());
        assert!(String::from(agent.predict("rust")).contains("Rust"));

        // Retrain with different data
        agent.train(&[make_example("golang concurrency", "Go uses goroutines")]);
        let result = String::from(agent.predict("rust"));
        assert!(result.contains("No matching answer found"));
        assert!(String::from(agent.predict("golang")).contains("goroutines"));
    }

    #[test]
    fn test_append_preserves_existing() {
        let mut agent = TfidfAgent::new();
        agent.train(&[make_example("rust programming", "Rust answer")]);
        agent.append(&[make_example("python scripting", "Python answer")]);
        assert!(String::from(agent.predict("rust")).contains("Rust"));
        assert!(String::from(agent.predict("python")).contains("Python"));
    }

    #[test]
    fn test_train_single_appends() {
        let mut agent = TfidfAgent::new();
        agent.train(&[make_example("rust programming", "Rust answer")]);
        agent.train_single(&make_example("golang concurrency", "Go answer"));
        assert!(String::from(agent.predict("rust")).contains("Rust"));
        assert!(String::from(agent.predict("golang")).contains("Go"));
    }

    #[test]
    fn test_single_document() {
        let mut agent = TfidfAgent::new();
        agent.train(&[make_example("hello world", "greeting")]);
        assert_eq!(String::from(agent.predict("hello")), "greeting");
    }

    #[test]
    fn test_default_trait() {
        let agent = TfidfAgent::default();
        assert_eq!(agent.bm25_k1, 1.2);
        assert_eq!(agent.bm25_b, 0.75);
    }

    // === ConfidenceAgent Tests ===

    #[test]
    fn test_calculate_confidence_match() {
        let mut agent = TfidfAgent::new();
        agent.train(&tech_data());
        let conf = agent.calculate_confidence("rust programming language");
        assert!(conf > 0.0);
        assert!(conf <= 1.0);
    }

    #[test]
    fn test_calculate_confidence_no_match() {
        let mut agent = TfidfAgent::new();
        agent.train(&tech_data());
        let conf = agent.calculate_confidence("quantum physics");
        assert_eq!(conf, 0.0);
    }

    #[test]
    fn test_calculate_confidence_empty_training() {
        let agent = TfidfAgent::new();
        assert_eq!(agent.calculate_confidence("anything"), 0.0);
    }

    #[test]
    fn test_calculate_confidence_empty_query() {
        let mut agent = TfidfAgent::new();
        agent.train(&tech_data());
        assert_eq!(agent.calculate_confidence(""), 0.0);
    }

    #[test]
    fn test_predict_top_n() {
        let mut agent = TfidfAgent::new();
        agent.train(&tech_data());
        let results = agent.predict_top_n("programming language", 3);
        assert!(!results.is_empty());
        assert!(results.len() <= 3);
        // Results should be sorted by confidence descending
        for window in results.windows(2) {
            assert!(window[0].confidence >= window[1].confidence);
        }
    }

    #[test]
    fn test_predict_top_n_limits_results() {
        let mut agent = TfidfAgent::new();
        agent.train(&tech_data());
        let results = agent.predict_top_n("programming language", 1);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_predict_top_n_empty() {
        let agent = TfidfAgent::new();
        let results = agent.predict_top_n("anything", 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_predict_top_n_zero() {
        let mut agent = TfidfAgent::new();
        agent.train(&tech_data());
        let results = agent.predict_top_n("rust", 0);
        assert!(results.is_empty());
    }

    #[test]
    fn test_predict_top_n_no_match() {
        let mut agent = TfidfAgent::new();
        agent.train(&tech_data());
        let results = agent.predict_top_n("quantum physics", 5);
        assert!(results.is_empty());
    }
}
