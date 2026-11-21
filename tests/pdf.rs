#![cfg(feature = "pdf")]

use airust::pdf_loader::PdfError;
use airust::{PdfLoader, PdfLoaderConfig};

#[test]
fn structural_limits_return_errors_instead_of_partial_knowledge() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pdf");
    let loader = PdfLoader::with_config(PdfLoaderConfig {
        min_chunk_size: 1,
        clean_text: false,
        ..Default::default()
    });
    let pages = loader
        .extract_pages_from_pdf(&root.join("forms-depth-10.pdf"))
        .unwrap();
    assert_eq!(pages.len(), 1);
    for marker in ["AIRUSTFORMSTART", "AIRUSTFORMEND", "AIRUSTFORMLEAF"] {
        assert!(pages[0].text.contains(marker), "missing {marker}");
    }
    for fixture in ["forms-depth-11.pdf", "fallback-count-depth-50.pdf"] {
        let path = root.join(fixture);
        for _ in 0..2 {
            assert!(matches!(
                loader.extract_pages_from_pdf(&path),
                Err(PdfError::PdfError(_))
            ));
            assert!(loader.pdf_to_training_examples(&path).is_err());
            assert!(loader.pdf_to_knowledge_base(&path).is_err());
        }
    }
}

#[cfg(feature = "cli")]
#[test]
fn structural_limit_failures_preserve_output_in_direct_and_supervised_jobs() {
    let folder = tempfile::tempdir().unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pdf");
    let output = folder.path().join("retained.json");
    for fixture in ["forms-depth-11.pdf", "fallback-count-depth-50.pdf"] {
        for supervised in [false, true] {
            std::fs::write(&output, b"previous complete knowledge").unwrap();
            let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_pdf2kb"));
            command
                .arg(root.join(fixture))
                .arg(&output)
                .arg("--overwrite");
            if supervised {
                command.args(["--timeout-seconds", "30"]);
            }
            let result = command.output().unwrap();
            assert_eq!(result.status.code(), Some(1), "fixture={fixture}");
            assert!(String::from_utf8_lossy(&result.stderr).contains("PDF conversion failed"));
            assert_eq!(
                std::fs::read(&output).unwrap(),
                b"previous complete knowledge"
            );
        }
    }
}

#[test]
fn long_import_chunks_retain_original_line_breaks() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("line-breaks.pdf");
    let lines = [
        "First sentence.",
        "Second sentence.",
        "Third sentence.",
        "Fourth sentence.",
    ];
    let mut contents = "BT /F1 12 Tf 40 740 Td ".to_owned();
    for (i, line) in lines.iter().enumerate() {
        if i != 0 {
            contents.push_str("0 -28 Td ");
        }
        contents.push_str(&format!("({line}) Tj "));
    }
    contents.push_str("ET");
    std::fs::write(&path, pdf_fixture::pages_with_contents(&[contents])).unwrap();
    for clean_text in [false, true] {
        let loader = PdfLoader::with_config(PdfLoaderConfig {
            max_chunk_size: 40,
            min_chunk_size: 10,
            chunk_overlap: 5,
            clean_text,
            ..Default::default()
        });
        let pages = loader.extract_pages_from_pdf(&path).unwrap();
        let examples = loader.pdf_to_training_examples(&path).unwrap();
        assert!(examples.len() > 1);
        assert!(examples.iter().any(|e| e.input.contains('\n')));
        for example in examples {
            assert!(
                pages[0].text.contains(&example.input),
                "fabricated excerpt: {:?}",
                example.input
            );
            assert!(example.input.chars().count() <= 40);
            assert_eq!(example.metadata.as_ref().unwrap()["page_number"], 1);
        }
    }
}

