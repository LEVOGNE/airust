# AIRust 0.2.30: Optional disk search index

a dedicated disk-backed full-text index with atomic revisions, scoped retrieval, bounded original-record export and integrity checks.

## Validation

The suite passes 194 default and 649 all-feature Rust tests, including documentation, plus five Python tests. Isolated feature checks, strict Clippy/rustdoc, Rust 1.89/1.92, patch compatibility against 0.2.29, audit, local/registry consumers, extracted-package tests, dry-run and exact-commit CI gate publication. One native-model test requires external assets and remains ignored.

See [API and limits](DISK-INDEX.md) and [measurements](measurements/0.2.30/). Dataset/model identities, explicit uncertainty and application-owned authorization remain part of the contract. Public measurements do not establish production quality or an independent security audit. No DenkMaps/MCP integration.

## Executed identical-record comparison

100,000 synthetic records were tested in separate processes with one release executable. Peak process RSS: memory 215.06 MiB, disk 8.03 MiB. Build times: 184.32/519.91 ms; query times: 1.297/1.452 ms. Disk reopen: 0.158 ms; database size: 19374080 bytes. The top ten IDs agree on this fixture; native scoring recipes differ. Disk uses less process RSS and a slower build here. OS filesystem cache is outside much of RSS; reopen is not guaranteed cold I/O. The disk child also performs the full integrity scan. This is not real application/native-model capacity validation. Raw outputs/resources and binary/source hashes are in measurements.

The inherited README test-count sentence was corrected to the executed 194 default / 649 all-feature totals. Earlier release-specific reports and CI logs retain their actual historical counts.
