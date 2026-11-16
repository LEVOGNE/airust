use airust::provenance::{ProvenanceStore, SourceReference};
use serde_json::json;
use std::{error::Error, hint::black_box, time::Instant};

fn main() -> Result<(), Box<dyn Error>> {
    let count = std::env::args()
        .nth(1)
        .map(|s| s.parse::<usize>())
        .transpose()?
        .unwrap_or(1000);
    if count == 0 {
        return Err("Binding count must be positive".into());
    }
    let shared = count.min(1000);
    let source_id = |n| format!("reference-{n:09}");
    let entry_id = |n| format!("entry-{n:09}");
    let mut store = ProvenanceStore::new();
    let began = Instant::now();
    for n in 0..shared {
        store.insert_source(
            SourceReference::new(source_id(n))?
                .with_document(format!("document-{}", n / 10))?
                .with_page(n % 10 + 1)?
                .with_revision("synthetic-v1")?,
        )?;
    }
    for n in 0..count {
        store.bind(entry_id(n), &source_id(n % shared))?;
    }
    let build_ms = began.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(store.source_count(), shared);
    assert_eq!(store.entry_count(), count);
    let began = Instant::now();
    let encoded = store.to_json()?;
    let export_ms = began.elapsed().as_secs_f64() * 1000.0;
    let began = Instant::now();
    let restored = ProvenanceStore::from_json(&encoded)?;
    let import_ms = began.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(store, restored);
    let mut samples = Vec::new();
    for n in 0..1000 {
        let id = entry_id(n * 7919 % count);
        let began = Instant::now();
        let reference = store.sources_for(&id).next().expect("registered source");
        black_box(reference);
        samples.push(began.elapsed().as_secs_f64() * 1000000.0);
        assert_eq!(reference.id(), source_id((n * 7919 % count) % shared));
    }
    samples.sort_by(f64::total_cmp);
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"airust_version":airust::VERSION,"entry_bindings":count,"shared_references":shared,"build_ms":build_ms,"json_bytes":encoded.len(),"export_ms":export_ms,"import_ms":import_ms,"lookup_count":1000,"lookup_p50_us":samples[499],"lookup_p95_us":samples[949],"lookup_samples_us":samples})
        )?
    );
    Ok(())
}
