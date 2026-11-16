use crate::{ResponseFormat, TrainingExample};
use rusqlite::{params, Connection};
use std::sync::Mutex;

pub struct Database {
    pub(crate) conn: Mutex<Connection>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ChatRow {
    pub id: i64,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub archived: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct MessageRow {
    pub id: i64,
    pub chat_id: i64,
    pub role: String,
    pub content: String,
    pub confidence: Option<f64>,
    pub timestamp: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CategoryRow {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub description: String,
    pub created_at: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct TrainingDataRow {
    pub id: i64,
    pub category_id: Option<i64>,
    pub input: String,
    pub output_text: String,
    pub output_format: String,
    pub weight: f64,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl TrainingDataRow {
    pub fn into_example(self) -> TrainingExample {
        TrainingExample {
            input: self.input,
            output: decode_response(self.output_text, &self.output_format),
            weight: self.weight as f32,
            metadata: self.metadata,
        }
    }
}

pub(crate) fn decode_response(output: String, format: &str) -> ResponseFormat {
    match format {
        "markdown" => ResponseFormat::Markdown(output),
        "json" => ResponseFormat::Json(
            serde_json::from_str(&output).unwrap_or(serde_json::Value::String(output)),
        ),
        _ => ResponseFormat::Text(output),
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct VectorCollectionRow {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub created_at: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct VectorEntryRow {
    pub id: i64,
    pub collection_id: i64,
    pub content: String,
    pub metadata_json: String,
    pub embedding_json: String,
    pub created_at: String,
}

impl Database {
    pub fn open(path: &str) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| format!("DB open error: {}", e))?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.init_tables()?;
        db.seed_defaults()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS translations (
                key TEXT NOT NULL,
                language_code TEXT NOT NULL,
                text TEXT NOT NULL,
                PRIMARY KEY (key, language_code)
            );
            CREATE TABLE IF NOT EXISTS chats (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                archived INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                chat_id INTEGER NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                confidence REAL,
                timestamp TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (chat_id) REFERENCES chats(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS bots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                bot_type TEXT NOT NULL,
                config TEXT NOT NULL DEFAULT '{}',
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at TEXT DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS bot_runs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                bot_id INTEGER NOT NULL,
                status TEXT NOT NULL,
                started_at TEXT DEFAULT (datetime('now')),
                finished_at TEXT,
                items_found INTEGER DEFAULT 0,
                items_added INTEGER DEFAULT 0,
                error_msg TEXT,
                FOREIGN KEY (bot_id) REFERENCES bots(id)
            );

            CREATE TABLE IF NOT EXISTS raw_data (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                bot_run_id INTEGER NOT NULL,
                url TEXT,
                title TEXT,
                content TEXT NOT NULL,
                content_hash TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                created_at TEXT DEFAULT (datetime('now')),
                FOREIGN KEY (bot_run_id) REFERENCES bot_runs(id)
            );
            CREATE INDEX IF NOT EXISTS idx_raw_data_hash ON raw_data(content_hash);
            CREATE INDEX IF NOT EXISTS idx_raw_data_status ON raw_data(status);

            CREATE TABLE IF NOT EXISTS approved_data (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                raw_data_id INTEGER,
                input TEXT NOT NULL,
                output TEXT NOT NULL,
                weight REAL NOT NULL DEFAULT 1.0,
                source_url TEXT,
                added_to_kb INTEGER NOT NULL DEFAULT 0,
                created_at TEXT DEFAULT (datetime('now')),
                FOREIGN KEY (raw_data_id) REFERENCES raw_data(id)
            );

            CREATE TABLE IF NOT EXISTS vectors (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source_id INTEGER NOT NULL,
                term TEXT NOT NULL,
                tfidf_score REAL NOT NULL,
                FOREIGN KEY (source_id) REFERENCES approved_data(id)
            );
            CREATE INDEX IF NOT EXISTS idx_vectors_term ON vectors(term);

            CREATE TABLE IF NOT EXISTS categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                color TEXT NOT NULL DEFAULT '#7c6ef0',
                description TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS training_data (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                category_id INTEGER,
                input TEXT NOT NULL,
                output_text TEXT NOT NULL,
                output_format TEXT NOT NULL DEFAULT 'text',
                weight REAL NOT NULL DEFAULT 1.0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE SET NULL
            );

            CREATE TABLE IF NOT EXISTS vector_collections (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS vector_entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                collection_id INTEGER NOT NULL,
                content TEXT NOT NULL,
                metadata_json TEXT NOT NULL DEFAULT '{}',
                embedding_json TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (collection_id) REFERENCES vector_collections(id) ON DELETE CASCADE
            );
            ",
        )
        .map_err(|e| format!("Init tables error: {}", e))?;
        let has_metadata: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM pragma_table_info('training_data') WHERE name = 'metadata_json')", [], |row| row.get(0)
        ).map_err(|error| format!("Training metadata migration check: {error}"))?;
        if !has_metadata {
            conn.execute(
                "ALTER TABLE training_data ADD COLUMN metadata_json TEXT",
                [],
            )
            .map_err(|error| format!("Training metadata migration: {error}"))?;
        }
        Ok(())
    }

    fn seed_defaults(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        let defaults = [
            ("theme", "dark"),
            ("language", "en"),
            ("font_size", "13"),
            ("dark.accent_color", "#ffd6d6"),
            ("dark.bg_color", "#002929"),
            ("dark.bg_auto", "true"),
            ("dark.text_color", "#e4e4f0"),
            ("dark.text_auto", "true"),
            ("light.accent_color", "#3b3a45"),
            ("light.bg_color", "#c4c5ba"),
            ("light.bg_auto", "true"),
            ("light.text_color", "#1a1b2e"),
            ("light.text_auto", "true"),
        ];
        for (k, v) in &defaults {
            conn.execute(
                "INSERT OR IGNORE INTO settings (key, value) VALUES (?1, ?2)",
                params![k, v],
            )
            .map_err(|e| format!("Seed settings error: {}", e))?;
        }

        Self::seed_translations(&conn)?;

        Self::seed_new_translations(&conn)?;
        Self::seed_landing_translations(&conn)?;
        Self::refresh_product_text(&conn)?;

        Ok(())
    }

    fn seed_translations(conn: &Connection) -> Result<(), String> {
        let translations: Vec<(&str, &str, &str)> = vec![
            ("agent.tfidf.name", "en", "Smart Search"),
            (
                "agent.tfidf.desc",
                "en",
                "Ranks answers by shared words (BM25)",
            ),
            ("agent.exact.name", "en", "Word-for-Word"),
            ("agent.exact.desc", "en", "Must match exactly, ignores caps"),
            ("agent.fuzzy.name", "en", "Close Enough"),
            (
                "agent.fuzzy.desc",
                "en",
                "Also finds typos and similar words",
            ),
            ("agent.context.name", "en", "Remembers Chat"),
            (
                "agent.context.desc",
                "en",
                "Uses previous messages for better answers",
            ),
            ("kb.search", "en", "Search..."),
            ("nav.agent", "en", "Assistant"),
            ("nav.chat", "en", "Chat"),
            ("nav.knowledge", "en", "Brain"),
            ("nav.settings", "en", "Settings"),
            ("nav.guide", "en", "Guide"),
            ("nav.chats", "en", "Chats"),
            ("chat.placeholder", "en", "Ask a question..."),
            ("chat.send", "en", "Send"),
            ("chat.new", "en", "New Chat"),
            ("chat.archive", "en", "Archive"),
            ("chat.delete", "en", "Delete"),
            ("chat.show_archive", "en", "Show Archive"),
            ("chat.hide_archive", "en", "Hide Archive"),
            ("chat.confidence", "en", "Confidence"),
            ("kb.add", "en", "Teach Something"),
            ("kb.input", "en", "Question"),
            ("kb.output", "en", "Answer"),
            ("kb.weight", "en", "Weight"),
            ("kb.save", "en", "Save Brain"),
            ("kb.load", "en", "Load Brain"),
            (
                "kb.path_placeholder",
                "en",
                "Path (e.g. ./knowledge/my.json)",
            ),
            ("pdf.title", "en", "Upload PDF"),
            ("pdf.drop", "en", "Drop PDF here or click"),
            ("settings.theme", "en", "Appearance"),
            ("settings.dark", "en", "Dark"),
            ("settings.light", "en", "Light"),
            ("settings.color", "en", "Accent Color"),
            ("settings.bg_color", "en", "Background"),
            ("settings.bg_auto", "en", "Auto from Accent"),
            ("settings.text_color", "en", "Text Color"),
            ("settings.text_auto", "en", "Auto from BG"),
            ("settings.language", "en", "Language"),
            ("settings.font_size", "en", "Text Size"),
            ("status.agent", "en", "Assistant"),
            ("status.examples", "en", "Knowledge"),
            ("status.version", "en", "Version"),
            ("bot.title", "en", "Bots"),
            ("bot.new", "en", "New Bot"),
            ("bot.crawler", "en", "Web Crawler"),
            ("bot.processor", "en", "Data Processor"),
            ("bot.start", "en", "Start"),
            ("bot.stop", "en", "Stop"),
            ("bot.running", "en", "Running..."),
            ("bot.name", "en", "Name"),
            ("bot.type", "en", "Type"),
            ("bot.url", "en", "URL"),
            ("bot.mode", "en", "Mode"),
            ("data.title", "en", "Data Review"),
            ("data.approve", "en", "Approve"),
            ("data.reject", "en", "Reject"),
            ("data.approve_all", "en", "Approve All"),
            ("data.add_to_kb", "en", "Add to Brain"),
        ];

        let mut stmt = conn
            .prepare(
                "INSERT OR IGNORE INTO translations (key, language_code, text) VALUES (?1, ?2, ?3)",
            )
            .map_err(|e| format!("Prepare error: {}", e))?;

        for (key, lang, text) in &translations {
            stmt.execute(params![key, lang, text])
                .map_err(|e| format!("Insert translation error: {}", e))?;
        }

        Ok(())
    }

    pub fn get_all_settings(&self) -> Result<Vec<(String, String)>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let mut stmt = conn
            .prepare("SELECT key, value FROM settings")
            .map_err(|e| format!("Query error: {}", e))?;
        let rows = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| format!("Query error: {}", e))?;
        let mut result = Vec::new();
        for r in rows {
            result.push(r.map_err(|e| format!("Row error: {}", e))?);
        }
        Ok(result)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )
        .map_err(|e| format!("Set setting error: {}", e))?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let result = conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        );
        match result {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(format!("Get setting error: {}", e)),
        }
    }

