use crate::agent::ResponseFormat;
use crate::knowledge::KnowledgeBase;
use crate::pdf_loader::PdfLoader;
use crate::web::state::SharedState;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::StatusCode;
use axum::response::Html;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub async fn index(State(state): State<SharedState>) -> Html<String> {
    let s = state.read().await;
    let html = include_str!("static/index.html").replace("{{AIRUST_VERSION}}", crate::VERSION);
    if s.show_landing {
        Html(html.to_string())
    } else {
        Html(html.replace(
            "</head>",
            "<style>#landing{display:none!important}</style></head>",
        ))
    }
}

#[derive(Serialize)]
pub struct StatusResponse {
    agent: String,
    active_agents: Vec<String>,
    examples: usize,
    version: String,
}

pub async fn status(State(state): State<SharedState>) -> Json<StatusResponse> {
    let s = state.read().await;
    Json(StatusResponse {
        agent: s.active_agents_display(),
        active_agents: s.active_agents.clone(),
        examples: s.knowledge_base.get_examples().len(),
        version: crate::VERSION.to_string(),
    })
}

#[derive(Deserialize)]
pub struct QueryRequest {
    input: String,
    #[serde(default)]
    add_context: bool,
    #[serde(default)]
    chat_id: Option<i64>,
}

#[derive(Serialize)]
pub struct SettingsChange {
    key: String,
    value: String,
}

#[derive(Serialize)]
pub struct QueryResponse {
    response: String,
    confidence: f32,
    agent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    agents_used: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    settings_changed: Option<Vec<SettingsChange>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<serde_json::Value>,
}

struct SettingsCommand {
    changes: Vec<(String, String)>,
    description: String,
}

fn detect_settings_command(input: &str, current_theme: &str) -> Option<SettingsCommand> {
    let lower = input.strip_prefix("/settings ")?.to_ascii_lowercase();
    let mut changes = Vec::new();
    let mut descriptions = Vec::new();
    if lower.split_whitespace().any(|word| word == "dark") {
        changes.push(("theme".to_owned(), "dark".to_owned()));
        descriptions.push("Dark theme");
    } else if lower.split_whitespace().any(|word| word == "light") {
        changes.push(("theme".to_owned(), "light".to_owned()));
        descriptions.push("Light theme");
    }
    let theme = if current_theme == "light" {
        "light"
    } else {
        "dark"
    };
    for (name, hex) in [
        ("green", "#44d7a8"),
        ("red", "#e05566"),
        ("blue", "#4488ee"),
        ("purple", "#7c6ef0"),
        ("orange", "#e08844"),
        ("pink", "#e055aa"),
        ("yellow", "#e0c844"),
        ("cyan", "#44c8d7"),
        ("black", "#0a0a0a"),
        ("white", "#f5f5f5"),
    ] {
        if lower.split_whitespace().any(|word| word == name) {
            if lower.split_whitespace().any(|word| word == "background") {
                changes.push((format!("{theme}.bg_color"), hex.to_owned()));
                changes.push((format!("{theme}.bg_auto"), "false".to_owned()));
            } else {
                changes.push((format!("{theme}.accent_color"), hex.to_owned()));
            }
            descriptions.push(name);
            break;
        }
    }
    if changes.is_empty() {
        None
    } else {
        Some(SettingsCommand {
            changes,
            description: format!("Settings updated: {}", descriptions.join(", ")),
        })
    }
}

pub async fn query(
    State(state): State<SharedState>,
    Json(req): Json<QueryRequest>,
) -> Result<Json<QueryResponse>, (StatusCode, String)> {
    if req.input.trim().is_empty() || req.input.len() > 4096 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Query must contain 1 to 4096 bytes".to_owned(),
        ));
    }
    let mut s = state.write().await;

    let current_theme =
        s.db.get_setting("theme")
            .unwrap_or(None)
            .unwrap_or_else(|| "dark".to_string());

    let settings_cmd = detect_settings_command(&req.input, &current_theme);

    if let Some(ref cmd) = settings_cmd {
        for (key, value) in &cmd.changes {
            let _ = s.db.set_setting(key, value);
        }
    }

    let (response_str, confidence, agent_name, agents_used, metadata) =
        if let Some(ref cmd) = settings_cmd {
            (
                cmd.description.clone(),
                1.0_f32,
                s.active_agents_display(),
                None,
                None,
            )
        } else {
            let (result, winning_agent) = s.query_best(&req.input);
            let resp: String = result.response.clone().into();
            let conf = result.confidence;
            let used: Vec<String> = s.active_agents.clone();
            if req.add_context {
                s.add_context_to_agents(req.input.clone(), result.response);
            }
            (resp, conf, winning_agent, Some(used), result.metadata)
        };

    if let Some(chat_id) = req.chat_id {
        let _ = s.db.add_message(chat_id, "user", &req.input, None);
        let _ =
            s.db.add_message(chat_id, "bot", &response_str, Some(confidence as f64));
    }

    let sc = settings_cmd.map(|cmd| {
        cmd.changes
            .into_iter()
            .map(|(key, value)| SettingsChange { key, value })
            .collect()
    });

    Ok(Json(QueryResponse {
        response: response_str,
        confidence,
        agent: agent_name,
        agents_used,
        settings_changed: sc,
        metadata,
    }))
}

