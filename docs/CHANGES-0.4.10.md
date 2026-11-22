# AIRust 0.4.10: construction normalization and ordered candidate merge

This release targets index construction and MCP latency while preserving stability, reliability and the existing API. It adds no model, dependency, feature flag, database migration or DenkMaps integration. The authoritative KnowledgeIndex, stored entries, ranking arithmetic, source positions, knowledge formats, MCP tools and schemas remain.

Observed final results: the real-core build median falls from **6.328 to 5.448 seconds (13.92%)**. Canonical MCP BM25 median latency falls **16.61% / 11.93%** in the two preplanned campaigns, and its p95 also falls in both. TF-IDF is mixed: its second canonical median is slightly higher. The first snapshot BM25 median and snapshot TF-IDF p95 are higher. Resident index/process memory is essentially unchanged. These local observations are not universal guarantees; all tables include slower and higher-byte cases.

## What changed and why

`text::normalize` previously sent pure ASCII through the full NFKC/lowercase/NFKC pipeline. It now handles whole ASCII inputs directly, preserving lowercase conversion and the same whitespace rules. That includes vertical tab, which is Unicode whitespace but is not included in Rust byte is_ascii_whitespace. Leading/trailing whitespace disappears; other controls remain. Any non-ASCII input still uses the general Unicode pipeline. In the retained real corpus, 68,858 of 167,942 texts are ASCII, representing 53,614,192 of 134,584,217 source-text bytes: about 40% of text bytes.

Lexical queries previously inserted every posting ID into a temporary BTreeSet, even though each posting list was already sorted. With up to 32 known distinct terms, a private min-heap now merges those lists into unique sorted borrowed IDs. The final vector borrows existing Arc key values rather than copying wide string views. No ID/text is cloned for this collection. Broader known-term queries retain the prior ordered-set path. Unknown/repeated terms do not increase merge width. The heap is bounded to 32 cursors; total candidate count and process RAM are not hard-bounded by that choice.

Every posting occurrence, including duplicate candidate IDs, retains a cancellation/deadline checkpoint. Collection visit/unique counts remain observable. Scoring traverses the same sorted IDs and sorted terms, keeps the same arithmetic, scope checks, TF-IDF norms, stable top-K ties and matched-term explanations. Shared clones, updates, caches and original source text retain their behavior. The query descriptor holds one additional postings reference per known term, so fallback/wide queries can still request more temporary bytes.

Applications keep the same Engine, Retriever, ExcerptOptions and McpServer calls. MCP uses this central Engine without a second index or adapter. Source binding, exact reads, argument/output byte guards, error codes, progress, absolute deadlines, bounded workers and session rules remain. See [API/performance](CORE-PERFORMANCE.md), [API cookbook](API-COOKBOOK.md), [MCP](MCP.md) and [security boundaries](MCP-SECURITY.md).

## Validation and refinement

The final runtime passes **224 default tests, 872 all-feature checks including 39 documentation examples, 39 isolated feature entries and five Python tests**. Strict Clippy/docs, Rust 1.89 provider-neutral checks and Rust 1.92 all-feature checks pass. API comparison against tagged 0.4.9 reports **229 passes and 31 tool skips**. An independent Rust 1.89 application exercises public retrieval, coverage, quotes, document positions and MCP construction/schema access. The unchanged PDF backend 0.3.5 passes fourteen regressions and the 381-source-file provenance check. The refreshed dependency audit finds zero known vulnerabilities in its checked database, retaining existing informational chrono/paste/ttf-parser maintenance notices.

New oracles compare ASCII normalization against the general Unicode expression, including all ASCII code points, deterministic generated inputs, vertical tab, non-whitespace controls and mixed Unicode. Candidate-set tests cover zero, one, two, seven, 32, 33 and 64 terms, unknown words, overlapping lists, Unicode/prefixed IDs, scopes, both scorers, explanations and updates. Existing full ranking, extreme BM25, cache, observer cancellation, time-budget, shared-clone, hostile MCP input, saturation and disconnect regressions remain.

