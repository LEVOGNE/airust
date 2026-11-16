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

            -- Bot ecosystem tables
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

            -- Training & VectorDB tables
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

        // Default settings (only insert if not exists)
        let defaults = [
            ("theme", "dark"),
            ("language", "en"),
            ("font_size", "13"),
            // Dark theme colors
            ("dark.accent_color", "#ffd6d6"),
            ("dark.bg_color", "#002929"),
            ("dark.bg_auto", "true"),
            ("dark.text_color", "#e4e4f0"),
            ("dark.text_auto", "true"),
            // Light theme colors
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

        // Translations: check if any exist first
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM translations", [], |r| r.get(0))
            .map_err(|e| format!("Count error: {}", e))?;

        if count == 0 {
            Self::seed_translations(&conn)?;
        }

        Self::seed_new_translations(&conn)?;
        Self::seed_landing_translations(&conn)?;
        Self::refresh_product_text(&conn)?;

        Ok(())
    }

    fn seed_translations(conn: &Connection) -> Result<(), String> {
        let translations: Vec<(&str, &str, &str)> = vec![
            // Agent names
            ("agent.tfidf.name", "en", "Smart Search"),
            ("agent.tfidf.name", "de", "Schlaue Suche"),
            ("agent.tfidf.name", "tr", "Akıllı Arama"),
            (
                "agent.tfidf.desc",
                "en",
                "Ranks answers by shared words (BM25)",
            ),
            (
                "agent.tfidf.desc",
                "de",
                "Ordnet Antworten nach gemeinsamen Wörtern (BM25)",
            ),
            (
                "agent.tfidf.desc",
                "tr",
                "Ortak kelimelere göre sıralar (BM25)",
            ),
            ("agent.exact.name", "en", "Word-for-Word"),
            ("agent.exact.name", "de", "Wort für Wort"),
            ("agent.exact.name", "tr", "Kelimesi Kelimesine"),
            ("agent.exact.desc", "en", "Must match exactly, ignores caps"),
            (
                "agent.exact.desc",
                "de",
                "Muss genau passen, Groß/Klein egal",
            ),
            (
                "agent.exact.desc",
                "tr",
                "Tam eşleşmeli, büyük/küçük harf yok",
            ),
            ("agent.fuzzy.name", "en", "Close Enough"),
            ("agent.fuzzy.name", "de", "Fast Richtig"),
            ("agent.fuzzy.name", "tr", "Yaklaşık Eşleşme"),
            (
                "agent.fuzzy.desc",
                "en",
                "Also finds typos and similar words",
            ),
            (
                "agent.fuzzy.desc",
                "de",
                "Findet auch Tippfehler und ähnliche Wörter",
            ),
            (
                "agent.fuzzy.desc",
                "tr",
                "Yazım hataları ve benzer kelimeleri de bulur",
            ),
            ("agent.context.name", "en", "Remembers Chat"),
            ("agent.context.name", "de", "Merkt sich Chat"),
            ("agent.context.name", "tr", "Sohbeti Hatırlar"),
            (
                "agent.context.desc",
                "en",
                "Uses previous messages for better answers",
            ),
            (
                "agent.context.desc",
                "de",
                "Nutzt vorherige Nachrichten für bessere Antworten",
            ),
            (
                "agent.context.desc",
                "tr",
                "Daha iyi cevaplar için önceki mesajları kullanır",
            ),
            // KB search
            ("kb.search", "en", "Search..."),
            ("kb.search", "de", "Suchen..."),
            ("kb.search", "tr", "Ara..."),
            // Navigation / Headers
            ("nav.agent", "en", "Assistant"),
            ("nav.agent", "de", "Assistent"),
            ("nav.agent", "tr", "Asistan"),
            ("nav.chat", "en", "Chat"),
            ("nav.chat", "de", "Chat"),
            ("nav.chat", "tr", "Sohbet"),
            ("nav.knowledge", "en", "Brain"),
            ("nav.knowledge", "de", "Gehirn"),
            ("nav.knowledge", "tr", "Beyin"),
            ("nav.settings", "en", "Settings"),
            ("nav.settings", "de", "Einstellungen"),
            ("nav.settings", "tr", "Ayarlar"),
            ("nav.guide", "en", "Guide"),
            ("nav.guide", "de", "Anleitung"),
            ("nav.guide", "tr", "Rehber"),
            ("nav.chats", "en", "Chats"),
            ("nav.chats", "de", "Chats"),
            ("nav.chats", "tr", "Sohbetler"),
            // Chat UI
            ("chat.placeholder", "en", "Ask a question..."),
            ("chat.placeholder", "de", "Stelle eine Frage..."),
            ("chat.placeholder", "tr", "Bir soru sor..."),
            ("chat.send", "en", "Send"),
            ("chat.send", "de", "Senden"),
            ("chat.send", "tr", "Gönder"),
            ("chat.new", "en", "New Chat"),
            ("chat.new", "de", "Neuer Chat"),
            ("chat.new", "tr", "Yeni Sohbet"),
            ("chat.archive", "en", "Archive"),
            ("chat.archive", "de", "Archivieren"),
            ("chat.archive", "tr", "Arşivle"),
            ("chat.delete", "en", "Delete"),
            ("chat.delete", "de", "Löschen"),
            ("chat.delete", "tr", "Sil"),
            ("chat.show_archive", "en", "Show Archive"),
            ("chat.show_archive", "de", "Archiv anzeigen"),
            ("chat.show_archive", "tr", "Arşivi Göster"),
            ("chat.hide_archive", "en", "Hide Archive"),
            ("chat.hide_archive", "de", "Archiv ausblenden"),
            ("chat.hide_archive", "tr", "Arşivi Gizle"),
            ("chat.confidence", "en", "Confidence"),
            ("chat.confidence", "de", "Sicherheit"),
            ("chat.confidence", "tr", "Güven"),
            // Knowledge Base
            ("kb.add", "en", "Teach Something"),
            ("kb.add", "de", "Etwas Beibringen"),
            ("kb.add", "tr", "Bir Şey Öğret"),
            ("kb.input", "en", "Question"),
            ("kb.input", "de", "Frage"),
            ("kb.input", "tr", "Soru"),
            ("kb.output", "en", "Answer"),
            ("kb.output", "de", "Antwort"),
            ("kb.output", "tr", "Cevap"),
            ("kb.weight", "en", "Weight"),
            ("kb.weight", "de", "Gewicht"),
            ("kb.weight", "tr", "Ağırlık"),
            ("kb.save", "en", "Save Brain"),
            ("kb.save", "de", "Gehirn Speichern"),
            ("kb.save", "tr", "Beyni Kaydet"),
            ("kb.load", "en", "Load Brain"),
            ("kb.load", "de", "Gehirn Laden"),
            ("kb.load", "tr", "Beyni Yükle"),
            (
                "kb.path_placeholder",
                "en",
                "Path (e.g. ./knowledge/my.json)",
            ),
            (
                "kb.path_placeholder",
                "de",
                "Pfad (z.B. ./knowledge/my.json)",
            ),
            (
                "kb.path_placeholder",
                "tr",
                "Yol (örn. ./knowledge/my.json)",
            ),
            // PDF
            ("pdf.title", "en", "Upload PDF"),
            ("pdf.title", "de", "PDF Hochladen"),
            ("pdf.title", "tr", "PDF Yükle"),
            ("pdf.drop", "en", "Drop PDF here or click"),
            ("pdf.drop", "de", "PDF hierher ziehen oder klicken"),
            ("pdf.drop", "tr", "PDF'yi buraya bırak veya tıkla"),
            // Settings
            ("settings.theme", "en", "Appearance"),
            ("settings.theme", "de", "Aussehen"),
            ("settings.theme", "tr", "Görünüm"),
            ("settings.dark", "en", "Dark"),
            ("settings.dark", "de", "Dunkel"),
            ("settings.dark", "tr", "Koyu"),
            ("settings.light", "en", "Light"),
            ("settings.light", "de", "Hell"),
            ("settings.light", "tr", "Açık"),
            ("settings.color", "en", "Accent Color"),
            ("settings.color", "de", "Akzentfarbe"),
            ("settings.color", "tr", "Vurgu Rengi"),
            ("settings.bg_color", "en", "Background"),
            ("settings.bg_color", "de", "Hintergrund"),
            ("settings.bg_color", "tr", "Arka Plan"),
            ("settings.bg_auto", "en", "Auto from Accent"),
            ("settings.bg_auto", "de", "Auto aus Akzent"),
            ("settings.bg_auto", "tr", "Aksan'dan Otomatik"),
            ("settings.text_color", "en", "Text Color"),
            ("settings.text_color", "de", "Textfarbe"),
            ("settings.text_color", "tr", "Yazı Rengi"),
            ("settings.text_auto", "en", "Auto from BG"),
            ("settings.text_auto", "de", "Auto aus Hintergrund"),
            ("settings.text_auto", "tr", "Arka Plandan Otomatik"),
            ("settings.language", "en", "Language"),
            ("settings.language", "de", "Sprache"),
            ("settings.language", "tr", "Dil"),
            ("settings.font_size", "en", "Text Size"),
            ("settings.font_size", "de", "Textgröße"),
            ("settings.font_size", "tr", "Yazı Boyutu"),
            // Status
            ("status.agent", "en", "Assistant"),
            ("status.agent", "de", "Assistent"),
            ("status.agent", "tr", "Asistan"),
            ("status.examples", "en", "Knowledge"),
            ("status.examples", "de", "Wissen"),
            ("status.examples", "tr", "Bilgi"),
            ("status.version", "en", "Version"),
            ("status.version", "de", "Version"),
            ("status.version", "tr", "Sürüm"),
            // Bot ecosystem
            ("bot.title", "en", "Bots"),
            ("bot.title", "de", "Bots"),
            ("bot.title", "tr", "Botlar"),
            ("bot.new", "en", "New Bot"),
            ("bot.new", "de", "Neuer Bot"),
            ("bot.new", "tr", "Yeni Bot"),
            ("bot.crawler", "en", "Web Crawler"),
            ("bot.crawler", "de", "Web-Crawler"),
            ("bot.crawler", "tr", "Web Tarayıcı"),
            ("bot.processor", "en", "Data Processor"),
            ("bot.processor", "de", "Datenverarbeiter"),
            ("bot.processor", "tr", "Veri İşlemci"),
            ("bot.start", "en", "Start"),
            ("bot.start", "de", "Starten"),
            ("bot.start", "tr", "Başlat"),
            ("bot.stop", "en", "Stop"),
            ("bot.stop", "de", "Stoppen"),
            ("bot.stop", "tr", "Durdur"),
            ("bot.running", "en", "Running..."),
            ("bot.running", "de", "Läuft..."),
            ("bot.running", "tr", "Çalışıyor..."),
            ("bot.name", "en", "Name"),
            ("bot.name", "de", "Name"),
            ("bot.name", "tr", "İsim"),
            ("bot.type", "en", "Type"),
            ("bot.type", "de", "Typ"),
            ("bot.type", "tr", "Tür"),
            ("bot.url", "en", "URL"),
            ("bot.url", "de", "URL"),
            ("bot.url", "tr", "URL"),
            ("bot.mode", "en", "Mode"),
            ("bot.mode", "de", "Modus"),
            ("bot.mode", "tr", "Mod"),
            ("data.title", "en", "Data Review"),
            ("data.title", "de", "Datenprüfung"),
            ("data.title", "tr", "Veri İnceleme"),
            ("data.approve", "en", "Approve"),
            ("data.approve", "de", "Genehmigen"),
            ("data.approve", "tr", "Onayla"),
            ("data.reject", "en", "Reject"),
            ("data.reject", "de", "Ablehnen"),
            ("data.reject", "tr", "Reddet"),
            ("data.approve_all", "en", "Approve All"),
            ("data.approve_all", "de", "Alle Genehmigen"),
            ("data.approve_all", "tr", "Tümünü Onayla"),
            ("data.add_to_kb", "en", "Add to Brain"),
            ("data.add_to_kb", "de", "Ins Gehirn"),
            ("data.add_to_kb", "tr", "Beyine Ekle"),
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

    // ── Settings ──────────────────────────────────────────────────────────────

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

    // ── Translations ──────────────────────────────────────────────────────────

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

    // ── Chats ─────────────────────────────────────────────────────────────────

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

    // ── Messages ──────────────────────────────────────────────────────────────

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
        // Update chat timestamp
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

    // ── New Translations (INSERT OR IGNORE) ──────────────────────────────────

    fn seed_new_translations(conn: &Connection) -> Result<(), String> {
        let translations: Vec<(&str, &str, &str)> = vec![
            // Center tabs
            ("tab.chat", "en", "Chat"),
            ("tab.chat", "de", "Chat"),
            ("tab.chat", "tr", "Sohbet"),
            ("tab.training", "en", "Training"),
            ("tab.training", "de", "Training"),
            ("tab.training", "tr", "Eğitim"),
            ("tab.files", "en", "Files"),
            ("tab.files", "de", "Dateien"),
            ("tab.files", "tr", "Dosyalar"),
            ("tab.vectordb", "en", "VectorDB"),
            ("tab.vectordb", "de", "VektorDB"),
            ("tab.vectordb", "tr", "VektörDB"),
            // Training
            ("training.categories", "en", "Categories"),
            ("training.categories", "de", "Kategorien"),
            ("training.categories", "tr", "Kategoriler"),
            ("training.add_example", "en", "Add Example"),
            ("training.add_example", "de", "Beispiel hinzufügen"),
            ("training.add_example", "tr", "Örnek Ekle"),
            ("training.import", "en", "Import JSON"),
            ("training.import", "de", "JSON Import"),
            ("training.import", "tr", "JSON İçe Aktar"),
            ("training.export", "en", "Export JSON"),
            ("training.export", "de", "JSON Export"),
            ("training.export", "tr", "JSON Dışa Aktar"),
            ("training.category_name", "en", "Category Name"),
            ("training.category_name", "de", "Kategoriename"),
            ("training.category_name", "tr", "Kategori Adı"),
            // Files
            ("files.new_file", "en", "New File"),
            ("files.new_file", "de", "Neue Datei"),
            ("files.new_file", "tr", "Yeni Dosya"),
            ("files.new_folder", "en", "New Folder"),
            ("files.new_folder", "de", "Neuer Ordner"),
            ("files.new_folder", "tr", "Yeni Klasör"),
            ("files.save", "en", "Save"),
            ("files.save", "de", "Speichern"),
            ("files.save", "tr", "Kaydet"),
            ("files.delete", "en", "Delete"),
            ("files.delete", "de", "Löschen"),
            ("files.delete", "tr", "Sil"),
            ("files.rename", "en", "Rename"),
            ("files.rename", "de", "Umbenennen"),
            ("files.rename", "tr", "Yeniden Adlandır"),
            // VectorDB
            ("vectordb.collections", "en", "Collections"),
            ("vectordb.collections", "de", "Sammlungen"),
            ("vectordb.collections", "tr", "Koleksiyonlar"),
            ("vectordb.add_entry", "en", "Add Entry"),
            ("vectordb.add_entry", "de", "Eintrag hinzufügen"),
            ("vectordb.add_entry", "tr", "Giriş Ekle"),
            ("vectordb.search", "en", "Search"),
            ("vectordb.search", "de", "Suchen"),
            ("vectordb.search", "tr", "Ara"),
            ("vectordb.new_collection", "en", "New Collection"),
            ("vectordb.new_collection", "de", "Neue Sammlung"),
            ("vectordb.new_collection", "tr", "Yeni Koleksiyon"),
            // Tatoeba Import
            ("import.title", "en", "Import Dataset"),
            ("import.title", "de", "Datensatz Import"),
            ("import.title", "tr", "Veri Seti İçe Aktar"),
            ("import.tatoeba", "en", "Tatoeba (TSV)"),
            ("import.tatoeba", "de", "Tatoeba (TSV)"),
            ("import.tatoeba", "tr", "Tatoeba (TSV)"),
            ("import.drop", "en", "Drop TSV file here or click"),
            ("import.drop", "de", "TSV-Datei hierher ziehen oder klicken"),
            (
                "import.drop",
                "tr",
                "TSV dosyasını buraya bırakın veya tıklayın",
            ),
            ("import.importing", "en", "Importing..."),
            ("import.importing", "de", "Importiere..."),
            ("import.importing", "tr", "İçe aktarılıyor..."),
            (
                "import.info",
                "en",
                "Tatoeba bilingual sentence pairs from manythings.org/anki/",
            ),
            (
                "import.info",
                "de",
                "Tatoeba zweisprachige Satzpaare von manythings.org/anki/",
            ),
            (
                "import.info",
                "tr",
                "manythings.org/anki/ kaynağından Tatoeba iki dilli cümle çiftleri",
            ),
            // Console
            ("console.title", "en", "Console"),
            ("console.title", "de", "Konsole"),
            ("console.title", "tr", "Konsol"),
            ("console.clear", "en", "Clear"),
            ("console.clear", "de", "Leeren"),
            ("console.clear", "tr", "Temizle"),
            ("console.placeholder", "en", "Type a command..."),
            ("console.placeholder", "de", "Befehl eingeben..."),
            ("console.placeholder", "tr", "Komut yazın..."),
            // New navigation (redesign)
            ("nav.tools", "en", "Tools"),
            ("nav.tools", "de", "Werkzeuge"),
            ("nav.tools", "tr", "Araçlar"),
            ("nav.review", "en", "Review"),
            ("nav.review", "de", "Prüfung"),
            ("nav.review", "tr", "İnceleme"),
            // Knowledge sub-tabs
            ("kb.all", "en", "All"),
            ("kb.all", "de", "Alle"),
            ("kb.all", "tr", "Tümü"),
            ("kb.categories", "en", "Categories"),
            ("kb.categories", "de", "Kategorien"),
            ("kb.categories", "tr", "Kategoriler"),
            ("kb.import", "en", "Import"),
            ("kb.import", "de", "Import"),
            ("kb.import", "tr", "İçe Aktar"),
            // Updated naming
            ("nav.knowledge", "en", "Knowledge"),
            ("nav.knowledge", "de", "Wissen"),
            ("nav.knowledge", "tr", "Bilgi"),
            ("kb.save", "en", "Save"),
            ("kb.save", "de", "Speichern"),
            ("kb.save", "tr", "Kaydet"),
            ("kb.load", "en", "Load"),
            ("kb.load", "de", "Laden"),
            ("kb.load", "tr", "Yükle"),
            ("data.add_to_kb", "en", "Add to Knowledge"),
            ("data.add_to_kb", "de", "Zum Wissen hinzufügen"),
            ("data.add_to_kb", "tr", "Bilgiye Ekle"),
            ("vectordb.search_btn", "en", "Search"),
            ("vectordb.search_btn", "de", "Suchen"),
            ("vectordb.search_btn", "tr", "Ara"),
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

    /// Update shipped product text while preserving customized translations.
    fn refresh_product_text(conn: &Connection) -> Result<(), String> {
        let updates: &[(&str, &str, &str, &str)] = &[
            ("agent.tfidf.desc", "en", "Finds best answer by meaning", "Ranks answers by shared words (BM25)"),
            ("agent.tfidf.desc", "de", "Findet beste Antwort nach Bedeutung", "Ordnet Antworten nach gemeinsamen Wörtern (BM25)"),
            ("agent.tfidf.desc", "tr", "Anlama göre en iyi cevabı bulur", "Ortak kelimelere göre sıralar (BM25)"),
            ("landing.tagline", "en", "A trainable, modular AI engine written in Rust", "A Modular Knowledge Intelligence Engine"),
            ("landing.tagline", "de", "Eine trainierbare, modulare KI-Engine in Rust", "Wissen finden, vergleichen und verbinden"),
            ("landing.tagline", "tr", "Rust ile yazılmış eğitilebilir, modüler bir yapay zeka motoru", "Rust ile modüler bilgi arama ve karşılaştırma"),
            ("landing.desc", "en", "Build intelligent agents, manage knowledge bases, extract wisdom from PDFs, and deploy a full web dashboard \u{2014} all without external AI APIs.", "Find knowledge, compare text, and propose connections. This optional dashboard uses the AIRust Rust library."),
            ("landing.desc", "de", "Erstelle intelligente Agenten, verwalte Wissensbasen, extrahiere Wissen aus PDFs und starte ein vollständiges Web-Dashboard \u{2014} alles ohne externe KI-APIs.", "Finde Wissen, vergleiche Texte und prüfe mögliche Verbindungen. Dieses optionale Dashboard verwendet die AIRust-Bibliothek."),
            ("landing.desc", "tr", "Akıllı ajanlar oluşturun, bilgi tabanlarını yönetin, PDF'lerden bilgi çıkarın ve tam bir web paneli kurun \u{2014} harici yapay zeka API'leri olmadan.", "Bilgi bulun, metinleri karşılaştırın ve bağlantı önerilerini inceleyin. Bu isteğe bağlı panel AIRust kütüphanesini kullanır."),
            ("landing.badge.version", "en", "v0.1.7", concat!("v", env!("CARGO_PKG_VERSION"))),
            ("landing.badge.version", "de", "v0.1.7", concat!("v", env!("CARGO_PKG_VERSION"))),
            ("landing.badge.version", "tr", "v0.1.7", concat!("v", env!("CARGO_PKG_VERSION"))),
            ("landing.badge.rust", "en", "Rust 1.85+", "Rust 1.89+"),
            ("landing.badge.rust", "de", "Rust 1.85+", "Rust 1.89+"),
            ("landing.badge.rust", "tr", "Rust 1.85+", "Rust 1.89+"),
            ("landing.badge.tests", "en", "156 Tests", "Tests & CI"),
            ("landing.badge.tests", "de", "156 Tests", "Tests & CI"),
            ("landing.badge.tests", "tr", "156 Test", "Testler & CI"),
            ("landing.claim", "en", "100% Local \u{00b7} Zero Cloud Dependencies \u{00b7} Fully Private", "Local core \u{00b7} Optional integrations"),
            ("landing.claim", "de", "100% Lokal \u{00b7} Keine Cloud-Abhängigkeiten \u{00b7} Vollständig Privat", "Lokaler Kern \u{00b7} Optionale Erweiterungen"),
            ("landing.claim", "tr", "100% Yerel \u{00b7} Sıfır Bulut Bağımlılığı \u{00b7} Tamamen Gizli", "Yerel çekirdek \u{00b7} İsteğe bağlı eklentiler"),
            ("landing.features.sub", "en", "Everything you need to build, train, and deploy an AI system", "Optional tools around the AIRust knowledge library"),
            ("landing.features.sub", "de", "Alles, was du brauchst, um ein KI-System zu bauen, zu trainieren und bereitzustellen", "Optionale Werkzeuge rund um die AIRust-Wissensbibliothek"),
            ("landing.features.sub", "tr", "Bir yapay zeka sistemi oluşturmak, eğitmek ve dağıtmak için ihtiyacınız olan her şey", "AIRust bilgi kütüphanesi için isteğe bağlı araçlar"),
            ("landing.feat.agents.desc", "en", "Exact Match, Fuzzy Match, TF-IDF/BM25 Semantic, Context-Aware", "Exact Match, Fuzzy Match, BM25 lexical search, Context-Aware"),
            ("landing.feat.agents.desc", "de", "Exakt, Fuzzy, TF-IDF/BM25 Semantisch, Kontextbewusst", "Exakt, Fuzzy, BM25-Wortsuche, Kontextbewusst"),
            ("landing.feat.agents.desc", "tr", "Tam Eşleşme, Bulanık Eşleşme, TF-IDF/BM25 Semantik, Bağlam Duyarlı", "Tam Eşleşme, Bulanık Eşleşme, BM25 kelime arama, Bağlam Duyarlı"),
            ("landing.agent.tfidf.desc", "en", "Semantic search using TF-IDF and BM25 ranking. Understands meaning, not just words. Handles synonyms and related concepts.", "Lexical BM25 ranking using shared terms. Synonyms and meaning require an optional external model."),
            ("landing.agent.tfidf.desc", "de", "Semantische Suche mit TF-IDF und BM25-Ranking. Versteht Bedeutung, nicht nur Wörter. Verarbeitet Synonyme und verwandte Konzepte.", "BM25 ordnet Treffer nach gemeinsamen Wörtern. Für Synonyme und Bedeutung ist ein optionales externes Modell nötig."),
            ("landing.agent.tfidf.desc", "tr", "TF-IDF ve BM25 sıralaması ile semantik arama. Sadece kelimeleri değil, anlamı anlar. Eş anlamlıları ve ilgili kavramları işler.", "BM25 ortak kelimelere göre sıralar. Anlam ve eş anlamlılar için isteğe bağlı harici bir model gerekir."),
            ("landing.bench.tests.label", "en", "Tests Passing", "Automated Checks"),
            ("landing.bench.tests.label", "de", "Bestandene Tests", "Automatische Prüfungen"),
            ("landing.bench.tests.label", "tr", "Başarılı Testler", "Otomatik kontroller"),
            ("landing.bench.apis.label", "en", "External APIs", "Required LLM APIs"),
            ("landing.bench.apis.label", "de", "Externe APIs", "Benötigte LLM-APIs"),
            ("landing.bench.apis.label", "tr", "Harici API'ler", "Gerekli LLM API'leri"),
            ("landing.bench.local.label", "en", "Local & Private", "Local Core"),
            ("landing.bench.local.label", "de", "Lokal & Privat", "Lokaler Kern"),
            ("landing.bench.local.label", "tr", "Yerel & Gizli", "Yerel çekirdek"),
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
            // Hero
            ("landing.tagline", "en", "A Modular Knowledge Intelligence Engine"),
            ("landing.tagline", "de", "Wissen finden, vergleichen und verbinden"),
            ("landing.tagline", "tr", "Rust ile modüler bilgi arama ve karşılaştırma"),
            ("landing.desc", "en", "Find knowledge, compare text, and propose connections. This optional dashboard uses the AIRust Rust library."),
            ("landing.desc", "de", "Finde Wissen, vergleiche Texte und prüfe mögliche Verbindungen. Dieses optionale Dashboard verwendet die AIRust-Bibliothek."),
            ("landing.desc", "tr", "Bilgi bulun, metinleri karşılaştırın ve bağlantı önerilerini inceleyin. Bu isteğe bağlı panel AIRust kütüphanesini kullanır."),
            ("landing.badge.version", "en", concat!("v", env!("CARGO_PKG_VERSION"))),
            ("landing.badge.version", "de", concat!("v", env!("CARGO_PKG_VERSION"))),
            ("landing.badge.version", "tr", concat!("v", env!("CARGO_PKG_VERSION"))),
            ("landing.badge.license", "en", "MIT License"),
            ("landing.badge.license", "de", "MIT-Lizenz"),
            ("landing.badge.license", "tr", "MIT Lisansı"),
            ("landing.badge.rust", "en", "Rust 1.89+"),
            ("landing.badge.rust", "de", "Rust 1.89+"),
            ("landing.badge.rust", "tr", "Rust 1.89+"),
            ("landing.badge.tests", "en", "Tests & CI"),
            ("landing.badge.tests", "de", "Tests & CI"),
            ("landing.badge.tests", "tr", "Testler & CI"),
            ("landing.claim", "en", "Local core \u{00b7} Optional integrations"),
            ("landing.claim", "de", "Lokaler Kern \u{00b7} Optionale Erweiterungen"),
            ("landing.claim", "tr", "Yerel çekirdek \u{00b7} İsteğe bağlı eklentiler"),
            ("landing.cta", "en", "Try it now"),
            ("landing.cta", "de", "Jetzt ausprobieren"),
            ("landing.cta", "tr", "Şimdi deneyin"),
            // Features
            ("landing.features.title", "en", "Features"),
            ("landing.features.title", "de", "Funktionen"),
            ("landing.features.title", "tr", "Özellikler"),
            ("landing.features.sub", "en", "Optional tools around the AIRust knowledge library"),
            ("landing.features.sub", "de", "Optionale Werkzeuge rund um die AIRust-Wissensbibliothek"),
            ("landing.features.sub", "tr", "AIRust bilgi kütüphanesi için isteğe bağlı araçlar"),
            ("landing.feat.agents.name", "en", "4 Agent Types"),
            ("landing.feat.agents.name", "de", "4 Agententypen"),
            ("landing.feat.agents.name", "tr", "4 Ajan Türü"),
            ("landing.feat.agents.desc", "en", "Exact Match, Fuzzy Match, BM25 lexical search, Context-Aware"),
            ("landing.feat.agents.desc", "de", "Exakt, Fuzzy, BM25-Wortsuche, Kontextbewusst"),
            ("landing.feat.agents.desc", "tr", "Tam Eşleşme, Bulanık Eşleşme, BM25 kelime arama, Bağlam Duyarlı"),
            ("landing.feat.kb.name", "en", "Knowledge Base"),
            ("landing.feat.kb.name", "de", "Wissensbasis"),
            ("landing.feat.kb.name", "tr", "Bilgi Tabanı"),
            ("landing.feat.kb.desc", "en", "JSON-based, compile-time embedded, runtime expandable"),
            ("landing.feat.kb.desc", "de", "JSON-basiert, zur Kompilierzeit eingebettet, zur Laufzeit erweiterbar"),
            ("landing.feat.kb.desc", "tr", "JSON tabanlı, derleme zamanı gömülü, çalışma zamanı genişletilebilir"),
            ("landing.feat.pdf.name", "en", "PDF Processing"),
            ("landing.feat.pdf.name", "de", "PDF-Verarbeitung"),
            ("landing.feat.pdf.name", "tr", "PDF İşleme"),
            ("landing.feat.pdf.desc", "en", "Convert PDFs to structured training data with smart chunking"),
            ("landing.feat.pdf.desc", "de", "PDFs in strukturierte Trainingsdaten mit intelligentem Chunking umwandeln"),
            ("landing.feat.pdf.desc", "tr", "PDF'leri akıllı parçalama ile yapılandırılmış eğitim verisine dönüştürün"),
            ("landing.feat.dashboard.name", "en", "Web Dashboard"),
            ("landing.feat.dashboard.name", "de", "Web-Dashboard"),
            ("landing.feat.dashboard.name", "tr", "Web Paneli"),
            ("landing.feat.dashboard.desc", "en", "Full UI with chat, training manager, bot control, file browser"),
            ("landing.feat.dashboard.desc", "de", "Vollständige Oberfläche mit Chat, Trainingsmanager, Bot-Steuerung, Dateibrowser"),
            ("landing.feat.dashboard.desc", "tr", "Sohbet, eğitim yöneticisi, bot kontrolü, dosya tarayıcısı ile tam arayüz"),
            ("landing.feat.bots.name", "en", "Bot Ecosystem"),
            ("landing.feat.bots.name", "de", "Bot-Ökosystem"),
            ("landing.feat.bots.name", "tr", "Bot Ekosistemi"),
            ("landing.feat.bots.desc", "en", "Automated web scraping with review workflow"),
            ("landing.feat.bots.desc", "de", "Automatisches Web-Scraping mit Prüfungs-Workflow"),
            ("landing.feat.bots.desc", "tr", "İnceleme iş akışı ile otomatik web kazıma"),
            ("landing.feat.vectordb.name", "en", "Vector Database"),
            ("landing.feat.vectordb.name", "de", "Vektor-Datenbank"),
            ("landing.feat.vectordb.name", "tr", "Vektör Veritabanı"),
            ("landing.feat.vectordb.desc", "en", "Embedding storage and similarity search"),
            ("landing.feat.vectordb.desc", "de", "Einbettungsspeicher und Ähnlichkeitssuche"),
            ("landing.feat.vectordb.desc", "tr", "Gömme depolama ve benzerlik araması"),
            ("landing.feat.chat.name", "en", "Chat History"),
            ("landing.feat.chat.name", "de", "Chatverlauf"),
            ("landing.feat.chat.name", "tr", "Sohbet Geçmişi"),
            ("landing.feat.chat.desc", "en", "Persistent conversations with archiving"),
            ("landing.feat.chat.desc", "de", "Dauerhafte Gespräche mit Archivierung"),
            ("landing.feat.chat.desc", "tr", "Arşivleme ile kalıcı konuşmalar"),
            ("landing.feat.multilang.name", "en", "Multi-Language UI"),
            ("landing.feat.multilang.name", "de", "Mehrsprachige Oberfläche"),
            ("landing.feat.multilang.name", "tr", "Çok Dilli Arayüz"),
            ("landing.feat.multilang.desc", "en", "English, German, Turkish"),
            ("landing.feat.multilang.desc", "de", "Englisch, Deutsch, Türkisch"),
            ("landing.feat.multilang.desc", "tr", "İngilizce, Almanca, Türkçe"),
            ("landing.feat.api.name", "en", "REST API"),
            ("landing.feat.api.name", "de", "REST-API"),
            ("landing.feat.api.name", "tr", "REST API"),
            ("landing.feat.api.desc", "en", "50+ endpoints for full programmatic control"),
            ("landing.feat.api.desc", "de", "50+ Endpunkte für vollständige programmatische Steuerung"),
            ("landing.feat.api.desc", "tr", "Tam programatik kontrol için 50+ uç nokta"),
            ("landing.feat.console.name", "en", "WebSocket Console"),
            ("landing.feat.console.name", "de", "WebSocket-Konsole"),
            ("landing.feat.console.name", "tr", "WebSocket Konsolu"),
            ("landing.feat.console.desc", "en", "Live terminal with server logs, shell access, built-in commands"),
            ("landing.feat.console.desc", "de", "Live-Terminal mit Server-Logs, Shell-Zugang, eingebauten Befehlen"),
            ("landing.feat.console.desc", "tr", "Sunucu günlükleri, kabuk erişimi, yerleşik komutlarla canlı terminal"),
            ("landing.feat.docker.name", "en", "Docker Support"),
            ("landing.feat.docker.name", "de", "Docker-Unterstützung"),
            ("landing.feat.docker.name", "tr", "Docker Desteği"),
            ("landing.feat.docker.desc", "en", "One-command deployment with containers"),
            ("landing.feat.docker.desc", "de", "Ein-Befehl-Bereitstellung mit Containern"),
            ("landing.feat.docker.desc", "tr", "Konteynerlerle tek komutla dağıtım"),
            ("landing.feat.cli.name", "en", "CLI Tools"),
            ("landing.feat.cli.name", "de", "CLI-Werkzeuge"),
            ("landing.feat.cli.name", "tr", "CLI Araçları"),
            ("landing.feat.cli.desc", "en", "Interactive mode, query tools, PDF conversion"),
            ("landing.feat.cli.desc", "de", "Interaktiver Modus, Abfragewerkzeuge, PDF-Konvertierung"),
            ("landing.feat.cli.desc", "tr", "Etkileşimli mod, sorgulama araçları, PDF dönüştürme"),
            // Architecture
            ("landing.arch.title", "en", "Architecture"),
            ("landing.arch.title", "de", "Architektur"),
            ("landing.arch.title", "tr", "Mimari"),
            ("landing.arch.sub", "en", "A layered architecture: agents do the thinking, the knowledge base stores the data, and the web server provides the interface"),
            ("landing.arch.sub", "de", "Eine Schichtarchitektur: Agenten denken, die Wissensbasis speichert die Daten, und der Webserver liefert die Oberfläche"),
            ("landing.arch.sub", "tr", "Katmanlı bir mimari: ajanlar düşünür, bilgi tabanı verileri saklar ve web sunucusu arayüzü sağlar"),
            ("landing.traits.th.trait", "en", "Trait"),
            ("landing.traits.th.trait", "de", "Trait"),
            ("landing.traits.th.trait", "tr", "Trait"),
            ("landing.traits.th.purpose", "en", "Purpose"),
            ("landing.traits.th.purpose", "de", "Zweck"),
            ("landing.traits.th.purpose", "tr", "Amaç"),
            ("landing.traits.agent.purpose", "en", "Base trait: predict(), confidence(), can_answer()"),
            ("landing.traits.agent.purpose", "de", "Basis-Trait: predict(), confidence(), can_answer()"),
            ("landing.traits.agent.purpose", "tr", "Temel trait: predict(), confidence(), can_answer()"),
            ("landing.traits.trainable.purpose", "en", "Adds train(), add_example()"),
            ("landing.traits.trainable.purpose", "de", "Fügt train(), add_example() hinzu"),
            ("landing.traits.trainable.purpose", "tr", "train(), add_example() ekler"),
            ("landing.traits.contextual.purpose", "en", "Adds add_context(), clear_context() for conversation memory"),
            ("landing.traits.contextual.purpose", "de", "Fügt add_context(), clear_context() für Gesprächsspeicher hinzu"),
            ("landing.traits.contextual.purpose", "tr", "Konuşma belleği için add_context(), clear_context() ekler"),
            ("landing.traits.confidence.purpose", "en", "Adds calculate_confidence(), predict_top_n()"),
            ("landing.traits.confidence.purpose", "de", "Fügt calculate_confidence(), predict_top_n() hinzu"),
            ("landing.traits.confidence.purpose", "tr", "calculate_confidence(), predict_top_n() ekler"),
            // Agent Types
            ("landing.agents.title", "en", "Agent Types"),
            ("landing.agents.title", "de", "Agententypen"),
            ("landing.agents.title", "tr", "Ajan Türleri"),
            ("landing.agents.sub", "en", "The brain of AIRust \u{2014} choose the right agent for your use case"),
            ("landing.agents.sub", "de", "Das Gehirn von AIRust \u{2014} wähle den richtigen Agenten für deinen Anwendungsfall"),
            ("landing.agents.sub", "tr", "AIRust'ın beyni \u{2014} kullanım durumunuz için doğru ajanı seçin"),
            ("landing.agent.match.title", "en", "MatchAgent"),
            ("landing.agent.match.title", "de", "MatchAgent"),
            ("landing.agent.match.title", "tr", "MatchAgent"),
            ("landing.agent.match.desc", "en", "The simplest and fastest agent. Compares questions directly against training data using exact or fuzzy matching with Levenshtein distance."),
            ("landing.agent.match.desc", "de", "Der einfachste und schnellste Agent. Vergleicht Fragen direkt mit Trainingsdaten mittels exakter oder unscharfer Übereinstimmung mit Levenshtein-Distanz."),
            ("landing.agent.match.desc", "tr", "En basit ve en hızlı ajan. Soruları, Levenshtein mesafesi ile tam veya bulanık eşleştirme kullanarak doğrudan eğitim verileriyle karşılaştırır."),
            ("landing.agent.match.use", "en", "Best for: FAQ bots, command recognition, structured Q&A"),
            ("landing.agent.match.use", "de", "Ideal für: FAQ-Bots, Befehlserkennung, strukturierte Fragen & Antworten"),
            ("landing.agent.match.use", "tr", "En iyi: SSS botları, komut tanıma, yapılandırılmış Soru & Cevap"),
            ("landing.agent.tfidf.title", "en", "TfidfAgent"),
            ("landing.agent.tfidf.title", "de", "TfidfAgent"),
            ("landing.agent.tfidf.title", "tr", "TfidfAgent"),
            ("landing.agent.tfidf.desc", "en", "Lexical BM25 ranking using shared terms. Synonyms and meaning require an optional external model."),
            ("landing.agent.tfidf.desc", "de", "BM25 ordnet Treffer nach gemeinsamen Wörtern. Für Synonyme und Bedeutung ist ein optionales externes Modell nötig."),
            ("landing.agent.tfidf.desc", "tr", "BM25 ortak kelimelere göre sıralar. Anlam ve eş anlamlılar için isteğe bağlı harici bir model gerekir."),
            ("landing.agent.tfidf.use", "en", "Best for: Natural language Q&A, documentation search"),
            ("landing.agent.tfidf.use", "de", "Ideal für: Natürlichsprachige Fragen & Antworten, Dokumentationssuche"),
            ("landing.agent.tfidf.use", "tr", "En iyi: Doğal dil Soru & Cevap, dokümantasyon araması"),
            ("landing.agent.context.title", "en", "ContextAgent"),
            ("landing.agent.context.title", "de", "ContextAgent"),
            ("landing.agent.context.title", "tr", "ContextAgent"),
            ("landing.agent.context.desc", "en", "Wraps any agent and adds conversation memory. Maintains context across messages for multi-turn dialogue."),
            ("landing.agent.context.desc", "de", "Umhüllt jeden Agenten und fügt Gesprächsspeicher hinzu. Hält den Kontext über Nachrichten hinweg für mehrstufige Dialoge."),
            ("landing.agent.context.desc", "tr", "Herhangi bir ajanı sarar ve konuşma belleği ekler. Çok turlu diyalog için mesajlar arasında bağlamı korur."),
            ("landing.agent.context.use", "en", "Best for: Chatbots, support agents, interactive assistants"),
            ("landing.agent.context.use", "de", "Ideal für: Chatbots, Support-Agenten, interaktive Assistenten"),
            ("landing.agent.context.use", "tr", "En iyi: Sohbet botları, destek ajanları, etkileşimli asistanlar"),
            // Benchmarks
            ("landing.bench.title", "en", "By the Numbers"),
            ("landing.bench.title", "de", "In Zahlen"),
            ("landing.bench.title", "tr", "Rakamlarla"),
            ("landing.bench.sub", "en", "Built for performance, privacy, and simplicity"),
            ("landing.bench.sub", "de", "Gebaut für Leistung, Datenschutz und Einfachheit"),
            ("landing.bench.sub", "tr", "Performans, gizlilik ve basitlik için tasarlandı"),
            ("landing.bench.tests.label", "en", "Automated Checks"),
            ("landing.bench.tests.label", "de", "Automatische Prüfungen"),
            ("landing.bench.tests.label", "tr", "Otomatik kontroller"),
            ("landing.bench.apis.label", "en", "Required LLM APIs"),
            ("landing.bench.apis.label", "de", "Benötigte LLM-APIs"),
            ("landing.bench.apis.label", "tr", "Gerekli LLM API'leri"),
            ("landing.bench.endpoints.label", "en", "REST Endpoints"),
            ("landing.bench.endpoints.label", "de", "REST-Endpunkte"),
            ("landing.bench.endpoints.label", "tr", "REST Uç Noktaları"),
            ("landing.bench.local.label", "en", "Local Core"),
            ("landing.bench.local.label", "de", "Lokaler Kern"),
            ("landing.bench.local.label", "tr", "Yerel çekirdek"),
            // Installation
            ("landing.install.title", "en", "Get Started"),
            ("landing.install.title", "de", "Loslegen"),
            ("landing.install.title", "tr", "Başlayın"),
            ("landing.install.sub", "en", "Up and running in seconds"),
            ("landing.install.sub", "de", "In Sekunden einsatzbereit"),
            ("landing.install.sub", "tr", "Saniyeler içinde çalışır durumda"),
            // Showroom
            ("landing.showroom", "en", "SHOWROOM"),
            ("landing.showroom", "de", "SHOWROOM"),
            ("landing.showroom", "tr", "VİTRİN"),
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

    // ── Categories CRUD ──────────────────────────────────────────────────────

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

    // ── Training Data CRUD ───────────────────────────────────────────────────

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

    /// Persist the complete example, including PDF provenance, across restarts.
    pub fn add_training_example(
        &self,
        category_id: Option<i64>,
        example: &TrainingExample,
    ) -> Result<i64, String> {
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

    pub fn get_all_training_data(&self) -> Result<Vec<TrainingDataRow>, String> {
        self.list_training_data(None)
    }

    // ── Vector Collections CRUD ──────────────────────────────────────────────

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

    // ── Vector Entries CRUD ──────────────────────────────────────────────────

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
