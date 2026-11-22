# AIRust 0.4.11: TF-IDF hints and sparse broad-query scoring

This release improves existing TF-IDF and broad lexical execution. Public signatures, ranking arithmetic, deterministic ties, source text/positions, knowledge formats and MCP schemas remain. No dependency, feature flag, model, database migration or DenkMaps integration is added.

Observed results are specific: repeated canonical MCP TF-IDF medians are **15.29% and 16.67% lower** in two preplanned campaigns. Authored sparse 512-term BM25/TF-IDF calls are about **74% faster** in their isolated fixtures. This does not translate into a 74% end-to-end MCP gain: synthetic wide MCP changes are small and mixed, and the second snapshot campaign has slower medians. Several unchanged microfixtures also run slower. Cache allocation/retained-memory costs vary and are included below.

## Implementation and unchanged contracts

The transient norm-hint dictionary changes from BTreeMap to Rust's default randomly seeded HashMap. It is used only for lookup, not to enumerate ranking contributions or saved data. Collision protection remains. Complete norms are published only after checkpoints. Lexical/corpus changes still invalidate hints; metadata-only changes retain them. Contended/poisoned locks still fall back to exact nonblocking computation. BM25 and unknown-word queries still avoid allocating a norm cache. Hash-table growth/reservation can cost more allocated or retained bytes; this is not an unconditional memory optimization.

Candidate sets for more than 32 known terms now borrow existing Arc key values instead of wider string slices. Sorted unique IDs and original ranking traversal remain. Sparse broad scoring chooses document-word iteration only when the prepared query has more than 32 known distinct terms and exceeds twice the document's unique word count. Its sorted words binary-search the sorted prepared query; other cases retain query-word lookups. Both routes add matched contributions in the same lexical order and retain exact multiplication/division. Every 128 inspected terms reaches a checkpoint. Winner explanations remain unchanged.

The authoritative KnowledgeIndex, source ownership, shared clones, caches and update path stay centralized. There is no second index or public tuning option. Existing Engine/SearchOptions, Retriever and McpServer callers use the same calls, including SearchAlgorithm::Tfidf or MCP algorithm tfidf. Source binding, byte budgets, worker admission, progress, cancellation and absolute time limits remain. See [API/performance](CORE-PERFORMANCE.md), [cookbook](API-COOKBOOK.md), [MCP](MCP.md) and [security scope](MCP-SECURITY.md).

## Executed validation

The full matrix passes **226 default tests, 874 all-feature checks including 39 documentation examples, 39 isolated feature entries and five Python tests**. Strict Clippy/docs and Rust 1.89 provider-neutral / Rust 1.92 all-feature checks pass. API comparison against tagged 0.4.10 reports **229 passes and 31 tool skips**. A separate Rust 1.89 application exercises retrieval, coverage, quotations, document positions and MCP construction/schema access. The unchanged PDF backend 0.3.5 passes fourteen regressions and its 381-source-file provenance check. A refreshed audit reports zero known vulnerabilities in its checked database, with existing informational chrono/paste/ttf-parser maintenance notices.

A new independent full scan compares exact BM25 and TF-IDF score bits, IDs, order and explanations across widths 32/33/64/129/256, sparse/dense entries, repeated/unknown terms, scopes and cold/repeated caches. A private checkpoint regression interrupts sparse scoring before it can publish a result. Existing norm invalidation, metadata reuse, poison/contention, incomplete-norm rejection, shared-clone, cancellation/deadline and hostile MCP cases pass.

Ten finite Tree Borrows Miri cases pass, including the new score oracle, prior normalization/candidate oracles, source positions, scope, coverage, query fallback and repeated quote ranges. The new oracle uses four sparse entries plus a vocabulary entry under Miri and 48 sparse entries normally. Only extra floating-point rounding injection is disabled; memory/alias checks remain. Async/native PDF/ONNX work is outside these cases. This is not an independent security or full soundness audit. Extracted-package tests, exact-commit CI, fresh exact-registry consumer and VCS/tag/checksum matching are publication gates.

## Query working costs and broad requests

The [public method/programs](validation-0.4.11/README.md) compare exact registry 0.4.10 with candidate 0.4.11. Retain the previous sixteen fixtures and add nine TF-IDF/sparse-broad cases. The old Engine has 1,000 entries, the original wide fixture has 256 entries/64 terms, and the added sparse fixture has 256 short entries plus one 512-term vocabulary entry. Inputs are authored distributions, not production traffic.

