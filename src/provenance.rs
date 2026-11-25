#![doc = include_str!("../docs/PROVENANCE.md")]

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ProvenanceError {
    #[error("{0} must be nonempty and contain no control characters")]
    InvalidIdentity(&'static str),
    #[error("page numbers start at one")]
    InvalidPage,
    #[error("a source span must end at or after its start")]
    InvalidSpan,
    #[error("code lines and UTF-8 byte columns start at one")]
    InvalidCodePosition,
    #[error("source position is outside the supplied text or splits a UTF-8 character")]
    SourceBounds,
    #[error("a code span requires a file path")]
    MissingFile,
    #[error("conflicting definitions for source reference: {0}")]
    Conflict(String),
    #[error("unknown source reference: {0}")]
    UnknownReference(String),
    #[error("duplicate {kind}: {id}")]
    DuplicateId { kind: &'static str, id: String },
    #[error("unsupported provenance format version: {0}")]
    UnsupportedVersion(u32),
    #[error("provenance JSON exceeds the configured byte limit")]
    InputTooLarge,
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

fn identity(field: &'static str, value: &str) -> Result<(), ProvenanceError> {
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        return Err(ProvenanceError::InvalidIdentity(field));
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "TextSpanWire")]
pub struct TextSpan {
    start_byte: usize,
    end_byte: usize,
}
impl TextSpan {
    pub fn new(start_byte: usize, end_byte: usize) -> Result<Self, ProvenanceError> {
        if end_byte < start_byte {
            return Err(ProvenanceError::InvalidSpan);
        }
        Ok(Self {
            start_byte,
            end_byte,
        })
    }
    pub fn start_byte(self) -> usize {
        self.start_byte
    }
    pub fn end_byte(self) -> usize {
        self.end_byte
    }
    pub fn validate_in(self, text: &str) -> Result<(), ProvenanceError> {
        if self.end_byte > text.len()
            || !text.is_char_boundary(self.start_byte)
            || !text.is_char_boundary(self.end_byte)
        {
            return Err(ProvenanceError::SourceBounds);
        }
        Ok(())
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TextSpanWire {
    start_byte: usize,
    end_byte: usize,
}
impl TryFrom<TextSpanWire> for TextSpan {
    type Error = ProvenanceError;
    fn try_from(value: TextSpanWire) -> Result<Self, Self::Error> {
        Self::new(value.start_byte, value.end_byte)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "CodePositionWire")]
pub struct CodePosition {
    line: usize,
    byte_column: usize,
}
impl CodePosition {
    pub fn new(line: usize, byte_column: usize) -> Result<Self, ProvenanceError> {
        if line == 0 || byte_column == 0 {
            return Err(ProvenanceError::InvalidCodePosition);
        }
        Ok(Self { line, byte_column })
    }
    pub fn line(self) -> usize {
        self.line
    }
    pub fn byte_column(self) -> usize {
        self.byte_column
    }
    pub fn validate_in(self, text: &str) -> Result<(), ProvenanceError> {
        let line = text
            .split('\n')
            .nth(self.line - 1)
            .ok_or(ProvenanceError::SourceBounds)?;
        let line = line.strip_suffix('\r').unwrap_or(line);
        let offset = self.byte_column - 1;
        if offset > line.len() || !line.is_char_boundary(offset) {
            return Err(ProvenanceError::SourceBounds);
        }
        Ok(())
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CodePositionWire {
    line: usize,
    byte_column: usize,
}
impl TryFrom<CodePositionWire> for CodePosition {
    type Error = ProvenanceError;
    fn try_from(value: CodePositionWire) -> Result<Self, Self::Error> {
        Self::new(value.line, value.byte_column)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "CodeSpanWire")]
pub struct CodeSpan {
    start: CodePosition,
    end: CodePosition,
}
impl CodeSpan {
    pub fn new(start: CodePosition, end: CodePosition) -> Result<Self, ProvenanceError> {
        if end < start {
            return Err(ProvenanceError::InvalidSpan);
        }
        Ok(Self { start, end })
    }
    pub fn start(self) -> CodePosition {
        self.start
    }
    pub fn end(self) -> CodePosition {
        self.end
    }
    pub fn validate_in(self, text: &str) -> Result<(), ProvenanceError> {
        self.start.validate_in(text)?;
        self.end.validate_in(text)
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CodeSpanWire {
    start: CodePosition,
    end: CodePosition,
}
impl TryFrom<CodeSpanWire> for CodeSpan {
    type Error = ProvenanceError;
    fn try_from(value: CodeSpanWire) -> Result<Self, Self::Error> {
        Self::new(value.start, value.end)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "SourceReferenceWire")]
pub struct SourceReference {
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    document_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text_span: Option<TextSpan>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    code_span: Option<CodeSpan>,
    #[serde(skip_serializing_if = "Option::is_none")]
    revision: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<serde_json::Value>,
}

impl SourceReference {
    pub fn new(id: impl Into<String>) -> Result<Self, ProvenanceError> {
        let value = Self {
            id: id.into(),
            document_id: None,
            page: None,
            text_span: None,
            file_path: None,
            code_span: None,
            revision: None,
            uri: None,
            metadata: None,
        };
        value.validate()?;
        Ok(value)
    }
    fn validate(&self) -> Result<(), ProvenanceError> {
        identity("reference ID", &self.id)?;
        for (field, value) in [
            ("document ID", &self.document_id),
            ("file path", &self.file_path),
            ("revision", &self.revision),
            ("URI", &self.uri),
        ] {
            if let Some(value) = value {
                identity(field, value)?;
            }
        }
        if self.page == Some(0) {
            return Err(ProvenanceError::InvalidPage);
        }
        if self.code_span.is_some() && self.file_path.is_none() {
            return Err(ProvenanceError::MissingFile);
        }
        Ok(())
    }
    pub fn with_document(mut self, document: impl Into<String>) -> Result<Self, ProvenanceError> {
        let document = document.into();
        identity("document ID", &document)?;
        self.document_id = Some(document);
        Ok(self)
    }
    pub fn with_page(mut self, page: usize) -> Result<Self, ProvenanceError> {
        if page == 0 {
            return Err(ProvenanceError::InvalidPage);
        }
        self.page = Some(page);
        Ok(self)
    }
    pub fn with_text_span(mut self, span: TextSpan) -> Self {
        self.text_span = Some(span);
        self
    }
    pub fn with_file(mut self, path: impl Into<String>) -> Result<Self, ProvenanceError> {
        let path = path.into();
        identity("file path", &path)?;
        self.file_path = Some(path);
        Ok(self)
    }
    pub fn with_code(
        mut self,
        path: impl Into<String>,
        span: CodeSpan,
    ) -> Result<Self, ProvenanceError> {
        self = self.with_file(path)?;
        self.code_span = Some(span);
        Ok(self)
    }
    pub fn with_revision(mut self, revision: impl Into<String>) -> Result<Self, ProvenanceError> {
        let revision = revision.into();
        identity("revision", &revision)?;
        self.revision = Some(revision);
        Ok(self)
    }
    pub fn with_uri(mut self, uri: impl Into<String>) -> Result<Self, ProvenanceError> {
        let uri = uri.into();
        identity("URI", &uri)?;
        self.uri = Some(uri);
        Ok(self)
    }
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn document_id(&self) -> Option<&str> {
        self.document_id.as_deref()
    }
    pub fn page(&self) -> Option<usize> {
        self.page
    }
    pub fn text_span(&self) -> Option<TextSpan> {
        self.text_span
    }
    pub fn file_path(&self) -> Option<&str> {
        self.file_path.as_deref()
    }
    pub fn code_span(&self) -> Option<CodeSpan> {
        self.code_span
    }
    pub fn revision(&self) -> Option<&str> {
        self.revision.as_deref()
    }
    pub fn uri(&self) -> Option<&str> {
        self.uri.as_deref()
    }
    pub fn metadata(&self) -> Option<&serde_json::Value> {
        self.metadata.as_ref()
    }
    pub fn validate_source_text(&self, text: &str) -> Result<(), ProvenanceError> {
        if let Some(span) = self.text_span {
            span.validate_in(text)?;
        }
        if let Some(span) = self.code_span {
            span.validate_in(text)?;
        }
        Ok(())
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceReferenceWire {
    id: String,
    document_id: Option<String>,
    page: Option<usize>,
    text_span: Option<TextSpan>,
    file_path: Option<String>,
    code_span: Option<CodeSpan>,
    revision: Option<String>,
    uri: Option<String>,
    metadata: Option<serde_json::Value>,
}
impl TryFrom<SourceReferenceWire> for SourceReference {
    type Error = ProvenanceError;
    fn try_from(value: SourceReferenceWire) -> Result<Self, Self::Error> {
        let result = Self {
            id: value.id,
            document_id: value.document_id,
            page: value.page,
            text_span: value.text_span,
            file_path: value.file_path,
            code_span: value.code_span,
            revision: value.revision,
            uri: value.uri,
            metadata: value.metadata,
        };
        result.validate()?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub enum ProvenanceStatus {
    Missing,
    Referenced,
    Omitted,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ProvenanceStore {
    sources: BTreeMap<String, SourceReference>,
    entries: BTreeMap<String, BTreeSet<String>>,
}

impl ProvenanceStore {
    pub const FORMAT_VERSION: u32 = 1;
    pub const DEFAULT_JSON_LIMIT: usize = 16 * 1024 * 1024;
    pub fn new() -> Self {
        Self::default()
    }
    pub fn source_count(&self) -> usize {
        self.sources.len()
    }
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn source(&self, id: &str) -> Option<&SourceReference> {
        self.sources.get(id)
    }
    pub fn sources(&self) -> impl Iterator<Item = &SourceReference> {
        self.sources.values()
    }
    pub fn sources_for(&self, entry: &str) -> impl Iterator<Item = &SourceReference> {
        self.entries
            .get(entry)
            .into_iter()
            .flat_map(|ids| ids.iter())
            .map(|id| self.sources.get(id).expect("source binding is validated"))
    }
    pub fn status(&self, entry: &str) -> ProvenanceStatus {
        if self.entries.contains_key(entry) {
            ProvenanceStatus::Referenced
        } else {
            ProvenanceStatus::Missing
        }
    }
    fn compatible(&self, reference: &SourceReference) -> Result<(), ProvenanceError> {
        reference.validate()?;
        if self
            .sources
            .get(reference.id())
            .is_some_and(|old| old != reference)
        {
            return Err(ProvenanceError::Conflict(reference.id.clone()));
        }
        Ok(())
    }
    pub fn insert_source(&mut self, reference: SourceReference) -> Result<bool, ProvenanceError> {
        self.compatible(&reference)?;
        if self.sources.contains_key(reference.id()) {
            return Ok(false);
        }
        self.sources.insert(reference.id.clone(), reference);
        Ok(true)
    }
    pub fn bind(
        &mut self,
        entry: impl Into<String>,
        reference_id: &str,
    ) -> Result<bool, ProvenanceError> {
        let entry = entry.into();
        identity("entry ID", &entry)?;
        identity("reference ID", reference_id)?;
        if !self.sources.contains_key(reference_id) {
            return Err(ProvenanceError::UnknownReference(reference_id.to_owned()));
        }
        Ok(self
            .entries
            .entry(entry)
            .or_default()
            .insert(reference_id.to_owned()))
    }
    pub fn add(
        &mut self,
        entry: impl Into<String>,
        reference: SourceReference,
    ) -> Result<bool, ProvenanceError> {
        let entry = entry.into();
        identity("entry ID", &entry)?;
        self.compatible(&reference)?;
        let id = reference.id.clone();
        self.sources.entry(id.clone()).or_insert(reference);
        Ok(self.entries.entry(entry).or_default().insert(id))
    }
    pub fn replace(
        &mut self,
        entry: impl Into<String>,
        references: impl IntoIterator<Item = SourceReference>,
    ) -> Result<usize, ProvenanceError> {
        let entry = entry.into();
        identity("entry ID", &entry)?;
        let mut staged = BTreeMap::new();
        for reference in references {
            self.compatible(&reference)?;
            if staged
                .get(reference.id())
                .is_some_and(|old| old != &reference)
            {
                return Err(ProvenanceError::Conflict(reference.id.clone()));
            }
            staged.insert(reference.id.clone(), reference);
        }
        let ids: BTreeSet<_> = staged.keys().cloned().collect();
        let count = ids.len();
        for (id, reference) in staged {
            self.sources.entry(id).or_insert(reference);
        }
        if ids.is_empty() {
            self.entries.remove(&entry);
        } else {
            self.entries.insert(entry, ids);
        }
        Ok(count)
    }
    pub fn merge_entries(
        &mut self,
        target: impl Into<String>,
        entries: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<usize, ProvenanceError> {
        let target = target.into();
        identity("entry ID", &target)?;
        let mut ids = self.entries.get(&target).cloned().unwrap_or_default();
        let old = ids.len();
        for entry in entries {
            identity("entry ID", entry.as_ref())?;
            if let Some(sources) = self.entries.get(entry.as_ref()) {
                ids.extend(sources.iter().cloned());
            }
        }
        let added = ids.len() - old;
        if !ids.is_empty() {
            self.entries.insert(target, ids);
        }
        Ok(added)
    }
    pub fn merge(&mut self, other: &Self) -> Result<(), ProvenanceError> {
        for reference in other.sources.values() {
            self.compatible(reference)?;
        }
        for (id, reference) in &other.sources {
            self.sources
                .entry(id.clone())
                .or_insert_with(|| reference.clone());
        }
        for (entry, ids) in &other.entries {
            self.entries
                .entry(entry.clone())
                .or_default()
                .extend(ids.iter().cloned());
        }
        Ok(())
    }
    pub fn remove_entry(&mut self, entry: &str) -> bool {
        self.entries.remove(entry).is_some()
    }
    pub fn remove_source(&mut self, id: &str) -> bool {
        if self.sources.remove(id).is_none() {
            return false;
        }
        self.entries.retain(|_, ids| {
            ids.remove(id);
            !ids.is_empty()
        });
        true
    }
    pub fn prune_unreferenced(&mut self) -> usize {
        let used: BTreeSet<_> = self.entries.values().flat_map(|ids| ids.iter()).collect();
        let old = self.sources.len();
        self.sources.retain(|id, _| used.contains(id));
        old - self.sources.len()
    }
    pub fn clear(&mut self) {
        *self = Self::new();
    }
    pub fn to_json(&self) -> Result<String, ProvenanceError> {
        Ok(serde_json::to_string(self)?)
    }
    pub fn from_json(json: &str) -> Result<Self, ProvenanceError> {
        Self::from_json_with_limit(json, Self::DEFAULT_JSON_LIMIT)
    }
    pub fn from_json_with_limit(json: &str, max_bytes: usize) -> Result<Self, ProvenanceError> {
        if json.len() > max_bytes {
            return Err(ProvenanceError::InputTooLarge);
        }
        Self::try_from(serde_json::from_str::<StoreWire>(json).map_err(crate::input::json_error)?)
    }
}

#[derive(Serialize)]
struct EntryExport<'a> {
    entry_id: &'a str,
    source_ids: &'a BTreeSet<String>,
}
struct SourcesExport<'a>(&'a BTreeMap<String, SourceReference>);
impl Serialize for SourcesExport<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.0.values())
    }
}
struct EntriesExport<'a>(&'a BTreeMap<String, BTreeSet<String>>);
impl Serialize for EntriesExport<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.0.iter().map(|(id, sources)| EntryExport {
            entry_id: id,
            source_ids: sources,
        }))
    }
}
#[derive(Serialize)]
struct StoreExport<'a> {
    version: u32,
    sources: SourcesExport<'a>,
    entries: EntriesExport<'a>,
}
impl Serialize for ProvenanceStore {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        StoreExport {
            version: Self::FORMAT_VERSION,
            sources: SourcesExport(&self.sources),
            entries: EntriesExport(&self.entries),
        }
        .serialize(serializer)
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EntryWire {
    entry_id: String,
    source_ids: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StoreWire {
    version: u32,
    sources: Vec<SourceReference>,
    entries: Vec<EntryWire>,
}
impl TryFrom<StoreWire> for ProvenanceStore {
    type Error = ProvenanceError;
    fn try_from(wire: StoreWire) -> Result<Self, Self::Error> {
        if wire.version != Self::FORMAT_VERSION {
            return Err(ProvenanceError::UnsupportedVersion(wire.version));
        }
        let mut store = Self::new();
        for reference in wire.sources {
            let id = reference.id.clone();
            if store.sources.insert(id.clone(), reference).is_some() {
                return Err(ProvenanceError::DuplicateId {
                    kind: "reference ID",
                    id,
                });
            }
        }
        let mut seen = BTreeSet::new();
        for entry in wire.entries {
            identity("entry ID", &entry.entry_id)?;
            if !seen.insert(entry.entry_id.clone()) {
                return Err(ProvenanceError::DuplicateId {
                    kind: "entry ID",
                    id: entry.entry_id,
                });
            }
            let mut ids = BTreeSet::new();
            for id in entry.source_ids {
                if !store.sources.contains_key(&id) {
                    return Err(ProvenanceError::UnknownReference(id));
                }
                if !ids.insert(id.clone()) {
                    return Err(ProvenanceError::DuplicateId {
                        kind: "source binding",
                        id,
                    });
                }
            }
            if !ids.is_empty() {
                store.entries.insert(entry.entry_id, ids);
            }
        }
        Ok(store)
    }
}
impl<'de> Deserialize<'de> for ProvenanceStore {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::try_from(StoreWire::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}
