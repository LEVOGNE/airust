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
