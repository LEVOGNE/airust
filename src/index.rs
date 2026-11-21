#![cfg_attr(feature="shared-index", doc=include_str!("../docs/SHARED-INDEX.md"))]
use crate::context::Context;
use crate::knowledge::{KnowledgeEntry, RelationKind, RelationSuggestion};
use crate::matching::{MatchError, MatchOptions, MatchResult, MatchStrategy, Similarity};
use crate::ranking::TopK;
use crate::search::{tfidf_idf, SearchAlgorithm, SearchOptions, SearchResult};
use crate::text;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use thiserror::Error;

#[cfg(feature = "shared-index")]
pub(crate) type Shared<T> = std::sync::Arc<T>;
#[cfg(not(feature = "shared-index"))]
pub(crate) type Shared<T> = T;
pub(crate) fn shared_new<T>(value: T) -> Shared<T> {
    #[cfg(feature = "shared-index")]
    {
        std::sync::Arc::new(value)
    }
    #[cfg(not(feature = "shared-index"))]
    {
        value
    }
}
pub(crate) fn shared_mut<T: Clone>(value: &mut Shared<T>) -> &mut T {
    #[cfg(feature = "shared-index")]
    {
        std::sync::Arc::make_mut(value)
    }
    #[cfg(not(feature = "shared-index"))]
    {
        value
    }
}
pub(crate) fn shared_owned<T: Clone>(value: Shared<T>) -> T {
    #[cfg(feature = "shared-index")]
    {
        std::sync::Arc::unwrap_or_clone(value)
    }
    #[cfg(not(feature = "shared-index"))]
    {
        value
    }
}
pub(crate) fn shared_ref<T>(value: &Shared<T>) -> &T {
    #[cfg(feature = "shared-index")]
    {
        value.as_ref()
    }
    #[cfg(not(feature = "shared-index"))]
    {
        value
    }
}

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

