//! Small model-free stdio host with explicit application resource budgets.
//! These are data/work limits, not a process-wide memory quota.
use airust::mcp::{McpLimits, McpServer};
use std::{error::Error, path::PathBuf};

fn limits() -> McpLimits {
    McpLimits {
        max_entries: 2048,
        max_dataset_bytes: 2 * 1024 * 1024,
        max_entry_bytes: 4096,
        max_id_bytes: 256,
        max_metadata_bytes: 256,
        max_documents: 32,
        max_file_bytes: 4 * 1024 * 1024,
        max_query_bytes: 1024,
        max_hits: 6,
        max_excerpt_bytes: 1024,
        max_result_bytes: 16 * 1024,
        max_concurrent: 1,
        default_timeout_ms: 2000,
        max_timeout_ms: 5000,
        progress_interval_ms: 100,
        max_frame_bytes: 16 * 1024,
        max_pending_requests: 4,
        max_messages_per_second: 64,
        max_requests_per_session: 50_000,
        max_request_id_bytes: 64,
        max_inbound_bytes: 64 * 1024 * 1024,
        max_inbound_bytes_per_second: 128 * 1024,
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.is_empty() || args.len() > 2 || (args.len() == 2 && args[1] != "--check") {
        return Err("expected bounded_mcp DATASET.json [--check]".into());
    }
    let server = McpServer::load_knowledge(PathBuf::from(&args[0]), limits())?;
    if args.len() == 2 {
        println!("{}", serde_json::to_string(&server.stats())?);
        return Ok(());
    }
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        // macOS/Windows stdio uses blocking tasks. Reserve capacity for stdin,
        // stdout/progress and the one admitted CPU worker to avoid starvation.
        .max_blocking_threads(3)
        .build()?
        .block_on(server.serve_stdio())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use airust::{mcp::McpError, KnowledgeEntry};

    #[test]
    fn small_host_validates_and_rejects_entry_or_dataset_overflow() {
        limits().validate().unwrap();
        let oversized = KnowledgeEntry::new("large", "x".repeat(limits().max_entry_bytes + 1));
        assert!(matches!(
            McpServer::from_entries([oversized], limits()),
            Err(McpError::Data(_))
        ));
        let entries =
            (0..=limits().max_entries).map(|i| KnowledgeEntry::new(i.to_string(), "small text"));
        assert!(matches!(
            McpServer::from_entries(entries, limits()),
            Err(McpError::Data(_))
        ));
        let entries = (0..1024).map(|i| KnowledgeEntry::new(i.to_string(), "x".repeat(4096)));
        assert!(matches!(
            McpServer::from_entries(entries, limits()),
            Err(McpError::Data(_))
        ));
    }

    #[test]
    fn rejected_file_inputs_preserve_error_categories_and_originals() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("entries.json");
        let file = std::fs::File::create(&path).unwrap();
        file.set_len(limits().max_file_bytes as u64 + 1).unwrap();
        assert!(matches!(
            McpServer::load_knowledge(&path, limits()),
            Err(McpError::Data(_))
        ));
        assert_eq!(
            file.metadata().unwrap().len(),
            limits().max_file_bytes as u64 + 1
        );
        std::fs::write(&path, [0xFF]).unwrap();
        assert!(matches!(
            McpServer::load_knowledge(&path, limits()),
            Err(McpError::Io(_))
        ));
        assert_eq!(std::fs::read(&path).unwrap(), [0xFF]);
        std::fs::write(&path, b"invalid JSON").unwrap();
        assert!(matches!(
            McpServer::load_knowledge(&path, limits()),
            Err(McpError::Json(_))
        ));
        assert!(matches!(
            McpServer::load_knowledge(dir.path(), limits()),
            Err(McpError::Io(_))
        ));
    }
}
