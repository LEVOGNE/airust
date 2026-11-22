# AIRust 0.4.13 validation

Compare exact registry 0.4.12 with candidate 0.4.13 using identical programs and inputs. The change is a bounded request-local memo for exact IDF weights during cold document-norm computation. Read the [report](../CHANGES-0.4.13.md), [performance contract](../CORE-PERFORMANCE.md), [API cookbook](../API-COOKBOOK.md) and [MCP boundaries](../MCP.md).

## Applications and frozen inputs

Copy each manifest as Cargo.toml and its paired Rust program alongside it. Before publication, add a local path only to candidate dependencies. Retain exact locks, program/library hashes and measured executable hashes. After publication a fresh Rust 1.89 application must use exact registry 0.4.13 without a path override. The consumer exercises retrieval, coverage, quotations, source positions and MCP/schema construction.

Retain the private 72-file, 1,910-chunk document snapshot and all 342 previously authored questions. These are exposed regressions, not fresh holdouts or logged user traffic. Compare complete original JSON strings for literal BM25, prepared BM25 and coverage: 1,026 comparisons. Use six passages, 100 candidates, 2,000 bytes per passage and 12,000 total text bytes. One warm-up and five samples per question/configuration/version yield 10,260 round trips, including serialization, IPC and Python parsing. Managed compilers must finish before timing.

## Cold queries, warm controls and heap

Keep all 25 prior warm fixtures, including TF-IDF, 32/33-term boundaries and sparse broad BM25/TF-IDF at 33/128/512 terms. Seven alternating process pairs produce 282,800 calls in fourteen plain processes: nineteen fixtures use 1,000 calls per sample and six broad cases use 200. Two separate instrumented runs use 100 calls per fixture, 5,000 total. Warm-ups initialize hints before timing. Signatures must match and temporary-call retained deltas must be zero. Instrumented durations are not latency measurements.

Plain `cold_time 1` executes twenty fresh-cache queries per fixture using clones of a never-queried Engine. The four fixtures are the prior 1,000-entry index, 256-entry wide index, 257-entry sparse/vocabulary index and a one-entry tiny control. Seven alternating pairs yield 1,120 cold norm-cache calls across fourteen processes. Cloning/setup is outside the timer. Normal process, instruction and filesystem caches remain; this is not cold-disk measurement.

Instrumented `norm_heap 1 PRIVATE_PROFILE.json` measures the four initial-query allocation/retained-hint cases after dropping results. Source/index setup is outside tracking. `corpus_norm_heap 1 PRIVATE_PROFILE.json CANONICAL.json` loads and indexes all 167,942 entries before profiling ten fixed initial TF-IDF queries, once per version. Allocated query bytes include temporary work; retained net bytes measure hint growth. Keep these separate from whole-process RSS.

The new memo has 256 slots, full live term-allocation identity keys and exact IDF values. Collisions recompute and replace. It is allocated lazily after a cold norm checkpoint and released after the query; it adds 4,096 payload bytes on the measured 64-bit build. BM25, unknown queries and complete warm norm hits allocate no memo. Persistent norm storage/invalidation, arithmetic/order, nonblocking lock/poison fallback and checkpoints remain. Independent score-bit oracles and new collision/interruption/warm-allocation cases check correctness. Retain all tiny-input overhead and slower timings.

## Actual MCP: first, initial and repeated requests

`prepare_wide.py PRIVATE_DIRECTORY` creates the deterministic synthetic corpus and ten queries at 32/33/64/128 terms. Keep extended hashes alongside the original frozen hashes. Two preplanned complete campaigns each use canonical knowledge, the revision-bound NumPy snapshot and the synthetic input: five alternating pairs per input, thirty servers per campaign, sixty total.

Each server executes every query/algorithm once, recording initial cold/partial-cache samples; the very first TF-IDF call is available separately. Two later complete passes must reproduce every initial structured payload exactly. Discovery/schema/source binding, initial exact readbacks, progress, concurrency, matching/relations, rejected arguments, health and EOF shutdown remain checked. Compare complete old/new payloads by JSON-RPC request ID, excluding only status version and asserted version-qualified schema URNs. Concurrent replies may arrive out of order.

Readbacks are outside search timing. Search includes progress and stdio transport; startup includes synchronous loading/indexing. No norm prewarming or startup work shift is introduced. Whole-child RSS includes input buffers, index, cache and transport. Hash inputs before/after each probe and retain both full campaigns. Stage-start CPU/compiler observations do not establish continuously idle hardware.

A separate three-pair development pilot is preserved privately and excluded from final samples. Its baseline binary hash/library source were verified; it is not substituted for the exact-registry baseline in final comparisons. Retain its slower warm observation as well as its promising cold result.

## Construction, consistency and release gates

Five alternating pairs for canonical knowledge and three synthetic distributions produce forty plain construction processes, with parsing/input generation outside timing. Six separate profiles check derived live heap. Stats/signatures and derived live bytes must agree. Construction remains a control.

The execution/shared-index corpus worker checks updates/deletions/restoration, rebuilds, JSON round trips, concurrent readers, retained clones and cancellation against all 167,942 entries. Source/portable exports must agree. This is finite consistency validation, not a new PDF parser, semantic-quality or million-user capacity test.

Run the default/all-feature/isolated-feature/Clippy/docs/Python matrix, minimum Rust versions, API comparison with 0.4.12, refreshed dependency audit, backend regressions/provenance, an independent consumer and fourteen targeted Tree Borrows Miri cases. Native/async/FFI work is outside those finite interpreter cases. Test the extracted package and require exact-commit CI, then verify registry source/VCS/checksum, tag and release asset. The prior snapshot stress-example conflict fix and deterministic regression remain in `test.sh`.

Archive packages, logs, locks and small exact measured programs before removing only the named temporary Cargo/Miri targets. Preserve PDFs, source snapshots, knowledge and all raw results. No DenkMaps integration is performed.
