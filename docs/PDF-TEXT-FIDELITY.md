# PDF text fidelity in AIRust 0.3.6

AIRust 0.3.6 corrects text cleaning without changing public method signatures or JSON formats. Corrected imports can change text, chunk boundaries and ranking. The default core and the optional PDF backend remain unchanged; the backend is still version 0.3.4.

## What was fixed

- Line joining no longer turns a wrapped digit range such as `10-` followed by `20 units` into `1020 units`. Digits and underscores do not qualify as alphabetic join boundaries.
- A blank paragraph is kept separate. A hyphen at the end of one paragraph no longer joins it to the next paragraph.
- Chains of short wrapped fragments are handled in one pass without consuming the boundary letters of the next match.
- Unicode is normalized again after controls are removed and lines are joined. This keeps composed characters consistent, including a Hangul syllable split across a wrap.
- Page-number removal considers only the first and last nonempty extracted lines. Numeric rows inside the page are retained and become searchable.

The regression PDF contains table values, a wrapped range and a footer. Its imported value `100` is searchable, and the false fused value `1020` is absent. String regressions also cover Roman numerals, paragraphs, CRLF, Greek, Hangul, controls and a 20,000-wrap chain.

## Use the existing API

```toml
[dependencies]
airust = { version = "=0.3.6", default-features = false, features = ["pdf"] }
```

For technical tables or source code, explicitly keep possible page-edge values and meaningful hyphens:

```rust
use airust::{PdfImportLimits, PdfLoader, PdfLoaderConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = PdfLoaderConfig {
        remove_page_numbers: false,
        dehyphenate: false,
        ..Default::default()
    };
    let loader = PdfLoader::with_config(config).with_limits(PdfImportLimits::default());
    let knowledge = loader.pdf_to_knowledge_base("document.pdf")?;
    loader.save_knowledge_base_if_absent(&knowledge, "knowledge.json")?;
    Ok(())
}
```

The CLI provides the same controls:

```bash
cargo install airust --version 0.3.6 --locked --features cli,pdf --bin pdf2kb
pdf2kb document.pdf knowledge.json --keep-page-numbers --no-dehyphenate --max-file-mib 128 --timeout-seconds 90
```

`--no-dehyphenate` preserves hyphenated wraps while other cleaning still runs. `--keep-page-numbers` preserves numeric edge lines. `--no-clean` bypasses AIRust's cleaning step; native extraction and chunk formatting still apply. The collecting `extract_pages_from_pdf` method returns physical page numbers and page text. `clean_text` itself has no resource limit and is intended for already extracted text.

## Important limits

Cleaning remains heuristic. The first or last numeric line can be real data rather than a page number. An alphabetic compound can contain a meaningful hard hyphen. Use the controls above when that distinction matters. Joining recognizes an ordinary LF/CRLF wrap with spaces or tabs; ambiguous or other whitespace is retained conservatively. The method does not infer word meaning or recover exact PDF glyph positions. Apply cleaning to the extracted page once rather than repeatedly cleaning saved exports: another pass of edge-number filtering can expose a different numeric edge. Disable that heuristic for reused or literal numeric data.

The existing [import budgets](PDF-IMPORT-PIPELINE.md) and [worker supervision](PDF-WORKERS.md) remain active. More retained text can exhaust a tight budget that an older, more destructive import happened to pass. No partial result is returned. Failed imports preserve previous output. Native initialization/current-page parsing can still be expensive; limits are not hard process RAM or CPU quotas. No OCR or password-entry API is added.

## Real documents and independent checks

Three additional official references add 9,365 physical pages:

| Source | Pages | 0.3.6 chunks |
| --- | ---: | ---: |
| [PostgreSQL 18 A4 manual](https://www.postgresql.org/files/documentation/pdf/18/postgresql-18-A4.pdf) | 3,178 | 11,570 |
| [Vulkan ratified specification](https://registry.khronos.org/vulkan/specs/latest-ratified/pdf/vkspec.pdf) | 5,491 | 17,744 |
| [RISC-V unprivileged manual](https://docs.riscv.org/reference/isa/v20260120/_attachments/riscv-unprivileged.pdf) | 696 | 2,157 |

The full retained corpus now contains 21 real documents and 25,321 pages. An old/new differential run shares the same unchanged parser and the root lockfile's dependency versions. Every page is checked against an independent Python implementation using Unicode properties and lookaround. The independent normalization runtime has its own recorded Unicode version; this is a corpus/known-script check, not proof for every Unicode release.

The declared policy changes affect 2,782 cleaned pages. Previously populated pages remain present. Every unchanged cleaned page has identical chunks and metadata. All raw sources remain unchanged. The fresh import contains 91,766 entries, compared with 91,732 when the previous importer processes the same 21 documents. Independent MuPDF extraction and selected pypdf probes complement this comparison. The new RISC-V reference's one omitted page is independently blank.

Across the fixed literal page probes, 24 rank lists change, with no gained or lost expected-page top-ten hit. The original 115 probes retain BM25 111/115 and TF-IDF 102/115. The prior 27-probe group retains 26/27 and 18/27; the new 31-probe group yields 28/31 and 26/31. These are literal extraction/retrieval probes using different sampling groups, not human relevance judgments or proof of semantic understanding.

A separate 4,096-case string campaign matches the independent cleaner under all tested flag combinations. The 4,096 malformed-PDF campaign has zero caught parser panics, abnormal parent exits or outer timeouts; failed-import outputs are preserved. Acceptance classification is checked against the prior version case by case. Owned difficult-format tests, a real worker deadline and an owned-worker failure check remain in the evidence. Accepted damaged PDFs are not certified complete.

See [measurements and validation](CHANGES-0.3.6.md). Default output sizes intentionally differ, so timing/RSS comparisons do not establish a same-output speedup. With `--no-clean`, paired outputs for the three new references remain byte-identical.

## Upgrade and retain knowledge

Older exports remain valid and are kept byte for byte. The new combined canonical export in the ignored local lab is `imports/canonical-combined-036.json`, with 91,766 unique IDs from all 21 originals. The previous 60,266-entry export and all earlier versions remain separate. No authored-format, mutation or soak data enters canonical knowledge.

Reimport explicitly and rebuild derived indexes, excerpts or vectors from the new text. A page/chunk-derived ID can refer to different text after rechunking. Do not append both versions of the same source blindly. Record the importer version, configuration and lockfile along with the source identity; a PDF hash alone cannot identify differently cleaned text. Keep the previous snapshot while validating a replacement. No application database is changed here.

No DenkMaps or MCP integration is performed.