#[derive(Deserialize)]
pub struct SwitchRequest {
    #[serde(default)]
    agent_type: Option<String>,
    #[serde(default)]
    agent_types: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct SwitchResponse {
    agent: String,
    active_agents: Vec<String>,
    examples: usize,
}

pub async fn switch_agent(
    State(state): State<SharedState>,
    Json(req): Json<SwitchRequest>,
) -> Result<Json<SwitchResponse>, (StatusCode, String)> {
    let mut s = state.write().await;

    let requested: Vec<String> = if let Some(types) = req.agent_types {
        types
    } else if let Some(t) = req.agent_type {
        vec![t]
    } else {
        return Err((
            StatusCode::BAD_REQUEST,
            "No agent type(s) specified".to_string(),
        ));
    };

    let valid_types = ["exact", "fuzzy", "tfidf", "context"];
    for t in &requested {
        if !valid_types.contains(&t.as_str()) {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Unknown agent type: {}", t),
            ));
        }
    }

    if requested.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "At least one agent must be selected".to_string(),
        ));
    }

    s.active_agents = requested;

    Ok(Json(SwitchResponse {
        agent: s.active_agents_display(),
        active_agents: s.active_agents.clone(),
        examples: s.knowledge_base.get_examples().len(),
    }))
}

#[derive(Serialize)]
pub struct KnowledgeEntry {
    index: usize,
    input: String,
    output: String,
    weight: f32,
}

#[derive(Deserialize)]
pub struct KnowledgeQuery {
    #[serde(default)]
    search: Option<String>,
    #[serde(default = "default_kb_offset")]
    offset: usize,
    #[serde(default = "default_kb_limit")]
    limit: usize,
}

fn default_kb_offset() -> usize {
    0
}
fn default_kb_limit() -> usize {
    20
}

#[derive(Serialize)]
pub struct KnowledgeListResponse {
    entries: Vec<KnowledgeEntry>,
    total: usize,
    filtered: usize,
    offset: usize,
    limit: usize,
}

pub async fn list_knowledge(
    State(state): State<SharedState>,
    Query(q): Query<KnowledgeQuery>,
) -> Json<KnowledgeListResponse> {
    let s = state.read().await;
    let all = s.knowledge_base.get_examples();
    let total = all.len();

    let filtered_entries: Vec<(usize, &crate::agent::TrainingExample)> =
        if let Some(ref search) = q.search {
            let lower = search.to_lowercase();
            all.iter()
                .enumerate()
                .filter(|(_, ex)| {
                    ex.input.to_lowercase().contains(&lower)
                        || String::from(ex.output.clone())
                            .to_lowercase()
                            .contains(&lower)
                })
                .collect()
        } else {
            all.iter().enumerate().collect()
        };
    let filtered = filtered_entries.len();

    let entries: Vec<KnowledgeEntry> = filtered_entries
        .into_iter()
        .skip(q.offset)
        .take(q.limit)
        .map(|(i, ex)| KnowledgeEntry {
            index: i,
            input: ex.input.clone(),
            output: String::from(ex.output.clone()),
            weight: ex.weight,
        })
        .collect();

    Json(KnowledgeListResponse {
        entries,
        total,
        filtered,
        offset: q.offset,
        limit: q.limit,
    })
}

#[derive(Deserialize)]
pub struct AddKnowledgeRequest {
    input: String,
    output: String,
    #[serde(default = "default_weight")]
    weight: f32,
}

fn default_weight() -> f32 {
    1.0
}

#[derive(Serialize)]
pub struct AddKnowledgeResponse {
    examples: usize,
}

pub async fn add_knowledge(
    State(state): State<SharedState>,
    Json(req): Json<AddKnowledgeRequest>,
) -> Result<Json<AddKnowledgeResponse>, (StatusCode, String)> {
    validate_web_example(&crate::TrainingExample {
        input: req.input.clone(),
        output: req.output.clone().into(),
        weight: req.weight,
        metadata: None,
    })?;
    let mut s = state.write().await;
    s.knowledge_base
        .add_example(req.input, ResponseFormat::Text(req.output), req.weight);

    let examples = s.knowledge_base.get_examples().to_vec();
    s.train_all(&examples);

    Ok(Json(AddKnowledgeResponse {
        examples: examples.len(),
    }))
}

#[derive(Serialize)]
pub struct DeleteKnowledgeResponse {
    removed: String,
    examples: usize,
}

pub async fn delete_knowledge(
    State(state): State<SharedState>,
    Path(index): Path<usize>,
) -> Result<Json<DeleteKnowledgeResponse>, (StatusCode, String)> {
    let mut s = state.write().await;

    let removed = s
        .knowledge_base
        .remove_example(index)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let examples = s.knowledge_base.get_examples().to_vec();
    s.train_all(&examples);
    let count = examples.len();

    Ok(Json(DeleteKnowledgeResponse {
        removed: removed.input,
        examples: count,
    }))
}

