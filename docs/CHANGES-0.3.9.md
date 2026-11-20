# AIRust 0.3.9: Reduce overlap-processing waste

PDF chunking previously allocated a temporary string for every candidate window, including windows containing only whitespace, before trimming and checking budgets. Accepted text was copied again into the retained chunk. Strong overlap could repeatedly scan/copy the same whitespace. Version 0.3.9 carries trimmed-edge positions forward and checks quotas before retaining text. A conservative UTF-8 bound handles the common budget case; tighter budgets use exact byte counting.

The internal change preserves existing public methods/configuration, boundary choices, outer trimming and JSON fields. The separately published optional backend stays exactly 0.3.4. No default-core dependency, DenkMaps/MCP integration or 0.4.0 expansion is added. See [API, commands, compatibility and limits](PDF-OVERLAP.md).

## Validation scope

The new R reference (3,995 pages), Octave manual (1,262) and NumPy reference (2,101) extend the corpus to 30 documents / 40,154 physical pages. All 142,482 default chunks are contiguous extracted/cleaned source excerpts. Full exports and all 267 fixed literal probes retain identical ranked IDs between versions. The separate raw-mode check verifies 22,059 chunks from the three new sources with identical exports/rankings. The 32 new-document pages without chunks are also empty in independent MuPDF extraction. Six selected source renderings were visually checked. Tested snapshot URLs/hashes identify the actual bytes, without claiming that mutable URLs represent a particular latest stable revision. Independent MuPDF extraction, selected pypdf probes and rendered samples supplement native extraction.

The root-pinned differential harness compares registry 0.3.8 and current 0.3.9 with the same backend. It compares complete imports and fixed BM25/TF-IDF rankings and checks current chunks against extracted/cleaned page text. The current combined export appends 21,993 new entries. The previous 120,489 entries match fresh current imports exactly; that older canonical file and all earlier exports remain byte-identical. The current library reopens the new export and returns ten bounded hits. Original PDF hashes remain unchanged. Authored and damaged fixtures are excluded from canonical knowledge; public evidence excludes private text and source paths.

Three new unit regressions cover Unicode trimming equivalence, high-overlap whitespace and exact UTF-8/count-budget behavior, including failed reservations. Existing fidelity tests remain enabled. Synthetic extracted-text benchmarks and actual PDF CLI timings are reported separately. Early literal-space PDF experiments did not retain the intended whitespace through native extraction and are excluded from the synthetic performance claim.

The full suite passes 197 default and 705 all-feature tests, compiled examples, five Python tests, all 37 isolated feature entries and strict lint/docs. After the final README update, all 32 doctests and strict default/all-feature documentation pass. The ten backend regressions and provenance check pass. Rust 1.89/1.92, 229 patch API checks against 0.3.8 (31 tool skips) and the independent local consumer pass. Current main/backend audits find zero known vulnerabilities, retaining three/four maintenance warnings; these scans are not security certification.

The final malformed-input campaigns run 12,288 executions. The ordinary and verbose base campaigns each accept 2,585 inputs and return 1,511 controlled errors. The long-page campaign accepts 2,762 and rejects 1,334 normally. All classifications match 0.3.8 case by case; all 7,932 successful output files are SHA-256 identical. There are zero caught parser panics, abnormal parent exits, worker deadlines or outer timeouts. Every failed import preserves old output. Eight authored format outcomes and successful bytes match 0.3.8. A real one-second deadline and verified direct worker SIGKILL preserve prior output and reap the child. Initial-candidate campaigns remain retained separately.

The [reproduction guide](validation-0.3.9/README.md) and [aggregate evidence](measurements/0.3.9/) record full tests, strict lint/docs, isolated features, backend provenance/regressions, minimum Rust versions, patch compatibility, independent consumers, current audits, packaging, extracted tests, dry-run, exact-commit CI and post-publication artifact identity. Maintenance warnings remain visible.

## Limits

High overlap still repeats nonempty text and consumes chunk/text quotas. Full-page preparation, native opening/extraction, cleanup, index construction and serialization retain their costs. Synthetic chunker results are not whole-PDF speedups or hard CPU/RAM bounds. Source-excerpt fidelity does not guarantee native reading order, original glyph coordinates or perfect cleaning. Worker supervision provides a wall-clock budget, not an OS memory sandbox. No OCR/password-entry API, Docker execution, exhaustive fuzzing or security certification is claimed.


## Real PDF import cost

The 24 matched CLI runs use three default samples and one no-clean sample per version and new document. Fresh children alternate order with the same thin-LTO/codegen1 Rust/macOS ARM64 profiles and warm input/file caches. Deliberate validation/build/fuzz work completed first; other desktop activity is not controlled. Per-child `wait4` records peak RSS. Every output matches its same-version/mode API export, and paired complete JSON bytes are identical.

| Input | Pages | 0.3.8 median | 0.3.9 median | 0.3.8 RSS | 0.3.9 RSS |
| --- | ---: | ---: | ---: | ---: | ---: |
| R reference | 3,995 | 5.702 s | 5.684 s | 287.95 MiB | 290.23 MiB |
| Octave | 1,262 | 2.068 s | 2.054 s | 138.08 MiB | 139.30 MiB |
| NumPy | 2,101 | 4.177 s | 4.205 s | 150.27 MiB | 152.42 MiB |

Whole-import medians remain close in this sample; observed RSS is somewhat higher for 0.3.9. These three-sample desktop measurements do not establish a universal time/RAM reduction or regression. All [aggregates and binary identities](measurements/0.3.9/paired-cli.json), raw runs and exports remain retained.

## Synthetic extracted-text stress

The same benchmark test was appended to preserved 0.3.8/current source copies, leaving their production algorithms unchanged and sharing the verified backend/transitive versions. Six fresh processes alternate version order, with three samples per workload. Timing is inside the chunker and excludes input construction and output hashing. This does not invoke native PDF parsing. Output counts and same-toolchain `DefaultHasher` fingerprints agree; these fingerprints are not cryptographic identities. Real-document exports are checked separately by SHA-256.

| Workload | Unicode scalars | Chunks | 0.3.8 median | 0.3.9 median |
| --- | ---: | ---: | ---: | ---: |
| dense | 1,100,000 | 1,375 | 2.245 ms | 2.748 ms |
| whitespace | 1,000,002 | 2 | 1604.566 ms | 6.855 ms |

The whitespace-heavy high-overlap case falls from about 1.60 seconds to 6.86 milliseconds in this sample. Dense-text samples include higher medians for 0.3.9; the change targets repeated whitespace work and unnecessary copies rather than guaranteeing improvement for every input. The [complete workload results](measurements/0.3.9/chunk-cost.json) retain all sizes, conditions and binary identities. These synthetic results are not whole-PDF speedups, a claim about every input or hard CPU/RAM bounds. Earlier experiments and the initially slower dense-data candidate remain retained privately.
