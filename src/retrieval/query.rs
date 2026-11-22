#![doc = include_str!("../../docs/LEXICAL-QUERY.md")]

use super::RetrievalError;
use crate::text;
use std::collections::BTreeSet;
use unicode_normalization::{char::is_combining_mark, UnicodeNormalization};

#[derive(Debug, Clone)]
pub struct LexicalQueryOptions {
    filter: bool,
    identifiers: bool,
    input_bytes: usize,
    effective_bytes: usize,
    terms: usize,
}
impl Default for LexicalQueryOptions {
    fn default() -> Self {
        Self {
            filter: true,
            identifiers: true,
            input_bytes: 16 * 1024,
            effective_bytes: 64 * 1024,
            terms: 256,
        }
    }
}
impl LexicalQueryOptions {
    pub fn without_word_filter(mut self) -> Self {
        self.filter = false;
        self
    }
    pub fn without_identifier_parts(mut self) -> Self {
        self.identifiers = false;
        self
    }
    pub fn with_limits(
        mut self,
        input_bytes: usize,
        effective_bytes: usize,
        terms: usize,
    ) -> Result<Self, RetrievalError> {
        if input_bytes == 0 || effective_bytes == 0 || terms == 0 {
            return Err(RetrievalError::Configuration(
                "lexical query limits must be positive",
            ));
        }
        self.input_bytes = input_bytes;
        self.effective_bytes = effective_bytes;
        self.terms = terms;
        Ok(self)
    }
    pub fn word_filter(&self) -> bool {
        self.filter
    }
    pub fn identifier_parts(&self) -> bool {
        self.identifiers
    }
    pub fn recipe(&self) -> String {
        format!(
            "airust-lexical-query-v1:{}:{}:{}:{}:{}",
            self.filter, self.identifiers, self.input_bytes, self.effective_bytes, self.terms
        )
    }
    pub fn analyze(&self, query: &str) -> Result<LexicalQueryAnalysis, RetrievalError> {
        if query.trim().is_empty() {
            return Err(RetrievalError::Configuration("lexical query is blank"));
        }
        if query.len() > self.input_bytes {
            return Err(RetrievalError::Budget("lexical query input bytes"));
        }
        let mut cased = String::new();
        for ch in query.nfkc() {
            if ch.len_utf8() > self.effective_bytes.saturating_sub(cased.len()) {
                return Err(RetrievalError::Budget("lexical query normalized bytes"));
            }
            cased.push(ch);
        }
        let original = text::tokenize(&cased);
        let mut originals = Vec::new();
        let mut original_set = BTreeSet::new();
        let mut original_bytes = 0;
        for term in original {
            insert_term(
                term,
                &mut original_set,
                &mut originals,
                &mut original_bytes,
                self,
            )?;
        }
        let mut protected = BTreeSet::new();
        let mut parts = Vec::new();
        let mut part_set = BTreeSet::new();
        let mut part_bytes = 0;
        let mut quote = None;
        let mut run_start = None;
        let mut run_protected = false;
        for (offset, ch) in cased
            .char_indices()
            .chain(std::iter::once((cased.len(), ' ')))
        {
            if ch.is_alphanumeric() || is_combining_mark(ch) {
                if run_start.is_none() {
                    run_start = Some(offset);
                    run_protected = quote.is_some();
                }
                continue;
            }
            if let Some(start) = run_start.take() {
                let run = &cased[start..offset];
                let scoped = cased[..start].ends_with("::")
                    || cased[offset..].starts_with("::")
                    || cased[..start].ends_with('_')
                    || cased[offset..].starts_with('_');
                if run_protected || scoped {
                    protected.extend(text::tokenize(run));
                }
                if self.identifiers {
                    let split = identifier_parts(run);
                    if split.len() > 1 {
                        protected.extend(text::tokenize(run));
                        for part in split {
                            for term in text::tokenize(part) {
                                insert_term(
                                    term,
                                    &mut part_set,
                                    &mut parts,
                                    &mut part_bytes,
                                    self,
                                )?;
                            }
                        }
                    }
                }
            }
            if ch == '`' || ch == '"' {
                if quote == Some(ch) {
                    quote = None;
                } else if quote.is_none() {
                    quote = Some(ch);
                }
            }
        }
        let mut terms = Vec::new();
        let mut seen = BTreeSet::new();
        let mut bytes = 0;
        let mut removed = Vec::new();
        for term in &originals {
            if self.filter && question_word(term) && !protected.contains(term) {
                removed.push(term.clone());
            } else {
                insert_term(term.clone(), &mut seen, &mut terms, &mut bytes, self)?;
            }
        }
        let fallback = terms.is_empty() && !originals.is_empty();
        if fallback {
            removed.clear();
            for term in &originals {
                insert_term(term.clone(), &mut seen, &mut terms, &mut bytes, self)?;
            }
        }
        let mut added = Vec::new();
        for part in parts {
            if (!self.filter || !question_word(&part) || protected.contains(&part))
                && !seen.contains(&part)
            {
                added.push(part.clone());
                insert_term(part, &mut seen, &mut terms, &mut bytes, self)?;
            }
        }
        Ok(LexicalQueryAnalysis {
            effective_query: terms.join(" "),
            removed_terms: removed,
            added_terms: added,
            original_fallback: fallback,
        })
    }
}

