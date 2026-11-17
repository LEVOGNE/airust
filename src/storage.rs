#![doc = include_str!("../docs/STORAGE.md")]

use crate::{index::validate_entry, Engine, IndexError, KnowledgeEntry};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, Write},
};
use thiserror::Error;

#[cfg(feature = "sqlite-storage")]
pub mod sqlite;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum StorageError {
    #[error("invalid storage configuration: {0}")]
    Configuration(&'static str),
    #[error("storage limit exceeded: {0}")]
    Limit(&'static str),
    #[error("storage revision or identity changed")]
    Conflict,
    #[error("storage generation is exhausted")]
    GenerationExhausted,
    #[error("storage is read-only")]
    ReadOnly,
    #[error("invalid storage data: {0}")]
    InvalidData(&'static str),
    #[error("storage operation is not supported: {0}")]
    Unsupported(&'static str),
    #[error("storage backend failed: {0}")]
    Backend(String),
    #[error(transparent)]
    Index(#[from] IndexError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[cfg(feature = "sqlite-storage")]
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RevisionWire")]
pub struct StoreRevision {
    namespace: String,
    identity: String,
    generation: u64,
}
#[derive(Deserialize)]
struct RevisionWire {
    namespace: String,
    identity: String,
    generation: u64,
}
impl StoreRevision {
    pub fn new(
        namespace: impl Into<String>,
        identity: impl Into<String>,
        generation: u64,
    ) -> Result<Self, StorageError> {
        let namespace = namespace.into();
        let identity = identity.into();
        validate_namespace(&namespace)?;
        if identity.len() != 32
            || !identity
                .bytes()
                .all(|b| b.is_ascii_digit() || matches!(b, b'a'..=b'f'))
            || generation > i64::MAX as u64
        {
            return Err(StorageError::Configuration(
                "invalid store identity/generation",
            ));
        }
        Ok(Self {
            namespace,
            identity,
            generation,
        })
    }
    pub fn namespace(&self) -> &str {
        &self.namespace
    }
    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub(crate) fn fresh(namespace: &str) -> Result<Self, StorageError> {
        let mut bytes = [0u8; 16];
        getrandom::fill(&mut bytes).map_err(|e| StorageError::Backend(e.to_string()))?;
        Self::new(
            namespace,
            bytes.iter().map(|b| format!("{b:02x}")).collect::<String>(),
            0,
        )
    }
    pub(crate) fn next(&self) -> Result<Self, StorageError> {
        let mut next = self.clone();
        next.generation = next
            .generation
            .checked_add(1)
            .filter(|n| *n <= i64::MAX as u64)
            .ok_or(StorageError::GenerationExhausted)?;
        Ok(next)
    }
}
impl TryFrom<RevisionWire> for StoreRevision {
    type Error = StorageError;
    fn try_from(v: RevisionWire) -> Result<Self, Self::Error> {
        Self::new(v.namespace, v.identity, v.generation)
    }
}
fn validate_namespace(name: &str) -> Result<(), StorageError> {
    if name.trim().is_empty() || name.len() > 128 || name.chars().any(char::is_control) {
        return Err(StorageError::Configuration(
            "namespace must contain 1..128 bytes and no controls",
        ));
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct StorageLimits {
    max_id_bytes: usize,
    max_entry_bytes: usize,
    max_batch_operations: usize,
    max_batch_bytes: usize,
    max_page_rows: usize,
    max_page_bytes: usize,
    max_metadata_depth: usize,
    max_metadata_nodes: usize,
}
impl Default for StorageLimits {
    fn default() -> Self {
        Self {
            max_id_bytes: 4096,
            max_entry_bytes: 1024 * 1024,
            max_batch_operations: 1000,
            max_batch_bytes: 16 * 1024 * 1024,
            max_page_rows: 1000,
            max_page_bytes: 4 * 1024 * 1024,
            max_metadata_depth: 64,
            max_metadata_nodes: 100_000,
        }
    }
}
impl StorageLimits {
    pub fn with_entry_limits(
        mut self,
        id_bytes: usize,
        entry_bytes: usize,
    ) -> Result<Self, StorageError> {
        if id_bytes == 0 || entry_bytes == 0 {
            return Err(StorageError::Configuration("entry limits must be positive"));
        }
        self.max_id_bytes = id_bytes;
        self.max_entry_bytes = entry_bytes;
        Ok(self)
    }
    pub fn with_batch_limits(
        mut self,
        operations: usize,
        bytes: usize,
    ) -> Result<Self, StorageError> {
        if operations == 0 || bytes == 0 {
            return Err(StorageError::Configuration("batch limits must be positive"));
        }
        self.max_batch_operations = operations;
        self.max_batch_bytes = bytes;
        Ok(self)
    }
    pub fn with_page_limits(mut self, rows: usize, bytes: usize) -> Result<Self, StorageError> {
        if rows == 0 || bytes == 0 {
            return Err(StorageError::Configuration("page limits must be positive"));
        }
        self.max_page_rows = rows;
        self.max_page_bytes = bytes;
        Ok(self)
    }
    pub fn with_metadata_limits(
        mut self,
        depth: usize,
        nodes: usize,
    ) -> Result<Self, StorageError> {
        if depth == 0 || depth > 64 || nodes == 0 {
            return Err(StorageError::Configuration(
                "metadata depth must be 1..64 and node budget positive",
            ));
        }
        self.max_metadata_depth = depth;
        self.max_metadata_nodes = nodes;
        Ok(self)
    }
    pub fn max_id_bytes(&self) -> usize {
        self.max_id_bytes
    }
    pub fn max_entry_bytes(&self) -> usize {
        self.max_entry_bytes
    }
    pub fn max_batch_operations(&self) -> usize {
        self.max_batch_operations
    }
    pub fn max_batch_bytes(&self) -> usize {
        self.max_batch_bytes
    }
    pub fn max_page_rows(&self) -> usize {
        self.max_page_rows
    }
    pub fn max_page_bytes(&self) -> usize {
        self.max_page_bytes
    }
    pub fn max_metadata_depth(&self) -> usize {
        self.max_metadata_depth
    }
    pub fn max_metadata_nodes(&self) -> usize {
        self.max_metadata_nodes
    }
}
#[derive(Debug, Clone)]
pub enum StoreChange {
    Put(KnowledgeEntry),
    Delete(String),
}
#[derive(Debug, Clone, Serialize)]
pub struct StoreCommit {
    pub revision: StoreRevision,
    pub upserts: usize,
    pub deleted: usize,
}
#[derive(Debug, Clone, Serialize)]
pub struct StoragePage {
    pub entries: Vec<KnowledgeEntry>,
    pub next_after: Option<String>,
}
pub trait StorageBackend {
    fn limits(&self) -> &StorageLimits;
    fn is_durable(&self) -> bool {
        false
    }
    fn is_read_only(&self) -> bool {
        false
    }
    fn revision(&mut self) -> Result<StoreRevision, StorageError>;
    fn get(
        &mut self,
        expected: &StoreRevision,
        id: &str,
    ) -> Result<Option<KnowledgeEntry>, StorageError>;
    fn page(
        &mut self,
        expected: &StoreRevision,
        after: Option<&str>,
        limit: usize,
    ) -> Result<StoragePage, StorageError>;
    fn apply(
        &mut self,
        expected: &StoreRevision,
        changes: &[StoreChange],
    ) -> Result<StoreCommit, StorageError>;
    fn renew_identity(&mut self, _expected: &StoreRevision) -> Result<StoreRevision, StorageError> {
        Err(StorageError::Unsupported("identity renewal"))
    }
}

fn validate_id(id: &str, limits: &StorageLimits) -> Result<(), StorageError> {
    if id.trim().is_empty() {
        return Err(IndexError::EmptyId.into());
    }
    if id.len() > limits.max_id_bytes {
        return Err(StorageError::Limit("entry ID bytes"));
    }
    Ok(())
}
pub(crate) fn validate_metadata(value: &Value, limits: &StorageLimits) -> Result<(), StorageError> {
    let mut stack = vec![(value, 1usize)];
    let mut nodes = 0usize;
    while let Some((value, depth)) = stack.pop() {
        nodes += 1;
        if nodes > limits.max_metadata_nodes || depth > limits.max_metadata_depth {
            return Err(StorageError::Limit("metadata depth/node count"));
        }
        match value {
            Value::Array(values) => {
                if values.len()
                    > limits
                        .max_metadata_nodes
                        .saturating_sub(nodes + stack.len())
                {
                    return Err(StorageError::Limit("metadata nodes"));
                }
                stack.extend(values.iter().map(|v| (v, depth + 1)));
            }
            Value::Object(values) => {
                if values.len()
                    > limits
                        .max_metadata_nodes
                        .saturating_sub(nodes + stack.len())
                {
                    return Err(StorageError::Limit("metadata nodes"));
                }
                stack.extend(values.values().map(|v| (v, depth + 1)));
            }
            _ => {}
        }
    }
    Ok(())
}
struct LimitedBytes {
    bytes: Vec<u8>,
    limit: usize,
    exceeded: bool,
}
impl Write for LimitedBytes {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            self.exceeded = true;
            return Err(io::Error::other("encoded entry limit"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
pub(crate) fn encode_entry(
    entry: &KnowledgeEntry,
    limits: &StorageLimits,
) -> Result<String, StorageError> {
    validate_id(&entry.id, limits)?;
    if entry.text.len() > limits.max_entry_bytes {
        return Err(StorageError::Limit("entry text bytes"));
    }
    validate_entry(entry)?;
    if let Some(value) = &entry.metadata {
        validate_metadata(value, limits)?;
    }
    let mut output = LimitedBytes {
        bytes: Vec::new(),
        limit: limits.max_entry_bytes,
        exceeded: false,
    };
    let encoded = serde_json::to_writer(&mut output, entry);
    if output.exceeded {
        return Err(StorageError::Limit("encoded entry bytes"));
    }
    encoded?;
    String::from_utf8(output.bytes)
        .map_err(|_| StorageError::InvalidData("encoded entry is not UTF-8"))
}
fn decode_entry(
    id: &str,
    encoded: &str,
    limits: &StorageLimits,
) -> Result<KnowledgeEntry, StorageError> {
    if encoded.len() > limits.max_entry_bytes {
        return Err(StorageError::Limit("stored entry bytes"));
    }
    let entry: KnowledgeEntry = serde_json::from_str(encoded)?;
    if entry.id != id {
        return Err(StorageError::InvalidData(
            "stored row and record IDs differ",
        ));
    }
    encode_entry(&entry, limits)?;
    Ok(entry)
}
fn prepare(
    changes: &[StoreChange],
    limits: &StorageLimits,
) -> Result<Vec<(String, Option<String>)>, StorageError> {
    if changes.len() > limits.max_batch_operations {
        return Err(StorageError::Limit("batch operation count"));
    }
    let mut prepared = Vec::new();
    let mut ids = BTreeSet::new();
    let mut bytes = 0usize;
    for change in changes {
        let (id, value) = match change {
            StoreChange::Put(entry) => (&entry.id, Some(encode_entry(entry, limits)?)),
            StoreChange::Delete(id) => {
                validate_id(id, limits)?;
                (id, None)
            }
        };
        if !ids.insert(id) {
            return Err(StorageError::InvalidData("duplicate change ID"));
        }
        bytes = bytes
            .checked_add(value.as_ref().map_or(id.len(), String::len))
            .filter(|n| *n <= limits.max_batch_bytes)
            .ok_or(StorageError::Limit("batch bytes"))?;
        prepared.push((id.clone(), value));
    }
    Ok(prepared)
}
fn check_revision(actual: &StoreRevision, expected: &StoreRevision) -> Result<(), StorageError> {
    if actual != expected {
        return Err(StorageError::Conflict);
    }
    Ok(())
}
fn check_page(
    after: Option<&str>,
    limit: usize,
    limits: &StorageLimits,
) -> Result<(), StorageError> {
    if limit == 0 || limit > limits.max_page_rows {
        return Err(StorageError::Configuration(
            "page rows must be positive and within backend limits",
        ));
    }
    if let Some(id) = after {
        validate_id(id, limits)?;
    }
    Ok(())
}

pub struct MemoryStore {
    revision: StoreRevision,
    limits: StorageLimits,
    records: BTreeMap<String, String>,
}
impl MemoryStore {
    pub fn new(namespace: &str) -> Result<Self, StorageError> {
        Self::with_limits(namespace, StorageLimits::default())
    }
    pub fn with_limits(namespace: &str, limits: StorageLimits) -> Result<Self, StorageError> {
        Ok(Self {
            revision: StoreRevision::fresh(namespace)?,
            limits,
            records: BTreeMap::new(),
        })
    }
}
impl StorageBackend for MemoryStore {
    fn limits(&self) -> &StorageLimits {
        &self.limits
    }
    fn revision(&mut self) -> Result<StoreRevision, StorageError> {
        Ok(self.revision.clone())
    }
    fn get(
        &mut self,
        expected: &StoreRevision,
        id: &str,
    ) -> Result<Option<KnowledgeEntry>, StorageError> {
        check_revision(&self.revision, expected)?;
        validate_id(id, &self.limits)?;
        self.records
            .get(id)
            .map(|v| decode_entry(id, v, &self.limits))
            .transpose()
    }
    fn page(
        &mut self,
        expected: &StoreRevision,
        after: Option<&str>,
        limit: usize,
    ) -> Result<StoragePage, StorageError> {
        check_revision(&self.revision, expected)?;
        check_page(after, limit, &self.limits)?;
        let mut entries = Vec::new();
        let mut bytes = 0usize;
        let mut more = false;
        for (id, value) in self.records.range((
            std::ops::Bound::Excluded(after.unwrap_or("").to_string()),
            std::ops::Bound::Unbounded,
        )) {
            if entries.len() == limit {
                more = true;
                break;
            }
            if value.len() > self.limits.max_page_bytes.saturating_sub(bytes) {
                if entries.is_empty() {
                    return Err(StorageError::Limit("one record exceeds page bytes"));
                }
                more = true;
                break;
            }
            entries.push(decode_entry(id, value, &self.limits)?);
            bytes += value.len();
        }
        let next_after = more.then(|| entries.last().unwrap().id.clone());
        Ok(StoragePage {
            entries,
            next_after,
        })
    }
    fn apply(
        &mut self,
        expected: &StoreRevision,
        changes: &[StoreChange],
    ) -> Result<StoreCommit, StorageError> {
        let prepared = prepare(changes, &self.limits)?;
        check_revision(&self.revision, expected)?;
        if prepared.is_empty() {
            return Ok(StoreCommit {
                revision: self.revision.clone(),
                upserts: 0,
                deleted: 0,
            });
        }
        let next = self.revision.next()?;
        let (mut upserts, mut deleted) = (0, 0);
        for (id, value) in prepared {
            if let Some(value) = value {
                self.records.insert(id, value);
                upserts += 1;
            } else {
                deleted += usize::from(self.records.remove(&id).is_some());
            }
        }
        self.revision = next;
        Ok(StoreCommit {
            revision: self.revision.clone(),
            upserts,
            deleted,
        })
    }
    fn renew_identity(&mut self, expected: &StoreRevision) -> Result<StoreRevision, StorageError> {
        check_revision(&self.revision, expected)?;
        self.revision = StoreRevision::fresh(self.revision.namespace())?;
        Ok(self.revision.clone())
    }
}

#[derive(Debug, Clone)]
pub struct SnapshotOptions {
    page_rows: usize,
    max_entries: usize,
    max_bytes: usize,
}
impl Default for SnapshotOptions {
    fn default() -> Self {
        Self {
            page_rows: 128,
            max_entries: 1_000_000,
            max_bytes: 128 * 1024 * 1024,
        }
    }
}
impl SnapshotOptions {
    pub fn new(
        page_rows: usize,
        max_entries: usize,
        max_bytes: usize,
    ) -> Result<Self, StorageError> {
        if page_rows == 0 || max_entries == 0 || max_bytes == 0 {
            return Err(StorageError::Configuration(
                "snapshot limits must be positive",
            ));
        }
        Ok(Self {
            page_rows,
            max_entries,
            max_bytes,
        })
    }
    pub fn page_rows(&self) -> usize {
        self.page_rows
    }
    pub fn max_entries(&self) -> usize {
        self.max_entries
    }
    pub fn max_bytes(&self) -> usize {
        self.max_bytes
    }
}
#[derive(Debug)]
pub struct StoredSnapshot {
    revision: StoreRevision,
    entries: Vec<KnowledgeEntry>,
}
#[derive(Debug)]
pub struct LoadedIndex {
    revision: StoreRevision,
    index: Engine,
}
impl LoadedIndex {
    pub fn revision(&self) -> &StoreRevision {
        &self.revision
    }
    pub fn index(&self) -> &Engine {
        &self.index
    }
    pub fn into_parts(self) -> (StoreRevision, Engine) {
        (self.revision, self.index)
    }
}
impl StoredSnapshot {
    pub fn revision(&self) -> &StoreRevision {
        &self.revision
    }
    pub fn entries(&self) -> &[KnowledgeEntry] {
        &self.entries
    }
    pub fn into_parts(self) -> (StoreRevision, Vec<KnowledgeEntry>) {
        (self.revision, self.entries)
    }
    pub fn into_index(self) -> Result<LoadedIndex, StorageError> {
        Ok(LoadedIndex {
            revision: self.revision,
            index: Engine::from_entries(self.entries)?,
        })
    }
}
pub fn read_snapshot(
    backend: &mut (impl StorageBackend + ?Sized),
    options: &SnapshotOptions,
) -> Result<StoredSnapshot, StorageError> {
    let revision = backend.revision()?;
    let limits = backend.limits().clone();
    check_page(None, options.page_rows, &limits)?;
    let (mut entries, mut after, mut bytes) = (Vec::new(), None::<String>, 0usize);
    loop {
        let page = backend.page(&revision, after.as_deref(), options.page_rows)?;
        if page.entries.len() > options.page_rows {
            return Err(StorageError::InvalidData("backend exceeded page rows"));
        }
        let mut previous = after.as_deref();
        let mut page_bytes = 0usize;
        for entry in &page.entries {
            if previous.is_some_and(|id| entry.id.as_str() <= id) {
                return Err(StorageError::InvalidData(
                    "backend page IDs are not strictly ordered",
                ));
            }
            previous = Some(&entry.id);
            let encoded_bytes = encode_entry(entry, &limits)?.len();
            page_bytes = page_bytes
                .checked_add(encoded_bytes)
                .filter(|n| *n <= limits.max_page_bytes)
                .ok_or(StorageError::InvalidData("backend exceeded page bytes"))?;
            bytes = bytes
                .checked_add(encoded_bytes)
                .filter(|n| *n <= options.max_bytes)
                .ok_or(StorageError::Limit("snapshot bytes"))?;
        }
        if entries
            .len()
            .checked_add(page.entries.len())
            .is_none_or(|n| n > options.max_entries)
        {
            return Err(StorageError::Limit("snapshot entry count"));
        }
        if page
            .next_after
            .as_deref()
            .is_some_and(|next| page.entries.last().is_none_or(|last| next != last.id))
        {
            return Err(StorageError::InvalidData(
                "backend cursor does not follow its last entry",
            ));
        }
        let done = page.next_after.is_none();
        after = page.next_after;
        entries.extend(page.entries);
        if done {
            break;
        }
    }
    check_revision(&backend.revision()?, &revision)?;
    Ok(StoredSnapshot { revision, entries })
}