Seven alternating old/new process pairs give fourteen plain processes. Nineteen fixtures use 1,000 calls per sample; six sparse broad fixtures use 200, giving **282,800 timed calls**. Two separate DHAT builds run 100 calls per fixture, 5,000 calls total. Warm queries initialize their hints before timing/profiling. Signatures agree and per-batch temporary-call retained-byte/block deltas are zero in both versions. Instrumented durations are not performance measurements.

| Fixture | Blocks per call, old → new | Bytes per call, old → new | Median time change |
| --- | ---: | ---: | ---: |
| `coverage_repeated` | 1047 → 1047 | 92942 → 92942 | -0.40% |
| `coverage_small` | 784 → 784 | 78046 → 78046 | +1.33% |
| `excerpt_ascii` | 762 → 762 | 252438 → 252438 | -0.74% |
| `excerpt_mixed` | 569 → 569 | 417113 → 417113 | +2.07% |
| `excerpt_unicode` | 197 → 197 | 72254 → 72254 | +0.67% |
| `long_repeated` | 4 → 4 | 18798 → 18798 | +15.21% |
| `mixed_unicode` | 7 → 7 | 3155 → 3155 | -0.63% |
| `punctuation` | 1 → 1 | 768 → 768 | +3.48% |
| `relevant_retrieval` | 812 → 812 | 93562 → 93562 | -1.30% |
| `repeated_search` | 39 → 39 | 19736 → 19736 | +5.70% |
| `repeated_short` | 4 → 4 | 1656 → 1656 | +5.06% |
| `sparse_bm25_128` | 359 → 359 | 29954 → 29074 | -48.93% |
| `sparse_bm25_33` | 241 → 241 | 16661 → 16309 | -10.14% |
| `sparse_bm25_512` | 1337 → 1337 | 122820 → 119124 | -73.59% |
| `sparse_tfidf_128` | 474 → 474 | 36020 → 35140 | -47.65% |
| `sparse_tfidf_33` | 241 → 241 | 16661 → 16309 | -10.17% |
| `sparse_tfidf_512` | 1337 → 1337 | 122820 → 119124 | -74.29% |
| `tfidf_union_32` | 244 → 244 | 19834 → 19834 | -2.11% |
| `tfidf_union_33` | 296 → 296 | 31198 → 27854 | -1.93% |
| `tfidf_warm` | 71 → 71 | 19335 → 19335 | -16.72% |
| `union_32` | 244 → 244 | 19834 → 19834 | -2.27% |
| `union_33` | 296 → 296 | 31198 → 27854 | -2.68% |
| `unique_long` | 299 → 299 | 32672 → 32672 | +9.11% |
| `unique_search` | 72 → 72 | 19350 → 19350 | +0.05% |
| `unique_short` | 296 → 296 | 18548 → 18548 | +2.87% |

Negative changes are lower observed latency. The approximately 74% reductions apply to the authored sparse 512-term calls, not every query or transport. Unchanged cases include slower long-repeated normalization (+15.21%), unique-long normalization (+9.11%) and repeated search (+5.70%). All slower rows remain visible. Binary layout, hash seeds, caches, scheduling and small process-pair counts are not isolated; these are not significance tests or universal latency guarantees.

## Cold norm-hint storage: fewer blocks is not always fewer bytes

Two additional instrumented processes separately initialize cold hints for three fixed inputs, then drop returned result vectors before reading retained deltas. Original sources and completed index setup are outside tracking. These are hint allocations, not total index/process memory. Fixed result signatures agree.

| Cold hint fixture | Allocation blocks, old → new | Allocated bytes, old → new | Retained hint bytes, old → new |
| --- | ---: | ---: | ---: |
| index | 206 → 49 | 67960 → 121300 | 49120 → 51288 |
| wide | 80 → 55 | 16177 → 19517 | 9496 → 6488 |
| sparse | 1381 → 1346 | 135492 → 144768 | 12672 → 12888 |

For the 1,000-entry fixture, retained hint bytes increase from 49,120 to 51,288, while cumulative allocation traffic rises from 67,960 to 121,300 bytes despite fewer allocation calls. Hash-table growth explains an additional cost to retain and measure, not a promise of lower cold work. The original wide fixture retains fewer hint bytes; the sparse fixture retains slightly more. Do not generalize one load factor or fixture into a RAM guarantee. First-pass latency and steady repeated requests are reported separately next.

## Actual MCP: initial pass and repeated passes separated

Both complete campaigns are preplanned. Each uses canonical 167,942-entry knowledge, the revision-bound NumPy snapshot and the deterministic synthetic broad corpus, with five alternating old/new process pairs per input: **thirty servers per campaign, sixty total**. The synthetic MCP requests have 32/33/64/128 terms within existing argument/excerpt limits; the Core-only 512-term fixtures are a different scope.

