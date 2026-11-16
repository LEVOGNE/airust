// src/bin/airust.rs – Docker-style CLI
use airust::agent::{Agent, ContextualAgent, ResponseFormat, TrainableAgent, TrainingExample};
use airust::context_agent::ContextAgent;
use airust::knowledge::KnowledgeBase;
use airust::match_agent::MatchAgent;
use airust::tfidf_agent::TfidfAgent;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;

const DEFAULT_PORT: u16 = 7070;
const PID_FILE: &str = "/tmp/airust.pid";

fn main() {
    let args: Vec<String> = env::args().collect();
    if args
        .get(1)
        .is_some_and(|arg| matches!(arg.as_str(), "--version" | "-V"))
    {
        println!("{}", airust::version_info());
        return;
    }

    let mut port: u16 = DEFAULT_PORT;
    let mut detached = false;
    let mut is_daemon = false;
    let mut show_landing = false;
    let mut command: Option<String> = None;
    let mut cli_args_start = args.len();
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "--_daemon" => is_daemon = true,
            "-d" => detached = true,
            "--landingpage" => show_landing = true,
            "--port" => {
                if i + 1 < args.len() {
                    port = args[i + 1].parse().unwrap_or_else(|_| {
                        eprintln!("Invalid port: {}", args[i + 1]);
                        process::exit(1);
                    });
                    if port == 0 || port == u16::MAX {
                        eprintln!("Port must be between 1 and 65534 (console uses port+1)");
                        process::exit(1);
                    }
                    i += 1;
                } else {
                    eprintln!("--port requires a value");
                    process::exit(1);
                }
            }
            "stop" | "cli" | "help" if command.is_none() => {
                command = Some(args[i].clone());
                cli_args_start = i + 1;
            }
            "-h" | "--help" => {
                command = Some("help".to_string());
            }
            _ => {
                if command.is_none() {
                    eprintln!("Unknown command: {}", args[i]);
                    print_help();
                    return;
                }
            }
        }
        i += 1;
    }

    #[cfg(not(feature = "web"))]
    let _ = (port, detached, show_landing);

    // Internal daemon mode (used by -d to respawn)
    if is_daemon {
        #[cfg(feature = "web")]
        run_web_server(port, show_landing);
        return;
    }

    match command.as_deref() {
        Some("stop") => stop_server(),
        Some("cli") => handle_cli(&args[cli_args_start..]),
        Some("help") => print_help(),
        None => {
            #[cfg(feature = "web")]
            {
                if detached {
                    start_detached(port, show_landing);
                } else {
                    run_web_server(port, show_landing);
                }
            }
            #[cfg(not(feature = "web"))]
            {
                print_help();
            }
        }
        _ => print_help(),
    }
}

// ── Web Server ────────────────────────────────────────────────────────────────

#[cfg(feature = "web")]
fn run_web_server(port: u16, show_landing: bool) {
    if is_server_running() {
        eprintln!("AIRust is already running. Use 'airust stop' first.");
        process::exit(1);
    }

    let pid = process::id();
    fs::write(PID_FILE, pid.to_string()).ok();

    tokio::runtime::Runtime::new()
        .expect("Failed to create Tokio runtime")
        .block_on(airust::web::start_server(port, show_landing));

    fs::remove_file(PID_FILE).ok();
}

#[cfg(feature = "web")]
fn start_detached(port: u16, show_landing: bool) {
    if is_server_running() {
        eprintln!("AIRust is already running. Use 'airust stop' first.");
        process::exit(1);
    }

    let exe = env::current_exe().expect("Failed to get executable path");

    let log_file = fs::File::create("/tmp/airust.log").expect("Failed to create log file");
    let err_file = log_file.try_clone().expect("Failed to clone log file");

    let mut daemon_args = vec![
        "--_daemon".to_string(),
        "--port".to_string(),
        port.to_string(),
    ];
    if show_landing {
        daemon_args.push("--landingpage".to_string());
    }

    let mut child = process::Command::new(exe)
        .args(&daemon_args)
        .stdout(log_file)
        .stderr(err_file)
        .stdin(process::Stdio::null())
        .spawn()
        .expect("Failed to start server");

    let pid = child.id();
    // Detach child so it doesn't become a zombie
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    fs::write(PID_FILE, pid.to_string()).ok();

    println!("AIRust running at http://localhost:{}", port);
    println!("  PID:  {}", pid);
    println!("  Logs: /tmp/airust.log");
    println!("  Stop: airust stop");
}