#[derive(Deserialize)]
pub struct SaveRequest {
    path: String,
}

#[derive(Serialize)]
pub struct SaveResponse {
    success: bool,
    path: String,
}

pub async fn save_knowledge(
    State(state): State<SharedState>,
    Json(req): Json<SaveRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, String)> {
    let s = state.read().await;
    s.knowledge_base
        .save(Some(safe_path(&req.path)?))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(SaveResponse {
        success: true,
        path: req.path,
    }))
}

#[derive(Deserialize)]
pub struct LoadRequest {
    path: String,
}

#[derive(Serialize)]
pub struct LoadResponse {
    success: bool,
    examples: usize,
}

pub async fn load_knowledge(
    State(state): State<SharedState>,
    Json(req): Json<LoadRequest>,
) -> Result<Json<LoadResponse>, (StatusCode, String)> {
    let kb = KnowledgeBase::load(safe_path(&req.path)?)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let mut s = state.write().await;
    let count = kb.get_examples().len();
    s.knowledge_base = kb;

    let examples = s.knowledge_base.get_examples().to_vec();
    s.train_all(&examples);

    Ok(Json(LoadResponse {
        success: true,
        examples: count,
    }))
}

#[derive(Serialize)]
pub struct PdfUploadResponse {
    success: bool,
    added: usize,
    total: usize,
}

pub async fn upload_pdf(
    State(state): State<SharedState>,
    mut multipart: Multipart,
) -> Result<Json<PdfUploadResponse>, (StatusCode, String)> {
    let field = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Multipart error: {}", e)))?
        .ok_or((StatusCode::BAD_REQUEST, "No file uploaded".to_string()))?;

    let filename = field
        .file_name()
        .and_then(|name| std::path::Path::new(name).file_name())
        .map(|name| name.to_string_lossy().chars().take(255).collect::<String>())
        .unwrap_or_else(|| "upload.pdf".to_owned());
    let data = field
        .bytes()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Read error: {}", e)))?;

    let pdf_kb = tokio::task::spawn_blocking(move || -> Result<KnowledgeBase, String> {
        let mut temporary = tempfile::Builder::new()
            .prefix("airust-upload-")
            .suffix(".pdf")
            .tempfile()
            .map_err(|error| format!("Temporary upload: {error}"))?;
        std::io::Write::write_all(&mut temporary, &data)
            .map_err(|error| format!("Temporary upload: {error}"))?;
        let loader = PdfLoader::new();
        let kb = loader
            .pdf_to_knowledge_base(temporary.path())
            .map_err(|error| format!("PDF processing: {error}"))?;
        let mut examples = kb.get_examples().to_vec();
        for example in &mut examples {
            if let Some(metadata) = example
                .metadata
                .as_mut()
                .and_then(serde_json::Value::as_object_mut)
            {
                metadata.insert(
                    "source_file".to_owned(),
                    serde_json::Value::String(filename.clone()),
                );
            }
        }
        let kb = KnowledgeBase::from_examples(examples);
        Ok(kb)
    })
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Task error: {}", e),
        )
    })?
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let added = pdf_kb.get_examples().len();

    let mut s = state.write().await;

    let batch = pdf_kb
        .get_examples()
        .iter()
        .cloned()
        .map(|example| (None, example))
        .collect::<Vec<_>>();
    s.db.add_training_batch(&batch)
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error))?;

    s.knowledge_base.merge(&pdf_kb);

    let examples = s.knowledge_base.get_examples().to_vec();
    s.train_all(&examples);
    let total = examples.len();

    Ok(Json(PdfUploadResponse {
        success: true,
        added,
        total,
    }))
}

pub async fn get_settings(
    State(state): State<SharedState>,
) -> Result<Json<HashMap<String, String>>, (StatusCode, String)> {
    let s = state.read().await;
    let settings =
        s.db.get_all_settings()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let mut map: HashMap<String, String> = settings.into_iter().collect();
    map.insert("language".to_owned(), "en".to_owned());
    Ok(Json(map))
}

pub async fn update_settings(
    State(state): State<SharedState>,
    Json(req): Json<HashMap<String, String>>,
) -> Result<Json<HashMap<String, String>>, (StatusCode, String)> {
    let s = state.read().await;
    if req.get("language").is_some_and(|language| language != "en") {
        return Err((
            StatusCode::BAD_REQUEST,
            "The interface language is English".to_owned(),
        ));
    }
    for (key, value) in &req {
        s.db.set_setting(key, value)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }
    let settings =
        s.db.get_all_settings()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let mut map: HashMap<String, String> = settings.into_iter().collect();
    map.insert("language".to_owned(), "en".to_owned());
    Ok(Json(map))
}

pub async fn get_guide() -> String {
    include_str!("../../STARTUP.md").to_string()
}

pub async fn get_translations(
    State(state): State<SharedState>,
    Path(_lang): Path<String>,
) -> Result<Json<HashMap<String, String>>, (StatusCode, String)> {
    let s = state.read().await;
    let translations =
        s.db.get_translations("en")
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let map: HashMap<String, String> = translations.into_iter().collect();
    Ok(Json(map))
}