    pub fn get_translations(&self, lang: &str) -> Result<Vec<(String, String)>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let mut stmt = conn
            .prepare("SELECT key, text FROM translations WHERE language_code = ?1")
            .map_err(|e| format!("Query error: {}", e))?;
        let rows = stmt
            .query_map(params![lang], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| format!("Query error: {}", e))?;
        let mut result = Vec::new();
        for r in rows {
            result.push(r.map_err(|e| format!("Row error: {}", e))?);
        }
        Ok(result)
    }

    pub fn create_chat(&self, title: &str) -> Result<i64, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute("INSERT INTO chats (title) VALUES (?1)", params![title])
            .map_err(|e| format!("Create chat error: {}", e))?;
        Ok(conn.last_insert_rowid())
    }

    pub fn list_chats(&self, include_archived: bool) -> Result<Vec<ChatRow>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let sql = if include_archived {
            "SELECT id, title, created_at, updated_at, archived FROM chats ORDER BY updated_at DESC"
        } else {
            "SELECT id, title, created_at, updated_at, archived FROM chats WHERE archived = 0 ORDER BY updated_at DESC"
        };
        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| format!("Query error: {}", e))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(ChatRow {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    created_at: row.get(2)?,
                    updated_at: row.get(3)?,
                    archived: row.get::<_, i32>(4)? != 0,
                })
            })
            .map_err(|e| format!("Query error: {}", e))?;
        let mut result = Vec::new();
        for r in rows {
            result.push(r.map_err(|e| format!("Row error: {}", e))?);
        }
        Ok(result)
    }

    pub fn delete_chat(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute("DELETE FROM messages WHERE chat_id = ?1", params![id])
            .map_err(|e| format!("Delete messages error: {}", e))?;
        conn.execute("DELETE FROM chats WHERE id = ?1", params![id])
            .map_err(|e| format!("Delete chat error: {}", e))?;
        Ok(())
    }

    pub fn archive_chat(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute(
            "UPDATE chats SET archived = 1, updated_at = datetime('now') WHERE id = ?1",
            params![id],
        )
        .map_err(|e| format!("Archive chat error: {}", e))?;
        Ok(())
    }

    pub fn update_chat_title(&self, id: i64, title: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute(
            "UPDATE chats SET title = ?2, updated_at = datetime('now') WHERE id = ?1",
            params![id, title],
        )
        .map_err(|e| format!("Update title error: {}", e))?;
        Ok(())
    }

    pub fn add_message(
        &self,
        chat_id: i64,
        role: &str,
        content: &str,
        confidence: Option<f64>,
    ) -> Result<i64, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute(
            "INSERT INTO messages (chat_id, role, content, confidence) VALUES (?1, ?2, ?3, ?4)",
            params![chat_id, role, content, confidence],
        )
        .map_err(|e| format!("Add message error: {}", e))?;

        conn.execute(
            "UPDATE chats SET updated_at = datetime('now') WHERE id = ?1",
            params![chat_id],
        )
        .map_err(|e| format!("Update chat timestamp error: {}", e))?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_messages(&self, chat_id: i64) -> Result<Vec<MessageRow>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let mut stmt = conn
            .prepare("SELECT id, chat_id, role, content, confidence, timestamp FROM messages WHERE chat_id = ?1 ORDER BY id ASC")
            .map_err(|e| format!("Query error: {}", e))?;
        let rows = stmt
            .query_map(params![chat_id], |row| {
                Ok(MessageRow {
                    id: row.get(0)?,
                    chat_id: row.get(1)?,
                    role: row.get(2)?,
                    content: row.get(3)?,
                    confidence: row.get(4)?,
                    timestamp: row.get(5)?,
                })
            })
            .map_err(|e| format!("Query error: {}", e))?;
        let mut result = Vec::new();
        for r in rows {
            result.push(r.map_err(|e| format!("Row error: {}", e))?);
        }
        Ok(result)
    }

    fn seed_new_translations(conn: &Connection) -> Result<(), String> {
        let translations: Vec<(&str, &str, &str)> = vec![
            ("tab.chat", "en", "Chat"),
            ("tab.training", "en", "Training"),
            ("tab.files", "en", "Files"),
            ("tab.vectordb", "en", "VectorDB"),
            ("training.categories", "en", "Categories"),
            ("training.add_example", "en", "Add Example"),
            ("training.import", "en", "Import JSON"),
            ("training.export", "en", "Export JSON"),
            ("training.category_name", "en", "Category Name"),
            ("files.new_file", "en", "New File"),
            ("files.new_folder", "en", "New Folder"),
            ("files.save", "en", "Save"),
            ("files.delete", "en", "Delete"),
            ("files.rename", "en", "Rename"),
            ("vectordb.collections", "en", "Collections"),
            ("vectordb.add_entry", "en", "Add Entry"),
            ("vectordb.search", "en", "Search"),
            ("vectordb.new_collection", "en", "New Collection"),
            ("import.title", "en", "Import Dataset"),
            ("import.tatoeba", "en", "Tatoeba (TSV)"),
            ("import.drop", "en", "Drop TSV file here or click"),
            ("import.importing", "en", "Importing..."),
            (
                "import.info",
                "en",
                "Tatoeba bilingual sentence pairs from manythings.org/anki/",
            ),
            ("console.title", "en", "Console"),
            ("console.clear", "en", "Clear"),
            ("console.placeholder", "en", "Type a command..."),
            ("nav.tools", "en", "Tools"),
            ("nav.review", "en", "Review"),
            ("kb.all", "en", "All"),
            ("kb.categories", "en", "Categories"),
            ("kb.import", "en", "Import"),
            ("nav.knowledge", "en", "Knowledge"),
            ("kb.save", "en", "Save"),
            ("kb.load", "en", "Load"),
            ("data.add_to_kb", "en", "Add to Knowledge"),
            ("vectordb.search_btn", "en", "Search"),
        ];

        let mut stmt = conn
            .prepare(
                "INSERT OR IGNORE INTO translations (key, language_code, text) VALUES (?1, ?2, ?3)",
            )
            .map_err(|e| format!("Prepare error: {}", e))?;

        for (key, lang, text) in &translations {
            stmt.execute(params![key, lang, text])
                .map_err(|e| format!("Insert translation error: {}", e))?;
        }

        Ok(())
    }

    fn refresh_product_text(conn: &Connection) -> Result<(), String> {
        let updates: &[(&str, &str, &str, &str)] = &[
            ("landing.feat.multilang.name", "en", "Multi-Language UI", "English Interface"),
            ("landing.feat.multilang.desc", "en", "English, German, Turkish", "Clear English labels and documentation"),
            ("landing.badge.version", "en", "v0.1.8", concat!("v", env!("CARGO_PKG_VERSION"))),
            ("agent.tfidf.desc", "en", "Finds best answer by meaning", "Ranks answers by shared words (BM25)"),
            ("landing.tagline", "en", "A trainable, modular AI engine written in Rust", "A Modular Knowledge Intelligence Engine"),
            ("landing.desc", "en", "Build intelligent agents, manage knowledge bases, extract wisdom from PDFs, and deploy a full web dashboard \u{2014} all without external AI APIs.", "Find knowledge, compare text, and propose connections. This optional dashboard uses the AIRust Rust library."),
            ("landing.badge.version", "en", "v0.1.7", concat!("v", env!("CARGO_PKG_VERSION"))),
            ("landing.badge.rust", "en", "Rust 1.85+", "Rust 1.89+"),
            ("landing.badge.tests", "en", "156 Tests", "Tests & CI"),
            ("landing.claim", "en", "100% Local \u{00b7} Zero Cloud Dependencies \u{00b7} Fully Private", "Local core \u{00b7} Optional integrations"),
            ("landing.features.sub", "en", "Everything you need to build, train, and deploy an AI system", "Optional tools around the AIRust knowledge library"),
            ("landing.feat.agents.desc", "en", "Exact Match, Fuzzy Match, TF-IDF/BM25 Semantic, Context-Aware", "Exact Match, Fuzzy Match, BM25 lexical search, Context-Aware"),
            ("landing.agent.tfidf.desc", "en", "Semantic search using TF-IDF and BM25 ranking. Understands meaning, not just words. Handles synonyms and related concepts.", "Lexical BM25 ranking using shared terms. Synonyms and meaning require an optional external model."),
            ("landing.bench.tests.label", "en", "Tests Passing", "Automated Checks"),
            ("landing.bench.apis.label", "en", "External APIs", "Required LLM APIs"),
            ("landing.bench.local.label", "en", "Local & Private", "Local Core"),
        ];
        for (key, language, previous, current) in updates {
            conn.execute(
                "UPDATE translations SET text = ?1 WHERE key = ?2 AND language_code = ?3 AND text = ?4",
                params![current, key, language, previous],
            ).map_err(|error| format!("Product text migration error: {error}"))?;
        }
        Ok(())
    }

    fn seed_landing_translations(conn: &Connection) -> Result<(), String> {
        let translations: Vec<(&str, &str, &str)> = vec![

            ("landing.tagline", "en", "A Modular Knowledge Intelligence Engine"),
            ("landing.desc", "en", "Find knowledge, compare text, and propose connections. This optional dashboard uses the AIRust Rust library."),
            ("landing.badge.version", "en", concat!("v", env!("CARGO_PKG_VERSION"))),
            ("landing.badge.license", "en", "MIT License"),
            ("landing.badge.rust", "en", "Rust 1.89+"),
            ("landing.badge.tests", "en", "Tests & CI"),
            ("landing.claim", "en", "Local core \u{00b7} Optional integrations"),
            ("landing.cta", "en", "Try it now"),

            ("landing.features.title", "en", "Features"),
            ("landing.features.sub", "en", "Optional tools around the AIRust knowledge library"),
            ("landing.feat.agents.name", "en", "4 Agent Types"),
            ("landing.feat.agents.desc", "en", "Exact Match, Fuzzy Match, BM25 lexical search, Context-Aware"),
            ("landing.feat.kb.name", "en", "Knowledge Base"),
            ("landing.feat.kb.desc", "en", "JSON-based, compile-time embedded, runtime expandable"),
            ("landing.feat.pdf.name", "en", "PDF Processing"),
            ("landing.feat.pdf.desc", "en", "Convert PDFs to structured training data with smart chunking"),
            ("landing.feat.dashboard.name", "en", "Web Dashboard"),
            ("landing.feat.dashboard.desc", "en", "Full UI with chat, training manager, bot control, file browser"),
            ("landing.feat.bots.name", "en", "Bot Ecosystem"),
            ("landing.feat.bots.desc", "en", "Automated web scraping with review workflow"),
            ("landing.feat.vectordb.name", "en", "Vector Database"),
            ("landing.feat.vectordb.desc", "en", "Embedding storage and similarity search"),
            ("landing.feat.chat.name", "en", "Chat History"),
            ("landing.feat.chat.desc", "en", "Persistent conversations with archiving"),
            ("landing.feat.multilang.name", "en", "English Interface"),
            ("landing.feat.multilang.desc", "en", "Clear English labels and documentation"),
            ("landing.feat.api.name", "en", "REST API"),
            ("landing.feat.api.desc", "en", "50+ endpoints for full programmatic control"),
            ("landing.feat.console.name", "en", "WebSocket Console"),
            ("landing.feat.console.desc", "en", "Live terminal with server logs, shell access, built-in commands"),
            ("landing.feat.docker.name", "en", "Docker Support"),
            ("landing.feat.docker.desc", "en", "One-command deployment with containers"),
            ("landing.feat.cli.name", "en", "CLI Tools"),
            ("landing.feat.cli.desc", "en", "Interactive mode, query tools, PDF conversion"),

            ("landing.arch.title", "en", "Architecture"),
            ("landing.arch.sub", "en", "A layered architecture: agents do the thinking, the knowledge base stores the data, and the web server provides the interface"),
            ("landing.traits.th.trait", "en", "Trait"),
            ("landing.traits.th.purpose", "en", "Purpose"),
            ("landing.traits.agent.purpose", "en", "Base trait: predict(), confidence(), can_answer()"),
            ("landing.traits.trainable.purpose", "en", "Adds train(), add_example()"),
            ("landing.traits.contextual.purpose", "en", "Adds add_context(), clear_context() for conversation memory"),
            ("landing.traits.confidence.purpose", "en", "Adds calculate_confidence(), predict_top_n()"),

            ("landing.agents.title", "en", "Agent Types"),
            ("landing.agents.sub", "en", "The brain of AIRust \u{2014} choose the right agent for your use case"),
            ("landing.agent.match.title", "en", "MatchAgent"),
            ("landing.agent.match.desc", "en", "The simplest and fastest agent. Compares questions directly against training data using exact or fuzzy matching with Levenshtein distance."),
            ("landing.agent.match.use", "en", "Best for: FAQ bots, command recognition, structured Q&A"),
            ("landing.agent.tfidf.title", "en", "TfidfAgent"),
            ("landing.agent.tfidf.desc", "en", "Lexical BM25 ranking using shared terms. Synonyms and meaning require an optional external model."),
            ("landing.agent.tfidf.use", "en", "Best for: Natural language Q&A, documentation search"),
            ("landing.agent.context.title", "en", "ContextAgent"),
            ("landing.agent.context.desc", "en", "Wraps any agent and adds conversation memory. Maintains context across messages for multi-turn dialogue."),
            ("landing.agent.context.use", "en", "Best for: Chatbots, support agents, interactive assistants"),

            ("landing.bench.title", "en", "By the Numbers"),
            ("landing.bench.sub", "en", "Built for performance, privacy, and simplicity"),
            ("landing.bench.tests.label", "en", "Automated Checks"),
            ("landing.bench.apis.label", "en", "Required LLM APIs"),
            ("landing.bench.endpoints.label", "en", "REST Endpoints"),
            ("landing.bench.local.label", "en", "Local Core"),

            ("landing.install.title", "en", "Get Started"),
            ("landing.install.sub", "en", "Up and running in seconds"),

            ("landing.showroom", "en", "SHOWROOM"),
        ];

        let mut stmt = conn
            .prepare(
                "INSERT OR IGNORE INTO translations (key, language_code, text) VALUES (?1, ?2, ?3)",
            )
            .map_err(|e| format!("Prepare error: {}", e))?;

        for (key, lang, text) in &translations {
            stmt.execute(params![key, lang, text])
                .map_err(|e| format!("Insert landing translation error: {}", e))?;
        }

        Ok(())
    }

    pub fn list_categories(&self) -> Result<Vec<CategoryRow>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, color, description, created_at FROM categories ORDER BY name ASC",
            )
            .map_err(|e| format!("Query error: {}", e))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(CategoryRow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    color: row.get(2)?,
                    description: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .map_err(|e| format!("Query error: {}", e))?;
        let mut result = Vec::new();
        for r in rows {
            result.push(r.map_err(|e| format!("Row error: {}", e))?);
        }
        Ok(result)
    }

    pub fn create_category(
        &self,
        name: &str,
        color: &str,
        description: &str,
    ) -> Result<i64, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute(
            "INSERT INTO categories (name, color, description) VALUES (?1, ?2, ?3)",
            params![name, color, description],
        )
        .map_err(|e| format!("Create category error: {}", e))?;
        Ok(conn.last_insert_rowid())
    }

    pub fn delete_category(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute(
            "UPDATE training_data SET category_id = NULL WHERE category_id = ?1",
            params![id],
        )
        .map_err(|e| format!("Update training data error: {}", e))?;
        conn.execute("DELETE FROM categories WHERE id = ?1", params![id])
            .map_err(|e| format!("Delete category error: {}", e))?;
        Ok(())
    }

    pub fn list_training_data(
        &self,
        category_id: Option<i64>,
    ) -> Result<Vec<TrainingDataRow>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        fn extract_row(row: &rusqlite::Row) -> rusqlite::Result<TrainingDataRow> {
            Ok(TrainingDataRow {
                id: row.get(0)?,
                category_id: row.get(1)?,
                input: row.get(2)?,
                output_text: row.get(3)?,
                output_format: row.get(4)?,
                weight: row.get(5)?,
                created_at: row.get(6)?,
                metadata: row
                    .get::<_, Option<String>>(7)?
                    .map(|json| {
                        serde_json::from_str(&json).map_err(|error| {
                            rusqlite::Error::FromSqlConversionFailure(
                                7,
                                rusqlite::types::Type::Text,
                                Box::new(error),
                            )
                        })
                    })
                    .transpose()?,
            })
        }

        let mut result = Vec::new();
        if let Some(cid) = category_id {
            let mut stmt = conn.prepare(
                "SELECT id, category_id, input, output_text, output_format, weight, created_at, metadata_json FROM training_data WHERE category_id = ?1 ORDER BY id DESC"
            ).map_err(|e| format!("Query error: {}", e))?;
            let rows = stmt
                .query_map(params![cid], extract_row)
                .map_err(|e| format!("Query error: {}", e))?;
            for r in rows {
                result.push(r.map_err(|e| format!("Row error: {}", e))?);
            }
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, category_id, input, output_text, output_format, weight, created_at, metadata_json FROM training_data ORDER BY id DESC"
            ).map_err(|e| format!("Query error: {}", e))?;
            let rows = stmt
                .query_map([], extract_row)
                .map_err(|e| format!("Query error: {}", e))?;
            for r in rows {
                result.push(r.map_err(|e| format!("Row error: {}", e))?);
            }
        }
        Ok(result)
    }

    pub fn add_training_data(
        &self,
        category_id: Option<i64>,
        input: &str,
        output_text: &str,
        output_format: &str,
        weight: f64,
    ) -> Result<i64, String> {
        if !weight.is_finite() || !(0.0..=f32::MAX as f64).contains(&weight) {
            return Err("Training weight must be finite, nonnegative, and fit f32".to_owned());
        }
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute(
            "INSERT INTO training_data (category_id, input, output_text, output_format, weight) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![category_id, input, output_text, output_format, weight],
        )
        .map_err(|e| format!("Add training data error: {}", e))?;
        Ok(conn.last_insert_rowid())
    }

    pub fn delete_training_data(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute("DELETE FROM training_data WHERE id = ?1", params![id])
            .map_err(|e| format!("Delete training data error: {}", e))?;
        Ok(())
    }

    pub fn add_training_example(
        &self,
        category_id: Option<i64>,
        example: &TrainingExample,
    ) -> Result<i64, String> {
        example.validate().map_err(|error| error.to_string())?;
        let conn = self
            .conn
            .lock()
            .map_err(|error| format!("Lock error: {error}"))?;
        let format = match &example.output {
            ResponseFormat::Text(_) => "text",
            ResponseFormat::Markdown(_) => "markdown",
            ResponseFormat::Json(_) => "json",
        };
        let metadata = example
            .metadata
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|error| format!("Metadata serialization: {error}"))?;
        conn.execute(
            "INSERT INTO training_data (category_id, input, output_text, output_format, weight, metadata_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![category_id, example.input, example.output.to_string(), format, example.weight as f64, metadata],
        ).map_err(|error| format!("Add training example: {error}"))?;
        Ok(conn.last_insert_rowid())
    }

    pub fn add_training_batch(
        &self,
        examples: &[(Option<i64>, TrainingExample)],
    ) -> Result<(), String> {
        for (_, example) in examples {
            example.validate().map_err(|error| error.to_string())?;
        }
        let mut connection = self
            .conn
            .lock()
            .map_err(|error| format!("Lock error: {error}"))?;
        let transaction = connection
            .transaction()
            .map_err(|error| format!("Training transaction: {error}"))?;
        {
            let mut statement = transaction.prepare("INSERT INTO training_data (category_id, input, output_text, output_format, weight, metadata_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6)").map_err(|error| error.to_string())?;
            for (category, example) in examples {
                let format = match &example.output {
                    ResponseFormat::Text(_) => "text",
                    ResponseFormat::Markdown(_) => "markdown",
                    ResponseFormat::Json(_) => "json",
                };
                let metadata = example
                    .metadata
                    .as_ref()
                    .map(serde_json::to_string)
                    .transpose()
                    .map_err(|error| error.to_string())?;
                statement
                    .execute(params![
                        category,
                        example.input,
                        example.output.to_string(),
                        format,
                        example.weight as f64,
                        metadata
                    ])
                    .map_err(|error| error.to_string())?;
            }
        }
        transaction
            .commit()
            .map_err(|error| format!("Training commit: {error}"))
    }

    pub fn get_all_training_data(&self) -> Result<Vec<TrainingDataRow>, String> {
        self.list_training_data(None)
    }

    pub fn list_vector_collections(&self) -> Result<Vec<VectorCollectionRow>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let mut stmt = conn
            .prepare("SELECT id, name, description, created_at FROM vector_collections ORDER BY name ASC")
            .map_err(|e| format!("Query error: {}", e))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(VectorCollectionRow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })
            .map_err(|e| format!("Query error: {}", e))?;
        let mut result = Vec::new();
        for r in rows {
            result.push(r.map_err(|e| format!("Row error: {}", e))?);
        }
        Ok(result)
    }

    pub fn create_vector_collection(&self, name: &str, description: &str) -> Result<i64, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute(
            "INSERT INTO vector_collections (name, description) VALUES (?1, ?2)",
            params![name, description],
        )
        .map_err(|e| format!("Create collection error: {}", e))?;
        Ok(conn.last_insert_rowid())
    }

    pub fn delete_vector_collection(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute(
            "DELETE FROM vector_entries WHERE collection_id = ?1",
            params![id],
        )
        .map_err(|e| format!("Delete entries error: {}", e))?;
        conn.execute("DELETE FROM vector_collections WHERE id = ?1", params![id])
            .map_err(|e| format!("Delete collection error: {}", e))?;
        Ok(())
    }

    pub fn list_vector_entries(&self, collection_id: i64) -> Result<Vec<VectorEntryRow>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let mut stmt = conn
            .prepare("SELECT id, collection_id, content, metadata_json, embedding_json, created_at FROM vector_entries WHERE collection_id = ?1 ORDER BY id DESC")
            .map_err(|e| format!("Query error: {}", e))?;
        let rows = stmt
            .query_map(params![collection_id], |row| {
                Ok(VectorEntryRow {
                    id: row.get(0)?,
                    collection_id: row.get(1)?,
                    content: row.get(2)?,
                    metadata_json: row.get(3)?,
                    embedding_json: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })
            .map_err(|e| format!("Query error: {}", e))?;
        let mut result = Vec::new();
        for r in rows {
            result.push(r.map_err(|e| format!("Row error: {}", e))?);
        }
        Ok(result)
    }

    pub fn add_vector_entry(
        &self,
        collection_id: i64,
        content: &str,
        metadata_json: &str,
        embedding_json: &str,
    ) -> Result<i64, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute(
            "INSERT INTO vector_entries (collection_id, content, metadata_json, embedding_json) VALUES (?1, ?2, ?3, ?4)",
            params![collection_id, content, metadata_json, embedding_json],
        )
        .map_err(|e| format!("Add entry error: {}", e))?;
        Ok(conn.last_insert_rowid())
    }

    pub fn delete_vector_entry(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute("DELETE FROM vector_entries WHERE id = ?1", params![id])
            .map_err(|e| format!("Delete entry error: {}", e))?;
        Ok(())
    }

    pub fn get_all_vector_entries_in_collection(
        &self,
        collection_id: i64,
    ) -> Result<Vec<VectorEntryRow>, String> {
        self.list_vector_entries(collection_id)
    }
}