#[test]
fn preview_stops_at_usable_pages_and_preserves_physical_positions() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("sparse.pdf");
    let contents = [
        String::new(),
        "BT /F1 12 Tf 40 740 Td (First useful page with enough words to retain.) Tj ET".to_owned(),
        String::new(),
        "BT /F1 12 Tf 40 740 Td (Second useful page with different text to retain.) Tj ET"
            .to_owned(),
    ];
    std::fs::write(&path, pdf_fixture::pages_with_contents(&contents)).unwrap();
    for clean_text in [true, false] {
        let loader = PdfLoader::with_config(PdfLoaderConfig {
            clean_text,
            ..Default::default()
        });
        let full = loader.extract_pages_from_pdf(&path).unwrap();
        assert_eq!(
            full.iter().map(|p| p.page_number).collect::<Vec<_>>(),
            [2, 4]
        );
        for maximum in [1, 2, usize::MAX] {
            let preview = loader.preview_pages_from_pdf(&path, maximum).unwrap();
            assert_eq!(preview.len(), maximum.min(full.len()));
            for (a, b) in preview.iter().zip(&full) {
                assert_eq!(a.page_number, b.page_number);
                assert_eq!(a.text, b.text);
            }
        }
    }
}

#[test]
fn preview_enforces_whole_file_limits_and_visited_text_limits() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("two-pages.pdf");
    let bytes = pdf_fixture::repeated_pages(2);
    std::fs::write(&path, &bytes).unwrap();
    let loader = PdfLoader::new().with_limits(bounded_limits(bytes.len() as u64, 2, 4096, 1));
    assert_eq!(loader.preview_pages_from_pdf(&path, 1).unwrap().len(), 1);
    for result in [
        loader.preview_pages_from_pdf(&path, 2),
        loader.extract_pages_from_pdf(&path),
    ] {
        assert!(result
            .err()
            .expect("full text exceeds budget")
            .to_string()
            .contains("extracted text bytes"));
    }
    for (file_bytes, pages, text, expected) in [
        (bytes.len() as u64 - 1, 2, 4096, "input file bytes"),
        (bytes.len() as u64, 1, 4096, "page count"),
        (bytes.len() as u64, 2, 1, "extracted text bytes"),
    ] {
        let error = PdfLoader::new()
            .with_limits(bounded_limits(file_bytes, pages, text, 1))
            .preview_pages_from_pdf(&path, 1)
            .err()
            .expect("preview exceeds budget");
        assert!(error.to_string().contains(expected), "{error}");
    }
    assert_eq!(std::fs::read(path).unwrap(), bytes);
}

#[test]
fn preview_validates_arguments_before_io_and_accepts_empty_documents() {
    let missing = std::path::Path::new("missing-preview-test.pdf");
    assert!(matches!(
        PdfLoader::new().preview_pages_from_pdf(missing, 0),
        Err(PdfError::InvalidConfig(_))
    ));
    let loader = PdfLoader::with_config(PdfLoaderConfig {
        max_chunk_size: 0,
        ..Default::default()
    });
    assert!(matches!(
        loader.preview_pages_from_pdf(missing, 1),
        Err(PdfError::InvalidConfig(_))
    ));
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("blank.pdf");
    std::fs::write(&path, pdf_fixture::pages_with_contents(&[String::new()])).unwrap();
    assert!(PdfLoader::new()
        .preview_pages_from_pdf(&path, 3)
        .unwrap()
        .is_empty());
}

#[cfg(feature = "cli")]
#[test]
fn verbose_preview_does_not_scan_the_document_before_chunk_failure() {
    let folder = tempfile::tempdir().unwrap();
    let input = folder.path().join("many-pages.pdf");
    let output = folder.path().join("previous.json");
    std::fs::write(&input, pdf_fixture::repeated_pages(500)).unwrap();
    std::fs::write(&output, b"previous knowledge").unwrap();
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_pdf2kb"))
        .arg(&input)
        .arg(&output)
        .args([
            "--verbose",
            "--max-text-mib",
            "1",
            "--max-chunks",
            "1",
            "--overwrite",
        ])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(1));
    let message = String::from_utf8_lossy(&result.stderr);
    assert!(message.contains("chunk count"), "{message}");
    assert_eq!(std::fs::read(output).unwrap(), b"previous knowledge");
}