#[derive(Deserialize)]
pub struct ChatsQuery {
    #[serde(default)]
    include_archived: bool,
}

pub async fn list_chats(
    State(state): State<SharedState>,
    Query(q): Query<ChatsQuery>,
) -> Result<Json<Vec<crate::web::db::ChatRow>>, (StatusCode, String)> {
    let s = state.read().await;
    let chats =
        s.db.list_chats(q.include_archived)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(chats))
}

#[derive(Deserialize)]
pub struct CreateChatRequest {
    #[serde(default)]
    title: String,
}

#[derive(Serialize)]
pub struct CreateChatResponse {
    id: i64,
    title: String,
}

pub async fn create_chat(
    State(state): State<SharedState>,
    Json(req): Json<CreateChatRequest>,
) -> Result<Json<CreateChatResponse>, (StatusCode, String)> {
    let s = state.read().await;
    let title = if req.title.is_empty() {
        "New Chat".to_string()
    } else {
        req.title
    };
    let id =
        s.db.create_chat(&title)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(CreateChatResponse { id, title }))
}

pub async fn get_chat_messages(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<crate::web::db::MessageRow>>, (StatusCode, String)> {
    let s = state.read().await;
    let messages =
        s.db.get_messages(id)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(messages))
}

#[derive(Serialize)]
pub struct DeleteChatResponse {
    success: bool,
}

pub async fn delete_chat(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
) -> Result<Json<DeleteChatResponse>, (StatusCode, String)> {
    let s = state.read().await;
    s.db.delete_chat(id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(DeleteChatResponse { success: true }))
}

#[derive(Serialize)]
pub struct ArchiveChatResponse {
    success: bool,
}

pub async fn archive_chat(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
) -> Result<Json<ArchiveChatResponse>, (StatusCode, String)> {
    let s = state.read().await;
    s.db.archive_chat(id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(ArchiveChatResponse { success: true }))
}

pub async fn list_categories(
    State(state): State<SharedState>,
) -> Result<Json<Vec<crate::web::db::CategoryRow>>, (StatusCode, String)> {
    let s = state.read().await;
    let cats =
        s.db.list_categories()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(cats))
}

#[derive(Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    #[serde(default = "default_color")]
    pub color: String,
    #[serde(default)]
    pub description: String,
}

fn default_color() -> String {
    "#7c6ef0".to_string()
}

pub async fn create_category(
    State(state): State<SharedState>,
    Json(req): Json<CreateCategoryRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state.read().await;
    let id =
        s.db.create_category(&req.name, &req.color, &req.description)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(serde_json::json!({"id": id})))
}

pub async fn delete_category(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state.read().await;
    s.db.delete_category(id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(serde_json::json!({"success": true})))
}

#[derive(Deserialize)]
pub struct TrainingDataQuery {
    #[serde(default)]
    pub category_id: Option<i64>,
}

pub async fn list_training_data(
    State(state): State<SharedState>,
    Query(q): Query<TrainingDataQuery>,
) -> Result<Json<Vec<crate::web::db::TrainingDataRow>>, (StatusCode, String)> {
    let s = state.read().await;
    let data =
        s.db.list_training_data(q.category_id)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(data))
}

#[derive(Deserialize)]
pub struct AddTrainingDataRequest {
    #[serde(default)]
    pub category_id: Option<i64>,
    pub input: String,
    #[serde(alias = "output_text")]
    pub output: String,
    #[serde(default = "default_format", alias = "output_format")]
    pub format: String,
    #[serde(default = "default_weight_f64")]
    pub weight: f64,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

fn default_format() -> String {
    "text".to_string()
}
fn default_weight_f64() -> f64 {
    1.0
}

pub async fn add_training_data(
    State(state): State<SharedState>,
    Json(req): Json<AddTrainingDataRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut s = state.write().await;
    let example = crate::TrainingExample {
        input: req.input,
        output: crate::web::db::decode_response(req.output, &req.format),
        weight: req.weight as f32,
        metadata: req.metadata,
    };
    validate_web_example(&example)?;
    let id =
        s.db.add_training_example(req.category_id, &example)
            .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error))?;
    s.knowledge_base.push_example(example);
    let examples = s.knowledge_base.get_examples().to_vec();
    s.train_all(&examples);
    Ok(Json(
        serde_json::json!({"id": id, "examples": examples.len()}),
    ))
}

pub async fn delete_training_data(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut s = state.write().await;
    s.db.delete_training_data(id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    rebuild_kb_from_db(&mut s).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let count = s.knowledge_base.get_examples().len();
    Ok(Json(
        serde_json::json!({"success": true, "examples": count}),
    ))
}

#[derive(Deserialize)]
pub struct TrainingImportItem {
    pub input: String,
    #[serde(alias = "output_text")]
    pub output: String,
    #[serde(default = "default_format", alias = "output_format")]
    pub format: String,
    #[serde(default = "default_weight_f64")]
    pub weight: f64,
    #[serde(default)]
    pub category_id: Option<i64>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

fn validate_web_example(example: &crate::TrainingExample) -> Result<(), (StatusCode, String)> {
    example
        .validate()
        .map_err(|error| (StatusCode::BAD_REQUEST, error.to_string()))?;
    if example.input.trim().is_empty()
        || example.input.len() > 1024 * 1024
        || example.output.to_string().len() > 1024 * 1024
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "Training input and output are limited to 1 MiB; input must be nonempty".to_owned(),
        ));
    }
    Ok(())
}

