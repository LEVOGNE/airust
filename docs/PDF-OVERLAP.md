# PDF overlap processing in AIRust 0.3.9

AIRust can repeat the end of one knowledge chunk at the start of the next. This keeps nearby context together. A large overlap can also repeat a lot of work.

Before 0.3.9, each window became a temporary string before trimming and budget checks. Even windows containing only whitespace were copied. Accepted text was then copied again into its final chunk. Version 0.3.9 tracks the trimmed edges as windows move forward, skips empty windows without a temporary string and checks the chunk/UTF-8 budget before retaining the result.

When the remaining byte budget can safely hold the maximum UTF-8 size, the chunk is encoded directly. A tighter remaining budget uses an exact byte count first. Both routes retain the same byte accounting and errors. This is an internal optimization; public methods, configuration fields and JSON formats are unchanged.

## Use the existing API

```toml
[dependencies]
airust = { version = "=0.3.9", default-features = false, features = ["pdf"] }
```

```rust
use airust::{PdfImportLimits, PdfLoader, PdfLoaderConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let loader = PdfLoader::with_config(PdfLoaderConfig {
        max_chunk_size: 1000,
        min_chunk_size: 50,
        chunk_overlap: 200,
        ..Default::default()
    })
    .with_limits(PdfImportLimits::default());
    let knowledge = loader.pdf_to_knowledge_base("document.pdf")?;
    loader.save_knowledge_base_if_absent(&knowledge, "knowledge.json")?;
    Ok(())
}
```

The sizes above count Unicode scalar characters, not bytes or tokens. The overlap must be smaller than the maximum chunk size. A final chunk can be shorter than the preferred minimum; hard limits and the end of the page take priority. Existing sentence/paragraph boundary choices and outer trimming remain unchanged. Chunks remain contiguous excerpts of their extracted/cleaned page, as established in 0.3.8.

For technical text, use `remove_page_numbers: false` and `dehyphenate: false` when appropriate. `clean_text: false` bypasses AIRust cleaning, while native extraction and outer trimming still apply. See [chunk fidelity and controls](PDF-CHUNKS.md), [cleaning](PDF-TEXT-FIDELITY.md) and [preview behavior](PDF-PREVIEW.md).

## Command line

```bash
cargo install airust --version 0.3.9 --locked --features cli,pdf --bin pdf2kb
pdf2kb document.pdf knowledge.json --max-file-mib 128 --timeout-seconds 90
```

`--max-chunk N`, `--min-chunk N` and `--overlap N` select the same settings. `--no-clean` bypasses cleaning. Existing destinations still require explicit `--overwrite`. The supervised worker validates complete output before publication and preserves old output on failure. See [all import budgets](PDF-LIMITS.md) and [worker controls](PDF-WORKERS.md).

## Compatibility and retained knowledge

No reimport is required for this optimization. The release checks compare complete knowledge text, metadata and fixed search rankings with 0.3.8. Existing exports remain usable. The lab keeps older exports, original PDFs, references, fixtures and raw measurements. Additional official documents are stored under separate namespaces; authored and damaged test inputs are excluded from canonical knowledge.

No DenkMaps or MCP integration is added. No database, application adapter, model or network service becomes mandatory.

## Limits and measurement scope

A high overlap still repeats nonempty content, consumes text-byte budgets and can create many chunks. This optimization does not remove those legitimate costs. Page extraction, cleanup, segmentation, index construction and final serialization retain their own time/RAM costs. Native PDF reading order can be imperfect. Image-only pages need external OCR; password entry is not exposed.

Library calls have no hard total CPU/RAM bound. The optional CLI worker provides a supervised wall-clock budget, not an operating-system memory sandbox. Synthetic extracted-text stress results measure chunk processing, not whole-PDF parsing or a universal speedup. Real-document timing and scope are reported separately in [the release report](CHANGES-0.3.9.md) and [validation guide](validation-0.3.9/README.md).
