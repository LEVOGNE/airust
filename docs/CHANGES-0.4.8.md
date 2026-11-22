# AIRust 0.4.8: less query allocation and reliable MCP progress deadlines

This release focuses on stability, performance, reliability and existing API/MCP contracts. It reduces temporary query allocations and makes progress delivery observe the request's original time budget. Public Rust signatures, ranking arithmetic, evidence recipes, source positions, saved knowledge and default features remain. No new dependency, model or DenkMaps integration is added.

The measured gain is specific: repeated terms and optional coverage scoring allocate less. Whole-document retrieval is approximately unchanged to slightly slower in this run, and index/process RAM is essentially unchanged. This is not a general search-speed or semantic-quality improvement.

## What changed and why

`text::term_frequencies` previously allocated an owned token string even when that key already existed. The final implementation keeps the previous entry-lookup path until the first repetition. Later repeats use a borrowed lookup and update the existing count. The first duplicate still makes one temporary string. New distinct terms encountered after repetition need an additional lookup. Normalization, token boundaries, counts and sorted order remain.

An initial implementation used a borrowed lookup for every occurrence. It saved allocations but made an authored 256-distinct-short-token case about 14.7% slower. That candidate, its exact binaries and all measurements remain privately retained. The final implementation avoids the additional lookup for an entirely unique input. Its complete validation and measurements were repeated. No universal latency guarantee follows from a small fixture.

The optional `LexicalCoverageSource` no longer rebuilds a set for the prepared question or a set for every candidate's matched terms. Its built-in Engine already returns each matched effective term once, and question preparation already deduplicates effective terms. The counts reuse those invariants; no assumption is imposed on custom sources. Scores, raw component diagnostics and final ordering remain identical. See [API/performance behavior](CORE-PERFORMANCE.md) and [coverage contract](LEXICAL-COVERAGE.md).

MCP progress delivery previously waited up to its separate 100 ms send limit without selecting on the shorter remaining operation deadline. Start, heartbeat and final notifications now share the original monotonic deadline and observe connection cancellation. Expiry returns the existing `deadline_exceeded` error instead of becoming a generic progress-delivery cancellation. Expiry alone does not mark a session as a failed transport. A further checkpoint prevents starting calculation after expiry during initial notification delivery. Final result delivery also rechecks execution state.

Actual transport errors and sends reaching their 100 ms transport-stall limit still stop the session. A final progress notification may be omitted on expiry. Queued notifications, final response writes, OS scheduling and non-yielding work remain outside a hard wall-clock guarantee. Hosts must finish their loading indication on result, error, cancellation or connection closure, rather than requiring a completion notification. See [MCP API and timing](MCP.md) and [security boundaries](MCP-SECURITY.md).

## Executed checks

The final complete local script passes **222 default tests, 865 all-feature checks including 39 documentation examples, 39 isolated feature entries and five Python tests**. Strict Clippy/docs pass. Rust 1.89 checks provider-neutral features; Rust 1.92 checks all features. Public API comparison against tagged 0.4.7 reports **229 passes and 31 tool skips**. A separate Rust 1.89 application exercises public retrieval, coverage, quotations, document positions and MCP construction/schema access.

Four new private progress tests cover expired requests racing ready notifications, short deadlines interrupting stalled sends, later use of that session, request/session cancellation and actual transport failure/stall. The real protocol deadline test checks health recovery before subsequent work. New Unicode/frequency and independent effective-term coverage oracles check repetitions, canonical equivalents, identifiers, unknown terms and punctuation. Existing cancellation, saturation, hostile input, connection and schema regressions remain in the matrix.

Five targeted Tree Borrows Miri cases pass: reference BM25/repeated query counting, the new distinct-term coverage oracle, scope intersections, query fallback and repeated citation occurrences. Extra floating-point rounding injection alone is disabled; ordinary memory/alias checks remain enabled. Native PDF/ONNX and async MCP are outside these finite interpreter cases. They are not a proof of complete soundness.

The unchanged PDF backend 0.3.5 passes 14 regressions and its 381-source-file provenance check. The refreshed dependency audit reports zero known vulnerabilities in its checked database, with existing informational maintenance notices for `chrono`, `paste` and `ttf-parser`. This is not an independent security audit or deployment certification. Extracted-package tests, exact-commit CI, a fresh registry consumer and checksum/VCS/tag matching are publication gates.

