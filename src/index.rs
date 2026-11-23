#![cfg_attr(feature="shared-index", doc=include_str!("../docs/SHARED-INDEX.md"))]
use crate::context::Context;
use crate::knowledge::{KnowledgeEntry, RelationKind, RelationSuggestion};
use crate::matching::{MatchError, MatchOptions, MatchResult, MatchStrategy, Similarity};
use crate::ranking::TopK;
use crate::search::{tfidf_idf, SearchAlgorithm, SearchOptions, SearchResult};
use crate::text;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap};
use std::sync::{Arc, OnceLock, RwLock, RwLockReadGuard};
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
    normalized: Arc<str>,
    terms: BTreeMap<Arc<str>, usize>,
    length: usize,
    char_length: usize,
}

struct RankedDocument<'a> {
    document: &'a IndexedEntry,
    score: f64,
}
impl crate::ranking::Ranked for RankedDocument<'_> {
    fn score(&self) -> f64 {
        self.score
    }
    fn id(&self) -> &str {
        &self.document.entry.id
    }
}
struct QueryTerm<'a> {
    text: &'a str,
    frequency: f64,
    idf: f64,
    postings: &'a BTreeMap<Arc<str>, usize>,
}

struct TermCursor<'a> {
    postings: &'a BTreeMap<Arc<str>, usize>,
    rest: std::collections::btree_map::Range<'a, Arc<str>, usize>,
    next: Option<(&'a Arc<str>, &'a usize)>,
}

impl<'a> TermCursor<'a> {
    fn new(postings: &'a BTreeMap<Arc<str>, usize>, first: &str) -> Self {
        let mut rest = postings
            .range::<str, _>((std::ops::Bound::Included(first), std::ops::Bound::Unbounded));
        let next = rest.next();
        Self {
            postings,
            rest,
            next,
        }
    }

