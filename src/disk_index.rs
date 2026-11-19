#![doc = include_str!("../docs/DISK-INDEX.md")]
use crate::{
    retrieval::{
        RankingEvidence, RankingKind, RetrievalBatch, RetrievalCandidate, RetrievalError,
        RetrievalOptions, RetrievalSource,
    },
    storage::{encode_entry, StorageLimits, StoreChange, StoreRevision},
    Context, KnowledgeEntry,
};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::Path, time::Duration};
use thiserror::Error;
const APP_ID: i64 = 0x41495246;
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DiskError {
    #[error("invalid disk-index configuration/data: {0}")]
    Invalid(&'static str),
    #[error("disk-index limit exceeded: {0}")]
    Limit(&'static str),
    #[error("disk-index revision conflict")]
    Conflict,
    #[error("scoped search scan limit exceeded; no partial result returned")]
    ScanLimit,
    #[cfg(feature = "execution")]
    #[error(transparent)]
    Execution(#[from] crate::execution::ExecutionError),
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Storage(#[from] crate::storage::StorageError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
#[derive(Debug, Clone)]
pub struct DiskOptions {
    max_entries: u64,
    max_encoded_bytes: u64,
    page_cache_kib: usize,
    limits: StorageLimits,
}
impl Default for DiskOptions {
    fn default() -> Self {
        Self {
            max_entries: 1000000,
            max_encoded_bytes: 4 * 1024 * 1024 * 1024,
            page_cache_kib: 2048,
            limits: StorageLimits::default(),
        }
    }
}
impl DiskOptions {
    pub fn new(entries: u64, bytes: u64, page_cache_kib: usize) -> Result<Self, DiskError> {
        if !(1..=10000000).contains(&entries)
            || bytes == 0
            || bytes > 64 * 1024 * 1024 * 1024
            || !(64..=65536).contains(&page_cache_kib)
        {
            return Err(DiskError::Invalid("disk limits"));
        }
        Ok(Self {
            max_entries: entries,
            max_encoded_bytes: bytes,
            page_cache_kib,
            ..Self::default()
        })
    }
}
#[derive(Debug, Clone, Default)]
pub struct DiskWork {
    #[cfg(feature = "execution")]
    control: Option<crate::execution::WorkControl>,
}
impl DiskWork {
    #[cfg(feature = "execution")]
    pub fn with_control(mut self, c: crate::execution::WorkControl) -> Self {
        self.control = Some(c);
        self
    }
    fn check(&self) -> Result<(), DiskError> {
        #[cfg(feature = "execution")]
        if let Some(c) = &self.control {
            c.checkpoint()?;
        }
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct DiskSearchOptions {
    limit: usize,
    max_scan: usize,
    context: Option<Context>,
    work: DiskWork,
}
impl DiskSearchOptions {
    pub fn new(limit: usize) -> Result<Self, DiskError> {
        if limit > 1000 {
            return Err(DiskError::Limit("hit count"));
        }
        Ok(Self {
            limit,
            max_scan: 100000,
            context: None,
            work: DiskWork::default(),
        })
    }
    pub fn with_context(mut self, c: Context) -> Self {
        self.context = Some(c);
        self
    }
    pub fn with_scan_limit(mut self, max_scan: usize) -> Result<Self, DiskError> {
        if !(1..=1000000).contains(&max_scan) {
            return Err(DiskError::Limit("scan count"));
        }
        self.max_scan = max_scan;
        Ok(self)
    }
    #[cfg(feature = "execution")]
    pub fn with_control(mut self, c: crate::execution::WorkControl) -> Self {
        self.work = self.work.with_control(c);
        self
    }
}
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct DiskHit {
    pub entry: KnowledgeEntry,
    pub score: f64,
    pub matched_terms: Vec<String>,
}
#[derive(Debug, Clone, Serialize)]
pub struct DiskResults {
    revision: StoreRevision,
    hits: Vec<DiskHit>,
}
impl DiskResults {
    pub fn revision(&self) -> &StoreRevision {
        &self.revision
    }
    pub fn hits(&self) -> &[DiskHit] {
        &self.hits
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct State {
    schema_version: u32,
    revision: StoreRevision,
    recipe: String,
    entries: u64,
    encoded_bytes: u64,
}
#[derive(Debug, Clone, Serialize)]
pub struct DiskPage {
    pub revision: StoreRevision,
    pub entries: Vec<KnowledgeEntry>,
    pub next_after: Option<String>,
}
pub struct DiskIndex {
    connection: Connection,
    options: DiskOptions,
    recipe: String,
}
impl DiskIndex {
    pub fn open(
        path: impl AsRef<Path>,
        namespace: &str,
        options: DiskOptions,
    ) -> Result<Self, DiskError> {
        let mut connection = Connection::open(path)?;
        connection.busy_timeout(Duration::from_millis(500))?;
        connection.execute_batch("PRAGMA trusted_schema=OFF;PRAGMA synchronous=FULL;")?;
        let recipe = format!(
            "airust-sqlite-fts5-v1/nfkc-lower-nfkc/{:?}-{:?}/sqlite-{}",
            unicode_normalization::UNICODE_VERSION,
            std::char::UNICODE_VERSION,
            rusqlite::version()
        );
        {
            let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let app: i64 = tx.query_row("PRAGMA application_id", [], |r| r.get(0))?;
            let objects: i64 = tx.query_row(
                "SELECT count(*) FROM sqlite_master WHERE name NOT LIKE 'sqlite_%'",
                [],
                |r| r.get(0),
            )?;
            if app == 0 && objects == 0 {
                let revision = StoreRevision::fresh(namespace)?;
                tx.execute_batch("CREATE TABLE meta(id INTEGER PRIMARY KEY CHECK(id=1),data TEXT NOT NULL);CREATE TABLE records(rowid INTEGER PRIMARY KEY,id TEXT UNIQUE NOT NULL,body TEXT NOT NULL);CREATE VIRTUAL TABLE search USING fts5(terms,tokenize='unicode61 remove_diacritics 0');")?;
                tx.execute_batch(&format!(
                    "PRAGMA application_id={APP_ID};PRAGMA user_version=1;"
                ))?;
                let state = State {
                    schema_version: 1,
                    revision,
                    recipe: recipe.clone(),
                    entries: 0,
                    encoded_bytes: 0,
                };
                tx.execute(
                    "INSERT INTO meta(id,data)VALUES(1,?1)",
                    [serde_json::to_string(&state)?],
                )?;
            } else {
                let version: i64 = tx.query_row("PRAGMA user_version", [], |r| r.get(0))?;
                if app != APP_ID || version != 1 {
                    return Err(DiskError::Invalid("unrelated database/schema"));
                }
                let unexpected:i64=tx.query_row("SELECT count(*) FROM sqlite_master WHERE name NOT LIKE 'sqlite_%' AND (type NOT IN ('table') OR name NOT IN ('meta','records','search','search_data','search_idx','search_content','search_docsize','search_config'))",[],|r|r.get(0))?;
                if unexpected != 0 {
                    return Err(DiskError::Invalid("unexpected schema objects"));
                }
                let state = read_state(&tx)?;
                if state.revision.namespace() != namespace || state.recipe != recipe {
                    return Err(DiskError::Invalid(
                        "namespace/analysis recipe mismatch; rebuild required",
                    ));
                }
                check_limits(&state, &options)?;
            }
            tx.commit()?;
        }
        connection.execute_batch(&format!(
            "PRAGMA cache_size=-{};PRAGMA mmap_size=0;PRAGMA temp_store=FILE;",
            options.page_cache_kib
        ))?;
        let mode: String = connection.query_row("PRAGMA journal_mode", [], |r| r.get(0))?;
        if mode.to_lowercase() != "delete" {
            return Err(DiskError::Invalid("rollback-journal mode required"));
        }
        Ok(Self {
            connection,
            options,
            recipe,
        })
    }
    pub fn recipe(&self) -> &str {
        &self.recipe
    }
    pub fn revision(&self) -> Result<StoreRevision, DiskError> {
        Ok(read_state(&self.connection)?.revision)
    }
    pub fn len(&self) -> Result<u64, DiskError> {
        Ok(read_state(&self.connection)?.entries)
    }
    pub fn is_empty(&self) -> Result<bool, DiskError> {
        Ok(self.len()? == 0)
    }
    fn hook(&mut self, work: &DiskWork) -> Result<(), DiskError> {
        work.check()?;
        #[cfg(feature = "execution")]
        if let Some(control) = &work.control {
            let control = control.clone();
            self.connection
                .progress_handler(1000, Some(move || control.checkpoint().is_err()))?;
        } else {
            self.connection.progress_handler(0, None::<fn() -> bool>)?;
        }
        Ok(())
    }
    fn clear_hook(&mut self) {
        #[cfg(feature = "execution")]
        let _ = self.connection.progress_handler(0, None::<fn() -> bool>);
    }
    pub fn apply(
        &mut self,
        expected: &StoreRevision,
        changes: &[StoreChange],
        work: &DiskWork,
    ) -> Result<StoreRevision, DiskError> {
        self.hook(work)?;
        let result = self.apply_inner(expected, changes, work);
        self.clear_hook();
        if result.is_err() {
            work.check()?;
        }
        result
    }
    fn apply_inner(
        &mut self,
        expected: &StoreRevision,
        changes: &[StoreChange],
        work: &DiskWork,
    ) -> Result<StoreRevision, DiskError> {
        if changes.len() > self.options.limits.max_batch_operations() {
            return Err(DiskError::Limit("batch count"));
        }
        let mut staged = Vec::new();
        let mut seen = BTreeSet::new();
        let mut bytes = 0usize;
        for change in changes {
            work.check()?;
            match change {
                StoreChange::Put(e) => {
                    let body = encode_entry(e, &self.options.limits)?;
                    bytes = bytes
                        .checked_add(body.len())
                        .ok_or(DiskError::Limit("batch bytes"))?;
                    if !seen.insert(e.id.clone()) {
                        return Err(DiskError::Invalid("duplicate batch ID"));
                    }
                    staged.push((e.id.clone(), Some((body, e.text.clone()))));
                }
                StoreChange::Delete(id) => {
                    if id.trim().is_empty() || id.len() > 4096 || !seen.insert(id.clone()) {
                        return Err(DiskError::Invalid("delete/batch ID"));
                    }
                    bytes = bytes
                        .checked_add(id.len())
                        .ok_or(DiskError::Limit("batch bytes"))?;
                    staged.push((id.clone(), None));
                }
            }
            if bytes > self.options.limits.max_batch_bytes() {
                return Err(DiskError::Limit("batch bytes"));
            }
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut state = read_state(&tx)?;
        check_limits(&state, &self.options)?;
        if &state.revision != expected {
            return Err(DiskError::Conflict);
        }
        if staged.is_empty() {
            return Ok(state.revision);
        }
        let revision = state.revision.next()?;
        for (id, entry) in staged {
            work.check()?;
            let old: Option<(i64, i64)> = tx
                .query_row(
                    "SELECT rowid,length(CAST(body AS BLOB))FROM records WHERE id=?1",
                    [&id],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional()?;
            if let Some((rowid, len)) = old {
                let len =
                    u64::try_from(len).map_err(|_| DiskError::Invalid("negative stored length"))?;
                if len > self.options.limits.max_entry_bytes() as u64 {
                    return Err(DiskError::Limit("stored row bytes"));
                }
                tx.execute("DELETE FROM search WHERE rowid=?1", [rowid])?;
                tx.execute("DELETE FROM records WHERE rowid=?1", [rowid])?;
                state.entries = state
                    .entries
                    .checked_sub(1)
                    .ok_or(DiskError::Invalid("entry counters"))?;
                state.encoded_bytes = state
                    .encoded_bytes
                    .checked_sub(len)
                    .ok_or(DiskError::Invalid("byte counters"))?;
            }
            if let Some((body, text)) = entry {
                state.entries = state
                    .entries
                    .checked_add(1)
                    .ok_or(DiskError::Limit("entries"))?;
                state.encoded_bytes = state
                    .encoded_bytes
                    .checked_add(body.len() as u64)
                    .ok_or(DiskError::Limit("encoded bytes"))?;
                check_limits(&state, &self.options)?;
                let terms = crate::text::normalize(&text);
                if terms.len() > 4 * 1024 * 1024 {
                    return Err(DiskError::Limit("normalized text bytes"));
                }
                tx.execute(
                    "INSERT INTO records(id,body)VALUES(?1,?2)",
                    params![id, body],
                )?;
                let rowid = tx.last_insert_rowid();
                tx.execute(
                    "INSERT INTO search(rowid,terms)VALUES(?1,?2)",
                    params![rowid, terms],
                )?;
            }
        }
        state.revision = revision.clone();
        tx.execute(
            "UPDATE meta SET data=?1 WHERE id=1",
            [serde_json::to_string(&state)?],
        )?;
        work.check()?;
        tx.commit()?;
        Ok(revision)
    }
    pub fn search(
        &mut self,
        query: &str,
        options: &DiskSearchOptions,
    ) -> Result<DiskResults, DiskError> {
        self.hook(&options.work)?;
        let result = self.search_inner(query, options);
        self.clear_hook();
        options.work.check()?;
        result
    }
    fn search_inner(
        &mut self,
        query: &str,
        options: &DiskSearchOptions,
    ) -> Result<DiskResults, DiskError> {
        options.work.check()?;
        if query.len() > 16384 {
            return Err(DiskError::Limit("query bytes"));
        }
        let terms = crate::text::unique_terms(query);
        if terms.len() > 64 {
            return Err(DiskError::Limit("query terms"));
        }
        let tx = self.connection.transaction()?;
        let state = read_state(&tx)?;
        check_limits(&state, &self.options)?;
        let mut hits = Vec::new();
        if terms.is_empty() || options.limit == 0 {
            return Ok(DiskResults {
                revision: state.revision,
                hits,
            });
        }
        let expression = terms
            .iter()
            .map(|s| format!("\"{}\"", s.replace('"', "\"\"")))
            .collect::<Vec<_>>()
            .join(" OR ");
        {
            let mut statement=tx.prepare("SELECT r.id,length(CAST(r.body AS BLOB)),r.body,bm25(search) FROM search JOIN records r ON r.rowid=search.rowid WHERE search MATCH ?1 ORDER BY bm25(search),r.id")?;
            let mut rows = statement.query([expression])?;
            let mut scanned = 0;
            while let Some(row) = rows.next()? {
                options.work.check()?;
                if scanned >= options.max_scan {
                    return Err(DiskError::ScanLimit);
                }
                scanned += 1;
                let id: String = row.get(0)?;
                let len = u64::try_from(row.get::<_, i64>(1)?)
                    .map_err(|_| DiskError::Invalid("negative stored length"))?;
                if id.len() > 4096 || len > self.options.limits.max_entry_bytes() as u64 {
                    return Err(DiskError::Limit("stored result bytes"));
                }
                let body: String = row.get(2)?;
                let entry: KnowledgeEntry = serde_json::from_str(&body)?;
                if entry.id != id {
                    return Err(DiskError::Invalid("stored ID mismatch"));
                }
                encode_entry(&entry, &self.options.limits)?;
                if options
                    .context
                    .as_ref()
                    .is_some_and(|c| !c.includes(&entry))
                {
                    continue;
                }
                let score = -row.get::<_, f64>(3)?;
                if !score.is_finite() {
                    return Err(DiskError::Invalid("nonfinite search score"));
                }
                let doc_terms = crate::text::unique_terms(&entry.text);
                let matched_terms = terms.intersection(&doc_terms).cloned().collect();
                hits.push(DiskHit {
                    entry,
                    score,
                    matched_terms,
                });
                if hits.len() == options.limit {
                    break;
                }
            }
        }
        options.work.check()?;
        tx.commit()?;
        Ok(DiskResults {
            revision: state.revision,
            hits,
        })
    }
    pub fn page(
        &mut self,
        expected: &StoreRevision,
        after: Option<&str>,
        limit: usize,
    ) -> Result<DiskPage, DiskError> {
        if !(1..=1000).contains(&limit)
            || after.is_some_and(|id| id.trim().is_empty() || id.len() > 4096)
        {
            return Err(DiskError::Limit("export page"));
        }
        let tx = self.connection.transaction()?;
        let state = read_state(&tx)?;
        check_limits(&state, &self.options)?;
        if &state.revision != expected {
            return Err(DiskError::Conflict);
        }
        let mut entries = Vec::new();
        let mut bytes = 0usize;
        let mut more = false;
        {
            let mut statement=tx.prepare("SELECT id,length(CAST(body AS BLOB)),body FROM records WHERE (?1 IS NULL OR id>?1) ORDER BY id LIMIT ?2")?;
            let mut rows = statement.query(params![after, (limit + 1) as i64])?;
            while let Some(row) = rows.next()? {
                if entries.len() == limit {
                    more = true;
                    break;
                }
                let id: String = row.get(0)?;
                let len = row.get::<_, i64>(1)?;
                if len < 0 || len as u64 > self.options.limits.max_entry_bytes() as u64 {
                    return Err(DiskError::Limit("export row bytes"));
                }
                if bytes + len as usize > self.options.limits.max_page_bytes() {
                    more = true;
                    break;
                }
                let body: String = row.get(2)?;
                let entry: KnowledgeEntry = serde_json::from_str(&body)?;
                encode_entry(&entry, &self.options.limits)?;
                if entry.id != id {
                    return Err(DiskError::Invalid("export ID mismatch"));
                }
                bytes += len as usize;
                entries.push(entry);
            }
        }
        let next_after = if more {
            entries.last().map(|e| e.id.clone())
        } else {
            None
        };
        tx.commit()?;
        Ok(DiskPage {
            revision: state.revision,
            entries,
            next_after,
        })
    }
    pub fn integrity_check(&mut self, work: &DiskWork) -> Result<(), DiskError> {
        self.hook(work)?;
        let result = self.integrity_inner(work);
        self.clear_hook();
        work.check()?;
        result
    }
    fn integrity_inner(&mut self, work: &DiskWork) -> Result<(), DiskError> {
        let tx = self.connection.transaction()?;
        let answer: String = tx.query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
        if answer != "ok" {
            return Err(DiskError::Invalid("SQLite integrity"));
        }
        tx.execute("INSERT INTO search(search)VALUES('integrity-check')", [])?;
        let state = read_state(&tx)?;
        check_limits(&state, &self.options)?;
        let mut count = 0u64;
        let mut bytes = 0u64;
        {
            let mut statement=tx.prepare("SELECT r.id,length(CAST(r.body AS BLOB)),r.body,s.terms,length(CAST(s.terms AS BLOB)) FROM records r LEFT JOIN search s ON s.rowid=r.rowid")?;
            let mut rows = statement.query([])?;
            while let Some(row) = rows.next()? {
                work.check()?;
                let id: String = row.get(0)?;
                let len = row.get::<_, i64>(1)?;
                if len < 0 || len as u64 > self.options.limits.max_entry_bytes() as u64 {
                    return Err(DiskError::Limit("integrity row bytes"));
                }
                let body: String = row.get(2)?;
                let term_bytes: Option<i64> = row.get(4)?;
                if term_bytes.is_none_or(|n| !(0..=4 * 1024 * 1024).contains(&n)) {
                    return Err(DiskError::Limit("integrity normalized bytes"));
                }
                let terms: Option<String> = row.get(3)?;
                let entry: KnowledgeEntry = serde_json::from_str(&body)?;
                encode_entry(&entry, &self.options.limits)?;
                if entry.id != id
                    || terms.as_deref() != Some(crate::text::normalize(&entry.text).as_str())
                {
                    return Err(DiskError::Invalid("record/FTS mismatch"));
                }
                count += 1;
                bytes = bytes
                    .checked_add(len as u64)
                    .ok_or(DiskError::Limit("integrity bytes"))?;
            }
        }
        let fts_count: i64 = tx.query_row("SELECT count(*) FROM search", [], |r| r.get(0))?;
        if count != state.entries
            || bytes != state.encoded_bytes
            || fts_count < 0
            || fts_count as u64 != count
        {
            return Err(DiskError::Invalid("index counter/FTS mismatch"));
        }
        work.check()?;
        tx.commit()?;
        Ok(())
    }
}
fn read_state(connection: &Connection) -> Result<State, DiskError> {
    let bytes: i64 = connection.query_row(
        "SELECT length(CAST(data AS BLOB)) FROM meta WHERE id=1",
        [],
        |r| r.get(0),
    )?;
    if !(1..=16384).contains(&bytes) {
        return Err(DiskError::Invalid("state bytes"));
    }
    let json: String =
        connection.query_row("SELECT data FROM meta WHERE id=1", [], |r| r.get(0))?;
    let state: State = serde_json::from_str(&json)?;
    if state.schema_version != 1 {
        return Err(DiskError::Invalid("state schema"));
    }
    Ok(state)
}
fn check_limits(state: &State, options: &DiskOptions) -> Result<(), DiskError> {
    if state.entries > options.max_entries || state.encoded_bytes > options.max_encoded_bytes {
        return Err(DiskError::Limit("index entries/encoded bytes"));
    }
    Ok(())
}
pub struct DiskSource<'a> {
    index: &'a mut DiskIndex,
    hits: Vec<DiskHit>,
    last_revision: Option<StoreRevision>,
}
impl<'a> DiskSource<'a> {
    pub fn new(index: &'a mut DiskIndex) -> Self {
        Self {
            index,
            hits: Vec::new(),
            last_revision: None,
        }
    }
    pub fn last_revision(&self) -> Option<&StoreRevision> {
        self.last_revision.as_ref()
    }
}
impl RetrievalSource for DiskSource<'_> {
    fn retrieve<'a>(
        &'a mut self,
        query: &str,
        options: &RetrievalOptions,
    ) -> Result<RetrievalBatch<'a>, RetrievalError> {
        self.hits.clear();
        self.last_revision = None;
        let mut search = DiskSearchOptions::new(options.candidates()).map_err(map_error)?;
        if let Some(context) = options.context() {
            search = search.with_context(context.clone());
        }
        #[cfg(feature = "execution")]
        if let Some(control) = options.control() {
            search = search.with_control(control.clone());
        }
        let result = self.index.search(query, &search).map_err(map_error)?;
        self.last_revision = Some(result.revision);
        self.hits = result.hits;
        let candidates = self
            .hits
            .iter()
            .map(|h| {
                let mut evidence = RankingEvidence::new(RankingKind::Bm25, h.score);
                evidence.matched_terms = h.matched_terms.clone();
                RetrievalCandidate {
                    entry: &h.entry,
                    evidence,
                }
            })
            .collect();
        Ok(RetrievalBatch::new(self.index.recipe(), candidates))
    }
}
fn map_error(error: DiskError) -> RetrievalError {
    match error {
        #[cfg(feature = "execution")]
        DiskError::Execution(error) => RetrievalError::Execution(error),
        DiskError::Limit(_) | DiskError::ScanLimit => RetrievalError::Budget("disk backend"),
        _ => RetrievalError::Source(error.to_string()),
    }
}
