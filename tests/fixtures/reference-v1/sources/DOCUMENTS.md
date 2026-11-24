# Document revisions and reliable text positions

Enable `documents` when a text position must refer to one exact version of a document. AIRust identifies the original source bytes, your processing recipe and an immutable UTF-8 text snapshot. A passage belongs to that document revision. It cannot silently resolve against different text.

This optional feature uses existing provenance spans and SHA-256. It does not enable PDF parsing, models, HTTP, retrieval, storage or an application adapter. The default library remains unchanged.

## Create a revision

```toml
[dependencies]
airust = { version = "=0.4.0", default-features = false, features = ["documents"] }
```

```rust
use airust::documents::{DocumentLimits, DocumentSnapshot, SourceFingerprint, TextSpace};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let limits = DocumentLimits::default();
let original = "Grüße Rust".as_bytes();
let source = SourceFingerprint::from_bytes(original, &limits)?;
let snapshot = DocumentSnapshot::new(
    "notes:42", source, "plain-utf8-v1", TextSpace::Supplied,
    "Grüße Rust".into(), &limits,
)?;
let start = snapshot.text().find("Rust").unwrap();
let passage = snapshot.passage(start, start + 4)?;
assert_eq!(passage.resolve(&snapshot)?, "Rust");
snapshot.verify_source(original)?;
# Ok(()) }
```

Use a stable application-owned document ID, such as a database ID. A filename alone does not distinguish two files with the same name. Include extraction/cleanup versions and settings in the recipe. `Supplied`, `Extracted` and `Cleaned` label the supplied text; they do not run a transformation or certify how it was produced.

For large sources, `SourceFingerprint::from_reader(reader, &limits)` hashes in a fixed 64 KiB buffer. It reads at most the source limit plus one byte before rejecting oversized input. Interrupted reads are retried; other errors return no partial fingerprint. Blocking reader calls have no hard time bound. Keep the supplied source stable while preparing its text. A fingerprint does not prove that arbitrary caller-supplied text was extracted correctly from those bytes.

`revision()` is a content identity, not a timestamp or increasing counter. Document ID, original bytes/length, recipe, text space, exact text and page map all affect it. Identical complete inputs produce the same revision. Changing source bytes invalidates old positions even when the resulting text is unchanged. A different same-length text is also a different revision.

## Pages and coordinate spaces

```rust
use airust::documents::{DocumentLimits, DocumentSnapshot, SourceFingerprint, TextSpace};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let limits = DocumentLimits::default();
let source = SourceFingerprint::from_bytes(b"identified source", &limits)?;
let snapshot = DocumentSnapshot::from_pages(
    "manual:17", source, "extract-v1;join=LF2", TextSpace::Extracted,
    [(2, "Rust Rust".into()), (5, "More text.".into())], &limits,
)?;
let page = snapshot.page_passage(2)?;
let first = page.subspan(&snapshot, 0, 4)?;
let second = page.subspan(&snapshot, 5, 9)?;
assert_eq!(first.resolve(&snapshot)?, second.resolve(&snapshot)?);
assert_ne!(first.id(), second.id());
assert_eq!(second.page_number(), Some(2));
# Ok(()) }
```

`from_pages` preserves each supplied page's text exactly and joins pages with two LF bytes (`\n\n`). Page spans exclude the inserted separators. Physical page numbers start at one, must increase and may have gaps. Blank provided pages remain represented. Empty input creates an empty snapshot with no known pages. This does not assert that every original page was supplied.

Every `TextSpan` is a zero-based UTF-8 byte range with an exclusive end **in the complete snapshot text**. It is not page-relative, a character/grapheme index, a PDF byte offset or a graphical position. Both ends must be valid UTF-8 boundaries. Combining marks may have their own boundaries. Empty ranges are valid cursor positions; empty/whitespace passages cannot become searchable entries.

A passage gets a physical page only if its entire range lies in a known page span. A range crossing pages or selecting an inserted separator has no guessed page. An empty cursor at a page's end belongs to that page; an empty cursor at the next page's start belongs to the next. `pages()` exposes exact page spans, and `page_passage(number)` uses physical numbering, not printed page labels.

`subspan(snapshot, start, end)` accepts offsets relative to a parent passage and returns a new checked global passage. It validates the parent revision and both UTF-8 boundaries before projecting. Repeated equal text has different IDs when its ranges differ. No first-occurrence text search is used to invent a location.

## Search without losing the location

```rust
use airust::{Engine, documents::{DocumentLimits, DocumentSnapshot, SourceFingerprint, TextSpace}};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let limits = DocumentLimits::default();
let source = SourceFingerprint::from_bytes(b"source", &limits)?;
let snapshot = DocumentSnapshot::new("manual:9", source, "v1", TextSpace::Supplied,
    "Background. Rust ownership protects memory.".into(), &limits)?;
let parent = snapshot.passage(0, snapshot.text().len())?;
let engine = Engine::from_entries([parent.to_entry(&snapshot)?])?;
let hit = &engine.search("Rust ownership", 1)[0];
parent.validate_entry(&snapshot, hit.entry)?;
let start = hit.entry.text.find("Rust ownership").unwrap();
let exact = parent.subspan(&snapshot, start, start + "Rust ownership".len())?;
let reference = exact.source_reference()?;
assert_eq!(reference.revision(), Some(snapshot.revision()));
assert_eq!(exact.resolve(&snapshot)?, "Rust ownership");
# Ok(()) }
```