type TrainingBatch = Vec<(Option<i64>, crate::TrainingExample)>;
type WebError = (StatusCode, String);

fn validated_import(items: Vec<TrainingImportItem>) -> Result<TrainingBatch, WebError> {
    if items.len() > 1000 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Training imports are limited to 1000 items".to_owned(),
        ));
    }
    items
        .into_iter()
        .map(|item| {
            let example = crate::TrainingExample {
                input: item.input,
                output: crate::web::db::decode_response(item.output, &item.format),
                weight: item.weight as f32,
                metadata: item.metadata,
            };
            validate_web_example(&example)?;
            Ok((item.category_id, example))
        })
        .collect()
}

pub async fn import_training(
    State(state): State<SharedState>,
    Json(items): Json<Vec<TrainingImportItem>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let batch = validated_import(items)?;
    let imported = batch.len();
    let mut s = state.write().await;
    s.db.add_training_batch(&batch)
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error))?;
    for (_, example) in batch {
        s.knowledge_base.push_example(example);
    }
    let examples = s.knowledge_base.get_examples().to_vec();
    s.train_all(&examples);
    Ok(Json(
        serde_json::json!({"imported": imported, "examples": examples.len()}),
    ))
}

pub async fn export_training(
    State(state): State<SharedState>,
) -> Result<Json<Vec<crate::web::db::TrainingDataRow>>, (StatusCode, String)> {
    let s = state.read().await;
    let data =
        s.db.get_all_training_data()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(data))
}

pub async fn upload_json(
    State(state): State<SharedState>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let field = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Multipart error: {}", e)))?
        .ok_or((StatusCode::BAD_REQUEST, "No file uploaded".to_string()))?;

    let data = field
        .bytes()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Read error: {}", e)))?;

    let items: Vec<TrainingImportItem> = serde_json::from_slice(&data)
        .map_err(crate::input::json_error)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("JSON parse error: {}", e)))?;

    let batch = validated_import(items)?;
    let imported = batch.len();
    let mut s = state.write().await;
    s.db.add_training_batch(&batch)
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error))?;
    for (_, example) in batch {
        s.knowledge_base.push_example(example);
    }
    let examples = s.knowledge_base.get_examples().to_vec();
    s.train_all(&examples);
    Ok(Json(
        serde_json::json!({"imported": imported, "examples": examples.len()}),
    ))
}

fn rebuild_kb_from_db(s: &mut crate::web::state::AppState) -> Result<(), String> {
    let mut examples = KnowledgeBase::from_embedded().get_examples().to_vec();
    examples.extend(
        s.db.get_all_training_data()?
            .into_iter()
            .map(crate::web::db::TrainingDataRow::into_example),
    );
    s.knowledge_base = KnowledgeBase::from_examples(examples);
    let examples = s.knowledge_base.get_examples().to_vec();
    s.train_all(&examples);
    Ok(())
}

pub fn rebuild_kb_from_db_public(s: &mut crate::web::state::AppState) -> Result<(), String> {
    rebuild_kb_from_db(s)
}

fn safe_path(input: &str) -> Result<std::path::PathBuf, (StatusCode, String)> {
    let root = std::env::current_dir()
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    confined_path(&root, input)
}

pub(crate) fn confined_path(
    root: &std::path::Path,
    input: &str,
) -> Result<std::path::PathBuf, (StatusCode, String)> {
    use std::path::Component;
    let reject = || {
        (StatusCode::FORBIDDEN, "Path must stay inside the workspace and cannot contain hidden or symbolic-link components".to_owned())
    };
    if input.is_empty() || input.contains('\\') || input.contains('\0') {
        return Err(reject());
    }
    let root = root.canonicalize().map_err(|_| reject())?;
    let mut resolved = root.clone();
    for component in std::path::Path::new(input).components() {
        match component {
            Component::CurDir => {}
            Component::Normal(name) if !name.to_string_lossy().starts_with('.') => {
                resolved.push(name)
            }
            _ => return Err(reject()),
        }
        match std::fs::symlink_metadata(&resolved) {
            Ok(metadata) if metadata.file_type().is_symlink() => return Err(reject()),
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(reject()),
        }
    }
    if !resolved.starts_with(&root) {
        return Err(reject());
    }
    Ok(resolved)
}

#[derive(Deserialize)]
pub struct FileListQuery {
    #[serde(default = "default_file_path")]
    pub path: String,
}

fn default_file_path() -> String {
    ".".to_string()
}

#[derive(Serialize)]
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: String,
}

