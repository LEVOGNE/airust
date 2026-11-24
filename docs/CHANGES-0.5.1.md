# AIRust 0.5.1: fuzz-checked patch, unchanged runtime

0.5.1 does not change search, matching, retrieval, cache, PDF import or MCP behavior. Runtime source matches 0.5.0. The published version string is 0.5.1. No new public API, dependency, default feature, schema shape, saved format or DenkMaps integration.

The 0.5.0 measurements still describe this runtime. They were not repeated and are not new 0.5.1 samples. Slower outcomes from that campaign remain. Read [those results](CHANGES-0.5.0.md) and [upgrade](MIGRATION-0.5.1.md).

## Differential fuzz

An external release-profile harness, with overflow checks enabled, drove the library. The harness is not part of the published crate. It is a fixed-seed differential check, not coverage-guided libFuzzer or AFL, and it is not a proof that no defect exists.

Ten thousand seeds exercised the lexical core with `retrieval`, `advanced-search`, `documents`, `shared-index`, `execution` and `cache`:

- Random upsert and remove steps were compared with `Engine::from_entries`. The comparison covered JSON, stats, duplicate groups, BM25 and TF-IDF hits (score bits and matched terms), exact, fuzzy and token overlap matches, and `relate`.
- An independent oracle recomputed BM25 and TF-IDF from `text::term_frequencies`. It used limit 8 and the parameter pairs k1/b = 1.2/0.75, 0.4/0.0 and 3.5/1.0. Scores had to match bit for bit. Sparse queries with more than 32 known terms keep the existing document-term contribution order on short documents.
- Seed 1 checked equivalent queries (case, spaces, tabs, vertical tab, NBSP and NUL), clone isolation, and a 1,600-entry index that takes the large-candidate document cursor, then removals and a wide query.
- Retrieval excerpt slices, `select_excerpt`, German-stem text spans, `DocumentSnapshot::from_json` and `Engine::from_json` received random text.
- Every 17th seed flipped a saved cache payload, rewrote the declared length and SHA-256, then loaded and searched when loading succeeded.
- Every 64th seed, and seed 1, built a 48-term sparse query.

Result: 10,000 seeds, no panic, and no divergent score, span, JSON or retained clone.

## PDF import fuzz

One hundred sixty cases each had an eight-second limit. Fourteen were the repository fixtures unchanged. One hundred forty-six were mutations of those fixtures: a flipped byte, an inserted byte, a deleted byte, truncation, or replacement by random bytes. `PdfLoader` used limits of 262,144 file bytes, 32 pages, 1,048,576 text bytes and 256 chunks. Each case called `extract_pages_from_pdf`, `preview_pages_from_pdf(2)` and `pdf_to_training_examples`.

Result: no process panic and no timeout. A panic inside `pdf_oxide` is still caught by `checked_pdf_call` and returned as `PdfError`. That containment is existing behavior. It does not prove that every malformed PDF is rejected for one specific reason, and it does not cover unbounded files or the separate `airust-pdf-backend` crate beyond this import path.

## Not covered by this fuzzer

MCP framing, embeddings, the disk index, graph, hybrid search, reranking and ONNX were outside the harness. The release script `test.sh` still runs the existing feature tests, examples and strict Clippy/docs. A cache whose payload and checksum are both replaced remains accepted when the structure matches the supplied originals. Checksums detect accidental damage. They do not stop a caller who can rewrite the file. Wrong derived terms in such a file stay trusted structural data.

## Release script

`./test.sh` was run again on the same host with Rust 1.98.0. It passed: rustfmt, 253 default tests, 909 all-feature checks including 39 doctests, strict Clippy and documentation, 39 isolated feature checks, the packaged examples, and the package-boundary check (1,362 files). The all-feature run still ignores `persistence::tests::crash_writer_child`; the parent crash-recovery test launches that child. No Miri run and no new timing campaign were added.

## Host

The fuzz runs and `test.sh` used Rust 1.98.0 on macOS ARM64. The crate's minimum for provider-neutral code remains Rust 1.89. Optional local ONNX adapters remain Rust 1.92. One host and these seeds establish no zero-bug, semantic-quality, OS-quota or independent security-audit guarantee.