## Fixed comparison and reproducibility

The [public applications, manifests and executable harness](validation-0.4.8/README.md) compare exact registry 0.4.7 with the local 0.4.8 candidate. Compiler jobs finish before timed runs. The unchanged private document corpus has 72 Markdown files, 2,755,468 source bytes and 1,910 chunks. All 342 questions are exposed regression cases, with 276 authored positives and 66 intentionally unsupported questions. No fresh holdout, LLM answer or independent human semantic judgment is added.

Literal BM25, prepared BM25 and prepared coverage each use six passages, 100 candidates, 2,000 bytes per passage and 12,000 total source-text bytes. **All 1,026 complete response JSON strings match byte-for-byte**, including scores, terms, IDs, order, recipes, spans and selection details. Every repeated sample also matches its initial result. Therefore reference evidence/ranks in these cases remain unchanged, including earlier misses; no improved answer correctness or token saving is claimed. Full private responses remain compressed locally.

A warm-up precedes five repetitions per question/configuration/version, giving **10,260 timed samples**. These round trips include source work, excerpts, serialization, IPC and Python response parsing, not isolated BM25 time.

| Median / p95 context round trip, ms | 0.4.7 | 0.4.8 |
| --- | ---: | ---: |
| Literal BM25 | 0.899 / 1.367 | 0.935 / 1.463 |
| Prepared BM25 | 1.290 / 1.758 | 1.323 / 1.820 |
| Coverage | 1.285 / 1.755 | 1.318 / 1.851 |

The final candidate is slightly slower in this mixed-query IPC comparison. Reduced working allocations do not establish faster end-to-end search. Sampling, scheduling and binary/code-layout effects are not isolated here. Do not infer an index-build or RAM improvement from these numbers.

## Query allocation traffic and isolated timing

The same prebuilt 1,000-entry index and ten authored fixtures are used in both versions. Two independently compiled configurations separate DHAT allocation instrumentation from timing. Seven alternating process pairs measure 1,000 calls per fixture, **140,000 timed calls** in fourteen processes. Two instrumented processes measure 100 calls per fixture, 2,000 calls total. Signatures agree across versions; measured per-batch retained-byte/block deltas are zero in both versions.

| Fixture | Allocation blocks per call, 0.4.7 → 0.4.8 | Allocated bytes per call, 0.4.7 → 0.4.8 | Median time change |
| --- | ---: | ---: | ---: |
| Repeated short token | 258 → 4 | 2,672 → 1,656 | -12.17% |
| Repeated Unicode terms | 258 → 7 | 5,040 → 3,155 | -6.96% |
| Repeated long token | 18 → 4 | 33,120 → 18,798 | +0.98% |
| Distinct short tokens | 296 → 296 | 18,548 → 18,548 | -1.39% |
| Distinct long tokens | 299 → 299 | 32,672 → 32,672 | +0.40% |
| Punctuation | 1 → 1 | 768 → 768 | +2.50% |
| Repeated-term Engine search | 322 → 196 | 38,396 → 37,640 | -2.56% |
| Distinct-term Engine search | 229 → 229 | 37,014 → 37,014 | -0.47% |
| Coverage inspection | 1,244 → 941 | 115,950 → 95,566 | -4.65% |
| Coverage with repeated input | 1,507 → 1,204 | 130,846 → 110,462 | -2.94% |

A negative time change is faster in this fixture. Small differences are not statistical proof, and unchanged paths can vary with scheduling/code layout. The table includes slower cases. Allocation traffic is not resident index memory or a promise for all queries. The initial distinct-token slowdown and complete first-candidate counters are retained in the sanitized records; they are not silently replaced.

## Actual MCP processes on retained PDF-derived data

Five alternating old/new process pairs run for each of two inputs: the full **167,942-entry canonical knowledge** and a revision-bound NumPy reference snapshot. That gives twenty fresh server processes. The original source files hash identically before/after every probe. Discovery, BM25/TF-IDF, progress sequence, source binding, exact snapshot readback, concurrent requests, matching, relationships, rejected arguments, health and EOF shutdown are checked.