pub async fn list_files(
    Query(q): Query<FileListQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let path = safe_path(&q.path)?;
    if !path.exists() {
        return Err((StatusCode::NOT_FOUND, "Path not found".to_string()));
    }
    if !path.is_dir() {
        return Err((StatusCode::BAD_REQUEST, "Not a directory".to_string()));
    }

    let mut entries = Vec::new();
    let read_dir = std::fs::read_dir(path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Read dir error: {}", e),
        )
    })?;

    for entry in read_dir {
        let entry = entry.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Entry error: {}", e),
            )
        })?;
        let filename = entry.file_name();
        if filename.to_string_lossy().starts_with('.')
            || entry
                .file_type()
                .map(|kind| kind.is_symlink())
                .unwrap_or(true)
        {
            continue;
        }
        let metadata = entry.metadata().map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Metadata error: {}", e),
            )
        })?;
        let modified = metadata
            .modified()
            .map(|t| {
                let datetime: chrono::DateTime<chrono::Utc> = t.into();
                datetime.format("%Y-%m-%d %H:%M").to_string()
            })
            .unwrap_or_default();
        entries.push(FileEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            is_dir: metadata.is_dir(),
            size: metadata.len(),
            modified,
        });
    }

    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(Json(
        serde_json::json!({"path": q.path, "entries": entries}),
    ))
}

#[derive(Deserialize)]
pub struct FileReadQuery {
    pub path: String,
}

pub async fn read_file(
    Query(q): Query<FileReadQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let path = safe_path(&q.path)?;
    if !path.exists() {
        return Err((StatusCode::NOT_FOUND, "File not found".to_string()));
    }
    if !path.is_file() {
        return Err((StatusCode::BAD_REQUEST, "Not a file".to_string()));
    }
    if std::fs::metadata(&path)
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?
        .len()
        > 1024 * 1024
    {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            "Text files are limited to 1 MiB".to_owned(),
        ));
    }
    let content = std::fs::read_to_string(&path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Read error: {}", e),
        )
    })?;
    Ok(Json(
        serde_json::json!({"path": q.path, "content": content}),
    ))
}

#[derive(Deserialize)]
pub struct FileWriteRequest {
    pub path: String,
    pub content: String,
}

pub async fn write_file(
    Json(req): Json<FileWriteRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if req.content.len() > 1024 * 1024 {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            "Text files are limited to 1 MiB".to_owned(),
        ));
    }
    let path = safe_path(&req.path)?;
    if path.is_dir() {
        return Err((StatusCode::BAD_REQUEST, "Expected a file path".to_owned()));
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    }
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&path)
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    std::io::Write::write_all(&mut file, req.content.as_bytes())
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    Ok(Json(serde_json::json!({"success": true, "path": req.path})))
}

#[derive(Deserialize)]
pub struct MkdirRequest {
    pub path: String,
}

pub async fn mkdir(
    Json(req): Json<MkdirRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let path = safe_path(&req.path)?;
    std::fs::create_dir_all(&path)
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    Ok(Json(serde_json::json!({"success": true, "path": req.path})))
}

#[derive(Deserialize)]
pub struct FileDeleteQuery {
    pub path: String,
}

pub async fn delete_file(
    Query(q): Query<FileDeleteQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let path = safe_path(&q.path)?;
    if !path.exists() {
        return Err((StatusCode::NOT_FOUND, "Path not found".to_string()));
    }

    let cwd = std::env::current_dir()
        .unwrap_or_default()
        .canonicalize()
        .unwrap_or_default();
    if path == cwd {
        return Err((
            StatusCode::FORBIDDEN,
            "Cannot delete working directory".to_string(),
        ));
    }
    if path.is_dir() {
        std::fs::remove_dir_all(&path).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Remove dir error: {}", e),
            )
        })?;
    } else {
        std::fs::remove_file(&path).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Remove file error: {}", e),
            )
        })?;
    }
    Ok(Json(serde_json::json!({"success": true})))
}

#[derive(Deserialize)]
pub struct RenameRequest {
    pub from: String,
    pub to: String,
}

pub async fn rename_file(
    Json(req): Json<RenameRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let from = safe_path(&req.from)?;
    let destination = safe_path(&req.to)?;
    std::fs::rename(&from, &destination).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Rename error: {}", e),
        )
    })?;
    Ok(Json(serde_json::json!({"success": true})))
}

#[derive(Deserialize)]
pub struct CopyRequest {
    pub from: String,
    pub to: String,
}

pub async fn copy_file(
    Json(req): Json<CopyRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let from = safe_path(&req.from)?;
    if from.is_dir() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Cannot copy directories".to_string(),
        ));
    }
    let to = safe_path(&req.to)?;
    std::fs::copy(&from, &to).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Copy error: {}", e),
        )
    })?;
    Ok(Json(serde_json::json!({"success": true})))
}

pub async fn list_vector_collections(
    State(state): State<SharedState>,
) -> Result<Json<Vec<crate::web::db::VectorCollectionRow>>, (StatusCode, String)> {
    let s = state.read().await;
    let cols =
        s.db.list_vector_collections()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(cols))
}

#[derive(Deserialize)]
pub struct CreateCollectionRequest {
    pub name: String,
    #[serde(default)]
    pub description: String,
}