    fn frequency<E>(
        &mut self,
        id: &Arc<str>,
        checkpoint: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<Option<usize>, E> {
        let mut skipped = 0;
        while let Some((key, frequency)) = self.next {
            let order = if Arc::ptr_eq(key, id) {
                std::cmp::Ordering::Equal
            } else {
                key.as_ref().cmp(id.as_ref())
            };
            match order {
                std::cmp::Ordering::Equal => {
                    let frequency = *frequency;
                    self.next = self.rest.next();
                    return Ok(Some(frequency));
                }
                std::cmp::Ordering::Greater => return Ok(None),
                std::cmp::Ordering::Less => {
                    skipped += 1;
                    if skipped == 32 {
                        checkpoint()?;
                        self.rest = self.postings.range::<str, _>((
                            std::ops::Bound::Included(id.as_ref()),
                            std::ops::Bound::Unbounded,
                        ));
                    }
                    self.next = self.rest.next();
                }
            }
        }
        Ok(None)
    }
}

fn matching_terms_checked<E>(
    prepared: &[QueryTerm<'_>],
    document: &IndexedEntry,
    id: &Arc<str>,
    cursors: Option<&mut [TermCursor<'_>]>,
    checkpoint: &mut impl FnMut() -> Result<(), E>,
    mut matched: impl FnMut(&QueryTerm<'_>, usize),
) -> Result<(), E> {
    if let Some(cursors) = cursors {
        checkpoint()?;
        for (term, cursor) in prepared.iter().zip(cursors) {
            if let Some(frequency) = cursor.frequency(id, checkpoint)? {
                matched(term, frequency);
            }
        }
    } else {
        dictionary_terms_checked(prepared, document, checkpoint, matched)?;
    }
    Ok(())
}

fn dictionary_terms_checked<E>(
    prepared: &[QueryTerm<'_>],
    document: &IndexedEntry,
    checkpoint: &mut impl FnMut() -> Result<(), E>,
    mut matched: impl FnMut(&QueryTerm<'_>, usize),
) -> Result<(), E> {
    if prepared.len() > 32 && prepared.len() > document.terms.len().saturating_mul(2) {
        for (position, (word, frequency)) in document.terms.iter().enumerate() {
            if position.is_multiple_of(128) {
                checkpoint()?;
            }
            if let Ok(index) = prepared.binary_search_by(|term| term.text.cmp(word.as_ref())) {
                matched(&prepared[index], *frequency);
            }
        }
    } else {
        for (position, term) in prepared.iter().enumerate() {
            if position.is_multiple_of(128) {
                checkpoint()?;
            }
            if let Some(frequency) = document.terms.get(term.text) {
                matched(term, *frequency);
            }
        }
    }
    Ok(())
}

fn matched_terms_checked<E>(
    prepared: &[QueryTerm<'_>],
    document: &IndexedEntry,
    checkpoint: &mut impl FnMut() -> Result<(), E>,
) -> Result<Vec<String>, E> {
    let mut terms = Vec::new();
    dictionary_terms_checked(prepared, document, checkpoint, |term, _| {
        terms.push(term.text.to_owned());
    })?;
    Ok(terms)
}

struct PostingCursor<'a> {
    id: &'a Arc<str>,
    rest: std::collections::btree_map::Keys<'a, Arc<str>, usize>,
}
impl PartialEq for PostingCursor<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for PostingCursor<'_> {}
impl PartialOrd for PostingCursor<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for PostingCursor<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.id.cmp(self.id)
    }
}

#[inline(never)]
fn merge_pair_checked<'a, E>(
    left: &'a BTreeMap<Arc<str>, usize>,
    right: &'a BTreeMap<Arc<str>, usize>,
    candidates: &mut Vec<&'a Arc<str>>,
    visit: &mut impl FnMut() -> Result<(), E>,
) -> Result<(), E> {
    candidates.reserve(left.len().max(right.len()).min(32));
    let mut left = left.keys();
    let mut right = right.keys();
    let mut first = left.next();
    let mut second = right.next();
    while let (Some(a), Some(b)) = (first, second) {
        let order = if Arc::ptr_eq(a, b) {
            std::cmp::Ordering::Equal
        } else {
            a.as_ref().cmp(b.as_ref())
        };
        visit()?;
        match order {
            std::cmp::Ordering::Less => {
                candidates.push(a);
                first = left.next();
            }
            std::cmp::Ordering::Greater => {
                candidates.push(b);
                second = right.next();
            }
            std::cmp::Ordering::Equal => {
                visit()?;
                candidates.push(a);
                first = left.next();
                second = right.next();
            }
        }
    }
    for id in first.into_iter().chain(left).chain(second).chain(right) {
        visit()?;
        candidates.push(id);
    }
    Ok(())
}

struct TokenBatch<'a> {
    counts: Vec<(&'a str, usize)>,
    length: usize,
}
impl<'a> TokenBatch<'a> {
    const LIMIT: usize = 1024;

    fn take(tokens: &mut impl Iterator<Item = &'a str>, bytes: usize) -> Self {
        let first = tokens.next();
        let second = first.and_then(|_| tokens.next());
        let capacity = if second.is_some() {
            bytes.div_ceil(5).clamp(4, Self::LIMIT).next_power_of_two()
        } else {
            usize::from(first.is_some())
        };
        let mut counts = Vec::with_capacity(capacity);
        counts.extend(first.map(|word| (word, 1)));
        counts.extend(second.map(|word| (word, 1)));
        counts.extend(
            tokens
                .take(Self::LIMIT - counts.len())
                .map(|word| (word, 1)),
        );
        let length = counts.len();
        counts.sort_unstable_by(|left, right| left.0.cmp(right.0));
        counts.dedup_by(|current, previous| {
            if current.0 == previous.0 {
                previous.1 += current.1;
                true
            } else {
                false
            }
        });
        Self { counts, length }
    }

    fn into_terms(self, index: &KnowledgeIndex) -> BTreeMap<Arc<str>, usize> {
        self.counts
            .into_iter()
            .map(|(word, count)| (index.intern_term(word), count))
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct IndexStats {
    pub entries: usize,
    pub terms: usize,
    pub tokens: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct NormKey(usize);

impl NormKey {
    fn from_id(id: &Arc<str>) -> Self {
        Self(Arc::as_ptr(id).addr())
    }
}

#[derive(Debug, Default)]
struct TfidfNorms {
    values: RwLock<HashMap<NormKey, f64>>,
}

#[derive(Clone, Copy, Default)]
struct QueryWeight {
    key: usize,
    idf: f64,
}

struct QueryWeights<const N: usize = 256> {
    slots: [QueryWeight; N],
}

impl<const N: usize> Default for QueryWeights<N> {
    fn default() -> Self {
        Self {
            slots: [QueryWeight::default(); N],
        }
    }
}

impl<const N: usize> QueryWeights<N> {
    fn get(&mut self, key: usize, compute: impl FnOnce() -> f64) -> f64 {
        let slot = &mut self.slots[((key >> 4) ^ (key >> 12)) & (N - 1)];
        if slot.key != key {
            slot.idf = compute();
            slot.key = key;
        }
        slot.idf
    }
}

struct NormQuery<'a> {
    cache: &'a TfidfNorms,
    values: Option<RwLockReadGuard<'a, HashMap<NormKey, f64>>>,
    read_hits: usize,
    cold_terms: usize,
    small: Option<QueryWeights<8>>,
    weights: Option<Box<QueryWeights>>,
}

impl<'a> NormQuery<'a> {
    fn new(cache: &'a TfidfNorms) -> Self {
        Self {
            cache,
            values: None,
            read_hits: 0,
            cold_terms: 0,
            small: None,
            weights: None,
        }
    }

    fn release(&mut self) {
        self.values.take();
        self.read_hits = 0;
    }

    fn get(&mut self, key: NormKey) -> Option<f64> {
        if self.values.is_none() {
            self.values = self.cache.values.try_read().ok();
        }
        let value = self.values.as_ref()?.get(&key).copied();
        if value.is_some() {
            self.read_hits += 1;
        }
        if value.is_none() || self.read_hits == 128 {
            self.release();
        }
        value
    }

    fn weight(&mut self, key: usize, compute: impl FnOnce() -> f64) -> f64 {
        if self.cold_terms < 128 {
            return self
                .small
                .get_or_insert_with(QueryWeights::default)
                .get(key, compute);
        }
        let weights = self.weights.get_or_insert_with(|| {
            let mut weights = Box::new(QueryWeights::default());
            if let Some(small) = self.small.take() {
                for slot in small.slots {
                    if slot.key != 0 {
                        weights.get(slot.key, || slot.idf);
                    }
                }
            }
            weights
        });
        weights.get(key, compute)
    }
}

#[derive(Debug, Clone, Default)]
pub struct KnowledgeIndex {
    documents: BTreeMap<Arc<str>, Shared<IndexedEntry>>,
    postings: HashMap<Arc<str>, Shared<BTreeMap<Arc<str>, usize>>>,
    normalized_ids: BTreeMap<Arc<str>, Shared<BTreeSet<Arc<str>>>>,
    total_tokens: usize,
    vocabulary_capacity: usize,
    tfidf_norms: OnceLock<Arc<TfidfNorms>>,
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
            if index.documents.contains_key(entry.id.as_str()) {
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
        let Some(old) = self.documents.get_mut(entry.id.as_str()) else {
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
        let mut tokens = text::tokens_normalized(&normalized);
        let batch = TokenBatch::take(&mut tokens, normalized.len());
        let mut length = batch.length;
        let mut terms = batch.into_terms(self);
        for token in tokens {
            length += 1;
            if let Some(count) = terms.get_mut(token) {
                *count += 1;
            } else {
                terms.insert(self.intern_term(token), 1);
            }
        }
        let char_length = normalized.chars().count();
        self.insert_prepared(entry, normalized, terms, length, char_length)
    }
    fn intern_term(&self, term: &str) -> Arc<str> {
        self.postings
            .get_key_value(term)
            .map(|(key, _)| key.clone())
            .unwrap_or_else(|| Arc::from(term))
    }
    #[cfg(any(feature = "cache", feature = "advanced-search"))]
    fn intern_terms(&self, terms: BTreeMap<String, usize>) -> BTreeMap<Arc<str>, usize> {
        terms
            .into_iter()
            .map(|(term, count)| {
                let key = self.intern_term(&term);
                (key, count)
            })
            .collect()
    }
    #[cfg(feature = "advanced-search")]
    pub(crate) fn insert_analyzed(
        &mut self,
        entry: KnowledgeEntry,
        normalized: String,
        terms: BTreeMap<String, usize>,
    ) -> Option<KnowledgeEntry> {
        let char_length = normalized.chars().count();
        let length = terms.values().sum();
        let terms = self.intern_terms(terms);
        self.insert_prepared(entry, normalized, terms, length, char_length)
    }
    fn insert_prepared(
        &mut self,
        entry: KnowledgeEntry,
        normalized: String,
        terms: BTreeMap<Arc<str>, usize>,
        length: usize,
        char_length: usize,
    ) -> Option<KnowledgeEntry> {
        let normalized: Arc<str> = Arc::from(normalized);
        self.tfidf_norms.take();
        let previous = self.remove(&entry.id);
        let id: Arc<str> = Arc::from(entry.id.as_str());
        for (term, count) in &terms {
            shared_mut(self.postings.entry(term.clone()).or_default()).insert(id.clone(), *count);
        }
        self.vocabulary_capacity = self.vocabulary_capacity.max(self.postings.capacity());
        let group = self.normalized_ids.entry(normalized);
        let normalized = group.key().clone();
        shared_mut(group.or_default()).insert(id.clone());
        self.total_tokens += length;
        self.documents.insert(
            id,
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
        self.tfidf_norms.take();
        self.total_tokens -= document.length;
        for term in document.terms.keys() {
            if let Some(postings) = self.postings.get_mut(term) {
                shared_mut(postings).remove(id);
                if postings.is_empty() {
                    self.postings.remove(term);
                }
            }
        }
        if self.postings.is_empty() {
            self.postings = HashMap::new();
            self.vocabulary_capacity = 0;
        } else if self.vocabulary_capacity > 64
            && self.postings.len() < self.vocabulary_capacity / 4
        {
            self.postings.shrink_to_fit();
            self.vocabulary_capacity = self.postings.capacity();
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
        self.documents.get(id).map(|d| d.normalized.as_ref())
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
        let mut prepared = Vec::new();
        for (term, frequency) in query_terms {
            checkpoint()?;
            if let Some(postings) = self.postings.get(term.as_str()) {
                let idf = match options.algorithm {
                    SearchAlgorithm::Bm25 => crate::search::bm25_idf(self.len(), postings.len()),
                    SearchAlgorithm::Tfidf => tfidf_idf(self.len(), postings.len()),
                };
                prepared.push(QueryTerm {
                    text: term.as_str(),
                    frequency: *frequency as f64,
                    idf,
                    postings,
                });
            }
        }
        let mut visit = || {
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
            Ok::<_, E>(())
        };
        let mut merged = Vec::new();
        let mut wide = BTreeSet::new();
        if let [term] = prepared.as_slice() {
            merged.reserve(term.postings.len().min(32));
            for id in term.postings.keys() {
                visit()?;
                merged.push(id);
            }
        } else if let [left, right] = prepared.as_slice() {
            merge_pair_checked(left.postings, right.postings, &mut merged, &mut visit)?;
        } else if prepared.len() <= 32 {
            let mut pending = BinaryHeap::with_capacity(prepared.len());
            for term in &prepared {
                let mut rest = term.postings.keys();
                if let Some(id) = rest.next() {
                    pending.push(PostingCursor { id, rest });
                }
            }
            while let Some(mut cursor) = pending.pop() {
                visit()?;
                if merged.last().copied() != Some(cursor.id) {
                    merged.push(cursor.id);
                }
                if let Some(id) = cursor.rest.next() {
                    cursor.id = id;
                    pending.push(cursor);
                }
            }
        } else {
            for term in &prepared {
                for id in term.postings.keys() {
                    visit()?;
                    wide.insert(id);
                }
            }
        }
        let candidate_count = merged.len() + wide.len();
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
            prepared
                .iter()
                .map(|term| (term.frequency * term.idf).powi(2))
                .sum::<f64>()
                .sqrt()
        } else {
            0.0
        };
        let mut norms =
            (options.algorithm == SearchAlgorithm::Tfidf && candidate_count > 0).then(|| {
                NormQuery::new(
                    self.tfidf_norms
                        .get_or_init(|| Arc::new(TfidfNorms::default())),
                )
            });
        let mut results = TopK::new(options.limit);
        let mut admitted = 0usize;
        let mut cursors = None;
        for (position, id) in merged
            .into_iter()
            .map(|id| id.as_ref())
            .chain(wide.into_iter().map(|id| id.as_ref()))
            .enumerate()
        {
            #[cfg(not(feature = "execution"))]
            let _ = position;
            checkpoint()?;
            #[cfg(feature = "execution")]
            if position.is_multiple_of(512) {
                if let Some(control) = control {
                    if let Some(norms) = norms.as_mut() {
                        norms.release();
                    }
                    control.progress(
                        crate::execution::WorkPhase::Ranking,
                        position,
                        Some(candidate_count),
                    );
                }
            }
            let (key, document) = self
                .documents
                .get_key_value(id)
                .expect("indexed candidate ID");
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
            if admitted == 128 && candidate_count >= 1024 && (1..=4).contains(&prepared.len()) {
                cursors = Some(
                    prepared
                        .iter()
                        .map(|term| TermCursor::new(term.postings, id))
                        .collect::<Vec<_>>(),
                );
            }
            admitted += 1;
            let score = match options.algorithm {
                SearchAlgorithm::Bm25 => {
                    let norm = options.bm25.length_norm(document.length, average);
                    let mut score = 0.0;
                    matching_terms_checked(
                        &prepared,
                        document,
                        key,
                        cursors.as_deref_mut(),
                        &mut checkpoint,
                        |term, tf| {
                            score += options.bm25.term_score(tf, term.idf, norm);
                        },
                    )?;
                    score
                }
                SearchAlgorithm::Tfidf => {
                    let mut dot = 0.0;
                    matching_terms_checked(
                        &prepared,
                        document,
                        key,
                        cursors.as_deref_mut(),
                        &mut checkpoint,
                        |term, tf| {
                            dot += term.frequency * tf as f64 * term.idf * term.idf;
                        },
                    )?;
                    let norm = self.tfidf_norm_checked(
                        key,
                        document,
                        norms.as_mut().expect("TF-IDF cache"),
                        &mut checkpoint,
                    )?;
                    if norm > 0.0 && query_norm > 0.0 {
                        (dot / (norm * query_norm)).clamp(0.0, 1.0)
                    } else {
                        0.0
                    }
                }
            };
            if score > 0.0 && score.is_finite() {
                results.push(RankedDocument { document, score });
            }
        }
        drop(norms);
        checkpoint()?;
        let winners = results.finish();
        let mut result = Vec::with_capacity(winners.len());
        for winner in winners {
            checkpoint()?;
            let matched_terms = matched_terms_checked(&prepared, winner.document, &mut checkpoint)?;
            result.push(SearchResult {
                entry: &winner.document.entry,
                score: winner.score,
                matched_terms,
            });
        }
        checkpoint()?;
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

    fn tfidf_norm_checked<E>(
        &self,
        id: &Arc<str>,
        document: &IndexedEntry,
        query: &mut NormQuery<'_>,
        checkpoint: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<f64, E> {
        let key = NormKey::from_id(id);
        if let Some(value) = query.get(key) {
            return Ok(value);
        }
        self.compute_tfidf_norm_checked(key, document, query, checkpoint)
    }

    #[inline(never)]
    fn compute_tfidf_norm_checked<E>(
        &self,
        key: NormKey,
        document: &IndexedEntry,
        query: &mut NormQuery<'_>,
        checkpoint: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<f64, E> {
        let mut sum = 0.0;
        for (position, (term, tf)) in document.terms.iter().enumerate() {
            if position.is_multiple_of(128) {
                checkpoint()?;
            }
            let idf = query.weight(Arc::as_ptr(term).addr(), || {
                tfidf_idf(self.len(), self.postings[term].len())
            });
            sum += (*tf as f64 * idf).powi(2);
        }
        let norm = sum.sqrt();
        checkpoint()?;
        if let Ok(mut values) = query.cache.values.try_write() {
            values.entry(key).or_insert(norm);
        }
        query.cold_terms = query.cold_terms.saturating_add(document.terms.len());
        Ok(norm)
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
        let exact_ids = self.normalized_ids.get(normalized.as_str());
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
            let character = if normalized.as_str() == document.normalized.as_ref() {
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
                exact: normalized.as_str() == document.normalized.as_ref(),
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
                    let document = &self.documents[result.entry.id.as_str()];
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
        let source_terms: BTreeSet<&str> = source
            .terms
            .keys()
            .map(|term| term.as_ref())
            .filter(|term| !text::is_stopword(term))
            .collect();
        let candidates = self.candidates_checked(
            source_terms.iter().copied(),
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
            let target_terms: BTreeSet<&str> = self.documents[target_id]
                .terms
                .keys()
                .map(|term| term.as_ref())
                .filter(|term| !text::is_stopword(term))
                .collect();
            let shared_terms: Vec<String> = source_terms
                .intersection(&target_terms)
                .map(|term| (*term).to_owned())
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
                postcard::to_io(&(item.0.as_ref(), item.1), &mut *writer)?;
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
        let mut index = KnowledgeIndex::new();
        for entry in originals {
            let cached = self
                .documents
                .remove(&entry.id)
                .expect("validated cached source ID");
            let terms = index.intern_terms(cached.terms);
            index.insert_prepared(
                entry,
                cached.normalized,
                terms,
                cached.length,
                cached.char_length,
            );
        }
        debug_assert_eq!(index.total_tokens, self.total_tokens);
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

#[cfg(test)]
mod optimization_tests {
    use super::*;

    #[test]
    fn winner_terms_match_independent_sorted_intersection_across_query_widths() {
        let count = if cfg!(miri) { 96 } else { 512 };
        let index = KnowledgeIndex::from_entries([
            KnowledgeEntry::new("focus", "term0 term0 term5 term13 café café हिंदी 中文"),
            KnowledgeEntry::new(
                "vocabulary",
                (0..count).map(|n| format!("term{n} ")).collect::<String>() + " café हिंदी 中文",
            ),
        ])
        .unwrap();
        let widths: &[usize] = if cfg!(miri) {
            &[0, 1, 32, 33, 64, 65]
        } else {
            &[0, 1, 32, 33, 64, 65, 128, 257, 512]
        };
        let mut queries: Vec<_> = widths
            .iter()
            .map(|width| (0..*width).map(|n| format!("term{n} ")).collect::<String>() + " absent")
            .collect();
        queries.push("CAFÉ cafe\u{301} हिंदी 中文 absent".into());
        queries.push(
            (0..65).map(|n| format!("term{n} ")).collect::<String>()
                + " CAFÉ cafe\u{301} हिंदी 中文 term0 term0 absent",
        );
        let document = index.documents.get("focus").unwrap();
        for query in queries {
            let frequencies = text::term_frequencies(&query);
            let prepared: Vec<_> = frequencies
                .iter()
                .filter_map(|(word, count)| {
                    index.postings.get(word.as_str()).map(|postings| QueryTerm {
                        text: word,
                        frequency: *count as f64,
                        idf: 1.0,
                        postings,
                    })
                })
                .collect();
            let wanted: BTreeSet<_> = frequencies.keys().map(String::as_str).collect();
            let words: BTreeSet<_> = text::tokenize(&document.entry.text).into_iter().collect();
            let expected: Vec<_> = words
                .iter()
                .map(String::as_str)
                .filter(|word| wanted.contains(word))
                .map(str::to_owned)
                .collect();
            let actual = matched_terms_checked(&prepared, document, &mut || Ok::<_, ()>(()));
            assert_eq!(actual.unwrap(), expected);
            let mut scored = Vec::new();
            dictionary_terms_checked(&prepared, document, &mut || Ok::<_, ()>(()), |term, tf| {
                scored.push((term.text.to_owned(), tf));
            })
            .unwrap();
            assert_eq!(
                scored,
                expected
                    .into_iter()
                    .map(|word| {
                        let tf = document.terms[word.as_str()];
                        (word, tf)
                    })
                    .collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn winner_terms_reject_partial_explanations_at_inspection_checkpoint() {
        let count = if cfg!(miri) { 129 } else { 512 };
        let body = (0..129).map(|n| format!("term{n} ")).collect::<String>();
        let query = (0..count).map(|n| format!("term{n} ")).collect::<String>();
        let index = KnowledgeIndex::from_entries([
            KnowledgeEntry::new("focus", body),
            KnowledgeEntry::new("vocabulary", &query),
        ])
        .unwrap();
        let frequencies = text::term_frequencies(&query);
        let prepared: Vec<_> = frequencies
            .iter()
            .map(|(word, count)| QueryTerm {
                text: word,
                frequency: *count as f64,
                idf: 1.0,
                postings: &index.postings[word.as_str()],
            })
            .collect();
        let document = index.documents.get("focus").unwrap();
        let before = index.to_json().unwrap();
        let mut checkpoints = 0;
        let result = matched_terms_checked(&prepared, document, &mut || {
            checkpoints += 1;
            if checkpoints == 2 {
                Err("interrupted")
            } else {
                Ok(())
            }
        });
        assert_eq!(result, Err("interrupted"));
        assert_eq!(checkpoints, 2);
        assert_eq!(index.to_json().unwrap(), before);
        assert_eq!(
            matched_terms_checked(&prepared, document, &mut || Ok::<_, ()>(()))
                .unwrap()
                .len(),
            129
        );
    }

    #[test]
    fn paired_postings_match_independent_union_with_shared_and_equal_ids() {
        let ids: Vec<Arc<str>> = ["a\0b", "e\u{301}", "z", "é", "資料/😀"]
            .into_iter()
            .map(Arc::from)
            .collect();
        for left_mask in [0usize, 1, 7, 31] {
            for right_mask in [0usize, 2, 15, 31] {
                let left: BTreeMap<_, _> = ids
                    .iter()
                    .enumerate()
                    .filter(|(n, _)| left_mask & (1 << n) != 0)
                    .map(|(n, id)| (id.clone(), n + 1))
                    .collect();
                let right: BTreeMap<_, _> = ids
                    .iter()
                    .enumerate()
                    .filter(|(n, _)| right_mask & (1 << n) != 0)
                    .map(|(n, id)| {
                        let key = if n % 2 == 0 {
                            Arc::from(id.as_ref())
                        } else {
                            id.clone()
                        };
                        (key, n + 2)
                    })
                    .collect();
                let expected: BTreeSet<_> = left
                    .keys()
                    .chain(right.keys())
                    .map(|id| id.as_ref())
                    .collect();
                let mut candidates = Vec::new();
                let mut calls = 0;
                merge_pair_checked(&left, &right, &mut candidates, &mut || {
                    calls += 1;
                    Ok::<_, ()>(())
                })
                .unwrap();
                assert_eq!(calls, left.len() + right.len());
                assert_eq!(
                    candidates.iter().map(|id| id.as_ref()).collect::<Vec<_>>(),
                    expected.into_iter().collect::<Vec<_>>()
                );
            }
        }
    }

    #[test]
    fn paired_postings_can_interrupt_every_visit_including_overlap_and_tails() {
        for (a, b) in [
            (vec!["a", "c", "e"], vec!["a", "b", "d", "f"]),
            (vec!["a", "b"], vec![]),
            (vec![], vec!["a", "b"]),
        ] {
            let left: BTreeMap<Arc<str>, usize> =
                a.into_iter().map(|id| (Arc::from(id), 1)).collect();
            let right: BTreeMap<Arc<str>, usize> =
                b.into_iter().map(|id| (Arc::from(id), 2)).collect();
            for stop in 1..=left.len() + right.len() {
                let mut candidates = Vec::new();
                let mut calls = 0;
                let result = merge_pair_checked(&left, &right, &mut candidates, &mut || {
                    calls += 1;
                    if calls == stop {
                        Err("interrupted")
                    } else {
                        Ok(())
                    }
                });
                assert_eq!(result, Err("interrupted"));
                assert_eq!(calls, stop);
            }
        }
    }

    #[test]
    fn paired_posting_collection_rejects_partial_results_before_filtering() {
        let index = KnowledgeIndex::from_entries(
            (0..16).map(|n| KnowledgeEntry::new(format!("id{n:02}"), "common second")),
        )
        .unwrap();
        let before = index.to_json().unwrap();
        let terms = text::term_frequencies("common second absent");
        let mut calls = 0;
        let result = index.search_analyzed_checked(
            &terms,
            &SearchOptions::default(),
            |_| panic!("candidate collection must finish before filtering"),
            || {
                calls += 1;
                if calls == 10 {
                    Err("interrupted")
                } else {
                    Ok(())
                }
            },
            #[cfg(feature = "execution")]
            None,
        );
        assert!(matches!(result, Err("interrupted")));
        assert_eq!(calls, 10);
        assert_eq!(index.to_json().unwrap(), before);
        assert_eq!(index.search("common second", 16).len(), 16);
    }

    #[test]
    fn single_posting_collection_checks_every_candidate_before_filtering() {
        let index = KnowledgeIndex::from_entries(
            (0..16).map(|n| KnowledgeEntry::new(format!("id{n:02}"), "common")),
        )
        .unwrap();
        let before = index.to_json().unwrap();
        let terms = text::term_frequencies("common");
        let mut calls = 0;
        let result = index.search_analyzed_checked(
            &terms,
            &SearchOptions::default(),
            |_| panic!("candidate collection must finish before filtering"),
            || {
                calls += 1;
                if calls == 10 {
                    Err("interrupted")
                } else {
                    Ok(())
                }
            },
            #[cfg(feature = "execution")]
            None,
        );
        assert!(matches!(result, Err("interrupted")));
        assert_eq!(calls, 10);
        assert_eq!(index.to_json().unwrap(), before);
        assert_eq!(index.search("common", 16).len(), 16);
    }

    #[test]
    fn posting_frequency_cursor_keeps_exact_counts_and_equal_id_fallback() {
        let count = if cfg!(miri) { 40 } else { 128 };
        let postings: BTreeMap<Arc<str>, usize> = (0..count)
            .map(|n| (Arc::from(format!("id{n:04}")), n % 7 + 1))
            .collect();
        let mut cursor = TermCursor::new(&postings, "id0000");
        let owned: Arc<str> = Arc::from("id0000");
        assert!(!Arc::ptr_eq(postings.first_key_value().unwrap().0, &owned));
        let mut calls = 0;
        for id in [
            "id0000".to_owned(),
            "id0001a".into(),
            "id0002".into(),
            format!("id{:04}", count - 1),
            "id9999".into(),
        ] {
            let key: Arc<str> = if let Some((key, _)) = postings.get_key_value(id.as_str()) {
                if id == "id0000" {
                    owned.clone()
                } else {
                    key.clone()
                }
            } else {
                Arc::from(id.as_str())
            };
            let actual = cursor
                .frequency(&key, &mut || {
                    calls += 1;
                    Ok::<_, ()>(())
                })
                .unwrap();
            assert_eq!(actual, postings.get(id.as_str()).copied());
        }
        assert_eq!(calls, 1);
    }

    #[test]
    fn posting_frequency_cursor_checks_cancellation_before_seeking_over_a_gap() {
        let count = if cfg!(miri) { 40 } else { 128 };
        let postings: BTreeMap<Arc<str>, usize> = (0..count)
            .map(|n| (Arc::from(format!("id{n:04}")), n + 1))
            .collect();
        let mut cursor = TermCursor::new(&postings, "id0000");
        cursor
            .frequency(postings.first_key_value().unwrap().0, &mut || {
                Ok::<_, &str>(())
            })
            .unwrap();
        let mut calls = 0;
        let result = cursor.frequency(postings.last_key_value().unwrap().0, &mut || {
            calls += 1;
            Err("cancelled before seek")
        });
        assert_eq!(result, Err("cancelled before seek"));
        assert_eq!(calls, 1);
        assert_eq!(cursor.next.unwrap().0.as_ref(), "id0032");
    }
    fn signature(index: &KnowledgeIndex) -> Vec<(String, u64)> {
        index
            .search_with_options(
                "alpha beta",
                &SearchOptions {
                    algorithm: SearchAlgorithm::Tfidf,
                    limit: 10,
                    ..Default::default()
                },
            )
            .into_iter()
            .map(|h| (h.entry.id.clone(), h.score.to_bits()))
            .collect()
    }
    #[test]
    fn empty_vocabulary_releases_its_hash_allocation_after_removal() {
        let mut index = KnowledgeIndex::from_entries([KnowledgeEntry::new(
            "large",
            (0..20000)
                .map(|i| format!("word{i}"))
                .collect::<Vec<_>>()
                .join(" "),
        )])
        .unwrap();
        assert!(index.postings.capacity() >= 20000);
        index.remove("large");
        assert!(index.is_empty());
        assert_eq!(index.postings.capacity(), 0);
    }
    #[test]
    fn sparse_vocabulary_shrinks_without_changing_retained_readers() {
        let mut index = KnowledgeIndex::from_entries(
            (0..4096).map(|i| KnowledgeEntry::new(format!("doc-{i}"), format!("word{i}"))),
        )
        .unwrap();
        let retained = index.clone();
        for i in 0..4090 {
            index.remove(&format!("doc-{i}"));
        }
        assert_eq!(index.stats().terms, 6);
        assert!(index.postings.capacity() <= 64);
        assert_eq!(retained.stats().terms, 4096);
        assert_eq!(retained.search("word0", 1).len(), 1);
        assert!(index.search("word0", 1).is_empty());
        assert_eq!(index.search("word4095", 1)[0].entry.id, "doc-4095");
    }
    #[test]
    fn derived_storage_shares_normalized_words_and_identifiers() {
        let index = KnowledgeIndex::from_entries([
            KnowledgeEntry::new("a", "ALPHA beta"),
            KnowledgeEntry::new("b", "alpha BETA"),
        ])
        .unwrap();
        let a = &index.documents["a"];
        let b = &index.documents["b"];
        assert!(Arc::ptr_eq(&a.normalized, &b.normalized));
        let group = index.normalized_ids.get_key_value("alpha beta").unwrap();
        assert!(Arc::ptr_eq(group.0, &a.normalized));
        for term in ["alpha", "beta"] {
            let word = index.postings.get_key_value(term).unwrap().0;
            assert!(Arc::ptr_eq(word, a.terms.get_key_value(term).unwrap().0));
            assert!(Arc::ptr_eq(word, b.terms.get_key_value(term).unwrap().0));
            let id = index.documents.get_key_value("a").unwrap().0;
            assert!(Arc::ptr_eq(
                id,
                index.postings[term].get_key_value("a").unwrap().0
            ));
        }
    }
    #[test]
    fn bm25_and_unknown_queries_allocate_no_norm_cache() {
        let index = KnowledgeIndex::from_entries([KnowledgeEntry::new("a", "alpha beta")]).unwrap();
        index.search("alpha", 10);
        index.search_with_options(
            "absent",
            &SearchOptions {
                algorithm: SearchAlgorithm::Tfidf,
                ..Default::default()
            },
        );
        assert!(index.tfidf_norms.get().is_none());
    }
    #[test]
    fn metadata_only_changes_reuse_norms_and_invalid_updates_leave_them_valid() {
        let mut index = KnowledgeIndex::from_entries([
            KnowledgeEntry::new("a", "alpha beta"),
            KnowledgeEntry::new("b", "beta"),
        ])
        .unwrap();
        let before = signature(&index);
        let cache = index.tfidf_norms.get().unwrap().clone();
        index
            .upsert(
                KnowledgeEntry::new("a", "alpha beta")
                    .with_metadata(serde_json::json!({"new":true})),
            )
            .unwrap();
        assert!(Arc::ptr_eq(&cache, index.tfidf_norms.get().unwrap()));
        assert!(index.upsert(KnowledgeEntry::new("a", " ")).is_err());
        assert!(Arc::ptr_eq(&cache, index.tfidf_norms.get().unwrap()));
        assert_eq!(before, signature(&index));
    }
    #[test]
    fn norm_hints_do_not_retain_identifier_allocations() {
        let index = KnowledgeIndex::from_entries([
            KnowledgeEntry::new("資料/é\0a", "alpha beta"),
            KnowledgeEntry::new("資料/e\u{301}\0b", "beta"),
        ])
        .unwrap();
        let counts: Vec<_> = index.documents.keys().map(Arc::strong_count).collect();
        assert_eq!(signature(&index).len(), 2);
        assert_eq!(
            index
                .documents
                .keys()
                .map(Arc::strong_count)
                .collect::<Vec<_>>(),
            counts
        );
    }
    #[test]
    fn warm_norms_survive_retained_clones_and_reused_ids_without_stale_values() {
        let mut index = KnowledgeIndex::from_entries([
            KnowledgeEntry::new("a", "alpha beta beta"),
            KnowledgeEntry::new("b", "beta"),
            KnowledgeEntry::new("c", "alpha gamma"),
        ])
        .unwrap();
        let before = signature(&index);
        let retained = index.clone();
        let cache = retained.tfidf_norms.get().unwrap().clone();
        index
            .upsert(
                KnowledgeEntry::new("a", "alpha beta beta")
                    .with_metadata(serde_json::json!({"tag":1})),
            )
            .unwrap();
        assert!(Arc::ptr_eq(&cache, index.tfidf_norms.get().unwrap()));
        assert_eq!(signature(&index), before);
        let rounds = if cfg!(miri) { 3 } else { 64 };
        for round in 0..rounds {
            index.remove("a");
            index
                .upsert(KnowledgeEntry::new(
                    "a",
                    format!("alpha {}", "beta ".repeat(round + 1)),
                ))
                .unwrap();
            index
                .upsert(KnowledgeEntry::new("b", format!("beta unique{round}")))
                .unwrap();
            let rebuilt = KnowledgeIndex::from_entries(index.entries().cloned()).unwrap();
            assert_eq!(signature(&index), signature(&rebuilt));
            assert_eq!(signature(&retained), before);
        }
        index.clear();
        index
            .upsert(KnowledgeEntry::new("a", "beta gamma"))
            .unwrap();
        let rebuilt = KnowledgeIndex::from_entries(index.entries().cloned()).unwrap();
        assert_eq!(signature(&index), signature(&rebuilt));
        assert_eq!(signature(&retained), before);
        let restored = KnowledgeIndex::from_json(&retained.to_json().unwrap()).unwrap();
        drop(index);
        drop(retained);
        assert_eq!(signature(&restored), before);
    }
    #[test]
    fn concurrent_cold_norm_initialization_keeps_exact_scores() {
        let index = KnowledgeIndex::from_entries((0..128).map(|n| {
            KnowledgeEntry::new(
                format!("id-{n}"),
                format!("alpha beta {}", "gamma ".repeat(n % 11)),
            )
        }))
        .unwrap();
        let rebuilt = KnowledgeIndex::from_entries(index.entries().cloned()).unwrap();
        let expected = signature(&rebuilt);
        assert!(index.tfidf_norms.get().is_none());
        std::thread::scope(|scope| {
            let handles: Vec<_> = (0..4).map(|_| scope.spawn(|| signature(&index))).collect();
            for handle in handles {
                assert_eq!(handle.join().unwrap(), expected);
            }
        });
        assert_eq!(signature(&index), expected);
    }
    #[test]
    fn query_weight_collisions_recompute_instead_of_reusing_another_word() {
        fn check<const N: usize>() {
            let mut memo = QueryWeights::<N>::default();
            let mut computes = 0;
            let first = 16;
            let collision = first + (1 << 20);
            for (key, expected, calls) in [
                (first, 3.5, 1),
                (first, 3.5, 1),
                (collision, 7.25, 2),
                (collision, 7.25, 2),
                (first, 3.5, 3),
            ] {
                let actual = memo.get(key, || {
                    computes += 1;
                    expected
                });
                assert_eq!(actual.to_bits(), expected.to_bits());
                assert_eq!(computes, calls);
            }
        }
        check::<8>();
        check::<256>();
        let cache = TfidfNorms::default();
        let mut query = NormQuery::new(&cache);
        assert_eq!(query.weight(16, || 3.5), 3.5);
        query.cold_terms = 128;
        assert_eq!(
            query.weight(16, || panic!("promotion lost the exact weight")),
            3.5
        );
        assert!(query.small.is_none());
        assert!(query.weights.is_some());
        assert_eq!(query.weight(16 + (1 << 20), || 7.25), 7.25);
        assert_eq!(query.weight(16, || 3.5), 3.5);
    }

    #[test]
    fn warm_and_interrupted_norms_do_not_allocate_query_weights() {
        let index = KnowledgeIndex::from_entries([
            KnowledgeEntry::new("a", "alpha beta"),
            KnowledgeEntry::new("b", "beta gamma"),
        ])
        .unwrap();
        let (id, document) = index.documents.first_key_value().unwrap();
        let cache = TfidfNorms::default();
        let mut query = NormQuery::new(&cache);
        let interrupted =
            index.tfidf_norm_checked(id, document, &mut query, &mut || Err("cancelled"));
        assert_eq!(interrupted, Err("cancelled"));
        assert!(query.weights.is_none());
        assert_eq!(query.cold_terms, 0);
        assert!(query.small.is_none());
        assert!(cache.values.read().unwrap().is_empty());
        let cold = index
            .tfidf_norm_checked(id, document, &mut query, &mut || Ok::<_, ()>(()))
            .unwrap();
        assert!(query.weights.is_none());
        let mut fresh = NormQuery::new(&cache);
        let warm = index
            .tfidf_norm_checked(id, document, &mut fresh, &mut || Ok::<_, ()>(()))
            .unwrap();
        assert_eq!(cold.to_bits(), warm.to_bits());
        assert!(fresh.weights.is_none());
        assert_eq!(fresh.cold_terms, 0);
        assert!(fresh.small.is_none());
    }
    #[test]
    fn batched_norm_reader_releases_before_misses_at_boundaries_and_on_drop() {
        let index = KnowledgeIndex::from_entries([
            KnowledgeEntry::new("a", "alpha beta"),
            KnowledgeEntry::new("b", "beta gamma gamma"),
        ])
        .unwrap();
        let cache = TfidfNorms::default();
        let mut query = NormQuery::new(&cache);
        let (a, document) = index.documents.get_key_value("a").unwrap();
        let first = index
            .tfidf_norm_checked(a, document, &mut query, &mut || Ok::<_, ()>(()))
            .unwrap();
        for _ in 0..128 {
            assert_eq!(
                query.get(NormKey::from_id(a)).unwrap().to_bits(),
                first.to_bits()
            );
        }
        assert!(cache.values.try_write().is_ok());
        assert!(query.get(NormKey::from_id(a)).is_some());
        assert!(cache.values.try_write().is_err());
        let (b, document) = index.documents.get_key_value("b").unwrap();
        index
            .tfidf_norm_checked(b, document, &mut query, &mut || Ok::<_, ()>(()))
            .unwrap();
        assert_eq!(cache.values.try_read().unwrap().len(), 2);
        assert!(cache.values.try_write().is_ok());
        assert!(query.get(NormKey::from_id(b)).is_some());
        drop(query);
        assert!(cache.values.try_write().is_ok());
    }
    #[test]
    fn query_weight_scratch_waits_for_completed_cold_work() {
        let index = KnowledgeIndex::from_entries([
            KnowledgeEntry::new(
                "seed",
                (0..128).map(|n| format!("word{n} ")).collect::<String>(),
            ),
            KnowledgeEntry::new("next", "alpha beta"),
            KnowledgeEntry::new("last", "beta gamma"),
        ])
        .unwrap();
        let cache = TfidfNorms::default();
        let mut query = NormQuery::new(&cache);
        let (id, document) = index.documents.get_key_value("seed").unwrap();
        assert_eq!(
            index.tfidf_norm_checked(id, document, &mut query, &mut || Err("cancelled")),
            Err("cancelled")
        );
        assert_eq!(query.cold_terms, 0);
        assert!(query.weights.is_none());
        for name in ["seed", "next", "last"] {
            let (id, document) = index.documents.get_key_value(name).unwrap();
            let actual = index
                .tfidf_norm_checked(id, document, &mut query, &mut || Ok::<_, ()>(()))
                .unwrap();
            let expected = document
                .terms
                .iter()
                .map(|(term, tf)| {
                    (*tf as f64 * tfidf_idf(index.len(), index.postings[term].len())).powi(2)
                })
                .sum::<f64>()
                .sqrt();
            assert_eq!(actual.to_bits(), expected.to_bits());
            assert_eq!(query.weights.is_some(), name != "seed");
        }
    }
    #[test]
    fn warm_norm_reader_does_not_block_cold_norms_and_hints_recover() {
        let index = KnowledgeIndex::from_entries([
            KnowledgeEntry::new("a", "alpha beta"),
            KnowledgeEntry::new("b", "gamma gamma"),
        ])
        .unwrap();
        let cache = TfidfNorms::default();
        let mut warm = NormQuery::new(&cache);
        let (a, document) = index.documents.get_key_value("a").unwrap();
        index
            .tfidf_norm_checked(a, document, &mut warm, &mut || Ok::<_, ()>(()))
            .unwrap();
        assert!(warm.get(NormKey::from_id(a)).is_some());
        let (b, document) = index.documents.get_key_value("b").unwrap();
        let expected = (2.0 * tfidf_idf(index.len(), 1)).powi(2).sqrt();
        std::thread::scope(|scope| {
            let (send, receive) = std::sync::mpsc::channel();
            let cache = &cache;
            let index = &index;
            scope.spawn(move || {
                let value =
                    index.tfidf_norm_checked(b, document, &mut NormQuery::new(cache), &mut || {
                        Ok::<_, ()>(())
                    });
                let _ = send.send(value);
            });
            let result = receive.recv_timeout(std::time::Duration::from_secs(2));
            drop(warm);
            assert_eq!(result.unwrap().unwrap().to_bits(), expected.to_bits());
        });
        assert_eq!(cache.values.read().unwrap().len(), 1);
        let mut recovered = NormQuery::new(&cache);
        let actual = index
            .tfidf_norm_checked(b, document, &mut recovered, &mut || Ok::<_, ()>(()))
            .unwrap();
        assert_eq!(actual.to_bits(), expected.to_bits());
        assert_eq!(cache.values.read().unwrap().len(), 2);
    }
    #[test]
    fn poisoned_norm_hint_falls_back_to_exact_computation() {
        let index = KnowledgeIndex::from_entries([
            KnowledgeEntry::new("a", "alpha beta"),
            KnowledgeEntry::new("b", "beta"),
        ])
        .unwrap();
        let before = signature(&index);
        let cache = index.tfidf_norms.get().unwrap().clone();
        assert!(std::panic::catch_unwind(move || {
            let _guard = cache.values.write().unwrap();
            panic!("injected cache poison");
        })
        .is_err());
        assert_eq!(before, signature(&index));
    }
    #[test]
    fn busy_norm_hint_never_blocks_a_query() {
        let index = KnowledgeIndex::from_entries([
            KnowledgeEntry::new("a", "alpha beta"),
            KnowledgeEntry::new("b", "beta"),
        ])
        .unwrap();
        let before = signature(&index);
        let cache = index.tfidf_norms.get().unwrap().clone();
        std::thread::scope(|scope| {
            let guard = cache.values.write().unwrap();
            let (send, receive) = std::sync::mpsc::channel();
            let index = &index;
            scope.spawn(move || {
                let _ = send.send(signature(index));
            });
            let result = receive.recv_timeout(std::time::Duration::from_secs(2));
            drop(guard);
            assert_eq!(result.unwrap(), before);
        });
    }
    #[test]
    fn interrupted_norm_computation_never_publishes_a_partial_value() {
        let text = (0..1000)
            .map(|i| format!("word{i}"))
            .collect::<Vec<_>>()
            .join(" ");
        let index = KnowledgeIndex::from_entries([KnowledgeEntry::new("a", text)]).unwrap();
        let (id, document) = index.documents.first_key_value().unwrap();
        let cache = TfidfNorms::default();
        let mut checks = 0;
        let result =
            index.tfidf_norm_checked(id, document, &mut NormQuery::new(&cache), &mut || {
                checks += 1;
                if checks == 2 {
                    Err("stopped")
                } else {
                    Ok(())
                }
            });
        assert_eq!(result, Err("stopped"));
        assert!(cache.values.read().unwrap().is_empty());
        let expected = document
            .terms
            .iter()
            .map(|(term, tf)| {
                (*tf as f64 * tfidf_idf(index.len(), index.postings[term].len())).powi(2)
            })
            .sum::<f64>()
            .sqrt();
        assert_eq!(
            index
                .tfidf_norm_checked(
                    id,
                    document,
                    &mut NormQuery::new(&cache),
                    &mut || Ok::<_, ()>(())
                )
                .unwrap()
                .to_bits(),
            expected.to_bits()
        );
    }
}

#[cfg(test)]
mod token_batch_tests {
    use super::*;

    fn reference(words: &[&str]) -> BTreeMap<String, usize> {
        let mut counts = BTreeMap::new();
        for word in words {
            *counts.entry((*word).to_owned()).or_default() += 1;
        }
        counts
    }

    #[test]
    fn aggregated_prefix_preserves_total_count_and_sorted_unicode_frequencies() {
        for words in [
            vec![],
            vec!["東京"],
            vec!["café", "café"],
            vec!["東京", "β", "café", "β", "東京", "a", "β", "café"],
        ] {
            let mut tokens = words.iter().copied();
            let batch = TokenBatch::take(&mut tokens, words.iter().map(|w| w.len()).sum());
            assert_eq!(batch.length, words.len());
            assert_eq!(
                batch.counts.iter().map(|(_, n)| n).sum::<usize>(),
                words.len()
            );
            let expected = reference(&words);
            assert_eq!(
                batch.counts,
                expected
                    .iter()
                    .map(|(w, n)| (w.as_str(), *n))
                    .collect::<Vec<_>>()
            );
            assert_eq!(tokens.next(), None);
        }
    }

    #[test]
    fn prefix_stops_at_limit_and_leaves_the_entire_streaming_tail() {
        let words = ["β", "a", "東京", "a", "café"];
        for count in [1023, 1024, 1025, 2051] {
            let input: Vec<_> = words.into_iter().cycle().take(count).collect();
            let mut tokens = input.iter().copied();
            let batch = TokenBatch::take(&mut tokens, usize::MAX);
            let prefix = count.min(TokenBatch::LIMIT);
            assert_eq!(batch.length, prefix);
            assert!(batch.counts.capacity() <= TokenBatch::LIMIT);
            let expected = reference(&input[..prefix]);
            assert_eq!(
                batch.counts,
                expected
                    .iter()
                    .map(|(w, n)| (w.as_str(), *n))
                    .collect::<Vec<_>>()
            );
            assert_eq!(tokens.collect::<Vec<_>>(), input[prefix..]);
        }
    }

    #[test]
    fn empty_and_single_tokens_do_not_reserve_for_the_entire_text() {
        let mut empty = std::iter::empty();
        let batch = TokenBatch::take(&mut empty, usize::MAX);
        assert_eq!(batch.length, 0);
        assert_eq!(batch.counts.capacity(), 0);
        let text = "資料".repeat(32768);
        let mut tokens = std::iter::once(text.as_str());
        let batch = TokenBatch::take(&mut tokens, text.len());
        assert_eq!(batch.counts, [(text.as_str(), 1)]);
        assert_eq!(batch.counts.capacity(), 1);
    }

    #[test]
    fn aggregated_terms_reuse_vocabulary_without_mutating_retained_readers() {
        let index =
            KnowledgeIndex::from_entries([KnowledgeEntry::new("old", "café 東京")]).unwrap();
        let retained = index.clone();
        let input = ["東京", "café", "東京", "new"];
        let batch = TokenBatch::take(&mut input.into_iter(), 64);
        let terms = batch.into_terms(&index);
        for word in ["café", "東京"] {
            assert!(Arc::ptr_eq(
                terms.get_key_value(word).unwrap().0,
                index.postings.get_key_value(word).unwrap().0
            ));
        }
        assert_eq!(terms["東京"], 2);
        assert_eq!(terms["new"], 1);
        drop(index);
        assert_eq!(retained.search("東京", 1)[0].entry.id, "old");
        assert_eq!(terms["café"], 1);
    }
}

#[cfg(test)]
mod wide_scoring_tests {
    use super::*;

    #[test]
    fn sparse_wide_scoring_stops_on_checkpoint_without_publishing_results() {
        let index = KnowledgeIndex::from_entries([
            KnowledgeEntry::new(
                "dense",
                (0..400).map(|n| format!("term{n} ")).collect::<String>(),
            ),
            KnowledgeEntry::new(
                "sparse",
                (0..150).map(|n| format!("term{n} ")).collect::<String>(),
            ),
        ])
        .unwrap();
        let terms =
            text::term_frequencies(&(0..400).map(|n| format!("term{n} ")).collect::<String>());
        let prepared: Vec<_> = terms
            .iter()
            .map(|(word, n)| QueryTerm {
                text: word,
                frequency: *n as f64,
                idf: 1.0,
                postings: &index.postings[word.as_str()],
            })
            .collect();
        let mut checks = 0;
        let mut matched = 0;
        let result = matching_terms_checked(
            &prepared,
            &index.documents["sparse"],
            index.documents.get_key_value("sparse").unwrap().0,
            None,
            &mut || {
                checks += 1;
                if checks == 2 {
                    Err("stopped")
                } else {
                    Ok(())
                }
            },
            |_, _| matched += 1,
        );
        assert_eq!(result, Err("stopped"));
        assert!(matched > 0 && matched < 150);
        assert_eq!(index.stats().entries, 2);
    }
}