**1,380 actual tool result/error responses validate against their advertised schemas**. Across old/new versions, **690 complete structured results compare equal when paired by JSON-RPC request ID**, excluding only the explicit status version. Five schema definitions compare equal after asserting/removing their expected release-qualified `$id` URNs. There are 400 sequential timed searches, eighty additional concurrent searches, 820 exact snapshot readbacks, forty matching/relationship calls, twenty invalid-argument calls and twenty health calls. Discovery calls are additional.

| MCP observation | 0.4.7 | 0.4.8 |
| --- | ---: | ---: |
| Canonical startup to initialized, median ms | 5298.76 | 5262.72 |
| Canonical warm BM25, median / p95 ms | 8.041 / 13.771 | 8.225 / 13.741 |
| Canonical warm TF-IDF, median / p95 ms | 22.716 / 39.726 | 22.514 / 39.477 |
| Canonical whole-child peak RSS, median MiB | 1727.80 | 1727.89 |
| Snapshot startup, median ms | 144.59 | 144.46 |
| Snapshot warm BM25, median / p95 ms | 1.466 / 2.030 | 1.460 / 1.975 |
| Snapshot warm TF-IDF, median / p95 ms | 1.761 / 3.676 | 1.709 / 3.612 |
| Snapshot whole-child peak RSS, median MiB | 57.14 | 57.19 |

Timings include progress and stdio response transport. Snapshot follow-up readbacks are checked separately from timed searches. Startup is synchronous loading/indexing, outside request deadlines. Whole-child peak RSS includes parsing, source buffers, index and transport. No meaningful process-RAM reduction, general MCP speedup or hard resource quota is established.

The harness initially forgot expected versioned schema URNs, and later incorrectly paired concurrent replies by arrival order. Those initial records are retained. Pairing now asserts request IDs explicitly; concurrency does not promise arrival order. A final aggregation label collided with the actual crate-version field after all twenty processes completed. The aggregation was corrected offline and all saved raw response pairs rechecked, without remeasuring those runs. No runtime or data was changed for those harness corrections, and no failed-quality response was selectively retried.

## Large-corpus consistency and limits

A separate complete in-memory readiness run uses all 167,942 entries in each version with `execution,shared-index`. Updates/deletions, restoration of 128 affected entries, rebuilding, JSON round trips, four concurrent readers with forty queries each, cancellation/deadline rejection and retained-clone isolation pass. Portable export hashes agree across versions. All original input data remains unchanged. This is one finite run per version, not a million-user capacity test or a fresh PDF parser benchmark.

Observed initial index builds are 5.771 / 5.956 seconds, and complete child runs are 22.048 / 22.616 seconds for old/new. These single observations include validation and setup and do not establish a construction improvement. The source algorithm and index format are unchanged.

Measurements use one AC-powered Apple M5 Max with 64 GB, Rust 1.98.0 and Python 3.14.7. Authored distributions, exposed questions, two known MCP inputs, process caches, scheduling, one host and small process-pair counts limit generalization. The [sanitized measurements](measurements/0.4.8/README.md) retain positive/negative costs, every scalar query outcome, allocation counts, initial attempts and source/method identities. Full sources, transcripts, exact binaries, locks and logs remain private. Public counters/hashes cannot independently reconstruct the private corpus.

The earlier semantic limitations remain: lexical overlap, exact quote verification and correct JSON do not establish truth or entailment. This release preserves previous retrieval behavior and improves execution reliability; it does not claim to solve wrong-component answers from 0.4.7.

## Knowledge and build storage

Canonical knowledge remains **167,942 entries from 35 PDFs and 49,086 physical pages**, SHA-256 `13168d87872a137355fd8499c90a6084f52243a2ec7456f29b5fcb2a1c7423c3`. PDFs, source snapshots, all earlier knowledge, failed candidates and measurements remain. No application database is migrated.

Only the named temporary Cargo and Miri targets hold regenerable build output. Release packages, exact locks, logs, small measured executables and receipts are archived before deleting those targets after verified publication. No local model daemon was needed for these tests. Default features remain empty, and no DenkMaps integration is performed.
