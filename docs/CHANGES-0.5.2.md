# AIRust 0.5.2: coverage-guided fuzzing with versioned seeds

0.5.2 does not change search, matching, retrieval, cache, PDF import or MCP behavior. Runtime source matches 0.5.1 and therefore 0.5.0, apart from the published version string. No new public API, default feature, schema shape, saved format or DenkMaps integration.

`sha2` 0.10 is a new dev-dependency so the seed replay can check SHA-256 file names. It is not a new runtime dependency. Optional features already depend on the same `sha2` 0.10 line. The 0.5.0 measurements still describe this runtime. They were not repeated and are not new 0.5.2 samples. Read [those results](CHANGES-0.5.0.md) and [upgrade](MIGRATION-0.5.2.md).

## What was fuzzed

Two nightly libFuzzer binaries, built from the `fuzz/` directory on Rust 1.102.0-nightly (5f4effd57, 2026-11-13) for `aarch64-apple-darwin`, drove the library for 420 seconds each. Sanitizer-coverage instrumentation used 8-bit counters, PC tables and compare tracing. Host build scripts were not instrumented (`target-applies-to-host = false`). The PDF link used `-Wl,-undefined,dynamic_lookup` because `office_oxide` produces a dynamic library that would otherwise miss the coverage runtime. Panic strategy stayed unwind. Release optimization was level 3 with overflow checks and without LTO. The harness is not part of the published crate. These runs are not a proof that no defect exists.

The lexical target enables `retrieval`, `advanced-search`, `documents`, `shared-index`, `execution` and `cache`. Each input builds a small corpus, rebuilds it with `Engine::from_entries`, round-trips JSON, and compares BM25 and TF-IDF hits with an independent formula over `text::term_frequencies`. Scores and matched-term vectors must match bit for bit, including the document-term contribution order of the sparse path. The same input also checks duplicate groups, exact, fuzzy and token-overlap matches, `relate`, clone isolation, a cancelled `WorkControl`, text-search spans, excerpt slices, one flipped document-snapshot byte, and, on some inputs, a saved cache with a damaged payload, a rehashed payload, or a bad magic. A fixed 48-term sparse query, a 1,100-token document and the 4,096/4,097 repeated-word boundary run on every input. The 1,100-document cursor runs only when the input starts with the bytes `0xA5 0x5A`. `Engine::from_json` receives up to 256 bytes from the tail.

The PDF target writes the input to a temporary file and calls `extract_pages_from_pdf`, `preview_pages_from_pdf(2)` and `pdf_to_training_examples`. `PdfImportLimits` are 262,144 file bytes, 32 pages, 1,048,576 text bytes and 256 chunks. Import errors are ignored. A panic inside `pdf_oxide` remains contained by `checked_pdf_call` and is returned as `PdfError`.

Flags were `-max_total_time=420`, `-timeout=8`, `-rss_limit_mb=4096`, and `-max_len=4096` for lexical or `-max_len=65536` for PDF. The host was macOS ARM64. One worker was used.

| Target | Executions | Exec/s | Peak RSS | Coverage edges | Features | Slowest input |
| --- | --- | --- | --- | --- | --- | --- |
| Lexical | 51,748 | 122 | 46 MB | 5,258 to 6,010 | 10,207 to 16,588 | 0 s |
| PDF | 74,296 | 176 | 136 MB | 13,695 to 22,521 | 26,490 to 70,295 | 0 s |

The lexical binary contained 78,549 inline 8-bit counters. The lexical starter corpus had 27 non-empty files, 1 to 88 bytes. The PDF starter corpus had 43 non-empty files, 9 to 18,810 bytes. Saved inputs did not grow past those starter maxima, so `-max_len` did not enlarge the retained corpus. No crash, timeout, or out-of-memory artifact was produced. The uninstrumented warning about missing `__sanitizer_acquire_crash_state` is the usual libFuzzer note when AddressSanitizer is not linked. It is not a library failure.

## Versioned seeds

libFuzzer `-merge=1` reduced the lexical corpus to 973 files and 57,751 bytes, remeasured at 6,026 coverage edges. The PDF merge reduced the campaign corpus to 2,554 files and 15,883,870 bytes, remeasured at 22,507 edges. That PDF set does not fit a crates.io archive together with the rest of the crate. `-set_cover_merge=1` then reduced it to 1,516 files and 11,338,506 bytes. A merge of that set reported 22,476 coverage edges. Those 1,516 PDF files and all 973 lexical files are the versioned seeds under `tests/fixtures/fuzz-seeds/v0.5.2/`. `MANIFEST.txt` names each file by the SHA-256 of its bytes. `cargo test` replays them without libFuzzer: `tests/fuzz_seeds.rs` and `tests/fuzz_seeds_pdf.rs`. On Rust 1.98.0 debug builds those replays passed in 10.05 seconds and 38.13 seconds. They ship with the crate because `tests/**` is in the package include list. The `fuzz/` package does not.

`scripts/check_package.py` during `./test.sh` verified 3,859 intended files. An earlier `cargo package --no-verify`, taken before the release-check sentences below were written, produced a 4,899,878-byte crate. That figure is not the published archive size. The PDF seeds are the set-cover subset so the archive stays within the crates.io size limit.

## Not covered

MCP framing, embeddings, the disk index, graph, hybrid search, reranking and ONNX were outside the harness. A cache whose payload and checksum are both replaced remains accepted when the structure matches the supplied originals. Checksums detect accidental damage. They do not stop a caller who can rewrite the file. The bounded PDF limits are not the unlimited importer, and this run does not publish a new `airust-pdf-backend`.

## Release checks

`./test.sh` on Rust 1.98.0 (macOS ARM64) passed: rustfmt, 253 default tests, 911 all-feature checks including 39 doctests, strict Clippy and documentation, the unchanged isolated feature and example commands, and the package-boundary check (3,859 files). The all-feature total includes the two versioned-seed replays, 11.88 seconds for lexical and 41.51 seconds for PDF. The all-feature library result ignores `persistence::tests::crash_writer_child`. The parent crash-recovery test launches that child, and the child passed in the isolated persistence run.

Nightly 1.102.0-nightly (5f4effd57, 2026-11-13) Tree Borrows Miri, with `MIRIFLAGS=-Zmiri-no-extra-rounding-error` and one test thread, passed 71 tests. That set is 41 `index::` tests (`--no-default-features --features shared-index,execution`), 25 `tests/core.rs` tests (`--features shared-index`), 1 miri-reduced `stability_state` test, and 4 synchronous MCP `response_tests` (`--features mcp`). Four large index tests were left out because Miri did not finish them in reasonable time. They did not fail, and they passed natively in `./test.sh`: `empty_vocabulary_releases_its_hash_allocation_after_removal`, `sparse_vocabulary_shrinks_without_changing_retained_readers`, `token_ties_keep_id_order_and_character_scores_for_long_text`, and `token_top_k_preserves_full_comparison_scores_and_unicode_explanations`. Two async MCP tests in `shutdown_tests` were not part of the Miri set: `final_reply_work_keeps_admission_and_shutdown_until_cancelled_work_returns` and `shutdown_does_not_wait_for_other_server_clones_worker_permits`.

Rust 1.89, Rust 1.92, the 0.5.0 patch-level API comparison and the independent consumer were not repeated. No new measurement JSON was added. These checks are not a proof that no bug exists.