Each server executes every query/algorithm once, recording initial cold/partial-cache samples. Two complete later passes are timed separately and must match their initial full structured payload exactly. A first TF-IDF call is shown separately from the aggregate initial pass. Older campaign tables mixed cache states; do not compare their absolute values directly with this new method. Both crate versions receive the same changed probe, inputs, limits and compiled program configuration.

Across both campaigns, **5,720 actual tool result/error responses validate against advertised schemas**, and **2,860 complete old/new structured pairs match by JSON-RPC request ID**, excluding only explicit status version and asserted release-qualified schema URNs. Discovery, source binding, initial exact snapshot readbacks, progress, concurrency, matching/relations, rejected arguments, health and EOF shutdown pass. There are 1,200 initial sequential searches, 2,400 repeated sequential searches, 240 additional concurrent requests, 1,640 exact snapshot readbacks, 120 matching/relationship calls, sixty rejected-argument calls and sixty health calls. Discovery is additional. All input hashes remain unchanged.

### First complete campaign

| MCP observation, ms unless stated | 0.4.10 | 0.4.11 |
| --- | ---: | ---: |
| Canonical startup median | 4900.13 | 5312.60 |
| Canonical first TF-IDF call median | 24.782 | 25.860 |
| Canonical initial pass TFIDF, median / p95 | 24.620 / 53.457 | 23.858 / 40.140 |
| Canonical repeated BM25, median / p95 | 6.607 / 17.354 | 6.078 / 11.832 |
| Canonical repeated TFIDF, median / p95 | 7.184 / 21.594 | 6.085 / 14.084 |
| Canonical whole-child peak RSS, median MiB | 1728.23 | 1726.73 |
| Snapshot startup median | 156.19 | 145.46 |
| Snapshot first TF-IDF call median | 1.961 | 1.789 |
| Snapshot initial pass TFIDF, median / p95 | 1.521 / 3.847 | 1.504 / 3.791 |
| Snapshot repeated BM25, median / p95 | 1.442 / 1.669 | 1.436 / 1.592 |
| Snapshot repeated TFIDF, median / p95 | 1.457 / 1.932 | 1.457 / 1.801 |
| Snapshot whole-child peak RSS, median MiB | 57.36 | 57.28 |
| Synthetic wide startup median | 4.05 | 4.06 |
| Synthetic wide first TF-IDF call median | 1.631 | 1.627 |
| Synthetic wide initial pass TFIDF, median / p95 | 1.470 / 1.622 | 1.469 / 1.624 |
| Synthetic wide repeated BM25, median / p95 | 1.455 / 1.502 | 1.446 / 1.495 |
| Synthetic wide repeated TFIDF, median / p95 | 1.472 / 1.656 | 1.460 / 1.676 |
| Synthetic wide whole-child peak RSS, median MiB | 7.89 | 7.89 |

### Second preplanned campaign

| MCP observation, ms unless stated | 0.4.10 | 0.4.11 |
| --- | ---: | ---: |
| Canonical startup median | 5188.23 | 4655.47 |
| Canonical first TF-IDF call median | 26.688 | 26.338 |
| Canonical initial pass TFIDF, median / p95 | 24.933 / 41.966 | 24.066 / 39.968 |
| Canonical repeated BM25, median / p95 | 6.384 / 12.528 | 5.486 / 10.976 |
| Canonical repeated TFIDF, median / p95 | 6.902 / 14.548 | 5.752 / 12.649 |
| Canonical whole-child peak RSS, median MiB | 1728.19 | 1725.23 |
| Snapshot startup median | 132.88 | 130.98 |
| Snapshot first TF-IDF call median | 1.629 | 1.576 |
| Snapshot initial pass TFIDF, median / p95 | 1.505 / 3.693 | 1.516 / 3.637 |
| Snapshot repeated BM25, median / p95 | 1.442 / 1.559 | 1.459 / 1.614 |
| Snapshot repeated TFIDF, median / p95 | 1.474 / 1.697 | 1.505 / 1.647 |
| Snapshot whole-child peak RSS, median MiB | 57.14 | 57.19 |
| Synthetic wide startup median | 4.31 | 4.38 |
| Synthetic wide first TF-IDF call median | 1.642 | 1.644 |
| Synthetic wide initial pass TFIDF, median / p95 | 1.444 / 1.641 | 1.447 / 1.647 |
| Synthetic wide repeated BM25, median / p95 | 1.400 / 1.465 | 1.393 / 1.453 |
| Synthetic wide repeated TFIDF, median / p95 | 1.406 / 1.943 | 1.401 / 1.619 |
| Synthetic wide whole-child peak RSS, median MiB | 7.77 | 7.94 |