#[cfg(feature = "execution")]
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ControlledIndexError {
    #[error(transparent)]
    Execution(#[from] crate::execution::ExecutionError),
    #[error(transparent)]
    Match(#[from] MatchError),
    #[error(transparent)]
    Index(#[from] IndexError),
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
    documents: BTreeMap<String, Shared<IndexedEntry>>,
    postings: BTreeMap<String, Shared<BTreeMap<Arc<str>, usize>>>,
    normalized_ids: BTreeMap<String, Shared<BTreeSet<Arc<str>>>>,
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
            let normalized = validate_entry(&entry)?;
            index.insert_validated(entry, normalized);
        }
        Ok(index)
    }

    pub fn upsert(&mut self, entry: KnowledgeEntry) -> Result<Option<KnowledgeEntry>, IndexError> {
        let entry = match self.replace_unchanged_entry(entry) {
            Ok(previous) => return Ok(Some(previous)),
            Err(entry) => entry,
        };
        let normalized = validate_entry(&entry)?;
        Ok(self.insert_validated(entry, normalized))
    }

    pub(crate) fn replace_unchanged_entry(
        &mut self,
        entry: KnowledgeEntry,
    ) -> Result<KnowledgeEntry, KnowledgeEntry> {
        let Some(old) = self.documents.get_mut(&entry.id) else {
            return Err(entry);
        };
        if old.entry.text != entry.text {
            return Err(entry);
        }
        if crate::knowledge::entries_identical(&old.entry, &entry) {
            return Ok(entry);
        }
        Ok(std::mem::replace(&mut shared_mut(old).entry, entry))
    }
    pub(crate) fn insert_validated(
        &mut self,
        entry: KnowledgeEntry,
        normalized: String,
    ) -> Option<KnowledgeEntry> {
        let terms = text::frequencies_normalized(&normalized);
        self.insert_analyzed(entry, normalized, terms)
    }

    pub(crate) fn insert_analyzed(
        &mut self,
        entry: KnowledgeEntry,
        normalized: String,
        terms: BTreeMap<String, usize>,
    ) -> Option<KnowledgeEntry> {
        let char_length = normalized.chars().count();
        let length = terms.values().sum();
        let previous = self.remove(&entry.id);
        let id: Arc<str> = Arc::from(entry.id.as_str());
        for (term, count) in &terms {
            shared_mut(self.postings.entry(term.clone()).or_default()).insert(id.clone(), *count);
        }
        shared_mut(self.normalized_ids.entry(normalized.clone()).or_default()).insert(id);
        self.total_tokens += length;
        self.documents.insert(
            entry.id.clone(),
            shared_new(IndexedEntry {
                entry,
                normalized,
                terms,
                length,
                char_length,
            }),
        );
        previous
    }

    pub fn remove(&mut self, id: &str) -> Option<KnowledgeEntry> {
        let document = shared_owned(self.documents.remove(id)?);
        self.total_tokens -= document.length;
        for term in document.terms.keys() {
            if let Some(postings) = self.postings.get_mut(term) {
                shared_mut(postings).remove(id);
                if postings.is_empty() {
                    self.postings.remove(term);
                }
            }
        }
        if let Some(ids) = self.normalized_ids.get_mut(&document.normalized) {
            shared_mut(ids).remove(id);
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
    #[cfg(feature = "context-assembly")]
    pub(crate) fn normalized_text(&self, id: &str) -> Option<&str> {
        self.documents.get(id).map(|d| d.normalized.as_str())
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
        self.search_analyzed(&query_terms, options, |_| true)
    }

    pub(crate) fn search_analyzed(
        &self,
        query_terms: &BTreeMap<String, usize>,
        options: &SearchOptions,
        includes: impl Fn(&str) -> bool,
    ) -> Vec<SearchResult<'_>> {
        match self.search_analyzed_checked(
            query_terms,
            options,
            includes,
            || Ok::<_, std::convert::Infallible>(()),
            #[cfg(feature = "execution")]
            None,
        ) {
            Ok(results) => results,
            Err(error) => match error {},
        }
    }
    #[cfg(feature = "execution")]
    pub fn search_controlled(
        &self,
        query: &str,
        options: &SearchOptions,
        control: &crate::execution::WorkControl,
    ) -> Result<Vec<SearchResult<'_>>, crate::execution::ExecutionError> {
        control.checkpoint()?;
        let terms = text::term_frequencies(query);
        self.search_analyzed_checked(
            &terms,
            options,
            |_| true,
            || control.checkpoint(),
            Some(control),
        )
    }
    pub(crate) fn search_analyzed_checked<E>(
        &self,
        query_terms: &BTreeMap<String, usize>,
        options: &SearchOptions,
        includes: impl Fn(&str) -> bool,
        mut checkpoint: impl FnMut() -> Result<(), E>,
        #[cfg(feature = "execution")] control: Option<&crate::execution::WorkControl>,
    ) -> Result<Vec<SearchResult<'_>>, E> {
        checkpoint()?;
        if options.limit == 0 || self.is_empty() {
            return Ok(Vec::new());
        }
        #[cfg(feature = "execution")]
        if let Some(control) = control {
            control.progress(crate::execution::WorkPhase::CollectingCandidates, 0, None);
        }
        #[cfg(feature = "execution")]
        let mut visited = 0usize;
        let mut candidates = BTreeSet::new();
        for term in query_terms.keys() {
            checkpoint()?;
            if let Some(postings) = self.postings.get(term) {
                for id in postings.keys() {
                    checkpoint()?;
                    #[cfg(feature = "execution")]
                    {
                        visited += 1;
                        if visited.is_multiple_of(512) {
                            if let Some(control) = control {
                                control.progress(
                                    crate::execution::WorkPhase::CollectingCandidates,
                                    visited,
                                    None,
                                );
                            }
                        }
                    }
                    candidates.insert(id.as_ref());
                }
            }
        }
        #[cfg(feature = "execution")]
        let candidate_count = candidates.len();
        #[cfg(feature = "execution")]
        if let Some(control) = control {
            control.progress(
                crate::execution::WorkPhase::CollectingCandidates,
                visited,
                Some(visited),
            );
            control.progress(
                crate::execution::WorkPhase::Ranking,
                0,
                Some(candidate_count),
            );
        }
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
        for (position, id) in candidates.into_iter().enumerate() {
            #[cfg(not(feature = "execution"))]
            let _ = position;
            checkpoint()?;
            #[cfg(feature = "execution")]
            if position.is_multiple_of(512) {
                if let Some(control) = control {
                    control.progress(
                        crate::execution::WorkPhase::Ranking,
                        position,
                        Some(candidate_count),
                    );
                }
            }
            let document = &self.documents[id];
            if options
                .context
                .as_ref()
                .is_some_and(|context| !context.includes(&document.entry))
            {
                continue;
            }
            if !includes(id) {
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
        checkpoint()?;
        let result = results.finish();
        #[cfg(feature = "execution")]
        if let Some(control) = control {
            control.progress(
                crate::execution::WorkPhase::Ranking,
                candidate_count,
                Some(candidate_count),
            );
        }
        checkpoint()?;
        Ok(result)
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
        match self.match_checked(
            input,
            options,
            || Ok::<_, std::convert::Infallible>(()),
            #[cfg(feature = "execution")]
            None,
        ) {
            Ok(result) => result,
            Err(error) => match error {},
        }
    }

    #[cfg(feature = "execution")]
    pub fn match_controlled(
        &self,
        input: &str,
        options: &MatchOptions,
        control: &crate::execution::WorkControl,
    ) -> Result<Vec<MatchResult<'_>>, ControlledIndexError> {
        control.checkpoint()?;
        options.validate()?;
        Ok(self.match_checked(input, options, || control.checkpoint(), Some(control))?)
    }

    fn match_checked<E>(
        &self,
        input: &str,
        options: &MatchOptions,
        mut checkpoint: impl FnMut() -> Result<(), E>,
        #[cfg(feature = "execution")] control: Option<&crate::execution::WorkControl>,
    ) -> Result<Vec<MatchResult<'_>>, E> {
        checkpoint()?;
        let normalized = text::normalize(input);
        if normalized.is_empty() || options.limit == 0 {
            return Ok(Vec::new());
        }
        let query_terms: BTreeSet<&str> = text::tokens_normalized(&normalized).collect();
        let query_len = normalized.chars().count();
        let exact_ids = self.normalized_ids.get(&normalized);
        let candidates: Box<dyn Iterator<Item = &IndexedEntry> + '_> = match options.strategy {
            MatchStrategy::Exact => Box::new(
                exact_ids
                    .into_iter()
                    .flat_map(|ids| ids.iter())
                    .map(|id| shared_ref(&self.documents[id.as_ref()])),
            ),
            MatchStrategy::TokenOverlap => Box::new(
                self.candidates_checked(
                    query_terms.iter().copied(),
                    &mut checkpoint,
                    #[cfg(feature = "execution")]
                    control.is_some(),
                    #[cfg(not(feature = "execution"))]
                    false,
                )?
                .into_iter()
                .map(|id| shared_ref(&self.documents[id])),
            ),
            MatchStrategy::Fuzzy => Box::new(self.documents.values().map(shared_ref)),
        };
        let mut results = TopK::new(options.limit);
        #[cfg(feature = "execution")]
        if let Some(control) = control {
            control.progress(crate::execution::WorkPhase::Ranking, 0, None);
        }
        #[cfg(feature = "execution")]
        let mut visited = 0usize;
        for document in candidates {
            checkpoint()?;
            #[cfg(feature = "execution")]
            if let Some(control) = control {
                visited += 1;
                if visited.is_multiple_of(128) {
                    control.progress(crate::execution::WorkPhase::Ranking, visited, None);
                }
            }
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
            } else if options.strategy == MatchStrategy::TokenOverlap {
                0.0
            } else {
                let distance = if options.strategy == MatchStrategy::Fuzzy {
                    let Some(distance) = checked_distance(
                        &normalized,
                        &document.normalized,
                        Some(crate::matching::distance_limit(
                            max_len,
                            options.min_similarity,
                        )),
                        &mut checkpoint,
                        #[cfg(feature = "execution")]
                        control,
                    )?
                    else {
                        continue;
                    };
                    distance
                } else {
                    checked_distance(
                        &normalized,
                        &document.normalized,
                        None,
                        &mut checkpoint,
                        #[cfg(feature = "execution")]
                        control,
                    )?
                    .unwrap_or(max_len)
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
        let mut results = results.finish();
        if options.strategy == MatchStrategy::TokenOverlap {
            for result in &mut results {
                checkpoint()?;
                if !result.similarity.exact {
                    let document = &self.documents[&result.entry.id];
                    let distance = checked_distance(
                        &normalized,
                        &document.normalized,
                        None,
                        &mut checkpoint,
                        #[cfg(feature = "execution")]
                        control,
                    )?
                    .unwrap_or(query_len.max(document.char_length));
                    result.similarity.character =
                        1.0 - distance as f64 / query_len.max(document.char_length) as f64;
                }
            }
        }
        checkpoint()?;
        Ok(results)
    }

    pub fn duplicates(&self) -> Vec<Vec<&KnowledgeEntry>> {
        self.normalized_ids
            .values()
            .filter(|ids| ids.len() > 1)
            .map(|ids| {
                ids.iter()
                    .map(|id| &self.documents[id.as_ref()].entry)
                    .collect()
            })
            .collect()
    }

    pub fn relate(&self, id: &str, limit: usize) -> Result<Vec<RelationSuggestion>, IndexError> {
        let source = self
            .documents
            .get(id)
            .ok_or_else(|| IndexError::UnknownId(id.to_owned()))?;
        match self.relate_checked(
            source,
            id,
            limit,
            || Ok::<_, std::convert::Infallible>(()),
            #[cfg(feature = "execution")]
            None,
        ) {
            Ok(result) => Ok(result),
            Err(error) => match error {},
        }
    }

    #[cfg(feature = "execution")]
    pub fn relate_controlled(
        &self,
        id: &str,
        limit: usize,
        control: &crate::execution::WorkControl,
    ) -> Result<Vec<RelationSuggestion>, ControlledIndexError> {
        control.checkpoint()?;
        let source = self
            .documents
            .get(id)
            .ok_or_else(|| IndexError::UnknownId(id.to_owned()))?;
        Ok(self.relate_checked(source, id, limit, || control.checkpoint(), Some(control))?)
    }

    fn relate_checked<E>(
        &self,
        source: &IndexedEntry,
        id: &str,
        limit: usize,
        mut checkpoint: impl FnMut() -> Result<(), E>,
        #[cfg(feature = "execution")] control: Option<&crate::execution::WorkControl>,
    ) -> Result<Vec<RelationSuggestion>, E> {
        checkpoint()?;
        if limit == 0 {
            return Ok(Vec::new());
        }
        let source_terms: BTreeSet<&String> = source
            .terms
            .keys()
            .filter(|term| !text::is_stopword(term))
            .collect();
        let candidates = self.candidates_checked(
            source_terms.iter().map(|term| term.as_str()),
            &mut checkpoint,
            #[cfg(feature = "execution")]
            control.is_some(),
            #[cfg(not(feature = "execution"))]
            false,
        )?;
        let mut suggestions = TopK::new(limit);
        #[cfg(feature = "execution")]
        if let Some(control) = control {
            control.progress(crate::execution::WorkPhase::Ranking, 0, None);
        }
        #[cfg(feature = "execution")]
        let mut visited = 0usize;
        for target_id in candidates {
            checkpoint()?;
            #[cfg(feature = "execution")]
            if let Some(control) = control {
                visited += 1;
                if visited.is_multiple_of(128) {
                    control.progress(crate::execution::WorkPhase::Ranking, visited, None);
                }
            }
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
        checkpoint()?;
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

    fn candidates_checked<'a, 't, E>(
        &'a self,
        terms: impl IntoIterator<Item = &'t str>,
        checkpoint: &mut impl FnMut() -> Result<(), E>,
        controlled: bool,
    ) -> Result<BTreeSet<&'a str>, E> {
        if !controlled {
            return Ok(terms
                .into_iter()
                .filter_map(|term| self.postings.get(term))
                .flat_map(|postings| postings.keys().map(|id| id.as_ref()))
                .collect());
        }
        let mut result = BTreeSet::new();
        for term in terms {
            checkpoint()?;
            if let Some(postings) = self.postings.get(term) {
                for id in postings.keys() {
                    checkpoint()?;
                    result.insert(id.as_ref());
                }
            }
        }
        Ok(result)
    }
}

pub(crate) fn validate_entry(entry: &KnowledgeEntry) -> Result<String, IndexError> {
    check_source_entry(entry)?;
    let normalized = text::normalize(&entry.text);
    if normalized.is_empty() {
        return Err(IndexError::EmptyText(entry.id.clone()));
    }
    Ok(normalized)
}

pub(crate) fn check_source_entry(entry: &KnowledgeEntry) -> Result<(), IndexError> {
    if entry.id.trim().is_empty() {
        return Err(IndexError::EmptyId);
    }
    if entry.text.trim().is_empty() {
        return Err(IndexError::EmptyText(entry.id.clone()));
    }
    Ok(())
}

#[cfg(feature = "cache")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct CacheDocumentHeader {
    pub id: String,
    pub normalized: String,
    pub length: usize,
    pub char_length: usize,
    pub term_count: usize,
}
#[cfg(feature = "cache")]
#[derive(serde::Serialize)]
struct CacheDocumentRef<'a> {
    id: &'a str,
    normalized: &'a str,
    length: usize,
    char_length: usize,
    term_count: usize,
}
#[cfg(feature = "cache")]
#[derive(Debug, Clone)]
pub(crate) struct CacheDocument {
    pub id: String,
    pub normalized: String,
    pub length: usize,
    pub char_length: usize,
    pub terms: BTreeMap<String, usize>,
}
#[cfg(feature = "cache")]
impl KnowledgeIndex {
    pub(crate) fn cache_term_bindings(&self) -> Result<usize, crate::cache::CacheError> {
        self.documents.values().try_fold(0usize, |total, document| {
            total
                .checked_add(document.terms.len())
                .ok_or(crate::cache::CacheError::Limit("term bindings"))
        })
    }
    pub(crate) fn write_cached_documents(
        &self,
        writer: &mut impl std::io::Write,
    ) -> Result<(), postcard::Error> {
        for document in self.documents.values() {
            postcard::to_io(
                &CacheDocumentRef {
                    id: &document.entry.id,
                    normalized: &document.normalized,
                    length: document.length,
                    char_length: document.char_length,
                    term_count: document.terms.len(),
                },
                &mut *writer,
            )?;
            for item in &document.terms {
                postcard::to_io(&item, &mut *writer)?;
            }
        }
        Ok(())
    }
}