#[test]
fn imported_pdf_keeps_internal_numbers_and_wrapped_ranges() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("numbers-and-ranges.pdf");
    let lines = [
        "Header",
        "100",
        "200",
        "Range 10-",
        "20 units.",
        "Footer",
        "42",
    ];
    let mut contents = "BT /F1 12 Tf 40 740 Td ".to_owned();
    for (i, line) in lines.iter().enumerate() {
        if i != 0 {
            contents.push_str("0 -28 Td ");
        }
        contents.push_str(&format!("({line}) Tj "));
    }
    contents.push_str("ET");
    std::fs::write(&path, pdf_fixture::pages_with_contents(&[contents])).unwrap();
    let examples = PdfLoader::new().pdf_to_training_examples(&path).unwrap();
    assert_eq!(examples.len(), 1);
    let input = &examples[0].input;
    for expected in ["100", "200", "10-", "20 units."] {
        assert!(input.contains(expected), "{input:?} misses {expected:?}");
    }
    assert!(!input.contains("1020"));
    assert!(!input.ends_with("42"));
    assert_eq!(examples[0].metadata.as_ref().unwrap()["page_number"], 1);
    let index =
        airust::Engine::from_entries(vec![airust::KnowledgeEntry::new("table", input.clone())])
            .unwrap();
    assert_eq!(index.search("100", 1)[0].entry.id, "table");
    assert!(index.search("1020", 1).is_empty());
}

#[test]
fn exhausted_chunk_budgets_stop_before_extracting_later_pages() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("two-pages.pdf");
    std::fs::write(&path, pdf_fixture::repeated_pages(2)).unwrap();
    for (chunks, overlap, expected) in [(1, 200, "chunk count"), (1000, 950, "chunk text bytes")] {
        let loader = PdfLoader::with_config(PdfLoaderConfig {
            chunk_overlap: overlap,
            ..Default::default()
        })
        .with_limits(bounded_limits(1024 * 1024, 2, 4096, chunks));
        let raw_error = loader
            .extract_pages_from_pdf(&path)
            .err()
            .expect("both raw pages exceed the text budget");
        assert!(raw_error.to_string().contains("extracted text bytes"));
        let result = loader.pdf_to_training_examples(&path).unwrap_err();
        assert!(result.to_string().contains(expected), "{result}");
    }
}

fn bounded_limits(
    file_bytes: u64,
    pages: usize,
    text_bytes: usize,
    chunks: usize,
) -> airust::PdfImportLimits {
    airust::PdfImportLimits::new(file_bytes, pages, text_bytes, chunks).unwrap()
}

#[test]
fn bounded_pdf_file_and_page_limits_have_exact_boundaries() {
    let path = std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/pdf/sentences.pdf"
    ));
    let bytes = std::fs::metadata(path).unwrap().len();
    let baseline = PdfLoader::new().pdf_to_training_examples(path).unwrap();
    let bounded = PdfLoader::new()
        .with_limits(bounded_limits(bytes, 2, 100000, 2))
        .pdf_to_training_examples(path)
        .unwrap();
    assert_eq!(
        serde_json::to_value(baseline).unwrap(),
        serde_json::to_value(bounded).unwrap()
    );
    let file_error = PdfLoader::new()
        .with_limits(bounded_limits(bytes - 1, 2, 100000, 2))
        .pdf_to_training_examples(path)
        .unwrap_err();
    assert!(file_error.to_string().contains("input file bytes"));
    let page_error = PdfLoader::new()
        .with_limits(bounded_limits(bytes, 1, 100000, 2))
        .pdf_to_training_examples(path)
        .unwrap_err();
    assert!(page_error.to_string().contains("page count"));
}

