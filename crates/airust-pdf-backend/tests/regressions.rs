use airust_pdf_backend::{object::ObjectRef, PdfDocument};
use std::collections::BTreeMap;

mod parser {
    pub use airust_pdf_backend::parser::parse_object;
}
mod object {
    pub use airust_pdf_backend::object::Object;
}
#[path = "../src/airust_small_object.rs"]
mod small;

fn document(bodies: Vec<Vec<u8>>) -> Vec<u8> {
    let mut pdf = b"%PDF-1.4\n".to_vec();
    let mut offsets = vec![0];
    for (index, body) in bodies.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.extend_from_slice(format!("{} 0 obj", index + 1).as_bytes());
        pdf.extend_from_slice(body);
        pdf.extend_from_slice(b"endobj ");
    }
    pdf.push(b'\n');
    let xref = pdf.len();
    pdf.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", offsets.len()).as_bytes());
    for offset in &offsets[1..] {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            offsets.len()
        )
        .as_bytes(),
    );
    pdf
}

fn tagged(count: usize, payload_size: usize) -> Vec<u8> {
    let mut content = Vec::new();
    for i in (0..count).rev() {
        content.extend_from_slice(
            format!(
                "/P <</MCID {i}>> BDC BT /F1 12 Tf 40 {} Td (Word{i}) Tj ET EMC\n",
                740 - i as i32 * 12
            )
            .as_bytes(),
        );
    }
    let mut bodies = vec![
        b"<</Type/Catalog/Pages 2 0 R/StructTreeRoot 6 0 R/MarkInfo<</Marked true>>>>".to_vec(),
        b"<</Type/Pages/Count 1/Kids[3 0 R]>>".to_vec(),
        b"<</Type/Page/Parent 2 0 R/MediaBox[0 0 612 792]/Resources<</Font<</F1 4 0 R>>>>/Contents 5 0 R>>".to_vec(),
        b"<</Type/Font/Subtype/Type1/BaseFont/Helvetica>>".to_vec(),
        [format!("<</Length {}>>stream\n", content.len()).into_bytes(), content, b"endstream".to_vec()].concat(),
        b"<</Type/StructTreeRoot/K 7 0 R>>".to_vec(),
        format!("<</Type/StructElem/S/Document/P 6 0 R/K[{}]>>", (0..count).map(|i|format!("{} 0 R ",8+i)).collect::<String>()).into_bytes(),
    ];
    for i in 0..count {
        bodies.push(format!("<</Type/StructElem/S/P/P 7 0 R/Pg 3 0 R/K {i}/Value {i}/Literal(literal endobj and >> are data {})>>", "x".repeat(payload_size)).into_bytes());
    }
    document(bodies)
}

macro_rules! normalizer {
    ($name:ident, $variant:path) => {
        fn $name(value: $variant) -> serde_json::Value {
            use $variant as O;
            match value {
                O::Null => serde_json::json!(["null"]),
                O::Boolean(v) => serde_json::json!(["bool", v]),
                O::Integer(v) => serde_json::json!(["int", v]),
                O::Real(v) => serde_json::json!(["real_bits", v.to_bits()]),
                O::String(v) => serde_json::json!(["string", v]),
                O::Name(v) => serde_json::json!(["name", v]),
                O::Reference(v) => serde_json::json!(["ref", v.id, v.gen]),
                O::Array(v) => {
                    serde_json::json!(["array", v.into_iter().map($name).collect::<Vec<_>>()])
                }
                O::Dictionary(v) => serde_json::json!([
                    "dict",
                    v.into_iter()
                        .map(|(k, v)| (k, $name(v)))
                        .collect::<BTreeMap<_, _>>()
                ]),
                O::Stream { dict, data } => serde_json::json!([
                    "stream",
                    dict.into_iter()
                        .map(|(k, v)| (k, $name(v)))
                        .collect::<BTreeMap<_, _>>(),
                    data.to_vec()
                ]),
            }
        }
    };
}
normalizer!(new_value, airust_pdf_backend::object::Object);
normalizer!(old_value, pdf_oxide::object::Object);

