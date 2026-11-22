# AIRust 0.4.13: bounded cold TF-IDF work

This release targets TF-IDF norm computation during first requests, including optional MCP. It keeps exact scoring arithmetic, deterministic ties, public Rust calls, source positions, saved knowledge formats and MCP schemas. No dependency, feature flag, model, database migration or DenkMaps integration is added.

On the canonical MCP input, initial TF-IDF pass median changes are **-23.25% and -14.90%** in two preplanned campaigns. Repeated TF-IDF median changes are **+0.37% and +0.91%**. Negative changes are shorter observed latency. First calls, later initial requests and repeated calls are distinct scopes. All small-input, slower and temporary-memory costs remain visible below; these observations are not a universal speed guarantee.

## Implementation and resource contract

One query can reuse exact IDF word weights while computing missing document norms. A private direct-mapped memo has 256 slots. Each contains a full live interned-term allocation identity and the original f64 IDF. Compare the full key before reuse; a collision computes the original formula from the current corpus document frequency and replaces the slot. Repeated slots do not grow a hash table. The index is immutably borrowed for the query, so those term allocations and frequencies stay valid. No key is dereferenced, serialized or accepted as a public ID; no unsafe code is added.

The memo is a lazy Box: **4,096 payload bytes and one additional allocation** on the measured 64-bit build when a norm must actually be computed. Allocator/temporary stack costs are additional. It is dropped on completion, cancellation or failure. BM25, unknown-word requests and complete warm norm hits allocate no memo. Persistent document-norm storage retains the compact 0.4.12 keys, invalidation, complete-value publication and nonblocking contention/poison fallback. Existing shared clones retain coherent ownership. There is no new index field or retained corpus-sized IDF table.

Document norm terms retain their original sorted order, floating-point multiplication/addition/square-root operations and checkpoints every 128 inspected terms. Reusing an exact weight changes neither scores nor explanations. Norms remain lazy: startup does not compute or prewarm them. The improvement targets executed request work instead of moving that work to startup. The fixed memo can still add cost for tiny or collision-heavy inputs, and warm code/layout costs can differ.

Use existing `Engine::search_with_options` and `SearchAlgorithm::Tfidf`, or MCP `airust_search` with `algorithm: "tfidf"`. No new setting is needed. Scopes, original IDs, source binding, spans, worker admission, byte budgets, progress, cancellation and absolute deadlines remain. Read [API](API.md), [cookbook](API-COOKBOOK.md), [performance](CORE-PERFORMANCE.md), [MCP](MCP.md) and [security scope](MCP-SECURITY.md).

## Executed validation

The complete matrix passes **231 default tests, 879 all-feature checks including 39 documentation examples, 39 isolated feature entries and five Python tests**. Strict Clippy/docs, Rust 1.89 provider-neutral and Rust 1.92 all-feature checks pass. API comparison against tagged 0.4.12 reports **229 pass, 31 skip**. A separate Rust 1.89 consumer exercises retrieval, coverage, quotations, document positions and MCP/schema construction. The unchanged PDF backend passes 14 regressions and its 381-source-file provenance check. A refreshed audit reports zero known dependency vulnerabilities in the checked database, retaining existing informational chrono/paste/ttf-parser maintenance notices.

New regressions force memo-slot collisions and verify exact recomputation, as well as no memo allocation for an interrupted first norm or a complete warm hit. Existing concurrent cold norm queries, identifier ownership, retained-clone/reused-ID, metadata reuse, mutation invalidation, poison/contention, incomplete-value rejection, independent full-scan score-bit oracles, scopes and hostile protocol tests pass. The 0.4.12 stress-example preparation/publication conflict fix and deterministic example test remain in the full test runner.

Fourteen finite Tree Borrows Miri cases pass, including both new memo regressions and twelve prior score/ownership/source/scope/query/quote cases. Only extra floating-point rounding injection is disabled; memory/alias checks remain. Async/native PDF/ONNX execution is outside these cases. This is not an independent security or soundness audit. Extracted-package tests, exact-commit CI and an exact-registry consumer with matching VCS/tag/checksum are publication gates.

