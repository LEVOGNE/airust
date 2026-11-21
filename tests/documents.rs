#![cfg(feature = "documents")]

use airust::{
    documents::{DocumentError, DocumentLimits, DocumentSnapshot, SourceFingerprint, TextSpace},
    Engine,
};
use serde_json::{json, Value};
use std::io::{self, Read};

fn snapshot(id: &str, source: &[u8], recipe: &str, text: &str) -> DocumentSnapshot {
    let limits = DocumentLimits::default();
    DocumentSnapshot::new(
        id,
        SourceFingerprint::from_bytes(source, &limits).unwrap(),
        recipe,
        TextSpace::Supplied,
        text.into(),
        &limits,
    )
    .unwrap()
}
fn pages() -> DocumentSnapshot {
    let limits = DocumentLimits::default();
    DocumentSnapshot::from_pages(
        "manual",
        SourceFingerprint::from_bytes(b"source", &limits).unwrap(),
        "extract-v1",
        TextSpace::Extracted,
        [
            (1, "Grüße 🌍".into()),
            (3, "Rust Rust".into()),
            (5, String::new()),
        ],
        &limits,
    )
    .unwrap()
}
fn wire(snapshot: &DocumentSnapshot) -> Value {
    serde_json::from_str(&snapshot.to_json(65536).unwrap()).unwrap()
}

#[test]
fn standard_source_sha256_and_bounded_streaming_agree() {
    let limits = DocumentLimits::default();
    let bytes = SourceFingerprint::from_bytes(b"abc", &limits).unwrap();
    assert_eq!(
        bytes.sha256(),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        bytes,
        SourceFingerprint::from_reader(io::Cursor::new(b"abc"), &limits).unwrap()
    );
    assert_eq!(bytes.byte_len(), 3);
    assert_eq!(
        SourceFingerprint::from_bytes(b"", &limits)
            .unwrap()
            .sha256(),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    let limits = DocumentLimits {
        max_source_bytes: 3,
        ..limits
    };
    assert!(SourceFingerprint::from_bytes(b"abcd", &limits).is_err());
    let mut cursor = io::Cursor::new(b"abcdefgh");
    assert!(SourceFingerprint::from_reader(&mut cursor, &limits).is_err());
    assert_eq!(cursor.position(), 4);
    assert!(SourceFingerprint::from_reader(io::Cursor::new(b"abc"), &limits).is_ok());
}

#[test]
fn interrupted_partial_and_failed_readers_never_return_partial_fingerprints() {
    struct Partial {
        interrupted: bool,
        inner: io::Cursor<Vec<u8>>,
    }
    impl Read for Partial {
        fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
            if !self.interrupted {
                self.interrupted = true;
                return Err(io::ErrorKind::Interrupted.into());
            }
            let size = out.len().min(7);
            self.inner.read(&mut out[..size])
        }
    }
    let limits = DocumentLimits::default();
    let data = "a😀e\u{301}".repeat(20000).into_bytes();
    let streamed = SourceFingerprint::from_reader(
        Partial {
            interrupted: false,
            inner: io::Cursor::new(data.clone()),
        },
        &limits,
    )
    .unwrap();
    assert_eq!(
        streamed,
        SourceFingerprint::from_bytes(&data, &limits).unwrap()
    );
    struct Failure;
    impl Read for Failure {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("reader failed"))
        }
    }
    assert!(matches!(
        SourceFingerprint::from_reader(Failure, &limits),
        Err(DocumentError::Io(_))
    ));
}

#[test]
fn same_basename_distinct_ids_keep_distinct_entries_and_references() {
    let a = snapshot("a/manual.pdf", b"source", "raw", "Rust");
    let b = snapshot("b/manual.pdf", b"source", "raw", "Rust");
    let pa = a.passage(0, 4).unwrap();
    let pb = b.passage(0, 4).unwrap();
    assert_ne!(a.revision(), b.revision());
    assert_ne!(pa.id(), pb.id());
    assert!(matches!(pa.resolve(&b), Err(DocumentError::StalePassage)));
    let engine =
        Engine::from_entries([pa.to_entry(&a).unwrap(), pb.to_entry(&b).unwrap()]).unwrap();
    assert_eq!(engine.search("Rust", 10).len(), 2);
    assert_ne!(
        pa.source_reference().unwrap().id(),
        pb.source_reference().unwrap().id()
    );
}