#[test]
fn long_minified_tagged_objects_keep_values_text_and_complete_structure() {
    let bytes = tagged(256, 0);
    let old = pdf_oxide::PdfDocument::from_bytes(bytes.clone()).unwrap();
    let new = PdfDocument::from_bytes(bytes).unwrap();
    assert_eq!(new.page_count().unwrap(), old.page_count().unwrap());
    assert_eq!(new.extract_text(0).unwrap(), old.extract_text(0).unwrap());
    let tree = new.structure_tree().unwrap().unwrap();
    assert_eq!(tree.root_elements.len(), 1);
    assert_eq!(tree.root_elements[0].children.len(), 256);
    for id in 1..=263 {
        assert_eq!(
            new_value(new.load_object(ObjectRef { id, gen: 0 }).unwrap()),
            old_value(
                old.load_object(pdf_oxide::object::ObjectRef { id, gen: 0 })
                    .unwrap()
            ),
            "object{id}"
        );
    }
}

#[test]
fn large_objects_use_fallback_without_losing_literal_end_markers() {
    let bytes = tagged(4, 5000);
    let old = pdf_oxide::PdfDocument::from_bytes(bytes.clone()).unwrap();
    let new = PdfDocument::from_bytes(bytes).unwrap();
    assert_eq!(new.extract_text(0).unwrap(), old.extract_text(0).unwrap());
    for id in 8..12 {
        assert_eq!(
            new_value(new.load_object(ObjectRef { id, gen: 0 }).unwrap()),
            old_value(
                old.load_object(pdf_oxide::object::ObjectRef { id, gen: 0 })
                    .unwrap()
            )
        );
    }
}

#[test]
fn bounded_probe_accepts_complete_nonstream_types_and_whitespace() {
    for body in [
        b"42".as_slice(),
        b"-42",
        b"1.25",
        b"true",
        b"false",
        b"null",
        b"/Name",
        b"(literal endobj \\) text)",
        b"<414243>",
        b"[1 2 (endobj)]",
        b"<</K 0/ActualText(test)>>",
        b"7 0 R",
    ] {
        let bytes = [
            b"12 0 obj ".as_slice(),
            body,
            b" % endobj in comment\r\n endobj \n",
        ]
        .concat();
        let (start, end) =
            small::short_object_bounds(&bytes).expect("complete bounded nonstream object");
        let (_, parsed) = airust_pdf_backend::parser::parse_object(&bytes[start..end]).unwrap();
        let (_, expected) = airust_pdf_backend::parser::parse_object(body).unwrap();
        assert_eq!(parsed, expected);
    }
}

#[test]
fn streams_always_use_the_original_loader() {
    for length in [0, 4, 4096, 1000000] {
        let bytes = format!("1 0 obj <</Length {length}>>stream\nendstream endobj \n");
        assert!(small::short_object_bounds(bytes.as_bytes()).is_none());
    }
}

#[test]
fn incomplete_headers_and_body_or_end_markers_fall_back() {
    for bytes in [
        b"".as_slice(),
        b"1 0 obj",
        b"1 0 object <</K 0>> endobj ",
        b"1 0 obj <</K 0",
        b"1 0 obj (unfinished endobj ",
        b"1 0 obj 42 endobject ",
        b"4294967296 0 obj 42 endobj ",
        b"1 65536 obj 42 endobj ",
        b"1 0 obj 42 %comment without newline endobj ",
        b"\x001 0 obj 42 endobj ",
    ] {
        assert!(small::short_object_bounds(bytes).is_none(), "{:?}", bytes);
    }
}

#[test]
fn complete_object_beyond_probe_budget_is_not_accepted() {
    let bytes = format!("1 0 obj ({} ) endobj \n", "x".repeat(5000));
    assert!(small::short_object_bounds(bytes.as_bytes()).is_none());
}

#[test]
fn utf8_literal_data_does_not_change_boundaries() {
    let bytes = "9 0 obj (ä😀 endobj €) endobj \n".as_bytes();
    let (start, end) = small::short_object_bounds(bytes).unwrap();
    let (_, value) = airust_pdf_backend::parser::parse_object(&bytes[start..end]).unwrap();
    assert_eq!(value.as_string(), Some("ä😀 endobj €".as_bytes()));
}

#[test]
fn malformed_utf8_header_no_longer_unwinds() {
    let bytes = include_bytes!("fixtures/invalid-header-utf8.pdf").to_vec();
    let result = std::panic::catch_unwind(|| PdfDocument::from_bytes(bytes));
    assert!(result.is_ok(), "malformed header must return normally");
}

