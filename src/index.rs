use crate::context::Context;
use crate::knowledge::{KnowledgeEntry, RelationKind, RelationSuggestion};
use crate::matching::{MatchError, MatchOptions, MatchResult, MatchStrategy, Similarity};
use crate::ranking::TopK;
use crate::search::{tfidf_idf, SearchAlgorithm, SearchOptions, SearchResult};
use crate::text;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IndexError {
    #[error("entry ID must not be empty")]
    EmptyId,
    #[error("entry {0:?} must have nonempty text")]
    EmptyText(String),
    #[error("duplicate entry ID: {0}")]
    DuplicateId(String),
    #[error("unknown entry ID: {0}")]
    UnknownId(String),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
struct IndexedEntry {
    entry: KnowledgeEntry,
    normalized: String,
    terms: BTreeMap<String, usize>,
    length: usize,
    char_length: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct IndexStats {
    pub entries: usize,
    pub terms: usize,
    pub tokens: usize,
}

#[derive(Debug, Clone, Default)]
pub struct KnowledgeIndex {
    documents: BTreeMap<String, IndexedEntry>,
    postings: BTreeMap<String, BTreeMap<String, usize>>,
    normalized_ids: BTreeMap<String, BTreeSet<String>>,
    total_tokens: usize,
}

impl KnowledgeIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_entries(
        entries: impl IntoIterator<Item = KnowledgeEntry>,
    ) -> Result<Self, IndexError> {
        let mut index = Self::new();
        for entry in entries {
            if index.documents.contains_key(&entry.id) {
                return Err(IndexError::DuplicateId(entry.id));
            }
            index.upsert(entry)?;
        }
        Ok(index)
    }

    pub fn upsert(&mut self, entry: KnowledgeEntry) -> Result<Option<KnowledgeEntry>, IndexError> {
        if entry.id.trim().is_empty() {
            return Err(IndexError::EmptyId);
        }
        let normalized = text::normalize(&entry.text);
        if normalized.is_empty() {
            return Err(IndexError::EmptyText(entry.id));
        }
        let terms = text::frequencies_normalized(&normalized);
        let char_length = normalized.chars().count();
        let length = terms.values().sum();
        let previous = self.remove(&entry.id);
        for (term, count) in &terms {
            self.postings
                .entry(term.clone())
                .or_default()
                .insert(entry.id.clone(), *count);
        }
        self.normalized_ids
            .entry(normalized.clone())
            .or_default()
            .insert(entry.id.clone());
        self.total_tokens += length;
        self.documents.insert(
            entry.id.clone(),
            IndexedEntry {
                entry,
                normalized,
                terms,
                length,
                char_length,
            },
        );
        Ok(previous)
    }

    pub fn remove(&mut self, id: &str) -> Option<KnowledgeEntry> {
        let document = self.documents.remove(id)?;
        self.total_tokens -= document.length;
        for term in document.terms.keys() {
            if let Some(postings) = self.postings.get_mut(term) {
                postings.remove(id);
                if postings.is_empty() {
                    self.postings.remove(term);
                }
            }
        }
        if let Some(ids) = self.normalized_ids.get_mut(&document.normalized) {
            ids.remove(id);
            if ids.is_empty() {
                self.normalized_ids.remove(&document.normalized);
            }
        }
        Some(document.entry)
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }
    pub fn len(&self) -> usize {
        self.documents.len()
    }
    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }
    pub fn get(&self, id: &str) -> Option<&KnowledgeEntry> {
        self.documents.get(id).map(|d| &d.entry)
    }
    pub fn entries(&self) -> impl Iterator<Item = &KnowledgeEntry> {
        self.documents.values().map(|d| &d.entry)
    }
    pub fn stats(&self) -> IndexStats {
        IndexStats {
            entries: self.len(),
            terms: self.postings.len(),
            tokens: self.total_tokens,
        }
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchResult<'_>> {
        self.search_with_options(
            query,
            &SearchOptions {
                limit,
                ..SearchOptions::default()
            },
        )
    }

    pub fn search_in_context(
        &self,
        query: &str,
        context: &Context,
        limit: usize,
    ) -> Vec<SearchResult<'_>> {
        self.search_with_options(
            query,
            &SearchOptions {
                limit,
                context: Some(context.clone()),
                ..SearchOptions::default()
            },
        )
    }

    pub fn search_with_options(
        &self,
        query: &str,
        options: &SearchOptions,
    ) -> Vec<SearchResult<'_>> {
        if options.limit == 0 || self.is_empty() {
            return Vec::new();
        }
        let query_terms = text::term_frequencies(query);
        let candidates = self.candidates(query_terms.keys().map(String::as_str));
        let average = self.total_tokens as f64 / self.len() as f64;
        let query_norm = if options.algorithm == SearchAlgorithm::Tfidf {
            query_terms
                .iter()
                .filter_map(|(term, tf)| {
                    self.postings
                        .get(term)
                        .map(|p| (*tf as f64 * tfidf_idf(self.len(), p.len())).powi(2))
                })
                .sum::<f64>()
                .sqrt()
        } else {
            0.0
        };
        let mut results = TopK::new(options.limit);
        for id in candidates {
            let document = &self.documents[id];
            if options
                .context
                .as_ref()
                .is_some_and(|context| !context.includes(&document.entry))
            {
                continue;
            }
            let matched_terms: Vec<String> = query_terms
                .keys()
                .filter(|term| document.terms.contains_key(*term))
                .cloned()
                .collect();
            let score = match options.algorithm {
                SearchAlgorithm::Bm25 => matched_terms
                    .iter()
                    .map(|term| {
                        options.bm25.term_score(
                            document.terms[term],
                            self.postings[term].len(),
                            self.len(),
                            document.length,
                            average,
                        )
                    })
                    .sum(),
                SearchAlgorithm::Tfidf => {
                    let dot = matched_terms
                        .iter()
                        .map(|term| {
                            let idf = tfidf_idf(self.len(), self.postings[term].len());
                            query_terms[term] as f64 * document.terms[term] as f64 * idf * idf
                        })
                        .sum::<f64>();
                    let norm = document
                        .terms
                        .iter()
                        .map(|(term, tf)| {
                            (*tf as f64 * tfidf_idf(self.len(), self.postings[term].len())).powi(2)
                        })
                        .sum::<f64>()
                        .sqrt();
                    if norm > 0.0 && query_norm > 0.0 {
                        (dot / (norm * query_norm)).clamp(0.0, 1.0)
                    } else {
                        0.0
                    }
                }
            };
            if score > 0.0 && score.is_finite() {
                results.push(SearchResult {
                    entry: &document.entry,
                    score,
                    matched_terms,
                });
            }
        }
        results.finish()
    }

    pub fn find_matches(&self, text: &str, limit: usize) -> Vec<MatchResult<'_>> {
        self.match_validated(
            text,
            &MatchOptions {
                limit,
                ..MatchOptions::default()
            },
        )
    }

    pub fn r#match(&self, text: &str, limit: usize) -> Vec<MatchResult<'_>> {
        self.find_matches(text, limit)
    }

    pub fn match_with_options(
        &self,
        text: &str,
        options: &MatchOptions,
    ) -> Result<Vec<MatchResult<'_>>, MatchError> {
        options.validate()?;
        Ok(self.match_validated(text, options))
    }

    fn match_validated(&self, input: &str, options: &MatchOptions) -> Vec<MatchResult<'_>> {
        let normalized = text::normalize(input);
        if normalized.is_empty() || options.limit == 0 {
            return Vec::new();
        }
        let query_terms: BTreeSet<&str> = text::tokens_normalized(&normalized).collect();
        let query_len = normalized.chars().count();
        let exact_ids = self.normalized_ids.get(&normalized);
        let candidates: Box<dyn Iterator<Item = &IndexedEntry> + '_> = match options.strategy {
            MatchStrategy::Exact => Box::new(
                exact_ids
                    .into_iter()
                    .flatten()
                    .map(|id| &self.documents[id]),
            ),
            MatchStrategy::TokenOverlap => Box::new(
                self.candidates(query_terms.iter().copied())
                    .into_iter()
                    .map(|id| &self.documents[id]),
            ),
            MatchStrategy::Fuzzy => Box::new(self.documents.values()),
        };
        let mut results = TopK::new(options.limit);
        for document in candidates {
            let shared = query_terms
                .iter()
                .filter(|term| document.terms.contains_key(**term))
                .count();
            let union = query_terms.len() + document.terms.len() - shared;
            let token_score = if union == 0 {
                0.0
            } else {
                shared as f64 / union as f64
            };
            if options.strategy == MatchStrategy::TokenOverlap
                && token_score < options.min_similarity
            {
                continue;
            }
            let max_len = query_len.max(document.char_length);
            if options.strategy == MatchStrategy::Fuzzy
                && (1.0 - query_len.abs_diff(document.char_length) as f64 / max_len as f64)
                    < options.min_similarity
            {
                continue;
            }
            let character = if normalized == document.normalized {
                1.0
            } else {
                let distance = if options.strategy == MatchStrategy::Fuzzy {
                    let Some(distance) = crate::matching::bounded_distance(
                        &normalized,
                        &document.normalized,
                        crate::matching::distance_limit(max_len, options.min_similarity),
                    ) else {
                        continue;
                    };
                    distance
                } else {
                    crate::matching::levenshtein_distance(&normalized, &document.normalized)
                };
                1.0 - distance as f64 / max_len as f64
            };
            let similarity = Similarity {
                exact: normalized == document.normalized,
                character,
                token: token_score,
            };
            let score = match options.strategy {
                MatchStrategy::Exact => {
                    if similarity.exact {
                        1.0
                    } else {
                        0.0
                    }
                }
                MatchStrategy::Fuzzy => similarity.character,
                MatchStrategy::TokenOverlap => similarity.token,
            };
            if score > 0.0 && score >= options.min_similarity {
                results.push(MatchResult {
                    entry: &document.entry,
                    score,
                    similarity,
                });
            }
        }
        results.finish()
    }

    pub fn duplicates(&self) -> Vec<Vec<&KnowledgeEntry>> {
        self.normalized_ids
            .values()
            .filter(|ids| ids.len() > 1)
            .map(|ids| ids.iter().map(|id| &self.documents[id].entry).collect())
            .collect()
    }

    pub fn relate(&self, id: &str, limit: usize) -> Result<Vec<RelationSuggestion>, IndexError> {
        let source = self
            .documents
            .get(id)
            .ok_or_else(|| IndexError::UnknownId(id.to_owned()))?;
        if limit == 0 {
            return Ok(Vec::new());
        }
        let source_terms: BTreeSet<&String> = source
            .terms
            .keys()
            .filter(|term| !text::is_stopword(term))
            .collect();
        let candidates = self.candidates(source_terms.iter().map(|term| term.as_str()));
        let mut suggestions = TopK::new(limit);
        for target_id in candidates {
            if target_id == id {
                continue;
            }
            let target_terms: BTreeSet<&String> = self.documents[target_id]
                .terms
                .keys()
                .filter(|term| !text::is_stopword(term))
                .collect();
            let shared_terms: Vec<String> = source_terms
                .intersection(&target_terms)
                .map(|term| (*term).clone())
                .collect();
            let union = source_terms.union(&target_terms).count();
            if !shared_terms.is_empty() {
                suggestions.push(RelationSuggestion {
                    source_id: id.to_owned(),
                    target_id: target_id.to_owned(),
                    kind: RelationKind::Related,
                    score: shared_terms.len() as f64 / union as f64,
                    shared_terms,
                });
            }
        }
        Ok(suggestions.finish())
    }

    pub fn to_json(&self) -> Result<String, IndexError> {
        Ok(serde_json::to_string_pretty(
            &self.entries().collect::<Vec<_>>(),
        )?)
    }

    pub fn from_json(json: &str) -> Result<Self, IndexError> {
        Self::from_entries(serde_json::from_str::<Vec<KnowledgeEntry>>(json)?)
    }

    fn candidates<'a>(&'a self, terms: impl IntoIterator<Item = &'a str>) -> BTreeSet<&'a str> {
        terms
            .into_iter()
            .filter_map(|term| self.postings.get(term))
            .flat_map(|postings| postings.keys().map(String::as_str))
            .collect()
    }
}
