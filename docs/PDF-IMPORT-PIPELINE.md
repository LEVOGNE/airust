# PDF import limits in AIRust 0.3.5

AIRust now checks chunk budgets as it processes each page. A document that already exceeds its chunk count or stored-text budget no longer needs to finish extracting its later pages. Successful imports keep the same text, page metadata and JSON format. The default core and all public method signatures are unchanged.

## Use the existing API

```toml
[dependencies]
airust = { version = "=0.3.6", default-features = false, features = ["pdf"] }
```

```rust
use airust::{PdfImportLimits, PdfLoader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let limits = PdfImportLimits::new(128 * 1024 * 1024, 6000, 64 * 1024 * 1024, 100000)?;
    let loader = PdfLoader::new().with_limits(limits);
    let knowledge = loader.pdf_to_knowledge_base("document.pdf")?;
    loader.save_knowledge_base_if_absent(&knowledge, "knowledge.json")?;
    Ok(())
}
```

The four limits are input-file bytes, physical pages, text bytes and chunk count. Overlapping text counts toward the chunk-text budget. Saving to a new path preserves existing files. Use explicit replacement only when that is intended.

The CLI uses the same options:

```bash
cargo install airust --version 0.3.14 --locked --features cli,pdf --bin pdf2kb
pdf2kb document.pdf knowledge.json --max-file-mib 128 --max-pages 6000 --max-text-mib 64 --max-chunks 100000 --timeout-seconds 90
```

The supervised child provides a way to stop a blocking parser call. [Worker supervision](PDF-WORKERS.md) explains its exact boundary.

## What changed internally

Earlier conversion collected all cleaned pages before checking chunk count and overlap-expanded chunk bytes. Even a one-chunk request could extract a whole large manual before failing.

Conversion now checks these budgets immediately after cleaning each page. It stages one copy of the validated chunks. After extraction succeeds, it releases the parser and creates the training input/output pairs. This avoids retaining the parser's caches alongside duplicated training text. Chunk strings are moved into their final records instead of being copied twice.

The collecting method `extract_pages_from_pdf` still returns complete page contents. It applies file, page, raw-text and cleaned-text limits; it does not split or limit chunks. Conversion uses the same extraction path but adds its chunk checks during the visit. If several limits could fail, the first error can differ from the older processing order. No partial result is returned, and failed CLI conversion preserves previous output.

## Larger and difficult inputs

Three additional official documents add 6,996 physical pages:

| Source | Pages | Imported chunks |
| --- | ---: | ---: |
| [Intel combined software developer manual](https://cdrdv2.intel.com/v1/dl/getContent/671200) | 5,364 | 20,134 |
| [Adobe PDF 1.7 specification](https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/PDF32000_2008.pdf) | 756 | 3,137 |
| [ECMA-262 specification](https://ecma-international.org/wp-content/uploads/ECMA-262.pdf) | 876 | 3,287 |

The retained real corpus now has 18 documents and 15,956 physical pages. An independent MuPDF extraction covers every new page. Separate pypdf checks supply 27 literal page probes. All eight omitted pages in the new corpus are independently blank. Checked text, provenance and rankings match the previous importer. The new probes find the expected page in the top ten for BM25 26/27 times and TF-IDF 18/27 times. These are extraction/retrieval probes, not human relevance judgments or semantic understanding.

Eight authored format fixtures exercise 2,163 test pages per converter, separately from the real corpus. They cover compressed object streams, an incremental revision, rotations, German/Greek/Cyrillic text, AES-128/AES-256 with an empty user password, password protection and image-only pages. Successful output pairs match. Password-protected and image-only cases return normal errors and preserve previous output. This release adds neither OCR nor a password-entry API. Rendering caught an unsuitable font in the first Unicode fixture; the corrected fixture is independently rendered and checked against all original phrases.

The 4,096 retained mutations produce 2,585 accepted files and 1,511 normal errors, with zero caught parser panics, unexpected parent exits or outer-watchdog timeouts. All failure outputs are preserved. The source-equivalent 0.3.4 classifications remain unchanged. Accepted damaged files are not certified complete or semantically correct.

## Measurements and limits

[Aggregate measurements](measurements/0.3.5/) retain both the initial candidate and the final comparison. The first candidate kept parser caches alive beside training input/output copies and increased full-import RAM. It was corrected before release. Successful full-import latency and RSS vary by file and load; no universal speed or RAM improvement is claimed. The useful change is earlier rejection after a chunk budget fails. See the [release report](CHANGES-0.3.5.md) for the final repeated measurements.

This is not a streaming public API or a hard RAM/CPU quota. The importer still retains the complete validated result, a bounded input snapshot and parser data. Native initialization and extraction of the current page may still be expensive. Verbose CLI preview retains its separate extraction pass. Use a supervised job for a deadline around blocking native work. Library methods remain synchronous.

The historical 0.3.5 importer retained `airust-pdf-backend = 0.3.4` with unchanged source/notices. Current 0.3.13 uses backend 0.3.5 and the maintained [structure-limit fix](PDF-STRUCTURE-LIMITS.md). The earlier [parser optimization](PDF-PARSER.md) remains active. Fresh advisory checks report zero known vulnerabilities and three maintenance warnings, recorded in [audit evidence](measurements/0.3.5/audit.json); this is not a security certification.

## Retained knowledge

The local lab keeps the old 33,708-entry canonical export intact and adds a separate combined export with 60,266 unique entries. New IDs have a document namespace; source/page/chunk metadata remain intact. No fuzz, authored-format or edited soak output enters that canonical knowledge. Original PDFs, references, binaries, exports and raw logs remain local and excluded from Git, crate packaging and Docker context. The current `saved_knowledge` example can reopen the combined index.

README, the API cookbook and the release evidence explain what changed. No DenkMaps or MCP code, storage or transport is integrated.


AIRust 0.3.6 retains these budget/worker APIs and intentionally corrects text cleaning. Historical 0.3.5 output-equality measurements do not imply unchanged 0.3.6 text. See [current controls and upgrade notes](PDF-TEXT-FIDELITY.md).


Current 0.3.13 also rejects the covered native Form/page-count work-limit omissions. See [API and upgrade](PDF-STRUCTURE-LIMITS.md) and [fresh validation](CHANGES-0.3.13.md). Historical measurements above retain their identified release scopes.


Version 0.3.14 validates full page-tree metadata before extraction, including previews, and rejects inconsistent declared counts. See [current API and budgets](PDF-PAGE-COUNTS.md). Historical timing and recovery claims above keep their identified release scopes.