#[test]
fn deterministic_probe_mutations_do_not_unwind_or_cross_the_budget() {
    let original = b"12 0 obj <</Literal(endobj)/K [1 2 3]>> endobj \n";
    let mut state = 20261120u64;
    for _ in 0..2000 {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let mut bytes = original.to_vec();
        let i = (state as usize) % bytes.len();
        bytes[i] ^= (state >> 32) as u8;
        let r = std::panic::catch_unwind(|| small::short_object_bounds(&bytes));
        assert!(r.is_ok());
        if let Some((a, b)) = r.unwrap() {
            assert!(a <= b && b <= 4096 && b <= bytes.len());
        }
    }
}

#[test]
fn file_backed_cold_object_loads_stay_correct_with_parallel_readers() {
    let folder = tempfile::tempdir().unwrap();
    let file = folder.path().join("tagged.pdf");
    std::fs::write(&file, tagged(256, 0)).unwrap();
    let new = std::sync::Arc::new(PdfDocument::open(&file).unwrap());
    let old = pdf_oxide::PdfDocument::open(&file).unwrap();
    std::thread::scope(|scope| {
        let mut workers = Vec::new();
        for lane in 0..4 {
            let doc = new.clone();
            workers.push(scope.spawn(move || {
                for i in (lane..256).step_by(4) {
                    let object = doc
                        .load_object(ObjectRef {
                            id: 8 + i as u32,
                            gen: 0,
                        })
                        .unwrap();
                    assert_eq!(
                        object.as_dict().unwrap()["Value"].as_integer(),
                        Some(i as i64)
                    );
                }
            }));
        }
        for worker in workers {
            worker.join().unwrap();
        }
    });
    assert_eq!(new.extract_text(0).unwrap(), old.extract_text(0).unwrap());
}

fn form_stream(content: &[u8], resources: &str) -> Vec<u8> {
    [
        format!("<< /Length {} /Type /XObject /Subtype /Form /FormType 1 /BBox [0 0 400 400] /Resources << /Font << /F1 4 0 R >> {} >> >>\nstream\n", content.len(), resources).into_bytes(),
        content.to_vec(),
        b"\nendstream\n".to_vec(),
    ].concat()
}

fn ordinary_stream(content: &[u8]) -> Vec<u8> {
    [
        format!("<< /Length {} >>\nstream\n", content.len()).into_bytes(),
        content.to_vec(),
        b"\nendstream\n".to_vec(),
    ]
    .concat()
}

fn nested_forms(depth: usize) -> Vec<u8> {
    let mut bodies = vec![
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Count 1 /Kids [3 0 R] >>".to_vec(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 400] /Resources << /Font << /F1 4 0 R >> /XObject << /First 6 0 R >> >> /Contents 5 0 R >>".to_vec(),
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_vec(),
        ordinary_stream(b"BT /F1 12 Tf 20 300 Td (AIRUSTFORMSTART) Tj ET /First Do BT /F1 12 Tf 20 250 Td (AIRUSTFORMEND) Tj ET"),
    ];
    for i in 0..depth {
        let last = i + 1 == depth;
        let content = if last {
            b"BT /F1 12 Tf 20 200 Td (AIRUSTFORMLEAF) Tj ET".as_slice()
        } else {
            b"/Next Do".as_slice()
        };
        let resources = if last {
            String::new()
        } else {
            format!("/XObject << /Next {} 0 R >>", 7 + i)
        };
        bodies.push(form_stream(content, &resources));
    }
    document(bodies)
}

#[test]
fn supported_nested_forms_preserve_the_innermost_text() {
    for depth in [1, 5, 9, 10] {
        let doc = PdfDocument::from_bytes(nested_forms(depth)).unwrap();
        let text = doc.extract_text(0).unwrap();
        for marker in ["AIRUSTFORMSTART", "AIRUSTFORMEND", "AIRUSTFORMLEAF"] {
            assert!(text.contains(marker), "depth={depth} missing {marker}");
        }
    }
}