pub async fn create_vector_collection(
    State(state): State<SharedState>,
    Json(req): Json<CreateCollectionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state.read().await;
    let id =
        s.db.create_vector_collection(&req.name, &req.description)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(serde_json::json!({"id": id})))
}

pub async fn delete_vector_collection(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state.read().await;
    s.db.delete_vector_collection(id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(serde_json::json!({"success": true})))
}

#[derive(Deserialize)]
pub struct VectorEntriesQuery {
    pub collection_id: i64,
}

pub async fn list_vector_entries(
    State(state): State<SharedState>,
    Query(q): Query<VectorEntriesQuery>,
) -> Result<Json<Vec<crate::web::db::VectorEntryRow>>, (StatusCode, String)> {
    let s = state.read().await;
    let entries =
        s.db.list_vector_entries(q.collection_id)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(entries))
}

#[derive(Deserialize)]
pub struct AddVectorEntryRequest {
    pub collection_id: i64,
    pub content: String,
    #[serde(default = "default_metadata")]
    pub metadata_json: String,
}

fn default_metadata() -> String {
    "{}".to_string()
}

pub async fn add_vector_entry(
    State(state): State<SharedState>,
    Json(req): Json<AddVectorEntryRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state.read().await;

    let existing =
        s.db.list_vector_entries(req.collection_id)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let embedding = crate::web::vectordb::compute_embedding_for_storage(&req.content, &existing);
    let id =
        s.db.add_vector_entry(
            req.collection_id,
            &req.content,
            &req.metadata_json,
            &embedding,
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(serde_json::json!({"id": id})))
}

pub async fn delete_vector_entry(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state.read().await;
    s.db.delete_vector_entry(id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(serde_json::json!({"success": true})))
}

#[derive(Deserialize)]
pub struct VectorSearchRequest {
    pub collection_id: i64,
    pub query: String,
    #[serde(default = "default_vector_top_k")]
    pub top_k: usize,
}

fn default_vector_top_k() -> usize {
    5
}

pub async fn vector_search(
    State(state): State<SharedState>,
    Json(req): Json<VectorSearchRequest>,
) -> Result<Json<Vec<crate::web::vectordb::SearchResult>>, (StatusCode, String)> {
    let s = state.read().await;
    let entries =
        s.db.list_vector_entries(req.collection_id)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let results = crate::web::vectordb::similarity_search(&req.query, &entries, req.top_k);
    Ok(Json(results))
}

#[derive(Deserialize)]
pub struct DbPathQuery {
    pub path: String,
}

pub async fn db_tables(
    Query(q): Query<DbPathQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let db_path = safe_path(&q.path)?;
    let conn =
        rusqlite::Connection::open_with_flags(&db_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Open error: {}", e),
                )
            })?;

    let mut stmt = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name"
    ).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {}", e)))?;

    let tables: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Query error: {}", e),
            )
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(Json(serde_json::json!({"tables": tables})))
}

#[derive(Deserialize)]
pub struct DbQueryRequest {
    pub path: String,
    pub table: String,
    #[serde(default = "default_db_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_db_limit() -> i64 {
    100
}

pub async fn db_query(
    Query(q): Query<DbQueryRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let db_path = safe_path(&q.path)?;
    let conn =
        rusqlite::Connection::open_with_flags(&db_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Open error: {}", e),
                )
            })?;

    if !(1..=1000).contains(&q.limit) || q.offset < 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Limit must be 1..=1000 and offset must be nonnegative".to_owned(),
        ));
    }

    if !q.table.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err((StatusCode::BAD_REQUEST, "Invalid table name".to_string()));
    }

    let mut pragma = conn
        .prepare(&format!("PRAGMA table_info(\"{}\")", q.table))
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Pragma error: {}", e),
            )
        })?;
    let columns: Vec<(String, String)> = pragma
        .query_map([], |row| {
            Ok((row.get::<_, String>(1)?, row.get::<_, String>(2)?))
        })
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Pragma error: {}", e),
            )
        })?
        .filter_map(|r| r.ok())
        .collect();

    if columns.is_empty() {
        return Err((StatusCode::NOT_FOUND, "Table not found".to_string()));
    }

    let total: i64 = conn
        .query_row(
            &format!("SELECT COUNT(*) FROM \"{}\"", q.table),
            [],
            |row| row.get(0),
        )
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Count error: {}", e),
            )
        })?;

    let sql = format!(
        "SELECT * FROM \"{}\" LIMIT {} OFFSET {}",
        q.table, q.limit, q.offset
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Query error: {}", e),
        )
    })?;

    let col_count = stmt.column_count();
    let mut rows: Vec<Vec<serde_json::Value>> = Vec::new();

    let mut result_rows = stmt.query([]).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Query error: {}", e),
        )
    })?;

    while let Some(row) = result_rows.next().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Row error: {}", e),
        )
    })? {
        let mut vals = Vec::new();
        for i in 0..col_count {
            let val: serde_json::Value = match row.get_ref(i) {
                Ok(rusqlite::types::ValueRef::Null) => serde_json::Value::Null,
                Ok(rusqlite::types::ValueRef::Integer(n)) => serde_json::json!(n),
                Ok(rusqlite::types::ValueRef::Real(f)) => serde_json::json!(f),
                Ok(rusqlite::types::ValueRef::Text(s)) => {
                    serde_json::Value::String(String::from_utf8_lossy(s).to_string())
                }
                Ok(rusqlite::types::ValueRef::Blob(b)) => {
                    serde_json::Value::String(format!("[BLOB {} bytes]", b.len()))
                }
                Err(_) => serde_json::Value::Null,
            };
            vals.push(val);
        }
        rows.push(vals);
    }

    let col_info: Vec<serde_json::Value> = columns
        .iter()
        .map(|(name, typ)| serde_json::json!({"name": name, "type": typ}))
        .collect();

    Ok(Json(serde_json::json!({
        "columns": col_info,
        "rows": rows,
        "total": total,
        "limit": q.limit,
        "offset": q.offset
    })))
}