#[test]
fn source_recipe_text_and_coordinate_changes_each_invalidate_old_positions() {
    let old = snapshot("manual", b"one", "raw", "Rust");
    let passage = old.passage(0, 4).unwrap();
    for new in [
        snapshot("manual", b"two", "raw", "Rust"),
        snapshot("manual", b"one", "other", "Rust"),
        snapshot("manual", b"one", "raw", "Java"),
    ] {
        assert_ne!(old.revision(), new.revision());
        assert!(matches!(
            passage.resolve(&new),
            Err(DocumentError::StalePassage)
        ));
    }
    let limits = DocumentLimits::default();
    let clean = DocumentSnapshot::new(
        "manual",
        old.source().clone(),
        "raw",
        TextSpace::Cleaned,
        "Rust".into(),
        &limits,
    )
    .unwrap();
    assert!(matches!(
        passage.resolve(&clean),
        Err(DocumentError::StalePassage)
    ));
    assert!(old.verify_source(b"one").is_ok());
    assert!(matches!(
        old.verify_source(b"two"),
        Err(DocumentError::SourceMismatch)
    ));
    assert!(old.verify_source(b"one longer").is_err());
    assert_eq!(passage.resolve(&old).unwrap(), "Rust");
}

#[test]
fn framing_and_page_labels_are_part_of_revision_identity() {
    assert_ne!(
        snapshot("ab", b"s", "c", "text").revision(),
        snapshot("a", b"s", "bc", "text").revision()
    );
    let limits = DocumentLimits::default();
    let source = SourceFingerprint::from_bytes(b"s", &limits).unwrap();
    let a = DocumentSnapshot::from_pages(
        "id",
        source.clone(),
        "r",
        TextSpace::Extracted,
        [(1, "same".into())],
        &limits,
    )
    .unwrap();
    let b = DocumentSnapshot::from_pages(
        "id",
        source.clone(),
        "r",
        TextSpace::Extracted,
        [(2, "same".into())],
        &limits,
    )
    .unwrap();
    let c = DocumentSnapshot::new(
        "id",
        source,
        "r",
        TextSpace::Extracted,
        "same".into(),
        &limits,
    )
    .unwrap();
    assert_eq!(a.text(), b.text());
    assert_ne!(a.revision(), b.revision());
    assert_ne!(a.revision(), c.revision());
}

#[test]
fn independent_python_format_v1_fixture_matches_runtime_revision_and_passage_id() {
    let limits = DocumentLimits::default();
    let fixed =
        DocumentSnapshot::from_json(include_str!("fixtures/documents/snapshot-v1.json"), &limits)
            .unwrap();
    let expected: Value =
        serde_json::from_str(include_str!("fixtures/documents/identity-v1.json")).unwrap();
    assert_eq!(fixed.revision(), expected["revision"].as_str().unwrap());
    fixed.verify_source(b"source:001").unwrap();
    let passage = fixed
        .page_passage(7)
        .unwrap()
        .subspan(&fixed, 0, 4)
        .unwrap();
    assert_eq!(passage.id(), expected["first_rust_id"]);
    assert_eq!(passage.resolve(&fixed).unwrap(), "Rust");
}

#[test]
fn every_unicode_boundary_slices_exact_snapshot_and_split_characters_fail() {
    for text in [
        "",
        "a",
        "Grüße 世界🙂",
        "e\u{301} coﬅ 👩‍💻 🇩🇪",
        "عَرَبِيّ\r\nRust\0tail",
    ] {
        let s = snapshot("unicode", b"s", "exact", text);
        for start in 0..=text.len() + 1 {
            for end in 0..=text.len() + 1 {
                match text.get(start..end) {
                    Some(expected) => assert_eq!(
                        s.passage(start, end).unwrap().resolve(&s).unwrap(),
                        expected
                    ),
                    None => assert!(s.passage(start, end).is_err()),
                }
            }
        }
        assert!(s.passage(usize::MAX, usize::MAX).is_err());
        assert!(s.passage(1, 0).is_err());
    }
}

