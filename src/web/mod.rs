pub mod bots;
pub mod console;
pub mod db;
pub mod routes;
pub mod security;
pub mod state;
pub mod vectordb;

use crate::agent::TrainableAgent;
use crate::context_agent::ContextAgent;
use crate::knowledge::KnowledgeBase;
use crate::match_agent::MatchAgent;
use crate::tfidf_agent::TfidfAgent;
use bots::scheduler::BotScheduler;
use console::ConsoleState;
use db::Database;
use state::{AgentWrapper, AppState, SharedState};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

fn build_main_router(state: SharedState) -> axum::Router {
    axum::Router::new()
        .route("/", axum::routing::get(routes::index))
        .route("/api/status", axum::routing::get(routes::status))
        .route("/api/query", axum::routing::post(routes::query))
        .route(
            "/api/agent/switch",
            axum::routing::post(routes::switch_agent),
        )
        .route("/api/knowledge", axum::routing::get(routes::list_knowledge))
        .route(
            "/api/knowledge/add",
            axum::routing::post(routes::add_knowledge),
        )
        .route(
            "/api/knowledge/:index",
            axum::routing::delete(routes::delete_knowledge),
        )
        .route(
            "/api/knowledge/save",
            axum::routing::post(routes::save_knowledge),
        )
        .route(
            "/api/knowledge/load",
            axum::routing::post(routes::load_knowledge),
        )
        .route("/api/pdf/upload", axum::routing::post(routes::upload_pdf))
        .route(
            "/api/settings",
            axum::routing::get(routes::get_settings).post(routes::update_settings),
        )
        .route("/api/guide", axum::routing::get(routes::get_guide))
        .route(
            "/api/translations/:lang",
            axum::routing::get(routes::get_translations),
        )
        .route(
            "/api/chats",
            axum::routing::get(routes::list_chats).post(routes::create_chat),
        )
        .route(
            "/api/chats/:id/messages",
            axum::routing::get(routes::get_chat_messages),
        )
        .route("/api/chats/:id", axum::routing::delete(routes::delete_chat))
        .route(
            "/api/chats/:id/archive",
            axum::routing::post(routes::archive_chat),
        )
        .route(
            "/api/bots",
            axum::routing::get(bots::routes::list_bots).post(bots::routes::create_bot),
        )
        .route(
            "/api/bots/:id",
            axum::routing::get(bots::routes::get_bot)
                .put(bots::routes::update_bot)
                .delete(bots::routes::delete_bot),
        )
        .route(
            "/api/bots/:id/start",
            axum::routing::post(bots::routes::start_bot),
        )
        .route(
            "/api/bots/:id/stop",
            axum::routing::post(bots::routes::stop_bot),
        )
        .route(
            "/api/bots/:id/runs",
            axum::routing::get(bots::routes::get_bot_runs),
        )
        .route(
            "/api/bots/:bot_id/runs/:run_id/data",
            axum::routing::get(bots::routes::get_run_data),
        )
        .route(
            "/api/data/pending",
            axum::routing::get(bots::routes::get_pending_data),
        )
        .route(
            "/api/data/:id/approve",
            axum::routing::post(bots::routes::approve_data),
        )
        .route(
            "/api/data/:id/reject",
            axum::routing::post(bots::routes::reject_data),
        )
        .route(
            "/api/data/approve-all",
            axum::routing::post(bots::routes::approve_all_data),
        )
        .route(
            "/api/data/approved",
            axum::routing::get(bots::routes::get_approved_data),
        )
        .route(
            "/api/data/add-to-kb",
            axum::routing::post(bots::routes::add_to_kb),
        )
        .route(
            "/api/vectors/stats",
            axum::routing::get(bots::routes::vector_stats),
        )
        .route(
            "/api/vectors/rebuild",
            axum::routing::post(bots::routes::rebuild_vectors),
        )
        .route(
            "/api/training/categories",
            axum::routing::get(routes::list_categories).post(routes::create_category),
        )
        .route(
            "/api/training/categories/:id",
            axum::routing::delete(routes::delete_category),
        )
        .route(
            "/api/training/data",
            axum::routing::get(routes::list_training_data).post(routes::add_training_data),
        )
        .route(
            "/api/training/data/:id",
            axum::routing::delete(routes::delete_training_data),
        )
        .route(
            "/api/training/import",
            axum::routing::post(routes::import_training),
        )
        .route(
            "/api/training/export",
            axum::routing::get(routes::export_training),
        )
        .route("/api/upload/json", axum::routing::post(routes::upload_json))
        .route(
            "/api/files",
            axum::routing::get(routes::list_files).delete(routes::delete_file),
        )
        .route("/api/files/read", axum::routing::get(routes::read_file))
        .route("/api/files/write", axum::routing::post(routes::write_file))
        .route("/api/files/mkdir", axum::routing::post(routes::mkdir))
        .route(
            "/api/files/rename",
            axum::routing::post(routes::rename_file),
        )
        .route("/api/files/copy", axum::routing::post(routes::copy_file))
        .route(
            "/api/files/db/tables",
            axum::routing::get(routes::db_tables),
        )
        .route("/api/files/db/query", axum::routing::get(routes::db_query))
        .route(
            "/api/files/db/execute",
            axum::routing::post(routes::db_execute),
        )
        .route(
            "/api/vectors/collections",
            axum::routing::get(routes::list_vector_collections)
                .post(routes::create_vector_collection),
        )
        .route(
            "/api/vectors/collections/:id",
            axum::routing::delete(routes::delete_vector_collection),
        )
        .route(
            "/api/vectors/entries",
            axum::routing::get(routes::list_vector_entries).post(routes::add_vector_entry),
        )
        .route(
            "/api/vectors/entries/:id",
            axum::routing::delete(routes::delete_vector_entry),
        )
        .route(
            "/api/vectors/search",
            axum::routing::post(routes::vector_search),
        )
        .route(
            "/api/import/tatoeba",
            axum::routing::post(routes::import_tatoeba),
        )
        .with_state(state)
}

