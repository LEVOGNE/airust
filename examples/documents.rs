use airust::documents::{DocumentLimits, DocumentSnapshot, SourceFingerprint, TextSpace};
use airust::Engine;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let limits = DocumentLimits::default();
    let source = SourceFingerprint::from_bytes(b"identified original bytes", &limits)?;
    let snapshot = DocumentSnapshot::from_pages(
        "manual:42",
        source,
        "extract-v1;clean-nfc-v1",
        TextSpace::Cleaned,
        [
            (1, "Rust keeps knowledge searchable. Grüße 🌍".into()),
            (3, "Rust positions refer to this exact snapshot.".into()),
        ],
        &limits,
    )?;
    let passage = snapshot.page_passage(3)?;
    let entry = passage.to_entry(&snapshot)?;
    let engine = Engine::from_entries([entry])?;
    let hit = &engine.search("positions", 1)[0];
    passage.validate_entry(&snapshot, hit.entry)?;
    let start = hit.entry.text.find("positions").unwrap();
    let located = passage.subspan(&snapshot, start, start + "positions".len())?;
    println!(
        "{} page {:?} bytes {}..{}: {}",
        located.revision(),
        located.page_number(),
        located.span().start_byte(),
        located.span().end_byte(),
        located.resolve(&snapshot)?
    );
    let json = snapshot.to_json(limits.max_json_bytes)?;
    let restored = DocumentSnapshot::from_json(&json, &limits)?;
    let saved_passage = located.to_json(limits.max_json_bytes)?;
    assert_eq!(
        restored
            .passage_from_json(&saved_passage, &limits)?
            .resolve(&restored)?,
        "positions"
    );
    snapshot.verify_source(b"identified original bytes")?;
    Ok(())
}
