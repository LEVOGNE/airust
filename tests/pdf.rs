#![cfg(feature = "pdf")]

use airust::pdf_loader::PdfError;
use airust::{PdfLoader, PdfLoaderConfig};

#[cfg(feature = "cli")]
#[path = "support/pdf_fixture.rs"]
mod pdf_fixture;

#[cfg(feature = "cli")]
#[test]
fn output_created_during_conversion_is_not_overwritten() {
    use std::{
        io::{BufRead, BufReader, Read},
        process::{Child, Command, Stdio},
        sync::mpsc,
        time::{Duration, Instant},
    };
    struct ChildGuard(Child);
    impl Drop for ChildGuard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let folder = tempfile::tempdir().unwrap();
    let input = folder.path().join("input.pdf");
    let output = folder.path().join("output.json");
    std::fs::write(&input, pdf_fixture::repeated_pages(500)).unwrap();
    let mut child = ChildGuard(
        Command::new(env!("CARGO_BIN_EXE_pdf2kb"))
            .args([
                input.to_str().unwrap(),
                output.to_str().unwrap(),
                "--verbose",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let stdout = child.0.stdout.take().unwrap();
    let (tx, rx) = mpsc::channel();
    let reader = std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            let line = line.unwrap();
            if line == "Raw preview:" {
                let _ = tx.send(());
            }
        }
    });
    rx.recv_timeout(Duration::from_secs(30))
        .expect("preview must signal completed first extraction");
    std::fs::write(&output, "concurrent owner data").unwrap();
    let deadline = Instant::now() + Duration::from_secs(30);
    let status = loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            break status;
        }
        assert!(
            Instant::now() < deadline,
            "conversion exceeded its test deadline"
        );
        std::thread::sleep(Duration::from_millis(10));
    };
    reader.join().unwrap();
    let mut stderr = String::new();
    child
        .0
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut stderr)
        .unwrap();
    assert!(
        !status.success(),
        "conversion must refuse a late-created output"
    );
    assert!(stderr.contains("already exists"), "{stderr}");
    assert_eq!(
        std::fs::read_to_string(output).unwrap(),
        "concurrent owner data"
    );
}

#[test]
fn unusable_pdf_inputs_fail_instead_of_creating_empty_knowledge() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pdf");
    for name in [
        "blank.pdf",
        "broken.pdf",
        "encrypted.pdf",
        "invalid-header-utf8.pdf",
    ] {
        assert!(
            PdfLoader::new()
                .pdf_to_knowledge_base(root.join(name))
                .is_err(),
            "Unexpected success: {name}"
        );
    }
}

#[cfg(feature = "cli")]
#[test]
fn malformed_unicode_header_returns_a_conversion_error_without_process_panic() {
    let folder = tempfile::tempdir().unwrap();
    let output = folder.path().join("output.json");
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_pdf2kb"))
        .args([
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/pdf/invalid-header-utf8.pdf"
            ),
            output.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&result.stderr).contains("PDF parser panicked"));
    assert!(!output.exists());
}

#[test]
fn exclusive_pdf_save_keeps_existing_output_and_roundtrips_complete_content() {
    let loader = PdfLoader::new();
    let kb = loader
        .pdf_to_knowledge_base(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/pdf/sentences.pdf"
        ))
        .unwrap();
    let folder = tempfile::tempdir().unwrap();
    let output = folder.path().join("output.json");
    loader.save_knowledge_base_if_absent(&kb, &output).unwrap();
    let bytes = std::fs::read(&output).unwrap();
    let error = loader
        .save_knowledge_base_if_absent(&airust::KnowledgeBase::new(), &output)
        .unwrap_err();
    assert!(matches!(error, PdfError::IoError(e) if e.kind()==std::io::ErrorKind::AlreadyExists));
    assert_eq!(std::fs::read(&output).unwrap(), bytes);
    let restored = airust::KnowledgeBase::load(output).unwrap();
    assert_eq!(restored.get_examples().len(), 2);
    assert!(restored.get_examples()[1].input.contains("finalmarker"));
    assert_eq!(
        restored.get_examples()[1].metadata.as_ref().unwrap()["page_number"],
        2
    );
}