#[test]
fn excessive_form_depth_errors_without_caching_a_partial_success() {
    for depth in [11, 16, 32, 64, 128] {
        let doc = PdfDocument::from_bytes(nested_forms(depth)).unwrap();
        for _ in 0..2 {
            let error = doc.extract_text(0).unwrap_err().to_string();
            assert!(error.contains("Recursion"), "depth={depth}: {error}");
            assert!(doc.extract_spans(0).is_err(), "depth={depth}");
            assert!(doc.extract_words(0).is_err(), "depth={depth}");
            assert!(doc.extract_chars(0).is_err(), "depth={depth}");
        }
    }
}

fn assert_form_budget_result(result: airust_pdf_backend::Result<String>) {
    match result {
        Ok(text) => {
            for marker in ["AIRUSTBEFORE", "AIRUSTAFTER", "AIRUSTLAST"] {
                assert!(text.contains(marker), "missing {marker}: {text}");
            }
        }
        Err(error) => assert!(error.to_string().contains("budget"), "{error}"),
    }
}

#[test]
fn form_processing_budget_errors_instead_of_publishing_a_prefix() {
    for count in [499, 500, 501] {
        let mut calls = String::from("BT /F1 12 Tf 20 300 Td (AIRUSTBEFORE) Tj ET\n");
        let mut names = String::new();
        for i in 0..count {
            calls.push_str(&format!("/F{i} Do\n"));
            names.push_str(&format!("/F{i} {} 0 R ", 6 + i));
        }
        calls.push_str("BT /F1 12 Tf 20 250 Td (AIRUSTAFTER) Tj ET");
        let mut bodies = vec![
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Count 1 /Kids [3 0 R] >>".to_vec(),
        format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 400] /Resources << /Font << /F1 4 0 R >> /XObject << {names} >> >> /Contents 5 0 R >>").into_bytes(),
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_vec(),
        ordinary_stream(calls.as_bytes()),
    ];
        for i in 0..count {
            bodies.push(form_stream(
                if i + 1 == count {
                    b"BT /F1 12 Tf 20 200 Td (AIRUSTLAST) Tj ET"
                } else {
                    b""
                },
                "",
            ));
        }
        let doc = PdfDocument::from_bytes(document(bodies)).unwrap();
        let first = doc.extract_text(0);
        if count > 500 {
            assert!(first.unwrap_err().to_string().contains("budget"));
        } else {
            assert_form_budget_result(Ok(first.unwrap()));
        }
        for _ in 0..2 {
            assert_form_budget_result(doc.extract_text(0));
            assert_form_budget_result(
                doc.extract_spans(0)
                    .map(|spans| spans.into_iter().map(|span| span.text).collect::<String>()),
            );
            assert_form_budget_result(
                doc.extract_words(0)
                    .map(|words| words.into_iter().map(|word| word.text).collect::<String>()),
            );
            assert_form_budget_result(
                doc.extract_chars(0)
                    .map(|chars| chars.into_iter().map(|item| item.char).collect::<String>()),
            );
        }
    }
}

#[test]
fn fallback_page_count_rejects_depth_exhaustion() {
    for depth in [16, 64] {
        let leaf = 6 + depth;
        let content = leaf + 1;
        let mut bodies = vec![
            b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
            b"<< /Type /Pages /Kids [3 0 R 6 0 R] >>".to_vec(),
            b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 400] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>".to_vec(),
            b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_vec(),
            ordinary_stream(b"BT /F1 12 Tf 20 300 Td (AIRUSTSHALLOW) Tj ET"),
        ];
        for i in 0..depth {
            let id = 6 + i;
            bodies.push(
                format!(
                    "<< /Type /Pages /Parent {} 0 R /Count 1 /Kids [{} 0 R] >>",
                    if i == 0 { 2 } else { id - 1 },
                    id + 1
                )
                .into_bytes(),
            );
        }
        bodies.push(format!("<< /Type /Page /Parent {} 0 R /MediaBox [0 0 400 400] /Resources << /Font << /F1 4 0 R >> >> /Contents {content} 0 R >>", leaf-1).into_bytes());
        bodies.push(ordinary_stream(
            b"BT /F1 12 Tf 20 300 Td (AIRUSTDEEP) Tj ET",
        ));
        let doc = PdfDocument::from_bytes(document(bodies)).unwrap();
        if depth == 16 {
            assert_eq!(doc.page_count().unwrap(), 2);
        } else {
            assert!(doc.page_count().is_err());
        }
    }
}
