# AIRust 0.4.0: Document revisions and reliable text positions

A text range can be within bounds and still refer to the wrong version. Two documents may share a filename; a changed source or cleaned text may have the same length as its predecessor. AIRust now supplies an optional document contract that binds each passage to one exact source/text/recipe/page revision. Resolving against a different revision fails before returning text.

The new `documents` feature adds streaming source fingerprints, immutable shared text snapshots, explicit physical page spans, checked UTF-8 passages, bounded version-one JSON and existing-provenance composition. It enables only provenance and SHA-256 alongside the core. PDF parsing, retrieval, storage, models and application transports remain independent options. See [API, defaults, interoperability and upgrade](DOCUMENTS.md).

## What remains compatible

Core search, matching, graph, storage, model adapters and existing knowledge formats retain their signatures. Patch-level API checking against 0.3.15 passes 229 checks with 31 tool skips, even though Cargo deliberately treats 0.4 as a new compatibility range. Applications update the version constraint explicitly. No automatic reindexing, document migration or knowledge deletion occurs.

PDF/parser/cleanup/chunk/worker source is unchanged from 0.3.15. Backend source remains exactly 0.3.5, including its independent source tag, notices and 381-file provenance check. Earlier native glyph/reference, chunk-output, damaged-input containment and core memory/soak evidence is inherited only through those unchanged modules. The fresh 0.4 campaign below tests native page extraction plus the new document contract, not another independent OCR or complete glyph audit.

## Executed validation

All 202 default tests, 758 all-feature tests, 35 documentation examples, 38 isolated feature entries and five Python tests pass. Strict lint/docs, Rust 1.89/1.92 and the independent public-API consumer pass. The document feature alone runs 23 tests under Rust 1.89. Fourteen unchanged-backend regressions and provenance checks pass. Current dependency audits find zero known vulnerabilities, retaining three main and four standalone-backend maintenance warnings.

The 26 document tests include same-basename separation, changed source/recipe/text/page-map identity, repeated occurrences, every possible UTF-8 range over small multilingual fixtures, exact JSON byte limits including escape expansion, page partition corruption, unsupported/duplicate/unknown fields, source reader errors, retained clones/readers, retrieval projection and staged dataset CAS. A standard-library Python calculation independently pins version-one revision and passage IDs.

The deterministic 4,096-edit manifest test checks UTF-8 validity before restoration. Validly encoded candidates must either fail or restore the exact original revision, text and page map. The retained snapshot continues to resolve throughout. This is a bounded property campaign, not exhaustive protocol or security certification.

The initial configuration construction and external-consumer manifest issues were corrected before acceptance. All failed setup/compiler logs remain retained; they are not presented as PDF failures. No confirmed new native text-loss defect was found or claimed in this release.

## Fresh real-document checks

All 35 retained PDFs were freshly extracted with bounded native limits. They cover 49,086 physical pages and yield 48,983 provided native text pages. The difference is pages without usable native text; this release does not re-establish independent all-page glyph/OCR coverage. `from_pages` preserves the provided text and explicitly inserts two LF bytes between page spans.

Each full-page passage is checked against the stored snapshot, physical page label, provenance span, entry metadata and restored passage JSON. Three selected relative UTF-8 ranges per page add 146,949 subspan checks. Complete snapshots are exported and restored with equal revisions, text and page maps. Changed-source candidates reject old passages while retained original snapshots continue to resolve. Ten thousand clones per document share the same text buffer.

Across 632 BM25/TF-IDF literal probes, complete ranked IDs, score bits and matched terms match published 0.3.15 over identical new page entries. Every returned entry has its revision/range metadata revalidated. This establishes core compatibility on that workload. Whole-page entries have a different granularity from the old 1,000-character chunks, so it does not promise identical rankings between those different corpora, better semantics or human relevance. The fixed private reference terms are extraction probes, not relevance judgments.

The old canonical export remains byte-identical: 167,942 entries, SHA-256 `13168d87872a137355fd8499c90a6084f52243a2ec7456f29b5fcb2a1c7423c3`. All eight checkpoints and original PDF hashes remain unchanged. New revisioned snapshots, page entries, query checks, raw measurements and binaries are additionally retained under ignored `local-lab/pdf-stability/document-snapshots/040`. No first-occurrence guess is used to assign positions to old records.

The legacy corpus consumer parses all 167,942 records and then actually runs a 1,000-entry prefix for one checked epoch: updates agree with rebuilds, JSON/concurrent signatures match, retained readers remain stable and cancellation/deadline checks pass. Larger previous core workloads remain inherited; this is not a fresh full-corpus stress or model-quality claim.

## Measured snapshot reuse

Twelve fresh release processes cover four retained snapshots, with three repeats per case and alternating case order. Rust 1.98 on macOS ARM64 uses Cargo's default release profile. Deliberate builds, native validation, audits and other checks finish before these measurements; file/input caches are warm and unrelated desktop activity is uncontrolled.

The restore timer starts after reading encoded input and includes JSON decoding, bounds/page validation and text/revision hashing. Position preparation includes all page references, three subspans per page, passage JSON and entry creation/validation. Lexical build includes copying the entries into the current index. The complete process additionally exports/restores data, builds both SDK indexes, compares queries and retains several working buffers.

| Case | Physical / provided pages | Checked restore, median ms | Positions and entries, ms | Current lexical build, ms | Complete process, seconds | Complete process peak, MiB |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| vkspec | 5,491 / 5,491 | 27.631 | 28.249 | 470.377 | 2.221 | 493.188 |
| scipy | 3,582 / 3,581 | 15.365 | 16.334 | 275.607 | 1.511 | 351.266 |
| racket-reference | 1,438 / 1,438 | 5.980 | 6.391 | 103.794 | 0.624 | 129.516 |
| pdf-12 | 494 / 494 | 2.547 | 2.454 | 47.224 | 0.214 | 62.594 |

This workload performs no native PDF extraction during restore. The complete process peak includes both 0.4.0 and 0.3.15 indexes, immutable snapshots and restored copies, selected-entry vectors, serialized JSON and round-trip verification. It is not the RAM required by one production snapshot or index. Production callers can retain one snapshot and shared clones rather than all verification views. `to_entry` still copies its selected text into the owning lexical engine.

These are absolute operation costs with complete raw samples, hashes and conditions retained. Three warm samples do not establish p95/p99, a cold-start bound, a universal speedup or every-host capacity. Earlier under-load native-validation timings remain separate from this quiet restore campaign.

## Boundaries and publication

Positions are exclusive-end UTF-8 byte ranges in the exact complete text snapshot. A range is not a grapheme index, printed page label, original PDF byte offset or drawing coordinate. Cross-page/separator passages have no invented page. Text-space labels and hashes do not prove extraction coverage, authenticity or semantic truth. Restored source descriptors should be compared with independently trusted original bytes when source verification is required.

Encoded/decoded source, text, page and JSON budgets are checked, but are not OS RAM/CPU isolation or blocking-call preemption. Applications own persistence transactions, access rules and retention of the matching snapshot revision. The feature introduces no second authoritative store and no automatic original-to-cleaned coordinate mapping.

Clean packaging, extracted-package tests, dry-run, exact-commit Linux CI, actual registry consumers and matching download/VCS/tag/release assets gate publication. Final outcomes and immutable artifact identity belong to the release verification asset. No Docker execution, new required model, DenkMaps or MCP integration is claimed. See [public aggregates](measurements/0.4.0/), [reproduction](validation-0.4.0/README.md), [complete API](DOCUMENTS.md) and [API cookbook](API-COOKBOOK.md).
