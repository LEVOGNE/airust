#![doc = include_str!("../docs/DOCUMENTS.md")]

use crate::{
    provenance::{ProvenanceError, SourceReference, TextSpan},
    KnowledgeEntry,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    borrow::Cow,
    fmt,
    io::{self, Read, Write},
    sync::Arc,
};
use thiserror::Error;

pub const DOCUMENT_FORMAT_VERSION: u32 = 1;
const FORMAT: u32 = DOCUMENT_FORMAT_VERSION;
const SEPARATOR: &str = "\n\n";

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DocumentError {
    #[error("document limit exceeded: {0}")]
    Limit(&'static str),
    #[error("document limits must be positive")]
    Configuration,
    #[error("{0} must be nonempty and contain no control characters")]
    Identity(&'static str),
    #[error("invalid document integrity: {0}")]
    Integrity(&'static str),
    #[error("unsupported document format version: {0}")]
    UnsupportedVersion(u32),
    #[error("passage belongs to another document or text revision")]
    StalePassage,
    #[error("source bytes do not match this snapshot")]
    SourceMismatch,
    #[error("range is outside the identified text or splits a UTF-8 character")]
    Bounds,
    #[error("page numbers must be positive, strictly increasing and within the page limit")]
    PageOrder,
    #[error("unknown physical page: {0}")]
    UnknownPage(usize),
    #[error("a searchable passage must contain non-whitespace text")]
    EmptyPassage,
    #[error("entry ID or text no longer agrees with the passage")]
    EntryMismatch,
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Provenance(#[from] ProvenanceError),
}

#[derive(Debug, Clone)]
pub struct DocumentLimits {
    pub max_source_bytes: u64,
    pub max_text_bytes: usize,
    pub max_pages: usize,
    pub max_id_bytes: usize,
    pub max_recipe_bytes: usize,
    pub max_json_bytes: usize,
}
impl Default for DocumentLimits {
    fn default() -> Self {
        Self {
            max_source_bytes: 128 * 1024 * 1024,
            max_text_bytes: 256 * 1024 * 1024,
            max_pages: 50_000,
            max_id_bytes: 1024,
            max_recipe_bytes: 16 * 1024,
            max_json_bytes: 512 * 1024 * 1024,
        }
    }
}
impl DocumentLimits {
    fn validate(&self) -> Result<(), DocumentError> {
        if self.max_source_bytes == 0
            || self.max_text_bytes == 0
            || self.max_pages == 0
            || self.max_id_bytes == 0
            || self.max_recipe_bytes == 0
            || self.max_json_bytes == 0
        {
            return Err(DocumentError::Configuration);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceFingerprint {
    sha256: String,
    byte_len: u64,
}
impl SourceFingerprint {
    pub fn from_bytes(bytes: &[u8], limits: &DocumentLimits) -> Result<Self, DocumentError> {
        limits.validate()?;
        let byte_len =
            u64::try_from(bytes.len()).map_err(|_| DocumentError::Limit("source bytes"))?;
        if byte_len > limits.max_source_bytes {
            return Err(DocumentError::Limit("source bytes"));
        }
        Ok(Self {
            sha256: hash(bytes),
            byte_len,
        })
    }
    pub fn from_reader(
        mut reader: impl Read,
        limits: &DocumentLimits,
    ) -> Result<Self, DocumentError> {
        limits.validate()?;
        let mut buffer = [0u8; 65536];
        let mut digest = Sha256::new();
        let mut total = 0u64;
        loop {
            let length =
                (limits.max_source_bytes - total).min((buffer.len() - 1) as u64) as usize + 1;
            let count = match reader.read(&mut buffer[..length]) {
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                other => other?,
            };
            if count == 0 {
                break;
            }
            total = total
                .checked_add(count as u64)
                .filter(|&n| n <= limits.max_source_bytes)
                .ok_or(DocumentError::Limit("source bytes"))?;
            digest.update(&buffer[..count]);
        }
        Ok(Self {
            sha256: format!("{:x}", digest.finalize()),
            byte_len: total,
        })
    }
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
    pub fn byte_len(&self) -> u64 {
        self.byte_len
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum TextSpace {
    Supplied,
    Extracted,
    Cleaned,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PageSpan {
    page_number: usize,
    span: TextSpan,
}
impl PageSpan {
    pub fn page_number(&self) -> usize {
        self.page_number
    }
    pub fn span(&self) -> TextSpan {
        self.span
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Identity {
    document_id: String,
    revision: String,
    text_space: TextSpace,
}
struct SnapshotData {
    identity: Arc<Identity>,
    source: SourceFingerprint,
    recipe: String,
    text_sha256: String,
    text: String,
    pages: Vec<PageSpan>,
}

#[derive(Clone)]
pub struct DocumentSnapshot(Arc<SnapshotData>);
impl fmt::Debug for DocumentSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DocumentSnapshot")
            .field("document_id", &self.document_id())
            .field("revision", &self.revision())
            .field("text_bytes", &self.text().len())
            .field("pages", &self.pages().len())
            .finish()
    }
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn frame(digest: &mut Sha256, bytes: &[u8]) {
    digest.update((bytes.len() as u64).to_le_bytes());
    digest.update(bytes);
}
fn identity(field: &'static str, value: &str, maximum: usize) -> Result<(), DocumentError> {
    if value.len() > maximum {
        return Err(DocumentError::Limit(field));
    }
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        return Err(DocumentError::Identity(field));
    }
    Ok(())
}
fn space_tag(space: TextSpace) -> &'static [u8] {
    match space {
        TextSpace::Supplied => b"supplied",
        TextSpace::Extracted => b"extracted",
        TextSpace::Cleaned => b"cleaned",
    }
}

impl DocumentSnapshot {
    pub fn new(
        document_id: &str,
        source: SourceFingerprint,
        recipe: &str,
        text_space: TextSpace,
        text: String,
        limits: &DocumentLimits,
    ) -> Result<Self, DocumentError> {
        Self::build(
            document_id,
            source,
            recipe,
            text_space,
            text,
            Vec::new(),
            limits,
        )
    }
    pub fn from_pages(
        document_id: &str,
        source: SourceFingerprint,
        recipe: &str,
        text_space: TextSpace,
        pages: impl IntoIterator<Item = (usize, String)>,
        limits: &DocumentLimits,
    ) -> Result<Self, DocumentError> {
        limits.validate()?;
        identity("document ID", document_id, limits.max_id_bytes)?;
        identity("processing recipe", recipe, limits.max_recipe_bytes)?;
        if source.byte_len > limits.max_source_bytes {
            return Err(DocumentError::Limit("source bytes"));
        }
        let mut text = String::new();
        let mut spans: Vec<PageSpan> = Vec::new();
        for (number, page) in pages {
            if spans.len() >= limits.max_pages {
                return Err(DocumentError::Limit("page count"));
            }
            if number == 0
                || number > limits.max_pages
                || spans.last().is_some_and(|last| last.page_number >= number)
            {
                return Err(DocumentError::PageOrder);
            }
            let separator = if spans.is_empty() { "" } else { SEPARATOR };
            text.len()
                .checked_add(separator.len())
                .and_then(|n| n.checked_add(page.len()))
                .filter(|&n| n <= limits.max_text_bytes)
                .ok_or(DocumentError::Limit("text bytes"))?;
            text.push_str(separator);
            let start = text.len();
            text.push_str(&page);
            spans.push(PageSpan {
                page_number: number,
                span: TextSpan::new(start, text.len())?,
            });
        }
        Self::build(document_id, source, recipe, text_space, text, spans, limits)
    }
    fn build(
        document_id: &str,
        source: SourceFingerprint,
        recipe: &str,
        text_space: TextSpace,
        text: String,
        pages: Vec<PageSpan>,
        limits: &DocumentLimits,
    ) -> Result<Self, DocumentError> {
        limits.validate()?;
        identity("document ID", document_id, limits.max_id_bytes)?;
        identity("processing recipe", recipe, limits.max_recipe_bytes)?;
        if source.byte_len > limits.max_source_bytes {
            return Err(DocumentError::Limit("source bytes"));
        }
        if !crate::text::Identifier::sha256(&source.sha256) {
            return Err(DocumentError::Integrity("source SHA-256"));
        }
        if text.len() > limits.max_text_bytes {
            return Err(DocumentError::Limit("text bytes"));
        }
        if pages.len() > limits.max_pages {
            return Err(DocumentError::Limit("page count"));
        }
        let mut previous_end = 0;
        let mut previous_page = 0;
        for (i, page) in pages.iter().enumerate() {
            if page.page_number <= previous_page || page.page_number > limits.max_pages {
                return Err(DocumentError::PageOrder);
            }
            page.span
                .validate_in(&text)
                .map_err(|_| DocumentError::Bounds)?;
            let expected = previous_end + if i == 0 { 0 } else { SEPARATOR.len() };
            if page.span.start_byte() != expected
                || (i != 0 && text.get(previous_end..expected) != Some(SEPARATOR))
            {
                return Err(DocumentError::Integrity("page partition"));
            }
            previous_end = page.span.end_byte();
            previous_page = page.page_number;
        }
        if !pages.is_empty() && previous_end != text.len() {
            return Err(DocumentError::Integrity("page partition"));
        }
        let text_sha256 = hash(text.as_bytes());
        let mut digest = Sha256::new();
        for bytes in [
            b"airust/document/v1".as_slice(),
            document_id.as_bytes(),
            source.sha256.as_bytes(),
            &source.byte_len.to_le_bytes(),
            recipe.as_bytes(),
            space_tag(text_space),
            text_sha256.as_bytes(),
            &(pages.len() as u64).to_le_bytes(),
        ] {
            frame(&mut digest, bytes);
        }
        for page in &pages {
            for value in [
                page.page_number,
                page.span.start_byte(),
                page.span.end_byte(),
            ] {
                frame(&mut digest, &(value as u64).to_le_bytes());
            }
        }
        Ok(Self(Arc::new(SnapshotData {
            identity: Arc::new(Identity {
                document_id: document_id.into(),
                revision: format!("{:x}", digest.finalize()),
                text_space,
            }),
            source,
            recipe: recipe.into(),
            text_sha256,
            text,
            pages,
        })))
    }
    pub fn document_id(&self) -> &str {
        &self.0.identity.document_id
    }
    pub fn revision(&self) -> &str {
        &self.0.identity.revision
    }
    pub fn text_space(&self) -> TextSpace {
        self.0.identity.text_space
    }
    pub fn source(&self) -> &SourceFingerprint {
        &self.0.source
    }
    pub fn recipe(&self) -> &str {
        &self.0.recipe
    }
    pub fn text_sha256(&self) -> &str {
        &self.0.text_sha256
    }
    pub fn text(&self) -> &str {
        &self.0.text
    }
    pub fn pages(&self) -> &[PageSpan] {
        &self.0.pages
    }
    pub fn verify_source(&self, bytes: &[u8]) -> Result<(), DocumentError> {
        if u64::try_from(bytes.len()).ok() != Some(self.source().byte_len)
            || hash(bytes) != self.source().sha256
        {
            return Err(DocumentError::SourceMismatch);
        }
        Ok(())
    }
    fn page_for(&self, span: TextSpan) -> Option<usize> {
        let index = self
            .pages()
            .partition_point(|p| p.span.start_byte() <= span.start_byte())
            .checked_sub(1)?;
        let page = &self.pages()[index];
        (span.end_byte() <= page.span.end_byte()).then_some(page.page_number)
    }
    pub fn passage(&self, start_byte: usize, end_byte: usize) -> Result<Passage, DocumentError> {
        let span = TextSpan::new(start_byte, end_byte).map_err(|_| DocumentError::Bounds)?;
        span.validate_in(self.text())
            .map_err(|_| DocumentError::Bounds)?;
        Ok(Passage {
            identity: self.0.identity.clone(),
            span,
            page_number: self.page_for(span),
        })
    }
    pub fn page_passage(&self, page_number: usize) -> Result<Passage, DocumentError> {
        let index = self
            .pages()
            .binary_search_by_key(&page_number, PageSpan::page_number)
            .map_err(|_| DocumentError::UnknownPage(page_number))?;
        let span = self.pages()[index].span;
        self.passage(span.start_byte(), span.end_byte())
    }
    fn wire(&self) -> SnapshotRef<'_> {
        SnapshotRef {
            format_version: FORMAT,
            document_id: self.document_id(),
            source: self.source(),
            recipe: self.recipe(),
            text_space: self.text_space(),
            text_sha256: self.text_sha256(),
            revision: self.revision(),
            text: self.text(),
            pages: self.pages(),
        }
    }
    pub fn to_json(&self, max_bytes: usize) -> Result<String, DocumentError> {
        encode(&self.wire(), max_bytes)
    }
    pub fn from_json(input: &str, limits: &DocumentLimits) -> Result<Self, DocumentError> {
        limits.validate()?;
        if input.len() > limits.max_json_bytes {
            return Err(DocumentError::Limit("encoded JSON bytes"));
        }
        let wire: SnapshotWire<'_> =
            serde_json::from_str(input).map_err(crate::input::json_error)?;
        if wire.format_version != FORMAT {
            return Err(DocumentError::UnsupportedVersion(wire.format_version));
        }
        identity("document ID", &wire.document_id, limits.max_id_bytes)?;
        identity("processing recipe", &wire.recipe, limits.max_recipe_bytes)?;
        if wire.text.len() > limits.max_text_bytes {
            return Err(DocumentError::Limit("text bytes"));
        }
        if wire.pages.len() > limits.max_pages {
            return Err(DocumentError::Limit("page count"));
        }
        if wire.source.byte_len > limits.max_source_bytes {
            return Err(DocumentError::Limit("source bytes"));
        }
        if !crate::text::Identifier::sha256(&wire.source.sha256) {
            return Err(DocumentError::Integrity("source SHA-256"));
        }
        let result = Self::build(
            &wire.document_id,
            SourceFingerprint {
                sha256: wire.source.sha256.into_owned(),
                byte_len: wire.source.byte_len,
            },
            &wire.recipe,
            wire.text_space,
            wire.text.into_owned(),
            wire.pages
                .into_iter()
                .map(|p| PageSpan {
                    page_number: p.page_number,
                    span: p.span,
                })
                .collect(),
            limits,
        )?;
        if wire.text_sha256 != result.text_sha256() {
            return Err(DocumentError::Integrity("text SHA-256"));
        }
        if wire.revision != result.revision() {
            return Err(DocumentError::Integrity("revision"));
        }
        Ok(result)
    }
    pub fn passage_from_json(
        &self,
        input: &str,
        limits: &DocumentLimits,
    ) -> Result<Passage, DocumentError> {
        limits.validate()?;
        if input.len() > limits.max_json_bytes {
            return Err(DocumentError::Limit("encoded JSON bytes"));
        }
        let wire: PassageWire<'_> =
            serde_json::from_str(input).map_err(crate::input::json_error)?;
        if wire.format_version != FORMAT {
            return Err(DocumentError::UnsupportedVersion(wire.format_version));
        }
        identity("document ID", &wire.document_id, limits.max_id_bytes)?;
        if wire.document_id != self.document_id()
            || wire.revision != self.revision()
            || wire.text_space != self.text_space()
        {
            return Err(DocumentError::StalePassage);
        }
        let passage = self.passage(wire.span.start_byte(), wire.span.end_byte())?;
        if passage.page_number != wire.page_number {
            return Err(DocumentError::Integrity("passage page"));
        }
        Ok(passage)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Passage {
    identity: Arc<Identity>,
    span: TextSpan,
    page_number: Option<usize>,
}
impl Passage {
    pub fn document_id(&self) -> &str {
        &self.identity.document_id
    }
    pub fn revision(&self) -> &str {
        &self.identity.revision
    }
    pub fn text_space(&self) -> TextSpace {
        self.identity.text_space
    }
    pub fn span(&self) -> TextSpan {
        self.span
    }
    pub fn page_number(&self) -> Option<usize> {
        self.page_number
    }
    pub fn id(&self) -> String {
        let mut digest = Sha256::new();
        for bytes in [
            b"airust/passage/v1".as_slice(),
            self.document_id().as_bytes(),
            self.revision().as_bytes(),
            &(self.span.start_byte() as u64).to_le_bytes(),
            &(self.span.end_byte() as u64).to_le_bytes(),
        ] {
            frame(&mut digest, bytes);
        }
        format!("airust-passage-v1:{:x}", digest.finalize())
    }
    pub fn resolve<'a>(&self, snapshot: &'a DocumentSnapshot) -> Result<&'a str, DocumentError> {
        if self.identity.as_ref() != snapshot.0.identity.as_ref() {
            return Err(DocumentError::StalePassage);
        }
        self.span
            .validate_in(snapshot.text())
            .map_err(|_| DocumentError::Bounds)?;
        if self.page_number != snapshot.page_for(self.span) {
            return Err(DocumentError::Integrity("passage page"));
        }
        snapshot
            .text()
            .get(self.span.start_byte()..self.span.end_byte())
            .ok_or(DocumentError::Bounds)
    }
    pub fn subspan(
        &self,
        snapshot: &DocumentSnapshot,
        start_byte: usize,
        end_byte: usize,
    ) -> Result<Self, DocumentError> {
        let text = self.resolve(snapshot)?;
        let relative = TextSpan::new(start_byte, end_byte).map_err(|_| DocumentError::Bounds)?;
        relative
            .validate_in(text)
            .map_err(|_| DocumentError::Bounds)?;
        let start = self
            .span
            .start_byte()
            .checked_add(start_byte)
            .ok_or(DocumentError::Bounds)?;
        let end = self
            .span
            .start_byte()
            .checked_add(end_byte)
            .ok_or(DocumentError::Bounds)?;
        snapshot.passage(start, end)
    }
    fn wire(&self) -> PassageRef<'_> {
        PassageRef {
            format_version: FORMAT,
            document_id: self.document_id(),
            revision: self.revision(),
            text_space: self.text_space(),
            span: self.span,
            page_number: self.page_number,
        }
    }
    pub fn to_json(&self, max_bytes: usize) -> Result<String, DocumentError> {
        encode(&self.wire(), max_bytes)
    }
    pub fn to_entry(&self, snapshot: &DocumentSnapshot) -> Result<KnowledgeEntry, DocumentError> {
        let text = self.resolve(snapshot)?;
        if text.trim().is_empty() {
            return Err(DocumentError::EmptyPassage);
        }
        Ok(KnowledgeEntry::new(self.id(), text)
            .with_metadata(serde_json::json!({"airust_document": self.wire()})))
    }
    pub fn validate_entry(
        &self,
        snapshot: &DocumentSnapshot,
        entry: &KnowledgeEntry,
    ) -> Result<(), DocumentError> {
        let text = self.resolve(snapshot)?;
        let expected = serde_json::to_value(self.wire())?;
        if entry.id != self.id()
            || entry.text != text
            || entry
                .metadata
                .as_ref()
                .and_then(|m| m.get("airust_document"))
                != Some(&expected)
        {
            return Err(DocumentError::EntryMismatch);
        }
        Ok(())
    }
    pub fn source_reference(&self) -> Result<SourceReference, DocumentError> {
        let mut reference = SourceReference::new(self.id())?.with_document(self.document_id())?
            .with_revision(self.revision())?.with_text_span(self.span)
            .with_metadata(serde_json::json!({"coordinate_space":"snapshot_utf8_bytes", "text_space":self.text_space(),"document_format_version":FORMAT}));
        if let Some(page) = self.page_number {
            reference = reference.with_page(page)?;
        }
        Ok(reference)
    }
}

#[derive(Serialize)]
struct SnapshotRef<'a> {
    format_version: u32,
    document_id: &'a str,
    source: &'a SourceFingerprint,
    recipe: &'a str,
    text_space: TextSpace,
    text_sha256: &'a str,
    revision: &'a str,
    text: &'a str,
    pages: &'a [PageSpan],
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PageWire {
    page_number: usize,
    span: TextSpan,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceWire<'a> {
    #[serde(borrow)]
    sha256: Cow<'a, str>,
    byte_len: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SnapshotWire<'a> {
    format_version: u32,
    #[serde(borrow)]
    document_id: Cow<'a, str>,
    #[serde(borrow)]
    source: SourceWire<'a>,
    #[serde(borrow)]
    recipe: Cow<'a, str>,
    text_space: TextSpace,
    #[serde(borrow)]
    text_sha256: Cow<'a, str>,
    #[serde(borrow)]
    revision: Cow<'a, str>,
    #[serde(borrow)]
    text: Cow<'a, str>,
    pages: Vec<PageWire>,
}
#[derive(Serialize)]
struct PassageRef<'a> {
    format_version: u32,
    document_id: &'a str,
    revision: &'a str,
    text_space: TextSpace,
    span: TextSpan,
    page_number: Option<usize>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PassageWire<'a> {
    format_version: u32,
    #[serde(borrow)]
    document_id: Cow<'a, str>,
    #[serde(borrow)]
    revision: Cow<'a, str>,
    text_space: TextSpace,
    span: TextSpan,
    page_number: Option<usize>,
}
struct JsonBuffer {
    bytes: Vec<u8>,
    maximum: usize,
    exceeded: bool,
}
impl Write for JsonBuffer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self
            .bytes
            .len()
            .checked_add(bytes.len())
            .is_none_or(|n| n > self.maximum)
        {
            self.exceeded = true;
            return Err(io::Error::other("encoded JSON byte limit exceeded"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn encode(value: &impl Serialize, maximum: usize) -> Result<String, DocumentError> {
    if maximum == 0 {
        return Err(DocumentError::Configuration);
    }
    let mut buffer = JsonBuffer {
        bytes: Vec::new(),
        maximum,
        exceeded: false,
    };
    let result = serde_json::to_writer(&mut buffer, value);
    if buffer.exceeded {
        return Err(DocumentError::Limit("encoded JSON bytes"));
    }
    result?;
    String::from_utf8(buffer.bytes).map_err(|_| DocumentError::Integrity("encoded UTF-8"))
}