#[test]
fn bounded_pdf_limits_reject_zero_and_overflowing_configuration() {
    use airust::PdfImportLimits;
    fn stable_traits<
        T: Send + Sync + std::panic::UnwindSafe + std::panic::RefUnwindSafe + Unpin,
    >() {
    }
    stable_traits::<PdfLoader>();
    stable_traits::<PdfImportLimits>();
    for args in [
        (0, 1, 1, 1),
        (u64::MAX, 1, 1, 1),
        (1024, 0, 1, 1),
        (1024, 1, 0, 1),
        (1024, 1, 1, 0),
    ] {
        assert!(PdfImportLimits::new(args.0, args.1, args.2, args.3).is_err());
    }
    let limits = PdfImportLimits::default();
    assert_eq!(limits.max_pages(), 20000);
    assert_eq!(limits.max_file_bytes(), 256 * 1024 * 1024);
}

#[test]
fn bounded_memory_reader_rejects_bad_header_without_native_panic() {
    let result = PdfLoader::new()
        .with_limits(airust::PdfImportLimits::default())
        .pdf_to_training_examples(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/pdf/invalid-header-utf8.pdf"
        ));
    assert!(matches!(result, Err(PdfError::PdfError(message)) if !message.contains("panicked")));
}

#[cfg(all(feature = "cli", unix))]
#[test]
fn cli_non_unicode_path_and_option_values_never_panic() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt, process::Command};
    let folder = tempfile::tempdir().unwrap();
    let output = folder.path().join("output.json");
    let missing = folder
        .path()
        .join(OsString::from_vec(b"missing-\xff.pdf".to_vec()));
    let missing_result = Command::new(env!("CARGO_BIN_EXE_pdf2kb"))
        .arg(missing)
        .arg(&output)
        .output()
        .unwrap();
    assert_eq!(missing_result.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&missing_result.stderr).contains("not a regular file"));
    let value_result = Command::new(env!("CARGO_BIN_EXE_pdf2kb"))
        .arg("--max-pages")
        .arg(OsString::from_vec(vec![0xff]))
        .output()
        .unwrap();
    assert_eq!(value_result.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&value_result.stderr).contains("valid Unicode value"));
    assert!(!output.exists());
}

#[cfg(feature = "cli")]
#[test]
fn cli_escapes_terminal_sequences_in_paths_without_changing_io_paths() {
    use std::process::Command;
    let folder = tempfile::tempdir().unwrap();
    let output = folder.path().join("result-\u{1b}[31m.json");
    let result = Command::new(env!("CARGO_BIN_EXE_pdf2kb"))
        .args([
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/pdf/sentences.pdf"
            ),
            output.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(output.exists());
    assert!(!result.stdout.contains(&0x1b));
    let failed = Command::new(env!("CARGO_BIN_EXE_pdf2kb"))
        .arg(folder.path().join("missing-\u{1b}[31m.pdf"))
        .output()
        .unwrap();
    assert_eq!(failed.status.code(), Some(1));
    assert!(!failed.stderr.contains(&0x1b));
}

#[test]
fn bounded_extraction_rejects_nonregular_paths_before_reading() {
    let folder = tempfile::tempdir().unwrap();
    assert!(matches!(
        PdfLoader::new()
            .with_limits(airust::PdfImportLimits::default())
            .extract_pages_from_pdf(folder.path()),
        Err(PdfError::InvalidPath(_))
    ));
}

#[test]
fn text_and_chunk_limits_apply_across_pages_and_overlap() {
    let path = std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/pdf/sentences.pdf"
    ));
    let doc = pdf_oxide::PdfDocument::open(path).unwrap();
    let lengths = (0..2)
        .map(|i| doc.extract_text(i).unwrap().len())
        .collect::<Vec<_>>();
    let raw_total = lengths.iter().sum::<usize>();
    let max_page = *lengths.iter().max().unwrap();
    let raw_error = PdfLoader::new()
        .with_limits(bounded_limits(100000, 2, max_page, 100))
        .pdf_to_training_examples(path)
        .unwrap_err();
    assert!(raw_error.to_string().contains("extracted text bytes"));
    let count_error = PdfLoader::new()
        .with_limits(bounded_limits(100000, 2, 100000, 1))
        .pdf_to_training_examples(path)
        .unwrap_err();
    assert!(count_error.to_string().contains("chunk count"));
    let loader = PdfLoader::with_config(PdfLoaderConfig {
        min_chunk_size: 1,
        max_chunk_size: 20,
        chunk_overlap: 19,
        split_by_paragraph: false,
        split_by_sentence: false,
        ..Default::default()
    })
    .with_limits(bounded_limits(100000, 2, raw_total, 1000));
    assert!(loader
        .pdf_to_training_examples(path)
        .unwrap_err()
        .to_string()
        .contains("chunk text bytes"));
}