## Method and development pilot

The [programs and reproduction method](validation-0.4.13/README.md) compare exact registry 0.4.12 (VCS 15bed6535d6e4d575f6af9cc20cbd0d915193753) with the candidate using the same programs/inputs. Source, method, locks and exact executable hashes are retained through the final measurements. Managed compilers finish before timed stages.

A separate development pilot used three old/new canonical MCP pairs with a retained 0.4.12 executable whose measured hash and published library source were checked. Its complete structured response pairs matched apart from explicit status version. That pilot motivated the implementation but is not merged into the final measurements; its slower warm observation, binaries, source identities and raw records remain private. Both final MCP campaigns were planned before final timing. No rejected performance sample is replaced or selected away.

## Cold norm calls and retained hint heap

Four fixed fixtures cover the prior 1,000-entry index, 256-entry wide index, 257-entry sparse/vocabulary index and a new one-entry tiny control. In seven alternating old/new process pairs, each plain process makes twenty first calls per fixture using fresh clones of a never-queried Engine. Clone/setup work is outside the timer. There are **1,120 cold norm-cache calls across fourteen processes**, with equal signatures. Process/instruction/filesystem caches are not flushed; this is not a cold-disk measurement. No instrumented duration is used as latency data.

| Cold norm-cache call, ns | 0.4.12 median / p95 | 0.4.13 median / p95 |
| --- | ---: | ---: |
| index | 169291.500 / 182498.250 | 111771.000 / 127398.200 |
| wide | 84583.000 / 90881.250 | 33395.500 / 40840.200 |
| sparse | 507750.500 / 559071.150 | 474124.500 / 518635.100 |
| tiny | 583.500 / 1211.050 | 667.000 / 1291.050 |

Two separate instrumented processes measure first-query allocation traffic and retained hint bytes after dropping result vectors, excluding original sources/index setup. Allocated bytes include query scratch and the temporary memo; retained deltas are persistent hint growth. These are different costs.

| Cold hint fixture | Allocation blocks, old → new | Allocated bytes, old → new | Retained hint bytes, old → new |
| --- | ---: | ---: | ---: |
| index | 49 → 50 | 88564 → 92660 | 34904 → 34904 |
| wide | 55 → 56 | 15453 → 19549 | 4440 → 4440 |
| sparse | 1346 → 1347 | 136608 → 140704 | 8792 → 8792 |
| tiny | 12 → 13 | 1014 → 5110 | 156 → 156 |

A further two instrumented processes load all 167,942 canonical entries before profiling and execute the same ten fixed queries used by MCP. Folded score signatures agree. Returned result vectors are dropped before net heap deltas. This isolates hint growth rather than whole-process RAM.

| Canonical initial TF-IDF pass | 0.4.12 | 0.4.13 |
| --- | ---: | ---: |
| Net retained norm-hint bytes | 2228312 | 2228312 |
| Net retained norm-hint blocks | 2 | 2 |
| Total allocated query bytes | 7183427 | 7216195 |
| Total query allocation blocks | 435 | 443 |

The memo adds temporary allocation even when retained norm storage is identical. Tiny queries can pay for a 4 KiB memo to avoid little repeated work. Whole-process RSS also includes source buffers, index trees, runtime and transport, so one component's footprint cannot be read as the same change in application RAM. The fixed memo capacity is not a total-process memory quota. The one-entry cold median is 14.31% higher (583.5 to 667 ns); its first query requests 5,110 instead of 1,014 total bytes. The larger cold fixtures have lower medians, so tiny-input overhead must not be hidden by their gains.

## Warm-query and allocation controls

