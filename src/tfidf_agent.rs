// src/tfidf_agent.rs - Optimized TfidfAgent
use crate::agent::{text_utils, Agent, ResponseFormat, TrainableAgent, TrainingExample};
use indexmap::IndexMap;
use std::collections::HashSet;

pub struct TfidfAgent {
    docs: Vec<TrainingExample>,
    // Uses IndexMap for better performance
    term_df: IndexMap<String, f32>, // Document frequency per term
    doc_term_freq: Vec<IndexMap<String, f32>>, // Term frequency per document
    doc_count: f32,                 // Number of documents
    // Configurable BM25 parameters
    bm25_k1: f32,
    bm25_b: f32,
}

impl TfidfAgent {
    pub fn new() -> Self {
        Self {
            docs: Vec::new(),
            term_df: IndexMap::new(),
            doc_term_freq: Vec::new(),
            doc_count: 0.0,
            bm25_k1: 1.2, // Default BM25 parameter
            bm25_b: 0.75, // Default BM25 parameter
        }
    }

    // Configurable BM25 parameters
    pub fn with_bm25_params(mut self, k1: f32, b: f32) -> Self {
        self.bm25_k1 = k1;
        self.bm25_b = b;
        self
    }

    // Calculates BM25 score between query and document
    fn bm25_score(&self, query_terms: &[String], doc_idx: usize) -> f32 {
        // Average document length
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
                if let Some(&df) = self.term_df.get(term) {
                    // IDF component (Inverse Document Frequency)
                    let idf = (self.doc_count - df + 0.5) / (df + 0.5);
                    let idf = (1.0 + idf).ln();

                    // TF component (Term Frequency) with BM25 normalization
                    let tf = self.doc_term_freq[doc_idx]
                        .get(term)
                        .cloned()
                        .unwrap_or(0.0);

                    // BM25 formula
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
    fn predict(&self, input: &str) -> ResponseFormat {
        if self.docs.is_empty() {
            return ResponseFormat::Text("No training data available.".to_string());
        }

        let query_terms = text_utils::tokenize(input);

        // Calculates BM25 score for each document
        let mut scores: Vec<(usize, f32)> = self
            .docs
            .iter()
            .enumerate()
            .map(|(i, doc)| {
                let score = self.bm25_score(&query_terms, i) * doc.weight; // Consider weighting
                (i, score)
            })
            .collect();

        // Sort by score descending
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Selects the best document (if available)
        if let Some(&(best_idx, score)) = scores.first() {
            if score > 0.0 {
                return self.docs[best_idx].output.clone();
            }
        }

        ResponseFormat::Text("No matching answer found.".to_string())
    }
}

impl TrainableAgent for TfidfAgent {
    fn train(&mut self, data: &[TrainingExample]) {
        self.docs = data.to_vec();
        self.doc_count = data.len() as f32;
        self.term_df.clear();
        self.doc_term_freq.clear();

        // Calculates term frequencies per document
        for doc in &self.docs {
            let mut doc_terms: IndexMap<String, f32> = IndexMap::new();
            let terms = text_utils::tokenize(&doc.input);

            // Counts term frequencies
            for term in &terms {
                *doc_terms.entry(term.clone()).or_insert(0.0) += 1.0;
            }

            // Collects unique terms for document frequency
            let unique_terms: HashSet<String> = terms.into_iter().collect();
            for term in unique_terms {
                *self.term_df.entry(term).or_insert(0.0) += 1.0;
            }

            self.doc_term_freq.push(doc_terms);
        }
    }
}
