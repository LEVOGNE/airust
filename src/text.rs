//! Shared, language-neutral text processing for the knowledge engine.

use std::collections::{BTreeMap, BTreeSet};
use unicode_normalization::{char::is_combining_mark, UnicodeNormalization};

/// Apply Unicode NFKC, lowercase, and collapse whitespace.
///
/// Accents and punctuation remain significant. This is Unicode normalization,
/// not stemming, full Unicode case folding, or synonym recognition.
pub fn normalize(text: &str) -> String {
    let mut normalized = String::with_capacity(text.len());
    let mut pending_space = false;
    for character in text.nfkc().flat_map(char::to_lowercase).nfkc() {
        if character.is_whitespace() {
            pending_space = !normalized.is_empty();
        } else {
            if pending_space {
                normalized.push(' ');
                pending_space = false;
            }
            normalized.push(character);
        }
    }
    normalized
}

/// Split at punctuation while preserving Unicode letters, numbers, and marks.
/// No stopwords are removed automatically: identifiers and short queries matter.
pub fn tokenize(text: &str) -> Vec<String> {
    let normalized = normalize(text);
    tokens_normalized(&normalized).map(str::to_owned).collect()
}

pub(crate) fn tokens_normalized(normalized: &str) -> impl Iterator<Item = &str> {
    normalized
        .split(|c: char| !c.is_alphanumeric() && !is_combining_mark(c))
        .filter(|token| token.chars().any(char::is_alphanumeric))
}

/// Count tokens in deterministic term order.
pub fn term_frequencies(text: &str) -> BTreeMap<String, usize> {
    frequencies_normalized(&normalize(text))
}

pub(crate) fn frequencies_normalized(normalized: &str) -> BTreeMap<String, usize> {
    let mut frequencies = BTreeMap::new();
    for token in tokens_normalized(normalized) {
        *frequencies.entry(token.to_owned()).or_insert(0) += 1;
    }
    frequencies
}

/// Return unique normalized terms in deterministic order.
pub fn unique_terms(text: &str) -> BTreeSet<String> {
    tokenize(text).into_iter().collect()
}

/// Common English and German function words, used only by relation suggestions.
pub fn is_stopword(term: &str) -> bool {
    matches!(
        term,
        "der"
            | "die"
            | "das"
            | "und"
            | "in"
            | "ist"
            | "von"
            | "mit"
            | "zum"
            | "zur"
            | "zu"
            | "ein"
            | "eine"
            | "eines"
            | "the"
            | "and"
            | "is"
            | "of"
            | "to"
            | "a"
            | "with"
            | "for"
            | "on"
            | "at"
            | "this"
            | "that"
    )
}