#[cfg(feature = "cache")]
pub(crate) struct PreparedCache {
    documents: BTreeMap<String, CacheDocument>,
    total_tokens: usize,
}
#[cfg(feature = "cache")]
pub(crate) fn prepare_cached_documents(
    originals: &[KnowledgeEntry],
    documents: Vec<CacheDocument>,
) -> Result<PreparedCache, crate::cache::CacheError> {
    use crate::cache::CacheError;
    let ids: BTreeSet<_> = originals.iter().map(|entry| entry.id.as_str()).collect();
    if documents.len() != originals.len() || ids.len() != originals.len() {
        return Err(CacheError::InvalidData("cached document count"));
    }
    let mut prepared = BTreeMap::new();
    let mut total_tokens = 0usize;
    for document in documents {
        if !ids.contains(document.id.as_str()) || prepared.contains_key(&document.id) {
            return Err(CacheError::InvalidData("cached document IDs"));
        }
        if document.normalized.trim().is_empty()
            || document.char_length != document.normalized.chars().count()
        {
            return Err(CacheError::InvalidData("cached normalized text"));
        }
        let mut length = 0usize;
        for (term, &frequency) in &document.terms {
            let mut words = text::tokens_normalized(term);
            if frequency == 0 || words.next() != Some(term.as_str()) || words.next().is_some() {
                return Err(CacheError::InvalidData("cached term"));
            }
            length = length
                .checked_add(frequency)
                .ok_or(CacheError::InvalidData("cached token count overflow"))?;
        }
        if length != document.length || length > document.char_length {
            return Err(CacheError::InvalidData("cached document length"));
        }
        total_tokens = total_tokens
            .checked_add(length)
            .ok_or(CacheError::InvalidData("cached total token count overflow"))?;
        prepared.insert(document.id.clone(), document);
    }
    Ok(PreparedCache {
        documents: prepared,
        total_tokens,
    })
}
#[cfg(feature = "cache")]
impl PreparedCache {
    pub(crate) fn restore(mut self, originals: Vec<KnowledgeEntry>) -> KnowledgeIndex {
        let mut index = KnowledgeIndex {
            total_tokens: self.total_tokens,
            ..KnowledgeIndex::new()
        };
        for entry in originals {
            let cached = self
                .documents
                .remove(&entry.id)
                .expect("validated cached source ID");
            let id: Arc<str> = Arc::from(entry.id.as_str());
            for (term, &frequency) in &cached.terms {
                shared_mut(index.postings.entry(term.clone()).or_default())
                    .insert(id.clone(), frequency);
            }
            shared_mut(
                index
                    .normalized_ids
                    .entry(cached.normalized.clone())
                    .or_default(),
            )
            .insert(id);
            index.documents.insert(
                entry.id.clone(),
                shared_new(IndexedEntry {
                    entry,
                    normalized: cached.normalized,
                    terms: cached.terms,
                    length: cached.length,
                    char_length: cached.char_length,
                }),
            );
        }
        index
    }
}

