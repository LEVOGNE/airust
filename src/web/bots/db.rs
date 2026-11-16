use super::models::{ApprovedData, Bot, BotConfig, BotRun, RawData};
use rusqlite::params;

impl crate::web::db::Database {
    // ── Bots CRUD ─────────────────────────────────────────────────────────────

    pub fn create_bot(
        &self,
        name: &str,
        bot_type: &str,
        config: &BotConfig,
    ) -> Result<i64, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let config_json =
            serde_json::to_string(config).map_err(|e| format!("JSON error: {}", e))?;
        conn.execute(
            "INSERT INTO bots (name, bot_type, config) VALUES (?1, ?2, ?3)",
            params![name, bot_type, config_json],
        )
        .map_err(|e| format!("Create bot error: {}", e))?;
        Ok(conn.last_insert_rowid())
    }

    pub fn list_bots(&self) -> Result<Vec<Bot>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let mut stmt = conn
            .prepare("SELECT id, name, bot_type, config, enabled, created_at FROM bots ORDER BY id")
            .map_err(|e| format!("Query error: {}", e))?;
        let rows = stmt
            .query_map([], |row| {
                let config_str: String = row.get(3)?;
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    config_str,
                    row.get::<_, i32>(4)?,
                    row.get::<_, String>(5)?,
                ))
            })
            .map_err(|e| format!("Query error: {}", e))?;
        let mut result = Vec::new();
        for r in rows {
            let (id, name, bot_type, config_str, enabled, created_at) =
                r.map_err(|e| format!("Row error: {}", e))?;
            let config: BotConfig = serde_json::from_str(&config_str).unwrap_or_default();
            result.push(Bot {
                id,
                name,
                bot_type,
                config,
                enabled: enabled != 0,
                created_at,
            });
        }
        Ok(result)
    }

    pub fn get_bot(&self, id: i64) -> Result<Option<Bot>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let result = conn.query_row(
            "SELECT id, name, bot_type, config, enabled, created_at FROM bots WHERE id = ?1",
            params![id],
            |row| {
                let config_str: String = row.get(3)?;
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    config_str,
                    row.get::<_, i32>(4)?,
                    row.get::<_, String>(5)?,
                ))
            },
        );
        match result {
            Ok((id, name, bot_type, config_str, enabled, created_at)) => {
                let config: BotConfig = serde_json::from_str(&config_str).unwrap_or_default();
                Ok(Some(Bot {
                    id,
                    name,
                    bot_type,
                    config,
                    enabled: enabled != 0,
                    created_at,
                }))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(format!("Get bot error: {}", e)),
        }
    }

    pub fn update_bot(
        &self,
        id: i64,
        name: &str,
        bot_type: &str,
        config: &BotConfig,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let config_json =
            serde_json::to_string(config).map_err(|e| format!("JSON error: {}", e))?;
        conn.execute(
            "UPDATE bots SET name = ?2, bot_type = ?3, config = ?4 WHERE id = ?1",
            params![id, name, bot_type, config_json],
        )
        .map_err(|e| format!("Update bot error: {}", e))?;
        Ok(())
    }

    pub fn delete_bot(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        // Delete dependent data first (foreign key constraints)
        conn.execute(
            "DELETE FROM raw_data WHERE bot_run_id IN (SELECT id FROM bot_runs WHERE bot_id = ?1)",
            params![id],
        )
        .map_err(|e| format!("Delete raw_data error: {}", e))?;
        conn.execute("DELETE FROM bot_runs WHERE bot_id = ?1", params![id])
            .map_err(|e| format!("Delete bot_runs error: {}", e))?;
        conn.execute("DELETE FROM bots WHERE id = ?1", params![id])
            .map_err(|e| format!("Delete bot error: {}", e))?;
        Ok(())
    }

    // ── Bot Runs ──────────────────────────────────────────────────────────────

    pub fn create_bot_run(&self, bot_id: i64) -> Result<i64, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute(
            "INSERT INTO bot_runs (bot_id, status) VALUES (?1, 'running')",
            params![bot_id],
        )
        .map_err(|e| format!("Create run error: {}", e))?;
        Ok(conn.last_insert_rowid())
    }

    pub fn finish_bot_run(
        &self,
        run_id: i64,
        status: &str,
        items_found: i64,
        items_added: i64,
        error_msg: Option<&str>,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute(
            "UPDATE bot_runs SET status = ?2, finished_at = datetime('now'), items_found = ?3, items_added = ?4, error_msg = ?5 WHERE id = ?1",
            params![run_id, status, items_found, items_added, error_msg],
        )
        .map_err(|e| format!("Finish run error: {}", e))?;
        Ok(())
    }

    pub fn get_bot_runs(&self, bot_id: i64) -> Result<Vec<BotRun>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let mut stmt = conn
            .prepare("SELECT id, bot_id, status, started_at, finished_at, items_found, items_added, error_msg FROM bot_runs WHERE bot_id = ?1 ORDER BY id DESC LIMIT 20")
            .map_err(|e| format!("Query error: {}", e))?;
        let rows = stmt
            .query_map(params![bot_id], |row| {
                Ok(BotRun {
                    id: row.get(0)?,
                    bot_id: row.get(1)?,
                    status: row.get(2)?,
                    started_at: row.get(3)?,
                    finished_at: row.get(4)?,
                    items_found: row.get(5)?,
                    items_added: row.get(6)?,
                    error_msg: row.get(7)?,
                })
            })
            .map_err(|e| format!("Query error: {}", e))?;
        let mut result = Vec::new();
        for r in rows {
            result.push(r.map_err(|e| format!("Row error: {}", e))?);
        }
        Ok(result)
    }

    // ── Raw Data ──────────────────────────────────────────────────────────────

    pub fn insert_raw_data(
        &self,
        bot_run_id: i64,
        url: Option<&str>,
        title: Option<&str>,
        content: &str,
        content_hash: &str,
    ) -> Result<i64, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        // Check for duplicate hash
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM raw_data WHERE content_hash = ?1",
                params![content_hash],
                |row| row.get(0),
            )
            .map_err(|e| format!("Hash check error: {}", e))?;
        if exists {
            return Ok(-1); // Duplicate
        }
        conn.execute(
            "INSERT INTO raw_data (bot_run_id, url, title, content, content_hash) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![bot_run_id, url, title, content, content_hash],
        )
        .map_err(|e| format!("Insert raw data error: {}", e))?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_raw_data_by_run_id(&self, run_id: i64) -> Result<Vec<RawData>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let mut stmt = conn
            .prepare("SELECT id, bot_run_id, url, title, content, content_hash, status, created_at FROM raw_data WHERE bot_run_id = ?1 ORDER BY id")
            .map_err(|e| format!("Query error: {}", e))?;
        let rows = stmt
            .query_map(params![run_id], |row| {
                Ok(RawData {
                    id: row.get(0)?,
                    bot_run_id: row.get(1)?,
                    url: row.get(2)?,
                    title: row.get(3)?,
                    content: row.get(4)?,
                    content_hash: row.get(5)?,
                    status: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })
            .map_err(|e| format!("Query error: {}", e))?;
        let mut result = Vec::new();
        for r in rows {
            result.push(r.map_err(|e| format!("Row error: {}", e))?);
        }
        Ok(result)
    }

    pub fn get_pending_raw_data(&self) -> Result<Vec<RawData>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let mut stmt = conn
            .prepare("SELECT id, bot_run_id, url, title, content, content_hash, status, created_at FROM raw_data WHERE status = 'pending' ORDER BY id DESC")
            .map_err(|e| format!("Query error: {}", e))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(RawData {
                    id: row.get(0)?,
                    bot_run_id: row.get(1)?,
                    url: row.get(2)?,
                    title: row.get(3)?,
                    content: row.get(4)?,
                    content_hash: row.get(5)?,
                    status: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })
            .map_err(|e| format!("Query error: {}", e))?;
        let mut result = Vec::new();
        for r in rows {
            result.push(r.map_err(|e| format!("Row error: {}", e))?);
        }
        Ok(result)
    }

    pub fn approve_raw_data(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute(
            "UPDATE raw_data SET status = 'approved' WHERE id = ?1",
            params![id],
        )
        .map_err(|e| format!("Approve error: {}", e))?;
        Ok(())
    }

    pub fn reject_raw_data(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute(
            "UPDATE raw_data SET status = 'rejected' WHERE id = ?1",
            params![id],
        )
        .map_err(|e| format!("Reject error: {}", e))?;
        Ok(())
    }

    pub fn approve_all_raw_data(&self) -> Result<u64, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let count = conn
            .execute(
                "UPDATE raw_data SET status = 'approved' WHERE status = 'pending'",
                [],
            )
            .map_err(|e| format!("Approve all error: {}", e))?;
        Ok(count as u64)
    }

    pub fn get_raw_data_by_id(&self, id: i64) -> Result<Option<RawData>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let result = conn.query_row(
            "SELECT id, bot_run_id, url, title, content, content_hash, status, created_at FROM raw_data WHERE id = ?1",
            params![id],
            |row| {
                Ok(RawData {
                    id: row.get(0)?,
                    bot_run_id: row.get(1)?,
                    url: row.get(2)?,
                    title: row.get(3)?,
                    content: row.get(4)?,
                    content_hash: row.get(5)?,
                    status: row.get(6)?,
                    created_at: row.get(7)?,
                })
            },
        );
        match result {
            Ok(data) => Ok(Some(data)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(format!("Get raw data error: {}", e)),
        }
    }

    // ── Approved Data ─────────────────────────────────────────────────────────

    pub fn insert_approved_data(
        &self,
        raw_data_id: Option<i64>,
        input: &str,
        output: &str,
        weight: f64,
        source_url: Option<&str>,
    ) -> Result<i64, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute(
            "INSERT INTO approved_data (raw_data_id, input, output, weight, source_url) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![raw_data_id, input, output, weight, source_url],
        )
        .map_err(|e| format!("Insert approved data error: {}", e))?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_approved_data(&self) -> Result<Vec<ApprovedData>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let mut stmt = conn
            .prepare("SELECT id, raw_data_id, input, output, weight, source_url, added_to_kb, created_at FROM approved_data ORDER BY id DESC")
            .map_err(|e| format!("Query error: {}", e))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(ApprovedData {
                    id: row.get(0)?,
                    raw_data_id: row.get(1)?,
                    input: row.get(2)?,
                    output: row.get(3)?,
                    weight: row.get(4)?,
                    source_url: row.get(5)?,
                    added_to_kb: row.get::<_, i32>(6)? != 0,
                    created_at: row.get(7)?,
                })
            })
            .map_err(|e| format!("Query error: {}", e))?;
        let mut result = Vec::new();
        for r in rows {
            result.push(r.map_err(|e| format!("Row error: {}", e))?);
        }
        Ok(result)
    }

    pub fn get_unadded_approved_data(&self) -> Result<Vec<ApprovedData>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let mut stmt = conn
            .prepare("SELECT id, raw_data_id, input, output, weight, source_url, added_to_kb, created_at FROM approved_data WHERE added_to_kb = 0")
            .map_err(|e| format!("Query error: {}", e))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(ApprovedData {
                    id: row.get(0)?,
                    raw_data_id: row.get(1)?,
                    input: row.get(2)?,
                    output: row.get(3)?,
                    weight: row.get(4)?,
                    source_url: row.get(5)?,
                    added_to_kb: row.get::<_, i32>(6)? != 0,
                    created_at: row.get(7)?,
                })
            })
            .map_err(|e| format!("Query error: {}", e))?;
        let mut result = Vec::new();
        for r in rows {
            result.push(r.map_err(|e| format!("Row error: {}", e))?);
        }
        Ok(result)
    }

    pub fn mark_added_to_kb(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute(
            "UPDATE approved_data SET added_to_kb = 1 WHERE id = ?1",
            params![id],
        )
        .map_err(|e| format!("Mark added error: {}", e))?;
        Ok(())
    }

    // ── Vectors ───────────────────────────────────────────────────────────────

    pub fn insert_vector(
        &self,
        source_id: i64,
        term: &str,
        tfidf_score: f64,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute(
            "INSERT INTO vectors (source_id, term, tfidf_score) VALUES (?1, ?2, ?3)",
            params![source_id, term, tfidf_score],
        )
        .map_err(|e| format!("Insert vector error: {}", e))?;
        Ok(())
    }

    pub fn clear_vectors_for_source(&self, source_id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute(
            "DELETE FROM vectors WHERE source_id = ?1",
            params![source_id],
        )
        .map_err(|e| format!("Clear vectors error: {}", e))?;
        Ok(())
    }

    pub fn clear_all_vectors(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute("DELETE FROM vectors", [])
            .map_err(|e| format!("Clear all vectors error: {}", e))?;
        Ok(())
    }

    pub fn get_vector_stats(&self) -> Result<(i64, i64), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let total: i64 = conn
            .query_row("SELECT COUNT(*) FROM vectors", [], |r| r.get(0))
            .map_err(|e| format!("Count error: {}", e))?;
        let terms: i64 = conn
            .query_row("SELECT COUNT(DISTINCT term) FROM vectors", [], |r| r.get(0))
            .map_err(|e| format!("Count error: {}", e))?;
        Ok((total, terms))
    }

    pub fn search_vectors(
        &self,
        terms: &[String],
        top_k: usize,
    ) -> Result<Vec<(i64, f64)>, String> {
        if terms.is_empty() {
            return Ok(Vec::new());
        }
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let placeholders: Vec<String> = terms
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", i + 1))
            .collect();
        let sql = format!(
            "SELECT source_id, SUM(tfidf_score) as score FROM vectors WHERE term IN ({}) GROUP BY source_id ORDER BY score DESC LIMIT ?{}",
            placeholders.join(","),
            terms.len() + 1
        );
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| format!("Query error: {}", e))?;
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = terms
            .iter()
            .map(|t| Box::new(t.clone()) as Box<dyn rusqlite::types::ToSql>)
            .collect();
        param_values.push(Box::new(top_k as i64));
        let refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        let rows = stmt
            .query_map(refs.as_slice(), |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| format!("Query error: {}", e))?;
        let mut result = Vec::new();
        for r in rows {
            result.push(r.map_err(|e| format!("Row error: {}", e))?);
        }
        Ok(result)
    }

    pub fn get_approved_data_by_id(&self, id: i64) -> Result<Option<ApprovedData>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let result = conn.query_row(
            "SELECT id, raw_data_id, input, output, weight, source_url, added_to_kb, created_at FROM approved_data WHERE id = ?1",
            params![id],
            |row| {
                Ok(ApprovedData {
                    id: row.get(0)?,
                    raw_data_id: row.get(1)?,
                    input: row.get(2)?,
                    output: row.get(3)?,
                    weight: row.get(4)?,
                    source_url: row.get(5)?,
                    added_to_kb: row.get::<_, i32>(6)? != 0,
                    created_at: row.get(7)?,
                })
            },
        );
        match result {
            Ok(data) => Ok(Some(data)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(format!("Get approved data error: {}", e)),
        }
    }
}
