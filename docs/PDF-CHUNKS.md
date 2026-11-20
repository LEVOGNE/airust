# Faithful PDF chunks in AIRust 0.3.8

AIRust divides long pages into smaller knowledge entries. Before 0.3.8, sentence splitting trimmed existing whitespace and added a space. Paragraph splitting could add blank lines or remove indentation. This could change code, lists and line breaks even with `clean_text = false`.

Version 0.3.8 preserves the original characters when choosing sentence and paragraph boundaries. Each resulting chunk is a contiguous excerpt of the page text supplied to the chunker. Outer whitespace is still trimmed from the page and each chunk. Short pages keep their existing behavior. The native parser, cleaning rules and optional backend remain unchanged.

## Use the existing API

```toml
[dependencies]
airust = { version = "=0.3.8", default-features = false, features = ["pdf"] }
```

```rust
use airust::{PdfImportLimits, PdfLoader, PdfLoaderConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let loader = PdfLoader::with_config(PdfLoaderConfig {
        remove_page_numbers: false,
        dehyphenate: false,
        ..Default::default()
    })
    .with_limits(PdfImportLimits::default());
    let knowledge = loader.pdf_to_knowledge_base("document.pdf")?;
    loader.save_knowledge_base_if_absent(&knowledge, "knowledge.json")?;
    Ok(())
}
```

No new public method or flag is required. The default chunk settings remain a minimum boundary preference of 50 characters, a maximum of 1,000 characters and an overlap of 200 characters. These are Unicode scalar counts, not bytes or tokens. A final chunk can be shorter than the preferred minimum. Hard maximums still take priority over sentence/paragraph boundaries and can split a word. A high overlap can repeat much of the text. Text-byte budgets include repeated chunk content.

`split_by_paragraph` prefers paragraph boundaries. With `split_by_sentence`, long paragraphs also offer sentence boundaries. Disabling both uses only the hard character limit and overlap. Sentence recognition remains heuristic, including abbreviations, decimal numbers and parentheses. It selects possible cut positions; it does not rewrite punctuation, indentation or separators.

Set `clean_text = false` to bypass AIRust cleaning before chunking. Native extraction and outer trimming still apply. Set `remove_page_numbers = false` and `dehyphenate = false` to keep possible numeric edge data and meaningful hyphens while retaining other cleaning. See [cleaning controls](PDF-TEXT-FIDELITY.md).

## Command line

```bash
cargo install airust --version 0.3.8 --locked --features cli,pdf --bin pdf2kb
pdf2kb document.pdf knowledge.json --keep-page-numbers --no-dehyphenate --max-file-mib 128 --timeout-seconds 90
```

Add `--no-clean` to bypass AIRust cleaning. The three-page raw preview API and verbose preview behavior introduced in 0.3.7 remain unchanged. See [preview semantics](PDF-PREVIEW.md), [import budgets](PDF-LIMITS.md) and [supervised workers](PDF-WORKERS.md). Existing destinations require explicit `--overwrite`.

## Upgrade without losing knowledge

Public Rust signatures and JSON fields remain compatible. Corrected long-page imports can change chunk text, counts, IDs, offsets and rankings. Preserve old exports and source/configuration versions. Reimport into a separate export, compare results, then explicitly rebuild derived lexical/vector/ANN indexes, caches and provenance bindings. Do not append both chunk revisions to the same live index as separate knowledge.

The lab keeps old exports unchanged and stores a fresh 0.3.8 corpus separately. Authored, damaged and edited stress-test inputs stay outside canonical knowledge. No application database is changed. No DenkMaps or MCP integration is included.

## Limits

A faithful chunk is an excerpt of the extracted and optionally cleaned text, not a promise that the native PDF reading order or cleaning is always correct. Original PDF glyph coordinates are not supplied. Chunk edges can lose whitespace through documented trimming; overlapping chunks repeat content. Image-only pages need external OCR. Password entry is not exposed. Native document opening, structure initialization and one extraction call have no hard library CPU/RAM deadline. CLI worker supervision provides a wall-clock budget, not an operating-system memory sandbox.

See [executed tests, measurements and known limits](CHANGES-0.3.8.md) and [reproduction steps](validation-0.3.8/README.md).
