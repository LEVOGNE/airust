# AIRust 0.3.1 acceptance plan

Harden the existing optional PDF importer before considering new 0.4.0 document capabilities. Preserve valid-corpus content, core ranking behavior and public APIs.

- Reproduce concurrent-output data loss, empty conversion and malformed-header parser panic using isolated owned fixtures.
- Fix publication races, refuse unusable conversion and contain parser Rust unwinding panics without changing global panic hooks.
- Compare the same baseline/candidate harness on all supplied manuals and an official multi-thousand-page PDF. Retain private knowledge and raw evidence locally; publish aggregate results only.
- Recheck deterministic sorting, top-K against a full ordering, page probes from a separate parser, and mutation outcomes. Treat unexpected positive exit codes, especially Rust panic 101, as abnormal.
- Profile slow extraction separately from chunking, index build and search. Disclose unresolved dependency paths and the limits of in-process containment.
- Run the full suite, each feature, strict Clippy/rustdoc, Rust 1.89/1.92, patch API comparison, audit, standalone consumer, extracted-package tests and publication dry-run.
- Require exact-commit CI before GitHub/crates.io publication. Verify registry consumer, checksum, VCS commit, tag and release asset afterward.

No DenkMaps or MCP integration. No arbitrary release-count target or claim of exhaustive security certification.
