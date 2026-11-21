# AIRust 0.3.10: Complete Latin-ligature cleaning with fewer text copies

The public cleaner previously expanded U+FB00–U+FB04 but left U+FB05/U+FB06 unchanged. The pre-fix regression demonstrates this with `firﬅ`. The limited Latin presentation-form policy now covers all seven forms, using plain `st` for the two missing cases. General compatibility folding of mathematical symbols and other scripts remains outside this step.

The initial NFC/ligature/control stage is now one pass instead of a chain of whole-text replacements. Later NFC composition still follows control removal and dehyphenation. Numeric boundaries, paragraph handling and optional page-number/hyphen controls remain. Public signatures and JSON fields are unchanged. Corrected text can alter derived chunks and ranking; retain old exports and rebuild derived data explicitly where affected. The optional backend stays exactly 0.3.4. See [API, commands and limits](PDF-CLEANING-PASS.md).

## Validation scope

The new Julia (2,108 pages), PostScript (912) and SciPy (3,582) references expand the corpus to 33 documents / 46,756 physical pages. All cleaned pages match the independent oracle. The 4,096 generated flag/string cases include 2,480 corrected outputs, exactly the cases containing the new st forms. Complete imports and all 296 fixed literal page-probe rank lists match 0.3.9 for this corpus; native extraction had already expanded the ligatures there. The SciPy PDF is an explicitly identified older reference, useful for format/scale coverage rather than a claim about the latest stable documentation. Every snapshot is identified by URL, byte count and hash. Independent MuPDF extraction, selected pypdf probes and rendered samples supplement native extraction.

The pinned registry/current differential harness compares cleaned pages, full imports and fixed BM25/TF-IDF rankings. An independent Python/Unicode-regex oracle checks every page and 4,096 generated flag/string cases. Source-excerpt checks, raw-mode and difficult authored formats supplement those checks. Native extraction may already expand ligatures; the corpus report records observed changes rather than assuming all documents change.

The combined corpus now contains 160,085 entries, adding 17,603 validated new-source entries. All previous 142,482 entries match fresh current imports exactly; that older canonical file and five earlier exports remain byte-identical. All original PDFs, current imports, references, fixtures and raw results remain retained locally. Authored/damaged test inputs stay outside canonical knowledge. Public evidence excludes private document text and source paths. Helper CPU/RSS tests use supplied owned text in separate processes and are reported separately from real PDF imports.

The [validation guide](validation-0.3.10/README.md) and [aggregates](measurements/0.3.10/) record full tests/examples, strict lint/docs, isolated features, backend provenance/regressions, minimum Rust versions, patch API checks, independent consumers, current audits, clean packaging/extracted tests, dry-run, exact-commit CI and post-publication artifact identity. Maintenance warnings remain visible.

## Test and failure checks

- 197 default tests and 708 all-feature tests pass, including three new regressions for the cleaner. The final check also passes 32 documentation examples.
- All 37 isolated feature checks, strict Clippy, default/all-feature rustdoc, five Python tests and ten maintained-backend regressions pass. Rust 1.89/1.92 checks and an independent local Rust 1.89 consumer pass. Patch API comparison against 0.3.9 passes 229 checks with 31 tool skips.
- Three deterministic malformed-input campaigns execute 12,288 conversions. Classifications match 0.3.9, all 7,932 successful outputs match by SHA-256, and failed conversions preserve prior output. No caught parser panic or outer/worker timeout was observed in these campaigns.
- An actual one-second deadline on a 2,488-page source and a separately killed owned worker preserve prior output and stop/reap the worker. Six valid authored-format cases match old output, including object streams, rotations, empty-user-password encryption and Unicode. Image-only and protected-password cases reject without replacing output.
- New-source checks verify 17,603 cleaned and 17,653 raw chunks as contiguous source excerpts. All 13 physical pages without chunks in the three new references are also empty in MuPDF. Earlier source-excerpt proof carries forward only because the 30 prior full exports match exactly.

Current lockfile audits find zero known vulnerabilities. They retain three main-lockfile and four standalone-backend maintenance warnings. This is a dated dependency check, not a security certification. The audit snapshots and mutation scope are included in the public aggregates.

The release gates also require clean packaging, tests from the extracted crate, a publish dry-run, exact-commit Linux CI, a registry-only consumer and matching crate/VCS/tag/asset checksums. Their final publication results are retained in the release verification asset rather than predicted before publication.

## Measured costs

Measurements ran locally on macOS ARM64 in fresh alternating child processes after intentional build/fuzz work finished. File caches were warm; other desktop activity was uncontrolled. Each default PDF median uses three runs per version; another paired raw run per source gives 24 CLI runs in total. All paired outputs match byte-for-byte and match their version/mode API export. RSS is the peak of each child process.

| Real PDF | Physical pages | Import seconds, 0.3.9 → 0.3.10 | Peak RSS MiB, 0.3.9 → 0.3.10 |
| --- | ---: | ---: | ---: |
| Julia | 2,108 | 3.455 → 3.459 | 116.72 → 117.12 |
| Postscript | 912 | 1.054 → 1.043 | 80.86 → 80.36 |
| Scipy | 3,582 | 7.260 → 7.122 | 353.33 → 352.78 |

These real imports show small mixed differences, not a universal PDF speed or RAM improvement. One current SciPy sample took 9.028 seconds; the reported 7.122-second value is its three-run median. All individual samples remain retained.

The separate 36-process helper test uses supplied repeated text at 1/16 MiB budgets. Both versions are linked in one release harness, called in separate fresh children with `dehyphenate=false` and `remove_page_numbers=false`. Method timing excludes input construction and output hashing. Process RSS includes input, output and runtime. Test strings deliberately use only the five previously supported ligatures, so all output SHA-256 pairs are equal; the two corrected forms are validated separately.

| Supplied text | Input budget MiB | Helper seconds, 0.3.9 → 0.3.10 | Process peak RSS MiB, 0.3.9 → 0.3.10 |
| --- | ---: | ---: | ---: |
| ascii | 1 | 0.016040 → 0.015244 | 15.19 → 11.16 |
| ascii | 16 | 0.245826 → 0.239548 | 153.30 → 73.22 |
| latin | 1 | 0.014409 → 0.012696 | 14.98 → 11.02 |
| latin | 16 | 0.237356 → 0.206617 | 144.69 → 68.86 |
| controls | 1 | 0.015815 → 0.014146 | 15.03 → 10.88 |
| controls | 16 | 0.248598 → 0.222897 | 156.89 → 65.75 |

All six helper cases have lower median time and peak RSS in this run. This supports the reduction of intermediate copying for these inputs. It does not measure global allocation counts, prove a universal reduction, bound standalone helper memory or establish a native PDF parsing speedup.

## Limits

Cleaning remains heuristic. Native reading order/glyph decoding can be imperfect; input sizes and configuration remain caller-controlled. The standalone cleaner has no quota or hard CPU/RAM limit. A supervised worker supplies a wall-clock budget, not an OS memory sandbox. No OCR/password-entry API, Docker execution, exhaustive fuzzing, security certification, DenkMaps/MCP integration or 0.4.0 expansion is claimed.
