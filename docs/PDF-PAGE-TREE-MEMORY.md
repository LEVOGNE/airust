# Compact PDF page-tree validation in 0.3.15

AIRust checks the reachable PDF page tree before extracting content. A very wide branch previously scheduled one full work item per child. Version 0.3.15 keeps a compact array of references and one iterator for each active branch. It visits one child at a time. This reduces temporary allocation while retaining the checks introduced in 0.3.14.

## Use the existing API

```toml
[dependencies]
airust = { version = "=0.3.15", default-features = false, features = ["pdf"] }
```

```rust,no_run
use airust::{PdfImportLimits, PdfLoader};
let loader = PdfLoader::new().with_limits(PdfImportLimits::default());
let preview = loader.preview_pages_from_pdf(std::path::Path::new("manual.pdf"), 1)?;
let chunks = loader.pdf_to_training_examples("manual.pdf")?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

No new public method or flag is needed. Existing JSON knowledge, IDs and metadata remain compatible. Upgrading from 0.3.14 does not require reindexing; no extraction/cleaning/chunking change is intended. Applications adding new documents still import and index those records explicitly. The optional maintained backend remains exactly 0.3.5; default library dependencies and features remain.

## Checks remain bounded

Every present root/subtree `/Count` must equal the actual leaves. Malformed nodes, repeated/cyclic page references and invalid indirect counts fail. The fixed metadata budgets remain depth 50, one million reference-work units and at most 50 indirect count steps. Caller page limits apply to actual leaves. References are charged before scheduling a branch's children. Empty live trees bypass native orphan recovery. Preview validates the full tree but extracts only its useful prefix. See the [complete count contract](PDF-PAGE-COUNTS.md).

The compact reference buffer still scales with the number of children; native parsed arrays, caches, decoded content and output also consume memory. This is not a constant-memory parser or hard OS memory sandbox. Multiply malformed input can produce a different first diagnostic; existing error categories remain.

## Evidence and limits

The [release report](CHANGES-0.3.15.md) separates an isolated metadata-helper comparison from complete converter and preview processes. The synthetic helper includes input construction and traversal but no native PDF parsing. Its memory reduction is not a percentage promised for every real import. Complete outputs and ranking evidence must remain equivalent.

Independent readers check page coverage, while selected literal queries test retrievability. Neither establishes perfect glyph decoding, reading order, OCR, semantic relevance or arbitrary damaged-file recovery. Source PDFs, earlier exports, failed attempts and measurements remain local; public aggregates omit private text. See [reproduction](validation-0.3.15/README.md).
