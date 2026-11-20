# AIRust 0.3.1: PDF stability from real documents

This patch hardens optional PDF conversion. The default engine, retrieval algorithms and valid source-entry formats retain their behavior. No DenkMaps or MCP integration is performed. The [PDF guide](PDF-STABILITY.md) explains the new safe-save method, retained local knowledge and limits.

## Verified fixes

- A late-created destination was overwritten without `--overwrite` in 0.3.0. Exclusive atomic publication now preserves the concurrent writer's bytes. A 500-page CLI regression and concurrent-writer tests protect this behavior.
- Blank/image-only or otherwise unusable PDFs could produce a successful empty training file. Conversion now fails before replacing an existing output.
- A deterministic malformed UTF-8 PDF header caused an uncaught panic inside pdf_oxide 0.3.78. The adapter now catches Rust unwinding panics from opening, page counting and text extraction and returns `PdfError::PdfError`, without returning a partial import. Native aborts, OOM and panic-abort builds remain outside this guarantee.
- Incremental parenthesis tracking removes repeated sentence-prefix scans. An interleaved page/chunk optimization was rejected after measurement showed no reliable benefit; the original extraction/chunking phases are retained. Valid-corpus output is preserved, without a universal RAM or speed claim.

## Document and adversarial evidence

15 PDFs contain 8,960 pages and produce 33,708 chunks. All checked imported contents and rankings match 0.3.0. A separate parser provides 115 literal page probes: BM25 finds the expected page in the top ten for 111, TF-IDF for 102. Missing probes remain visible as retrieval-quality limitations. Four omitted pages are blank or contain only their page number according to the independent extraction.

The official Python library reference contains 2,488 pages. The other 14 manuals are a private user-provided collection; they and their extracted text are retained locally and not redistributed. Benchmark results measure per-document indexes under background load, not an independently labeled production search task. The unchanged slow 494-page case is traced to approximately 23 seconds of first-page work in the PDF dependency and remains an investigation.

The first 134-input adversarial campaign exposed one panic exit (101). Its initial counter incorrectly counted only signal exits; that counter is corrected and the input is preserved. The repeated campaign after the fix accepts 43 inputs and rejects 91 with normal errors, with zero unexpected exits and zero worker timeouts. Permanent library/CLI regression tests also exercise the failing input.

The local lab contains the original baseline and candidate outputs, parsed indexes, reference page text, queries, binaries, mutation cases, source manifests and raw logs. Public [measurement summaries](measurements/0.3.1/) contain aggregates rather than private document text. Reuse instructions are in [the PDF guide](PDF-STABILITY.md).

## Validation and publication gates

The local check script passes 197 default and 664 all-feature tests, five Python tests, all 37 isolated feature entries, the runnable examples and strict Clippy/rustdoc. Rust 1.89/1.92, a patch-level API comparison against 0.3.0, a refreshed dependency audit and an independent consumer supplement those checks. After restoring the original allocation/phase behavior, the final minimal code again passes all 664 all-feature tests and strict Clippy. Exact-commit CI repeats the full check script on the release commit.

Clean packaging, extracted-crate tests, a publication dry-run and exact-commit CI gate release. A registry consumer and equality of crate checksum, VCS commit, annotated tag and GitHub asset verify the published artifact. Those post-commit results are recorded by the release verification process.

These are regression and bounded robustness checks, not an independent penetration audit, proof of perfect extraction or a hard native-parser memory/time bound. Further 0.3.x hardening remains findings-driven; 0.4.0 feature work is deferred.

## Package isolation

Release preflight caught unanchored Cargo include patterns selecting a local lab README and a dependency license file. No artifact was published. Top-level metadata patterns are now anchored and `local-lab/**` is explicitly excluded. `scripts/check_package.py` checks the selected package paths against intended repository areas, and runs in `test.sh`. Its `--allow-dirty` flag is used only with the read-only `cargo package --list` inspection; actual release packaging requires a clean commit. The package list also passes under Cargo 1.89. See [Cargo include-pattern semantics](https://doc.rust-lang.org/cargo/reference/manifest.html#the-exclude-and-include-fields).
