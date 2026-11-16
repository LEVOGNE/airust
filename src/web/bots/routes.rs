use super::models::BotConfig;
use super::vectordb::VectorDB;
use crate::web::state::SharedState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct BotResponse {
    pub id: i64,
    pub name: String,
    pub bot_type: String,
    pub config: BotConfig,
    pub enabled: bool,
    pub created_at: String,
    pub running: bool,
}

pub async fn list_bots(
    State(state): State<SharedState>,
) -> Result<Json<Vec<BotResponse>>, (StatusCode, String)> {
    let s = state.read().await;
    let bots =
        s.db.list_bots()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let running_ids = s.scheduler.running_bot_ids().await;
    let result: Vec<BotResponse> = bots
        .into_iter()
        .map(|b| BotResponse {
            running: running_ids.contains(&b.id),
            id: b.id,
            name: b.name,
            bot_type: b.bot_type,
            config: b.config,
            enabled: b.enabled,
            created_at: b.created_at,
        })
        .collect();
    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct CreateBotRequest {
    pub name: String,
    pub bot_type: String,
    #[serde(default)]
    pub config: BotConfig,
}

#[derive(Serialize)]
pub struct CreateBotResponse {
    pub id: i64,
}

pub async fn create_bot(
    State(state): State<SharedState>,
    Json(req): Json<CreateBotRequest>,
) -> Result<Json<CreateBotResponse>, (StatusCode, String)> {
    let s = state.read().await;
    let id =
        s.db.create_bot(&req.name, &req.bot_type, &req.config)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(CreateBotResponse { id }))
}

pub async fn get_bot(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
) -> Result<Json<BotResponse>, (StatusCode, String)> {
    let s = state.read().await;
    let bot =
        s.db.get_bot(id)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?
            .ok_or((StatusCode::NOT_FOUND, "Bot not found".to_string()))?;
    let running = s.scheduler.is_running(id).await;
    Ok(Json(BotResponse {
        id: bot.id,
        name: bot.name,
        bot_type: bot.bot_type,
        config: bot.config,
        enabled: bot.enabled,
        created_at: bot.created_at,
        running,
    }))
}

#[derive(Deserialize)]
pub struct UpdateBotRequest {
    pub name: String,
    pub bot_type: String,
    #[serde(default)]
    pub config: BotConfig,
}

pub async fn update_bot(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateBotRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state.read().await;
    s.db.update_bot(id, &req.name, &req.bot_type, &req.config)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(serde_json::json!({"success": true})))
}

pub async fn delete_bot(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state.read().await;
    s.db.delete_bot(id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(serde_json::json!({"success": true})))
}

pub async fn start_bot(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state.read().await;
    s.scheduler
        .start_bot(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(serde_json::json!({"success": true})))
}

pub async fn stop_bot(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state.read().await;
    s.scheduler
        .stop_bot(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(serde_json::json!({"success": true})))
}

pub async fn get_bot_runs(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<super::models::BotRun>>, (StatusCode, String)> {
    let s = state.read().await;
    let runs =
        s.db.get_bot_runs(id)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(runs))
}

pub async fn get_run_data(
    State(state): State<SharedState>,
    Path((_bot_id, run_id)): Path<(i64, i64)>,
) -> Result<Json<Vec<super::models::RawData>>, (StatusCode, String)> {
    let s = state.read().await;
    let data =
        s.db.get_raw_data_by_run_id(run_id)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(data))
}

pub async fn get_pending_data(
    State(state): State<SharedState>,
) -> Result<Json<Vec<super::models::RawData>>, (StatusCode, String)> {
    let s = state.read().await;
    let data =
        s.db.get_pending_raw_data()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(data))
}

pub async fn approve_data(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state.read().await;
    s.db.approve_raw_data(id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(serde_json::json!({"success": true})))
}

pub async fn reject_data(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state.read().await;
    s.db.reject_raw_data(id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(serde_json::json!({"success": true})))
}

pub async fn approve_all_data(
    State(state): State<SharedState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state.read().await;
    let count =
        s.db.approve_all_raw_data()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(
        serde_json::json!({"success": true, "approved": count}),
    ))
}

pub async fn get_approved_data(
    State(state): State<SharedState>,
) -> Result<Json<Vec<super::models::ApprovedData>>, (StatusCode, String)> {
    let s = state.read().await;
    let data =
        s.db.get_approved_data()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(data))
}

use crate::agent::ResponseFormat;

pub async fn add_to_kb(
    State(state): State<SharedState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut s = state.write().await;
    let unadded =
        s.db.get_unadded_approved_data()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let mut added = 0;
    for item in &unadded {
        s.knowledge_base.add_example(
            item.input.clone(),
            ResponseFormat::Text(item.output.clone()),
            item.weight as f32,
        );
        s.db.mark_added_to_kb(item.id)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
        added += 1;
    }

    let examples = s.knowledge_base.get_examples().to_vec();
    s.train_all(&examples);

    Ok(Json(
        serde_json::json!({"success": true, "added": added, "total": examples.len()}),
    ))
}

pub async fn vector_stats(
    State(state): State<SharedState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state.read().await;
    let (total, terms) =
        s.db.get_vector_stats()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(serde_json::json!({"vectors": total, "terms": terms})))
}

pub async fn rebuild_vectors(
    State(state): State<SharedState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state.read().await;
    let db = s.db.clone();
    drop(s);
    let (docs, vectors) = tokio::task::spawn_blocking(move || VectorDB::rebuild_all(&db))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Task error: {}", e),
            )
        })?
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(
        serde_json::json!({"success": true, "documents": docs, "vectors": vectors}),
    ))
}

#[derive(Deserialize)]
pub struct SearchRequest {
    pub query: String,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
}

fn default_top_k() -> usize {
    5
}

pub async fn search_vectors(
    State(state): State<SharedState>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<Vec<super::vectordb::SearchResult>>, (StatusCode, String)> {
    let s = state.read().await;
    let db = s.db.clone();
    drop(s);
    let results = tokio::task::spawn_blocking(move || VectorDB::search(&db, &req.query, req.top_k))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Task error: {}", e),
            )
        })?
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(results))
}