fn stop_server() {
    match fs::read_to_string(PID_FILE) {
        Ok(pid_str) => {
            let pid = pid_str.trim().to_string();
            match process::Command::new("kill").arg(&pid).status() {
                Ok(s) if s.success() => {
                    fs::remove_file(PID_FILE).ok();
                    println!("AIRust stopped (PID: {})", pid);
                }
                _ => {
                    fs::remove_file(PID_FILE).ok();
                    eprintln!("Process {} not found (cleaned up)", pid);
                }
            }
        }
        Err(_) => eprintln!("No running AIRust server found."),
    }
}

#[cfg(feature = "web")]
fn is_server_running() -> bool {
    if let Ok(pid_str) = fs::read_to_string(PID_FILE) {
        let pid = pid_str.trim();
        // Don't consider our own process as "already running"
        if pid == process::id().to_string() {
            return false;
        }
        if let Ok(status) = process::Command::new("kill")
            .args(["-0", pid])
            .stderr(process::Stdio::null())
            .status()
        {
            if status.success() {
                return true;
            }
        }
        fs::remove_file(PID_FILE).ok();
    }
    false
}

// ── CLI Mode ──────────────────────────────────────────────────────────────────

fn handle_cli(args: &[String]) {
    if args.is_empty() {
        run_interactive_mode();
        return;
    }

    match args[0].as_str() {
        "query" => {
            if args.len() < 3 {
                eprintln!("Usage: airust cli query <agent> <question>");
                eprintln!("Agents: simple, fuzzy, tfidf, context");
                return;
            }
            handle_query(&args[1], &args[2..].join(" "));
        }
        "interactive" => run_interactive_mode(),
        "knowledge" => run_knowledge_management(),
        _ => {
            eprintln!("Unknown CLI command: {}", args[0]);
            eprintln!("Available: query, interactive, knowledge");
        }
    }
}

fn print_help() {
    println!(
        "{} — Modular Knowledge Intelligence Engine",
        airust::version_info()
    );
    println!(
        "Web dashboard: compile with --features web; library example: cargo run --example core"
    );
    println!();
    println!("Usage:");
    println!(
        "  airust                 Start web server (port {})",
        DEFAULT_PORT
    );
    println!("  airust --port <PORT>   Start on specific port");
    println!("  airust -d              Start in background (detached)");
    println!("  airust --landingpage   Show landing page + dashboard");
    println!("  airust stop            Stop background server");
    println!("  airust cli             Interactive CLI mode");
    println!("  airust help            Show this help");
    println!();
    println!("CLI subcommands:");
    println!("  airust cli query <agent> <question>");
    println!("  airust cli knowledge");
    println!();
    println!("Agents: simple, fuzzy, tfidf, context");
}

// ── Query & Interactive ───────────────────────────────────────────────────────

fn handle_query(agent_type: &str, question: &str) {
    let kb = KnowledgeBase::from_embedded();
    let examples = kb.get_examples();

    let answer = match agent_type {
        "simple" => {
            let mut agent = MatchAgent::new_exact();
            agent.train(examples);
            agent.predict(question)
        }
        "fuzzy" => {
            let mut agent = MatchAgent::new_fuzzy();
            agent.train(examples);
            agent.predict(question)
        }
        "tfidf" => {
            let mut agent = TfidfAgent::new();
            agent.train(examples);
            agent.predict(question)
        }
        "context" => {
            let mut base_agent = TfidfAgent::new();
            base_agent.train(examples);
            let agent = ContextAgent::new(base_agent, 3);
            agent.predict(question)
        }
        _ => ResponseFormat::Text(format!("Unknown agent type: {}", agent_type)),
    };

    println!("Answer: {}", String::from(answer));
}