Nine targeted Tree Borrows Miri cases pass: reference BM25, the two new core oracles, ASCII/mixed excerpt positions, coverage, scope intersection, query fallback and repeated quotation positions. The candidate oracle uses 96 documents normally and six under Miri, preserving both merge-width boundaries. Only extra floating-point rounding injection is disabled; memory/alias checks remain. Async MCP, native PDF and ONNX are outside these finite interpreter cases. This is not an independent security or full soundness audit. Extracted-package tests, exact-commit CI, a fresh registry consumer and VCS/tag/checksum identity are publication gates.

A complete first candidate stored borrowed string views in its merged vector. Its 32-term fixture reduced allocation calls but requested more temporary bytes. That source, binaries, locks, full validation and both complete MCP campaigns remain privately retained. The final candidate instead borrows existing Arc key objects; the entire validation and all planned measurements were repeated. Initial sanitized summaries are kept alongside final records. No slower run was selectively replaced. All final tables below refer to the refined runtime.

## Repeated construction, outside input parsing

The [published applications and method](validation-0.4.10/README.md) compare exact registry 0.4.9 with local candidate 0.4.10. The construction application uses the default feature-free core. It parses or generates owned input records before the Rust timer, then times Engine::from_entries. A buffered reader loads the real JSON; file reading/deserialization and follow-up signatures are outside this timer. Five alternating old/new process pairs per fixture give **forty plain processes**: the complete 167,942-record corpus, 5,000 ASCII records, 5,000 mixed-Unicode records and 500 long-token records. Stats and fixed search signatures agree in every process. These are build-only measurements, not complete server startup.

| Construction only, ms | 0.4.9 median / p95 | 0.4.10 median / p95 | Median change |
| --- | ---: | ---: | ---: |
| canonical | 6328.362 / 6362.074 | 5447.676 / 5470.327 | -13.92% |
| ascii | 43.221 / 43.473 | 15.499 / 16.377 | -64.14% |
| mixed | 34.936 / 36.317 | 35.389 / 35.690 | +1.30% |
| long_tokens | 9.964 / 10.180 | 1.750 / 1.909 | -82.43% |

A negative time change is lower observed latency. The mixed fixture retains the general Unicode path; its small timing difference does not establish a Unicode algorithm improvement. Hash seeds, allocator/cache state, shared-host activity and small process counts remain possible influences. Do not compare these absolute build-only values directly with MCP startup or the differently configured shared-index consistency worker.

Six separate DHAT processes profile the three synthetic distributions. Counter snapshots cover construction only; input records were allocated before tracking and validation queries run after profiling stops. Tracked live bytes therefore describe derived allocations created during construction, not the already owned source records or total process RAM. Instrumented durations are not performance measurements.

| Construction profile | Blocks, old → new | Allocated bytes, old → new | Tracked derived live bytes, old → new |
| --- | ---: | ---: | ---: |
| ascii | 49346 → 49346 | 27569014 → 27569014 | 10144328 → 10144328 |
| mixed | 46234 → 46234 | 18349838 → 18349838 | 8910152 → 8910152 |
| long_tokens | 4241 → 4241 | 5335606 → 5335606 | 1687832 → 1687832 |

Construction allocation counts/bytes and measured retained derived storage are unchanged in these fixtures. This is a CPU-work optimization rather than a claim of a smaller retained index.

## Actual MCP: two preplanned complete campaigns

Each campaign uses five alternating old/new process pairs for the full canonical knowledge and a revision-bound NumPy snapshot: twenty fresh stdio servers per campaign. Both campaigns were planned before final timing. They keep the same inputs, probes, parameters and compiled executables. Discovery, schemas, progress, BM25/TF-IDF, source binding, exact snapshot readback, concurrency, matching, relationships, rejected arguments, health and EOF shutdown pass. Original inputs hash identically before/after every probe.

Across the final forty servers, **2,760 actual tool result/error responses validate against their advertised schemas**, and **1,380 complete old/new structured pairs match by JSON-RPC request ID**, excluding only the explicit status release version. Expected release-qualified schema $id URNs are asserted before comparison. There are 800 sequential timed searches, 160 additional concurrent searches, 1,640 exact snapshot readbacks, eighty matching/relationship calls, forty rejected arguments and forty health checks. Discovery is additional. Progress counts are timing-dependent.

