#![doc = include_str!("../docs/SNAPSHOTS.md")]

use crate::{
    provenance::{ProvenanceError, ProvenanceStore, SourceReference},
    semantic::{EmbeddingError, EmbeddingProvider, EmbeddingSpace, SemanticIndex},
    storage::{encode_entry, validate_metadata, StorageError, StorageLimits, StoreRevision},
    Engine, IndexError, KnowledgeEntry,
};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, Write},
    sync::{Arc, RwLock},
};
use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DatasetError {
    #[cfg(feature = "execution")]
    #[error(transparent)]
    Execution(#[from] crate::execution::ExecutionError),
    #[error("invalid dataset configuration: {0}")]
    Configuration(&'static str),
    #[error("dataset limit exceeded: {0}")]
    Limit(&'static str),
    #[error("dataset changed while this work was prepared")]
    Conflict,
    #[error("external source revision is missing, stale or from another lineage")]
    SourceRevision,
    #[error("changed text requires an embedding provider for this semantic dataset")]
    MissingProvider,
    #[error("invalid dataset change: {0}")]
    InvalidData(&'static str),
    #[error("dataset publication lock is poisoned")]
    Poisoned,
    #[error(transparent)]
    Storage(#[from] StorageError),
    #[error(transparent)]
    Index(#[from] IndexError),
    #[error(transparent)]
    Embedding(#[from] EmbeddingError),
    #[error(transparent)]
    Provenance(#[from] ProvenanceError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[cfg(feature = "advanced-search")]
    #[error(transparent)]
    Text(#[from] crate::text_search::TextSearchError),
}
#[derive(Debug, Clone)]
pub struct DatasetConfig {
    limits: StorageLimits,
    max_entries: usize,
    max_total_bytes: usize,
    max_vector_bytes: usize,
    max_dimensions: usize,
    max_sources_per_entry: usize,
    batch_size: usize,
    space: Option<EmbeddingSpace>,
    #[cfg(feature = "advanced-search")]
    analyzer: Option<crate::text_search::AnalyzerConfig>,
}
impl Default for DatasetConfig {
    fn default() -> Self {
        Self {
            limits: StorageLimits::default(),
            max_entries: 100_000,
            max_total_bytes: 128 * 1024 * 1024,
            max_vector_bytes: 256 * 1024 * 1024,
            max_dimensions: 8192,
            max_sources_per_entry: 64,
            batch_size: 32,
            space: None,
            #[cfg(feature = "advanced-search")]
            analyzer: None,
        }
    }
}
impl DatasetConfig {
    pub fn with_limits(
        mut self,
        entries: usize,
        encoded_bytes: usize,
        vector_bytes: usize,
    ) -> Result<Self, DatasetError> {
        if [entries, encoded_bytes, vector_bytes].contains(&0) {
            return Err(DatasetError::Configuration(
                "dataset limits must be positive",
            ));
        }
        self.max_entries = entries;
        self.max_total_bytes = encoded_bytes;
        self.max_vector_bytes = vector_bytes;
        Ok(self)
    }
    pub fn with_record_limits(mut self, limits: StorageLimits) -> Self {
        self.limits = limits;
        self
    }
    pub fn with_embedding_space(mut self, space: EmbeddingSpace) -> Result<Self, DatasetError> {
        validate_space(&space, self.max_dimensions)?;
        self.space = Some(space);
        Ok(self)
    }
    pub fn with_batch_size(mut self, size: usize) -> Result<Self, DatasetError> {
        if size == 0 {
            return Err(DatasetError::Configuration(
                "embedding batch size must be positive",
            ));
        }
        self.batch_size = size;
        Ok(self)
    }
    pub fn with_source_limit(mut self, count: usize) -> Result<Self, DatasetError> {
        if count == 0 {
            return Err(DatasetError::Configuration("source limit must be positive"));
        }
        self.max_sources_per_entry = count;
        Ok(self)
    }
    pub fn with_max_dimensions(mut self, count: usize) -> Result<Self, DatasetError> {
        if count == 0 {
            return Err(DatasetError::Configuration(
                "dimension limit must be positive",
            ));
        }
        if let Some(space) = &self.space {
            validate_space(space, count)?;
        }
        self.max_dimensions = count;
        Ok(self)
    }
    #[cfg(feature = "advanced-search")]
    pub fn with_text_analyzer(mut self, analyzer: crate::text_search::AnalyzerConfig) -> Self {
        self.analyzer = Some(analyzer);
        self
    }
    pub fn record_limits(&self) -> &StorageLimits {
        &self.limits
    }
    pub fn max_entries(&self) -> usize {
        self.max_entries
    }
    pub fn max_total_bytes(&self) -> usize {
        self.max_total_bytes
    }
    pub fn max_vector_bytes(&self) -> usize {
        self.max_vector_bytes
    }
    pub fn max_dimensions(&self) -> usize {
        self.max_dimensions
    }
    pub fn max_sources_per_entry(&self) -> usize {
        self.max_sources_per_entry
    }
    pub fn batch_size(&self) -> usize {
        self.batch_size
    }
    pub fn embedding_space(&self) -> Option<&EmbeddingSpace> {
        self.space.as_ref()
    }
}
fn validate_space(space: &EmbeddingSpace, max_dimensions: usize) -> Result<(), DatasetError> {
    if space.model().len() > 1024 || space.dimensions() > max_dimensions {
        return Err(DatasetError::Limit("embedding recipe/dimensions"));
    }
    Ok(())
}
#[derive(Debug, Serialize)]
pub struct DatasetRecord {
    entry: KnowledgeEntry,
    sources: Vec<SourceReference>,
}
impl DatasetRecord {
    pub fn new(entry: KnowledgeEntry) -> Self {
        Self {
            entry,
            sources: Vec::new(),
        }
    }
    pub fn with_sources(mut self, sources: Vec<SourceReference>) -> Self {
        self.sources = sources;
        self
    }
    pub fn entry(&self) -> &KnowledgeEntry {
        &self.entry
    }
    pub fn sources(&self) -> &[SourceReference] {
        &self.sources
    }
}
#[derive(Debug)]
#[non_exhaustive]
pub enum DatasetChange {
    Put(DatasetRecord),
    Delete(String),
}
#[derive(Debug)]
pub struct DatasetUpdate {
    changes: Vec<DatasetChange>,
    source_revision: Option<StoreRevision>,
}
impl DatasetUpdate {
    pub fn new(changes: Vec<DatasetChange>) -> Self {
        Self {
            changes,
            source_revision: None,
        }
    }
    pub fn with_source_revision(mut self, revision: StoreRevision) -> Self {
        self.source_revision = Some(revision);
        self
    }
}
#[derive(Debug, Clone, Default, Serialize)]
#[non_exhaustive]
pub struct PreparationStats {
    pub puts: usize,
    pub deleted: usize,
    pub embedded_entries: usize,
    pub reused_vectors: usize,
    pub text_entries_rebuilt: usize,
    pub text_entries_reused: usize,
    pub metadata_only_puts: usize,
    pub unchanged_puts: usize,
}
#[derive(Clone)]
enum Index {
    Lexical(Engine),
    Semantic(SemanticIndex),
}
impl Index {
    fn lexical(&self) -> &Engine {
        match self {
            Self::Lexical(i) => i,
            Self::Semantic(i) => i.lexical(),
        }
    }
    fn remove(&mut self, id: &str) {
        match self {
            Self::Lexical(i) => {
                i.remove(id);
            }
            Self::Semantic(i) => {
                i.remove(id);
            }
        }
    }
}
pub struct DatasetSnapshot {
    revision: StoreRevision,
    source_revision: Option<StoreRevision>,
    index: Index,
    sources: Arc<ProvenanceStore>,
    sizes: BTreeMap<String, usize>,
    total_bytes: usize,
    config: DatasetConfig,
    stats: PreparationStats,
    #[cfg(feature = "advanced-search")]
    text: Option<Arc<crate::text_search::TextIndex>>,
}
impl DatasetSnapshot {
    pub fn revision(&self) -> &StoreRevision {
        &self.revision
    }
    pub fn source_revision(&self) -> Option<&StoreRevision> {
        self.source_revision.as_ref()
    }
    pub fn lexical(&self) -> &Engine {
        self.index.lexical()
    }
    pub fn semantic(&self) -> Option<&SemanticIndex> {
        match &self.index {
            Index::Semantic(i) => Some(i),
            Index::Lexical(_) => None,
        }
    }
    pub fn provenance(&self) -> &ProvenanceStore {
        &self.sources
    }
    pub fn config(&self) -> &DatasetConfig {
        &self.config
    }
    pub fn encoded_record_bytes(&self) -> usize {
        self.total_bytes
    }
    pub fn stats(&self) -> &PreparationStats {
        &self.stats
    }
    #[cfg(feature = "advanced-search")]
    pub fn text_index(&self) -> Option<&crate::text_search::TextIndex> {
        self.text.as_deref()
    }
}
pub struct PreparedDataset {
    expected: StoreRevision,
    next: Arc<DatasetSnapshot>,
}
impl PreparedDataset {
    pub fn base_revision(&self) -> &StoreRevision {
        &self.expected
    }
    pub fn stats(&self) -> &PreparationStats {
        &self.next.stats
    }
}
#[derive(Clone)]
pub struct DatasetManager {
    current: Arc<RwLock<Arc<DatasetSnapshot>>>,
}
impl DatasetManager {
    pub fn new(namespace: &str, config: DatasetConfig) -> Result<Self, DatasetError> {
        let revision = StoreRevision::fresh(namespace)?;
        if let Some(space) = &config.space {
            validate_space(space, config.max_dimensions)?;
        }
        let index = match &config.space {
            Some(s) => Index::Semantic(SemanticIndex::new(s.clone())),
            None => Index::Lexical(Engine::new()),
        };
        #[cfg(feature = "advanced-search")]
        let text = config
            .analyzer
            .clone()
            .map(|c| Arc::new(crate::text_search::TextIndex::new(c)));
        Ok(Self {
            current: Arc::new(RwLock::new(Arc::new(DatasetSnapshot {
                revision,
                source_revision: None,
                index,
                sources: Arc::new(ProvenanceStore::new()),
                sizes: BTreeMap::new(),
                total_bytes: 0,
                config,
                stats: PreparationStats::default(),
                #[cfg(feature = "advanced-search")]
                text,
            }))),
        })
    }
    pub fn snapshot(&self) -> Result<Arc<DatasetSnapshot>, DatasetError> {
        Ok(self
            .current
            .read()
            .map_err(|_| DatasetError::Poisoned)?
            .clone())
    }
    pub fn require_current(&self, expected: &StoreRevision) -> Result<(), DatasetError> {
        if self.snapshot()?.revision != *expected {
            return Err(DatasetError::Conflict);
        }
        Ok(())
    }
    fn base(&self, expected: &StoreRevision) -> Result<Arc<DatasetSnapshot>, DatasetError> {
        let base = self.snapshot()?;
        if base.revision != *expected {
            return Err(DatasetError::Conflict);
        }
        Ok(base)
    }
    pub fn prepare(
        &self,
        expected: &StoreRevision,
        update: DatasetUpdate,
        provider: Option<&mut dyn EmbeddingProvider>,
    ) -> Result<PreparedDataset, DatasetError> {
        let base = self.base(expected)?;
        self.prepare_inner(
            base,
            update,
            provider,
            false,
            #[cfg(feature = "execution")]
            None,
        )
    }
    #[cfg(feature = "execution")]
    pub fn prepare_controlled(
        &self,
        expected: &StoreRevision,
        update: DatasetUpdate,
        provider: Option<&mut dyn EmbeddingProvider>,
        control: &crate::execution::WorkControl,
    ) -> Result<PreparedDataset, DatasetError> {
        control.checkpoint()?;
        self.prepare_inner(self.base(expected)?, update, provider, false, Some(control))
    }
    pub fn prepare_replace(
        &self,
        expected: &StoreRevision,
        records: Vec<DatasetRecord>,
        source_revision: Option<StoreRevision>,
        provider: Option<&mut dyn EmbeddingProvider>,
    ) -> Result<PreparedDataset, DatasetError> {
        self.prepare_replace_inner(
            expected,
            records,
            source_revision,
            provider,
            #[cfg(feature = "execution")]
            None,
        )
    }
    #[cfg(feature = "execution")]
    pub fn prepare_replace_controlled(
        &self,
        expected: &StoreRevision,
        records: Vec<DatasetRecord>,
        source_revision: Option<StoreRevision>,
        provider: Option<&mut dyn EmbeddingProvider>,
        control: &crate::execution::WorkControl,
    ) -> Result<PreparedDataset, DatasetError> {
        control.checkpoint()?;
        self.prepare_replace_inner(expected, records, source_revision, provider, Some(control))
    }
    fn prepare_replace_inner(
        &self,
        expected: &StoreRevision,
        records: Vec<DatasetRecord>,
        source_revision: Option<StoreRevision>,
        provider: Option<&mut dyn EmbeddingProvider>,
        #[cfg(feature = "execution")] control: Option<&crate::execution::WorkControl>,
    ) -> Result<PreparedDataset, DatasetError> {
        let base = self.base(expected)?;
        if records.len() > base.config.max_entries {
            return Err(DatasetError::Limit("entry count"));
        }
        let mut ids = BTreeSet::new();
        for record in &records {
            #[cfg(feature = "execution")]
            if let Some(control) = control {
                control.checkpoint()?;
            }
            validate_id(&record.entry.id, &base.config)?;
            if !ids.insert(record.entry.id.as_str()) {
                return Err(DatasetError::InvalidData("duplicate replacement ID"));
            }
        }
        let deletes: Vec<_> = base
            .lexical()
            .entries()
            .filter(|e| !ids.contains(e.id.as_str()))
            .map(|e| DatasetChange::Delete(e.id.clone()))
            .collect();
        drop(ids);
        let changes = records
            .into_iter()
            .map(DatasetChange::Put)
            .chain(deletes)
            .collect();
        self.prepare_inner(
            base,
            DatasetUpdate {
                changes,
                source_revision,
            },
            provider,
            true,
            #[cfg(feature = "execution")]
            control,
        )
    }
    fn prepare_inner(
        &self,
        base: Arc<DatasetSnapshot>,
        update: DatasetUpdate,
        provider: Option<&mut dyn EmbeddingProvider>,
        full: bool,
        #[cfg(feature = "execution")] control: Option<&crate::execution::WorkControl>,
    ) -> Result<PreparedDataset, DatasetError> {
        #[cfg(feature = "execution")]
        if let Some(control) = control {
            control.checkpoint()?;
        }
        let revision = base.revision.next()?;
        let DatasetUpdate {
            changes,
            source_revision,
        } = update;
        if changes.is_empty() && source_revision.is_none() {
            return Err(DatasetError::Configuration("empty update"));
        }
        validate_source_revision(base.source_revision.as_ref(), source_revision.as_ref())?;
        if !full && changes.len() > base.config.limits.max_batch_operations() {
            return Err(DatasetError::Limit("batch operation count"));
        }
        #[cfg(feature = "execution")]
        if let Some(control) = control {
            control.progress(
                crate::execution::WorkPhase::ValidatingDataset,
                0,
                Some(changes.len()),
            );
        }
        let mut ids = BTreeSet::new();
        let mut sizes = base.sizes.clone();
        let mut bytes = 0usize;
        let mut put_bytes = 0usize;
        let mut total = base.total_bytes;
        for change in &changes {
            #[cfg(feature = "execution")]
            if let Some(control) = control {
                control.checkpoint()?;
            }
            let (id, size) = match change {
                DatasetChange::Put(record) => (
                    &record.entry.id,
                    Some(validate_record(record, &base.config)?),
                ),
                DatasetChange::Delete(id) => {
                    validate_id(id, &base.config)?;
                    (id, None)
                }
            };
            if !ids.insert(id.as_str()) {
                return Err(DatasetError::InvalidData("duplicate change ID"));
            }
            bytes = bytes
                .checked_add(size.unwrap_or(id.len()))
                .ok_or(DatasetError::Limit("batch bytes"))?;
            if !full && bytes > base.config.limits.max_batch_bytes() {
                return Err(DatasetError::Limit("batch bytes"));
            }
            if let Some(old) = sizes.remove(id) {
                total -= old;
            }
            if let Some(size) = size {
                put_bytes = put_bytes
                    .checked_add(size)
                    .filter(|n| *n <= base.config.max_total_bytes)
                    .ok_or(DatasetError::Limit("total record bytes"))?;
                total = total
                    .checked_add(size)
                    .ok_or(DatasetError::Limit("total record bytes"))?;
                sizes.insert(id.clone(), size);
            }
        }
        if total > base.config.max_total_bytes {
            return Err(DatasetError::Limit("total record bytes"));
        }
        check_capacity(&base.config, sizes.len(), base.config.space.as_ref())?;
        #[cfg(feature = "execution")]
        if let Some(control) = control {
            control.progress(
                crate::execution::WorkPhase::ValidatingDataset,
                changes.len(),
                Some(changes.len()),
            );
            control.progress(
                crate::execution::WorkPhase::StagingDataset,
                0,
                Some(changes.len()),
            );
        }
        let mut sources = (*base.sources).clone();
        for id in &ids {
            #[cfg(feature = "execution")]
            if let Some(control) = control {
                control.checkpoint()?;
            }
            sources.remove_entry(id);
        }
        sources.prune_unreferenced();
        for change in &changes {
            #[cfg(feature = "execution")]
            if let Some(control) = control {
                control.checkpoint()?;
            }
            if let DatasetChange::Put(record) = change {
                sources.replace(&record.entry.id, record.sources.iter().cloned())?;
            }
        }
        sources.prune_unreferenced();
        let mut stats = PreparationStats::default();
        #[cfg(feature = "advanced-search")]
        let text = if let Some(old) = &base.text {
            let differs = changes.iter().any(|change| match change {
                DatasetChange::Delete(id) => old.get(id).is_some(),
                DatasetChange::Put(record) => !old
                    .get(&record.entry.id)
                    .is_some_and(|entry| crate::knowledge::entries_identical(entry, &record.entry)),
            });
            if !differs {
                stats.text_entries_reused = sizes.len();
                Some(old.clone())
            } else {
                let mut next = (**old).clone();
                #[cfg(feature = "execution")]
                let analysis_count = changes
                    .iter()
                    .filter(|change| match change {
                        DatasetChange::Put(record) => old
                            .get(&record.entry.id)
                            .is_none_or(|entry| entry.text != record.entry.text),
                        _ => false,
                    })
                    .count();
                #[cfg(feature = "execution")]
                if let Some(control) = control {
                    control.progress(
                        crate::execution::WorkPhase::AnalyzingText,
                        0,
                        Some(analysis_count),
                    );
                }
                for change in &changes {
                    #[cfg(feature = "execution")]
                    if let Some(control) = control {
                        control.checkpoint()?;
                    }
                    match change {
                        DatasetChange::Delete(id) => {
                            next.remove(id);
                        }
                        DatasetChange::Put(record) => {
                            let analyzed = old
                                .get(&record.entry.id)
                                .is_none_or(|entry| entry.text != record.entry.text);
                            if !old.get(&record.entry.id).is_some_and(|entry| {
                                crate::knowledge::entries_identical(entry, &record.entry)
                            }) {
                                next.upsert(record.entry.clone())?;
                            }
                            stats.text_entries_rebuilt += usize::from(analyzed);
                        }
                    }
                }
                #[cfg(feature = "execution")]
                if let Some(control) = control {
                    control.progress(
                        crate::execution::WorkPhase::AnalyzingText,
                        analysis_count,
                        Some(analysis_count),
                    );
                    control.checkpoint()?;
                }
                stats.text_entries_reused = sizes
                    .len()
                    .checked_sub(stats.text_entries_rebuilt)
                    .ok_or(DatasetError::InvalidData("text analysis count invariant"))?;
                Some(Arc::new(next))
            }
        } else {
            None
        };
        drop(ids);
        let mut index = base.index.clone();
        let mut embed = Vec::new();
        #[cfg(feature = "execution")]
        let change_count = changes.len();
        for change in changes {
            #[cfg(feature = "execution")]
            if let Some(control) = control {
                control.checkpoint()?;
            }
            match change {
                DatasetChange::Delete(id) => {
                    stats.deleted += usize::from(base.lexical().get(&id).is_some());
                    index.remove(&id);
                }
                DatasetChange::Put(record) => {
                    stats.puts += 1;
                    if let Some(old) = base.lexical().get(&record.entry.id) {
                        if crate::knowledge::entries_identical(old, &record.entry) {
                            stats.unchanged_puts += 1;
                        } else if old.text == record.entry.text {
                            stats.metadata_only_puts += 1;
                        }
                    }
                    match &mut index {
                        Index::Lexical(i) => {
                            i.upsert(record.entry)?;
                        }
                        Index::Semantic(i) => {
                            if base
                                .lexical()
                                .get(&record.entry.id)
                                .is_some_and(|old| old.text == record.entry.text)
                            {
                                if i.replace_unchanged_text(record.entry)?.is_none() {
                                    return Err(DatasetError::InvalidData(
                                        "unchanged vector/source invariant",
                                    ));
                                }
                            } else {
                                embed.push(record.entry);
                            }
                        }
                    }
                }
            }
        }
        #[cfg(feature = "execution")]
        if let Some(control) = control {
            control.progress(
                crate::execution::WorkPhase::StagingDataset,
                change_count,
                Some(change_count),
            );
        }
        if let Index::Semantic(index) = &mut index {
            stats.embedded_entries = embed.len();
            if !embed.is_empty() {
                let provider = provider.ok_or(DatasetError::MissingProvider)?;
                #[cfg(feature = "execution")]
                let mut wrapper;
                #[cfg(feature = "execution")]
                let provider = if let Some(control) = control {
                    wrapper = crate::semantic::ControlledProvider { provider, control };
                    &mut wrapper as &mut dyn EmbeddingProvider
                } else {
                    provider
                };
                index.upsert_batch(embed, provider, base.config.batch_size)?;
            }
            stats.reused_vectors = sizes.len() - stats.embedded_entries;
        }
        #[cfg(feature = "execution")]
        if let Some(control) = control {
            control.checkpoint()?;
        }
        Ok(PreparedDataset {
            expected: base.revision.clone(),
            next: Arc::new(DatasetSnapshot {
                revision,
                source_revision,
                index,
                sources: Arc::new(sources),
                sizes,
                total_bytes: total,
                config: base.config.clone(),
                stats,
                #[cfg(feature = "advanced-search")]
                text,
            }),
        })
    }
    pub fn prepare_model(
        &self,
        expected: &StoreRevision,
        provider: Option<&mut dyn EmbeddingProvider>,
    ) -> Result<PreparedDataset, DatasetError> {
        self.prepare_model_inner(
            expected,
            provider,
            #[cfg(feature = "execution")]
            None,
        )
    }
    #[cfg(feature = "execution")]
    pub fn prepare_model_controlled(
        &self,
        expected: &StoreRevision,
        provider: Option<&mut dyn EmbeddingProvider>,
        control: &crate::execution::WorkControl,
    ) -> Result<PreparedDataset, DatasetError> {
        control.checkpoint()?;
        self.prepare_model_inner(expected, provider, Some(control))
    }
    fn prepare_model_inner(
        &self,
        expected: &StoreRevision,
        provider: Option<&mut dyn EmbeddingProvider>,
        #[cfg(feature = "execution")] control: Option<&crate::execution::WorkControl>,
    ) -> Result<PreparedDataset, DatasetError> {
        #[cfg(feature = "execution")]
        if let Some(control) = control {
            control.checkpoint()?;
        }
        let base = self.base(expected)?;
        let revision = base.revision.next()?;
        let mut config = base.config.clone();
        let mut stats = PreparationStats::default();
        #[cfg(feature = "advanced-search")]
        {
            stats.text_entries_reused = base.text.as_ref().map_or(0, |text| text.len());
        }
        config.space = provider.as_ref().map(|p| p.space().clone());
        check_capacity(&config, base.lexical().len(), config.space.as_ref())?;
        let lexical = base.lexical().clone();
        let index = if let Some(provider) = provider {
            stats.embedded_entries = lexical.len();
            #[cfg(feature = "execution")]
            let mut wrapper;
            #[cfg(feature = "execution")]
            let provider = if let Some(control) = control {
                wrapper = crate::semantic::ControlledProvider { provider, control };
                &mut wrapper as &mut dyn EmbeddingProvider
            } else {
                provider
            };
            Index::Semantic(SemanticIndex::from_index(
                lexical,
                provider,
                config.batch_size,
            )?)
        } else {
            Index::Lexical(lexical)
        };
        #[cfg(feature = "execution")]
        if let Some(control) = control {
            control.checkpoint()?;
        }
        Ok(PreparedDataset {
            expected: base.revision.clone(),
            next: Arc::new(DatasetSnapshot {
                revision,
                source_revision: base.source_revision.clone(),
                index,
                sources: base.sources.clone(),
                sizes: base.sizes.clone(),
                total_bytes: base.total_bytes,
                config,
                stats,
                #[cfg(feature = "advanced-search")]
                text: base.text.clone(),
            }),
        })
    }
    pub fn publish(&self, prepared: PreparedDataset) -> Result<Arc<DatasetSnapshot>, DatasetError> {
        self.publish_inner(
            prepared,
            #[cfg(feature = "execution")]
            None,
        )
    }
    #[cfg(feature = "execution")]
    pub fn publish_controlled(
        &self,
        prepared: PreparedDataset,
        control: &crate::execution::WorkControl,
    ) -> Result<Arc<DatasetSnapshot>, DatasetError> {
        self.publish_inner(prepared, Some(control))
    }
    fn publish_inner(
        &self,
        prepared: PreparedDataset,
        #[cfg(feature = "execution")] control: Option<&crate::execution::WorkControl>,
    ) -> Result<Arc<DatasetSnapshot>, DatasetError> {
        #[cfg(feature = "execution")]
        if let Some(control) = control {
            control.checkpoint()?;
        }
        #[cfg(feature = "execution")]
        if let Some(control) = control {
            control.progress(crate::execution::WorkPhase::Publishing, 0, Some(1));
            control.checkpoint()?;
        }
        let mut current = self.current.write().map_err(|_| DatasetError::Poisoned)?;
        if current.revision != prepared.expected {
            drop(current);
            return Err(DatasetError::Conflict);
        }
        #[cfg(feature = "execution")]
        if let Some(control) = control {
            control.checkpoint()?;
        }
        let previous = std::mem::replace(&mut *current, prepared.next);
        let published = current.clone();
        drop(current);
        drop(previous);
        #[cfg(feature = "execution")]
        if let Some(control) = control {
            control.progress(crate::execution::WorkPhase::Publishing, 1, Some(1));
        }
        Ok(published)
    }
}
fn validate_source_revision(
    old: Option<&StoreRevision>,
    new: Option<&StoreRevision>,
) -> Result<(), DatasetError> {
    if let Some(old) = old {
        let new = new.ok_or(DatasetError::SourceRevision)?;
        if old.namespace() != new.namespace()
            || old.identity() != new.identity()
            || new.generation() <= old.generation()
        {
            return Err(DatasetError::SourceRevision);
        }
    }
    Ok(())
}
fn check_capacity(
    config: &DatasetConfig,
    count: usize,
    space: Option<&EmbeddingSpace>,
) -> Result<(), DatasetError> {
    if count > config.max_entries {
        return Err(DatasetError::Limit("entry count"));
    }
    if let Some(space) = space {
        validate_space(space, config.max_dimensions)?;
        count
            .checked_mul(space.dimensions())
            .and_then(|n| n.checked_mul(std::mem::size_of::<f32>()))
            .filter(|n| *n <= config.max_vector_bytes)
            .ok_or(DatasetError::Limit("vector bytes"))?;
    }
    Ok(())
}
fn validate_id(id: &str, config: &DatasetConfig) -> Result<(), DatasetError> {
    if id.len() > config.limits.max_id_bytes() || id.trim().is_empty() {
        return Err(DatasetError::InvalidData("invalid change ID"));
    }
    Ok(())
}
struct CountJson {
    bytes: usize,
    limit: usize,
    exceeded: bool,
}
impl Write for CountJson {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        if b.len() > self.limit.saturating_sub(self.bytes) {
            self.exceeded = true;
            return Err(io::Error::other("dataset encoded record limit"));
        }
        self.bytes += b.len();
        Ok(b.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn validate_record(record: &DatasetRecord, config: &DatasetConfig) -> Result<usize, DatasetError> {
    validate_id(&record.entry.id, config)?;
    encode_entry(&record.entry, &config.limits)?;
    if record.sources.len() > config.max_sources_per_entry {
        return Err(DatasetError::Limit("source bindings per entry"));
    }
    let mut seen = BTreeSet::new();
    for source in &record.sources {
        if !seen.insert(source.id()) {
            return Err(DatasetError::InvalidData("duplicate source binding"));
        }
        if let Some(metadata) = source.metadata() {
            validate_metadata(metadata, &config.limits)?;
        }
    }
    let mut writer = CountJson {
        bytes: 0,
        limit: config.limits.max_entry_bytes(),
        exceeded: false,
    };
    let encoded = serde_json::to_writer(&mut writer, record);
    if writer.exceeded {
        return Err(DatasetError::Limit("encoded record including sources"));
    }
    encoded?;
    Ok(writer.bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generation_exhaustion_does_not_replace_the_current_snapshot() {
        let manager = DatasetManager::new("exhausted", DatasetConfig::default()).unwrap();
        {
            let mut state = manager.current.write().unwrap();
            let view = Arc::get_mut(&mut state).unwrap();
            view.revision =
                StoreRevision::new("exhausted", view.revision.identity(), i64::MAX as u64).unwrap();
        }
        let base = manager.snapshot().unwrap();
        let result = manager.prepare(
            base.revision(),
            DatasetUpdate::new(vec![DatasetChange::Delete("missing".into())]),
            None,
        );
        assert!(matches!(
            result,
            Err(DatasetError::Storage(StorageError::GenerationExhausted))
        ));
        assert!(Arc::ptr_eq(&base, &manager.snapshot().unwrap()));
    }
    #[test]
    fn poisoned_publication_state_fails_closed() {
        let manager = DatasetManager::new("poisoned", DatasetConfig::default()).unwrap();
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = manager.current.write().unwrap();
            panic!("injected publication panic");
        }));
        assert!(matches!(manager.snapshot(), Err(DatasetError::Poisoned)));
    }
}
