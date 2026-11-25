#![doc = include_str!("../../docs/CITATIONS.md")]

use super::{ExcerptSpan, RetrievalResponse};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CitationError {
    #[error("invalid citation configuration: {0}")]
    Configuration(&'static str),
    #[error("citation budget exceeded: {0}")]
    Budget(&'static str),
    #[error("invalid cited answer: {0}")]
    Answer(&'static str),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
pub struct CitationLimits {
    input_bytes: usize,
    answer_bytes: usize,
    citations: usize,
    id_bytes: usize,
    quote_bytes: usize,
}
impl Default for CitationLimits {
    fn default() -> Self {
        Self {
            input_bytes: 128 * 1024,
            answer_bytes: 64 * 1024,
            citations: 128,
            id_bytes: 4096,
            quote_bytes: 8192,
        }
    }
}
impl CitationLimits {
    pub fn with_input_bytes(mut self, n: usize) -> Result<Self, CitationError> {
        if n == 0 {
            return Err(CitationError::Configuration("input bytes must be positive"));
        }
        self.input_bytes = n;
        Ok(self)
    }
    pub fn with_answer_bytes(mut self, n: usize) -> Result<Self, CitationError> {
        if n == 0 {
            return Err(CitationError::Configuration(
                "answer bytes must be positive",
            ));
        }
        self.answer_bytes = n;
        Ok(self)
    }
    pub fn with_citations(mut self, n: usize) -> Result<Self, CitationError> {
        if n == 0 {
            return Err(CitationError::Configuration(
                "citation count must be positive",
            ));
        }
        self.citations = n;
        Ok(self)
    }
    pub fn with_reference_bytes(mut self, id: usize, quote: usize) -> Result<Self, CitationError> {
        if id == 0 || quote == 0 {
            return Err(CitationError::Configuration(
                "ID and quote bytes must be positive",
            ));
        }
        self.id_bytes = id;
        self.quote_bytes = quote;
        Ok(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Citation {
    id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    quote: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    span: Option<ExcerptSpan>,
}
impl Citation {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            quote: None,
            span: None,
        }
    }
    pub fn with_quote(mut self, quote: impl Into<String>) -> Self {
        self.quote = Some(quote.into());
        self
    }
    pub fn with_span(mut self, span: ExcerptSpan) -> Self {
        self.span = Some(span);
        self
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn quote(&self) -> Option<&str> {
        self.quote.as_deref()
    }
    pub fn span(&self) -> Option<ExcerptSpan> {
        self.span
    }
}

#[derive(Debug, Serialize)]
pub struct CitedAnswer {
    answer: Option<String>,
    citations: Vec<Citation>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireAnswer {
    #[serde(deserialize_with = "required_answer")]
    answer: Option<String>,
    citations: Vec<Citation>,
}
fn required_answer<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    Option::<String>::deserialize(deserializer)
}
impl CitedAnswer {
    pub fn new(
        answer: Option<String>,
        citations: Vec<Citation>,
        limits: &CitationLimits,
    ) -> Result<Self, CitationError> {
        let value = Self { answer, citations };
        value.validate(limits)?;
        Ok(value)
    }
    pub fn from_json(input: &str, limits: &CitationLimits) -> Result<Self, CitationError> {
        if input.len() > limits.input_bytes {
            return Err(CitationError::Budget("encoded input bytes"));
        }
        let wire: WireAnswer = serde_json::from_str(input).map_err(crate::input::json_error)?;
        Self::new(wire.answer, wire.citations, limits)
    }
    fn validate(&self, limits: &CitationLimits) -> Result<(), CitationError> {
        let mut decoded_bytes = self.answer.as_ref().map_or(0, String::len);
        if decoded_bytes > limits.input_bytes {
            return Err(CitationError::Budget("decoded answer/reference bytes"));
        }
        if self.citations.len() > limits.citations {
            return Err(CitationError::Budget("citation count"));
        }
        if let Some(answer) = &self.answer {
            if answer.len() > limits.answer_bytes {
                return Err(CitationError::Budget("answer bytes"));
            }
            if answer.trim().is_empty() || answer.trim().eq_ignore_ascii_case("null") {
                return Err(CitationError::Answer(
                    "use JSON null for abstention, not blank text or a null string",
                ));
            }
            if self.citations.is_empty() {
                return Err(CitationError::Answer(
                    "a text answer needs at least one reference",
                ));
            }
        } else if !self.citations.is_empty() {
            return Err(CitationError::Answer(
                "an abstention must have no references",
            ));
        }
        for citation in &self.citations {
            decoded_bytes = decoded_bytes
                .checked_add(citation.id.len())
                .and_then(|n| n.checked_add(citation.quote.as_ref().map_or(0, String::len)))
                .filter(|n| *n <= limits.input_bytes)
                .ok_or(CitationError::Budget("decoded answer/reference bytes"))?;
            if citation.id.len() > limits.id_bytes {
                return Err(CitationError::Budget("reference ID bytes"));
            }
            if citation.id.trim().is_empty() || citation.id.chars().any(char::is_control) {
                return Err(CitationError::Answer("invalid reference ID"));
            }
            if citation
                .quote
                .as_ref()
                .is_some_and(|q| q.len() > limits.quote_bytes)
            {
                return Err(CitationError::Budget("quote bytes"));
            }
        }
        Ok(())
    }
    pub fn answer(&self) -> Option<&str> {
        self.answer.as_deref()
    }
    pub fn citations(&self) -> &[Citation] {
        &self.citations
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum CitationStatus {
    ReferenceOnly,
    QuoteVerified,
    UnknownReference,
    MissingQuote,
    EmptyQuote,
    QuoteNotFound,
    AmbiguousQuote,
    InvalidSpan,
    QuoteMismatch,
    DuplicateReference,
}
#[derive(Debug, Serialize)]
pub struct CitationCheck {
    id: String,
    status: CitationStatus,
    span: Option<ExcerptSpan>,
}
impl CitationCheck {
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn status(&self) -> CitationStatus {
        self.status
    }
    pub fn span(&self) -> Option<ExcerptSpan> {
        self.span
    }
}
#[derive(Debug, Serialize)]
pub struct CitationReport {
    abstained: bool,
    references_valid: bool,
    quotes_verified: bool,
    checks: Vec<CitationCheck>,
}
impl CitationReport {
    pub fn abstained(&self) -> bool {
        self.abstained
    }
    pub fn references_valid(&self) -> bool {
        self.references_valid
    }
    pub fn quotes_verified(&self) -> bool {
        self.quotes_verified
    }
    pub fn checks(&self) -> &[CitationCheck] {
        &self.checks
    }
}

pub struct CitationRegistry<'a> {
    response: &'a RetrievalResponse,
}
impl<'a> CitationRegistry<'a> {
    pub fn new(response: &'a RetrievalResponse) -> Self {
        Self { response }
    }
    pub fn check(
        &self,
        answer: &CitedAnswer,
        limits: &CitationLimits,
    ) -> Result<CitationReport, CitationError> {
        answer.validate(limits)?;
        let mut seen = BTreeSet::new();
        let checks: Vec<_> = answer
            .citations
            .iter()
            .map(|citation| {
                let key = (
                    &citation.id,
                    &citation.quote,
                    citation.span.map(|s| (s.start, s.end)),
                );
                let (status, span) = if !seen.insert(key) {
                    (CitationStatus::DuplicateReference, None)
                } else {
                    self.check_one(citation)
                };
                CitationCheck {
                    id: citation.id.clone(),
                    status,
                    span,
                }
            })
            .collect();
        Ok(CitationReport {
            abstained: answer.answer.is_none(),
            references_valid: checks.iter().all(|c| {
                matches!(
                    c.status,
                    CitationStatus::ReferenceOnly | CitationStatus::QuoteVerified
                )
            }),
            quotes_verified: !checks.is_empty()
                && checks
                    .iter()
                    .all(|c| c.status == CitationStatus::QuoteVerified),
            checks,
        })
    }
    fn check_one(&self, citation: &Citation) -> (CitationStatus, Option<ExcerptSpan>) {
        let Some(hit) = self.response.hits().iter().find(|h| h.id == citation.id) else {
            return (CitationStatus::UnknownReference, None);
        };
        let Some(quote) = &citation.quote else {
            return (
                if citation.span.is_some() {
                    CitationStatus::MissingQuote
                } else {
                    CitationStatus::ReferenceOnly
                },
                None,
            );
        };
        if quote.trim().is_empty() {
            return (CitationStatus::EmptyQuote, None);
        }
        if let Some(span) = citation.span {
            let Some(start) = span.start.checked_sub(hit.span.start) else {
                return (CitationStatus::InvalidSpan, None);
            };
            let Some(end) = span.end.checked_sub(hit.span.start) else {
                return (CitationStatus::InvalidSpan, None);
            };
            let Some(slice) = hit.excerpt.get(start..end) else {
                return (CitationStatus::InvalidSpan, None);
            };
            if slice != quote {
                return (CitationStatus::QuoteMismatch, None);
            }
            return (CitationStatus::QuoteVerified, Some(span));
        }
        let Some(start) = hit.excerpt.find(quote) else {
            return (CitationStatus::QuoteNotFound, None);
        };
        let next = start
            + hit.excerpt[start..]
                .chars()
                .next()
                .expect("nonempty matched quote")
                .len_utf8();
        if hit.excerpt[next..].contains(quote) {
            return (CitationStatus::AmbiguousQuote, None);
        }
        (
            CitationStatus::QuoteVerified,
            Some(ExcerptSpan {
                start: hit.span.start + start,
                end: hit.span.start + start + quote.len(),
            }),
        )
    }
}