#[cfg(test)]
mod token_match_tests {
    use super::*;

    fn check_complete_comparison(
        index: &KnowledgeIndex,
        query: &str,
        threshold: f64,
        limit: usize,
    ) {
        let mut expected: Vec<_> = index
            .entries()
            .map(|entry| (entry, crate::matching::compare(query, &entry.text)))
            .filter(|(_, similarity)| similarity.token > 0.0 && similarity.token >= threshold)
            .collect();
        expected.sort_by(|(left, a), (right, b)| {
            b.token
                .total_cmp(&a.token)
                .then_with(|| left.id.cmp(&right.id))
        });
        expected.truncate(limit);
        let actual = index
            .match_with_options(
                query,
                &MatchOptions {
                    strategy: MatchStrategy::TokenOverlap,
                    min_similarity: threshold,
                    limit,
                },
            )
            .unwrap();
        assert_eq!(actual.len(), expected.len());
        for (actual, (entry, similarity)) in actual.iter().zip(expected) {
            assert_eq!(actual.entry, entry);
            assert_eq!(actual.score, similarity.token);
            assert_eq!(actual.similarity, similarity);
        }
    }

    #[test]
    fn token_top_k_preserves_full_comparison_scores_and_unicode_explanations() {
        let texts = [
            "Alpha beta",
            "beta alpha",
            "ALPHA βeta",
            "alpha delta γ",
            "Über Straße ä😀",
            "alpha alpha beta",
            "alpha",
            "gamma",
            "🧠",
            "alpha\u{200d} beta",
            "café alpha",
            "cafe\u{301} alpha",
            "alpha—beta",
        ];
        let index = KnowledgeIndex::from_entries(
            texts
                .iter()
                .enumerate()
                .map(|(i, text)| KnowledgeEntry::new(format!("entry-{i:02}"), *text)),
        )
        .unwrap();
        for query in texts.into_iter().chain(["", "   ", "!", "東京 alpha"]) {
            for threshold in [0.0, f64::EPSILON, 0.2, 0.5, 0.8, 1.0] {
                for limit in [0, 1, 3, 99, usize::MAX] {
                    check_complete_comparison(&index, query, threshold, limit);
                }
            }
        }
    }