#[derive(Debug, Clone)]
pub struct LexicalQueryAnalysis {
    effective_query: String,
    removed_terms: Vec<String>,
    added_terms: Vec<String>,
    original_fallback: bool,
}
impl LexicalQueryAnalysis {
    pub fn effective_query(&self) -> &str {
        &self.effective_query
    }
    pub fn removed_terms(&self) -> &[String] {
        &self.removed_terms
    }
    pub fn added_terms(&self) -> &[String] {
        &self.added_terms
    }
    pub fn original_fallback(&self) -> bool {
        self.original_fallback
    }
}

fn insert_term(
    term: String,
    seen: &mut BTreeSet<String>,
    terms: &mut Vec<String>,
    bytes: &mut usize,
    options: &LexicalQueryOptions,
) -> Result<(), RetrievalError> {
    if seen.contains(&term) {
        return Ok(());
    }
    if terms.len() == options.terms {
        return Err(RetrievalError::Budget("lexical query term count"));
    }
    *bytes = bytes
        .checked_add(term.len())
        .and_then(|n| n.checked_add(usize::from(!terms.is_empty())))
        .filter(|n| *n <= options.effective_bytes)
        .ok_or(RetrievalError::Budget("lexical query effective bytes"))?;
    seen.insert(term.clone());
    terms.push(term);
    Ok(())
}
fn identifier_parts(run: &str) -> Vec<&str> {
    if !run.is_ascii() {
        return vec![run];
    }
    let bytes = run.as_bytes();
    let mut start = 0;
    let mut parts = Vec::new();
    for i in 1..bytes.len() {
        let previous = bytes[i - 1];
        let current = bytes[i];
        let next = bytes.get(i + 1).copied();
        if previous.is_ascii_lowercase() && current.is_ascii_uppercase()
            || previous.is_ascii_alphabetic() && current.is_ascii_digit()
            || previous.is_ascii_digit() && current.is_ascii_alphabetic()
            || previous.is_ascii_uppercase()
                && current.is_ascii_uppercase()
                && next.is_some_and(|c| c.is_ascii_lowercase())
        {
            parts.push(&run[start..i]);
            start = i;
        }
    }
    parts.push(&run[start..]);
    parts
}
fn question_word(term: &str) -> bool {
    text::is_stopword(term)
        || matches!(
            term,
            "was"
                | "wer"
                | "wie"
                | "wo"
                | "wann"
                | "warum"
                | "weshalb"
                | "welche"
                | "welcher"
                | "welches"
                | "welchen"
                | "welchem"
                | "wessen"
                | "what"
                | "who"
                | "how"
                | "where"
                | "when"
                | "why"
                | "which"
                | "does"
                | "do"
                | "did"
                | "are"
                | "sind"
                | "bei"
                | "als"
                | "aus"
                | "ob"
                | "an"
                | "es"
                | "den"
                | "dem"
                | "des"
                | "oder"
                | "im"
                | "am"
                | "nach"
                | "vor"
                | "wird"
                | "werden"
                | "hat"
                | "haben"
                | "sich"
                | "laut"
                | "kann"
                | "können"
                | "darf"
                | "müssen"
                | "muss"
        )
}
