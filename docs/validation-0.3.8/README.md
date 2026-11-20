# Reproduce AIRust 0.3.8 validation

Run `./test.sh` with Rust 1.92+ and Python 3.11+. Provider-neutral features use Rust 1.89; local ONNX features require 1.92. Scope formatting to `cargo fmt --package airust`. Separately run `python3 scripts/check_pdf_backend.py` and the backend regression target.

Three new unit tests check exact source reconstruction by segmentation, contiguous bounded chunks and 2,048 deterministic generated Unicode/whitespace cases across split flags and overlap settings. A new owned-PDF integration checks raw and cleaned long-page chunks against their extracted source. The pre-fix segmentation test demonstrably fails on 0.3.7 and its output is retained privately.

The private harness compares registry 0.3.7 and current 0.3.8 with the same backend and root-pinned transitive dependencies. It checks every current chunk against extracted/cleaned source text, records changed pages, preserves both full indexes and compares fixed BM25/TF-IDF page probes. Independent MuPDF extraction, selected pypdf probes and rendered samples cover new official sources. These are fidelity/coverage checks rather than universal reading-order or semantic guarantees.

The independent consumer pins 0.3.8 on Rust 1.89, using a local path before publication and registry-only sources afterward. It exercises owned-PDF preview/import and the existing APIs. Patch compatibility, current lockfile audits, clean packaging, extracted tests, dry-run, exact-commit Linux CI and registry/download/VCS/tag/asset checksum checks complete the release gates. Maintenance warnings remain visible.

The separately published optional backend stays exactly 0.3.4. All originals, older and current canonical knowledge, authored/damaged fixtures, changed-page evidence, binaries and measurements remain in ignored local-lab/pdf-stability. Public [aggregates](../measurements/0.3.8/) exclude private document text and source paths. See [API, migration and limits](../PDF-CHUNKS.md). No Docker execution, exhaustive security proof, OCR/password API or DenkMaps/MCP integration is claimed.
