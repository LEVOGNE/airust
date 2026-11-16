use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::Extension,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, Mutex};

const MAX_HISTORY: usize = 500;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

pub enum ServerCommand {
    Stop,
    Start,
    Restart,
    Reset,
    Exit,
}

#[derive(Clone)]
pub struct ConsoleState {
    pub history: Arc<Mutex<VecDeque<LogEntry>>>,
    pub tx: broadcast::Sender<LogEntry>,
    pub cmd_tx: mpsc::Sender<ServerCommand>,
    pub server_running: Arc<AtomicBool>,
}

impl ConsoleState {
    pub fn new(cmd_tx: mpsc::Sender<ServerCommand>, server_running: Arc<AtomicBool>) -> Self {
        let (tx, _) = broadcast::channel(256);
        Self {
            history: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_HISTORY))),
            tx,
            cmd_tx,
            server_running,
        }
    }

    pub async fn push_log(&self, level: &str, message: &str) {
        let entry = LogEntry {
            timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
            level: level.to_string(),
            message: message.to_string(),
        };
        {
            let mut hist = self.history.lock().await;
            if hist.len() >= MAX_HISTORY {
                hist.pop_front();
            }
            hist.push_back(entry.clone());
        }
        let _ = self.tx.send(entry);
    }
}

#[derive(Deserialize)]
struct ConsoleCommand {
    #[serde(rename = "type")]
    cmd_type: String,
    command: Option<String>,
}

pub async fn ws_console(
    ws: WebSocketUpgrade,
    Extension(console): Extension<ConsoleState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, console))
}

async fn handle_socket(mut socket: WebSocket, console: ConsoleState) {
    // Send full history on connect
    {
        let hist = console.history.lock().await;
        let entries: Vec<&LogEntry> = hist.iter().collect();
        if let Ok(json) = serde_json::to_string(&serde_json::json!({
            "type": "history",
            "entries": entries,
        })) {
            let _ = socket.send(Message::Text(json)).await;
        }
    }

    let mut rx = console.tx.subscribe();

    loop {
        tokio::select! {
            Ok(entry) = rx.recv() => {
                if let Ok(json) = serde_json::to_string(&serde_json::json!({
                    "type": "log",
                    "entry": entry,
                })) {
                    if socket.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(cmd) = serde_json::from_str::<ConsoleCommand>(&text) {
                            if cmd.cmd_type == "cmd" {
                                if let Some(command) = cmd.command {
                                    handle_console_command(&command, &console).await;
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }
}

async fn handle_console_command(command: &str, console: &ConsoleState) {
    let trimmed = command.trim();
    console.push_log("cmd", &format!("$ {}", trimmed)).await;

    let lower = trimmed.to_lowercase();
    match lower.as_str() {
        "clear" => {
            let mut hist = console.history.lock().await;
            hist.clear();
            let _ = console.tx.send(LogEntry {
                timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
                level: "info".to_string(),
                message: "__clear__".to_string(),
            });
            return;
        }
        "help" | "?" => {
            let help_lines = [
                "Built-in commands:",
                "  help       Show this help",
                "  clear      Clear console output",
                "  status     Show server status",
                "  restart    Restart the web server",
                "  reset      Delete DB + restart (fresh start)",
            ];
            for line in help_lines {
                console.push_log("info", line).await;
            }
            return;
        }
        "status" => {
            let running = console.server_running.load(Ordering::SeqCst);
            console
                .push_log("info", &format!("AIRust v{}", env!("CARGO_PKG_VERSION")))
                .await;
            console
                .push_log(
                    if running { "info" } else { "warn" },
                    &format!("Server: {}", if running { "running" } else { "stopped" }),
                )
                .await;
            if let Ok(cwd) = std::env::current_dir() {
                console
                    .push_log("info", &format!("CWD: {}", cwd.display()))
                    .await;
            }
            return;
        }
        "restart" | "reboot" => {
            console.push_log("warn", "Restarting server...").await;
            let _ = console.cmd_tx.send(ServerCommand::Restart).await;
            return;
        }
        "reset" => {
            console.push_log("warn", "Resetting...").await;
            let _ = console.cmd_tx.send(ServerCommand::Reset).await;
            return;
        }
        _ => {}
    }

    // Unknown command — suggest built-in if close match
    let builtins = ["help", "clear", "status", "restart", "reset"];
    for b in &builtins {
        if strsim::jaro_winkler(&lower, b) > 0.85 && lower != *b {
            console
                .push_log("info", &format!("Did you mean '{}'?", b))
                .await;
            return;
        }
    }

    console
        .push_log(
            "warn",
            &format!(
                "Unknown command: {}. Type 'help' for available commands.",
                trimmed
            ),
        )
        .await;
}
