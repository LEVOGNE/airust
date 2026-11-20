# Reproduce AIRust 0.3.6 validation

Run `./test.sh` with Rust 1.92+ and Python 3.11+. The default/provider-neutral MSRV is 1.89; full optional ONNX checks use 1.92. Formatting remains scoped to `cargo fmt --package airust`. The ten backend regressions and `python3 scripts/check_pdf_backend.py` verify the unchanged 0.3.4 parser independently.

New regressions cover numeric/paragraph boundaries, short wrap chains, Greek/Hangul/control composition, a 20,000-wrap chain, optional flags and internal numeric rows. The owned PDF regression verifies that `100` remains searchable and a false fused `1020` value is absent.

The private differential harness links registry 0.3.5 and current 0.3.6 to the same backend. Its transitive versions are checked against the root lockfile; the initial unconstrained lab lock is retained separately and was not used for the corpus results. Every extracted page is streamed to an independent Python/Unicode lookaround oracle. Raw changed-page evidence remains local. A separate 4,096-case string campaign covers all tested flag combinations. These are known-script/corpus checks rather than universal Unicode or semantic guarantees.

The exact-version independent consumer pins 0.3.6 on Rust 1.89. A local path override precedes publication; the final consumer uses registry sources only. Patch API checks compare 0.3.5, with tool skips reported. Fresh main/backend lockfile audits retain maintenance warnings. Clean packaging, extracted tests, dry-run, exact-commit CI and registry/VCS/tag/checksum/asset identity complete the release gates.

All old knowledge, sources, authored/damaged fixtures, raw measurements and tested binaries remain in ignored local-lab/pdf-stability. Public [measurements](../measurements/0.3.6/) exclude private document text and source paths. See [API/controls/migration/limits](../PDF-TEXT-FIDELITY.md). No Docker execution, exhaustive fuzzing, security certification, OCR/password API or application integration is claimed.
