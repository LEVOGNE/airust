# AIRust 0.3.7: Stop PDF previews after their useful prefix

Verbose PDF conversion previously extracted every page for a preview that displayed only three usable pages. It then extracted the document again for conversion. A 500-page owned regression demonstrates the consequence: a later raw-text limit hid an earlier first-page chunk-limit failure. The regression now reports the chunk limit and leaves the prior output untouched.

The additive optional `PdfLoader::preview_pages_from_pdf` API stops after the requested usable prefix. It preserves physical page numbers, configured cleaning and whole-input byte/page limits. The CLI uses it for its three raw preview pages. Full extraction, successful conversion, text cleaning, chunking, indexing and ranking keep their 0.3.6 behavior. The separately published optional backend remains exactly 0.3.4. See [API, commands, upgrade advice and limits](PDF-PREVIEW.md).

## Corpus and retained knowledge

Three further official snapshots add 3,593 physical pages: GCC (1,261), GDB (1,008) and PGF/TikZ (1,324). The GCC and GDB URLs include development documentation; recorded hashes identify the tested bytes, not a claim about the latest stable release. The complete corpus now has 24 documents and 28,914 physical pages. PyMuPDF independently extracts all new pages, and selected pypdf pages supply 33 fixed literal page probes. Six rendered samples were visually checked. The 37 new-document pages without AIRust chunks are also empty in the independent MuPDF extraction. Coverage does not establish perfect reading order or semantic fidelity.

The retained canonical export now contains 105,647 unique entries. Its previous 91,766 entries have identical text and metadata; all older export files remain byte-identical. All 33 new literal probes retrieve their expected page in the top ten with both BM25 and TF-IDF. This is a fixed page-coverage result, not a claim about semantic understanding.

For all 24 sources, the differential harness compares registry 0.3.6 and current 0.3.7 with the same backend and root-pinned dependencies. It checks raw and cleaned prefixes of one, three and five usable pages against the old complete extractor. Full knowledge exports and fixed BM25/TF-IDF rankings are compared separately. All original source hashes, previous canonical exports, independent references, fixtures, binaries and raw measurements remain in the private lab. Public aggregates contain IDs and hashes rather than document text. Authored and damaged test inputs are excluded from canonical knowledge.

## Failures and boundaries

The 4,096 deterministic malformed-PDF cases run in both ordinary and verbose modes (8,192 executions). Each mode has 2,585 accepted inputs and 1,511 controlled errors, with zero caught parser panics, abnormal parent exits, worker deadlines or outer timeouts. They retain their case-by-case 0.3.6 acceptance classification. Failed conversions preserve previous output. The eight authored difficult-format cases include object streams, incremental updates, rotations, Unicode, encryption and image-only pages. Protected/image-only cases still return controlled errors. A real one-second deadline on the 2,488-page reference and a verified SIGKILL to the owned worker both preserve output and reap the child.

A preview validates only the visited prefix. Later pages can still fail full conversion. Opening, bounded whole-file snapshots, page trees and native structure initialization retain costs. Synchronous native calls have no hard library time/RAM quota. Cleaning remains heuristic; explicit flags preserve numeric edge data and meaningful hyphens. No OCR, password-entry API, Docker execution, exhaustive security certification, DenkMaps/MCP integration or 0.4.0 feature expansion is claimed.

## Release gates

The complete suite passes 197 default and 698 all-feature tests, compiled examples, five Python tests, all 37 isolated feature entries and strict lint/documentation. The ten backend regressions and provenance check pass. Rust 1.89/1.92, 229 patch API checks (31 tool skips) and the independent local consumer with an owned-PDF preview/import check also pass. Current main/backend audits find zero known vulnerabilities, retaining three/four unmaintained-dependency warnings. These scans are not security certification.

The [reproduction guide](validation-0.3.7/README.md) and [aggregate evidence](measurements/0.3.7/) describe the executed checks. The pipeline requires full tests, strict lint/docs, isolated feature checks, backend provenance/regressions, Rust 1.89/1.92, patch compatibility against 0.3.6 and an independent Rust 1.89 consumer exercising the new API. Main/backend audits retain maintenance warnings. Clean package/extracted tests and dry-run precede exact-commit Linux CI. After publication, the registry consumer, downloaded bytes, VCS metadata, source tag and release-asset checksums are verified. Pre-fix regression evidence and all raw results remain retained locally.


## Measured cost

The reported comparison uses 36 fresh-process runs: three samples per version, mode and new document, with alternating order and the same thin-LTO/codegen1 release profiles on Rust/macOS ARM64. Input/file caches are warm. Deliberate validation/build/fuzz work completed first; other desktop activity is not controlled. Per-child `wait4` records peak RSS. A separate earlier 36-run set overlaps strict documentation checks and remains private; it is not used for this table.

| Input | Pages | Default 0.3.6 | Default 0.3.7 | Verbose 0.3.6 | Verbose 0.3.7 |
| --- | ---: | ---: | ---: | ---: | ---: |
| GCC | 1,261 | 3.066 s | 3.091 s | 4.775 s | 3.066 s |
| GDB | 1,008 | 2.341 s | 2.625 s | 4.488 s | 3.158 s |
| PGF/TikZ | 1,324 | 4.050 s | 4.156 s | 7.504 s | 4.833 s |

Verbose median wall time falls by about 30–36% in these three cases. Every paired complete JSON output and displayed preview is identical. Ordinary-mode medians vary, including higher values for 0.3.7 in this sample; these three-sample desktop measurements do not establish a general default-mode performance change. No universal speedup, peak-RAM reduction or hard resource guarantee is claimed. [Aggregate timing/RSS and binary identities](measurements/0.3.7/paired-cli.json) and all raw local runs remain retained.
