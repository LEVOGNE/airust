# Reproduce AIRust 0.3.10 validation

Run `./test.sh` with Rust 1.92+ and Python 3.11+. Provider-neutral features use Rust 1.89; local ONNX features need 1.92. Scope formatting to `cargo fmt --package airust`. Separately run `python3 scripts/check_pdf_backend.py` and the backend's ten regression tests.

Three new regressions cover all seven Latin presentation ligatures, composition after control filtering/dehyphenation, numeric boundaries, page-number controls and deliberately limited compatibility folding. The pre-fix test fails on 0.3.9 and is retained privately.

The pinned differential harness links registry 0.3.9 and current 0.3.10 against the same unchanged backend and root-pinned transitive versions. Every page's cleaned text is checked by an independent Python/Unicode-regex oracle; 4,096 generated flag/string cases supplement the corpus. Use the bundled Python runtime matching the privately installed regex extension. An initial system-Python ABI mismatch was retained separately and corrected before results were collected.

Both full indexes, changed-page evidence, fixed BM25/TF-IDF probes and all originals remain retained. Independent MuPDF extraction, selected pypdf probes, six rendered samples and separate source-excerpt checks cover new official references. Raw-mode and authored-format checks retain their explicit scope. Helper CPU/RSS measurements run each version in separate fresh processes; these are supplied-text tests, not native parsing timings.

An independent exact-version Rust 1.89 consumer exercises actual ligature cleaning, preview/import/chunk APIs locally before publication and from registry-only sources afterward. Patch compatibility reports tool skips; current lockfile audits retain maintenance warnings. Clean packaging, extracted tests, dry-run, exact-commit Linux CI and registry/download/VCS/tag/asset identity complete the release gates.

All knowledge, originals, references, fixtures, changed-page evidence, measurements and binaries remain in ignored local-lab/pdf-stability. Public [aggregates](../measurements/0.3.10/) exclude private text and paths. See [API/commands/limits](../PDF-CLEANING-PASS.md). No Docker execution, exhaustive security proof, OCR/password API or DenkMaps/MCP integration is claimed.


For the separate helper benchmark, repeat these UTF-8 patterns up to 1 MiB and 16 MiB budgets: `ASCII text without ligatures.\n`, `A ﬀ ﬁ ﬂ ﬃ ﬄ with text.\n`, and `A\0\u{301} e\u{301} ﬃ\ttext\r\n`. Each version/case uses three fresh alternating processes. Disable dehyphenation and page-number removal. Time only the `clean_text` call; measure per-process RSS separately and compare output SHA-256. These cost inputs exclude the two newly corrected st forms. The independent generated-case oracle checks that correction separately.
