#![doc = include_str!("../docs/CACHE.md")]

use crate::{Engine, IndexError, KnowledgeEntry};
use serde::{Deserialize, Serialize, Serializer};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{self, BufWriter, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};
use thiserror::Error;

#[cfg(feature = "embeddings")]
use std::collections::BTreeSet;

const MAGIC: &[u8; 8] = b"AIRUSTC1";
const HEADER_BYTES: usize = 64;
const FORMAT_VERSION: u32 = 1;
const ANALYSIS_VERSION: u32 = 1;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CacheError {
    #[error("invalid cache configuration: {0}")]
    Configuration(&'static str),
    #[error("cache limit exceeded: {0}")]
    Limit(&'static str),
    #[error("invalid cache data: {0}")]
    InvalidData(&'static str),
    #[error("unsupported cache format version: {0}")]
    UnsupportedVersion(u32),
    #[error("cache text-analysis recipe does not match this build")]
    AnalysisMismatch,
    #[error("cache dataset identity or source revision does not match")]
    IdentityMismatch,
    #[error("cache fingerprint does not match the original records")]
    SourceMismatch,
    #[error("cache kind does not match the requested index")]
    KindMismatch,
    #[error("cache embedding model recipe or dimension does not match")]
    ModelMismatch,
    #[error("cache payload checksum does not match")]
    ChecksumMismatch,
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Codec(#[from] postcard::Error),
    #[error(transparent)]
    Index(#[from] IndexError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[cfg(feature = "embeddings")]
    #[error(transparent)]
    Embedding(#[from] crate::semantic::EmbeddingError),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "IdentityWire")]
pub struct CacheIdentity {
    dataset: String,
    revision: Option<String>,
}
#[derive(Deserialize)]
struct IdentityWire {
    dataset: String,
    revision: Option<String>,
}
impl CacheIdentity {
    pub fn new(dataset: impl Into<String>) -> Result<Self, CacheError> {
        let dataset = dataset.into();
        validate_identity(&dataset)?;
        Ok(Self {
            dataset,
            revision: None,
        })
    }
    pub fn with_revision(mut self, revision: impl Into<String>) -> Result<Self, CacheError> {
        let revision = revision.into();
        validate_identity(&revision)?;
        self.revision = Some(revision);
        Ok(self)
    }
    pub fn dataset(&self) -> &str {
        &self.dataset
    }
    pub fn revision(&self) -> Option<&str> {
        self.revision.as_deref()
    }
}
impl TryFrom<IdentityWire> for CacheIdentity {
    type Error = CacheError;
    fn try_from(value: IdentityWire) -> Result<Self, Self::Error> {
        let mut identity = Self::new(value.dataset)?;
        if let Some(revision) = value.revision {
            identity = identity.with_revision(revision)?;
        }
        Ok(identity)
    }
}
fn validate_identity(value: &str) -> Result<(), CacheError> {
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        return Err(CacheError::Configuration(
            "identity strings must be nonempty and contain no controls",
        ));
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct CacheLimits {
    max_file_bytes: usize,
    max_entries: usize,
    max_dimensions: usize,
    max_term_bindings: usize,
}
impl Default for CacheLimits {
    fn default() -> Self {
        Self {
            max_file_bytes: 512 * 1024 * 1024,
            max_entries: 1_000_000,
            max_dimensions: 16_384,
            max_term_bindings: 10_000_000,
        }
    }
}
impl CacheLimits {
    pub fn new(max_file_bytes: usize, max_entries: usize) -> Result<Self, CacheError> {
        if max_file_bytes < HEADER_BYTES {
            return Err(CacheError::Configuration(
                "file limit must fit the 64-byte header",
            ));
        }
        Ok(Self {
            max_file_bytes,
            max_entries,
            ..Self::default()
        })
    }
    pub fn with_max_dimensions(mut self, dimensions: usize) -> Result<Self, CacheError> {
        if dimensions == 0 {
            return Err(CacheError::Configuration(
                "dimension limit must be positive",
            ));
        }
        self.max_dimensions = dimensions;
        Ok(self)
    }
    pub fn with_max_term_bindings(mut self, bindings: usize) -> Self {
        self.max_term_bindings = bindings;
        self
    }
    pub fn max_file_bytes(&self) -> usize {
        self.max_file_bytes
    }
    pub fn max_entries(&self) -> usize {
        self.max_entries
    }
    pub fn max_dimensions(&self) -> usize {
        self.max_dimensions
    }
    pub fn max_term_bindings(&self) -> usize {
        self.max_term_bindings
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CacheKind {
    Lexical,
    Semantic,
}
impl CacheKind {
    fn byte(self) -> u8 {
        match self {
            Self::Lexical => 0,
            Self::Semantic => 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModelWire {
    recipe: String,
    dimensions: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MetadataWire {
    created_by: String,
    identity: CacheIdentity,
    analysis_signature: String,
    source_fingerprint: [u8; 32],
    kind: CacheKind,
    entries: usize,
    term_bindings: usize,
    model: Option<ModelWire>,
}
#[derive(Debug, Clone, Serialize)]
pub struct CacheMetadata(MetadataWire);
impl CacheMetadata {
    pub fn format_version(&self) -> u32 {
        FORMAT_VERSION
    }
    pub fn identity(&self) -> &CacheIdentity {
        &self.0.identity
    }
    pub fn created_by(&self) -> &str {
        &self.0.created_by
    }
    pub fn analysis_signature(&self) -> &str {
        &self.0.analysis_signature
    }
    pub fn source_fingerprint(&self) -> &[u8; 32] {
        &self.0.source_fingerprint
    }
    pub fn source_fingerprint_hex(&self) -> String {
        self.0
            .source_fingerprint
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }
    pub fn kind(&self) -> CacheKind {
        self.0.kind
    }
    pub fn entries(&self) -> usize {
        self.0.entries
    }
    pub fn term_bindings(&self) -> usize {
        self.0.term_bindings
    }
    pub fn model_recipe(&self) -> Option<&str> {
        self.0.model.as_ref().map(|m| m.recipe.as_str())
    }
    pub fn dimensions(&self) -> Option<usize> {
        self.0.model.as_ref().map(|m| m.dimensions)
    }
}

#[derive(Debug, Clone)]
pub struct CacheLoad<T> {
    index: T,
    metadata: CacheMetadata,
}
impl<T> CacheLoad<T> {
    pub fn index(&self) -> &T {
        &self.index
    }
    pub fn metadata(&self) -> &CacheMetadata {
        &self.metadata
    }
    pub fn into_index(self) -> T {
        self.index
    }
    pub fn into_parts(self) -> (T, CacheMetadata) {
        (self.index, self.metadata)
    }
}

#[derive(Debug, Clone)]
pub struct IndexCache {
    path: PathBuf,
    identity: CacheIdentity,
    limits: CacheLimits,
}
impl IndexCache {
    pub fn new(path: impl Into<PathBuf>, identity: CacheIdentity) -> Self {
        Self {
            path: path.into(),
            identity,
            limits: CacheLimits::default(),
        }
    }
    pub fn with_limits(mut self, limits: CacheLimits) -> Self {
        self.limits = limits;
        self
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn identity(&self) -> &CacheIdentity {
        &self.identity
    }
    pub fn limits(&self) -> &CacheLimits {
        &self.limits
    }

    pub fn save(&self, index: &Engine) -> Result<CacheMetadata, CacheError> {
        self.save_index(index, None, |_| Ok(()))
    }

    pub fn load(
        &self,
        originals: &mut Vec<KnowledgeEntry>,
    ) -> Result<CacheLoad<Engine>, CacheError> {
        let (metadata, documents, vectors) = self.read(originals, CacheKind::Lexical, None)?;
        if vectors.is_some() {
            return Err(CacheError::InvalidData("lexical cache contains vectors"));
        }
        let prepared = crate::index::prepare_cached_documents(originals, documents)?;
        let index = prepared.restore(std::mem::take(originals));
        Ok(CacheLoad {
            index,
            metadata: CacheMetadata(metadata),
        })
    }

    #[cfg(feature = "embeddings")]
    pub fn save_semantic(
        &self,
        index: &crate::semantic::SemanticIndex,
    ) -> Result<CacheMetadata, CacheError> {
        let model = ModelWire {
            recipe: index.space().model().to_owned(),
            dimensions: index.space().dimensions(),
        };
        if model.dimensions > self.limits.max_dimensions {
            return Err(CacheError::Limit("embedding dimensions"));
        }
        self.save_index(index.lexical(), Some(model), |writer| {
            for entry in index.lexical().entries() {
                postcard::to_io(&entry.id, &mut *writer)?;
                postcard::to_io(
                    index
                        .embedding(&entry.id)
                        .expect("semantic and lexical IDs agree"),
                    &mut *writer,
                )?;
            }
            Ok(())
        })
    }

    #[cfg(feature = "embeddings")]
    pub fn load_semantic(
        &self,
        originals: &mut Vec<KnowledgeEntry>,
        space: &crate::semantic::EmbeddingSpace,
    ) -> Result<CacheLoad<crate::semantic::SemanticIndex>, CacheError> {
        if space.dimensions() > self.limits.max_dimensions {
            return Err(CacheError::Limit("embedding dimensions"));
        }
        let expected = ModelWire {
            recipe: space.model().to_owned(),
            dimensions: space.dimensions(),
        };
        let (metadata, documents, vectors) =
            self.read(originals, CacheKind::Semantic, Some(&expected))?;
        let prepared = crate::index::prepare_cached_documents(originals, documents)?;
        let vectors = vectors.ok_or(CacheError::InvalidData("semantic vectors are missing"))?;
        let original_ids: BTreeSet<_> = originals.iter().map(|entry| entry.id.as_str()).collect();
        let mut validated = BTreeMap::new();
        for (id, vector) in vectors {
            if !original_ids.contains(id.as_str()) || validated.contains_key(&id) {
                return Err(CacheError::InvalidData("semantic IDs are inconsistent"));
            }
            let norm = crate::semantic::vector_norm(&vector, space.dimensions())?;
            if (norm - 1.0).abs() > 0.0001 {
                return Err(CacheError::InvalidData("cached vector is not normalized"));
            }
            validated.insert(id, vector);
        }
        if validated.len() != originals.len() {
            return Err(CacheError::InvalidData(
                "semantic vector count is inconsistent",
            ));
        }
        drop(original_ids);
        let lexical = prepared.restore(std::mem::take(originals));
        let index =
            crate::semantic::SemanticIndex::from_cached_parts(lexical, space.clone(), validated);
        Ok(CacheLoad {
            index,
            metadata: CacheMetadata(metadata),
        })
    }
}

fn analysis_signature() -> String {
    format!(
        "airust-text-v1:std={:?}:nfkc={:?}",
        char::UNICODE_VERSION,
        unicode_normalization::UNICODE_VERSION
    )
}

struct HashSink(Sha256);
impl Write for HashSink {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.0.update(buffer);
        Ok(buffer.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
struct EngineRecords<'a>(&'a Engine);
impl Serialize for EngineRecords<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.0.entries())
    }
}
struct SortedRecords<'a, 'b>(&'a BTreeMap<&'b str, &'b KnowledgeEntry>);
impl Serialize for SortedRecords<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.0.values())
    }
}
fn fingerprint(records: &impl Serialize) -> Result<[u8; 32], CacheError> {
    let mut writer = HashSink(Sha256::new());
    serde_json::to_writer(&mut writer, records)?;
    Ok(writer.0.finalize().into())
}
fn prepare_sources<'a>(
    entries: &'a [KnowledgeEntry],
    limits: &CacheLimits,
) -> Result<BTreeMap<&'a str, &'a KnowledgeEntry>, CacheError> {
    if entries.len() > limits.max_entries {
        return Err(CacheError::Limit("source entries"));
    }
    let mut sources = BTreeMap::new();
    for entry in entries {
        crate::index::check_source_entry(entry)?;
        if sources.insert(entry.id.as_str(), entry).is_some() {
            return Err(IndexError::DuplicateId(entry.id.clone()).into());
        }
    }
    Ok(sources)
}

struct HashWriter<W> {
    inner: W,
    hash: Sha256,
    bytes: usize,
    limit: usize,
    exceeded: bool,
    failure: Option<io::Error>,
}
impl<W: Write> Write for HashWriter<W> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if buffer.len() > self.limit.saturating_sub(self.bytes) {
            self.exceeded = true;
            return Err(io::Error::other("cache size limit exceeded"));
        }
        match self.inner.write(buffer) {
            Ok(count) => {
                self.hash.update(&buffer[..count]);
                self.bytes += count;
                Ok(count)
            }
            Err(error) => {
                let returned = io::Error::new(error.kind(), error.to_string());
                self.failure = Some(error);
                Err(returned)
            }
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}
type PayloadWriter<'a> = HashWriter<BufWriter<&'a mut File>>;

fn header(kind: CacheKind, bytes: usize, digest: [u8; 32]) -> [u8; HEADER_BYTES] {
    let mut output = [0; HEADER_BYTES];
    output[..8].copy_from_slice(MAGIC);
    output[8..12].copy_from_slice(&FORMAT_VERSION.to_le_bytes());
    output[12..16].copy_from_slice(&ANALYSIS_VERSION.to_le_bytes());
    output[16] = kind.byte();
    output[24..32].copy_from_slice(&(bytes as u64).to_le_bytes());
    output[32..].copy_from_slice(&digest);
    output
}

impl IndexCache {
    fn save_index(
        &self,
        index: &Engine,
        model: Option<ModelWire>,
        vectors: impl FnOnce(&mut PayloadWriter<'_>) -> Result<(), postcard::Error>,
    ) -> Result<CacheMetadata, CacheError> {
        if index.len() > self.limits.max_entries {
            return Err(CacheError::Limit("source entries"));
        }
        let term_bindings = index.cache_term_bindings()?;
        if term_bindings > self.limits.max_term_bindings {
            return Err(CacheError::Limit("term bindings"));
        }
        let metadata = MetadataWire {
            created_by: format!("airust/{}", crate::VERSION),
            identity: self.identity.clone(),
            analysis_signature: analysis_signature(),
            source_fingerprint: fingerprint(&EngineRecords(index))?,
            kind: if model.is_some() {
                CacheKind::Semantic
            } else {
                CacheKind::Lexical
            },
            entries: index.len(),
            term_bindings,
            model,
        };
        let mut stage_error = None;
        let written = crate::persistence::atomic_write_private_with(&self.path, |file| {
            file.write_all(&[0; HEADER_BYTES])?;
            let (length, digest) = {
                let mut writer = HashWriter {
                    inner: BufWriter::new(&mut *file),
                    hash: Sha256::new(),
                    bytes: 0,
                    limit: self.limits.max_file_bytes - HEADER_BYTES,
                    exceeded: false,
                    failure: None,
                };
                let serialization = (|| {
                    postcard::to_io(&metadata, &mut writer)?;
                    postcard::to_io(&metadata.entries, &mut writer)?;
                    index.write_cached_documents(&mut writer)?;
                    let vector_count = if metadata.kind == CacheKind::Semantic {
                        metadata.entries
                    } else {
                        0
                    };
                    postcard::to_io(&vector_count, &mut writer)?;
                    vectors(&mut writer)?;
                    Ok::<_, postcard::Error>(())
                })();
                if let Err(error) = serialization {
                    stage_error = Some(if writer.exceeded {
                        CacheError::Limit("file bytes")
                    } else if let Some(error) = writer.failure.take() {
                        CacheError::Io(error)
                    } else {
                        CacheError::Codec(error)
                    });
                    return Err(io::Error::other("cache serialization failed"));
                }
                writer.flush()?;
                (writer.bytes, writer.hash.finalize().into())
            };
            file.seek(SeekFrom::Start(0))?;
            file.write_all(&header(metadata.kind, length, digest))?;
            Ok(())
        });
        if let Err(error) = written {
            return Err(stage_error.unwrap_or(CacheError::Io(error)));
        }
        Ok(CacheMetadata(metadata))
    }
}

struct Decoder<'a> {
    remaining: &'a [u8],
}
impl<'a> Decoder<'a> {
    fn read<T: Deserialize<'a>>(&mut self) -> Result<T, CacheError> {
        let (value, remaining) = postcard::take_from_bytes(self.remaining)?;
        self.remaining = remaining;
        Ok(value)
    }
    fn vector(&mut self, dimensions: usize) -> Result<Vec<f32>, CacheError> {
        let (declared, _) = postcard::take_from_bytes::<usize>(self.remaining)?;
        if declared != dimensions {
            return Err(CacheError::InvalidData(
                "declared vector length does not match its model",
            ));
        }
        self.read()
    }
}
type CachedVectors = Option<Vec<(String, Vec<f32>)>>;
type DecodedCache = (
    MetadataWire,
    Vec<crate::index::CacheDocument>,
    CachedVectors,
);

impl IndexCache {
    fn read(
        &self,
        originals: &[KnowledgeEntry],
        kind: CacheKind,
        model: Option<&ModelWire>,
    ) -> Result<DecodedCache, CacheError> {
        let sources = prepare_sources(originals, &self.limits)?;
        let disk = fs::symlink_metadata(&self.path)?;
        if !disk.is_file() || disk.file_type().is_symlink() {
            return Err(CacheError::InvalidData(
                "cache must be a regular file, not a symlink",
            ));
        }
        if disk.len() > self.limits.max_file_bytes as u64 {
            return Err(CacheError::Limit("file bytes"));
        }
        let mut file = File::open(&self.path)?;
        let disk = file.metadata()?;
        if !disk.is_file() {
            return Err(CacheError::InvalidData(
                "cache must be a regular file, not a symlink",
            ));
        }
        if disk.len() > self.limits.max_file_bytes as u64 {
            return Err(CacheError::Limit("file bytes"));
        }
        let mut prefix = [0; HEADER_BYTES];
        file.read_exact(&mut prefix)?;
        if &prefix[..8] != MAGIC {
            return Err(CacheError::InvalidData("cache magic"));
        }
        let version = u32::from_le_bytes(prefix[8..12].try_into().expect("fixed header"));
        if version != FORMAT_VERSION {
            return Err(CacheError::UnsupportedVersion(version));
        }
        if u32::from_le_bytes(prefix[12..16].try_into().expect("fixed header")) != ANALYSIS_VERSION
        {
            return Err(CacheError::AnalysisMismatch);
        }
        if prefix[16] != kind.byte() {
            return Err(CacheError::KindMismatch);
        }
        if prefix[17..24].iter().any(|&byte| byte != 0) {
            return Err(CacheError::InvalidData("reserved header bytes"));
        }
        let length = u64::from_le_bytes(prefix[24..32].try_into().expect("fixed header"));
        let declared = length
            .checked_add(HEADER_BYTES as u64)
            .ok_or(CacheError::Limit("file bytes"))?;
        if declared != disk.len() {
            return Err(CacheError::InvalidData("cache file length"));
        }
        if declared > self.limits.max_file_bytes as u64 {
            return Err(CacheError::Limit("file bytes"));
        }
        let mut payload = Vec::new();
        payload
            .try_reserve_exact(
                usize::try_from(length).map_err(|_| CacheError::Limit("file bytes"))?,
            )
            .map_err(|_| CacheError::Limit("payload allocation"))?;
        Read::by_ref(&mut file)
            .take(length)
            .read_to_end(&mut payload)?;
        if payload.len() as u64 != length {
            return Err(CacheError::InvalidData("cache payload length"));
        }
        let mut extra = [0u8; 1];
        if file.read(&mut extra)? != 0 {
            return Err(CacheError::InvalidData("cache grew during reading"));
        }
        let digest: [u8; 32] = Sha256::digest(&payload).into();
        if digest[..] != prefix[32..] {
            return Err(CacheError::ChecksumMismatch);
        }
        let mut reader = Decoder {
            remaining: &payload,
        };
        let metadata: MetadataWire = reader.read()?;
        if metadata.kind != kind {
            return Err(CacheError::KindMismatch);
        }
        if metadata.identity != self.identity {
            return Err(CacheError::IdentityMismatch);
        }
        if metadata.analysis_signature != analysis_signature() {
            return Err(CacheError::AnalysisMismatch);
        }
        if metadata.source_fingerprint != fingerprint(&SortedRecords(&sources))? {
            return Err(CacheError::SourceMismatch);
        }
        if metadata.entries != sources.len() {
            return Err(CacheError::InvalidData("source entry count"));
        }
        if metadata.term_bindings > self.limits.max_term_bindings {
            return Err(CacheError::Limit("term bindings"));
        }
        match (model, metadata.model.as_ref()) {
            (None, None) => {}
            (Some(expected), Some(actual))
                if expected.recipe == actual.recipe && expected.dimensions == actual.dimensions => {
            }
            _ => return Err(CacheError::ModelMismatch),
        }
        let count: usize = reader.read()?;
        if count != sources.len() {
            return Err(CacheError::InvalidData("declared document count"));
        }
        let mut documents = Vec::new();
        let mut remaining_terms = metadata.term_bindings;
        for _ in 0..count {
            let record: crate::index::CacheDocumentHeader = reader.read()?;
            if !sources.contains_key(record.id.as_str()) {
                return Err(CacheError::InvalidData("unknown cached source ID"));
            }
            if record.normalized.trim().is_empty()
                || record.char_length != record.normalized.chars().count()
                || record.term_count > record.char_length
                || record.length > record.char_length
                || record.length < record.term_count
            {
                return Err(CacheError::InvalidData("cached document header"));
            }
            if record.term_count > remaining_terms {
                return Err(CacheError::InvalidData("declared term count"));
            }
            remaining_terms -= record.term_count;
            let mut terms = BTreeMap::new();
            for _ in 0..record.term_count {
                let (term, frequency): (String, usize) = reader.read()?;
                if terms.insert(term, frequency).is_some() {
                    return Err(CacheError::InvalidData("duplicate cached term"));
                }
            }
            documents.push(crate::index::CacheDocument {
                id: record.id,
                normalized: record.normalized,
                length: record.length,
                char_length: record.char_length,
                terms,
            });
        }
        if remaining_terms != 0 {
            return Err(CacheError::InvalidData("term-binding count"));
        }
        let vector_count: usize = reader.read()?;
        let mut vectors = None;
        if kind == CacheKind::Semantic {
            if vector_count != sources.len() {
                return Err(CacheError::InvalidData("declared vector count"));
            }
            let dimensions = model.ok_or(CacheError::ModelMismatch)?.dimensions;
            let mut values = Vec::new();
            for _ in 0..vector_count {
                let id = reader.read::<String>()?;
                values.push((id, reader.vector(dimensions)?));
            }
            vectors = Some(values);
        } else if vector_count != 0 {
            return Err(CacheError::InvalidData("vectors in lexical cache"));
        }
        if !reader.remaining.is_empty() {
            return Err(CacheError::InvalidData("trailing cache data"));
        }
        Ok((metadata, documents, vectors))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn originals() -> Vec<KnowledgeEntry> {
        vec![
            KnowledgeEntry::new("a", "alpha beta"),
            KnowledgeEntry::new("b", "beta gamma"),
        ]
    }
    fn encode<T: Serialize>(out: &mut Vec<u8>, value: &T) {
        postcard::to_io(value, out).unwrap();
    }
    fn forged(cache: &IndexCache, mutate: impl FnOnce(&mut MetadataWire, &mut Vec<u8>)) {
        cache
            .save(&Engine::from_entries(originals()).unwrap())
            .unwrap();
        let valid = fs::read(cache.path()).unwrap();
        let (mut metadata, _): (MetadataWire, _) =
            postcard::take_from_bytes(&valid[HEADER_BYTES..]).unwrap();
        let mut body = Vec::new();
        mutate(&mut metadata, &mut body);
        let mut payload = Vec::new();
        encode(&mut payload, &metadata);
        payload.extend(body);
        let mut bytes = header(
            metadata.kind,
            payload.len(),
            Sha256::digest(&payload).into(),
        )
        .to_vec();
        bytes.extend(payload);
        fs::write(cache.path(), bytes).unwrap();
        let mut records = originals();
        assert!(cache.load(&mut records).is_err());
        assert_eq!(records, originals());
    }
    fn records(out: &mut Vec<u8>, ids: [&str; 2], term: &str, frequency: usize, chars: usize) {
        encode(out, &2usize);
        for id in ids {
            encode(
                out,
                &crate::index::CacheDocumentHeader {
                    id: id.into(),
                    normalized: "alpha beta".into(),
                    length: 2,
                    char_length: chars,
                    term_count: 2,
                },
            );
            encode(out, &(term, frequency));
            encode(out, &("beta", 1usize));
        }
        encode(out, &0usize);
    }
    #[test]
    fn valid_checksums_do_not_bypass_structure_counts_identity_or_analysis_checks() {
        let root = tempfile::tempdir().unwrap();
        let cache = IndexCache::new(
            root.path().join("cache"),
            CacheIdentity::new("test").unwrap(),
        );
        forged(&cache, |_, b| encode(b, &usize::MAX));
        forged(&cache, |m, _| {
            m.identity = CacheIdentity::new("wrong").unwrap()
        });
        forged(&cache, |m, _| m.analysis_signature = "old".into());
        forged(&cache, |m, _| m.source_fingerprint = [0; 32]);
        forged(&cache, |m, _| m.entries = usize::MAX);
        forged(&cache, |m, _| m.term_bindings = usize::MAX);
        forged(&cache, |_, b| records(b, ["a", "a"], "alpha", 1, 10));
        forged(&cache, |_, b| records(b, ["a", "missing"], "alpha", 1, 10));
        forged(&cache, |_, b| {
            records(b, ["a", "b"], "alpha", 1, usize::MAX)
        });
        forged(&cache, |_, b| records(b, ["a", "b"], "two words", 1, 10));
        forged(&cache, |_, b| records(b, ["a", "b"], "alpha", 0, 10));
        forged(&cache, |_, b| {
            records(b, ["a", "b"], "alpha", usize::MAX, 10)
        });
        forged(&cache, |_, b| records(b, ["a", "b"], "beta", 1, 10));
        forged(&cache, |_, b| {
            records(b, ["a", "b"], "alpha", 1, 10);
            b.push(0);
        });
    }
    #[cfg(feature = "embeddings")]
    #[test]
    fn rehashed_invalid_vectors_are_rejected_before_originals_are_consumed() {
        use crate::semantic::{EmbeddingSpace, SemanticIndex};
        let root = tempfile::tempdir().unwrap();
        let cache = IndexCache::new(
            root.path().join("cache"),
            CacheIdentity::new("test").unwrap(),
        );
        let space = EmbeddingSpace::new("recipe", 2).unwrap();
        let mut index = SemanticIndex::new(space.clone());
        for entry in originals() {
            index
                .upsert_embedded(
                    entry,
                    crate::semantic::Embedding::new(space.clone(), vec![1.0, 0.0]).unwrap(),
                )
                .unwrap();
        }
        cache.save_semantic(&index).unwrap();
        let valid = fs::read(cache.path()).unwrap();
        let mut reader = Decoder {
            remaining: &valid[HEADER_BYTES..],
        };
        let _: MetadataWire = reader.read().unwrap();
        let count: usize = reader.read().unwrap();
        for _ in 0..count {
            let doc: crate::index::CacheDocumentHeader = reader.read().unwrap();
            for _ in 0..doc.term_count {
                let _: (String, usize) = reader.read().unwrap();
            }
        }
        let prefix_len = valid.len() - reader.remaining.len();
        for (ids, vector) in [
            (["a", "a"], vec![1.0, 0.0]),
            (["a", "missing"], vec![1.0, 0.0]),
            (["a", "b"], vec![0.0, 0.0]),
            (["a", "b"], vec![f32::NAN, 0.0]),
            (["a", "b"], vec![2.0, 0.0]),
            (["a", "b"], vec![1.0]),
        ] {
            let mut payload = valid[HEADER_BYTES..prefix_len].to_vec();
            encode(&mut payload, &2usize);
            for id in ids {
                encode(&mut payload, &id);
                encode(&mut payload, &vector);
            }
            let mut bytes = header(
                CacheKind::Semantic,
                payload.len(),
                Sha256::digest(&payload).into(),
            )
            .to_vec();
            bytes.extend(payload);
            fs::write(cache.path(), bytes).unwrap();
            let mut input = originals();
            assert!(cache.load_semantic(&mut input, &space).is_err());
            assert_eq!(input, originals());
        }
    }
}
