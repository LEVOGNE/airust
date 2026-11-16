use std::collections::HashMap;

use crate::agent::text_utils::tokenize;
use crate::web::db::VectorEntryRow;

#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchResult {
    pub entry_id: i64,
    pub content: String,
    pub metadata_json: String,
    pub score: f64,
}

pub fn build_vocabulary(docs: &[&str]) -> Vec<String> {
    let mut vocab_set: std::collections::HashSet<String> = std::collections::HashSet::new();
    for doc in docs {
        for token in tokenize(doc) {
            vocab_set.insert(token);
        }
    }
    let mut vocab: Vec<String> = vocab_set.into_iter().collect();
    vocab.sort();
    vocab
}

pub fn compute_doc_frequencies(docs: &[&str]) -> HashMap<String, f64> {
    let mut df: HashMap<String, f64> = HashMap::new();
    for doc in docs {
        let unique: std::collections::HashSet<String> = tokenize(doc).into_iter().collect();
        for term in unique {
            *df.entry(term).or_insert(0.0) += 1.0;
        }
    }
    df
}

pub fn tfidf_vector(
    text: &str,
    vocab: &[String],
    df: &HashMap<String, f64>,
    total_docs: f64,
) -> Vec<f64> {
    let tokens = tokenize(text);
    let total_terms = tokens.len() as f64;
    if total_terms == 0.0 {
        return vec![0.0; vocab.len()];
    }

    let mut tf: HashMap<String, f64> = HashMap::new();
    for token in &tokens {
        *tf.entry(token.clone()).or_insert(0.0) += 1.0;
    }

    vocab
        .iter()
        .map(|term| {
            let term_freq = tf.get(term).copied().unwrap_or(0.0) / total_terms;
            let doc_freq = df.get(term).copied().unwrap_or(0.0);
            let idf = if doc_freq > 0.0 {
                (total_docs / doc_freq).ln() + 1.0
            } else {
                1.0
            };
            term_freq * idf
        })
        .collect()
}

pub fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let mag_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let mag_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }
    dot / (mag_a * mag_b)
}

pub fn similarity_search(
    query: &str,
    entries: &[VectorEntryRow],
    top_k: usize,
) -> Vec<SearchResult> {
    if entries.is_empty() {
        return Vec::new();
    }

    let docs: Vec<&str> = entries.iter().map(|e| e.content.as_str()).collect();
    let vocab = build_vocabulary(&docs);
    let df = compute_doc_frequencies(&docs);
    let total_docs = docs.len() as f64;

    let query_vec = tfidf_vector(query, &vocab, &df, total_docs);

    let mut scored: Vec<SearchResult> = entries
        .iter()
        .map(|entry| {
            let entry_vec = tfidf_vector(&entry.content, &vocab, &df, total_docs);
            let score = cosine_similarity(&query_vec, &entry_vec);
            SearchResult {
                entry_id: entry.id,
                content: entry.content.clone(),
                metadata_json: entry.metadata_json.clone(),
                score,
            }
        })
        .collect();

    scored.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    scored.truncate(top_k);

    scored.into_iter().filter(|r| r.score > 0.0).collect()
}

pub fn compute_embedding_for_storage(content: &str, existing_entries: &[VectorEntryRow]) -> String {
    let mut docs: Vec<&str> = existing_entries
        .iter()
        .map(|e| e.content.as_str())
        .collect();
    docs.push(content);
    let vocab = build_vocabulary(&docs);
    let df = compute_doc_frequencies(&docs);
    let total_docs = docs.len() as f64;
    let vec = tfidf_vector(content, &vocab, &df, total_docs);
    serde_json::to_string(&vec).unwrap_or_else(|_| "[]".to_string())
}
