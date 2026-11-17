use super::*;
use rusqlite::{params, Connection, OpenFlags, OptionalExtension, TransactionBehavior};
use std::{
    fs::{self, OpenOptions},
    path::Path,
    time::Duration,
};

const SCHEMA:&str="CREATE TABLE airust_store_format(id INTEGER PRIMARY KEY CHECK(id=1),version INTEGER NOT NULL);INSERT INTO airust_store_format VALUES(1,1);CREATE TABLE airust_store_state(namespace TEXT PRIMARY KEY COLLATE BINARY,identity TEXT NOT NULL,generation INTEGER NOT NULL CHECK(generation>=0));CREATE TABLE airust_store_entries(namespace TEXT NOT NULL,id TEXT NOT NULL COLLATE BINARY,record TEXT NOT NULL,PRIMARY KEY(namespace,id),FOREIGN KEY(namespace) REFERENCES airust_store_state(namespace) ON DELETE CASCADE);";

pub struct SqliteStore {
    connection: Connection,
    namespace: String,
    limits: StorageLimits,
    read_only: bool,
    durable: bool,
}
impl SqliteStore {
    pub fn open(path: impl AsRef<Path>, namespace: &str) -> Result<Self, StorageError> {
        Self::open_with_limits(path, namespace, StorageLimits::default())
    }
    pub fn open_with_limits(
        path: impl AsRef<Path>,
        namespace: &str,
        limits: StorageLimits,
    ) -> Result<Self, StorageError> {
        Self::open_file(path.as_ref(), namespace, limits, false)
    }
    pub fn open_read_only(path: impl AsRef<Path>, namespace: &str) -> Result<Self, StorageError> {
        Self::open_file(path.as_ref(), namespace, StorageLimits::default(), true)
    }
    fn open_file(
        path: &Path,
        namespace: &str,
        limits: StorageLimits,
        read_only: bool,
    ) -> Result<Self, StorageError> {
        validate_namespace(namespace)?;
        match fs::symlink_metadata(path) {
            Ok(meta) => {
                if !meta.is_file() || meta.file_type().is_symlink() {
                    return Err(StorageError::Configuration(
                        "database path must be a regular file, not a symlink",
                    ));
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound && !read_only => {
                let mut options = OpenOptions::new();
                options.read(true).write(true).create_new(true);
                #[cfg(unix)]
                {
                    use std::os::unix::fs::OpenOptionsExt;
                    options.mode(0o600);
                }
                match options.open(path) {
                    Ok(_) => {}
                    Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                        let meta = fs::symlink_metadata(path)?;
                        if !meta.is_file() || meta.file_type().is_symlink() {
                            return Err(StorageError::Configuration(
                                "database path changed to a non-regular file",
                            ));
                        }
                    }
                    Err(e) => return Err(e.into()),
                }
            }
            Err(error) => return Err(error.into()),
        }
        let path = path.canonicalize()?;
        let flags = if read_only {
            OpenFlags::SQLITE_OPEN_READ_ONLY
        } else {
            OpenFlags::SQLITE_OPEN_READ_WRITE
        };
        let connection =
            Connection::open_with_flags(path, flags | OpenFlags::SQLITE_OPEN_NO_MUTEX)?;
        Self::initialize(connection, namespace, limits, read_only, true)
    }
    pub fn in_memory(namespace: &str) -> Result<Self, StorageError> {
        Self::in_memory_with_limits(namespace, StorageLimits::default())
    }
    pub fn in_memory_with_limits(
        namespace: &str,
        limits: StorageLimits,
    ) -> Result<Self, StorageError> {
        Self::initialize(
            Connection::open_in_memory()?,
            namespace,
            limits,
            false,
            false,
        )
    }
    fn initialize(
        mut connection: Connection,
        namespace: &str,
        limits: StorageLimits,
        read_only: bool,
        durable: bool,
    ) -> Result<Self, StorageError> {
        validate_namespace(namespace)?;
        connection.busy_timeout(Duration::from_secs(5))?;
        let empty = check_schema(&connection)?;
        if read_only {
            if empty {
                return Err(StorageError::InvalidData(
                    "database has no AIRust storage schema",
                ));
            }
            read_revision(&connection, namespace)?;
        } else {
            let transaction =
                connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            if check_schema(&transaction)? {
                transaction.execute_batch(SCHEMA)?;
            }
            let exists: bool = transaction.query_row(
                "SELECT EXISTS(SELECT 1 FROM airust_store_state WHERE namespace=?1)",
                [namespace],
                |r| r.get(0),
            )?;
            if !exists {
                let token = StoreRevision::fresh(namespace)?;
                transaction.execute(
                    "INSERT INTO airust_store_state(namespace,identity,generation) VALUES(?1,?2,0)",
                    params![namespace, token.identity()],
                )?;
            }
            read_revision(&transaction, namespace)?;
            transaction.commit()?;
            connection.pragma_update(None, "foreign_keys", true)?;
            if durable {
                connection.pragma_update(None, "journal_mode", "WAL")?;
                connection.pragma_update(None, "synchronous", "FULL")?;
            }
        }
        Ok(Self {
            connection,
            namespace: namespace.to_string(),
            limits,
            read_only,
            durable,
        })
    }
    pub fn set_busy_timeout(&mut self, timeout: Duration) -> Result<(), StorageError> {
        self.connection.busy_timeout(timeout)?;
        Ok(())
    }
}
fn check_schema(connection: &Connection) -> Result<bool, StorageError> {
    let mut statement = connection.prepare(
        "SELECT type,name FROM sqlite_master WHERE name NOT LIKE 'sqlite_%' ORDER BY name",
    )?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    if rows.is_empty() {
        return Ok(true);
    }
    let expected = [
        "airust_store_entries",
        "airust_store_format",
        "airust_store_state",
    ];
    if rows.len() != 3
        || rows
            .iter()
            .zip(expected)
            .any(|((kind, name), expected)| kind != "table" || name != expected)
    {
        return Err(StorageError::InvalidData(
            "database is unrelated or has unsupported schema objects",
        ));
    }
    let versions = connection
        .prepare("SELECT id,version FROM airust_store_format")?
        .query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))?
        .collect::<Result<Vec<_>, _>>()?;
    if versions != [(1, 1)] {
        return Err(StorageError::InvalidData(
            "unsupported storage format version",
        ));
    }
    connection.prepare("SELECT namespace,identity,generation FROM airust_store_state LIMIT 0")?;
    connection.prepare("SELECT namespace,id,record FROM airust_store_entries LIMIT 0")?;
    Ok(false)
}
fn read_revision(connection: &Connection, namespace: &str) -> Result<StoreRevision, StorageError> {
    let row: Option<(String, i64)> = connection
        .query_row(
            "SELECT identity,generation FROM airust_store_state WHERE namespace=?1",
            [namespace],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    let (identity, generation) = row.ok_or(StorageError::InvalidData(
        "storage namespace does not exist",
    ))?;
    StoreRevision::new(
        namespace,
        identity,
        u64::try_from(generation)
            .map_err(|_| StorageError::InvalidData("negative storage generation"))?,
    )
}
fn record_from_row(
    row: &rusqlite::Row<'_>,
    limits: &StorageLimits,
) -> Result<(KnowledgeEntry, usize), StorageError> {
    let id_bytes: i64 = row.get(0)?;
    let bytes: i64 = row.get(2)?;
    let id_bytes = usize::try_from(id_bytes)
        .map_err(|_| StorageError::InvalidData("invalid row ID length"))?;
    let bytes =
        usize::try_from(bytes).map_err(|_| StorageError::InvalidData("invalid row length"))?;
    if id_bytes > limits.max_id_bytes || bytes > limits.max_entry_bytes {
        return Err(StorageError::Limit("stored record/ID bytes"));
    }
    let id: String = row.get(1)?;
    let encoded: String = row.get(3)?;
    Ok((decode_entry(&id, &encoded, limits)?, bytes))
}
impl StorageBackend for SqliteStore {
    fn limits(&self) -> &StorageLimits {
        &self.limits
    }
    fn is_durable(&self) -> bool {
        self.durable
    }
    fn is_read_only(&self) -> bool {
        self.read_only
    }
    fn revision(&mut self) -> Result<StoreRevision, StorageError> {
        read_revision(&self.connection, &self.namespace)
    }
    fn get(
        &mut self,
        expected: &StoreRevision,
        id: &str,
    ) -> Result<Option<KnowledgeEntry>, StorageError> {
        validate_id(id, &self.limits)?;
        let tx = self.connection.transaction()?;
        check_revision(&read_revision(&tx, &self.namespace)?, expected)?;
        let entry = {
            let mut statement=tx.prepare("SELECT length(CAST(id AS BLOB)),id,length(CAST(record AS BLOB)),record FROM airust_store_entries WHERE namespace=?1 AND id=?2")?;
            let mut rows = statement.query(params![self.namespace, id])?;
            rows.next()?
                .map(|row| record_from_row(row, &self.limits).map(|v| v.0))
                .transpose()?
        };
        tx.commit()?;
        Ok(entry)
    }
    fn page(
        &mut self,
        expected: &StoreRevision,
        after: Option<&str>,
        limit: usize,
    ) -> Result<StoragePage, StorageError> {
        check_page(after, limit, &self.limits)?;
        let sql_limit = i64::try_from(limit.saturating_add(1))
            .map_err(|_| StorageError::Configuration("page size exceeds SQLite integer range"))?;
        let tx = self.connection.transaction()?;
        check_revision(&read_revision(&tx, &self.namespace)?, expected)?;
        let mut entries = Vec::new();
        let mut bytes = 0usize;
        let mut more = false;
        {
            let mut statement=tx.prepare("SELECT length(CAST(id AS BLOB)),id,length(CAST(record AS BLOB)),record FROM airust_store_entries WHERE namespace=?1 AND id>?2 COLLATE BINARY ORDER BY id COLLATE BINARY LIMIT ?3")?;
            let mut rows =
                statement.query(params![self.namespace, after.unwrap_or(""), sql_limit])?;
            while let Some(row) = rows.next()? {
                if entries.len() == limit {
                    more = true;
                    break;
                }
                let size: i64 = row.get(2)?;
                let size = usize::try_from(size)
                    .map_err(|_| StorageError::InvalidData("invalid row length"))?;
                if size > self.limits.max_page_bytes.saturating_sub(bytes) {
                    if entries.is_empty() {
                        return Err(StorageError::Limit("one record exceeds page bytes"));
                    }
                    more = true;
                    break;
                }
                let (entry, size) = record_from_row(row, &self.limits)?;
                entries.push(entry);
                bytes += size;
            }
        }
        tx.commit()?;
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
        if self.read_only {
            return Err(StorageError::ReadOnly);
        }
        let prepared = prepare(changes, &self.limits)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let current = read_revision(&tx, &self.namespace)?;
        check_revision(&current, expected)?;
        if prepared.is_empty() {
            tx.commit()?;
            return Ok(StoreCommit {
                revision: current,
                upserts: 0,
                deleted: 0,
            });
        }
        let next = current.next()?;
        let (mut upserts, mut deleted) = (0, 0);
        for (id, value) in prepared {
            if let Some(value) = value {
                tx.execute("INSERT INTO airust_store_entries(namespace,id,record) VALUES(?1,?2,?3) ON CONFLICT(namespace,id) DO UPDATE SET record=excluded.record",params![self.namespace,id,value])?;
                upserts += 1;
            } else {
                deleted += tx.execute(
                    "DELETE FROM airust_store_entries WHERE namespace=?1 AND id=?2",
                    params![self.namespace, id],
                )?;
            }
        }
        tx.execute(
            "UPDATE airust_store_state SET generation=?1 WHERE namespace=?2",
            params![next.generation() as i64, self.namespace],
        )?;
        tx.commit()?;
        Ok(StoreCommit {
            revision: next,
            upserts,
            deleted,
        })
    }
    fn renew_identity(&mut self, expected: &StoreRevision) -> Result<StoreRevision, StorageError> {
        if self.read_only {
            return Err(StorageError::ReadOnly);
        }
        let next = StoreRevision::fresh(&self.namespace)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        check_revision(&read_revision(&tx, &self.namespace)?, expected)?;
        tx.execute(
            "UPDATE airust_store_state SET identity=?1,generation=0 WHERE namespace=?2",
            params![next.identity(), self.namespace],
        )?;
        tx.commit()?;
        Ok(next)
    }
}
