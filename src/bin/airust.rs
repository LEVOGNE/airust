use airust::agent::{Agent, ContextualAgent, ResponseFormat, TrainableAgent, TrainingExample};
use airust::{ContextAgent, KnowledgeBase, MatchAgent, TfidfAgent};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;

const DEFAULT_PORT: u16 = 7070;
type CliResult = Result<(), Box<dyn std::error::Error>>;

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        process::exit(1);
    }
}

fn run() -> CliResult {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut port = DEFAULT_PORT;
    let mut detached = false;
    let mut daemon = false;
    let mut landing = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--version" | "-V" if args.len() == 1 => {
                println!("{}", airust::version_info());
                return Ok(());
            }
            "help" | "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            "cli" => return handle_cli(&args[i + 1..]),
            "stop" if args.len() == 1 => return stop_server(),
            "--port" => {
                i += 1;
                port = args
                    .get(i)
                    .ok_or("--port requires a value")?
                    .parse::<u16>()?;
                if port == 0 || port == u16::MAX {
                    return Err("Port must be between 1 and 65534".into());
                }
            }
            "-d" => detached = true,
            "--_daemon" => daemon = true,
            "--landingpage" => landing = true,
            value => return Err(format!("Unknown command or option: {value}").into()),
        }
        i += 1;
    }
    #[cfg(feature = "web")]
    {
        let bind_ip = env::var("AIRUST_BIND_ADDR")
            .unwrap_or_else(|_| "127.0.0.1".to_owned())
            .parse::<std::net::IpAddr>()?;
        airust::web::security::SecurityConfig::from_environment(bind_ip, port)?;
        if detached && !daemon {
            start_detached(port, landing)
        } else {
            run_web_server(port, landing)
        }
    }
    #[cfg(not(feature = "web"))]
    {
        let _ = (port, detached, daemon, landing);
        if args.is_empty() {
            print_help();
            Ok(())
        } else {
            Err("Web support requires --features web".into())
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ServerRecord {
    pid: u32,
    executable: PathBuf,
    identity: String,
}

fn runtime_directory() -> io::Result<PathBuf> {
    let path = match env::var_os("AIRUST_RUNTIME_DIR") {
        Some(path) if !path.is_empty() => PathBuf::from(path),
        _ => PathBuf::from(
            env::var_os("HOME")
                .ok_or_else(|| io::Error::other("HOME or AIRUST_RUNTIME_DIR is required"))?,
        )
        .join(".local/state/airust"),
    };
    if !path.is_absolute() {
        return Err(io::Error::other(
            "AIRUST_RUNTIME_DIR must be an absolute path",
        ));
    }
    if !path.exists() {
        let mut builder = fs::DirBuilder::new();
        builder.recursive(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(&path)?;
    }
    let metadata = fs::symlink_metadata(&path)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(io::Error::other(
            "Runtime directory must be a real directory",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(io::Error::other(
                "Runtime directory must have private permissions (0700)",
            ));
        }
    }
    Ok(path)
}

fn record_path() -> io::Result<PathBuf> {
    Ok(runtime_directory()?.join("server.json"))
}

fn read_record() -> Result<Option<ServerRecord>, Box<dyn std::error::Error>> {
    let path = record_path()?;
    match fs::symlink_metadata(&path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
        Ok(metadata) if !metadata.is_file() || metadata.len() > 4096 => {
            return Err("Invalid server state file".into())
        }
        Ok(_) => {}
    }
    Ok(Some(serde_json::from_slice(&fs::read(path)?)?))
}

fn process_identity(pid: u32) -> io::Result<String> {
    if pid <= 1 {
        return Err(io::Error::other("Invalid process ID"));
    }
    let output = process::Command::new("ps")
        .env("LC_ALL", "C")
        .args([
            "-ww",
            "-p",
            &pid.to_string(),
            "-o",
            "lstart=",
            "-o",
            "comm=",
        ])
        .output()?;
    let value = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if !output.status.success() || value.is_empty() {
        return Err(io::Error::other("Process is not running"));
    }
    Ok(value)
}

fn record_is_running(record: &ServerRecord) -> bool {
    if env::current_exe().ok().as_ref() != Some(&record.executable)
        || process_identity(record.pid).ok().as_ref() != Some(&record.identity)
    {
        return false;
    }
    #[cfg(target_os = "linux")]
    {
        fs::read_link(format!("/proc/{}/exe", record.pid))
            .ok()
            .as_ref()
            == Some(&record.executable)
    }
    #[cfg(not(target_os = "linux"))]
    {
        process::Command::new("ps")
            .env("LC_ALL", "C")
            .args(["-ww", "-p", &record.pid.to_string(), "-o", "comm="])
            .output()
            .ok()
            .filter(|output| output.status.success())
            .is_some_and(|output| {
                let name = String::from_utf8_lossy(&output.stdout).trim().to_owned();
                let path = PathBuf::from(name);
                if path.is_absolute() {
                    path == record.executable
                } else {
                    path.file_name() == record.executable.file_name()
                }
            })
    }
}

fn stop_server() -> CliResult {
    let record = read_record()?.ok_or("No AIRust server state found")?;
    if !record_is_running(&record) {
        return Err("Server identity does not match; no process was signaled".into());
    }
    let status = process::Command::new("kill")
        .args(["-TERM", &record.pid.to_string()])
        .status()?;
    if !status.success() {
        return Err("Failed to stop AIRust".into());
    }
    fs::remove_file(record_path()?)?;
    println!("AIRust stopped (PID: {})", record.pid);
    Ok(())
}

#[cfg(feature = "web")]
fn run_web_server(port: u16, landing: bool) -> CliResult {
    if let Some(record) = read_record()? {
        if record_is_running(&record) {
            return Err("AIRust is already running. Use 'airust stop' first.".into());
        }
        fs::remove_file(record_path()?)?;
    }
    let record = ServerRecord {
        pid: process::id(),
        executable: env::current_exe()?,
        identity: process_identity(process::id())?,
    };
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let path = record_path()?;
    let mut file = options.open(&path)?;
    file.write_all(&serde_json::to_vec(&record)?)?;
    file.sync_all()?;
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(airust::web::start_server(port, landing));
    if read_record()?.is_some_and(|current| current.pid == record.pid) {
        fs::remove_file(path)?;
    }
    Ok(())
}

#[cfg(feature = "web")]
fn start_detached(port: u16, landing: bool) -> CliResult {
    if read_record()?.is_some_and(|record| record_is_running(&record)) {
        return Err("AIRust is already running".into());
    }
    let path = runtime_directory()?.join("airust.log");
    if fs::symlink_metadata(&path).is_ok_and(|metadata| !metadata.is_file()) {
        return Err("Log path must be a regular file".into());
    }
    let mut options = fs::OpenOptions::new();
    options.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let log = options.open(&path)?;
    let mut args = vec![
        "--_daemon".to_owned(),
        "--port".to_owned(),
        port.to_string(),
    ];
    if landing {
        args.push("--landingpage".to_owned());
    }
    let mut child = process::Command::new(env::current_exe()?)
        .args(args)
        .stdout(log.try_clone()?)
        .stderr(log)
        .stdin(process::Stdio::null())
        .spawn()?;
    for _ in 0..100 {
        if child.try_wait()?.is_some() {
            return Err(format!("Server failed to start; see {}", path.display()).into());
        }
        if read_record()?
            .is_some_and(|record| record.pid == child.id() && record_is_running(&record))
        {
            println!(
                "AIRust process started (PID: {}). Logs: {}",
                child.id(),
                path.display()
            );
            std::thread::spawn(move || {
                let _ = child.wait();
            });
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    let _ = child.kill();
    let _ = child.wait();
    Err("Server startup timed out".into())
}

fn handle_cli(args: &[String]) -> CliResult {
    if args.is_empty() {
        run_interactive_mode();
        return Ok(());
    }
    match args[0].as_str() {
        "query" if args.len() >= 3 => {
            if !matches!(args[1].as_str(), "simple" | "fuzzy" | "tfidf" | "context") {
                return Err(format!("Unknown agent type: {}", args[1]).into());
            }
            handle_query(&args[1], &args[2..].join(" "));
        }
        "query" => return Err("Usage: airust cli query <agent> <question>".into()),
        "interactive" if args.len() == 1 => run_interactive_mode(),
        "knowledge" if args.len() == 1 => run_knowledge_management(),
        value => return Err(format!("Unknown CLI command or arguments: {value}").into()),
    }
    Ok(())
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
            match weight_str.parse::<f32>() {
                Ok(weight) if weight.is_finite() && weight >= 0.0 => weight,
                _ => {
                    eprintln!("Weight must be a finite, nonnegative number.");
                    continue;
                }
            }
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

        if input.is_empty() || input.eq_ignore_ascii_case("exit") {
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
            match weight_str.parse::<f32>() {
                Ok(weight) if weight.is_finite() && weight >= 0.0 => weight,
                _ => {
                    eprintln!("Weight must be a finite, nonnegative number.");
                    continue;
                }
            }
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