#[test]
fn chunk_budget_stops_high_overlap_before_expanding_a_whole_page() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/pdf/sentences.pdf"
    );
    let loader = PdfLoader::with_config(PdfLoaderConfig {
        min_chunk_size: 1,
        max_chunk_size: 10,
        chunk_overlap: 9,
        split_by_paragraph: false,
        split_by_sentence: false,
        ..Default::default()
    })
    .with_limits(bounded_limits(100000, 2, 100000, 3));
    assert!(loader
        .pdf_to_training_examples(path)
        .unwrap_err()
        .to_string()
        .contains("chunk count"));
}

#[cfg(feature = "cli")]
#[test]
fn cli_limit_failures_preserve_existing_output_and_reject_overflow() {
    use std::process::Command;
    let folder = tempfile::tempdir().unwrap();
    let output = folder.path().join("output.json");
    std::fs::write(&output, "original complete data").unwrap();
    let input = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/pdf/sentences.pdf"
    );
    for flags in [
        vec!["--max-pages", "1"],
        vec!["--max-chunks", "1"],
        vec!["--max-text-mib", "0"],
        vec!["--max-file-mib", "18446744073709551615"],
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_pdf2kb"))
            .args([input, output.to_str().unwrap(), "--overwrite", "--verbose"])
            .args(flags)
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(1));
        assert_eq!(
            std::fs::read_to_string(&output).unwrap(),
            "original complete data"
        );
    }
}

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
    assert!(String::from_utf8_lossy(&result.stderr).contains("PDF conversion failed"));
    assert!(!String::from_utf8_lossy(&result.stderr).contains("panicked"));
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

#[cfg(feature = "cli")]
#[test]
fn supervised_cli_preserves_config_and_complete_output() {
    use std::process::Command;
    let folder = tempfile::tempdir().unwrap();
    let plain = folder.path().join("plain.json");
    let supervised = folder.path().join("supervised.json");
    let input = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/pdf/sentences.pdf"
    );
    let settings = [
        "--min-chunk",
        "5",
        "--max-chunk",
        "80",
        "--overlap",
        "10",
        "--weight",
        "0.75",
        "--no-metadata",
        "--no-clean",
        "--no-paragraph-split",
        "--no-sentence-split",
        "--no-dehyphenate",
        "--keep-page-numbers",
    ];
    let a = Command::new(env!("CARGO_BIN_EXE_pdf2kb"))
        .arg(input)
        .arg(&plain)
        .args(settings)
        .output()
        .unwrap();
    assert!(a.status.success(), "{}", String::from_utf8_lossy(&a.stderr));
    let b = Command::new(env!("CARGO_BIN_EXE_pdf2kb"))
        .arg(input)
        .arg(&supervised)
        .args(settings)
        .args(["--timeout-seconds", "30", "--verbose"])
        .output()
        .unwrap();
    assert!(b.status.success(), "{}", String::from_utf8_lossy(&b.stderr));
    assert_eq!(
        std::fs::read(&plain).unwrap(),
        std::fs::read(&supervised).unwrap()
    );
    let stderr = String::from_utf8_lossy(&b.stderr);
    assert!(stderr.contains("Starting PDF worker"));
    assert!(stderr.contains("output validated"));
    let stdout = String::from_utf8_lossy(&b.stdout);
    assert!(stdout.contains("Raw preview:"));
    assert!(stdout.contains("Cleaned preview:"));
    assert_eq!(stdout.matches("Saved ").count(), 1);
    assert!(!stdout.contains("airust-pdf-worker-"));
}

