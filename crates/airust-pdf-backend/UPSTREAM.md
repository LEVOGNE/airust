# Provenance and maintained patch

Base: published pdf_oxide 0.3.78, upstream Git commit ad49c4cb3638dc950a29350ef882161372fde473, https://github.com/yfedoseev/pdf_oxide. The source files derive from that exact crates.io distribution. UPSTREAM-SHA256.json records the original source hashes. upstream.patch records changes to existing files and the added small-object helper.

The independently named package and library are airust-pdf-backend / airust_pdf_backend. The package version tracks the AIRust release using it. This is not the official PDFOxide package. License texts, NOTICE and TRADEMARKS.md are preserved.

Code changes: bounded lookahead on long physical lines; reuse the existing parser to accept only a complete small object with an endobj boundary; preserve fallback, decryption, caches and structure-tree ordering; use byte slices for the preceding end marker check so invalid UTF-8 replacement characters cannot cause slicing panics.

Packaging changes: Rust library only, AIRust metadata and provenance, independent regression tests. Unrelated binding/example/benchmark build targets are omitted. Original upstream Rust test modules remain in source, but their unavailable external fixtures are not claimed as tested; lib test/doctest targets are disabled. AIRust regression tests, supported feature builds and real-PDF comparisons cover this maintained patch.

The original C ABI module is opt-in through `c-api`; default Rust use exports no competing PDF C symbols. Linux differential linking against the original parser exposed duplicate C symbols before this gate. Rust extraction and structure parsing are unaffected.
