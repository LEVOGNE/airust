# PDF structure failures in AIRust 0.3.13

This guide records 0.3.13. Version 0.3.14 also fixes the declared-count recovery gap described below; see [current page-count validation](PDF-PAGE-COUNTS.md).

A PDF can place text inside reusable blocks called Form XObjects. Earlier versions could stop at a nesting or processing limit and still report success with missing text. Fallback page counting could also return a partial count after reaching its depth limit.

Version 0.3.13 reports an import error on these covered paths. It restores local Form state before propagating the error and avoids storing failed span results as successful cache entries. The optional backend is pinned to `airust-pdf-backend = 0.3.5`. Its original licenses, notices, source hashes and maintained patch remain available in `crates/airust-pdf-backend`.

## Existing API

```toml
[dependencies]
airust = { version = "=0.3.13", default-features = false, features = ["pdf"] }
```

```rust,no_run
use airust::{PdfImportLimits, PdfLoader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let loader = PdfLoader::new().with_limits(PdfImportLimits::default());
    match loader.pdf_to_training_examples("manual.pdf") {
        Ok(examples) => println!("Validated {} training records", examples.len()),
        Err(error) => eprintln!("Import failed; retain previous knowledge: {error}"),
    }
    Ok(())
}
```

Methods, configuration fields, CLI flags and saved JSON formats retain their signatures. Existing `PdfError` categories represent failures; exact error strings are not a machine-readable classification contract. Handle an error before replacing application data. An unsuccessful import does not represent a valid empty replacement.

`extract_pages_from_pdf` returns pages after the requested extraction succeeds. `pdf_to_training_examples` and `pdf_to_knowledge_base` propagate the error. `preview_pages_from_pdf` intentionally stops after its requested usable prefix and does not validate unvisited pages. Success is not an exhaustive PDF conformance audit.

## Converter

```bash
cargo install airust --version 0.3.13 --locked --features cli,pdf --bin pdf2kb
pdf2kb manual.pdf knowledge.json --timeout-seconds 90 --max-file-mib 128 --max-pages 50000 --max-text-mib 256 --max-result-mib 256
```

Both direct conversion and supervised conversion keep existing output unchanged when these imports fail, including with `--overwrite`. The supervised worker reports its lifecycle and elapsed time; its progress does not predict a completion time. Use the existing [import limits](PDF-LIMITS.md) and [worker contract](PDF-WORKERS.md).

## Supported limits

The existing native counters remain: ten nested Form levels, a 500-attempt Form processing budget and a depth bound of 50 for fallback page counting. These are internal guards with existing counting rules, not newly configurable CLI limits. Cached or proven text-free Forms can avoid repeated processing; a later call may complete using valid caches. Successful calls must contain all known fixture text. Valid root page counts use the existing separate path.

A proposed increase to 64 recursive Form levels overflowed the default debug test-thread stack. That prototype was rejected and retained as evidence. The published fix preserves the existing limit and propagates failures. It does not claim support for arbitrarily deep PDFs.

Other best-effort recovery, glyph decoding, reading order and cleanup remain heuristic. This fix does not make every damaged PDF fully recoverable, add OCR/password handling, or impose an operating-system RAM sandbox. The main engine's public search, matching, ranking and storage APIs remain unchanged.

A separate three-case diagnostic confirms that the standard path still trusts a plausible positive root `/Count`. Deliberately changing a two-page fixture to declare one page returns its first page successfully; declaring three pages fails. This is a documented damaged-metadata recovery limit, separate from the fixed fallback-depth failures. A strict complete-document contract needs independent page-tree consistency evidence; it cannot infer that evidence from a successful import alone. The diagnostic and its fixtures remain retained for [0.4 preparation](PREPARING-0.4.md).

## Upgrade and retained knowledge

Keep existing exports. The 33 retained real PDFs produce the same complete output in the tested comparison. A source previously accepted through a covered limit may now fail; preserve the old export and review that source explicitly. Do not treat an old successful status as proof that it contained all original PDF text. Any deliberate reimport should record its source and configuration revision before replacing derived indexes.

See [executed results](CHANGES-0.3.13.md), [reproduction](validation-0.3.13/README.md), [stability gates](STABILITY-0.3.md) and [0.4 preparation](PREPARING-0.4.md). No DenkMaps or MCP integration is added.
