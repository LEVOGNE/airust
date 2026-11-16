use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bot {
    pub id: i64,
    pub name: String,
    pub bot_type: String,
    pub config: BotConfig,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    #[serde(default)]
    pub url: String,
    #[serde(default = "default_mode")]
    pub mode: String,
    #[serde(default = "default_max_depth")]
    pub max_depth: u32,
    #[serde(default = "default_rate_limit")]
    pub rate_limit_ms: u64,
    #[serde(default = "default_strategy")]
    pub strategy: String,
    #[serde(default = "default_min_length")]
    pub min_length: usize,
}

fn default_mode() -> String {
    "single_page".to_string()
}
fn default_max_depth() -> u32 {
    2
}
fn default_rate_limit() -> u64 {
    1000
}
fn default_strategy() -> String {
    "heading_content".to_string()
}
fn default_min_length() -> usize {
    20
}

impl Default for BotConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            mode: default_mode(),
            max_depth: default_max_depth(),
            rate_limit_ms: default_rate_limit(),
            strategy: default_strategy(),
            min_length: default_min_length(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotRun {
    pub id: i64,
    pub bot_id: i64,
    pub status: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub items_found: i64,
    pub items_added: i64,
    pub error_msg: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawData {
    pub id: i64,
    pub bot_run_id: i64,
    pub url: Option<String>,
    pub title: Option<String>,
    pub content: String,
    pub content_hash: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovedData {
    pub id: i64,
    pub raw_data_id: Option<i64>,
    pub input: String,
    pub output: String,
    pub weight: f64,
    pub source_url: Option<String>,
    pub added_to_kb: bool,
    pub created_at: String,
}