fn run_interactive_mode() {
    println!("=== Interactive Mode ===");
    println!("Select an agent type:");
    println!("1. Exact (SimpleAgent)");
    println!("2. Fuzzy (FuzzyAgent)");
    println!("3. TFIDF (TfidfAgent)");
    println!("4. Context (ContextAgent)");
    print!("> ");
    let _ = io::stdout().flush();

    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    let choice = input.trim();

    let kb = KnowledgeBase::from_embedded();
    let examples = kb.get_examples();

    match choice {
        "1" => {
            let mut a = MatchAgent::new_exact();
            a.train(examples);
            interactive_loop(&a, "Exact Matching Agent");
        }
        "2" => {
            let mut a = MatchAgent::new_fuzzy();
            a.train(examples);
            interactive_loop(&a, "Fuzzy Matching Agent");
        }
        "3" => {
            let mut a = TfidfAgent::new();
            a.train(examples);
            interactive_loop(&a, "TFIDF Agent (BM25)");
        }
        "4" => interactive_loop_context(examples),
        _ => println!("Invalid selection. Please restart the program."),
    }
}

fn interactive_loop(agent: &impl Agent, name: &str) {
    println!("=== {} ===", name);
    println!("Enter questions or 'exit' to quit.");

    loop {
        print!("> ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }
        let input = input.trim();

        if input.is_empty() || input.to_lowercase() == "exit" {
            break;
        }

        let answer = agent.predict(input);
        println!("Answer: {}", String::from(answer));
    }
}

fn interactive_loop_context(examples: &[TrainingExample]) {
    println!("=== Context Agent ===");
    println!("Enter questions or 'exit' to quit.");

    let mut base = TfidfAgent::new();
    base.train(examples);
    let mut agent = ContextAgent::new(base, 3);

    loop {
        print!("> ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }
        let input = input.trim();

        if input.is_empty() || input.to_lowercase() == "exit" {
            break;
        }

        let answer = agent.predict(input);
        println!("Answer: {}", String::from(answer.clone()));
        agent.add_context(input.to_string(), answer);
    }
}

// ── Knowledge Management ──────────────────────────────────────────────────────

fn run_knowledge_management() {
    println!("=== Knowledge Base Management ===");
    println!("1. Create new knowledge base");
    println!("2. Load knowledge base");
    println!("3. Back");
    print!("> ");
    let _ = io::stdout().flush();

    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    let choice = input.trim();

    match choice {
        "1" => create_knowledge_base(),
        "2" => load_knowledge_base(),
        _ => (),
    }
}

fn create_knowledge_base() {
    let mut kb = KnowledgeBase::new();

    println!("=== Create New Knowledge Base ===");
    println!("Enter examples. Press Enter without input to finish.");

    loop {
        println!("\nNew example:");
        print!("Question: ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);
        let input = input.trim();

        if input.is_empty() {
            break;
        }

        print!("Answer: ");
        let _ = io::stdout().flush();

        let mut output = String::new();
        let _ = io::stdin().read_line(&mut output);
        let output = output.trim();

        print!("Weight (Default 1.0): ");
        let _ = io::stdout().flush();

        let mut weight_str = String::new();
        let _ = io::stdin().read_line(&mut weight_str);
        let weight_str = weight_str.trim();

        let weight = if weight_str.is_empty() {
            1.0
        } else {
            weight_str.parse::<f32>().unwrap_or(1.0)
        };

        kb.add_example(
            input.to_string(),
            ResponseFormat::Text(output.to_string()),
            weight,
        );
        println!("Example added!");
    }

    println!("\nEnter path to save:");
    print!("> ");
    let _ = io::stdout().flush();

    let mut path_str = String::new();
    let _ = io::stdin().read_line(&mut path_str);
    let path_str = path_str.trim();

    if path_str.is_empty() {
        println!("No path specified. Aborting.");
        return;
    }

    let path = PathBuf::from(path_str);
    match kb.save(Some(path.clone())) {
        Ok(_) => {
            println!("Knowledge base saved to {:?}", path);
            println!("Would you like to test the knowledge base? (y/n)");
            print!("> ");
            let _ = io::stdout().flush();

            let mut test = String::new();
            let _ = io::stdin().read_line(&mut test);
            let test = test.trim();

            if test.to_lowercase() == "y" {
                test_knowledge_base(&kb);
            }
        }
        Err(e) => println!("Error saving: {}", e),
    }
}

fn load_knowledge_base() {
    println!("Enter path to knowledge base:");
    print!("> ");
    let _ = io::stdout().flush();

    let mut path_str = String::new();
    let _ = io::stdin().read_line(&mut path_str);
    let path_str = path_str.trim();

    if path_str.is_empty() {
        println!("No path specified. Aborting.");
        return;
    }

    let path = PathBuf::from(path_str);
    match KnowledgeBase::load(path) {
        Ok(kb) => {
            println!(
                "Knowledge base loaded! {} examples found.",
                kb.get_examples().len()
            );
            println!("\nWhat would you like to do?");
            println!("1. Test knowledge base");
            println!("2. Add examples");
            println!("3. Back");
            print!("> ");
            let _ = io::stdout().flush();

            let mut choice = String::new();
            let _ = io::stdin().read_line(&mut choice);
            let choice = choice.trim();

            match choice {
                "1" => test_knowledge_base(&kb),
                "2" => add_examples_to_kb(kb),
                _ => (),
            }
        }
        Err(e) => println!("Error loading: {}", e),
    }
}

fn test_knowledge_base(kb: &KnowledgeBase) {
    println!("=== Test Knowledge Base ===");
    println!("Select an agent type for testing:");
    println!("1. Exact (SimpleAgent)");
    println!("2. Fuzzy (FuzzyAgent)");
    println!("3. TFIDF (TfidfAgent)");
    print!("> ");
    let _ = io::stdout().flush();

    let mut choice = String::new();
    let _ = io::stdin().read_line(&mut choice);
    let choice = choice.trim();

    let examples = kb.get_examples();

    match choice {
        "1" => {
            let mut agent = MatchAgent::new_exact();
            agent.train(examples);
            test_loop(&agent)
        }
        "2" => {
            let mut agent = MatchAgent::new_fuzzy();
            agent.train(examples);
            test_loop(&agent)
        }
        "3" => {
            let mut agent = TfidfAgent::new();
            agent.train(examples);
            test_loop(&agent)
        }
        _ => println!("Invalid selection."),
    }
}

fn test_loop(agent: &impl Agent) {
    println!("Ask questions or enter 'exit' to quit.");

    loop {
        print!("> ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);
        let input = input.trim();

        if input.to_lowercase() == "exit" {
            break;
        }

        let answer = agent.predict(input);
        println!("Answer: {}", String::from(answer));
    }
}

fn add_examples_to_kb(mut kb: KnowledgeBase) {
    println!("=== Add Examples ===");
    println!("Enter examples. Press Enter without input to finish.");

    loop {
        println!("\nNew example:");
        print!("Question: ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);
        let input = input.trim();

        if input.is_empty() {
            break;
        }

        print!("Answer: ");
        let _ = io::stdout().flush();

        let mut output = String::new();
        let _ = io::stdin().read_line(&mut output);
        let output = output.trim();

        print!("Weight (Default 1.0): ");
        let _ = io::stdout().flush();

        let mut weight_str = String::new();
        let _ = io::stdin().read_line(&mut weight_str);
        let weight_str = weight_str.trim();

        let weight = if weight_str.is_empty() {
            1.0
        } else {
            weight_str.parse::<f32>().unwrap_or(1.0)
        };

        kb.add_example(
            input.to_string(),
            ResponseFormat::Text(output.to_string()),
            weight,
        );
        println!("Example added!");
    }

    println!("\nWould you like to save the changes? (y/n)");
    print!("> ");
    let _ = io::stdout().flush();

    let mut save = String::new();
    let _ = io::stdin().read_line(&mut save);
    let save = save.trim();

    if save.to_lowercase() == "y" {
        match kb.save(None) {
            Ok(_) => println!("Knowledge base saved!"),
            Err(e) => println!("Error saving: {}", e),
        }
    }
}
