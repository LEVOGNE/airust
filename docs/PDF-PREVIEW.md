# PDF previews in AIRust 0.3.7

A preview now extracts only the requested number of usable pages. The `pdf2kb --verbose` command uses this API for its three-page raw preview. It previously extracted the entire document for those three lines of output and then imported it again. On a long PDF, that wasted work and could report a later text-budget error before the import's earlier chunk-budget error.

## Library API

```toml
[dependencies]
airust = { version = "=0.3.7", default-features = false, features = ["pdf"] }
```

```rust
use airust::{PdfImportLimits, PdfLoader, PdfLoaderConfig};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let loader = PdfLoader::with_config(PdfLoaderConfig {
        remove_page_numbers: false,
        dehyphenate: false,
        ..Default::default()
    })
    .with_limits(PdfImportLimits::default());
    let pages = loader.preview_pages_from_pdf(Path::new("document.pdf"), 3)?;
    for page in pages {
        println!("Page {}: {} bytes", page.page_number, page.text.len());
    }
    let knowledge = loader.pdf_to_knowledge_base("document.pdf")?;
    loader.save_knowledge_base_if_absent(&knowledge, "knowledge.json")?;
    Ok(())
}
```

The signature is `preview_pages_from_pdf(&self, path: &Path, maximum: usize) -> Result<Vec<PageContent>, PdfError>`. Each result contains `page_number` and `text`. Page numbers are one-based physical PDF pages. Blank pages, unusable extracted text and text removed entirely by the configured cleaner do not count toward `maximum`. A document with fewer usable pages returns fewer results, possibly an empty vector. Zero is an invalid maximum and is rejected before file access. A very large maximum does not preallocate a correspondingly large vector.

The method uses the loader's existing cleaning configuration. Use `clean_text: false` for the raw, trimmed native extraction. The full-document `extract_pages_from_pdf` and knowledge conversion methods keep their behavior. The preview neither writes files nor updates an index. A failure while reading the requested prefix returns an error, without a partial vector.

## Budgets and limits

The whole input-file byte limit and total physical-page limit still apply, even when only one usable page is requested. Raw and cleaned-text budgets accumulate over all visited pages, including raw text subsequently rejected as unusable. Chunk-count and overlap-expanded chunk-byte budgets apply during knowledge conversion; previews do not create chunks.

A successful preview validates only its visited prefix. A later malformed page or exceeded text budget can still make the full import fail. Keep the full conversion and atomic output step before accepting knowledge. The preview still opens the PDF, reads its page tree and can initialize native structure data. A bounded loader reads a complete byte snapshot. Opening, native extraction of a single page, CPU and peak RAM have no hard library deadline. Image-only PDFs need external OCR; encrypted documents requiring a password have no password-entry API here.

For untrusted inputs, the optional CLI's owned worker provides a supervised wall-clock budget and protects previous output. It is not a hard operating-system memory sandbox. See [worker controls](PDF-WORKERS.md) and [all import budgets](PDF-LIMITS.md).

## Command line

```bash
cargo install airust --version 0.3.7 --locked --features cli,pdf --bin pdf2kb
pdf2kb document.pdf knowledge.json --verbose --max-file-mib 128 --timeout-seconds 90
```

The verbose raw preview still prints up to three usable pages with up to 300 displayed characters per page. It uses raw text regardless of cleaning selected for the final import. Full conversion still follows and can fail. With worker supervision, buffered preview output is relayed after the worker finishes; this is not a live progress bar. Existing output requires `--overwrite`. All existing flags and JSON formats remain compatible.

## Upgrade and retained knowledge

No text-cleaning, chunking, ranking, database or backend change is introduced by 0.3.7. Successful imports in the tested corpus produce the same knowledge as 0.3.6. Existing exports remain usable and do not need rebuilding for this preview fix. If several budgets would fail, verbose mode can now report the earlier conversion failure instead of a later error encountered by its former full-document preview.

The local lab retains original PDFs, independent reference text, old and new knowledge exports, test fixtures, measurement logs and release artifacts. Test/fuzz data stay separate from reusable knowledge and are excluded from Git, Cargo packages and Docker context. See [the measured report](CHANGES-0.3.7.md) and [reproduction steps](validation-0.3.7/README.md). No DenkMaps or MCP integration is included.
