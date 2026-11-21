# AIRust PDF backend 0.3.5

This optional internal backend supports AIRust PDF imports. It is an independently maintained derivative of pdf_oxide 0.3.78, not the official PDFOxide release. It preserves the complete structure-tree path and adds a bounded fast path for small objects on long physical lines. It also fixes an invalid UTF-8 boundary check in object-header detection.

Use the public `airust` crate and its `pdf` feature for the knowledge-engine API. This backend does not add a model, server, database, DenkMaps adapter or MCP integration. It is enabled only for PDF processing.

The patch checks at most 4 KiB for a complete nonstream object and a separate endobj token using the existing parser. Streams, incomplete probes and larger objects use the existing loader. No input bytes, source files or structural ordering are changed. See UPSTREAM.md and upstream.patch for provenance and exact code changes. Malformed/unusually large objects can still use the slower fallback; importer budgets and supervised jobs remain necessary.

Original licenses and notices are preserved: MIT OR Apache-2.0. Original authors retain their copyright. This fork is neither endorsed by nor affiliated with the upstream project. Native regression tests and real-document comparisons are maintained for AIRust's supported default backend configuration. Upstream's full fixture-dependent test suite is not included in the published upstream crate and is not claimed as executed here.

The original C ABI module is opt-in through `c-api`; default Rust use exports no competing PDF C symbols. Linux differential linking against the original parser exposed duplicate C symbols before this gate. Rust extraction and structure parsing are unaffected.


Version 0.3.5 returns an extraction error when the existing Form XObject depth or processing budget prevents complete text extraction. It propagates the error through nested Forms after restoring local state and avoids retaining partial span results as successful cache entries. Ten nested Form levels and the existing 500-processing-attempt budget remain unchanged. A later call may finish using valid caches; every successful result must still contain the complete known fixture text. A rejected 64-level recursion prototype overflowed the default test-thread stack and was not adopted.

Fallback page counting now propagates depth exhaustion instead of returning a successful partial count. Other best-effort recovery behavior remains; this patch does not certify complete extraction from arbitrary damaged PDFs. Four additional owned regressions cover supported depths, excessive depth, repeated text/span/word/character extraction, processing-budget boundaries and fallback counts. Original licenses, notices and upstream source hashes remain preserved.
