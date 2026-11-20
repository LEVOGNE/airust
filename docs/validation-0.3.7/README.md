# Reproduce AIRust 0.3.7 validation

Run `./test.sh` with Rust 1.92+ and Python 3.11+. The provider-neutral MSRV is 1.89; optional local ONNX checks use 1.92. Use `cargo fmt --package airust` to avoid changing vendor source. Run `python3 scripts/check_pdf_backend.py` and the backend's ten `regressions` tests separately.

Four new integration regressions cover preview prefixes, sparse pages and physical positions, raw versus cleaned text, whole-file/page and cumulative visited-text limits, invalid configuration and empty results. The verbose CLI regression first failed on 0.3.6: a 500-page owned fixture reported a later raw-text limit before its first-page chunk limit. It now reports the chunk limit and preserves the previous output.

The private differential harness compares registry 0.3.6 with current 0.3.7 using the same backend and root-pinned transitive versions. It checks first one/three/five usable pages in raw and cleaned modes against the previous full extractor, compares complete knowledge exports and fixed BM25/TF-IDF rankings, and retains all outputs. PyMuPDF scans every page of three new official references; selected pypdf pages supply literal probes. These are independent extraction/coverage checks, not human semantic judgments or universal extraction proof.

The exact-version consumer pins 0.3.7 on Rust 1.89. A local path override precedes publication; the final run uses registry sources only. Patch API checks compare 0.3.6 and report tool skips. Fresh main/backend lockfile audits retain maintenance warnings. Clean packaging, extracted tests, dry-run, exact-commit Linux CI and registry/VCS/tag/checksum/asset checks complete the release gates.

The unchanged 0.3.4 backend keeps its separate source/tag/checksum. All old knowledge, originals, authored/damaged fixtures, references, measurements and tested binaries stay in ignored `local-lab/pdf-stability`. Public [measurements](../measurements/0.3.7/) exclude private text and paths. See [API/commands/limits](../PDF-PREVIEW.md). No Docker execution, exhaustive fuzzing, security certification, OCR/password API or DenkMaps/MCP integration is claimed.