#[test]
fn repeated_text_keeps_each_occurrence_and_parent_relative_offsets() {
    let s = snapshot("manual", b"s", "r", "🌍 Rust Rust");
    let base = "🌍 ".len();
    let a = s.passage(base, base + 4).unwrap();
    let b = s.passage(base + 5, base + 9).unwrap();
    assert_eq!(a.resolve(&s).unwrap(), b.resolve(&s).unwrap());
    assert_ne!(a.id(), b.id());
    let outer = s.passage(base, s.text().len()).unwrap();
    assert_eq!(outer.subspan(&s, 5, 9).unwrap(), b);
    assert!(outer.subspan(&s, 0, 10).is_err());
    assert!(outer.subspan(&s, usize::MAX, usize::MAX).is_err());
    assert!(s
        .passage(0, s.text().len())
        .unwrap()
        .subspan(&s, 1, 4)
        .is_err());
}

#[test]
fn pages_preserve_physical_gaps_blank_pages_and_explicit_separator_offsets() {
    let s = pages();
    assert_eq!(s.text(), "Grüße 🌍\n\nRust Rust\n\n");
    assert_eq!(
        s.pages()
            .iter()
            .map(|p| p.page_number())
            .collect::<Vec<_>>(),
        [1, 3, 5]
    );
    for p in s.pages() {
        let passage = s.page_passage(p.page_number()).unwrap();
        assert_eq!(passage.span(), p.span());
        assert_eq!(passage.page_number(), Some(p.page_number()));
    }
    assert_eq!(s.page_passage(5).unwrap().resolve(&s).unwrap(), "");
    assert!(matches!(
        s.page_passage(2),
        Err(DocumentError::UnknownPage(2))
    ));
    let end = s.pages()[0].span().end_byte();
    assert_eq!(s.passage(0, end).unwrap().page_number(), Some(1));
    assert_eq!(s.passage(end, end).unwrap().page_number(), Some(1));
    assert_eq!(s.passage(end, end + 1).unwrap().page_number(), None);
    assert_eq!(s.passage(0, s.text().len()).unwrap().page_number(), None);
    assert_eq!(s.passage(end + 2, end + 6).unwrap().page_number(), Some(3));
}

#[test]
fn invalid_order_and_limits_reject_before_a_candidate_is_published() {
    let old = pages();
    let retained = old.clone();
    let limits = DocumentLimits {
        max_text_bytes: 4,
        max_pages: 3,
        ..Default::default()
    };
    for data in [
        vec![(0, "x".into())],
        vec![(2, "x".into()), (1, "y".into())],
        vec![(1, "x".into()), (1, "x".into())],
        vec![(4, "x".into())],
        vec![(1, "four".into()), (2, "x".into())],
    ] {
        assert!(DocumentSnapshot::from_pages(
            "id",
            old.source().clone(),
            "r",
            TextSpace::Extracted,
            data,
            &limits
        )
        .is_err());
    }
    assert_eq!(old.text(), retained.text());
    assert!(std::ptr::eq(old.text().as_ptr(), retained.text().as_ptr()));
    let exact = DocumentSnapshot::new(
        "id",
        old.source().clone(),
        "r",
        TextSpace::Supplied,
        "four".into(),
        &limits,
    )
    .unwrap();
    assert_eq!(exact.text().len(), 4);
    assert!(DocumentSnapshot::new(
        "id",
        old.source().clone(),
        "r",
        TextSpace::Supplied,
        "five!".into(),
        &limits
    )
    .is_err());
}

#[test]
fn zero_limits_invalid_identities_and_exact_identity_limits_are_checked() {
    for field in 0..6 {
        let mut l = DocumentLimits::default();
        match field {
            0 => l.max_source_bytes = 0,
            1 => l.max_text_bytes = 0,
            2 => l.max_pages = 0,
            3 => l.max_id_bytes = 0,
            4 => l.max_recipe_bytes = 0,
            _ => l.max_json_bytes = 0,
        }
        assert!(matches!(
            SourceFingerprint::from_bytes(b"x", &l),
            Err(DocumentError::Configuration)
        ));
        assert!(DocumentSnapshot::from_json("{}", &l).is_err());
    }
    let l = DocumentLimits {
        max_id_bytes: 2,
        max_recipe_bytes: 2,
        ..Default::default()
    };
    let source = SourceFingerprint::from_bytes(b"x", &l).unwrap();
    for id in ["", " ", "a\n", "x\0", "abc"] {
        assert!(DocumentSnapshot::new(
            id,
            source.clone(),
            "r",
            TextSpace::Supplied,
            "x".into(),
            &l
        )
        .is_err());
    }
    for recipe in ["", "\t", "x\r", "abc"] {
        assert!(DocumentSnapshot::new(
            "id",
            source.clone(),
            recipe,
            TextSpace::Supplied,
            "x".into(),
            &l
        )
        .is_err());
    }
    assert!(DocumentSnapshot::new("ä", source, "rr", TextSpace::Supplied, "x".into(), &l).is_ok());
}