struct ServerReset;

impl ServerReset {
    fn agents(kb: &KnowledgeBase) -> Vec<AgentWrapper> {
        let mut tfidf = TfidfAgent::new();
        tfidf.train(kb.get_examples());
        let mut exact = MatchAgent::new_exact();
        exact.train(kb.get_examples());
        let mut fuzzy = MatchAgent::new_fuzzy();
        fuzzy.train(kb.get_examples());
        let mut context = ContextAgent::new(TfidfAgent::new(), 5);
        context.train(kb.get_examples());
        vec![
            AgentWrapper::Tfidf(tfidf),
            AgentWrapper::Exact(exact),
            AgentWrapper::Fuzzy(fuzzy),
            AgentWrapper::Context(context),
        ]
    }

    fn active_agents() -> Vec<String> {
        ["tfidf", "exact", "fuzzy", "context"]
            .into_iter()
            .map(str::to_owned)
            .collect()
    }

    async fn run(console: &ConsoleState, state: &SharedState) -> bool {
        console.push_log("warn", "Deleting airust.db...").await;
        let _ = std::fs::remove_file("airust.db");
        console.push_log("info", "Reinitializing...").await;
        let database = match Database::open("airust.db") {
            Ok(database) => Arc::new(database),
            Err(error) => {
                console
                    .push_log(
                        "error",
                        &format!("Reset failed; airust.db could not be opened: {error}"),
                    )
                    .await;
                return false;
            }
        };
        let knowledge_base = KnowledgeBase::from_embedded();
        let agents = Self::agents(&knowledge_base);
        let scheduler = BotScheduler::new(database.clone());
        let mut s = state.write().await;
        s.agents = agents;
        s.active_agents = Self::active_agents();
        s.knowledge_base = knowledge_base;
        s.db = database;
        s.scheduler = scheduler;
        true
    }

    async fn announce(console: &ConsoleState, message: &str) {
        console.push_log("info", message).await;
        let _ = console.tx.send(console::LogEntry {
            timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
            level: "info".to_string(),
            message: "__reset__".to_string(),
        });
    }
}

