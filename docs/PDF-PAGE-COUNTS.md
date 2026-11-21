# PDF page counts in AIRust 0.3.14

A PDF declares how many pages belong to each branch of its page tree. Earlier imports trusted a plausible positive root count. A two-page file declaring one page could therefore produce a successful one-page import. Wrong intermediate counts could hide or repeat pages even when the root total was correct.

Version 0.3.14 independently walks the complete reachable page tree before extracting text. It counts actual leaf pages and compares every present root and subtree count with that result. Inconsistent declarations cause an explicit error. AIRust keeps the existing output and original PDF unchanged; it does not silently repair metadata or publish a partial replacement.

## Use the existing API

```toml
[dependencies]
airust = { version = "=0.3.14", default-features = false, features = ["pdf"] }
```

```rust,no_run
use airust::{PdfImportLimits, PdfLoader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let loader = PdfLoader::new().with_limits(PdfImportLimits::default());
    match loader.pdf_to_training_examples("manual.pdf") {
        Ok(records) => println!("Imported {} records", records.len()),
        Err(error) => eprintln!("Import failed; keep previous knowledge: {error}"),
    }
    Ok(())
}
```

The check applies to `extract_pages_from_pdf`, `preview_pages_from_pdf`, training-example and knowledge-base conversion, and both converter modes. It is part of the optional `pdf` feature. No public method, configuration field, flag, JSON field or mandatory core dependency is added. The backend stays exactly 0.3.5; the check lives in AIRust.

Existing `PdfError` categories represent failures. Messages identify the object, declared count and actual count when a declaration conflicts. Exact wording is not a stable machine-readable protocol. Configured page limits apply to actual leaves, so a false declaration cannot bypass `max_pages`.

## What is checked

- The catalog must identify a reachable page-tree root of type `Pages`.
- Every reached tree node must be a usable dictionary of type `Pages` or `Page`. Branches need an array of indirect child references.
- Every present count must resolve to a nonnegative integer and match its actual subtree. Bounded indirect count references are supported.
- Repeated object numbers, including generation aliases, and cycles fail. A repeated page reference is not silently counted as another page.
- Missing counts retain the existing recovery path when the full tree is usable. For a nonempty live tree, the backend's count must agree with the independent count. An empty live tree returns no pages and bypasses the native orphan-recovery scan.

Only pages reachable through the catalog's live tree are counted. Unreferenced old or orphan page objects are not extra document pages. The check does not scan arbitrary objects to guess another order or rewrite source files.

## Budgets and preview behavior

Validation uses an iterative walk with a maximum tree depth of 50, starting at zero for the root. It allows at most 1,000,000 units of reference work, including the root, scheduled child edges and indirect count lookups. Child batches are checked before scheduling them. Indirect count chains allow at most 50 lookups and reject cycles. These fixed internal guards apply even without explicit `PdfImportLimits`.

The caller's positive `max_pages` limit is also enforced while actual leaves are counted. A validation limit or failure returns no import result. Native object loading can still allocate inside one call; these counters are not an operating-system RAM sandbox. Use [import budgets](PDF-LIMITS.md) and [supervised jobs](PDF-WORKERS.md) for their additional documented protections.

A preview now validates structural metadata for the entire reachable tree before extracting its requested usable prefix. It still stops content extraction after that prefix. Later page content, fonts and glyphs are not validated by a prefix preview. The metadata pass adds work before the first result; the release report measures preview and complete conversion separately.

## Converter

```bash
cargo install airust --version 0.3.14 --locked --features cli,pdf --bin pdf2kb
pdf2kb manual.pdf knowledge.json --timeout-seconds 90 --max-file-mib 128 --max-pages 50000 --max-text-mib 256 --max-result-mib 256
```

Both direct and supervised failures preserve existing output, including with `--overwrite`. The source PDF is never changed. A mismatch requires source review or an explicit external repair before another import; do not replace old application data with an empty result.

## Upgrade and limits

Keep old exports. All 33 retained real PDFs produce unchanged complete output in the tested comparison. Previously accepted files with inconsistent counts or structures beyond the validation budgets can now fail. A source accepted by an older version may already have an incomplete export; review it explicitly before rebuilding derived data under a new source/configuration revision.

The check validates the reachable structure and its counts. It does not prove perfect reading order, glyph decoding, cleanup, OCR or arbitrary damaged-file recovery. It cannot recover deleted or unreachable content merely from a hash or a successful status. Existing knowledge-engine APIs and the default core remain unchanged.

See [executed results](CHANGES-0.3.14.md), [reproduction](validation-0.3.14/README.md) and [stability gates](STABILITY-0.3.md). No DenkMaps or MCP integration is added.