#[cfg(feature = "cli")]
#[test]
fn supervised_failures_never_replace_old_output() {
    use std::process::Command;
    let folder = tempfile::tempdir().unwrap();
    let output = folder.path().join("saved.json");
    std::fs::write(&output, b"old owner data").unwrap();
    for fixture in [
        "blank.pdf",
        "broken.pdf",
        "encrypted.pdf",
        "invalid-header-utf8.pdf",
    ] {
        let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/pdf")
            .join(fixture);
        let result = Command::new(env!("CARGO_BIN_EXE_pdf2kb"))
            .arg(input)
            .arg(&output)
            .args(["--timeout-seconds", "30", "--overwrite"])
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&result.stderr).contains("worker failed"));
        assert_eq!(std::fs::read(&output).unwrap(), b"old owner data");
    }
    let input = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/pdf/sentences.pdf"
    );
    let result = Command::new(env!("CARGO_BIN_EXE_pdf2kb"))
        .arg(input)
        .arg(&output)
        .args(["--timeout-seconds", "30", "--max-pages", "1", "--overwrite"])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&result.stderr).contains("page count"));
    assert_eq!(std::fs::read(&output).unwrap(), b"old owner data");
}

#[cfg(feature = "cli")]
#[test]
fn supervised_settings_reject_invalid_limits_before_work() {
    use std::process::Command;
    let folder = tempfile::tempdir().unwrap();
    let output = folder.path().join("saved.json");
    std::fs::write(&output, b"old owner data").unwrap();
    let input = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/pdf/sentences.pdf"
    );
    for flags in [
        vec!["--timeout-seconds", "0"],
        vec!["--timeout-seconds", "18446744073709551615"],
        vec!["--max-result-mib", "1"],
        vec!["--timeout-seconds", "30", "--max-result-mib", "0"],
        vec![
            "--timeout-seconds",
            "30",
            "--max-result-mib",
            "18446744073709551615",
        ],
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_pdf2kb"))
            .arg(input)
            .arg(&output)
            .arg("--overwrite")
            .args(flags)
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(1));
        assert_eq!(std::fs::read(&output).unwrap(), b"old owner data");
        assert!(!String::from_utf8_lossy(&result.stderr).contains("Starting PDF worker"));
    }
}

#[cfg(feature = "cli")]
#[test]
fn supervised_parent_rejects_oversized_result_before_publication() {
    let folder = tempfile::tempdir().unwrap();
    let input = folder.path().join("many.pdf");
    let output = folder.path().join("old.json");
    std::fs::write(&input, pdf_fixture::repeated_pages(200)).unwrap();
    std::fs::write(&output, b"keep old result").unwrap();
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_pdf2kb"))
        .arg(&input)
        .arg(&output)
        .args([
            "--timeout-seconds",
            "30",
            "--max-result-mib",
            "1",
            "--overwrite",
        ])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&result.stderr).contains("result byte budget"));
    assert_eq!(std::fs::read(&output).unwrap(), b"keep old result");
}

