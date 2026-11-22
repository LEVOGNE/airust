# AIRust 0.4.9: faster complete retrieval and bounded MCP serialization

This release focuses on stability, performance, reliability and the existing API/MCP contract. It reduces work when selecting relevant text and checking JSON size. Public signatures, ranking arithmetic, scores, evidence recipes, source positions, default features and knowledge formats remain. No dependency, model, storage migration or DenkMaps integration is added.

The result is specific: prepared document retrieval and coverage are about 23% faster at the median in this run. Literal BM25 changes little, including a slightly higher p95. MCP results are mixed; the first large-corpus campaign is slower, while its complete confirmation is close at the median with a higher BM25 tail. Snapshot search improves. Index/process RAM is essentially unchanged. There is no universal search/MCP speedup or semantic-quality claim.

## What changed

Legacy excerpt analysis previously normalized every grapheme through the general Unicode pipeline, including plain ASCII. Pure ASCII input now checks its normalized-byte budget first and collects lowercase tokens and original spans directly. Mixed text retains grapheme mapping and general normalization while using a simpler conversion for ASCII graphemes. Token limits, compatibility expansions, combining marks, original UTF-8 ranges, CRLF and phrase/highlight boundaries retain their behavior. Advanced TextAnalyzer behavior is unchanged. No unsafe conversion, persistent cache or second index is introduced.

Size-only retrieval/candidate/metadata and MCP checks previously serialized to a temporary buffer and discarded it. Retained output and counting now use one bounded writer. Counting uses a sink, preserving the actual serializer, JSON escaping, UTF-8 byte count, early budget rejection and serialization failures. JSON that is needed for responses, source descriptors or startup metadata accounting is still retained. This does not avoid traversing the value or eliminate already owned request/response memory.

Applications keep using `Engine`, `Retriever::query`, `select_excerpt`, `ExcerptOptions` and `McpServer` without a new setting. MCP schemas, `argument_budget`/`output_budget`, progress, request deadlines, cancellation and session rules remain. See [API/performance](CORE-PERFORMANCE.md), [excerpt behavior](EXCERPTS.md), [API cookbook](API-COOKBOOK.md), [MCP](MCP.md) and [security boundaries](MCP-SECURITY.md).

## Executed validation

The complete final local script passes **222 default tests, 870 all-feature checks including 39 documentation examples, 39 isolated feature entries and five Python tests**. Strict Clippy/docs pass. Rust 1.89 checks provider-neutral features; Rust 1.92 checks all features. API comparison against tagged 0.4.8 reports **229 passes and 31 tool skips**. An independent Rust 1.89 application exercises public retrieval, coverage, exact quotations, document positions and MCP schema/construction access.

New regressions independently compare counted/retained JSON against serde_json's encoded bytes at every budget around escaped UTF-8 output, and reject serializer failures. Actual MCP tests accept arguments exactly at the encoded limit, reject larger ASCII/Unicode/control-character inputs and then successfully search on the same session. Excerpt tests check controls, NUL, CRLF, uppercase tokens, compatibility forms, combining marks, highlights, original slices and exact normalized-byte/token limits. Existing hostile-input, saturation, disconnect, cancellation, absolute-deadline and progress tests remain.

The first strict Clippy run rejected placement of a new test module among production items. It was moved to the end of the module without suppressing the lint; the entire matrix was repeated successfully. Initial logs remain private. Seven targeted Tree Borrows Miri cases pass: BM25 reference arithmetic, both new excerpt cases, distinct-term coverage, scope intersection, query fallback and repeated quote positions. Extra floating-point rounding injection alone is disabled; memory/alias checks remain. Async MCP, native PDF and ONNX are outside these finite interpreter cases. This is not a full soundness or security audit.

The unchanged PDF backend 0.3.5 passes 14 regressions and its 381-source-file provenance check. A refreshed dependency audit reports zero known vulnerabilities in its checked database, with the existing informational maintenance notices for chrono, paste and ttf-parser. Extracted-package tests, exact-commit CI, a fresh exact-registry consumer and registry/VCS/tag/checksum matching are publication gates.

## Complete retrieval: fixed inputs and identical answers

The [public applications and harness](validation-0.4.9/README.md) compare exact registry 0.4.8 with candidate 0.4.9. The unchanged private snapshot has 72 Markdown files, 2,755,468 source bytes and 1,910 chunks. All 342 questions are previously exposed regression cases: 276 authored positives and 66 unsupported questions. No fresh holdout, local model reply or independent semantic judgment is added.

Literal BM25, prepared BM25 and coverage use six passages, 100 candidates, 2,000 bytes per passage and 12,000 total source-text bytes. **All 1,026 complete response JSON strings match byte-for-byte**, including scores, terms, IDs, order, excerpts, spans, recipes and selection details. Every repeated sample also matches its initial result. Previous misses and semantic limitations remain; this release changes execution cost rather than answer quality or token content.