#[test]
fn snapshot_and_passage_json_roundtrip_revalidate_identity_ranges_and_pages() {
    let s = pages();
    let json = s.to_json(65536).unwrap();
    let restored = DocumentSnapshot::from_json(&json, &DocumentLimits::default()).unwrap();
    assert_eq!(s.revision(), restored.revision());
    assert_eq!(s.text(), restored.text());
    assert_eq!(s.pages(), restored.pages());
    for page in s.pages() {
        let p = s.page_passage(page.page_number()).unwrap();
        let encoded = p.to_json(4096).unwrap();
        let q = restored
            .passage_from_json(&encoded, &DocumentLimits::default())
            .unwrap();
        assert_eq!(p, q);
        assert_eq!(p.resolve(&s).unwrap(), q.resolve(&restored).unwrap());
    }
    assert!(s.to_json(0).is_err());
    assert!(s.passage(0, 0).unwrap().to_json(0).is_err());
}

#[test]
fn json_limits_include_escape_expansion_and_exact_encoded_boundaries() {
    let s = snapshot("id", b"s", "r", "\0\"\\\n🌍".repeat(20).as_str());
    let encoded = s.to_json(65536).unwrap();
    assert!(encoded.len() > s.text().len());
    assert_eq!(s.to_json(encoded.len()).unwrap(), encoded);
    assert!(matches!(
        s.to_json(encoded.len() - 1),
        Err(DocumentError::Limit(_))
    ));
    let limits = DocumentLimits {
        max_json_bytes: encoded.len(),
        ..Default::default()
    };
    assert!(DocumentSnapshot::from_json(&encoded, &limits).is_ok());
    let limits = DocumentLimits {
        max_json_bytes: encoded.len() - 1,
        ..limits
    };
    assert!(matches!(
        DocumentSnapshot::from_json(&encoded, &limits),
        Err(DocumentError::Limit(_))
    ));
    let p = s.passage(0, 0).unwrap();
    let text = p.to_json(4096).unwrap();
    assert!(p.to_json(text.len() - 1).is_err());
}

#[test]
fn corrupt_snapshot_fields_versions_and_unknown_fields_are_never_trusted() {
    let s = pages();
    let original = wire(&s);
    let changes = [
        ("format_version", json!(2)),
        ("text", json!("changed")),
        ("text_sha256", json!("0".repeat(64))),
        ("revision", json!("0".repeat(64))),
        ("document_id", json!("other")),
        ("recipe", json!("other")),
        ("text_space", json!("cleaned")),
        ("unknown", json!(true)),
    ];
    for (field, value) in changes {
        let mut bad = original.clone();
        bad[field] = value;
        assert!(
            DocumentSnapshot::from_json(&bad.to_string(), &DocumentLimits::default()).is_err(),
            "{field}"
        );
    }
    for field in ["source", "pages"] {
        let mut bad = original.clone();
        bad[field] = json!(null);
        assert!(DocumentSnapshot::from_json(&bad.to_string(), &DocumentLimits::default()).is_err());
    }
    for digest in [
        "",
        "A".repeat(64).as_str(),
        "f".repeat(63).as_str(),
        "g".repeat(64).as_str(),
    ] {
        let mut bad = original.clone();
        bad["source"]["sha256"] = json!(digest);
        assert!(DocumentSnapshot::from_json(&bad.to_string(), &DocumentLimits::default()).is_err());
    }
    let duplicate = s
        .to_json(65536)
        .unwrap()
        .replacen("{", "{\"format_version\":1,", 1);
    assert!(DocumentSnapshot::from_json(&duplicate, &DocumentLimits::default()).is_err());
}