#[cfg(feature = "cli")]
#[test]
fn failed_empty_import_preserves_the_previous_output_even_with_overwrite() {
    let folder = tempfile::tempdir().unwrap();
    let output = folder.path().join("output.json");
    std::fs::write(&output, "previous complete data").unwrap();
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_pdf2kb"))
        .args([
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/pdf/blank.pdf"),
            output.to_str().unwrap(),
            "--overwrite",
        ])
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("No usable text"));
    assert_eq!(
        std::fs::read_to_string(output).unwrap(),
        "previous complete data"
    );
}

#[test]
fn invalid_pdf_configuration_is_reported_before_reading_a_file() {
    let loader = PdfLoader::with_config(PdfLoaderConfig {
        max_chunk_size: 100,
        chunk_overlap: 200,
        ..Default::default()
    });
    assert!(matches!(
        loader.pdf_to_training_examples("missing.pdf"),
        Err(PdfError::InvalidConfig(_))
    ));
    for config in [
        PdfLoaderConfig {
            max_chunk_size: 0,
            ..Default::default()
        },
        PdfLoaderConfig {
            min_chunk_size: 1001,
            ..Default::default()
        },
        PdfLoaderConfig {
            default_weight: f32::NAN,
            ..Default::default()
        },
    ] {
        assert!(config.validate().is_err());
    }
}

#[test]
fn actual_pdf_import_keeps_file_page_and_chunk_provenance() {
    let loader = PdfLoader::new();
    let kb = loader
        .pdf_to_knowledge_base(concat!(env!("CARGO_MANIFEST_DIR"), "/data/pdf/sample.pdf"))
        .unwrap();
    assert!(
        !kb.get_examples().is_empty(),
        "sample fixture must contain extractable text"
    );
    for example in kb.get_examples() {
        let metadata = example
            .metadata
            .as_ref()
            .expect("PDF provenance must be retained");
        assert_eq!(metadata["source_file"], "sample.pdf");
        assert_eq!(metadata["page_number"], 1);
        assert!(metadata["chunk_index"].is_number());
        assert!(example.input.chars().count() <= 1000);
    }
}

#[cfg(feature = "cli")]
#[test]
fn pdf_cli_rejects_bad_options_and_preserves_existing_files() {
    use std::{fs, process::Command};
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("input.pdf");
    let output = temp.path().join("output.json");
    fs::copy(
        concat!(env!("CARGO_MANIFEST_DIR"), "/data/pdf/sample.pdf"),
        &input,
    )
    .unwrap();
    let original = fs::read(&input).unwrap();
    fs::write(&output, "existing data").unwrap();
    for args in [
        vec![input.to_str().unwrap(), output.to_str().unwrap()],
        vec![
            input.to_str().unwrap(),
            input.to_str().unwrap(),
            "--overwrite",
        ],
        vec!["--unknown-option"],
    ] {
        assert!(!Command::new(env!("CARGO_BIN_EXE_pdf2kb"))
            .args(args)
            .current_dir(temp.path())
            .output()
            .unwrap()
            .status
            .success());
    }
    assert_eq!(fs::read(&input).unwrap(), original);
    assert_eq!(fs::read_to_string(&output).unwrap(), "existing data");
    assert!(!temp.path().join("knowledge").exists());
    let converted = Command::new(env!("CARGO_BIN_EXE_pdf2kb"))
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--overwrite",
            "--verbose",
        ])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(
        converted.status.success(),
        "{}",
        String::from_utf8_lossy(&converted.stderr)
    );
    assert!(!airust::KnowledgeBase::load(output)
        .unwrap()
        .get_examples()
        .is_empty());
}