### First final-runtime MCP campaign

| MCP observation | 0.4.9 | 0.4.10 |
| --- | ---: | ---: |
| Canonical startup, median ms | 6350.01 | 5444.46 |
| Canonical BM25, median / p95 ms | 9.809 / 17.195 | 8.180 / 14.422 |
| Canonical TFIDF, median / p95 ms | 29.218 / 52.202 | 27.355 / 48.986 |
| Canonical whole-child peak RSS, median MiB | 1727.80 | 1728.45 |
| Snapshot startup, median ms | 172.20 | 154.12 |
| Snapshot BM25, median / p95 ms | 1.486 / 1.893 | 1.542 / 1.877 |
| Snapshot TFIDF, median / p95 ms | 1.583 / 4.146 | 1.569 / 4.288 |
| Snapshot whole-child peak RSS, median MiB | 57.20 | 57.22 |

### Second preplanned final-runtime MCP campaign

| MCP observation | 0.4.9 | 0.4.10 |
| --- | ---: | ---: |
| Canonical startup, median ms | 5975.05 | 5343.62 |
| Canonical BM25, median / p95 ms | 9.223 / 15.792 | 8.123 / 13.533 |
| Canonical TFIDF, median / p95 ms | 26.949 / 47.078 | 27.182 / 47.166 |
| Canonical whole-child peak RSS, median MiB | 1727.73 | 1727.95 |
| Snapshot startup, median ms | 166.34 | 150.10 |
| Snapshot BM25, median / p95 ms | 1.481 / 1.962 | 1.477 / 1.624 |
| Snapshot TFIDF, median / p95 ms | 1.569 / 4.232 | 1.510 / 3.887 |
| Snapshot whole-child peak RSS, median MiB | 57.20 | 57.17 |

These values include progress and stdio result transport. Follow-up snapshot readbacks are checked outside search timing. Startup includes synchronous loading/indexing and remains outside request deadlines. Whole-child peak RSS includes source parsing, buffers, index and transport; a lower candidate-allocation count is not automatically lower process RAM. Read both sets of medians and tails; no universal latency guarantee follows from one host or five process pairs per case.

## Complete document retrieval and unchanged evidence

The unchanged private corpus contains 72 Markdown files, 2,755,468 source bytes and 1,910 chunks. All 342 questions are exposed regression cases: 276 authored positives and 66 unsupported questions. No new holdout, model reply or independent semantic judgment is added. Literal BM25, prepared BM25 and coverage use six passages, 100 candidates, 2,000 bytes per passage and 12,000 total source-text bytes.

All **1,026 complete response JSON strings match byte-for-byte**, including scores, terms, IDs, order, text, positions, recipes and selection details. Every repeat also matches its first response. Previous misses remain; no semantic-answer improvement or token saving is claimed. One warm-up precedes five samples per question/configuration/version, giving **10,260 warm timed round trips**. Timing includes evidence selection, serialization, IPC and Python parsing, not only scoring.

| Complete context round trip, median / p95 ms | 0.4.9 | 0.4.10 |
| --- | ---: | ---: |
| Literal BM25 | 0.934 / 1.457 | 0.831 / 1.283 |
| Prepared BM25 | 1.027 / 1.622 | 0.893 / 1.424 |
| Coverage | 1.024 / 1.636 | 0.880 / 1.409 |

Of 342 per-question median pairs, 16 literal, 11 prepared and 23 coverage cases are slower with the candidate. Aggregate improvements must not hide individual regressions or sampling variation. All scalar timings and outcomes remain available.

## Query allocations and the merge-width boundary

Sixteen authored fixtures retain the previous fourteen and add exactly 32/33 known terms over a separate prebuilt 256-entry index. Other Engine cases use the same prebuilt 1,000-entry index. Seven alternating plain process pairs measure 1,000 calls per fixture: **224,000 timed calls in fourteen processes**. Two separate instrumented builds measure 100 calls per fixture, 3,200 calls total. Signatures agree; per-batch retained-byte/block deltas are zero in both versions. DHAT durations are not used as latency results.

