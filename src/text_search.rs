#![doc = include_str!("../docs/TEXT-SEARCH.md")]

use crate::{
    index::validate_entry, text, Context, Engine, IndexError, IndexStats, KnowledgeEntry,
    SearchOptions,
};
use rust_stemmers::{Algorithm, Stemmer};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
use unicode_normalization::char::is_combining_mark;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum TextSearchError {
    #[cfg(feature = "execution")]
    #[error(transparent)]
    Execution(#[from] crate::execution::ExecutionError),
    #[error("invalid text search configuration: {0}")]
    Configuration(&'static str),
    #[error("text search limit exceeded: {0}")]
    Limit(&'static str),
    #[error("invalid text query: {0}")]
    Query(&'static str),
    #[error(transparent)]
    Index(#[from] IndexError),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub enum WordTokenizer {
    #[default]
    UnicodeWords,
    Alphanumeric,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub enum Stemming {
    #[default]
    None,
    German,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyzerConfig {
    tokenizer: WordTokenizer,
    stemming: Stemming,
    fold_apostrophes: bool,
    max_input_bytes: usize,
    max_normalized_bytes: usize,
    max_tokens: usize,
}
impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            tokenizer: WordTokenizer::UnicodeWords,
            stemming: Stemming::None,
            fold_apostrophes: true,
            max_input_bytes: 1024 * 1024,
            max_normalized_bytes: 4 * 1024 * 1024,
            max_tokens: 100_000,
        }
    }
}
impl AnalyzerConfig {
    pub fn german() -> Self {
        Self {
            stemming: Stemming::German,
            ..Self::default()
        }
    }
    pub fn with_tokenizer(mut self, tokenizer: WordTokenizer) -> Self {
        self.tokenizer = tokenizer;
        self
    }
    pub fn with_stemming(mut self, stemming: Stemming) -> Self {
        self.stemming = stemming;
        self
    }
    pub fn with_apostrophe_folding(mut self, enabled: bool) -> Self {
        self.fold_apostrophes = enabled;
        self
    }
    pub fn with_limits(
        mut self,
        input_bytes: usize,
        normalized_bytes: usize,
        tokens: usize,
    ) -> Result<Self, TextSearchError> {
        if input_bytes == 0 || normalized_bytes == 0 || tokens == 0 {
            return Err(TextSearchError::Configuration(
                "analysis limits must be positive",
            ));
        }
        self.max_input_bytes = input_bytes;
        self.max_normalized_bytes = normalized_bytes;
        self.max_tokens = tokens;
        Ok(self)
    }
    pub fn tokenizer(&self) -> WordTokenizer {
        self.tokenizer
    }
    pub fn stemming(&self) -> Stemming {
        self.stemming
    }
    pub fn folds_apostrophes(&self) -> bool {
        self.fold_apostrophes
    }
    pub fn max_input_bytes(&self) -> usize {
        self.max_input_bytes
    }
    pub fn max_normalized_bytes(&self) -> usize {
        self.max_normalized_bytes
    }
    pub fn max_tokens(&self) -> usize {
        self.max_tokens
    }
    pub fn recipe(&self) -> String {
        format!("airust-text-v1/nfkc-lower-nfkc/graphemes/unicode-{:?}-{:?}-{:?}/words-1.13.3/{:?}/stem-1.2.0-{:?}/apostrophes-{}",char::UNICODE_VERSION,unicode_normalization::UNICODE_VERSION,unicode_segmentation::UNICODE_VERSION,self.tokenizer,self.stemming,self.fold_apostrophes)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TextSpan {
    pub start: usize,
    pub end: usize,
}
impl TextSpan {
    pub fn slice<'a>(&self, text: &'a str) -> Option<&'a str> {
        text.get(self.start..self.end)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Token {
    pub term: String,
    pub position: usize,
    pub span: TextSpan,
}

pub struct TextAnalyzer {
    config: AnalyzerConfig,
    stemmer: Option<Stemmer>,
}
impl TextAnalyzer {
    pub fn new(config: AnalyzerConfig) -> Self {
        let stemmer =
            (config.stemming == Stemming::German).then(|| Stemmer::create(Algorithm::German));
        Self { config, stemmer }
    }
    pub fn config(&self) -> &AnalyzerConfig {
        &self.config
    }
    pub fn analyze(&self, source: &str) -> Result<Vec<Token>, TextSearchError> {
        if source.len() > self.config.max_input_bytes {
            return Err(TextSearchError::Limit("input bytes"));
        }
        let mut normalized = String::new();
        let mut mapping = Vec::new();
        for (start, grapheme) in source.grapheme_indices(true) {
            let begin = normalized.len();
            for mut c in text::normalized_characters(grapheme) {
                if self.config.fold_apostrophes && matches!(c, '\u{2018}' | '\u{2019}' | '\u{02bc}')
                {
                    c = '\'';
                }
                if c.len_utf8()
                    > self
                        .config
                        .max_normalized_bytes
                        .saturating_sub(normalized.len())
                {
                    return Err(TextSearchError::Limit("normalized bytes"));
                }
                normalized.push(c);
            }
            if normalized.len() > begin {
                mapping.push((begin, normalized.len(), start, start + grapheme.len()));
            }
        }
        let words: Vec<(usize, &str)> = match self.config.tokenizer {
            WordTokenizer::UnicodeWords => normalized
                .unicode_word_indices()
                .take(self.config.max_tokens.saturating_add(1))
                .collect(),
            WordTokenizer::Alphanumeric => {
                let mut words = Vec::new();
                let mut start = None;
                for (offset, c) in normalized
                    .char_indices()
                    .chain(std::iter::once((normalized.len(), ' ')))
                {
                    if c.is_alphanumeric() || is_combining_mark(c) {
                        if start.is_none() {
                            start = Some(offset)
                        }
                    } else if let Some(begin) = start.take() {
                        let word = &normalized[begin..offset];
                        if word.chars().any(char::is_alphanumeric) {
                            if words.len() == self.config.max_tokens {
                                return Err(TextSearchError::Limit("token count"));
                            }
                            words.push((begin, word));
                        }
                    }
                }
                words
            }
        };
        if words.len() > self.config.max_tokens {
            return Err(TextSearchError::Limit("token count"));
        }
        let mut tokens = Vec::with_capacity(words.len());
        for (offset, word) in words {
            let first = mapping.partition_point(|m| m.1 <= offset);
            let last = mapping.partition_point(|m| m.0 < offset + word.len()) - 1;
            let mut term = text::normalize(word);
            if let Some(stemmer) = &self.stemmer {
                let stem = stemmer.stem(&term);
                if !stem.is_empty() {
                    term = stem.into_owned();
                }
            }
            tokens.push(Token {
                term,
                position: tokens.len(),
                span: TextSpan {
                    start: mapping[first].2,
                    end: mapping[last].3,
                },
            });
        }
        Ok(tokens)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum TermOperator {
    #[default]
    Any,
    All,
}
#[derive(Debug, Clone)]
pub struct TextSearchOptions {
    search: SearchOptions,
    operator: TermOperator,
    max_query_bytes: usize,
    max_query_tokens: usize,
    max_phrases: usize,
    max_phrase_tokens: usize,
    max_spans: usize,
}
impl Default for TextSearchOptions {
    fn default() -> Self {
        Self {
            search: SearchOptions::default(),
            operator: TermOperator::Any,
            max_query_bytes: 16 * 1024,
            max_query_tokens: 256,
            max_phrases: 16,
            max_phrase_tokens: 64,
            max_spans: 64,
        }
    }
}
impl TextSearchOptions {
    pub fn new(limit: usize) -> Self {
        Self {
            search: SearchOptions {
                limit,
                ..SearchOptions::default()
            },
            ..Self::default()
        }
    }
    pub fn with_search(mut self, search: SearchOptions) -> Self {
        self.search = search;
        self
    }
    pub fn with_context(mut self, context: Context) -> Self {
        self.search.context = Some(context);
        self
    }
    pub fn with_operator(mut self, operator: TermOperator) -> Self {
        self.operator = operator;
        self
    }
    pub fn with_query_limits(
        mut self,
        bytes: usize,
        tokens: usize,
        phrases: usize,
        phrase_tokens: usize,
    ) -> Result<Self, TextSearchError> {
        if [bytes, tokens, phrases, phrase_tokens].contains(&0) {
            return Err(TextSearchError::Configuration(
                "query limits must be positive",
            ));
        }
        self.max_query_bytes = bytes;
        self.max_query_tokens = tokens;
        self.max_phrases = phrases;
        self.max_phrase_tokens = phrase_tokens;
        Ok(self)
    }
    pub fn with_max_spans(mut self, max: usize) -> Result<Self, TextSearchError> {
        if max == 0 {
            return Err(TextSearchError::Configuration(
                "span limit must be positive",
            ));
        }
        self.max_spans = max;
        Ok(self)
    }
    pub fn search_options(&self) -> &SearchOptions {
        &self.search
    }
    pub fn operator(&self) -> TermOperator {
        self.operator
    }
    pub fn max_query_bytes(&self) -> usize {
        self.max_query_bytes
    }
    pub fn max_query_tokens(&self) -> usize {
        self.max_query_tokens
    }
    pub fn max_phrases(&self) -> usize {
        self.max_phrases
    }
    pub fn max_phrase_tokens(&self) -> usize {
        self.max_phrase_tokens
    }
    pub fn max_spans(&self) -> usize {
        self.max_spans
    }
}

struct Phrase {
    terms: Vec<String>,
    prefix: Vec<usize>,
}
impl Phrase {
    fn new(terms: Vec<String>) -> Self {
        let mut prefix = vec![0; terms.len()];
        let mut j = 0;
        for i in 1..terms.len() {
            while j > 0 && terms[i] != terms[j] {
                j = prefix[j - 1];
            }
            if terms[i] == terms[j] {
                j += 1;
            }
            prefix[i] = j;
        }
        Self { terms, prefix }
    }
    fn matches(&self, tokens: &[Token], max: usize) -> (Vec<TextSpan>, bool) {
        let mut j = 0;
        let mut spans = Vec::new();
        for (i, token) in tokens.iter().enumerate() {
            while j > 0 && token.term != self.terms[j] {
                j = self.prefix[j - 1];
            }
            if token.term == self.terms[j] {
                j += 1;
            }
            if j == self.terms.len() {
                if spans.len() == max {
                    return (spans, true);
                }
                spans.push(TextSpan {
                    start: tokens[i + 1 - j].span.start,
                    end: token.span.end,
                });
                j = self.prefix[j - 1];
            }
        }
        (spans, false)
    }
}
struct Plan {
    terms: BTreeMap<String, usize>,
    unquoted: BTreeSet<String>,
    phrases: Vec<Phrase>,
}
impl Plan {
    fn new(
        query: &str,
        analyzer: &TextAnalyzer,
        options: &TextSearchOptions,
    ) -> Result<Self, TextSearchError> {
        if query.trim().is_empty() {
            return Err(TextSearchError::Query("empty query"));
        }
        if query.len() > options.max_query_bytes {
            return Err(TextSearchError::Limit("query bytes"));
        }
        let mut parts = Vec::new();
        let mut text = String::new();
        let mut quoted = false;
        let mut chars = query.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '\\' if chars.peek().is_some_and(|c| matches!(c, '"' | '\\')) => {
                    text.push(chars.next().unwrap())
                }
                '"' => {
                    if quoted && text.is_empty() {
                        return Err(TextSearchError::Query("empty phrase"));
                    }
                    if !text.is_empty() {
                        parts.push((quoted, std::mem::take(&mut text)));
                    }
                    quoted = !quoted;
                }
                _ => text.push(c),
            }
        }
        if quoted {
            return Err(TextSearchError::Query("unclosed quoted phrase"));
        }
        if !text.is_empty() {
            parts.push((false, text));
        }
        let mut plan = Self {
            terms: BTreeMap::new(),
            unquoted: BTreeSet::new(),
            phrases: Vec::new(),
        };
        let mut count = 0;
        for (quoted, text) in parts {
            let tokens = analyzer.analyze(&text)?;
            count += tokens.len();
            if count > options.max_query_tokens {
                return Err(TextSearchError::Limit("query tokens"));
            }
            if quoted {
                if tokens.is_empty() {
                    return Err(TextSearchError::Query("phrase has no searchable tokens"));
                }
                if tokens.len() > options.max_phrase_tokens {
                    return Err(TextSearchError::Limit("phrase tokens"));
                }
                if plan.phrases.len() >= options.max_phrases {
                    return Err(TextSearchError::Limit("phrase count"));
                }
            }
            let terms: Vec<_> = tokens.into_iter().map(|t| t.term).collect();
            for term in &terms {
                *plan.terms.entry(term.clone()).or_default() += 1;
                if !quoted {
                    plan.unquoted.insert(term.clone());
                }
            }
            if quoted {
                plan.phrases.push(Phrase::new(terms));
            }
        }
        Ok(plan)
    }
    fn includes(&self, tokens: &[Token], operator: TermOperator) -> bool {
        let unquoted = if self.unquoted.is_empty() {
            true
        } else {
            match operator {
                TermOperator::Any => tokens.iter().any(|t| self.unquoted.contains(&t.term)),
                TermOperator::All => {
                    let terms: BTreeSet<_> = tokens.iter().map(|t| t.term.as_str()).collect();
                    self.unquoted.iter().all(|t| terms.contains(t.as_str()))
                }
            }
        };
        unquoted && self.phrases.iter().all(|p| p.matches(tokens, 0).1)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PhraseEvidence {
    pub terms: Vec<String>,
    pub spans: Vec<TextSpan>,
    pub truncated: bool,
}
#[derive(Debug, Clone, Serialize)]
pub struct TextSearchResult<'a> {
    pub entry: &'a KnowledgeEntry,
    pub score: f64,
    pub matched_terms: Vec<String>,
    pub phrases: Vec<PhraseEvidence>,
}
#[cfg(feature = "reranking")]
impl crate::reranking::Rerankable for TextSearchResult<'_> {
    fn entry(&self) -> &KnowledgeEntry {
        self.entry
    }
}

pub struct TextIndex {
    index: Engine,
    analyzer: TextAnalyzer,
    tokens: BTreeMap<String, Vec<Token>>,
}
impl Default for TextIndex {
    fn default() -> Self {
        Self::new(AnalyzerConfig::default())
    }
}
impl TextIndex {
    pub fn new(config: AnalyzerConfig) -> Self {
        Self {
            index: Engine::new(),
            analyzer: TextAnalyzer::new(config),
            tokens: BTreeMap::new(),
        }
    }
    pub fn from_entries(
        config: AnalyzerConfig,
        entries: impl IntoIterator<Item = KnowledgeEntry>,
    ) -> Result<Self, TextSearchError> {
        let mut index = Self::new(config);
        for entry in entries {
            if index.get(&entry.id).is_some() {
                return Err(IndexError::DuplicateId(entry.id).into());
            }
            index.upsert(entry)?;
        }
        Ok(index)
    }
    #[cfg(feature = "execution")]
    pub fn from_entries_controlled(
        config: AnalyzerConfig,
        entries: impl IntoIterator<Item = KnowledgeEntry>,
        control: &crate::execution::WorkControl,
    ) -> Result<Self, TextSearchError> {
        control.checkpoint()?;
        let mut index = Self::new(config);
        control.progress(crate::execution::WorkPhase::AnalyzingText, 0, None);
        for entry in entries {
            control.checkpoint()?;
            if index.get(&entry.id).is_some() {
                return Err(IndexError::DuplicateId(entry.id).into());
            }
            index.upsert(entry)?;
            if index.len().is_multiple_of(512) {
                control.progress(
                    crate::execution::WorkPhase::AnalyzingText,
                    index.len(),
                    None,
                );
            }
        }
        control.progress(
            crate::execution::WorkPhase::AnalyzingText,
            index.len(),
            Some(index.len()),
        );
        control.checkpoint()?;
        Ok(index)
    }
    pub fn analyzer(&self) -> &TextAnalyzer {
        &self.analyzer
    }
    pub fn len(&self) -> usize {
        self.index.len()
    }
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }
    pub fn get(&self, id: &str) -> Option<&KnowledgeEntry> {
        self.index.get(id)
    }
    pub fn entries(&self) -> impl Iterator<Item = &KnowledgeEntry> {
        self.index.entries()
    }
    pub fn stats(&self) -> IndexStats {
        self.index.stats()
    }
    pub fn tokens(&self, id: &str) -> Option<&[Token]> {
        self.tokens.get(id).map(Vec::as_slice)
    }
    pub fn upsert(
        &mut self,
        entry: KnowledgeEntry,
    ) -> Result<Option<KnowledgeEntry>, TextSearchError> {
        if entry.text.len() > self.analyzer.config.max_input_bytes {
            return Err(TextSearchError::Limit("input bytes"));
        }
        let normalized = validate_entry(&entry)?;
        let tokens = self.analyzer.analyze(&entry.text)?;
        let mut terms = BTreeMap::new();
        for token in &tokens {
            *terms.entry(token.term.clone()).or_default() += 1;
        }
        let id = entry.id.clone();
        let previous = self.index.insert_analyzed(entry, normalized, terms);
        self.tokens.insert(id, tokens);
        Ok(previous)
    }
    pub fn remove(&mut self, id: &str) -> Option<KnowledgeEntry> {
        let previous = self.index.remove(id);
        self.tokens.remove(id);
        previous
    }
    pub fn clear(&mut self) {
        self.index.clear();
        self.tokens.clear();
    }
    pub fn search(
        &self,
        query: &str,
        options: &TextSearchOptions,
    ) -> Result<Vec<TextSearchResult<'_>>, TextSearchError> {
        self.search_inner(
            query,
            options,
            #[cfg(feature = "execution")]
            None,
        )
    }
    #[cfg(feature = "execution")]
    pub fn search_controlled(
        &self,
        query: &str,
        options: &TextSearchOptions,
        control: &crate::execution::WorkControl,
    ) -> Result<Vec<TextSearchResult<'_>>, TextSearchError> {
        self.search_inner(query, options, Some(control))
    }
    fn search_inner(
        &self,
        query: &str,
        options: &TextSearchOptions,
        #[cfg(feature = "execution")] control: Option<&crate::execution::WorkControl>,
    ) -> Result<Vec<TextSearchResult<'_>>, TextSearchError> {
        let checkpoint = || -> Result<(), TextSearchError> {
            #[cfg(feature = "execution")]
            if let Some(control) = control {
                control.checkpoint()?;
            }
            Ok(())
        };
        checkpoint()?;
        let plan = Plan::new(query, &self.analyzer, options)?;
        let results = self.index.search_analyzed_checked(
            &plan.terms,
            &options.search,
            |id| {
                self.tokens
                    .get(id)
                    .is_some_and(|tokens| plan.includes(tokens, options.operator))
            },
            checkpoint,
            #[cfg(feature = "execution")]
            control,
        )?;
        checkpoint()?;
        Ok(results
            .into_iter()
            .map(|hit| TextSearchResult {
                entry: hit.entry,
                score: hit.score,
                matched_terms: hit.matched_terms,
                phrases: plan
                    .phrases
                    .iter()
                    .map(|p| {
                        let (spans, truncated) =
                            p.matches(&self.tokens[&hit.entry.id], options.max_spans);
                        PhraseEvidence {
                            terms: p.terms.clone(),
                            spans,
                            truncated,
                        }
                    })
                    .collect(),
            })
            .collect())
    }
}