All 25 prior fixtures remain. Seven alternating old/new process pairs give fourteen plain processes and **282,800 timed calls**: nineteen fixtures use 1,000 calls per sample; six sparse broad fixtures use 200. Two separate DHAT builds run 100 calls per fixture, 5,000 calls total. Warm-ups initialize norms before measurement. Signatures match, with zero per-batch temporary-call retained-byte/block deltas. Instrumented durations are excluded from timing comparisons.

| Fixture | Blocks per call, old → new | Bytes per call, old → new | Median time change |
| --- | ---: | ---: | ---: |
| `coverage_repeated` | 1047 → 1047 | 92942 → 92942 | -1.09% |
| `coverage_small` | 784 → 784 | 78046 → 78046 | -0.51% |
| `excerpt_ascii` | 762 → 762 | 252438 → 252438 | +1.43% |
| `excerpt_mixed` | 569 → 569 | 417113 → 417113 | -0.97% |
| `excerpt_unicode` | 197 → 197 | 72254 → 72254 | +1.42% |
| `long_repeated` | 4 → 4 | 18798 → 18798 | +0.36% |
| `mixed_unicode` | 7 → 7 | 3155 → 3155 | -1.73% |
| `punctuation` | 1 → 1 | 768 → 768 | -2.49% |
| `relevant_retrieval` | 812 → 812 | 93562 → 93562 | -0.28% |
| `repeated_search` | 39 → 39 | 19736 → 19736 | -3.08% |
| `repeated_short` | 4 → 4 | 1656 → 1656 | -1.23% |
| `sparse_bm25_128` | 359 → 359 | 29074 → 29074 | -0.51% |
| `sparse_bm25_33` | 241 → 241 | 16309 → 16309 | -1.44% |
| `sparse_bm25_512` | 1337 → 1337 | 119124 → 119124 | +1.78% |
| `sparse_tfidf_128` | 474 → 474 | 35140 → 35140 | -0.35% |
| `sparse_tfidf_33` | 241 → 241 | 16309 → 16309 | -0.23% |
| `sparse_tfidf_512` | 1337 → 1337 | 119124 → 119124 | +0.96% |
| `tfidf_union_32` | 244 → 244 | 19834 → 19834 | -0.91% |
| `tfidf_union_33` | 296 → 296 | 27854 → 27854 | +1.36% |
| `tfidf_warm` | 71 → 71 | 19335 → 19335 | -0.20% |
| `union_32` | 244 → 244 | 19834 → 19834 | +0.56% |
| `union_33` | 296 → 296 | 27854 → 27854 | +2.62% |
| `unique_long` | 299 → 299 | 32672 → 32672 | -0.18% |
| `unique_search` | 72 → 72 | 19350 → 19350 | +0.02% |
| `unique_short` | 296 → 296 | 18548 → 18548 | -0.93% |

9 of the 25 warm fixture medians are slower. All positive rows remain. Most of these paths are unchanged; binary layout, hash seeds, caches, scheduling and small process-pair counts are not isolated. These are not significance tests or a promise that every warm call keeps the same latency. Temporary allocation counts and exact result equality are separate from runtime measurements.

## Actual MCP: initial pass and repeated passes

Each of two preplanned campaigns uses canonical 167,942-entry knowledge, a revision-bound NumPy snapshot and deterministic synthetic broad input, five alternating old/new pairs per input. That is thirty servers per campaign, sixty total. Synthetic requests use 32/33/64/128 terms within the existing argument/excerpt limits. Each server executes every query/algorithm once, then two repeated passes which must match their initial full structured payload.

Across both campaigns, **5,720 actual result/error responses validate against advertised schemas**, and **2,860 complete old/new structured pairs match by JSON-RPC request ID**, excluding only explicit status version and asserted release-qualified schema URNs. Discovery/source binding, exact initial snapshot readbacks, progress, concurrency, matching/relations, rejected arguments, health and EOF shutdown pass. Counts are 1,200 initial searches, 2,400 repeated searches, 240 concurrent calls, 1,640 readbacks, 120 matching/relationship calls, 60 rejected-argument calls and 60 health calls. Source hashes remain unchanged.

