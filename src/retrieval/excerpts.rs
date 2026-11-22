#![doc = include_str!("../../docs/EXCERPTS.md")]

use super::{ExcerptSpan, RankingEvidence, RetrievalError};
use crate::text;
use serde::Serialize;
use std::{
    cmp::Reverse,
    collections::{BTreeSet, BinaryHeap},
};
use unicode_normalization::char::is_combining_mark;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone)]
pub struct ExcerptOptions {
    max_input_bytes: usize,
    max_normalized_bytes: usize,
    max_tokens: usize,
    max_query_bytes: usize,
    max_terms: usize,
    max_matches: usize,
    max_highlights: usize,
    paragraph_boundaries: bool,
    leading_context: bool,
}
impl Default for ExcerptOptions {
    fn default() -> Self {
        Self {
            max_input_bytes: 1024 * 1024,
            max_normalized_bytes: 4 * 1024 * 1024,
            max_tokens: 100_000,
            max_query_bytes: 16 * 1024,
            max_terms: 256,
            max_matches: 4096,
            max_highlights: 64,
            paragraph_boundaries: false,
            leading_context: false,
        }
    }
}
impl ExcerptOptions {
    pub fn with_paragraph_boundaries(mut self) -> Self {
        self.paragraph_boundaries = true;
        self.leading_context = false;
        self
    }
    pub fn with_leading_context(mut self) -> Self {
        self.leading_context = true;
        self.paragraph_boundaries = false;
        self
    }
    pub fn leading_context(&self) -> bool {
        self.leading_context
    }
    pub fn paragraph_boundaries(&self) -> bool {
        self.paragraph_boundaries
    }
    pub fn with_analysis_limits(
        mut self,
        input: usize,
        normalized: usize,
        tokens: usize,
    ) -> Result<Self, RetrievalError> {
        if [input, normalized, tokens].contains(&0) {
            return Err(RetrievalError::Configuration(
                "excerpt analysis limits must be positive",
            ));
        }
        self.max_input_bytes = input;
        self.max_normalized_bytes = normalized;
        self.max_tokens = tokens;
        Ok(self)
    }
    pub fn with_query_limits(mut self, bytes: usize, terms: usize) -> Result<Self, RetrievalError> {
        if bytes == 0 || terms == 0 {
            return Err(RetrievalError::Configuration(
                "excerpt query limits must be positive",
            ));
        }
        self.max_query_bytes = bytes;
        self.max_terms = terms;
        Ok(self)
    }
    pub fn with_match_limits(
        mut self,
        matches: usize,
        highlights: usize,
    ) -> Result<Self, RetrievalError> {
        if matches == 0 || highlights == 0 {
            return Err(RetrievalError::Configuration(
                "excerpt match limits must be positive",
            ));
        }
        self.max_matches = matches;
        self.max_highlights = highlights;
        Ok(self)
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
    pub fn max_query_bytes(&self) -> usize {
        self.max_query_bytes
    }
    pub fn max_terms(&self) -> usize {
        self.max_terms
    }
    pub fn max_matches(&self) -> usize {
        self.max_matches
    }
    pub fn max_highlights(&self) -> usize {
        self.max_highlights
    }
}
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub enum ExcerptAnalyzer {
    #[default]
    Legacy,
    #[cfg(feature = "advanced-search")]
    Text(crate::text_search::AnalyzerConfig),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ExcerptReason {
    Phrase,
    Terms,
    NoLexicalMatch,
    NoFittingMatch,
    EmptyBudget,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ExcerptMatchKind {
    Term,
    Phrase,
}
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct ExcerptMatch {
    pub span: ExcerptSpan,
    pub kind: ExcerptMatchKind,
    pub term: Option<String>,
}
#[derive(Debug, Serialize)]
#[non_exhaustive]
pub struct ExcerptDetails {
    pub id: String,
    pub reason: ExcerptReason,
    pub matches: Vec<ExcerptMatch>,
    pub matches_truncated: bool,
}
#[derive(Debug)]
#[non_exhaustive]
pub struct SelectedExcerpt<'a> {
    pub text: &'a str,
    pub span: ExcerptSpan,
    pub reason: ExcerptReason,
    pub matches: Vec<ExcerptMatch>,
    pub matches_truncated: bool,
}
struct Token {
    term: String,
    span: ExcerptSpan,
}
pub(super) fn analyzed_terms(
    text: &str,
    analyzer: &ExcerptAnalyzer,
    options: &ExcerptOptions,
) -> Result<Vec<String>, RetrievalError> {
    Ok(analyze(text, analyzer, options)?
        .into_iter()
        .map(|t| t.term)
        .collect())
}
fn analyze(
    text: &str,
    analyzer: &ExcerptAnalyzer,
    options: &ExcerptOptions,
) -> Result<Vec<Token>, RetrievalError> {
    if text.len() > options.max_input_bytes {
        return Err(RetrievalError::Budget("excerpt input bytes"));
    }
    match analyzer {
        #[cfg(feature = "advanced-search")]
        ExcerptAnalyzer::Text(config) => {
            let config = config
                .clone()
                .with_limits(
                    options.max_input_bytes,
                    options.max_normalized_bytes,
                    options.max_tokens,
                )
                .map_err(|_| RetrievalError::Configuration("excerpt analyzer limits"))?;
            crate::text_search::TextAnalyzer::new(config)
                .analyze(text)
                .map(|tokens| {
                    tokens
                        .into_iter()
                        .map(|t| Token {
                            term: t.term,
                            span: ExcerptSpan {
                                start: t.span.start,
                                end: t.span.end,
                            },
                        })
                        .collect()
                })
                .map_err(|_| RetrievalError::Budget("excerpt text analysis"))
        }
        ExcerptAnalyzer::Legacy => legacy_tokens(text, options),
    }
}
fn legacy_tokens(source: &str, options: &ExcerptOptions) -> Result<Vec<Token>, RetrievalError> {
    let mut normalized = String::new();
    let mut mapping = Vec::new();
    for (start, grapheme) in source.grapheme_indices(true) {
        let begin = normalized.len();
        for c in text::normalized_characters(grapheme) {
            if c.len_utf8()
                > options
                    .max_normalized_bytes
                    .saturating_sub(normalized.len())
            {
                return Err(RetrievalError::Budget("excerpt normalized bytes"));
            }
            normalized.push(c);
        }
        if normalized.len() > begin {
            mapping.push((begin, normalized.len(), start, start + grapheme.len()));
        }
    }
    let mut tokens = Vec::new();
    let mut start = None;
    for (offset, c) in normalized
        .char_indices()
        .chain(std::iter::once((normalized.len(), ' ')))
    {
        if c.is_alphanumeric() || is_combining_mark(c) {
            if start.is_none() {
                start = Some(offset);
            }
        } else if let Some(begin) = start.take() {
            let word = &normalized[begin..offset];
            if !word.chars().any(char::is_alphanumeric) {
                continue;
            }
            if tokens.len() == options.max_tokens {
                return Err(RetrievalError::Budget("excerpt token count"));
            }
            let first = mapping.partition_point(|m| m.1 <= begin);
            let last = mapping.partition_point(|m| m.0 < offset) - 1;
            tokens.push(Token {
                term: text::normalize(word),
                span: ExcerptSpan {
                    start: mapping[first].2,
                    end: mapping[last].3,
                },
            });
        }
    }
    Ok(tokens)
}
#[derive(Clone, Copy)]
struct Match {
    span: ExcerptSpan,
    term: Option<usize>,
}
fn floor(boundaries: &[usize], offset: usize) -> usize {
    boundaries[boundaries.partition_point(|p| *p <= offset) - 1]
}
fn ceil(boundaries: &[usize], offset: usize) -> usize {
    boundaries[boundaries.partition_point(|p| *p < offset)]
}
fn fallback<'a>(
    text: &'a str,
    boundaries: &[usize],
    budget: usize,
    reason: ExcerptReason,
) -> SelectedExcerpt<'a> {
    let end = floor(boundaries, budget.min(text.len()));
    SelectedExcerpt {
        text: &text[..end],
        span: ExcerptSpan { start: 0, end },
        reason,
        matches: Vec::new(),
        matches_truncated: false,
    }
}
pub fn select_excerpt<'a>(
    text: &'a str,
    query: &str,
    evidence: &RankingEvidence,
    analyzer: &ExcerptAnalyzer,
    options: &ExcerptOptions,
    budget: usize,
) -> Result<SelectedExcerpt<'a>, RetrievalError> {
    if query.trim().is_empty() {
        return Err(RetrievalError::Configuration("excerpt query is blank"));
    }
    if query.len() > options.max_query_bytes {
        return Err(RetrievalError::Budget("excerpt query bytes"));
    }
    if budget == 0 {
        return Ok(SelectedExcerpt {
            text: "",
            span: ExcerptSpan { start: 0, end: 0 },
            reason: ExcerptReason::EmptyBudget,
            matches: Vec::new(),
            matches_truncated: false,
        });
    }
    if text.len() > options.max_input_bytes {
        return Err(RetrievalError::Budget("excerpt input bytes"));
    }
    let boundaries: Vec<_> = text
        .grapheme_indices(true)
        .map(|(p, _)| p)
        .chain(std::iter::once(text.len()))
        .collect();
    if evidence.matched_terms.len() > options.max_terms {
        return Err(RetrievalError::Budget("excerpt query terms"));
    }
    let mut term_bytes = 0usize;
    for term in &evidence.matched_terms {
        term_bytes = term_bytes
            .checked_add(term.len())
            .filter(|n| *n <= options.max_normalized_bytes)
            .ok_or(RetrievalError::Budget("excerpt term bytes"))?;
        if term.is_empty() {
            return Err(RetrievalError::Output("invalid excerpt term"));
        }
    }
    let terms: BTreeSet<_> = if evidence.matched_terms.is_empty() {
        analyze(query, analyzer, options)?
            .into_iter()
            .map(|t| t.term)
            .collect()
    } else {
        evidence.matched_terms.iter().cloned().collect()
    };
    if terms.len() > options.max_terms {
        return Err(RetrievalError::Budget("excerpt query terms"));
    }
    if terms
        .iter()
        .any(|t| t.is_empty() || t.len() > options.max_normalized_bytes)
    {
        return Err(RetrievalError::Output("invalid excerpt term"));
    }
    let terms: Vec<_> = terms.into_iter().collect();
    let mut matches = Vec::new();
    for token in analyze(text, analyzer, options)? {
        if let Ok(term) = terms.binary_search(&token.term) {
            if matches.len() == options.max_matches {
                return Err(RetrievalError::Budget("excerpt match count"));
            }
            matches.push(Match {
                span: token.span,
                term: Some(term),
            });
        }
    }
    for span in &evidence.phrase_spans {
        if span.start >= span.end
            || span.slice(text).is_none()
            || boundaries.binary_search(&span.start).is_err()
            || boundaries.binary_search(&span.end).is_err()
        {
            return Err(RetrievalError::Output("invalid excerpt phrase boundary"));
        }
        if matches.len() == options.max_matches {
            return Err(RetrievalError::Budget("excerpt match count"));
        }
        matches.push(Match {
            span: *span,
            term: None,
        });
    }
    matches.sort_by_key(|m| (m.span.start, m.span.end, m.term));
    matches.dedup_by_key(|m| (m.span.start, m.span.end, m.term));
    if matches.is_empty() {
        return Ok(fallback(
            text,
            &boundaries,
            budget,
            ExcerptReason::NoLexicalMatch,
        ));
    }
    matches.retain(|m| m.span.end - m.span.start <= budget);
    if matches.is_empty() {
        return Ok(fallback(
            text,
            &boundaries,
            budget,
            ExcerptReason::NoFittingMatch,
        ));
    }
    let (start, end) = best_window(&matches, &boundaries, budget, terms.len());
    let inside: Vec<_> = matches
        .iter()
        .filter(|m| m.span.start >= start && m.span.end <= end)
        .collect();
    let focus_start = inside
        .iter()
        .map(|m| m.span.start)
        .min()
        .expect("selected nonempty window");
    let focus_end = inside
        .iter()
        .map(|m| m.span.end)
        .max()
        .expect("selected nonempty window");
    let spare = budget.saturating_sub(focus_end - focus_start);
    let desired = floor(
        &boundaries,
        focus_start
            .saturating_sub(spare / 2)
            .min(text.len().saturating_sub(budget)),
    );
    let mut start = ceil(&boundaries, desired.max(focus_end.saturating_sub(budget)));
    let mut end = floor(&boundaries, start.saturating_add(budget).min(text.len()));
    if options.leading_context {
        let prefix_end = floor(&boundaries, budget.min(text.len()));
        let mut prefix_terms = vec![false; terms.len()];
        for matched in matches.iter().filter(|m| m.span.end <= prefix_end) {
            if let Some(term) = matched.term {
                prefix_terms[term] = true;
            }
        }
        if matches
            .iter()
            .filter(|m| m.span.start >= start && m.span.end <= end)
            .all(|m| {
                m.term
                    .map_or(m.span.end <= prefix_end, |term| prefix_terms[term])
            })
        {
            start = 0;
            end = prefix_end;
        }
    }
    if options.paragraph_boundaries {
        let mut required_terms = vec![false; terms.len()];
        let mut phrase_bounds: Option<ExcerptSpan> = None;
        for matched in matches
            .iter()
            .filter(|m| m.span.start >= start && m.span.end <= end)
        {
            if let Some(term) = matched.term {
                required_terms[term] = true;
            } else {
                phrase_bounds = Some(phrase_bounds.map_or(matched.span, |bounds| ExcerptSpan {
                    start: bounds.start.min(matched.span.start),
                    end: bounds.end.max(matched.span.end),
                }));
            }
        }
        if let Some((paragraph_start, paragraph_end)) =
            paragraph_window(text, &matches, &required_terms, budget, phrase_bounds)
        {
            start = paragraph_start;
            end = paragraph_end;
        }
    }
    let mut highlights = Vec::new();
    let mut count = 0usize;
    let mut phrase = false;
    for m in &matches {
        if m.span.start < start || m.span.end > end {
            continue;
        }
        count += 1;
        phrase |= m.term.is_none();
        if highlights.len() < options.max_highlights {
            highlights.push(ExcerptMatch {
                span: m.span,
                kind: if m.term.is_some() {
                    ExcerptMatchKind::Term
                } else {
                    ExcerptMatchKind::Phrase
                },
                term: m.term.map(|i| terms[i].clone()),
            });
        }
    }
    Ok(SelectedExcerpt {
        text: &text[start..end],
        span: ExcerptSpan { start, end },
        reason: if phrase {
            ExcerptReason::Phrase
        } else {
            ExcerptReason::Terms
        },
        matches: highlights,
        matches_truncated: count > options.max_highlights,
    })
}
fn paragraph_window(
    text: &str,
    matches: &[Match],
    required_terms: &[bool],
    budget: usize,
    phrase_bounds: Option<ExcerptSpan>,
) -> Option<(usize, usize)> {
    let mut paragraph_start = 0;
    let mut line_start = 0;
    let mut cursor = 0;
    let mut markers = vec![0usize; required_terms.len()];
    let required_count = required_terms.iter().filter(|required| **required).count();
    let mut sequence = 0usize;
    let mut best = None;
    let mut best_score = (false, 0usize, 0usize);
    for line in text.split_inclusive('\n').chain(std::iter::once("")) {
        let end = line_start;
        line_start += line.len();
        if !line.trim().is_empty() {
            continue;
        }
        let start = paragraph_start;
        paragraph_start = line_start;
        sequence += 1;
        while cursor < matches.len() && matches[cursor].span.start < start {
            cursor += 1;
        }
        let mut phrase = false;
        let mut distinct = 0;
        let mut retained_required = 0;
        let mut occurrences = 0;
        while cursor < matches.len() && matches[cursor].span.start < end {
            let m = matches[cursor];
            cursor += 1;
            if end - start > budget || m.span.end > end {
                continue;
            }
            if let Some(term) = m.term {
                if markers[term] != sequence {
                    distinct += 1;
                    retained_required += usize::from(required_terms[term]);
                    markers[term] = sequence;
                }
                occurrences += 1;
            } else {
                phrase = true;
            }
        }
        let score = (phrase, distinct, occurrences);
        let preserves_phrases =
            phrase_bounds.is_none_or(|span| start <= span.start && span.end <= end);
        if retained_required == required_count && preserves_phrases && score > best_score {
            best_score = score;
            best = Some((start, end));
        }
    }
    best
}
fn best_window(
    matches: &[Match],
    boundaries: &[usize],
    budget: usize,
    terms: usize,
) -> (usize, usize) {
    let mut starts = vec![0];
    for m in matches {
        starts.push(m.span.start);
        starts.push(ceil(boundaries, m.span.end.saturating_sub(budget)));
    }
    starts.sort_unstable();
    starts.dedup();
    let mut ends: Vec<_> = (0..matches.len()).collect();
    ends.sort_unstable_by_key(|i| matches[*i].span.end);
    let mut queue = BinaryHeap::new();
    let mut counts = vec![0usize; terms];
    let (mut next, mut phrases, mut distinct, mut occurrences) = (0usize, 0usize, 0usize, 0usize);
    let (mut best, mut score) = ((0, 0), (false, 0, 0));
    for start in starts {
        let end = floor(
            boundaries,
            start
                .saturating_add(budget)
                .min(*boundaries.last().unwrap()),
        );
        while next < ends.len() && matches[ends[next]].span.end <= end {
            let i = ends[next];
            next += 1;
            let m = matches[i];
            if m.span.start < start {
                continue;
            }
            queue.push(Reverse((m.span.start, i)));
            if let Some(t) = m.term {
                distinct += usize::from(counts[t] == 0);
                counts[t] += 1;
                occurrences += 1;
            } else {
                phrases += 1;
            }
        }
        while queue.peek().is_some_and(|Reverse((p, _))| *p < start) {
            let Reverse((_, i)) = queue.pop().unwrap();
            if let Some(t) = matches[i].term {
                counts[t] -= 1;
                distinct -= usize::from(counts[t] == 0);
                occurrences -= 1;
            } else {
                phrases -= 1;
            }
        }
        let current = (phrases > 0, distinct, occurrences);
        if current > score {
            score = current;
            best = (start, end);
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    fn score(matches: &[Match], start: usize, end: usize) -> (bool, usize, usize) {
        let contained: Vec<_> = matches
            .iter()
            .filter(|m| m.span.start >= start && m.span.end <= end)
            .collect();
        let phrase = contained.iter().any(|m| m.term.is_none());
        let distinct = contained
            .iter()
            .filter_map(|m| m.term)
            .collect::<BTreeSet<_>>()
            .len();
        let occurrences = contained.iter().filter(|m| m.term.is_some()).count();
        (phrase, distinct, occurrences)
    }
    #[test]
    fn sweep_matches_exhaustive_windows_with_nested_spans_and_variable_graphemes() {
        let mut seed = 17u64;
        for _ in 0..1500 {
            let mut next = || {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                (seed >> 32) as usize
            };
            let mut boundaries = vec![0];
            for _ in 0..25 {
                boundaries.push(boundaries.last().unwrap() + [1, 2, 4][next() % 3]);
            }
            let budget = next() % 30 + 1;
            let mut matches = Vec::new();
            for _ in 0..20 {
                let start = next() % 25;
                let end = (start + 1 + next() % 8).min(25);
                let span = ExcerptSpan {
                    start: boundaries[start],
                    end: boundaries[end],
                };
                if span.end - span.start <= budget {
                    matches.push(Match {
                        span,
                        term: match next() % 4 {
                            3 => None,
                            i => Some(i),
                        },
                    });
                }
            }
            if matches.is_empty() {
                continue;
            }
            let actual = best_window(&matches, &boundaries, budget, 3);
            let mut expected = ((0, 0), (false, 0, 0));
            for start in &boundaries {
                let end = *boundaries
                    .iter()
                    .filter(|p| **p <= start.saturating_add(budget))
                    .max()
                    .unwrap();
                let s = score(&matches, *start, end);
                if s > expected.1 {
                    expected = ((*start, end), s);
                }
            }
            assert_eq!(score(&matches, actual.0, actual.1), expected.1);
            assert_eq!(actual, expected.0);
        }
    }
}
