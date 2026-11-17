#![doc = include_str!("../docs/VERSIONING.md")]

use crate::{index::validate_entry, Engine, IndexError, KnowledgeEntry};
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    sync::Arc,
};
use thiserror::Error;

const FORMAT_VERSION: u32 = 1;
const DEFAULT_JSON_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum VersionError {
    #[error("revision must be a positive integer")]
    InvalidRevision,
    #[error("revision sequence is exhausted")]
    Exhausted,
    #[error("revision conflict for {id}: expected {expected:?}, current {current:?}")]
    Conflict {
        id: String,
        expected: ExpectedRevision,
        current: Option<Revision>,
    },
    #[error("entry is unknown: {0}")]
    UnknownEntry(String),
    #[error("entry is already deleted: {0}")]
    AlreadyDeleted(String),
    #[error("revision {revision:?} for {id} is unavailable or was pruned")]
    Unavailable { id: String, revision: Revision },
    #[error("invalid change metadata: {0}")]
    Metadata(&'static str),
    #[error("invalid versioned snapshot: {0}")]
    InvalidData(&'static str),
    #[error("unsupported versioned snapshot format: {0}")]
    UnsupportedVersion(u32),
    #[error("versioned snapshot exceeds its input byte limit")]
    InputLimit,
    #[error(transparent)]
    Index(#[from] IndexError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "u64", into = "u64")]
pub struct Revision(u64);
impl Revision {
    pub fn new(value: u64) -> Result<Self, VersionError> {
        Self::try_from(value)
    }
    pub fn get(self) -> u64 {
        self.0
    }
}
impl TryFrom<u64> for Revision {
    type Error = VersionError;
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value == 0 {
            Err(VersionError::InvalidRevision)
        } else {
            Ok(Self(value))
        }
    }
}
impl From<Revision> for u64 {
    fn from(value: Revision) -> Self {
        value.0
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpectedRevision {
    Absent,
    Exact(Revision),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum RevisionAction {
    Insert,
    Replace,
    Delete,
    Restore,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "MetadataWire")]
pub struct ChangeMetadata {
    actor: Option<String>,
    reason: Option<String>,
    metadata: Option<Value>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MetadataWire {
    actor: Option<String>,
    reason: Option<String>,
    metadata: Option<Value>,
}
fn label(value: impl Into<String>) -> Result<String, VersionError> {
    let value = value.into();
    if value.trim().is_empty()
        || value.chars().count() > 1024
        || value.chars().any(char::is_control)
    {
        return Err(VersionError::Metadata(
            "labels need 1..1024 characters and no controls",
        ));
    }
    Ok(value.trim().to_owned())
}
impl ChangeMetadata {
    pub fn with_actor(mut self, actor: impl Into<String>) -> Result<Self, VersionError> {
        self.actor = Some(label(actor)?);
        Ok(self)
    }
    pub fn with_reason(mut self, reason: impl Into<String>) -> Result<Self, VersionError> {
        self.reason = Some(label(reason)?);
        Ok(self)
    }
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
    pub fn actor(&self) -> Option<&str> {
        self.actor.as_deref()
    }
    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }
    pub fn metadata(&self) -> Option<&Value> {
        self.metadata.as_ref()
    }
}
impl TryFrom<MetadataWire> for ChangeMetadata {
    type Error = VersionError;
    fn try_from(w: MetadataWire) -> Result<Self, Self::Error> {
        let mut result = Self {
            metadata: w.metadata,
            ..Self::default()
        };
        if let Some(actor) = w.actor {
            result = result.with_actor(actor)?;
        }
        if let Some(reason) = w.reason {
            result = result.with_reason(reason)?;
        }
        Ok(result)
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "VersionWire")]
pub struct EntryVersion {
    revision: Revision,
    action: RevisionAction,
    change: Arc<ChangeMetadata>,
    deleted: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionWire {
    revision: Revision,
    action: RevisionAction,
    change: Arc<ChangeMetadata>,
    deleted: bool,
}
impl TryFrom<VersionWire> for EntryVersion {
    type Error = VersionError;
    fn try_from(w: VersionWire) -> Result<Self, Self::Error> {
        if matches!(w.action, RevisionAction::Delete) && !w.deleted
            || matches!(w.action, RevisionAction::Insert | RevisionAction::Replace) && w.deleted
        {
            return Err(VersionError::InvalidData(
                "revision action/deletion agreement",
            ));
        }
        Ok(Self {
            revision: w.revision,
            action: w.action,
            change: w.change,
            deleted: w.deleted,
        })
    }
}
impl EntryVersion {
    pub fn revision(&self) -> Revision {
        self.revision
    }
    pub fn action(&self) -> RevisionAction {
        self.action
    }
    pub fn change(&self) -> &ChangeMetadata {
        &self.change
    }
    pub fn is_deleted(&self) -> bool {
        self.deleted
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RecordWire")]
pub struct RevisionRecord {
    id: String,
    version: EntryVersion,
    entry: Option<KnowledgeEntry>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordWire {
    id: String,
    version: EntryVersion,
    entry: Option<KnowledgeEntry>,
}
impl TryFrom<RecordWire> for RevisionRecord {
    type Error = VersionError;
    fn try_from(w: RecordWire) -> Result<Self, Self::Error> {
        validate_state(&w.id, &w.version, w.entry.as_ref())?;
        Ok(Self {
            id: w.id,
            version: w.version,
            entry: w.entry,
        })
    }
}
impl RevisionRecord {
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn version(&self) -> &EntryVersion {
        &self.version
    }
    pub fn entry(&self) -> Option<&KnowledgeEntry> {
        self.entry.as_ref()
    }
}
#[derive(Debug, Clone, Copy)]
pub struct RevisionView<'a> {
    id: &'a str,
    version: &'a EntryVersion,
    entry: Option<&'a KnowledgeEntry>,
}
impl<'a> RevisionView<'a> {
    pub fn id(&self) -> &'a str {
        self.id
    }
    pub fn version(&self) -> &'a EntryVersion {
        self.version
    }
    pub fn entry(&self) -> Option<&'a KnowledgeEntry> {
        self.entry
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RevisionComparison {
    id: String,
    from: Revision,
    to: Revision,
    existence_changed: bool,
    text_changed: bool,
    metadata_changed: bool,
}
impl RevisionComparison {
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn from(&self) -> Revision {
        self.from
    }
    pub fn to(&self) -> Revision {
        self.to
    }
    pub fn existence_changed(&self) -> bool {
        self.existence_changed
    }
    pub fn text_changed(&self) -> bool {
        self.text_changed
    }
    pub fn metadata_changed(&self) -> bool {
        self.metadata_changed
    }
}
#[derive(Debug, Clone)]
pub struct VersionedIndex {
    index: Engine,
    versions: BTreeMap<String, EntryVersion>,
    sequence: u64,
    history_limit: usize,
    journal: VecDeque<RevisionRecord>,
}
impl Default for VersionedIndex {
    fn default() -> Self {
        Self::new()
    }
}
impl VersionedIndex {
    pub fn new() -> Self {
        Self {
            index: Engine::new(),
            versions: BTreeMap::new(),
            sequence: 0,
            history_limit: 0,
            journal: VecDeque::new(),
        }
    }
    pub fn from_index(index: Engine, change: ChangeMetadata) -> Result<Self, VersionError> {
        let mut result = Self {
            index,
            ..Self::new()
        };
        let change = Arc::new(change);
        for entry in result.index.entries() {
            result.sequence = result
                .sequence
                .checked_add(1)
                .ok_or(VersionError::Exhausted)?;
            result.versions.insert(
                entry.id.clone(),
                EntryVersion {
                    revision: Revision(result.sequence),
                    action: RevisionAction::Insert,
                    change: Arc::clone(&change),
                    deleted: false,
                },
            );
        }
        Ok(result)
    }
    pub fn with_history_limit(mut self, limit: usize) -> Self {
        self.history_limit = limit;
        while self.journal.len() > limit {
            self.journal.pop_front();
        }
        self
    }
    pub fn history_limit(&self) -> usize {
        self.history_limit
    }
    pub fn history_len(&self) -> usize {
        self.journal.len()
    }
    pub fn history(&self) -> impl Iterator<Item = &RevisionRecord> {
        self.journal.iter()
    }
    pub fn history_for<'a>(&'a self, id: &'a str) -> impl Iterator<Item = &'a RevisionRecord> {
        self.journal.iter().filter(move |r| r.id == id)
    }
    pub fn index(&self) -> &Engine {
        &self.index
    }
    pub fn into_index(self) -> Engine {
        self.index
    }
    pub fn sequence(&self) -> u64 {
        self.sequence
    }
    pub fn tracked_ids(&self) -> usize {
        self.versions.len()
    }
    pub fn version(&self, id: &str) -> Option<&EntryVersion> {
        self.versions.get(id)
    }
    pub fn revision(&self, id: &str) -> Option<Revision> {
        self.version(id).map(|v| v.revision)
    }
    fn prepare(&self, id: &str, expected: ExpectedRevision) -> Result<Revision, VersionError> {
        let current = self.revision(id);
        if !matches!((expected, current), (ExpectedRevision::Absent, None))
            && !matches!((expected,current),(ExpectedRevision::Exact(a),Some(b)) if a==b)
        {
            return Err(VersionError::Conflict {
                id: id.into(),
                expected,
                current,
            });
        }
        self.sequence
            .checked_add(1)
            .map(Revision)
            .ok_or(VersionError::Exhausted)
    }
    fn record(&mut self, id: String, version: EntryVersion) {
        self.sequence = version.revision.0;
        if self.history_limit > 0 {
            if self.journal.len() == self.history_limit {
                self.journal.pop_front();
            }
            self.journal.push_back(RevisionRecord {
                entry: self.index.get(&id).cloned(),
                id: id.clone(),
                version: version.clone(),
            });
        }
        self.versions.insert(id, version);
    }
    pub fn upsert(
        &mut self,
        entry: KnowledgeEntry,
        expected: ExpectedRevision,
        change: ChangeMetadata,
    ) -> Result<Revision, VersionError> {
        let normalized = validate_entry(&entry)?;
        let revision = self.prepare(&entry.id, expected)?;
        let action = if self.index.get(&entry.id).is_some() {
            RevisionAction::Replace
        } else {
            RevisionAction::Insert
        };
        let id = entry.id.clone();
        self.index.insert_validated(entry, normalized);
        self.record(
            id,
            EntryVersion {
                revision,
                action,
                change: Arc::new(change),
                deleted: false,
            },
        );
        Ok(revision)
    }
    pub fn delete(
        &mut self,
        id: &str,
        expected: ExpectedRevision,
        change: ChangeMetadata,
    ) -> Result<Revision, VersionError> {
        let revision = self.prepare(id, expected)?;
        let current = self
            .version(id)
            .ok_or_else(|| VersionError::UnknownEntry(id.into()))?;
        if current.deleted {
            return Err(VersionError::AlreadyDeleted(id.into()));
        }
        self.index.remove(id);
        self.record(
            id.into(),
            EntryVersion {
                revision,
                action: RevisionAction::Delete,
                change: Arc::new(change),
                deleted: true,
            },
        );
        Ok(revision)
    }
    pub fn at(&self, id: &str, revision: Revision) -> Result<RevisionView<'_>, VersionError> {
        if let Some((id, version)) = self
            .versions
            .get_key_value(id)
            .filter(|(_, v)| v.revision == revision)
        {
            return Ok(RevisionView {
                id,
                version,
                entry: self.index.get(id),
            });
        }
        if let Some(record) = self
            .journal
            .iter()
            .find(|r| r.id == id && r.version.revision == revision)
        {
            return Ok(RevisionView {
                id: &record.id,
                version: &record.version,
                entry: record.entry.as_ref(),
            });
        }
        Err(VersionError::Unavailable {
            id: id.into(),
            revision,
        })
    }
    pub fn compare(
        &self,
        id: &str,
        from: Revision,
        to: Revision,
    ) -> Result<RevisionComparison, VersionError> {
        let before = self.at(id, from)?.entry;
        let after = self.at(id, to)?.entry;
        Ok(RevisionComparison {
            id: id.into(),
            from,
            to,
            existence_changed: before.is_some() != after.is_some(),
            text_changed: before.map(|e| &e.text) != after.map(|e| &e.text),
            metadata_changed: before.map(|e| &e.metadata) != after.map(|e| &e.metadata),
        })
    }
    pub fn restore(
        &mut self,
        id: &str,
        target: Revision,
        expected: ExpectedRevision,
        change: ChangeMetadata,
    ) -> Result<Revision, VersionError> {
        let revision = self.prepare(id, expected)?;
        let entry = self.at(id, target)?.entry.cloned();
        let normalized = entry.as_ref().map(validate_entry).transpose()?;
        let deleted = entry.is_none();
        if let Some(entry) = entry {
            self.index
                .insert_validated(entry, normalized.expect("validated restore"));
        } else {
            self.index.remove(id);
        }
        self.record(
            id.into(),
            EntryVersion {
                revision,
                action: RevisionAction::Restore,
                change: Arc::new(change),
                deleted,
            },
        );
        Ok(revision)
    }
    pub fn to_json(&self) -> Result<String, VersionError> {
        Ok(serde_json::to_string(self)?)
    }
    pub fn from_json(input: &str) -> Result<Self, VersionError> {
        Self::from_json_with_limit(input, DEFAULT_JSON_BYTES)
    }
    pub fn from_json_with_limit(input: &str, max_bytes: usize) -> Result<Self, VersionError> {
        if input.len() > max_bytes {
            return Err(VersionError::InputLimit);
        }
        let wire: SnapshotWire = serde_json::from_str(input)?;
        Self::try_from(wire)
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HeadWire {
    id: String,
    version: EntryVersion,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SnapshotWire {
    format_version: u32,
    sequence: u64,
    history_limit: usize,
    entries: Vec<KnowledgeEntry>,
    versions: Vec<HeadWire>,
    journal: VecDeque<RevisionRecord>,
}
#[derive(Serialize)]
struct HeadRef<'a> {
    id: &'a str,
    version: &'a EntryVersion,
}
#[derive(Serialize)]
struct SnapshotRef<'a> {
    format_version: u32,
    sequence: u64,
    history_limit: usize,
    entries: Vec<&'a KnowledgeEntry>,
    versions: Vec<HeadRef<'a>>,
    journal: &'a VecDeque<RevisionRecord>,
}
impl Serialize for VersionedIndex {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        SnapshotRef {
            format_version: FORMAT_VERSION,
            sequence: self.sequence,
            history_limit: self.history_limit,
            entries: self.index.entries().collect(),
            versions: self
                .versions
                .iter()
                .map(|(id, version)| HeadRef { id, version })
                .collect(),
            journal: &self.journal,
        }
        .serialize(serializer)
    }
}
fn validate_shape(
    id: &str,
    version: &EntryVersion,
    entry: Option<&KnowledgeEntry>,
) -> Result<(), VersionError> {
    if id.trim().is_empty()
        || version.deleted != entry.is_none()
        || matches!(version.action, RevisionAction::Delete) && !version.deleted
        || matches!(
            version.action,
            RevisionAction::Insert | RevisionAction::Replace
        ) && version.deleted
    {
        return Err(VersionError::InvalidData("revision state/action"));
    }
    if let Some(entry) = entry {
        if entry.id != id {
            return Err(VersionError::InvalidData("revision entry ID"));
        }
    }
    Ok(())
}
fn validate_state(
    id: &str,
    version: &EntryVersion,
    entry: Option<&KnowledgeEntry>,
) -> Result<(), VersionError> {
    validate_shape(id, version, entry)?;
    if let Some(entry) = entry {
        validate_entry(entry)?;
    }
    Ok(())
}
impl TryFrom<SnapshotWire> for VersionedIndex {
    type Error = VersionError;
    fn try_from(w: SnapshotWire) -> Result<Self, Self::Error> {
        if w.format_version != FORMAT_VERSION {
            return Err(VersionError::UnsupportedVersion(w.format_version));
        }
        if w.journal.len() > w.history_limit {
            return Err(VersionError::InvalidData("journal capacity"));
        }
        let index = Engine::from_entries(w.entries)?;
        let mut versions = BTreeMap::new();
        let mut heads = BTreeSet::new();
        for head in w.versions {
            validate_shape(&head.id, &head.version, index.get(&head.id))?;
            if head.version.revision.0 > w.sequence
                || !heads.insert(head.version.revision)
                || versions.insert(head.id, head.version).is_some()
            {
                return Err(VersionError::InvalidData(
                    "duplicate or future revision head",
                ));
            }
        }
        if index.entries().any(|e| !versions.contains_key(&e.id))
            || heads.last().map_or(0, |r| r.0) != w.sequence
        {
            return Err(VersionError::InvalidData(
                "current entry/sequence agreement",
            ));
        }
        let mut previous = 0;
        for record in &w.journal {
            validate_shape(&record.id, &record.version, record.entry.as_ref())?;
            let revision = record.version.revision.0;
            if revision <= previous || revision > w.sequence {
                return Err(VersionError::InvalidData("journal revision order"));
            }
            previous = revision;
            let head = versions
                .get(&record.id)
                .ok_or(VersionError::InvalidData("journal ID has no head"))?;
            if record.version.revision > head.revision
                || record.version.revision == head.revision
                    && (&record.version != head || record.entry.as_ref() != index.get(&record.id))
            {
                return Err(VersionError::InvalidData("journal/current state agreement"));
            }
            if heads.contains(&record.version.revision) && record.version.revision != head.revision
            {
                return Err(VersionError::InvalidData("journal/head revision identity"));
            }
        }
        Ok(Self {
            index,
            versions,
            sequence: w.sequence,
            history_limit: w.history_limit,
            journal: w.journal,
        })
    }
}
impl<'de> Deserialize<'de> for VersionedIndex {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::try_from(SnapshotWire::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}