`to_entry` copies only the selected passage and adds an `airust_document` descriptor to metadata. Its deterministic ID includes the document revision and range. Retain the typed passage alongside the corresponding snapshot. `validate_entry` checks ID, exact text and the descriptor before using entry-relative offsets. Additional application metadata is allowed. Editing an entry's text under its old ID fails this check.

With `retrieval`, validate the entry and pass `hit.span.start` / `hit.span.end` to `subspan`. These refer to the original entry text; displayed snippets may have separate truncation marks. `source_reference()` reuses existing provenance with the document ID, revision, optional physical page and global snapshot span. Its metadata explicitly says `snapshot_utf8_bytes`. `SourceReference::validate_source_text` checks bounds only; use `Passage::resolve` for revision validation.

With `snapshots`, prepare `DatasetRecord::new(entry).with_sources(...)` and use the existing `DatasetManager` prepare/publish protocol. Keep document snapshots indexed by revision until no retained reader needs them. The application coordinates its own source/snapshot sidecar lifetime and database transaction; AIRust does not introduce a second authoritative store. Tests cover complete replacement, stale-publication rejection and retained readers.

## Save and restore checked data

`DocumentSnapshot::to_json(max_bytes)` and `Passage::to_json(max_bytes)` use a bounded writer. The limit includes JSON escape expansion. Failure returns no partial JSON value and changes no existing snapshot. Persist the complete result with your existing atomic file or database transaction.

`DocumentSnapshot::from_json(json, &limits)` checks the encoded input limit, version, allowed fields, source descriptor, decoded text/page limits, UTF-8 page partition, text hash and complete revision. Unescaped text can be borrowed during parsing so over-budget text is rejected before copying it into the snapshot. JSON decoding and page arrays still allocate within the encoded-input bound; these are not OS RAM limits.

Restore passages with `snapshot.passage_from_json(json, &limits)`. It checks document ID, revision, text space, range and derived page. Old/foreign revisions, corrupted data, duplicate or unknown fields and unknown format versions fail. Limits are validated as positive. Passage import limits apply to its input, not retroactively to an already constructed snapshot.

Snapshot clones share immutable text, page maps and identity. Passage clones share only identity and do not keep the document text alive. Debug output summarizes a snapshot without printing its full text.

| Default budget | Value |
| --- | ---: |
| Original source bytes | 128 MiB |
| Snapshot UTF-8 bytes, including inserted separators | 256 MiB |
| Provided page count and greatest physical page number | 50,000 |
| Document ID UTF-8 bytes | 1,024 |
| Recipe UTF-8 bytes | 16,384 |
| Encoded JSON input | 512 MiB |

Configure public `DocumentLimits` fields using `..Default::default()`. Export has its own explicit byte argument. Budgets cannot preempt a blocking reader/native parser or guarantee allocator/process memory. The document feature performs no native calls itself. Existing PDF workers remain available for bounded native conversion.

## Version-one interoperability

`DOCUMENT_FORMAT_VERSION` is 1. JSON stores explicit `format_version`, document ID, source `{sha256, byte_len}`, recipe, text space, text SHA-256, revision, exact text and page spans. Unknown versions are rejected; old knowledge formats are not rewritten.

Source and text hashes are standard lowercase SHA-256 of their exact bytes. A hash frame is an unsigned 64-bit little-endian byte length followed by those bytes. The revision hashes these frames in order:

1. ASCII `airust/document/v1`.
2. UTF-8 document ID.
3. ASCII source SHA-256 hex.
4. Source byte length encoded as eight little-endian bytes.
5. UTF-8 recipe.
6. ASCII text-space tag: `supplied`, `extracted` or `cleaned`.
7. ASCII text SHA-256 hex.
8. Provided-page count as eight little-endian bytes.
9. For each page, three separate eight-byte frames: physical number, start and end.

Passage IDs prefix `airust-passage-v1:` to a framed SHA-256 of ASCII `airust/passage/v1`, document ID, revision, eight-byte start and eight-byte end. The independently calculated [synthetic fixtures](../tests/fixtures/documents/) pin this contract.

Hashes detect inconsistency and identify supplied data. They are not signatures, access controls or proof of extraction coverage. After restoring data from an untrusted source, compare its source fingerprint with independently trusted original bytes. Even matching originals do not authenticate an arbitrary recipe or supplied text.

## Upgrade and boundaries

Core 0.3.15 APIs and JSON knowledge remain usable. Cargo treats 0.4 as a new compatibility range, so opt in with an explicit dependency update. No mandatory migration, reindexing or knowledge deletion occurs. Old records without a verified snapshot cannot acquire trustworthy positions from a filename or an arbitrary first matching phrase. Build new revisioned passages from an identified exact text and retain the old export.

The PDF parser and cleaning policy remain 0.3.15 behavior, using backend 0.3.5. PDF support is optional and its coverage/OCR/reading-order limits remain. The new document contract identifies text; it does not improve glyph decoding or recover arbitrary damaged content. See [API cookbook](API-COOKBOOK.md), [0.4.0 validation](CHANGES-0.4.0.md) and [publication reproduction](validation-0.4.0/README.md). No DenkMaps or MCP integration is included.
