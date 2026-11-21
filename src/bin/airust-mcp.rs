use airust::mcp::{McpLimits, McpServer};
use std::{io::Write, path::PathBuf, process::ExitCode, time::Duration};
const USAGE:&str="AIRust MCP server\n\nUsage:\n  airust-mcp --knowledge FILE [OPTIONS]\n  airust-mcp --document SNAPSHOT.json [--document SNAPSHOT.json ...] [OPTIONS]\n\nOptions:\n  --timeout-ms N        Default operation budget (1..60000; default 10000)\n  --max-concurrent N    Worker slots (1..64; default 4)\n  --max-file-mib N      Maximum bytes per input file in MiB (default 256)\n  --max-result-kib N    Tool result budget in KiB (4..1024; default 64)\n  --help               Print this help and exit\n  --version            Print the version and exit\n\nServer stdout contains MCP JSON-RPC only. Startup messages go to stderr.\nFiles are read at startup; tools cannot open paths, write data, run commands or fetch URLs.";
fn main() -> ExitCode {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args == ["--help"] {
        return print_line(USAGE);
    }
    if args == ["--version"] {
        return print_line(&format!("airust-mcp {}", airust::VERSION));
    }
    match run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let mut message = String::new();
            for c in error.to_string().chars().take(2048) {
                if c.is_control() {
                    message.extend(c.escape_default());
                } else {
                    message.push(c);
                }
            }
            let _ = writeln!(std::io::stderr(), "airust-mcp: {message}");
            ExitCode::FAILURE
        }
    }
}
fn print_line(text: &str) -> ExitCode {
    if writeln!(std::io::stdout(), "{text}").is_ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
fn run(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let mut knowledge = None;
    let mut documents = Vec::new();
    let mut limits = McpLimits::default();
    let mut i = 0;
    while i < args.len() {
        let key = &args[i];
        i += 1;
        let value = args
            .get(i)
            .filter(|v| !v.starts_with('-'))
            .ok_or("each option requires a value; use --help for usage")?;
        i += 1;
        match key.as_str() {
            "--knowledge" => {
                if knowledge.replace(PathBuf::from(value)).is_some() {
                    return Err("--knowledge may appear only once".into());
                }
            }
            "--document" => {
                if documents.len() >= limits.max_documents {
                    return Err("too many documents".into());
                }
                documents.push(PathBuf::from(value));
            }
            "--timeout-ms" => limits.default_timeout_ms = value.parse()?,
            "--max-concurrent" => limits.max_concurrent = value.parse()?,
            "--max-file-mib" => {
                limits.max_file_bytes = value
                    .parse::<usize>()?
                    .checked_mul(1024 * 1024)
                    .ok_or("file byte limit overflow")?
            }
            "--max-result-kib" => {
                limits.max_result_bytes = value
                    .parse::<usize>()?
                    .checked_mul(1024)
                    .ok_or("result byte limit overflow")?
            }
            _ => return Err("unknown option; use --help for usage".into()),
        }
    }
    limits.validate()?;
    let server = match (knowledge, documents.is_empty()) {
        (Some(path), true) => McpServer::load_knowledge(path, limits)?,
        (None, false) => McpServer::load_documents(documents, limits)?,
        _ => {
            return Err(
                "choose either --knowledge FILE or one or more --document FILE arguments".into(),
            )
        }
    };
    let _ = writeln!(
        std::io::stderr(),
        "AIRust {}: loaded {} entries and {} document revisions; read-only stdio MCP ready",
        airust::VERSION,
        server.stats().entries,
        server.stats().documents
    );
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()?;
    let result=runtime.block_on(async{tokio::select!{result=server.serve_stdio()=>result,signal=tokio::signal::ctrl_c()=>signal.map_err(airust::mcp::McpError::Io)}});
    runtime.shutdown_timeout(Duration::from_secs(1));
    result?;
    Ok(())
}
