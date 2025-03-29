use crate::agent::{TrainableAgent, TrainingExample};
use std::collections::HashMap;

pub struct TfidfAgent {
    docs: Vec<TrainingExample>,
    // Stores document frequency for each term (word) - in how many documents it appears
    term_df: HashMap<String, f32>,
    // Stores for each document a HashMap of term to term frequency
    doc_term_freq: Vec<HashMap<String, f32>>,
    // Total number of documents
    doc_count: f32,
}

impl TfidfAgent {
    pub fn new() -> Self {
        Self {
            docs: Vec::new(),
            term_df: HashMap::new(),
            doc_term_freq: Vec::new(),
            doc_count: 0.0,
        }
    }

    // Tokenizes text into words
    fn tokenize(text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }

    // Calculates BM25 score between query and document
    fn bm25_score(&self, query_terms: &[String], doc_idx: usize) -> f32 {
        const K1: f32 = 1.2;
        const B: f32 = 0.75;

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
                    let numerator = tf * (K1 + 1.0);
                    let denominator = tf + K1 * (1.0 - B + B * doc_len / avg_doc_len);

                    idf * numerator / denominator
                } else {
                    0.0
                }
            })
            .sum()
    }
}

impl TrainableAgent for TfidfAgent {
    fn train(&mut self, data: &[TrainingExample]) {
        self.docs = data.to_vec();
        self.doc_count = data.len() as f32;
        self.term_df.clear();
        self.doc_term_freq.clear();

        // Calculate term frequencies per document
        for doc in &self.docs {
            let mut doc_terms: HashMap<String, f32> = HashMap::new();
            let terms = Self::tokenize(&doc.input);

            // Count term frequencies
            for term in &terms {
                *doc_terms.entry(term.clone()).or_insert(0.0) += 1.0;
            }

            // Collect unique terms for Document Frequency
            let unique_terms: std::collections::HashSet<String> = terms.into_iter().collect();
            for term in unique_terms {
                *self.term_df.entry(term).or_insert(0.0) += 1.0;
            }

            self.doc_term_freq.push(doc_terms);
        }
    }

    fn predict(&self, input: &str) -> String {
        if self.docs.is_empty() {
            return "No training data available.".to_string();
        }

        let query_terms = Self::tokenize(input);

        // Calculate BM25 score for each document
        let mut scores: Vec<(usize, f32)> = self
            .docs
            .iter()
            .enumerate()
            .map(|(i, doc)| {
                let score = self.bm25_score(&query_terms, i) * doc.weight; // Consider weighting
                (i, score)
            })
            .collect();

        // Sort by score in descending order
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Choose the best document (if available)
        if let Some(&(best_idx, score)) = scores.first() {
            if score > 0.0 {
                return self.docs[best_idx].output.clone();
            }
        }

        "No matching answer found.".to_string()
    }
}
