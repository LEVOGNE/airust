# Reproduce AIRust 0.3.4 checks

Run `./test.sh` with Rust 1.92+ and Python 3.11+. Formatting is scoped to the owned `airust` package. The checked-in backend remains byte-for-byte at its maintained source state; `python3 scripts/check_pdf_backend.py` checks source provenance, licenses and formatting of its helper/tests using metadata for the current host.

Run backend regressions separately with `cargo test --manifest-path crates/airust-pdf-backend/Cargo.toml --test regressions --locked`. Its independent CI also verifies package extraction, opt-in C API compilation and Rust 1.89. Root feature gates cover AIRust's supported features, not every unrelated optional upstream binding/rendering mode.

The independent consumer.rs/consumer.toml application pins registry AIRust exactly and runs on Rust 1.89. Use a path override before publication, then remove it to check the registry artifact. Exact-commit CI, package/extracted tests, dry-run and checksum/VCS/tag/asset verification supplement the suite. The backend has a separate published checksum/tag/source commit, recorded in measurements/0.3.4/backend.json.

Private corpus inputs, canonical indexes, old/new executables, raw-page comparisons, paired repetitions and mutation data are retained in local-lab/pdf-stability. Public [measurements](../measurements/0.3.4/) contain aggregates only. See [parser contract](../PDF-PARSER.md), [worker supervision](../PDF-WORKERS.md) and the existing [release workflow](../validation-0.3.0/README.md). No Docker execution or independent security certification is claimed.
