use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use super::crawler::CrawlerBot;
use super::processor::ProcessorBot;
use crate::web::db::Database;

pub struct BotScheduler {
    running: Arc<Mutex<HashMap<i64, JoinHandle<()>>>>,
    db: Arc<Database>,
}

impl BotScheduler {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            running: Arc::new(Mutex::new(HashMap::new())),
            db,
        }
    }

    pub async fn start_bot(&self, bot_id: i64) -> Result<(), String> {
        let bot = self
            .db
            .get_bot(bot_id)?
            .ok_or_else(|| "Bot not found".to_string())?;

        let mut running_guard = self.running.lock().await;
        if running_guard.contains_key(&bot_id) {
            return Err("Bot is already running".to_owned());
        }
        let run_id = self.db.create_bot_run(bot_id)?;
        let db = self.db.clone();
        let running = self.running.clone();
        let config = bot.config.clone();
        let bot_type = bot.bot_type.clone();

        let handle = tokio::spawn(async move {
            let result = match bot_type.as_str() {
                "crawler" => CrawlerBot::run(&config, &db, run_id).await,
                "processor" => {
                    let db2 = db.clone();
                    let config2 = config.clone();
                    tokio::task::spawn_blocking(move || ProcessorBot::process(&db2, &config2))
                        .await
                        .map_err(|e| format!("Task error: {}", e))
                        .and_then(|r| r)
                }
                _ => Err(format!("Unknown bot type: {}", bot_type)),
            };

            match result {
                Ok((found, added)) => {
                    let _ = db.finish_bot_run(run_id, "success", found, added, None);
                }
                Err(e) => {
                    let _ = db.finish_bot_run(run_id, "error", 0, 0, Some(&e));
                }
            }

            let mut running = running.lock().await;
            running.remove(&bot_id);
        });

        running_guard.insert(bot_id, handle);

        Ok(())
    }

    pub async fn stop_bot(&self, bot_id: i64) -> Result<(), String> {
        let mut running = self.running.lock().await;
        if let Some(handle) = running.remove(&bot_id) {
            handle.abort();
            Ok(())
        } else {
            Err("Bot is not running".to_string())
        }
    }

    pub async fn is_running(&self, bot_id: i64) -> bool {
        let running = self.running.lock().await;
        running.contains_key(&bot_id)
    }

    pub async fn running_bot_ids(&self) -> Vec<i64> {
        let running = self.running.lock().await;
        running.keys().cloned().collect()
    }
}