    #[test]
    fn token_match_explanations_survive_changes_and_json_roundtrips() {
        let mut index = KnowledgeIndex::from_entries([
            KnowledgeEntry::new("a", "memory allocation"),
            KnowledgeEntry::new("b", "allocation memory memory"),
            KnowledgeEntry::new("c", "memory allocation function"),
            KnowledgeEntry::new("d", "Speicher Verwaltung"),
        ])
        .unwrap();
        index.remove("a");
        index
            .upsert(KnowledgeEntry::new("d", "memory allocation 😀"))
            .unwrap();
        let restored = KnowledgeIndex::from_json(&index.to_json().unwrap()).unwrap();
        for index in [&index, &restored] {
            for query in ["memory allocation", "Speicher", "MEMORY", "😀 allocation"] {
                for limit in [0, 1, 10] {
                    check_complete_comparison(index, query, 0.0, limit);
                }
            }
        }
    }

    #[test]
    fn token_ties_keep_id_order_and_character_scores_for_long_text() {
        let index = KnowledgeIndex::from_entries((0..64).rev().map(|i| {
            KnowledgeEntry::new(
                format!("entry-{i:02}"),
                format!("{} {}", "common memory 😀 ".repeat(32), "β".repeat(i)),
            )
        }))
        .unwrap();
        let query = "memory common ".repeat(16);
        for limit in [1, 10, 100] {
            check_complete_comparison(&index, &query, 0.0, limit);
        }
    }
}

