#![cfg(feature = "cli")]

use std::{fs, process::Command};

#[test]
fn cli_reports_version_and_rejects_invalid_ports_before_starting() {
    let version = Command::new(env!("CARGO_BIN_EXE_airust"))
        .arg("--version")
        .output()
        .unwrap();
    assert!(version.status.success());
    assert!(String::from_utf8(version.stdout)
        .unwrap()
        .contains(env!("CARGO_PKG_VERSION")));
    for args in [vec!["--port"], vec!["--port", "0"], vec!["--port", "65535"]] {
        let output = Command::new(env!("CARGO_BIN_EXE_airust"))
            .args(args)
            .output()
            .unwrap();
        assert!(!output.status.success());
    }
}

#[test]
fn merge_is_repeatable_and_never_reads_its_destination_as_a_source() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir(temp.path().join("knowledge")).unwrap();
    let source = temp.path().join("knowledge/source.json");
    fs::write(
        &source,
        r#"[{"input":"rust","output":"answer","metadata":{"source":"fixture"}}]"#,
    )
    .unwrap();
    for _ in 0..2 {
        let output = Command::new(env!("CARGO_BIN_EXE_merge_kb"))
            .current_dir(temp.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let merged = airust::KnowledgeBase::load(temp.path().join("knowledge/train.json")).unwrap();
        assert_eq!(merged.get_examples().len(), 1);
        assert!(merged.get_examples()[0].metadata.is_some());
    }
    let destination = temp.path().join("knowledge/train.json");
    let before = fs::read(&destination).unwrap();
    fs::write(&source, "invalid json").unwrap();
    let failed = Command::new(env!("CARGO_BIN_EXE_merge_kb"))
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(!failed.status.success());
    assert_eq!(before, fs::read(&destination).unwrap());
    fs::remove_file(&source).unwrap();
    let no_sources = Command::new(env!("CARGO_BIN_EXE_merge_kb"))
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(!no_sources.status.success());
    assert_eq!(before, fs::read(&destination).unwrap());
}

#[test]
fn query_text_is_not_parsed_as_server_options() {
    let output = Command::new(env!("CARGO_BIN_EXE_airust"))
        .args(["cli", "query", "simple", "--help", "--port", "0"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .starts_with("Answer:"));
    for args in [
        vec!["unknown-command"],
        vec!["cli", "unknown"],
        vec!["cli", "query", "unknown", "question"],
        vec!["cli", "query"],
    ] {
        assert!(!Command::new(env!("CARGO_BIN_EXE_airust"))
            .args(args)
            .output()
            .unwrap()
            .status
            .success());
    }
}

#[test]
fn stop_rejects_unrelated_processes_even_if_state_identity_matches() {
    let temp = tempfile::tempdir().unwrap();
    let mut child = Command::new("sleep").arg("30").spawn().unwrap();
    let identity = Command::new("ps")
        .env("LC_ALL", "C")
        .args([
            "-ww",
            "-p",
            &child.id().to_string(),
            "-o",
            "lstart=",
            "-o",
            "comm=",
        ])
        .output()
        .unwrap();
    let state = serde_json::json!({
        "pid": child.id(), "executable": env!("CARGO_BIN_EXE_airust"),
        "identity": String::from_utf8_lossy(&identity.stdout).trim()
    });
    fs::write(
        temp.path().join("server.json"),
        serde_json::to_vec(&state).unwrap(),
    )
    .unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_airust"))
        .arg("stop")
        .env("AIRUST_RUNTIME_DIR", temp.path())
        .output()
        .unwrap();
    let still_running = child.try_wait().unwrap().is_none();
    let _ = child.kill();
    let _ = child.wait();
    assert!(!result.status.success());
    assert!(still_running, "stop must not signal an unrelated process");
}

#[cfg(unix)]
#[test]
fn stop_rejects_symlinked_runtime_state() {
    use std::os::unix::fs::symlink;
    let temp = tempfile::tempdir().unwrap();
    let target = temp.path().join("target");
    fs::write(&target, "must remain intact").unwrap();
    symlink(&target, temp.path().join("server.json")).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_airust"))
        .arg("stop")
        .env("AIRUST_RUNTIME_DIR", temp.path())
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert_eq!(fs::read_to_string(target).unwrap(), "must remain intact");
}

#[test]
fn merge_can_explicitly_export_embedded_knowledge_without_a_source_directory() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("examples.json");
    let result = Command::new(env!("CARGO_BIN_EXE_merge_kb"))
        .args(["--include-embedded", "--output", output.to_str().unwrap()])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(!airust::KnowledgeBase::load(output)
        .unwrap()
        .get_examples()
        .is_empty());
    let missing_value = Command::new(env!("CARGO_BIN_EXE_merge_kb"))
        .args(["--output", "--include-embedded"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(!missing_value.status.success());
    assert!(!temp.path().join("--include-embedded").exists());
}

#[cfg(feature = "web")]
#[test]
fn web_startup_rejects_invalid_security_configuration_before_creating_runtime_state() {
    for (bind, token) in [
        ("0.0.0.0", None),
        ("127.0.0.1", Some("short")),
        ("not-an-ip-address", None),
    ] {
        for detached in [false, true] {
            let temp = tempfile::tempdir().unwrap();
            let runtime = temp.path().join("runtime");
            let mut command = Command::new(env!("CARGO_BIN_EXE_airust"));
            command
                .current_dir(temp.path())
                .env("AIRUST_RUNTIME_DIR", &runtime)
                .env("AIRUST_BIND_ADDR", bind)
                .env_remove("AIRUST_API_TOKEN")
                .env_remove("AIRUST_ALLOWED_HOSTS");
            if let Some(token) = token {
                command.env("AIRUST_API_TOKEN", token);
            }
            if detached {
                command.arg("-d");
            }
            let output = command.output().unwrap();
            assert!(!output.status.success(), "bind={bind}, detached={detached}");
            assert!(!output.stderr.is_empty());
            assert!(
                !runtime.exists(),
                "Rejected configuration must not create runtime state"
            );
            assert!(!temp.path().join("airust.db").exists());
        }
    }
}