One untimed warm-up precedes five samples per question/configuration/version: **10,260 warm timed samples**. Old/new order alternates deterministically. These are complete context round trips including source work, selected evidence, serialization, IPC and Python parsing, not isolated BM25 time.

| Complete context round trip, median / p95 ms | 0.4.8 | 0.4.9 |
| --- | ---: | ---: |
| Literal BM25 | 1.200 / 2.474 | 1.169 / 2.513 |
| Prepared BM25 | 1.708 / 3.093 | 1.321 / 2.586 |
| Coverage | 1.733 / 3.167 | 1.321 / 2.618 |

Prepared and coverage median round trips improve by 22.67% and 23.79% respectively. Literal median improves by 2.59%, while literal p95 increases by 1.57%. These are observations on this host rather than guarantees. Of 342 per-question median pairs, 107 literal, 21 prepared and 11 coverage cases are slower with the candidate. A better aggregate must not hide individual slower samples. All raw timings and per-question scalar outcomes remain in the records.

## Working allocations and isolated fixture timing

Fourteen authored fixtures retain the previous ten cases and add ASCII, mixed and Unicode excerpt selection plus complete relevant-excerpt retrieval. Engine cases use the same prebuilt 1,000-entry index. Seven alternating plain process pairs measure 1,000 calls per fixture: **196,000 timed calls in fourteen processes**. Two separate DHAT builds measure 100 calls per fixture, 2,800 calls total. Instrumented durations are not used as performance measurements. Signatures agree across versions, and measured per-batch retained-byte/block deltas are zero in both versions.

| Authored fixture | Allocation blocks per call, old → new | Allocated bytes per call, old → new | Median time change |
| --- | ---: | ---: | ---: |
| `coverage_repeated` | 1204 → 1204 | 110462 → 110462 | +1.80% |
| `coverage_small` | 941 → 941 | 95566 → 95566 | -0.67% |
| `excerpt_ascii` | 785 → 762 | 792974 → 252438 | -77.96% |
| `excerpt_mixed` | 569 → 569 | 417113 → 417113 | -31.72% |
| `excerpt_unicode` | 197 → 197 | 72254 → 72254 | -16.28% |
| `long_repeated` | 4 → 4 | 18798 → 18798 | +0.51% |
| `mixed_unicode` | 7 → 7 | 3155 → 3155 | -2.67% |
| `punctuation` | 1 → 1 | 768 → 768 | -0.66% |
| `relevant_retrieval` | 1623 → 969 | 186154 → 111226 | -6.48% |
| `repeated_search` | 196 → 196 | 37640 → 37640 | +2.94% |
| `repeated_short` | 4 → 4 | 1656 → 1656 | +0.06% |
| `unique_long` | 299 → 299 | 32672 → 32672 | +0.77% |
| `unique_search` | 229 → 229 | 37014 → 37014 | +5.10% |
| `unique_short` | 296 → 296 | 18548 → 18548 | -3.21% |

A negative time change is faster in that fixture. The large ASCII excerpt improvement applies to that authored distribution, not all documents. Mixed/Unicode fixtures still allocate source mapping, and lower allocation traffic is not lower resident index RAM. Unchanged search/frequency/coverage paths show both positive and negative timing differences, including slower search cases. Their allocation counts remain unchanged. Binary layout, caches, scheduling and small process counts are not isolated; no statistical significance or universal improvement is established.

## Actual MCP processes: both complete campaigns retained

Each campaign runs five alternating old/new process pairs on the full **167,942-entry canonical knowledge** and a revision-bound NumPy reference snapshot. That is twenty fresh stdio servers per campaign. Discovery, advertised schemas, progress, source binding, BM25/TF-IDF, exact snapshot readbacks, concurrent requests, matching, relationships, invalid arguments, health and EOF shutdown are checked. Source files hash identically before/after every probe.

Across both campaigns, **2,760 actual tool result/error responses validate against their schemas**, and **1,380 complete old/new structured results match by JSON-RPC request ID**, excluding only the explicit status version. Five schemas compare equal after asserting/removing their expected release-qualified $id URNs. There are 800 sequential timed searches, 160 additional concurrent searches, 1,640 snapshot readbacks, eighty matching/relationship calls, forty rejected-argument calls and forty health calls. Discovery is additional. Progress notification counts are timing-dependent.

### First complete MCP campaign