pub async fn start_server(port: u16, show_landing: bool) {
    let Some(console_port) = port.checked_add(1).filter(|_| port > 0) else {
        eprintln!("AIRust requires a port between 1 and 65534; the console uses port+1.");
        return;
    };
    let bind_ip: std::net::IpAddr = match std::env::var("AIRUST_BIND_ADDR")
        .unwrap_or_else(|_| "127.0.0.1".to_owned())
        .parse()
    {
        Ok(address) => address,
        Err(error) => {
            eprintln!("Invalid AIRUST_BIND_ADDR: {}", error);
            return;
        }
    };
    let security = match security::SecurityConfig::from_environment(bind_ip, port) {
        Ok(config) => Arc::new(config),
        Err(error) => {
            eprintln!("Invalid server security configuration: {error}");
            return;
        }
    };

    let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::channel::<console::ServerCommand>(8);
    let server_running = Arc::new(AtomicBool::new(false));

    let console = ConsoleState::new(cmd_tx, server_running.clone());
    console.push_log("info", "Initializing AIRust...").await;

    let database = Database::open("airust.db").expect("Failed to open database");
    let db = Arc::new(database);
    console
        .push_log("info", "Database loaded (airust.db)")
        .await;

    let kb = KnowledgeBase::from_embedded();
    let kb_size = kb.get_examples().len();

    let scheduler = BotScheduler::new(db.clone());

    let state: SharedState = Arc::new(RwLock::new(AppState {
        agents: ServerReset::agents(&kb),
        active_agents: ServerReset::active_agents(),
        knowledge_base: kb,
        db: db.clone(),
        scheduler,
        show_landing,
    }));

    {
        let mut s = state.write().await;
        if let Err(e) = routes::rebuild_kb_from_db_public(&mut s) {
            eprintln!("Warning: Could not load training data from DB: {}", e);
            console
                .push_log("warn", &format!("Could not load training data: {}", e))
                .await;
        }
    }

    console
        .push_log("info", &format!("Knowledge base: {} examples", kb_size))
        .await;

    let console_for_ws = console.clone();
    let console_security = security.clone();
    tokio::spawn(async move {
        let console_app = axum::Router::new()
            .route("/ws/console", axum::routing::get(console::ws_console))
            .layer(axum::Extension(console_for_ws))
            .layer(axum::middleware::from_fn_with_state(
                (console_security, true),
                security::protect,
            ));

        let addr = std::net::SocketAddr::new(bind_ip, console_port);
        let listener = match tokio::net::TcpListener::bind(addr).await {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Failed to bind console port {}: {}", console_port, e);
                return;
            }
        };
        let _ = axum::serve(listener, console_app).await;
    });

    console
        .push_log(
            "info",
            &format!("Console on ws://localhost:{}", console_port),
        )
        .await;

    let mut should_start = true;

    loop {
        if should_start {
            let app = build_main_router(state.clone())
                .layer(axum::extract::DefaultBodyLimit::max(8 * 1024 * 1024))
                .layer(axum::middleware::from_fn_with_state(
                    (security.clone(), false),
                    security::protect,
                ));

            let addr = std::net::SocketAddr::new(bind_ip, port);
            let listener = match tokio::net::TcpListener::bind(addr).await {
                Ok(l) => l,
                Err(e) => {
                    console
                        .push_log("error", &format!("Cannot bind port {}: {}", port, e))
                        .await;
                    eprintln!("Error: Cannot bind to port {}: {}", port, e);
                    should_start = false;
                    continue;
                }
            };

            server_running.store(true, Ordering::SeqCst);
            console
                .push_log(
                    "info",
                    &format!("Server listening on http://localhost:{}", port),
                )
                .await;
            println!("AIRust web server running at http://localhost:{}", port);

            let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
            let mut shutdown_tx = Some(shutdown_tx);

            let mut server_handle = tokio::spawn(async move {
                axum::serve(listener, app)
                    .with_graceful_shutdown(async {
                        let _ = shutdown_rx.await;
                    })
                    .await
            });

            loop {
                tokio::select! {
                    cmd = cmd_rx.recv() => {
                        match cmd {
                            Some(console::ServerCommand::Restart) => {
                                if let Some(tx) = shutdown_tx.take() { let _ = tx.send(()); }
                                let _ = server_handle.await;
                                server_running.store(false, Ordering::SeqCst);
                                console.push_log("info", "Restarting...").await;
                                should_start = true;
                                break;
                            }
                            Some(console::ServerCommand::Reset) => {
                                if let Some(tx) = shutdown_tx.take() { let _ = tx.send(()); }
                                let _ = server_handle.await;
                                server_running.store(false, Ordering::SeqCst);
                                if !ServerReset::run(&console, &state).await {
                                    should_start = false;
                                    break;
                                }
                                console.push_log("info", &format!("Knowledge base: {} examples", state.read().await.knowledge_base.get_examples().len())).await;
                                ServerReset::announce(&console, "Reset complete, restarting server...").await;
                                should_start = true;
                                break;
                            }
                            _ => continue,
                        }
                    }
                    result = &mut server_handle => {
                        server_running.store(false, Ordering::SeqCst);
                        match result {
                            Ok(Ok(())) => console.push_log("warn", "Server exited").await,
                            Ok(Err(e)) => console.push_log("error", &format!("Server error: {}", e)).await,
                            Err(e) => console.push_log("error", &format!("Server panic: {}", e)).await,
                        }
                        should_start = false;
                        break;
                    }
                }
            }
        } else {
            match cmd_rx.recv().await {
                Some(console::ServerCommand::Restart) => {
                    should_start = true;
                }
                Some(console::ServerCommand::Reset) => {
                    should_start = ServerReset::run(&console, &state).await;
                    if should_start {
                        ServerReset::announce(&console, "Reset complete, starting server...").await;
                    }
                }
                _ => {}
            }
        }
    }
}
