use std::collections::{BTreeMap, BTreeSet};
use unicode_normalization::{char::is_combining_mark, UnicodeNormalization};

#[cfg(any(
    feature = "feedback",
    feature = "learned-ranking",
    feature = "quality-gates",
    feature = "reasoning",
    feature = "adaptive",
    feature = "documents",
    feature = "mcp",
    feature = "local-reranking"
))]
pub(crate) struct Identifier;

#[cfg(any(
    feature = "feedback",
    feature = "learned-ranking",
    feature = "quality-gates",
    feature = "reasoning",
    feature = "adaptive",
    feature = "documents",
    feature = "mcp",
    feature = "local-reranking"
))]
impl Identifier {
    #[cfg(any(
        feature = "feedback",
        feature = "learned-ranking",
        feature = "quality-gates",
        feature = "reasoning",
        feature = "adaptive"
    ))]
    pub(crate) fn label(value: &str, max_bytes: usize) -> bool {
        !value.trim().is_empty() && value.len() <= max_bytes && !value.chars().any(char::is_control)
    }

    #[cfg(any(
        feature = "feedback",
        feature = "learned-ranking",
        feature = "quality-gates",
        feature = "evidence-freshness",
        feature = "documents",
        feature = "mcp",
        feature = "local-reranking"
    ))]
    pub(crate) fn sha256(value: &str) -> bool {
        value.len() == 64
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    }
}

pub fn normalize(text: &str) -> String {
    if text.is_ascii() {
        let mut normalized = String::with_capacity(text.len());
        let mut pending_space = false;
        for byte in text.bytes() {
            if matches!(byte, b' ' | b'\t'..=b'\r') {
                pending_space = !normalized.is_empty();
            } else {
                if pending_space {
                    normalized.push(' ');
                    pending_space = false;
                }
                normalized.push(char::from(byte.to_ascii_lowercase()));
            }
        }
        return normalized;
    }
    let mut normalized = String::with_capacity(text.len());
    let mut pending_space = false;
    for character in normalized_characters(text) {
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

pub(crate) fn normalized_characters(text: &str) -> impl Iterator<Item = char> + '_ {
    text.nfkc().flat_map(char::to_lowercase).nfkc()
}

pub fn tokenize(text: &str) -> Vec<String> {
    let normalized = normalize(text);
    tokens_normalized(&normalized).map(str::to_owned).collect()
}

pub(crate) fn tokens_normalized(normalized: &str) -> impl Iterator<Item = &str> {
    normalized
        .split(|c: char| !c.is_alphanumeric() && !is_combining_mark(c))
        .filter(|token| token.chars().any(char::is_alphanumeric))
}

pub fn term_frequencies(text: &str) -> BTreeMap<String, usize> {
    frequencies_normalized(&normalize(text))
}

pub(crate) fn frequencies_normalized(normalized: &str) -> BTreeMap<String, usize> {
    let mut frequencies = BTreeMap::new();
    let mut reuse_keys = false;
    for token in tokens_normalized(normalized) {
        if reuse_keys {
            if let Some(count) = frequencies.get_mut(token) {
                *count += 1;
                continue;
            }
        }
        match frequencies.entry(token.to_owned()) {
            std::collections::btree_map::Entry::Occupied(mut entry) => {
                *entry.get_mut() += 1;
                reuse_keys = true;
            }
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(1);
            }
        }
    }
    frequencies
}

pub fn unique_terms(text: &str) -> BTreeSet<String> {
    tokenize(text).into_iter().collect()
}

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

#[cfg(all(test, feature = "feedback"))]
mod identifier_tests {
    use super::Identifier;

    #[test]
    fn labels_are_bounded_in_bytes_and_reject_blank_or_control_text() {
        assert!(Identifier::label("ok", 2));
        assert!(!Identifier::label("ä", 1));
        assert!(Identifier::label("ä", 2));
        assert!(!Identifier::label("  ", 8));
        assert!(!Identifier::label("a\u{7}", 8));
        assert!(!Identifier::label("abc", 2));
    }

    #[test]
    fn sha256_accepts_only_sixty_four_lowercase_hex_digits() {
        assert!(Identifier::sha256(&"a".repeat(64)));
        assert!(!Identifier::sha256(&"A".repeat(64)));
        assert!(!Identifier::sha256(&"a".repeat(63)));
        assert!(!Identifier::sha256(&"g".repeat(64)));
    }
}