### First complete campaign

| MCP observation, ms unless stated | 0.4.12 | 0.4.13 |
| --- | ---: | ---: |
| Canonical startup median | 4477.33 | 4453.14 |
| Canonical first TF-IDF call median | 22.669 | 19.482 |
| Canonical initial pass TFIDF, median / p95 | 22.299 / 36.484 | 17.115 / 31.477 |
| Canonical repeated BM25, median / p95 | 5.479 / 10.380 | 5.237 / 9.862 |
| Canonical repeated TFIDF, median / p95 | 5.253 / 11.782 | 5.273 / 9.902 |
| Canonical whole-child peak RSS, median MiB | 1724.14 | 1724.16 |
| Snapshot startup median | 127.62 | 126.47 |
| Snapshot first TF-IDF call median | 1.521 | 1.488 |
| Snapshot initial pass TFIDF, median / p95 | 1.517 / 3.666 | 1.487 / 3.343 |
| Snapshot repeated BM25, median / p95 | 1.488 / 1.598 | 1.469 / 1.601 |
| Snapshot repeated TFIDF, median / p95 | 1.506 / 1.675 | 1.501 / 1.676 |
| Snapshot whole-child peak RSS, median MiB | 57.06 | 57.22 |
| Synthetic wide startup median | 4.13 | 4.41 |
| Synthetic wide first TF-IDF call median | 1.484 | 1.443 |
| Synthetic wide initial pass TFIDF, median / p95 | 1.466 / 1.541 | 1.465 / 1.567 |
| Synthetic wide repeated BM25, median / p95 | 1.464 / 1.607 | 1.456 / 1.608 |
| Synthetic wide repeated TFIDF, median / p95 | 1.479 / 1.585 | 1.474 / 1.574 |
| Synthetic wide whole-child peak RSS, median MiB | 8.03 | 7.98 |

### Second preplanned campaign

| MCP observation, ms unless stated | 0.4.12 | 0.4.13 |
| --- | ---: | ---: |
| Canonical startup median | 4998.37 | 4845.37 |
| Canonical first TF-IDF call median | 25.527 | 22.078 |
| Canonical initial pass TFIDF, median / p95 | 24.070 / 42.128 | 20.483 / 41.888 |
| Canonical repeated BM25, median / p95 | 6.005 / 11.345 | 6.033 / 12.347 |
| Canonical repeated TFIDF, median / p95 | 6.209 / 12.805 | 6.265 / 15.249 |
| Canonical whole-child peak RSS, median MiB | 1724.17 | 1724.30 |
| Snapshot startup median | 141.55 | 140.30 |
| Snapshot first TF-IDF call median | 1.760 | 1.678 |
| Snapshot initial pass TFIDF, median / p95 | 1.599 / 3.905 | 1.517 / 3.323 |
| Snapshot repeated BM25, median / p95 | 1.487 / 1.718 | 1.431 / 1.723 |
| Snapshot repeated TFIDF, median / p95 | 1.510 / 3.165 | 1.455 / 2.063 |
| Snapshot whole-child peak RSS, median MiB | 57.39 | 57.34 |
| Synthetic wide startup median | 5.09 | 4.53 |
| Synthetic wide first TF-IDF call median | 1.546 | 1.466 |
| Synthetic wide initial pass TFIDF, median / p95 | 1.590 / 2.268 | 1.513 / 2.002 |
| Synthetic wide repeated BM25, median / p95 | 1.573 / 2.552 | 1.509 / 1.819 |
| Synthetic wide repeated TFIDF, median / p95 | 1.577 / 1.804 | 1.527 / 1.744 |
| Synthetic wide whole-child peak RSS, median MiB | 8.02 | 8.02 |

