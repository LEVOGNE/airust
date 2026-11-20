# AIRust 0.3.2: Explicit PDF budgets and safe CLI input

This stabilization step adds opt-in input, page, text and chunk limits to the existing PDF importer. It prevents overlap-amplified chunk allocation from exceeding the chosen output budget and returns errors rather than partial knowledge. Existing constructors/default features and successful corpus contents retain their behavior. See [API and limits](PDF-LIMITS.md).

A separate invalid-byte CLI argument probe exposed an uncaught env::args panic. The converter now handles OS paths as OsString and rejects invalid Unicode numeric options normally. A harmless captured color-sequence probe also showed raw terminal controls in file-path errors; previews and converter-generated messages now escape terminal/directional controls without changing I/O paths or retained knowledge text.

Limits are policy bounds, not hard native-parser CPU or RAM quotas. The known slow first-page path remains under investigation. No DenkMaps or MCP integration is added, and 0.4.0 functionality remains deferred.

## Validation

The fifteen real input PDFs contain 8,960 physical pages. Reimporting them with PdfImportLimits::default produces 33,708 chunks. Their successful content hashes, page sets, chunk counts and checked rankings match the 0.3.1 baseline. The same literal page probes and bounded mutation campaign are repeated; exact totals and results are in measurements/0.3.2. User PDF text, queries and indexes remain in the ignored local lab.

The final source passes the complete test script: 197 default tests, 675 all-feature tests, all 37 isolated feature entries, compiled examples, five Python tests and strict Clippy/rustdoc. Rust 1.89/1.92, 229 patch API checks against 0.3.1 (31 skipped by the tool), a refreshed audit and a standalone consumer also pass. Clean package/extracted tests and publication dry-run are required before publishing. Exact-commit CI precedes publication, followed by registry consumer and checksum/VCS/tag/asset verification.

The local lab is additionally excluded from Docker context upload. No Docker image execution is claimed because Docker is unavailable. This is a stability/robustness campaign, not a guarantee of flawless parsing or an independent penetration audit.
