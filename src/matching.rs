use crate::{knowledge::KnowledgeEntry, text};
use serde::Serialize;
use thiserror::Error;

pub use strsim::levenshtein as levenshtein_distance;

pub(crate) fn bounded_distance(a: &str, b: &str, limit: usize) -> Option<usize> {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.len().abs_diff(b.len()) > limit {
        return None;
    }
    if a.is_empty() {
        return (b.len() <= limit).then_some(b.len());
    }
    if b.is_empty() {
        return (a.len() <= limit).then_some(a.len());
    }
    let limit = limit.min(a.len().max(b.len()));
    let unreachable = limit + 1;
    let mut previous = vec![unreachable; b.len() + 1];
    let mut current = vec![unreachable; b.len() + 1];
    for (i, cell) in previous
        .iter_mut()
        .enumerate()
        .take(limit.saturating_add(1))
    {
        *cell = i;
    }
    for (row, left) in a.iter().enumerate() {
        let row = row + 1;
        let start = row.saturating_sub(limit).max(1);
        let end = row.saturating_add(limit).min(b.len());
        current[0] = if row <= limit { row } else { unreachable };
        if start > 1 {
            current[start - 1] = unreachable;
        }
        let mut best = current[0];
        for column in start..=end {
            current[column] = (previous[column] + 1)
                .min(current[column - 1] + 1)
                .min(previous[column - 1] + usize::from(*left != b[column - 1]));
            best = best.min(current[column]);
        }
        if end < b.len() {
            current[end + 1] = unreachable;
        }
        if best > limit {
            return None;
        }
        std::mem::swap(&mut previous, &mut current);
    }
    (previous[b.len()] <= limit).then_some(previous[b.len()])
}

