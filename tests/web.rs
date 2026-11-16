#![cfg(feature = "web")]

use airust::web::db::Database;
use airust::{ResponseFormat, TrainingExample};
use serde_json::json;

#[test]
fn product_text_migration_updates_defaults_and_preserves_custom_translations() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("test.db");
    let path = path.to_str().unwrap();
    drop(Database::open(path).unwrap());
    let connection = rusqlite::Connection::open(path).unwrap();
    connection.execute("UPDATE translations SET text = 'v0.1.7' WHERE key = 'landing.badge.version' AND language_code = 'de'", []).unwrap();
    connection.execute("UPDATE translations SET text = 'Meine eigene Beschreibung' WHERE key = 'agent.tfidf.desc' AND language_code = 'de'", []).unwrap();
    drop(connection);
    let database = Database::open(path).unwrap();
    let translations = database.get_translations("de").unwrap();
    assert!(translations
        .iter()
        .any(|(key, text)| key == "landing.badge.version" && text == "v0.1.8"));
    assert!(translations
        .iter()
        .any(|(key, text)| key == "agent.tfidf.desc" && text == "Meine eigene Beschreibung"));
}

#[test]
fn legacy_database_migration_keeps_rows_and_persists_metadata_after_restart() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("legacy.db");
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection.execute_batch("CREATE TABLE training_data (
        id INTEGER PRIMARY KEY AUTOINCREMENT, category_id INTEGER, input TEXT NOT NULL,
        output_text TEXT NOT NULL, output_format TEXT NOT NULL DEFAULT 'text',
        weight REAL NOT NULL DEFAULT 1.0, created_at TEXT NOT NULL DEFAULT (datetime('now'))
    ); INSERT INTO training_data (input, output_text, weight) VALUES ('old input', 'old answer', 2.0);").unwrap();
    drop(connection);
    let database = Database::open(path.to_str().unwrap()).unwrap();
    let original = database.get_all_training_data().unwrap().remove(0);
    assert_eq!(original.id, 1);
    assert_eq!(original.weight, 2.0);
    assert!(original.metadata.is_none());
    let example = TrainingExample {
        input: "PDF passage".to_owned(),
        output: ResponseFormat::Json(json!({"answer": "content"})),
        weight: 1.5,
        metadata: Some(json!({"source_file": "sample.pdf", "page_number": 3})),
    };
    database.add_training_example(None, &example).unwrap();
    drop(database);
    let database = Database::open(path.to_str().unwrap()).unwrap();
    let rows = database.get_all_training_data().unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[1].input, "old input");
    let exported = serde_json::to_string(&rows).unwrap();
    let imported: Vec<airust::web::routes::TrainingImportItem> =
        serde_json::from_str(&exported).unwrap();
    assert_eq!(imported[0].format, "json");
    assert_eq!(imported[0].metadata, example.metadata);
    let restored = rows.into_iter().next().unwrap().into_example();
    assert_eq!(restored.metadata, example.metadata);
    assert_eq!(restored.output.to_string(), example.output.to_string());
}
