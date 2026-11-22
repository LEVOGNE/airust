# AIRust 0.4.8 validation

This is a stability/performance release. Existing ranking and context are expected to remain identical. No new LLM replies or semantic accuracy improvement are claimed. Read the [release report](../CHANGES-0.4.8.md), [query/API behavior](../CORE-PERFORMANCE.md) and [MCP timing contract](../MCP.md).

## Independent application

Copy `consumer.toml` to a private directory as `Cargo.toml`, and `consumer.rs` to `src/main.rs`. Run with Rust 1.89. Before publication the same application uses an explicit local dependency path; after publication a fresh lock uses exact registry 0.4.8. Verify source, version, checksum and VCS commit. It exercises the existing public query, excerpt, quotation and document-position contracts.

## Fixed retrieval corpus

Use the retained private 72-file / 1,910-chunk Markdown snapshot and all 342 previously authored questions. Every question is exposed regression data, not a new holdout or independent human evaluation. Compare exact registry 0.4.7 against candidate 0.4.8 for literal BM25, prepared BM25 and lexical coverage. All use six passages, 100 candidates, 2,000 bytes per passage and 12,000 total text bytes. Compare the original complete response JSON strings, including scores, terms, IDs, order, source spans, recipes and selection details: 1,026 comparisons. No version fields occur in those retrieval payloads.

Copy each `worker-*.toml` as `Cargo.toml` and `retrieval-worker.rs` as `worker.rs` to separate private directories. Build with `--release --features candidate,coverage` into one dedicated target; before publication give the candidate dependency its local crate path. Keep exact locks and executable hashes. One untimed warm-up precedes five repetitions per question/configuration/version, 10,260 timed samples. Alternate old/new order deterministically. Context timing includes serialization, IPC and Python context construction, not isolated BM25 time. Own compilers must finish before timing.

## Query working allocations and time

`query_work.rs` and `query-*.toml` define the same standalone application for both crate versions. Six authored token distributions include repetition, Unicode, long terms, mostly unique terms and punctuation; two Engine searches and two optional coverage inspections use a prebuilt 1,000-entry index. These authored fixtures are not production traffic.

Build once with the application's `heap` feature, preserve the resulting executable, then separately without that feature. With heap instrumentation, run `BINARY heap 100 PRIVATE_PROFILE.json`; DHAT measures allocation-block/byte deltas and retained bytes for each call batch after fixture/index setup. DHAT timings are not performance measurements. Without instrumentation, run `BINARY time 1000` in seven alternating old/new process pairs. Check signatures for every fixture. Report unique-term regressions as well as repeated-term improvements. This is whole-call temporary allocation traffic, not a claim of lower index RAM.

## Actual MCP processes and large retained knowledge

Use the same exact-version `airust-mcp` executables and retained inputs in both versions. `mcp_probe.py` checks discovery, output schemas through retained transcript validation, BM25 and TF-IDF, source text/spans, progress sequences, concurrent calls, matching/relations, rejected arguments, health and clean EOF shutdown. Original source files must hash identically before/after. Protocol transcripts contain private source data and stay private. Compare complete successful structured results across versions; only the explicit status version and schema `$id` version URNs change. Pair complete results by JSON-RPC request ID, because concurrent requests may finish in any order. Notifications are timing-dependent and are not required to have identical counts.

Run against the full canonical 167,942-entry knowledge and a large revision-bound document snapshot. This uses existing PDF-derived text; it is not a new PDF parser-speed evaluation. Five alternating process pairs retain cold startup and warm stdio round-trip samples, including failures. Whole-child RSS includes loading, parsing, buffers and the index; it is not an isolated heap measurement.

`corpus.rs` with `corpus-*.toml` separately checks retained knowledge through updates, cloning, rebuilds, deterministic BM25/TF-IDF queries, round trips, invalid operations and cancellation. Use the same input and 167,942-entry limit for both versions. Its output contains counters/hashes, not passages. Small targeted Miri cases exclude native PDF, ONNX and async MCP; they are not a complete soundness/security audit.

## Deadline regressions and release gates

Production progress delivery is centralized in `mcp::progress` (private). Tests cover expired deadlines racing ready notifications, stalled sends stopped by shorter request deadlines, later use of the same session, request/session cancellation and a genuine 100 ms transport stall/error. The actual protocol suite additionally checks deadline errors, health recovery, disconnects, worker saturation and hostile input. Request expiry is not an I/O failure; bounded sends still cannot force a blocked OS write or non-yielding future to finish on time.

Run the full test/Clippy/docs/feature matrix, Rust 1.89/1.92, API comparison with 0.4.7, refreshed dependency audit, backend regressions, separate consumer, extracted-package tests and exact-commit CI. After publication verify registry source/checksum/VCS and GitHub tag/asset identity. Archive measurements, logs, locks and small measured executables before removing only `/private/tmp/airust-stability-048-build` and `/private/tmp/airust-stability-048-miri`. PDFs, source snapshots, canonical knowledge and all earlier measurements remain.