pub(crate) fn distance_limit(length: usize, threshold: f64) -> usize {
    let mut limit = ((1.0 - threshold) * length as f64) as usize;
    while limit < length && 1.0 - (limit + 1) as f64 / length as f64 >= threshold {
        limit += 1;
    }
    while limit > 0 && (1.0 - limit as f64 / length as f64) < threshold {
        limit -= 1;
    }
    limit
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Similarity {
    pub exact: bool,

    pub character: f64,

    pub token: f64,
}

pub fn compare(a: &str, b: &str) -> Similarity {
    compare_normalized(&text::normalize(a), &text::normalize(b))
}

pub(crate) fn compare_normalized(a: &str, b: &str) -> Similarity {
    if a.is_empty() || b.is_empty() {
        return Similarity {
            exact: false,
            character: 0.0,
            token: 0.0,
        };
    }
    let max_len = a.chars().count().max(b.chars().count());
    let terms_a = text::unique_terms(a);
    let terms_b = text::unique_terms(b);
    let union = terms_a.union(&terms_b).count();
    Similarity {
        exact: a == b,
        character: 1.0 - levenshtein_distance(a, b) as f64 / max_len as f64,
        token: if union == 0 {
            0.0
        } else {
            terms_a.intersection(&terms_b).count() as f64 / union as f64
        },
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub enum MatchStrategy {
    Exact,
    #[default]
    Fuzzy,
    TokenOverlap,
}

#[derive(Debug, Clone)]
pub struct MatchOptions {
    pub strategy: MatchStrategy,
    pub min_similarity: f64,
    pub limit: usize,
}

impl Default for MatchOptions {
    fn default() -> Self {
        Self {
            strategy: MatchStrategy::Fuzzy,
            min_similarity: 0.8,
            limit: 10,
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("min_similarity must be finite and between 0 and 1")]
pub struct MatchError;

impl MatchOptions {
    pub(crate) fn validate(&self) -> Result<(), MatchError> {
        if !self.min_similarity.is_finite() || !(0.0..=1.0).contains(&self.min_similarity) {
            return Err(MatchError);
        }
        Ok(())
    }
}

#[cfg(test)]
mod bounded_tests {
    use super::*;

    #[test]
    fn bounded_distance_agrees_with_full_unicode_distance() {
        let mut words = vec![String::new()];
        let mut frontier = vec![String::new()];
        for _ in 0..4 {
            frontier = frontier
                .iter()
                .flat_map(|word| ['a', 'b', 'ä'].map(|letter| format!("{word}{letter}")))
                .collect();
            words.extend(frontier.iter().cloned());
        }
        for left in &words {
            for right in &words {
                let actual = levenshtein_distance(left, right);
                for limit in 0..=4 {
                    assert_eq!(
                        bounded_distance(left, right, limit),
                        (actual <= limit).then_some(actual),
                        "{left:?} {right:?} {limit}"
                    );
                }
            }
        }
        assert_eq!(bounded_distance("a", "bbb", usize::MAX), Some(3));
    }

    #[test]
    fn distance_threshold_preserves_floating_point_boundaries() {
        for length in 1..=100 {
            for distance in 0..=length {
                let threshold = 1.0 - distance as f64 / length as f64;
                let limit = distance_limit(length, threshold);
                assert!(1.0 - limit as f64 / length as f64 >= threshold);
                assert!(limit == length || 1.0 - (limit + 1) as f64 / (length as f64) < threshold);
            }
        }
    }

    #[cfg(feature = "execution")]
    #[test]
    fn controlled_distance_matches_independent_unicode_oracle() {
        let mut words = vec![String::new()];
        let mut frontier = words.clone();
        for _ in 0..4 {
            frontier = frontier
                .iter()
                .flat_map(|word| ['a', 'ä', '🌍'].map(|c| format!("{word}{c}")))
                .collect();
            words.extend(frontier.iter().cloned());
        }
        for a in &words {
            for b in &words {
                let oracle = strsim::levenshtein(a, b);
                for maximum in [None, Some(0), Some(1), Some(2), Some(4), Some(usize::MAX)] {
                    let actual = distance_checked(a, b, maximum, &mut || Ok::<_, ()>(())).unwrap();
                    assert_eq!(
                        actual,
                        maximum.is_none_or(|n| oracle <= n).then_some(oracle)
                    );
                }
            }
        }
    }

    #[cfg(feature = "execution")]
    #[test]
    fn controlled_distance_checks_inside_a_long_row() {
        let text = "ab".repeat(20000);
        let mut checks = 0;
        let result = distance_checked(&text, &text, None, &mut || {
            checks += 1;
            if checks == 10 {
                Err("cancelled inside row")
            } else {
                Ok(())
            }
        });
        assert_eq!(result, Err("cancelled inside row"));
        assert_eq!(checks, 10);
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MatchResult<'a> {
    pub entry: &'a KnowledgeEntry,
    pub score: f64,
    pub similarity: Similarity,
}

#[cfg(feature = "execution")]
pub(crate) fn distance_checked<E>(
    a: &str,
    b: &str,
    maximum: Option<usize>,
    checkpoint: &mut impl FnMut() -> Result<(), E>,
) -> Result<Option<usize>, E> {
    checkpoint()?;
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    checkpoint()?;
    let (a, b) = if a.len() < b.len() { (b, a) } else { (a, b) };
    let limit = maximum
        .unwrap_or(a.len().max(b.len()))
        .min(a.len().max(b.len()));
    if a.len().abs_diff(b.len()) > limit {
        return Ok(None);
    }
    if a.is_empty() {
        return Ok((b.len() <= limit).then_some(b.len()));
    }
    if b.is_empty() {
        return Ok((a.len() <= limit).then_some(a.len()));
    }
    let unreachable = limit + 1;
    let mut previous = vec![unreachable; b.len() + 1];
    let mut current = vec![unreachable; b.len() + 1];
    for (i, cell) in previous
        .iter_mut()
        .enumerate()
        .take(limit.saturating_add(1))
    {
        *cell = i;
    }
    for (index, left) in a.iter().enumerate() {
        checkpoint()?;
        let row = index + 1;
        let start = row.saturating_sub(limit).max(1);
        let end = row.saturating_add(limit).min(b.len());
        current[0] = if row <= limit { row } else { unreachable };
        if start > 1 {
            current[start - 1] = unreachable;
        }
        let mut best = current[0];
        for column in start..=end {
            if column.is_multiple_of(256) {
                checkpoint()?;
            }
            current[column] = (previous[column] + 1)
                .min(current[column - 1] + 1)
                .min(previous[column - 1] + usize::from(*left != b[column - 1]));
            best = best.min(current[column]);
        }
        if end < b.len() {
            current[end + 1] = unreachable;
        }
        if best > limit {
            return Ok(None);
        }
        std::mem::swap(&mut previous, &mut current);
    }
    checkpoint()?;
    Ok((previous[b.len()] <= limit).then_some(previous[b.len()]))
}