| Fixture | Blocks per call, old → new | Bytes per call, old → new | Median time change |
| --- | ---: | ---: | ---: |
| `coverage_repeated` | 1204 → 1047 | 110462 → 92942 | -49.99% |
| `coverage_small` | 941 → 784 | 95566 → 78046 | -49.17% |
| `excerpt_ascii` | 762 → 762 | 252438 → 252438 | -0.77% |
| `excerpt_mixed` | 569 → 569 | 417113 → 417113 | -22.79% |
| `excerpt_unicode` | 197 → 197 | 72254 → 72254 | -3.08% |
| `long_repeated` | 4 → 4 | 18798 → 18798 | -92.43% |
| `mixed_unicode` | 7 → 7 | 3155 → 3155 | +2.35% |
| `punctuation` | 1 → 1 | 768 → 768 | +1.22% |
| `relevant_retrieval` | 969 → 812 | 111226 → 93562 | -44.22% |
| `repeated_search` | 196 → 39 | 37640 → 19736 | -53.28% |
| `repeated_short` | 4 → 4 | 1656 → 1656 | -89.20% |
| `union_32` | 274 → 244 | 20410 → 19834 | -4.76% |
| `union_33` | 296 → 296 | 30206 → 31198 | -2.36% |
| `unique_long` | 299 → 299 | 32672 → 32672 | -83.84% |
| `unique_search` | 229 → 72 | 37014 → 19350 | -51.88% |
| `unique_short` | 296 → 296 | 18548 → 18548 | -59.56% |

Negative time changes are faster in those fixtures. The broad fallback can still request more bytes because query descriptors retain a postings reference. Allocation calls, cumulative bytes, retained heap and whole-process RSS are different measures. The final Arc-key vector replaces the first candidate's wider string views; initial counters remain in separate records. Small gains/losses on unchanged paths are not statistical proof. No query distribution is guaranteed to improve.

## Large-corpus consistency and scope

A full shared-index/execution worker in each version additionally validates all 167,942 records through 128 updates/deletions/restoration, rebuilding, JSON round trips, four concurrent readers with forty queries each, retained-clone isolation and cancellation/deadline rejection. Source and portable export hashes agree. This is finite consistency validation, not a million-user capacity test or new PDF parser benchmark.

The separate consistency worker observes initial build times 6.687 / 5.952 seconds and complete child runs 25.197 / 26.364 seconds for old/new. These single observations use execution/shared-index and include a different surrounding workload; the repeated feature-free construction experiment above is the more useful construction comparison.

Measurements use one AC-powered Apple M5 Max with 64 GB, Rust 1.98.0 and Python 3.14.7. Managed compiler commands finish before timed stages. Stage-start host snapshots are retained as aggregate CPU/compiler activity counts; they do not establish continuously idle or isolated hardware. Other workload, random hash seeds, scheduling, caches, exposed authored questions and small process-pair counts limit generalization. A causal explanation for every timing difference is not established. See [all sanitized records and their scopes](measurements/0.4.10/README.md). Private raw sources/transcripts and exact binaries/locks remain retained; aggregate hashes/counters cannot reconstruct the private corpus. Standalone synthetic fixture programs are public.

Lexical overlap, valid JSON and exact quotations still do not establish truth or entailment. There are no fresh LLM answers, semantic-quality gains, token-saving claims or new PDF extraction-speed results. Further optimization should be driven by an isolated profile of remaining parsing/scoring/transport costs and should preserve these regression inputs and API boundaries.

## Knowledge and build storage

Canonical knowledge remains **167,942 entries from 35 PDFs and 49,086 physical pages**, SHA-256 `13168d87872a137355fd8499c90a6084f52243a2ec7456f29b5fcb2a1c7423c3`. PDFs, source snapshots, original knowledge, the complete initial candidate and all final measurements remain. No application database is changed.

Verified release packages, test receipts, exact locks and small measured programs are archived before removing only the dedicated temporary Cargo and Miri targets after publication. No model daemon was started. Default features remain empty, and no DenkMaps integration is performed.
