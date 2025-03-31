// src/tfidf_agent.rs - Optimized TF-IDF/BM25 Agent
use crate::agent::{text_utils, Agent, ResponseFormat, TrainableAgent, TrainingExample};
use indexmap::IndexMap;
use std::collections::HashSet;

/// TF-IDF Agent using BM25 scoring for intelligent text matching
pub struct TfidfAgent {
    /// Stored training documents
    docs: Vec<TrainingExample>,

    /// Document frequency for each term (in how many documents a term appears)
    term_df: IndexMap<String, f32>,

    /// Term frequencies for each document
    doc_term_freq: Vec<IndexMap<String, f32>>,

    /// Total number of documents
    doc_count: f32,

    /// BM25 parameter k1 (controls term frequency scaling)
    bm25_k1: f32,

    /// BM25 parameter b (controls document length normalization)
    bm25_b: f32,
}

impl TfidfAgent {
    /// Creates a new TF-IDF agent with default BM25 parameters
    pub fn new() -> Self {
        Self {
            docs: Vec::new(),
            term_df: IndexMap::new(),
            doc_term_freq: Vec::new(),
            doc_count: 0.0,
            bm25_k1: 1.2, // Default term frequency scaling
            bm25_b: 0.75, // Default length normalization
        }
    }

    /// Configures custom BM25 parameters for fine-tuned matching
    pub fn with_bm25_params(mut self, k1: f32, b: f32) -> Self {
        self.bm25_k1 = k1;
        self.bm25_b = b;
        self
    }

    /// Calculates BM25 score between query terms and a specific document
    fn bm25_score(&self, query_terms: &[String], doc_idx: usize) -> f32 {
        // Calculate average document length
        let avg_doc_len: f32 = self
            .doc_term_freq
            .iter()
            .map(|doc| doc.values().sum::<f32>())
            .sum::<f32>()
            / self.doc_count;

        // Length of the current document
        let doc_len: f32 = self.doc_term_freq[doc_idx].values().sum();

        query_terms
            .iter()
            .map(|term| {
                // Check if term exists in the document frequency index
                if let Some(&df) = self.term_df.get(term) {
                    // Inverse Document Frequency (IDF) component
                    let idf = (self.doc_count - df + 0.5) / (df + 0.5);
                    let idf = (1.0 + idf).ln();

                    // Term Frequency (TF) with BM25 normalization
                    let tf = self.doc_term_freq[doc_idx]
                        .get(term)
                        .cloned()
                        .unwrap_or(0.0);

                    // BM25 scoring formula
                    let numerator = tf * (self.bm25_k1 + 1.0);
                    let denominator = tf
                        + self.bm25_k1 * (1.0 - self.bm25_b + self.bm25_b * doc_len / avg_doc_len);

                    idf * numerator / denominator
                } else {
                    0.0
                }
            })
            .sum()
    }
}

impl Agent for TfidfAgent {
    /// Predicts the most relevant response using BM25 scoring
    fn predict(&self, input: &str) -> ResponseFormat {
        // Handle empty training data
        if self.docs.is_empty() {
            return ResponseFormat::Text("No training data available.".to_string());
        }

        // Tokenize input into terms
        let query_terms = text_utils::tokenize(input);

        // Calculate BM25 scores for each document
        let mut scores: Vec<(usize, f32)> = self
            .docs
            .iter()
            .enumerate()
            .map(|(i, doc)| {
                // Calculate score with document weight
                let score = self.bm25_score(&query_terms, i) * doc.weight;
                (i, score)
            })
            .collect();

        // Sort scores in descending order
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Select best matching document
        if let Some(&(best_idx, score)) = scores.first() {
            if score > 0.0 {
                return self.docs[best_idx].output.clone();
            }
        }

        ResponseFormat::Text("No matching answer found.".to_string())
    }
}

impl TrainableAgent for TfidfAgent {
    /// Trains the agent by processing training documents
    fn train(&mut self, data: &[TrainingExample]) {
        // Reset existing data
        self.docs = data.to_vec();
        self.doc_count = data.len() as f32;
        self.term_df.clear();
        self.doc_term_freq.clear();

        // Process each document
        for doc in &self.docs {
            // Tokenize document input
            let mut doc_terms: IndexMap<String, f32> = IndexMap::new();
            let terms = text_utils::tokenize(&doc.input);

            // Count term frequencies
            for term in &terms {
                *doc_terms.entry(term.clone()).or_insert(0.0) += 1.0;
            }

            // Track unique terms for document frequency
            let unique_terms: HashSet<String> = terms.into_iter().collect();
            for term in unique_terms {
                *self.term_df.entry(term).or_insert(0.0) += 1.0;
            }

            self.doc_term_freq.push(doc_terms);
        }
    }
}

// Default implementation for creating a new TF-IDF agent
impl Default for TfidfAgent {
    fn default() -> Self {
        Self::new()
    }
}