| MCP observation | 0.4.8 | 0.4.9 |
| --- | ---: | ---: |
| Canonical startup, median ms | 5448.71 | 6022.87 |
| Canonical BM25, median / p95 ms | 7.494 / 13.589 | 8.677 / 19.272 |
| Canonical TFIDF, median / p95 ms | 23.642 / 39.782 | 25.821 / 50.585 |
| Canonical whole-child peak RSS, median MiB | 1727.75 | 1727.81 |
| Snapshot startup, median ms | 219.74 | 223.23 |
| Snapshot BM25, median / p95 ms | 2.351 / 8.019 | 1.673 / 3.447 |
| Snapshot TFIDF, median / p95 ms | 2.555 / 6.475 | 2.137 / 8.268 |
| Snapshot whole-child peak RSS, median MiB | 58.34 | 58.02 |

The canonical first campaign is slower, including startup and tail latency. Own compiler jobs had finished before timing. Unrelated Rust compiler activity and other CPU-intensive processes were observed on the shared host. Their causal contribution cannot be quantified from this experiment. This first result is retained and is not replaced with a faster selection.

### Complete diagnostic confirmation

The entire twenty-process MCP campaign was repeated with exactly the same inputs, compiled executables, schemas, probes and parameters. This was triggered by the mixed results and observed host activity. No code, binary or time budget was changed. Complete raw transcripts and the diagnostic reason remain private; sanitized scalars and method notes are published separately.

| MCP observation | 0.4.8 | 0.4.9 |
| --- | ---: | ---: |
| Canonical startup, median ms | 5914.30 | 6125.44 |
| Canonical BM25, median / p95 ms | 8.704 / 15.616 | 8.736 / 18.681 |
| Canonical TFIDF, median / p95 ms | 25.366 / 50.351 | 25.722 / 49.803 |
| Canonical whole-child peak RSS, median MiB | 1727.86 | 1727.83 |
| Snapshot startup, median ms | 148.48 | 148.66 |
| Snapshot BM25, median / p95 ms | 1.542 / 2.237 | 1.426 / 1.685 |
| Snapshot TFIDF, median / p95 ms | 1.941 / 3.861 | 1.466 / 3.487 |
| Snapshot whole-child peak RSS, median MiB | 56.88 | 57.19 |

In confirmation, canonical median BM25 is approximately unchanged and TF-IDF is slightly slower. Canonical BM25 p95 remains higher. Snapshot medians improve, with differing tails between the two campaigns. Whole-child RAM remains essentially unchanged. Startup spans roughly 5.3–8.0 seconds across canonical confirmation processes. This experiment does **not** establish a general MCP latency gain or isolate a repeatable core construction regression. Both campaigns and all slower results remain visible.

Warm MCP timings include progress and stdio result transport. Follow-up snapshot readbacks are validated outside the search timer. Startup is synchronous loading/indexing outside request deadlines. Peak RSS includes parsing, source buffers, index and transport. New size guards are not a hard process-time or RAM quota.

## Retained knowledge, construction and limits

A separate full in-memory run in each version uses 167,942 records with execution/shared-index. Updates/deletions, restoration of 128 affected entries, rebuilding, JSON round trips, four readers with forty queries each, cancellation/deadline rejection and retained-clone isolation pass. Source and portable export identities match. This is finite consistency validation rather than a million-user capacity test or new PDF parser benchmark.

Observed initial index builds are 6.151 / 8.001 seconds, and complete child runs are 23.609 / 26.561 seconds for old/new. The candidate is slower in these single observations. No construction improvement is claimed. The construction algorithm and format are unchanged; shared-host activity and binary effects are not isolated.

Measurements use one AC-powered Apple M5 Max with 64 GB, Rust 1.98.0 and Python 3.14.7. Exposed authored questions, one corpus, two MCP inputs, shared-host activity, cache state and small process-pair counts limit generalization. Inspect the [sanitized measurements](measurements/0.4.9/README.md). Full private sources, transcripts, failed initial logs, exact binaries and locks remain retained. Aggregate counters/hashes cannot independently reconstruct the private corpus. The standalone authored fixture programs are published for reproduction.

Lexical overlap and exact quote checking still do not establish truth or entailment. No improved semantic answer correctness, LLM performance, token savings or new PDF extraction quality is claimed. A useful next performance check is an idle-host repeated construction/MCP profile with the same inputs, followed by targeted candidate-collection/scoring work only if that profile identifies it as costly. Keep exposed regressions separate from any later independent answer-quality evaluation.

## Storage and release handling

Canonical knowledge remains **167,942 entries from 35 PDFs and 49,086 physical pages**, SHA-256 `13168d87872a137355fd8499c90a6084f52243a2ec7456f29b5fcb2a1c7423c3`. Original PDFs, source snapshots, earlier knowledge and all measurements remain. No application database or dependency is changed.

Release packages, exact locks, logs, small measured programs and proof receipts are archived before deleting only the dedicated temporary Cargo and Miri targets after verified publication. No model daemon was started for this release. Default features remain empty and no DenkMaps integration is performed.
