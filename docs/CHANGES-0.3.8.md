# AIRust 0.3.8: Preserve PDF text when making knowledge chunks

Sentence segmentation previously trimmed existing separators and inserted a space. Paragraph segmentation could remove indentation/blank lines and append new blank lines. On long pages this created chunk text that was not a contiguous excerpt of the extracted/cleaned source, including with cleaning disabled. The pre-fix unit regression demonstrates the changed characters directly.

Segmentation now preserves the input characters and uses boundaries only to select cuts. Existing page/chunk outer trimming, maximum character limits, overlap and budget checks remain. Public methods and JSON fields stay compatible; the optional backend remains exactly 0.3.4. Long-page text, counts, chunk IDs and rankings can change. Keep old exports and rebuild derived indexes explicitly. See [API, commands, upgrade advice and limits](PDF-CHUNKS.md).

## Validation scope

The expanded corpus contains 27 real documents and 32,796 physical pages. The three new official references are Glibc (1,290 pages), Emacs Lisp (1,596) and the FreeBSD Handbook (996). Sources are identified by tested snapshot URLs/hashes, without claiming that mutable documentation represents a particular latest stable release. A failed GNU response containing HTML is retained as a failed download and excluded from the corpus; a valid official Glibc snapshot is used instead.

The fresh current corpus has 120,489 entries (previous-version imports of the same 27 sources have 120,504). All current chunks are contiguous source excerpts, compared with 47,031 non-contiguous previous chunks. Changes affect 16,765 pages. All previously populated pages remain present. Across 236 fixed literal probes, 74 rank lists change, with no gained or lost expected-page top-ten hit. The old 105,647-entry canonical export remains byte-identical; current chunks are stored separately.

The differential harness compares registry 0.3.7 and current 0.3.8 with the same backend and root-pinned transitive dependencies. It checks every current default chunk against its extracted/cleaned page, records changed pages and retains both indexes. It compares fixed BM25/TF-IDF expected-page probes. Independent MuPDF extraction, selected pypdf probes and rendered samples check new sources. These are source-excerpt and page-coverage checks, not universal PDF reading-order, glyph-coordinate or semantic guarantees.

All 14,891 raw-mode chunks from the three new references also match contiguous native excerpts; previously 10,481 did not. Another 6,075 current chunks from six successful authored format cases match their source pages. The one new-document page without chunks is also empty in independent MuPDF extraction. All six selected source-page renderings were visually checked.

Three new unit regressions include 2,048 deterministic Unicode/whitespace inputs across split flags and overlap settings. They verify segmentation conservation, contiguous bounded chunks and preservation of non-whitespace content with zero overlap. An owned-PDF integration covers raw and cleaned long-page chunks against extracted text. Malformed-input campaigns, difficult formats, a real worker deadline and owned-child termination supplement these checks. The original 4,096 mutations run in ordinary and verbose modes: each has 2,585 accepted inputs and 1,511 controlled errors, with case-by-case classification unchanged from 0.3.7. A further 4,096 mutations of an owned long-page fixture exercise the corrected chunk path: 2,762 are accepted and 1,334 return controlled errors. Across all 12,288 executions there are zero caught parser panics, abnormal parent exits, worker deadlines or outer watchdog timeouts; every failed import preserves prior output. Protected and image-only authored cases still fail normally. A one-second real worker deadline and direct owned-worker SIGKILL preserve output and reap the child.

## Knowledge and limits

All original PDFs, earlier canonical exports, independent references, changed-page evidence, authored/damaged fixtures, binaries and raw measurements remain retained locally. The current canonical corpus is a separate fresh import, never a mixture of old/new chunk revisions. Authored and damaged inputs are excluded from canonical knowledge. Public evidence excludes private document text and source paths.

A contiguous chunk is still an excerpt of native extracted and optionally cleaned text. Cleaning can alter meaningful hyphens, numeric edge data and whitespace; use the documented controls for technical material. Chunk edges still trim whitespace; hard caps can split words and overlap repeats text. Native opening/extraction has no hard library CPU/RAM deadline. Worker supervision has explicit limits. No OCR/password-entry API, Docker execution, exhaustive fuzzing, security certification, DenkMaps/MCP integration or 0.4.0 expansion is claimed.

## Release gates

The complete suite passes 197 default and 702 all-feature tests, compiled examples, five Python tests, all 37 isolated feature entries and strict lint/docs. Final README/module doctests pass all 32 cases. Rust 1.89/1.92, 229 patch API checks against 0.3.7 (31 tool skips) and the independent local Rust 1.89 consumer pass. Backend provenance and ten regressions pass. Main/backend audits report zero known vulnerabilities, with three/four unmaintained-dependency warnings retained. These scans are not security certification.

The [reproduction guide](validation-0.3.8/README.md) and [aggregate evidence](measurements/0.3.8/) describe the executed checks. Gates include full tests/examples, strict lint/docs, all isolated features, unchanged backend provenance/regressions, Rust 1.89/1.92, patch compatibility against 0.3.7 and an independent exact-version consumer. Maintenance warnings remain visible in current audits. Clean packaging, extracted tests and dry-run precede exact-commit Linux CI. Publication is followed by a registry-only consumer and downloaded-byte/VCS/tag/release-asset checksum verification.


## Measured import cost

The 24 matched CLI runs use three default samples and one no-clean sample per version and new document. Fresh children run in alternating order with the same thin-LTO/codegen1 Rust/macOS ARM64 profiles and warm input/file caches. Deliberate build/validation/fuzz work completed first; other desktop activity is not controlled. Per-child `wait4` records peak RSS. Every CLI export matches the same version/mode API export.

| Input | Pages | 0.3.7 median | 0.3.8 median | 0.3.7 RSS | 0.3.8 RSS |
| --- | ---: | ---: | ---: | ---: | ---: |
| Glibc | 1,290 | 2.728 s | 2.662 s | 137.48 MiB | 138.14 MiB |
| Emacs Lisp | 1,596 | 3.085 s | 3.024 s | 143.44 MiB | 146.11 MiB |
| FreeBSD | 996 | 1.066 s | 1.072 s | 128.95 MiB | 131.44 MiB |

Corrected chunks intentionally produce different output, so these are import-cost observations rather than a same-output speedup. Three samples and uncontrolled desktop activity do not establish universal time/RAM bounds or a default-mode performance guarantee. All [timing/RSS aggregates and binary identities](measurements/0.3.8/paired-cli.json), individual raw runs and both versions' exports remain retained.