#[test]
fn malformed_page_partitions_and_tampered_page_numbers_fail() {
    let original = wire(&pages());
    for field in ["start_byte", "end_byte"] {
        for value in [0, 1, usize::MAX] {
            let mut bad = original.clone();
            bad["pages"][1]["span"][field] = json!(value);
            assert!(
                DocumentSnapshot::from_json(&bad.to_string(), &DocumentLimits::default()).is_err()
            );
        }
    }
    for number in [0, 1, 2, usize::MAX] {
        let mut bad = original.clone();
        bad["pages"][1]["page_number"] = json!(number);
        assert!(DocumentSnapshot::from_json(&bad.to_string(), &DocumentLimits::default()).is_err());
    }
    let mut bad = original.clone();
    bad["pages"].as_array_mut().unwrap().pop();
    assert!(DocumentSnapshot::from_json(&bad.to_string(), &DocumentLimits::default()).is_err());
    let mut bad = original;
    bad["pages"][0]["extra"] = json!(true);
    assert!(DocumentSnapshot::from_json(&bad.to_string(), &DocumentLimits::default()).is_err());
}

#[test]
fn tampered_and_stale_passage_json_is_validated_against_the_actual_snapshot() {
    let s = pages();
    let p = s.page_passage(3).unwrap();
    let original: Value = serde_json::from_str(&p.to_json(4096).unwrap()).unwrap();
    for (key, value) in [
        ("document_id", json!("other")),
        ("revision", json!("0".repeat(64))),
        ("text_space", json!("cleaned")),
        ("page_number", json!(1)),
        ("format_version", json!(2)),
        ("extra", json!(0)),
    ] {
        let mut bad = original.clone();
        bad[key] = value;
        assert!(
            s.passage_from_json(&bad.to_string(), &DocumentLimits::default())
                .is_err(),
            "{key}"
        );
    }
    let mut bad = original;
    bad["span"]["end_byte"] = json!(usize::MAX);
    assert!(s
        .passage_from_json(&bad.to_string(), &DocumentLimits::default())
        .is_err());
}

#[test]
fn entries_detect_id_text_and_location_mutation_before_projecting_excerpts() {
    let s = pages();
    let p = s.page_passage(3).unwrap();
    let entry = p.to_entry(&s).unwrap();
    p.validate_entry(&s, &entry).unwrap();
    let mut bad = entry.clone();
    bad.text = "Java Java".into();
    assert!(matches!(
        p.validate_entry(&s, &bad),
        Err(DocumentError::EntryMismatch)
    ));
    let mut bad = entry.clone();
    bad.id = "renamed".into();
    assert!(p.validate_entry(&s, &bad).is_err());
    let mut bad = entry.clone();
    bad.metadata.as_mut().unwrap()["airust_document"]["page_number"] = json!(1);
    assert!(p.validate_entry(&s, &bad).is_err());
    let mut tagged = entry;
    tagged.metadata.as_mut().unwrap()["application_label"] = json!("manual");
    p.validate_entry(&s, &tagged).unwrap();
    assert!(matches!(
        s.page_passage(5).unwrap().to_entry(&s),
        Err(DocumentError::EmptyPassage)
    ));
}

#[test]
fn provenance_records_identified_snapshot_positions_and_survives_json() {
    use airust::provenance::ProvenanceStore;
    let s = pages();
    let p = s.page_passage(3).unwrap().subspan(&s, 5, 9).unwrap();
    let r = p.source_reference().unwrap();
    assert_eq!(r.document_id(), Some(s.document_id()));
    assert_eq!(r.revision(), Some(s.revision()));
    assert_eq!(r.page(), Some(3));
    assert_eq!(r.text_span(), Some(p.span()));
    assert_eq!(
        r.metadata().unwrap()["coordinate_space"],
        "snapshot_utf8_bytes"
    );
    r.validate_source_text(s.text()).unwrap();
    assert!(r.validate_source_text("Rust").is_err());
    let mut store = ProvenanceStore::new();
    store.add(p.id(), r).unwrap();
    let restored = ProvenanceStore::from_json(&store.to_json().unwrap()).unwrap();
    assert_eq!(restored.sources_for(&p.id()).count(), 1);
}

