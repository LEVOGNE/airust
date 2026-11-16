#![cfg(feature = "pdf")]

use airust::pdf_loader::PdfError;
use airust::{PdfLoader, PdfLoaderConfig};

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