#[cfg(feature = "cli")]
#[test]
fn supervised_output_created_during_work_keeps_its_owner() {
    use std::{
        io::{BufRead, BufReader},
        process::{Child, Command, Stdio},
        sync::mpsc,
        time::{Duration, Instant},
    };
    struct Guard(Child);
    impl Drop for Guard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let folder = tempfile::tempdir().unwrap();
    let input = folder.path().join("many.pdf");
    let output = folder.path().join("shared.json");
    std::fs::write(&input, pdf_fixture::repeated_pages(500)).unwrap();
    let mut child = Guard(
        Command::new(env!("CARGO_BIN_EXE_pdf2kb"))
            .arg(&input)
            .arg(&output)
            .args(["--timeout-seconds", "30"])
            .stderr(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let stderr = child.0.stderr.take().unwrap();
    let (tx, rx) = mpsc::channel();
    let reader = std::thread::spawn(move || {
        for line in BufReader::new(stderr).lines() {
            if line.unwrap().starts_with("Starting PDF worker") {
                let _ = tx.send(());
            }
        }
    });
    rx.recv_timeout(Duration::from_secs(30))
        .expect("worker start notification");
    std::fs::write(&output, b"concurrent owner data").unwrap();
    let deadline = Instant::now() + Duration::from_secs(35);
    let status = loop {
        if let Some(s) = child.0.try_wait().unwrap() {
            break s;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(10));
    };
    reader.join().unwrap();
    assert_eq!(status.code(), Some(1));
    assert_eq!(std::fs::read(&output).unwrap(), b"concurrent owner data");
}

#[cfg(all(feature = "cli", unix))]
#[test]
fn supervised_cli_handles_platform_path_bytes_without_panicking() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};
    let folder = tempfile::tempdir().unwrap();
    let input = folder
        .path()
        .join(OsString::from_vec(b"source-\xff.pdf".to_vec()));
    let output = folder
        .path()
        .join(OsString::from_vec(b"output-\xfe.json".to_vec()));
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/pdf/sentences.pdf"
    ))
    .unwrap();
    std::fs::write(folder.path().join("control.pdf"), &bytes).unwrap();
    if let Err(error) = std::fs::write(&input, &bytes) {
        eprintln!("Filesystem cannot create a raw-byte path fixture: {error}");
        let rejected = std::process::Command::new(env!("CARGO_BIN_EXE_pdf2kb"))
            .args(["--timeout-seconds", "30", "--"])
            .arg(&input)
            .arg(&output)
            .output()
            .unwrap();
        assert_eq!(rejected.status.code(), Some(1));
        assert!(!output.exists());
        return;
    }
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_pdf2kb"))
        .args(["--timeout-seconds", "30", "--"])
        .arg(input)
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(output.is_file());
}

#[cfg(all(feature = "cli", unix))]
#[test]
fn closed_console_streams_do_not_panic_or_change_file_outcomes() {
    use std::{
        os::{fd::OwnedFd, unix::net::UnixStream},
        process::{Command, Stdio},
    };
    fn closed_stream() -> Stdio {
        let (reader, writer) = UnixStream::pair().unwrap();
        drop(reader);
        let fd: OwnedFd = writer.into();
        Stdio::from(fd)
    }
    let folder = tempfile::tempdir().unwrap();
    let output = folder.path().join("knowledge.json");
    let converted = Command::new(env!("CARGO_BIN_EXE_pdf2kb"))
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/pdf/sentences.pdf"
        ))
        .arg(&output)
        .args(["--timeout-seconds", "30", "--verbose"])
        .stdout(closed_stream())
        .stderr(closed_stream())
        .status()
        .unwrap();
    assert!(converted.success());
    let original = std::fs::read(&output).unwrap();
    let examples: Vec<airust::agent::TrainingExample> = serde_json::from_slice(&original).unwrap();
    assert!(!examples.is_empty());
    let help = Command::new(env!("CARGO_BIN_EXE_pdf2kb"))
        .arg("--help")
        .stdout(closed_stream())
        .stderr(closed_stream())
        .status()
        .unwrap();
    assert!(help.success());
    let failed = Command::new(env!("CARGO_BIN_EXE_pdf2kb"))
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/pdf/broken.pdf"
        ))
        .arg(&output)
        .args(["--timeout-seconds", "30", "--overwrite"])
        .stdout(closed_stream())
        .stderr(closed_stream())
        .status()
        .unwrap();
    assert_eq!(failed.code(), Some(1));
    assert_eq!(std::fs::read(&output).unwrap(), original);
}