Canonical repeated TF-IDF medians are slightly higher in both campaigns. Its second-campaign p95 rises from 12.805 to 15.249 ms, despite the lower initial-pass median. The first synthetic-wide initial p95 is also higher. These slower tails remain part of the result.

Interpret the first TF-IDF call separately from the aggregate initial pass: later initial requests can already reuse norms populated by earlier requests. Repeated requests are a third scope. Initial snapshot follow-up reads are outside search timing. Search timing includes progress and stdio response transport; whole-child peak RSS includes parsing, buffers, index, hints and runtime. Startup is synchronous loading/indexing outside request deadlines. Small or slower cases remain visible; these measurements establish neither a hard process quota nor a general transport speed guarantee.

## Complete document responses and construction controls

The retained private document corpus contains 72 Markdown files, 2,755,468 source bytes and 1,910 chunks. All 342 authored questions (276 positive, 66 unsupported) are exposed regressions, not new holdout questions or logged production traffic. Literal BM25, prepared BM25 and coverage retain six passages, 100 candidates, 2,000 bytes per passage and 12,000 total text bytes.

All **1,026 complete original document response JSON strings match byte-for-byte**, including scores, IDs, order, terms, source text/positions and recipes. Every timed repeat also matches its initial response. Prior misses remain. These three configurations use BM25; TF-IDF is checked separately by the score oracle and actual protocol comparisons. One warm-up and five samples per query/configuration/version give **10,260 timed round trips**, including source/evidence work, serialization, IPC and Python parsing. No new semantic judgment or model reply is added.

| Document context, ms | 0.4.12 median / p95 | 0.4.13 median / p95 |
| --- | ---: | ---: |
| baseline | 0.966 / 1.488 | 0.931 / 1.417 |
| prepared | 1.045 / 1.624 | 1.026 / 1.558 |
| evidence | 1.061 / 1.649 | 1.025 / 1.570 |

Construction is a control. Five alternating pairs for canonical knowledge and three synthetic distributions give 40 plain processes. Parsing/input generation is outside the construction timer; six separate profiles measure synthetic derived live heap. Stats/signatures and derived live bytes match.

| Construction only, ms | 0.4.12 median / p95 | 0.4.13 median / p95 |
| --- | ---: | ---: |
| canonical | 4913.770 / 5284.356 | 4882.508 / 4958.616 |
| ascii | 14.875 / 15.052 | 14.442 / 15.534 |
| mixed | 34.216 / 34.572 | 33.765 / 34.719 |
| long_tokens | 1.608 / 1.658 | 1.564 / 1.665 |

A separate execution/shared-index worker validates all 167,942 entries through 128 changes/restoration, rebuilds, JSON round trips, four readers with forty queries each, retained clones and cancellation/deadline rejection. Source and portable exports agree. This is finite consistency validation, not a new PDF parser or million-user capacity test.

## Conditions, limits and preservation

Measurements use one AC-powered Apple M5 Max with 64 GB, Rust 1.98.0 and Python 3.14.7. Stage-start host CPU/compiler snapshots are retained; they do not establish continuously idle hardware or attribute every difference. One corpus, exposed authored questions, randomized hashing and small process counts limit generalization. Read [all sanitized records](measurements/0.4.13/README.md). Raw sources, transcripts, profiles, locks, exact programs and logs remain private. Public aggregates cannot reconstruct the private corpus; standalone synthetic programs and generator are published.

Exact quotations and lexical scores do not establish truth or entailment. There is no new LLM, semantic-quality, token-saving or fresh PDF extraction claim. No daemon is started.

Canonical knowledge remains **167,942 entries from 35 PDFs and 49,086 physical pages**, SHA-256 `13168d87872a137355fd8499c90a6084f52243a2ec7456f29b5fcb2a1c7423c3`. Preserve PDFs, knowledge, snapshots and every measurement. After verified publication, archive packages/proof/locks/small measured programs before deleting only the named temporary Cargo/Miri targets. Defaults remain empty and no DenkMaps integration is performed.