#[derive(Deserialize)]
pub struct DbExecuteRequest {
    pub path: String,
    pub sql: String,
}

pub async fn db_execute(
    Json(req): Json<DbExecuteRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let sql_upper = req.sql.trim().to_uppercase();

    if !sql_upper.starts_with("INSERT")
        && !sql_upper.starts_with("UPDATE")
        && !sql_upper.starts_with("DELETE")
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "Only INSERT, UPDATE, DELETE allowed".to_string(),
        ));
    }

    let db_path = safe_path(&req.path)?;
    let conn = rusqlite::Connection::open(&db_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Open error: {}", e),
        )
    })?;

    let affected = conn.execute(&req.sql, []).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Execute error: {}", e),
        )
    })?;

    Ok(Json(
        serde_json::json!({"success": true, "affected": affected}),
    ))
}

#[derive(Serialize)]
pub struct TatoebaImportResponse {
    success: bool,
    imported: usize,
    skipped: usize,
    total: usize,
}

pub async fn import_tatoeba(
    State(state): State<SharedState>,
    mut multipart: Multipart,
) -> Result<Json<TatoebaImportResponse>, (StatusCode, String)> {
    let field = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Multipart error: {}", e)))?
        .ok_or((StatusCode::BAD_REQUEST, "No file uploaded".to_string()))?;

    let data = field
        .bytes()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Read error: {}", e)))?;

    let text = String::from_utf8(data.to_vec())
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid UTF-8: {}", e)))?;

    let mut pairs: Vec<(String, String)> = Vec::new();
    let mut skipped = 0;

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 {
            let src = parts[0].trim();
            let tgt = parts[1].trim();
            if !src.is_empty() && !tgt.is_empty() && src.len() >= 2 && tgt.len() >= 2 {
                pairs.push((src.to_string(), tgt.to_string()));
            } else {
                skipped += 1;
            }
        } else {
            skipped += 1;
        }
    }

    let imported = pairs.len();

    let mut s = state.write().await;
    for (input, output) in &pairs {
        s.knowledge_base
            .add_example(input.clone(), ResponseFormat::Text(output.clone()), 1.0);
        let _ = s.db.add_training_data(None, input, output, "text", 1.0);
    }

    let examples = s.knowledge_base.get_examples().to_vec();
    s.train_all(&examples);
    let total = examples.len();

    Ok(Json(TatoebaImportResponse {
        success: true,
        imported,
        skipped,
        total,
    }))
}

#[cfg(test)]
mod security_tests {
    use super::*;

    #[test]
    fn file_paths_reject_traversal_hidden_names_and_existing_symlink_ancestors() {
        let workspace = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        assert!(confined_path(workspace.path(), "notes/new.txt").is_ok());
        for path in [
            "../outside",
            "/tmp/outside",
            "notes/../../outside",
            ".git/config",
            "notes/.env",
            "notes\\secret",
        ] {
            assert!(confined_path(workspace.path(), path).is_err(), "{path}");
        }
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(outside.path(), workspace.path().join("link")).unwrap();
            assert!(confined_path(workspace.path(), "link/new.txt").is_err());
            assert!(confined_path(workspace.path(), "link/new-directory/file.txt").is_err());
            assert!(confined_path(workspace.path(), "link").is_err());
        }
    }

    #[test]
    fn ordinary_queries_cannot_change_interface_settings() {
        assert!(detect_settings_command("How do I make a light Rust package?", "dark").is_none());
        let command = detect_settings_command("/settings theme light", "dark").unwrap();
        assert_eq!(
            command.changes,
            vec![("theme".to_owned(), "light".to_owned())]
        );
    }
    #[test]
    fn training_requests_validate_weights_and_bounds_before_mutation() {
        let make = |weight| TrainingImportItem {
            input: "valid".to_owned(),
            output: "answer".to_owned(),
            format: "text".to_owned(),
            weight,
            category_id: None,
            metadata: None,
        };
        assert!(validated_import(vec![make(1.0), make(f64::MAX)]).is_err());
        assert!(validated_import(vec![make(-1.0)]).is_err());
        assert!(validated_import((0..1001).map(|_| make(1.0)).collect()).is_err());
        assert!(validated_import(vec![make(0.0)]).is_ok());
    }
}
