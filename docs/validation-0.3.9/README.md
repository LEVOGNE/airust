# Reproduce AIRust 0.3.9 validation

Run `./test.sh` with Rust 1.92+ and Python 3.11+. Provider-neutral features use Rust 1.89; local ONNX features need 1.92. Format only with `cargo fmt --package airust`. Run `python3 scripts/check_pdf_backend.py` and the ten backend regressions separately.

Three new unit tests compare monotonic windows across 2,048 generated Unicode sequences with standard string trimming, check a high-overlap whitespace-heavy case and compare UTF-8 quota accounting/errors with the existing string-budget route. Existing generated chunk-fidelity and owned-PDF cases remain enabled.

The private differential harness compares registry 0.3.8 and current 0.3.9 with the same unchanged backend and root-pinned transitive dependencies. It compares complete exports and fixed BM25/TF-IDF rank lists, and verifies every current chunk against its source page. Raw-mode checks, difficult formats, malformed inputs, actual worker deadlines and direct owned-child termination supplement these tests. Independent MuPDF extraction, selected pypdf probes and rendered samples check new official sources.

The synthetic chunk-cost benchmark appends the same test to preserved 0.3.8/current source copies without changing their production algorithms. Both use the same backend and dependency versions. It measures owned text inside the chunker; it does not measure native parsing. Output counts/fingerprints and real-document SHA identities are recorded separately. Preliminary and final measurements remain retained.

An exact-version Rust 1.89 consumer exercises existing preview/import/chunk APIs locally before publication and from registry-only sources afterward. Patch API checks report tool skips. Fresh main/backend audits retain maintenance warnings. Clean packaging, extracted tests, dry-run, exact-commit Linux CI and registry/download/VCS/tag/asset identity complete the release gates.

All old knowledge, originals, independent references, fixtures, raw results and binaries remain in ignored local-lab/pdf-stability. Public [aggregates](../measurements/0.3.9/) exclude private document text and paths. See [API/commands/limits](../PDF-OVERLAP.md). No Docker execution, exhaustive security proof, OCR/password API or DenkMaps/MCP integration is claimed.
