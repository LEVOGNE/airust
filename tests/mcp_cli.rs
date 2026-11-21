#![cfg(feature = "mcp")]

use airust::{
    documents::{DocumentLimits, DocumentSnapshot, SourceFingerprint, TextSpace},
    Engine, KnowledgeEntry,
};
use serde_json::{json, Value};
use std::{process::Stdio, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::Command,
};

#[test]
fn cli_rejects_missing_ambiguous_and_overflowing_options() {
    let binary = env!("CARGO_BIN_EXE_airust-mcp");
    for args in [
        vec![],
        vec!["--unknown", "x"],
        vec!["--knowledge"],
        vec!["--timeout-ms", "0"],
        vec!["--max-file-mib", "18446744073709551615"],
        vec!["--knowledge", "a", "--knowledge", "b"],
        vec!["--knowledge", "a", "--document", "b"],
        vec!["--max-concurrent", "65"],
    ] {
        let output = std::process::Command::new(binary)
            .args(&args)
            .output()
            .unwrap();
        assert!(!output.status.success(), "{args:?}");
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
    for arg in ["--help", "--version"] {
        let output = std::process::Command::new(binary)
            .arg(arg)
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(!output.stdout.is_empty());
    }
}

#[tokio::test]
async fn actual_stdio_process_loads_knowledge_and_document_files_without_modifying_them() {
    let dir = tempfile::tempdir().unwrap();
    let limits = DocumentLimits::default();
    let document = DocumentSnapshot::new(
        "manual",
        SourceFingerprint::from_bytes(b"original bytes", &limits).unwrap(),
        "supplied-v1",
        TextSpace::Supplied,
        "Rust ownership and memory".into(),
        &limits,
    )
    .unwrap();
    for (flag, bytes) in [
        (
            "--knowledge",
            Engine::from_entries([KnowledgeEntry::new("one", "Rust ownership and memory")])
                .unwrap()
                .to_json()
                .unwrap(),
        ),
        ("--document", document.to_json(10000).unwrap()),
    ] {
        let path = dir.path().join(format!("{flag}.json"));
        std::fs::write(&path, &bytes).unwrap();
        let mut child = Command::new(env!("CARGO_BIN_EXE_airust-mcp"))
            .args([flag, path.to_str().unwrap()])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .unwrap();
        let mut input = child.stdin.take().unwrap();
        let mut output = BufReader::new(child.stdout.take().unwrap());
        let requests = [
            json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"stdio-test","version":"1"}}}),
            json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"airust_search","arguments":{"query":"ownership"}}}),
        ];
        for request in requests {
            input
                .write_all(format!("{request}\n").as_bytes())
                .await
                .unwrap();
            let mut line = String::new();
            tokio::time::timeout(Duration::from_secs(5), output.read_line(&mut line))
                .await
                .unwrap()
                .unwrap();
            let response: Value = serde_json::from_str(&line).unwrap();
            assert_eq!(response["id"], request["id"]);
            if request["id"] == 0 {
                input
                    .write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n")
                    .await
                    .unwrap();
            } else {
                assert_ne!(response["result"]["isError"], true);
                let status =
                    &response["result"]["structuredContent"]["data"]["sources"][0]["status"];
                assert_eq!(
                    status,
                    if flag == "--document" {
                        "snapshot_bound"
                    } else {
                        "not_attached"
                    }
                );
            }
        }
        input.shutdown().await.unwrap();
        drop(input);
        let result = tokio::time::timeout(Duration::from_secs(5), child.wait_with_output())
            .await
            .unwrap()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(std::fs::read(&path).unwrap(), bytes.as_bytes());
    }
}

#[test]
fn bounded_startup_rejects_oversized_or_malformed_input_without_stdout() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("input.json");
    for bytes in [vec![b' '; 1024 * 1024 + 1], b"[{\"id\":\"bad\"}]".to_vec()] {
        std::fs::write(&path, &bytes).unwrap();
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_airust-mcp"))
            .args(["--knowledge", path.to_str().unwrap(), "--max-file-mib", "1"])
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }
}

#[test]
fn hostile_startup_error_fields_cannot_inject_terminal_controls_or_unbounded_logs() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("hostile.json");
    let mut object = serde_json::Map::new();
    object.insert(format!("\u{1b}[2J{}", "x".repeat(10000)), Value::Null);
    std::fs::write(&path, serde_json::to_vec(&object).unwrap()).unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_airust-mcp"))
        .args(["--document", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.contains(&0x1b));
    assert!(output.stderr.len() < 12000);
}