#[test]
fn shared_clones_and_threaded_readers_keep_old_revisions_unchanged() {
    let old = pages();
    let retained = old.clone();
    let p = old.page_passage(3).unwrap();
    let revision = old.revision().to_owned();
    let new = snapshot("manual", b"changed", "r", "Changed text");
    assert!(p.resolve(&new).is_err());
    std::thread::scope(|scope| {
        for _ in 0..8 {
            let s = retained.clone();
            let p = p.clone();
            scope.spawn(move || {
                for _ in 0..100 {
                    assert_eq!(p.resolve(&s).unwrap(), "Rust Rust");
                    assert_eq!(s.text().as_ptr(), old_pointer(&s));
                }
            });
        }
    });
    assert_eq!(old.revision(), revision);
    assert_eq!(retained.revision(), revision);
    assert_eq!(old.text().as_ptr(), retained.text().as_ptr());
    fn old_pointer(s: &DocumentSnapshot) -> *const u8 {
        s.clone().text().as_ptr()
    }
}

#[cfg(feature = "pdf")]
#[test]
fn cleanup_changes_require_a_different_snapshot_not_an_invented_original_mapping() {
    let raw = "e\u{301} coﬅ 🌍 Rust Rust";
    let cleaned = airust::PdfLoader::new().clean_text(raw);
    assert_ne!(raw, cleaned);
    let a = snapshot("doc", b"pdf-source", "raw", raw);
    let b = snapshot("doc", b"pdf-source", "clean-v1", &cleaned);
    let p = a.passage(0, raw.len()).unwrap();
    assert!(matches!(p.resolve(&b), Err(DocumentError::StalePassage)));
    assert_eq!(b.text(), cleaned);
    assert_eq!(a.text(), raw);
}

#[cfg(feature = "retrieval")]
#[test]
fn retrieval_original_entry_spans_project_into_the_identified_document() {
    use airust::retrieval::{ExcerptOptions, LexicalSource, RetrievalOptions, Retriever};
    let text = format!(
        "{} 🌍 Rust ownership prevents memory errors. {}",
        "Background. ".repeat(80),
        "Tail. ".repeat(40)
    );
    let s = snapshot("doc", b"source", "r", &text);
    let p = s.passage(0, text.len()).unwrap();
    let entry = p.to_entry(&s).unwrap();
    let index = Engine::from_entries([entry]).unwrap();
    let options = RetrievalOptions::new(1)
        .with_relevant_excerpts(ExcerptOptions::default())
        .with_byte_budgets(1024, 1024, 120, 16384)
        .unwrap();
    let response = Retriever::new(LexicalSource::new(&index))
        .query("Rust ownership", &options)
        .unwrap();
    let hit = &response.hits()[0];
    p.validate_entry(&s, index.get(&hit.id).unwrap()).unwrap();
    let located = p.subspan(&s, hit.span.start, hit.span.end).unwrap();
    assert_eq!(
        located.resolve(&s).unwrap(),
        &text[hit.span.start..hit.span.end]
    );
    assert!(located.resolve(&s).unwrap().contains("Rust ownership"));
}

#[cfg(feature = "snapshots")]
#[test]
fn staged_document_replacement_uses_existing_dataset_cas_and_retains_old_readers() {
    use airust::snapshots::{DatasetConfig, DatasetManager, DatasetRecord};
    let manager = DatasetManager::new("docs", DatasetConfig::default()).unwrap();
    let initial = manager.snapshot().unwrap();
    let a = snapshot("manual", b"source-a", "r", "Rust old source");
    let pa = a.passage(0, a.text().len()).unwrap();
    let record = DatasetRecord::new(pa.to_entry(&a).unwrap())
        .with_sources(vec![pa.source_reference().unwrap()]);
    let old = manager
        .publish(
            manager
                .prepare_replace(initial.revision(), vec![record], None, None)
                .unwrap(),
        )
        .unwrap();
    let b = snapshot("manual", b"source-b", "r", "Rust new source");
    let pb = b.passage(0, b.text().len()).unwrap();
    let records = || {
        vec![DatasetRecord::new(pb.to_entry(&b).unwrap())
            .with_sources(vec![pb.source_reference().unwrap()])]
    };
    let next = manager
        .prepare_replace(old.revision(), records(), None, None)
        .unwrap();
    let stale = manager
        .prepare_replace(old.revision(), records(), None, None)
        .unwrap();
    let current = manager.publish(next).unwrap();
    assert!(manager.publish(stale).is_err());
    assert!(old.lexical().get(&pa.id()).is_some());
    assert!(current.lexical().get(&pa.id()).is_none());
    assert!(current.lexical().get(&pb.id()).is_some());
    assert_eq!(pa.resolve(&a).unwrap(), "Rust old source");
    assert!(pa.resolve(&b).is_err());
}

