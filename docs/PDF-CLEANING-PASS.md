# PDF cleaning in AIRust 0.3.10

AIRust's cleaner previously expanded five Latin ligatures but left the two `st` presentation forms unchanged. For example, `firﬅ` stayed different from plain `first`. Version 0.3.10 expands all seven forms U+FB00–U+FB06. The `st` mapping follows the Unicode compatibility fallback. This step remains deliberately limited; it does not apply general NFKC folding to mathematical symbols or other scripts. See the [Unicode fallback table](https://www.unicode.org/cldr/charts/45/supplemental/character_fallback_substitutions.html).

The first cleaning stage now performs NFC normalization, Latin-ligature expansion and control filtering in one pass. It avoids the previous chain of whole-text replacement copies. The later normalization pass still composes characters after controls are removed or wrapped words are joined. Existing numeric-boundary, paragraph, page-number and dehyphenation rules remain.

## Existing API

```toml
[dependencies]
airust = { version = "=0.3.10", default-features = false, features = ["pdf"] }
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
    assert_eq!(loader.clean_text("firﬅ coﬆ"), "first cost");
    let knowledge = loader.pdf_to_knowledge_base("document.pdf")?;
    loader.save_knowledge_base_if_absent(&knowledge, "knowledge.json")?;
    Ok(())
}
```

No public method, field, flag or JSON schema is added. `clean_text(&str) -> String` explicitly cleans already extracted text. It does not apply file/page/text limits or consult the import bypass flag. Use it with caller-controlled input sizes.

During PDF import, `clean_text: false` bypasses AIRust cleaning. Native extraction can still expand ligatures or otherwise transform PDF glyph data. A raw import therefore is not a copy of the PDF's original bytes. Set `remove_page_numbers: false` for literal numeric edge data and `dehyphenate: false` for meaningful hyphens while retaining other cleaning.

## Command line

```bash
cargo install airust --version 0.3.10 --locked --features cli,pdf --bin pdf2kb
pdf2kb document.pdf knowledge.json --keep-page-numbers --no-dehyphenate --max-file-mib 128 --timeout-seconds 90
```

`--no-clean` bypasses AIRust cleaning. Existing destinations require explicit `--overwrite`. See [cleaning controls](PDF-TEXT-FIDELITY.md), [faithful chunks](PDF-CHUNKS.md), [overlap processing](PDF-OVERLAP.md), [import limits](PDF-LIMITS.md) and [worker supervision](PDF-WORKERS.md).

## Upgrade and retained knowledge

Existing exports remain usable. Inputs containing U+FB05/U+FB06 can produce corrected text when cleaning is enabled. Such changes can alter derived chunks, IDs, offsets and ranking. Preserve old exports and source/configuration revisions, reimport separately where needed and explicitly rebuild derived indexes. Do not merge two chunk revisions as duplicate knowledge.

The corpus report identifies actual changes rather than assuming that every document is affected. Native extraction may already have expanded these forms. Older knowledge, original PDFs, independent references, fixtures and raw measurements remain retained locally. Authored/damaged test data stay outside canonical knowledge. No DenkMaps or MCP integration is added.

## Limits and measurements

Cleaning remains heuristic: edge numbers and hyphenated wraps can be meaningful data. Native PDF reading order and glyph decoding can be imperfect. Image-only pages require external OCR; password entry is not exposed. The standalone helper and synchronous native calls have no hard total CPU/RAM bound. The CLI worker provides a supervised wall-clock budget rather than an operating-system memory sandbox.

The helper's owned-text CPU/RSS measurements are separate from real PDF imports. Avoiding intermediate copies does not establish a universal whole-import speedup or memory limit. See [executed results and scope](CHANGES-0.3.10.md) and [reproduction steps](validation-0.3.10/README.md).
