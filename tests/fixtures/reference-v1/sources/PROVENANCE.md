# Optional source references

Enable `provenance` for validated source locations and a separate `ProvenanceStore`. It adds no dependency and enables neither models nor graphs, HTTP or databases. Existing knowledge entries and their JSON format remain unchanged.

```rust
# #[cfg(feature = "provenance")]
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use airust::provenance::{ProvenanceStore, SourceReference, TextSpan};
let reference = SourceReference::new("manual-p3-passage-1")?
    .with_document("manual-v2")?
    .with_page(3)?
    .with_text_span(TextSpan::new(0, 5)?);
reference.validate_source_text("Rust works.")?;
let mut sources = ProvenanceStore::new();
sources.add("note-1", reference.clone())?;
sources.add("note-2", reference)?;
assert_eq!(sources.source_count(), 1);
assert_eq!(sources.entry_count(), 2);
sources.merge_entries("combined", ["note-1", "note-2"])?;
assert_eq!(sources.sources_for("combined").count(), 1);
let restored = ProvenanceStore::from_json(&sources.to_json()?)?;
assert_eq!(restored, sources);
# Ok(())
# }
# #[cfg(not(feature = "provenance"))]
# fn main() {}
```

## Location contract

`SourceReference::new(id)` records a stable reference ID supplied by the caller. A reference ID identifies one definition; change the ID when its document revision or location changes. Builders add optional `with_document`, `with_page`, `with_text_span`, `with_file`, `with_code`, `with_revision`, `with_uri`, and `with_metadata`. Getters expose each field. Missing values remain `None`; a reference ID alone does not invent a document, page, file, or position.

Identity strings must be nonempty and contain no control characters. Page numbers start at one. `TextSpan::new(start_byte, end_byte)` uses zero-based UTF-8 bytes with an exclusive end. Empty spans are allowed as positions and provide no quoted supporting text. `validate_in(&str)` checks bounds and character boundaries against exactly the caller-supplied source text.

`CodePosition::new(line, byte_column)` uses one-based lines and **one-based UTF-8 byte columns**, not character or UTF-16 columns. `CodeSpan::new(start, end)` has an exclusive end and requires lexicographic order. Its `validate_in` checks both positions in supplied text, treating LF and CRLF as line endings and permitting the end position after the final character. `with_code(path, span)` supplies the file path and span together. No file is opened automatically.

`validate_source_text` checks any declared byte/code positions against exact supplied bytes. Without source text, constructors and deserialization validate structure only; they cannot check a PDF's true page count, a file's current revision, or whether an excerpt supports a claim. URI and revision strings are opaque evidence, not fetched or cryptographically verified. Applications choose safe link protocols, redact private paths/metadata and evaluate source reliability. A source reference does not establish truth.

## Shared storage and merging

The store interns a reference once by reference ID and keeps sorted ID sets per entry. `add(entry_id, reference)` returns whether that binding is new. The same definition can support many entries without repeated source metadata. A conflicting definition under the same ID is an error. `replace` validates the entire replacement before changing bindings. `merge_entries(target, source_ids)` unions existing source bindings without altering knowledge text or removing the original entry bindings; missing source entries contribute no invented reference. `merge(&other_store)` validates all conflicts before mutation.

For repeated references, register a definition once with `insert_source(reference)`, then call `bind(entry_id, reference_id)` without repeatedly cloning source metadata. Binding an unknown reference is an error. Registered definitions can remain unbound until they are attached or pruned.

`source(id)`, `sources()`, and `sources_for(entry_id)` borrow definitions. Iteration is stable by source ID. `status(entry_id)` is `Missing` or `Referenced`, not a truth or review status. `remove_entry` removes its bindings; unreferenced definitions remain until `prune_unreferenced`. `remove_source` removes all its bindings and any newly empty entry sets. `clear` releases the store. `is_empty` reports no bound entries; `source_count` can still be positive until pruning.

This sidecar does not monitor changes to an index or database. When knowledge text changes, the caller must replace or invalidate its source bindings in the same application update pipeline. Matching an entry ID alone does not prove that an old reference still supports the current text. Preserve exact source revisions and validate source bytes when freshness matters.

## Serialization

`to_json` and `from_json` use format version 1 with separate reference and entry-binding lists. They reject unknown format versions, invalid locations, conflicting/duplicate IDs, duplicate bindings and dangling references. Empty imported binding lists normalize to missing provenance. Generic `Serialize`/`Deserialize` use the same schema validation. `from_json` additionally caps input at 16 MiB; `from_json_with_limit` accepts an explicit caller limit. A generic deserializer's input/resource policy belongs to its caller.

## Context assembly

When `context-assembly` is also enabled, attach `ContextAssembler::with_provenance(&store)`. Each selected `ContextReference` exposes borrowed `sources()` and `provenance_status()`. Missing sources are an explicit empty list with `Missing` status. Normalized duplicate knowledge entries keep their separate source-reference lists. `ContextOptions::with_source_references(false)` returns empty lists with `Omitted` status, independently of whether hidden references exist. Entry metadata controls remain separate from source metadata.

Context excerpt offsets still refer to the first knowledge entry's text. External source spans refer to the caller's original artifact and may use a different coordinate system. AIRust never adds excerpt offsets to external citation offsets or guesses a page mapping. Source references and their metadata are outside the context **text-byte** budget. The application must include them when measuring its complete serialized prompt.
