# Explicit PDF import limits

AIRust 0.3.2 adds opt-in bounds to the existing optional PDF importer. It also fixes PDF CLI argument panics and escapes terminal controls in its own previews and messages. It adds no model, service or application integration.

## Library API

```rust
use airust::{PdfImportLimits, PdfLoader};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let limits = PdfImportLimits::new(64 * 1024 * 1024, 3000, 64 * 1024 * 1024, 50000)?;
let loader = PdfLoader::new().with_limits(limits);
# let _ = loader;
# Ok(()) }
```

The four constructor arguments are maximum input-file bytes, physical pages, text bytes and chunks. Getters expose those values. Zero limits and a file limit without room for an overflow-detection byte are rejected. The type has private fields and no unchecked deserialization path. Existing PdfLoader constructors retain their earlier behavior unless with_limits is selected.

Default bounded policy: 256 MiB input, 20,000 pages, 64 MiB text and 100,000 chunks. These are configurable policy limits, not claims that a process will use that much RAM.

The bounded reader requires a regular file, checks its advertised size, then reads at most the configured maximum plus one byte. The extra byte detects growth beyond the cap. Parsing uses that bounded byte snapshot. A file can still change while it is read; this is not authenticated file versioning or a filesystem authorization boundary.

The page cap is checked before per-page text extraction. The text cap applies separately to cumulative raw extracted UTF-8 bytes, cleaned UTF-8 bytes and stored chunk-input UTF-8 bytes. Overlap counts toward chunk bytes, and the chunk count covers the entire document rather than resetting per page. Splitting checks the remaining budget before retaining another chunk. This prevents high-overlap settings from building an unbounded per-page chunk collection before rejecting the result.

A violation returns an error with the budget name and its actual/maximum value where representable. No partial import is returned. CLI conversion therefore leaves existing output untouched when any limit fails, including with --overwrite. Original text and ranking behavior are unchanged for successful imports in the tested corpus.

Native parsing can allocate or perform substantial work inside one call before text limits can be checked. The byte snapshot is copied by the current PDF dependency, parser caches add memory, and training responses duplicate text. These options are not hard process-memory limits, hard CPU deadlines or a constant-memory streaming API. The existing slow first-page case remains; 0.3.3 provides optional [supervised CLI jobs](PDF-WORKERS.md) for this boundary. Library methods remain synchronous.

## CLI

```bash
cargo run --locked --features cli,pdf --bin pdf2kb -- INPUT.pdf OUTPUT.json --max-file-mib 64 --max-pages 3000 --max-text-mib 64 --max-chunks 50000
```

The flags are --max-file-mib, --max-pages, --max-text-mib and --max-chunks. MiB means 1,048,576 bytes. Selecting any limit enables the complete bounded default set, then applies the explicit overrides. Omitting all limit flags preserves the previous unbounded policy. Values must be positive integers; multiplication overflow is an ordinary error. Verbose preview extraction also uses the chosen input/page/text limits; conversion then applies the chunk limits.

CLI paths are handled as operating-system strings. Numeric option values must be valid Unicode and valid numbers. Invalid-byte arguments no longer cause env::args to panic. On Unix, filesystem path bytes are preserved; display metadata may still use lossy Unicode, so filenames are not an application identity scheme.

The CLI escapes terminal controls and directional formatting controls in its own previews, error messages and displayed output paths. File I/O paths and knowledge text are not modified. The process-wide panic hook and dependency diagnostics are not globally replaced or filtered.

## Validation

Tests cover exact file/page boundaries, cumulative and overlap-expanded text, cross-page chunk counts, UTF-8 byte counting, invalid/overflowing settings, nonregular inputs, malformed headers, Unicode argument errors and output preservation. The existing 500-page concurrent-output regression remains active.

The local lab repeats the same fifteen real PDFs and literal page probes with bounded defaults and compares all content hashes, page sets, chunk counts and ranked IDs against 0.3.1. Adversarial tests use the same authored inputs and deterministic mutations as the [0.3.1 campaign](PDF-STABILITY.md). These are bounded robustness probes, not exhaustive security certification or independent relevance judgments.

Retained local lab data is now also explicitly excluded from the Docker build context. Docker is unavailable on the validation host; this exclusion is reviewed statically, and no executed Docker image test is claimed. Git and crate isolation checks remain in place.
