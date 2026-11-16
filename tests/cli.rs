#![cfg(feature = "cli")]

use std::{fs, process::Command};

#[test]
fn cli_reports_version_and_rejects_invalid_ports_before_starting() {
    let version = Command::new(env!("CARGO_BIN_EXE_airust"))
        .arg("--version")
        .output()
        .unwrap();
    assert!(version.status.success());
    assert!(String::from_utf8(version.stdout).unwrap().contains("0.1.8"));
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