Canonical repeated TF-IDF medians improve in both campaigns. Initial canonical TF-IDF medians improve more modestly. Small snapshots and synthetic wide requests do not gain uniformly: snapshot medians in the second campaign are higher, and the first wide repeated TF-IDF p95 is higher. The isolated broad scoring gain is therefore not a comparable end-to-end transport gain. Peak process RSS is essentially unchanged at this scale and includes parsing, source buffers, index, hints and transport.

Search timing includes progress and stdio response transport. Initial snapshot follow-up reads occur outside the search timer; repeated full payloads must match those initially validated values. Startup is synchronous loading/indexing outside request deadlines. A faster helper does not create a hard process-time or memory quota.

## Complete document retrieval and construction controls

The fixed private corpus contains 72 Markdown files, 2,755,468 source bytes and 1,910 chunks. All 342 authored questions are exposed regressions: 276 positives and 66 unsupported questions. No new holdout, model reply or independent semantic judgment is added. Literal BM25, prepared BM25 and coverage retain six passages, 100 candidates, 2,000 bytes per passage and 12,000 total text bytes. These three document configurations use BM25; TF-IDF equality is checked by the independent oracle and real protocol comparisons.

All **1,026 complete document response JSON strings match byte-for-byte**, including scores, terms, IDs, order, text, source positions, recipes and selection details. Every repeated sample also matches its first response. Previous misses remain. One warm-up and five samples per question/configuration/version give **10,260 timed round trips**, including source work, evidence selection, serialization, IPC and Python parsing.

| Document context, ms | 0.4.10 median / p95 | 0.4.11 median / p95 |
| --- | ---: | ---: |
| baseline | 0.735 / 1.106 | 0.752 / 1.127 |
| prepared | 0.815 / 1.239 | 0.820 / 1.244 |
| evidence | 0.805 / 1.229 | 0.818 / 1.260 |

Of 342 per-query median pairs, 240 literal, 167 prepared and 242 coverage cases are slower with the candidate. All scalar outcomes/timings remain.

Construction is a retained control, not this release's target. Five alternating pairs for full canonical input and three synthetic distributions give forty plain processes, with input parsing/generation outside the Rust construction timer. Six separate synthetic profiles record derived build allocations; already owned source records are excluded. Stats and fixed search signatures agree, and measured derived live bytes agree.

| Construction only, ms | 0.4.10 median / p95 | 0.4.11 median / p95 |
| --- | ---: | ---: |
| canonical | 4563.671 / 4602.632 | 4581.744 / 4618.639 |
| ascii | 14.173 / 14.568 | 14.071 / 14.141 |
| mixed | 33.002 / 33.388 | 33.104 / 34.005 |
| long_tokens | 1.491 / 1.539 | 1.516 / 1.559 |

A separate execution/shared-index worker validates all 167,942 entries through 128 changes/restoration, rebuilds, JSON round trips, four readers with forty queries each, retained-clone isolation and cancellation/deadline rejection. Source and portable exports match. This is finite consistency validation, not a new PDF parser or million-user capacity test.

## Conditions, preserved knowledge and limits

Measurements use one AC-powered Apple M5 Max with 64 GB, Rust 1.98.0 and Python 3.14.7. Managed compiler commands finish before timed stages. Stage-start host CPU/compiler snapshots are retained; they do not establish continuously idle hardware or attribute every change. Exposed authored inputs, one corpus, three MCP datasets, caches and small process counts limit generalization. See [all sanitized records](measurements/0.4.11/README.md). Raw sources/transcripts, profiles, exact locks/binaries and logs remain private. Public aggregate hashes/counters cannot reconstruct the private corpus; the standalone synthetic programs and generator are published.

Lexical overlap and exact quotations still do not establish truth or entailment. There is no semantic-quality, token-saving, model-speed or new PDF extraction claim. Future profiling should investigate remaining real cold-cache/transport costs separately, while retaining exact-output and resource checks.

Canonical knowledge remains **167,942 entries from 35 PDFs and 49,086 physical pages**, SHA-256 `13168d87872a137355fd8499c90a6084f52243a2ec7456f29b5fcb2a1c7423c3`. PDFs, snapshots, original knowledge and every measurement remain. No application database changes. Packages, locks, logs and small exact comparison programs are archived before deleting only named temporary Cargo and Miri targets after verified publication. No model daemon was started. Default features remain empty, without DenkMaps integration.
