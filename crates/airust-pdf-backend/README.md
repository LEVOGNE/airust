# AIRust PDF backend 0.3.4

This optional internal backend supports AIRust PDF imports. It is an independently maintained derivative of pdf_oxide 0.3.78, not the official PDFOxide release. It preserves the complete structure-tree path and adds a bounded fast path for small objects on long physical lines. It also fixes an invalid UTF-8 boundary check in object-header detection.

Use the public `airust` crate and its `pdf` feature for the knowledge-engine API. This backend does not add a model, server, database, DenkMaps adapter or MCP integration. It is enabled only for PDF processing.

The patch checks at most 4 KiB for a complete nonstream object and a separate endobj token using the existing parser. Streams, incomplete probes and larger objects use the existing loader. No input bytes, source files or structural ordering are changed. See UPSTREAM.md and upstream.patch for provenance and exact code changes. Malformed/unusually large objects can still use the slower fallback; importer budgets and supervised jobs remain necessary.

Original licenses and notices are preserved: MIT OR Apache-2.0. Original authors retain their copyright. This fork is neither endorsed by nor affiliated with the upstream project. Native regression tests and real-document comparisons are maintained for AIRust's supported default backend configuration. Upstream's full fixture-dependent test suite is not included in the published upstream crate and is not claimed as executed here.
