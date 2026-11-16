use std::collections::HashMap;
use std::sync::Arc;

use crate::web::db::Database;

pub struct VectorDB;

impl VectorDB {
    fn tokenize(text: &str) -> Vec<String> {
        crate::text::tokenize(text)
    }

    fn compute_tf(terms: &[String]) -> HashMap<String, f64> {
        let mut tf = HashMap::new();
        let total = terms.len() as f64;
        for term in terms {
            *tf.entry(term.clone()).or_insert(0.0) += 1.0;
        }
        for val in tf.values_mut() {
            *val /= total;
        }
        tf
    }

    pub fn rebuild_all(db: &Arc<Database>) -> Result<(usize, usize), String> {
        db.clear_all_vectors()?;

        let all_data = db.get_approved_data()?;
        if all_data.is_empty() {
            return Ok((0, 0));
        }

        let total_docs = all_data.len() as f64;
        let mut doc_freq: HashMap<String, f64> = HashMap::new();

        let mut doc_terms: Vec<Vec<String>> = Vec::new();
        for item in &all_data {
            let combined = format!("{} {}", item.input, item.output);
            let terms = Self::tokenize(&combined);
            let unique: std::collections::HashSet<&String> = terms.iter().collect();
            for term in unique {
                *doc_freq.entry(term.clone()).or_insert(0.0) += 1.0;
            }
            doc_terms.push(terms);
        }

        let mut idf: HashMap<String, f64> = HashMap::new();
        for (term, df) in &doc_freq {
            idf.insert(term.clone(), (total_docs / df).ln() + 1.0);
        }

        let mut total_vectors = 0usize;
        for (i, item) in all_data.iter().enumerate() {
            let tf = Self::compute_tf(&doc_terms[i]);
            for (term, tf_val) in &tf {
                if let Some(idf_val) = idf.get(term) {
                    let tfidf = tf_val * idf_val;
                    db.insert_vector(item.id, term, tfidf)?;
                    total_vectors += 1;
                }
            }
        }

        Ok((all_data.len(), total_vectors))
    }

    pub fn search(
        db: &Arc<Database>,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<SearchResult>, String> {
        let terms = Self::tokenize(query);
        if terms.is_empty() {
            return Ok(Vec::new());
        }

        let matches = db.search_vectors(&terms, top_k)?;

        let mut results = Vec::new();
        for (source_id, score) in matches {
            if let Some(data) = db.get_approved_data_by_id(source_id)? {
                results.push(SearchResult {
                    source_id,
                    input: data.input,
                    output: data.output,
                    score,
                    source_url: data.source_url,
                });
            }
        }

        Ok(results)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchResult {
    pub source_id: i64,
    pub input: String,
    pub output: String,
    pub score: f64,
    pub source_url: Option<String>,
}