#[test]
fn bounded_manifest_mutations_never_change_a_successful_restored_revision() {
    let s = pages();
    let original = s.to_json(65536).unwrap().into_bytes();
    let mut state = 202611210400u64;
    let mut accepted = 0;
    let mut rejected = 0;
    for i in 0..4096 {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let mut bytes = original.clone();
        let at = (state as usize) % bytes.len();
        match i % 4 {
            0 => bytes[at] ^= ((state >> 32) as u8) | 1,
            1 => bytes.truncate(at),
            2 => {
                bytes.remove(at);
            }
            _ => bytes.insert(at, b' '),
        }
        match std::str::from_utf8(&bytes)
            .ok()
            .and_then(|json| DocumentSnapshot::from_json(json, &DocumentLimits::default()).ok())
        {
            Some(restored) => {
                accepted += 1;
                assert_eq!(restored.revision(), s.revision());
                assert_eq!(restored.text(), s.text());
                assert_eq!(restored.pages(), s.pages());
            }
            None => rejected += 1,
        }
        assert_eq!(s.page_passage(3).unwrap().resolve(&s).unwrap(), "Rust Rust");
    }
    assert!(accepted > 0 && rejected > 0);
    assert_eq!(accepted + rejected, 4096);
}

#[test]
fn empty_snapshots_have_valid_empty_ranges_but_no_searchable_entries() {
    let limits = DocumentLimits::default();
    let source = SourceFingerprint::from_bytes(b"", &limits).unwrap();
    let s = DocumentSnapshot::from_pages(
        "empty",
        source,
        "none",
        TextSpace::Supplied,
        std::iter::empty(),
        &limits,
    )
    .unwrap();
    assert!(s.pages().is_empty());
    assert_eq!(s.passage(0, 0).unwrap().page_number(), None);
    assert!(s.passage(0, 0).unwrap().to_entry(&s).is_err());
    let restored = DocumentSnapshot::from_json(&s.to_json(1024).unwrap(), &limits).unwrap();
    assert_eq!(s.revision(), restored.revision());
    restored.verify_source(b"").unwrap();
}

#[test]
fn restore_applies_decoded_source_text_and_page_budgets() {
    let s = pages();
    let encoded = s.to_json(65536).unwrap();
    for limits in [
        DocumentLimits {
            max_text_bytes: 1,
            ..Default::default()
        },
        DocumentLimits {
            max_source_bytes: 1,
            ..Default::default()
        },
        DocumentLimits {
            max_pages: 1,
            ..Default::default()
        },
    ] {
        assert!(matches!(
            DocumentSnapshot::from_json(&encoded, &limits),
            Err(DocumentError::Limit(_))
        ));
    }
    let mut bad = wire(&s);
    bad["source"]["byte_len"] = json!(u64::MAX);
    assert!(matches!(
        DocumentSnapshot::from_json(&bad.to_string(), &DocumentLimits::default()),
        Err(DocumentError::Limit(_))
    ));
    let limits = DocumentLimits {
        max_pages: 3,
        ..Default::default()
    };
    assert!(DocumentSnapshot::from_pages(
        "bounded",
        s.source().clone(),
        "r",
        TextSpace::Extracted,
        (1..).map(|n| (n, String::new())),
        &limits
    )
    .is_err());
}

#[test]
fn failed_json_export_never_touches_retained_knowledge_or_locations() {
    let s = pages();
    let p = s.page_passage(3).unwrap();
    let old = s.to_json(65536).unwrap();
    for maximum in [1, 2, 32, old.len() - 1] {
        assert!(s.to_json(maximum).is_err());
        assert_eq!(p.resolve(&s).unwrap(), "Rust Rust");
    }
    assert_eq!(s.to_json(old.len()).unwrap(), old);
    assert!(!format!("{s:?}").contains("Grüße"));
}
