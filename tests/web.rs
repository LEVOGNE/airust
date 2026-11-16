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
    connection.execute("UPDATE translations SET text = 'v0.1.7' WHERE key = 'landing.badge.version' AND language_code = 'en'", []).unwrap();
    connection.execute("UPDATE translations SET text = 'My custom description' WHERE key = 'agent.tfidf.desc' AND language_code = 'en'", []).unwrap();
    drop(connection);
    let database = Database::open(path).unwrap();
    let translations = database.get_translations("en").unwrap();
    assert!(translations
        .iter()
        .any(|(key, text)| key == "landing.badge.version"
            && text == &format!("v{}", airust::VERSION)));
    assert!(translations
        .iter()
        .any(|(key, text)| key == "agent.tfidf.desc" && text == "My custom description"));
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

#[test]
fn invalid_training_batches_cannot_partially_update_the_database() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("training.db");
    let database = Database::open(path.to_str().unwrap()).unwrap();
    database
        .add_training_data(None, "existing", "answer", "text", 1.0)
        .unwrap();
    let valid = TrainingExample {
        input: "valid".to_owned(),
        output: "answer".into(),
        weight: 1.0,
        metadata: None,
    };
    let mut invalid = valid.clone();
    invalid.weight = f32::NAN;
    assert!(database
        .add_training_batch(&[(None, valid.clone()), (None, invalid)])
        .is_err());
    assert_eq!(database.get_all_training_data().unwrap().len(), 1);
    assert!(database
        .add_training_data(None, "invalid", "answer", "text", f64::INFINITY)
        .is_err());
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute(
            "CREATE UNIQUE INDEX unique_training_input ON training_data(input)",
            [],
        )
        .unwrap();
    drop(connection);
    assert!(database
        .add_training_batch(&[(None, valid.clone()), (None, valid)])
        .is_err());
    let rows = database.get_all_training_data().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].input, "existing");
}

#[tokio::test]
async fn scheduler_registers_tasks_before_they_can_finish() {
    use airust::web::{bots::models::BotConfig, bots::scheduler::BotScheduler};
    use std::sync::Arc;
    let temp = tempfile::tempdir().unwrap();
    let database = Arc::new(Database::open(temp.path().join("bots.db").to_str().unwrap()).unwrap());
    let id = database
        .create_bot("Processor", "processor", &BotConfig::default())
        .unwrap();
    let scheduler = BotScheduler::new(database);
    let (first, second) = tokio::join!(scheduler.start_bot(id), scheduler.start_bot(id));
    assert!(first.is_ok());
    assert!(second.is_err());
    for _ in 0..100 {
        if !scheduler.is_running(id).await {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(2)).await;
    }
    panic!("completed processor remained registered as running");
}