#[cfg(test)]
mod identifier_tests {
    use super::*;

    #[test]
    fn identifiers_keep_original_unicode_and_control_bytes() {
        let mut ids = vec![
            "é".to_owned(),
            "e\u{301}".to_owned(),
            "a\0b".to_owned(),
            " a ".to_owned(),
            format!("{}a", "資料/😀/".repeat(100)),
            format!("{}b", "資料/😀/".repeat(100)),
        ];
        let index = KnowledgeIndex::from_entries(
            ids.iter().map(|id| KnowledgeEntry::new(id, "shared words")),
        )
        .unwrap();
        ids.sort();
        assert_eq!(index.len(), ids.len());
        let json = index.to_json().unwrap();
        let restored = KnowledgeIndex::from_json(&json).unwrap();
        assert_eq!(restored.to_json().unwrap(), json);
        for view in [&index, &restored] {
            for id in &ids {
                assert_eq!(&view.get(id).unwrap().id, id);
            }
            let hits = view.search("shared", ids.len());
            assert_eq!(
                hits.iter().map(|hit| &hit.entry.id).collect::<Vec<_>>(),
                ids.iter().collect::<Vec<_>>()
            );
            let groups = view.duplicates();
            assert_eq!(groups.len(), 1);
            assert_eq!(
                groups[0].iter().map(|entry| &entry.id).collect::<Vec<_>>(),
                ids.iter().collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn retained_views_keep_long_identifiers_after_removal_and_clear() {
        let prefix = "long/document/資料/".repeat(32);
        let ids: Vec<_> = (0..32).map(|i| format!("{prefix}{i:03}")).collect();
        let mut index = KnowledgeIndex::from_entries(
            ids.iter()
                .map(|id| KnowledgeEntry::new(id, "original shared words")),
        )
        .unwrap();
        let original_json = index.to_json().unwrap();
        let retained = index.clone();
        for id in ids.iter().take(16) {
            index
                .upsert(KnowledgeEntry::new(id, "replacement marker"))
                .unwrap();
        }
        for id in ids.iter().skip(16) {
            assert_eq!(index.remove(id).unwrap().id, *id);
        }
        let rebuilt = KnowledgeIndex::from_entries(index.entries().cloned()).unwrap();
        assert_eq!(index.to_json().unwrap(), rebuilt.to_json().unwrap());
        assert_eq!(index.search("replacement", 100).len(), 16);
        index.clear();
        assert!(index.is_empty());
        assert_eq!(retained.to_json().unwrap(), original_json);
        for id in &ids {
            assert_eq!(&retained.get(id).unwrap().id, id);
        }
        assert_eq!(
            retained.find_matches("original shared words", 100).len(),
            32
        );
        assert_eq!(retained.relate(&ids[0], 100).unwrap().len(), 31);
    }
}

#[cfg(all(test, feature = "execution"))]
mod execution_tests {
    use super::*;
    #[test]
    fn cancellation_inside_shared_candidate_build_returns_no_partial_results() {
        let index = KnowledgeIndex::from_entries(
            (0..1000).map(|i| KnowledgeEntry::new(i.to_string(), "Rust ownership")),
        )
        .unwrap();
        let terms = text::term_frequencies("Rust");
        let mut checkpoints = 0;
        let result = index.search_analyzed_checked(
            &terms,
            &SearchOptions::default(),
            |_| true,
            || {
                checkpoints += 1;
                if checkpoints == 30 {
                    Err(crate::execution::ExecutionError::Cancelled)
                } else {
                    Ok(())
                }
            },
            None,
        );
        assert!(matches!(
            result,
            Err(crate::execution::ExecutionError::Cancelled)
        ));
        assert_eq!(checkpoints, 30);
    }
}
#[cfg(all(test, feature = "shared-index"))]
mod sharing_tests {
    use super::*;
    use std::sync::Arc;
    #[test]
    fn copies_share_immutable_payloads_and_detach_only_modified_bindings() {
        let original = KnowledgeIndex::from_entries([
            KnowledgeEntry::new("a", "common one"),
            KnowledgeEntry::new("b", "common two"),
            KnowledgeEntry::new("c", "untouched unique"),
        ])
        .unwrap();
        let mut changed = original.clone();
        assert!(Arc::ptr_eq(
            &original.documents["a"],
            &changed.documents["a"]
        ));
        assert!(Arc::ptr_eq(
            &original.postings["common"],
            &changed.postings["common"]
        ));
        changed
            .upsert(KnowledgeEntry::new("a", "replacement different"))
            .unwrap();
        assert!(Arc::ptr_eq(
            &original.documents["b"],
            &changed.documents["b"]
        ));
        assert!(!Arc::ptr_eq(
            &original.postings["common"],
            &changed.postings["common"]
        ));
        assert!(Arc::ptr_eq(
            &original.postings["unique"],
            &changed.postings["unique"]
        ));
        assert!(original.postings["common"].contains_key("a"));
        assert!(!changed.postings["common"].contains_key("a"));
    }
    #[test]
    fn duplicate_sets_detach_without_leaking_removal_to_old_views() {
        let original = KnowledgeIndex::from_entries([
            KnowledgeEntry::new("a", "same text"),
            KnowledgeEntry::new("b", "same text"),
        ])
        .unwrap();
        let mut changed = original.clone();
        assert!(Arc::ptr_eq(
            &original.normalized_ids["same text"],
            &changed.normalized_ids["same text"]
        ));
        changed.remove("a");
        assert_eq!(original.normalized_ids["same text"].len(), 2);
        assert_eq!(changed.normalized_ids["same text"].len(), 1);
        changed.remove("b");
        assert!(changed.normalized_ids.is_empty());
        assert_eq!(original.len(), 2);
    }
}

fn checked_distance<E>(
    a: &str,
    b: &str,
    maximum: Option<usize>,
    checkpoint: &mut impl FnMut() -> Result<(), E>,
    #[cfg(feature = "execution")] control: Option<&crate::execution::WorkControl>,
) -> Result<Option<usize>, E> {
    #[cfg(feature = "execution")]
    if control.is_some() {
        return crate::matching::distance_checked(a, b, maximum, checkpoint);
    }
    let _ = checkpoint;
    Ok(match maximum {
        Some(maximum) => crate::matching::bounded_distance(a, b, maximum),
        None => Some(crate::matching::levenshtein_distance(a, b)),
    })
}
