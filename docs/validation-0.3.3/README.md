# Reproduce 0.3.3 checks

Run `./test.sh` with Rust 1.92+ and Python 3.11+. In addition to the existing 0.3.2 boundaries, tests exercise supervised conversion, process reaping, nonzero exits, bounded diagnostics, result rejection, settings forwarding, concurrent writers and OS path handling. Raw-byte filename creation depends on filesystem support; unsupported filesystems exercise a clean CLI error.

The independent Rust 1.89 consumer in consumer.rs/consumer.toml pins the exact release and checks the public library APIs, including PDF limits. Use a path override for pre-publication validation and the exact registry dependency afterward. CLI behavior is also executed in extracted-package tests. Other gates follow the [release workflow](../validation-0.3.0/README.md).

The [worker contract](../PDF-WORKERS.md) gives CLI commands and supervision limits. Public [measurements](../measurements/0.3.3/) contain aggregates from the fifteen-document worker pass, deadline case, malformed-input campaign and combined real-PDF index soak. Private indexes, texts, queries, raw logs and exact release sources remain in the ignored local-lab/pdf-stability directory. No Docker execution or independent security certification is claimed.
