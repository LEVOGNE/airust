# AIRust 0.4.15: small MCP calls without the initial timer wait

The optional MCP handler no longer awaits the progress timer before selecting worker completion, cancellation and the original absolute deadline. It schedules the first periodic heartbeat for one configured interval in the future and immediately enters the existing biased selection loop. This removes an avoidable scheduling delay from small calls, including calls without a progress token. Core algorithms, exact scores, evidence, source positions, Rust APIs, tools, arguments, schemas and saved knowledge remain. No dependency, feature flag, startup prewarming or DenkMaps integration is added.

Two preplanned final campaigns show one-entry repeated TF-IDF MCP median changes of **-90.36% and -90.13%** against freshly built exact registry 0.4.14. Negative means shorter observed times. Canonical repeated TF-IDF changes are **+21.22% and +1.74%**. All rankers, initial calls, tails, startup, RAM, negative outcomes and controls remain below and in the [sanitized data](measurements/0.4.15/README.md). These observations are not a universal improvement or significance claim.

## Execution and resource contract

`progress::heartbeat` constructs an interval starting at now plus the existing period, with missed ticks skipped. Previously an immediate interval tick was explicitly awaited after worker launch, even without requested progress. A timer wakeup could therefore delay a finished worker before the result loop was entered. No awaited tick or sleep now occurs between admitted worker launch and the selector.

The start notification still precedes worker launch when requested. Completion/error notification still precedes the response; periodic updates start after the configured interval. Fast calls may have only start and completion. Notifications describe work, not remaining-time predictions. Existing cancellation and original absolute-deadline branches retain precedence over a ready worker. Notification delivery retains cancellation, connection-stop, deadline and bounded stalled-send checks. Worker leases, disconnect drain, admission, frame/rate/result budgets and read-only/source contracts remain.

Work is still executed on a blocking worker. Startup loading/indexing remains synchronous and outside a tool deadline. No hard real-time or total-memory guarantee is added. Read [MCP](MCP.md), [security scope](MCP-SECURITY.md), [API](API.md) and [cookbook](API-COOKBOOK.md).

## Executed validation

The complete matrix passes **234 default tests, 884 all-feature checks including 39 doctests, 39 isolated feature entries and five Python tests**. Strict Clippy/docs, Rust 1.89 provider-neutral and Rust 1.92 all-feature checks pass. API comparison with tagged 0.4.14 reports **229 pass, 31 skip**. An independent Rust 1.89 consumer exercises retrieval, coverage, quotes, document positions and MCP/schema construction. The unchanged PDF backend passes 14 regressions and its 381-source-file provenance check. A refreshed audit reports zero known vulnerabilities in the checked database; existing maintenance notices are retained in the resource record.

A new native regression polls the first heartbeat and ready-work selection directly without a wall-clock performance assertion. A new protocol regression exercises BM25, TF-IDF, exact matching, relations and source reads with/without progress, checks identical complete payloads and checks start/completion sequences while the periodic heartbeat is scheduled much later. Existing hostile-input, cancellation/suppressed-response, deadline, stalled-reader, busy/duplicate-token, source-binding, schema, disconnect and worker-drain checks pass.

Sixteen finite Tree Borrows Miri cases cover retained Core/quote/span invariants. Only extra floating-point rounding injection is disabled; memory/alias checks remain. New Tokio/protocol execution is checked natively, outside the finite interpreter scope. These are not independent security or soundness audits. Extracted-package tests, exact-commit CI, fresh registry consumer and checksum/VCS/tag matching gate publication.

## Inputs, method and pilot

The [programs/method](validation-0.4.15/README.md) compare exact registry 0.4.14 (VCS c0a718604ad36026a33c5a15a67ad7f49c5f41da) and candidate 0.4.15 with identical programs and inputs. Both final sides are freshly built before timing. All programs, locks, measured executable/source hashes and input hashes are preserved.

The separate [development pilot](measurements/0.4.15/development-pilot-summary.json) used the retained measured 0.4.14 executable and a fresh candidate. Tiny repeated TF-IDF medians fell from 1.253 to 0.123 ms, but canonical medians rose from 6.529 to 9.498 ms. All source snapshots, logs, binaries and slower observations remain. Pilot samples are not merged with or substituted for final results.

Final measurements retain canonical knowledge, a revision-bound NumPy snapshot and deterministic wide corpus. Small MCP inputs now cover 1/4/16 entries, each with and without requested progress. Both versions receive every addition. Each of two preplanned complete campaigns uses five alternating old/new pairs for nine input/progress cases: **90 servers per campaign, 180 total**. Historical absolute tables with other programs are not directly interchangeable.

## Actual MCP: two complete campaigns

Each server makes ten queries with both rankers, then two repeated passes, four concurrent searches, matching/relations, rejected arguments and health. Snapshot inputs also check exact UTF-8 spans and passage readbacks. Across both campaigns **13,880 actual result/error replies validate against advertised schemas**, and **6,940 complete old/new structured pairs match by request ID**, excluding only status version and asserted version-qualified schema URNs. Counts: 3,600 initial searches, 7,200 repeats, 720 concurrent searches, 1,640 readbacks, 360 matching/relations, 180 rejected arguments and 180 health calls. Discovery, progress/no-progress, source bounds, concurrent ordering, input hashes and EOF shutdown pass.

`tiny`, `tiny4`, `tiny16` request progress; `-no-progress` cases do not. Initial passes include cold/partly warm hints and differ from the first TF-IDF call and repeated passes. Search timing includes serialization, process transport, Python parsing and requested notifications; passage follow-ups are outside it. Startup includes input loading/indexing. Whole-child RSS includes buffers, index and runtime and differs from allocation traffic.

### First complete campaign

| Case | Ranker | Initial median / p95 ms: old → new | Repeated median / p95 ms: old → new | Repeated median change |
| --- | --- | --- | --- | --- |
| canonical | bm25 | 6.321 / 11.396 → 6.584 / 11.879 | 5.354 / 9.999 → 5.862 / 11.352 | +9.50% |
| canonical | tfidf | 18.293 / 31.670 → 19.934 / 34.708 | 5.240 / 10.054 → 6.352 / 12.867 | +21.22% |
| numpy | bm25 | 1.490 / 1.541 → 0.876 / 1.193 | 1.471 / 1.592 → 0.745 / 1.005 | -49.32% |
| numpy | tfidf | 1.469 / 3.433 → 0.969 / 2.721 | 1.493 / 1.678 → 0.715 / 1.054 | -52.10% |
| wide | bm25 | 1.452 / 1.579 → 0.504 / 0.627 | 1.473 / 1.584 → 0.420 / 0.664 | -71.51% |
| wide | tfidf | 1.469 / 1.527 → 0.458 / 0.819 | 1.477 / 1.576 → 0.420 / 0.777 | -71.61% |
| tiny | bm25 | 1.325 / 1.371 → 0.143 / 0.230 | 1.318 / 1.359 → 0.127 / 0.158 | -90.37% |
| tiny | tfidf | 1.322 / 1.365 → 0.140 / 0.187 | 1.318 / 1.349 → 0.127 / 0.147 | -90.36% |
| tiny4 | bm25 | 1.364 / 1.453 → 0.199 / 0.316 | 1.361 / 1.395 → 0.180 / 0.199 | -86.76% |
| tiny4 | tfidf | 1.351 / 1.402 → 0.195 / 0.250 | 1.362 / 1.401 → 0.178 / 0.222 | -86.90% |
| tiny16 | bm25 | 1.357 / 1.433 → 0.222 / 0.339 | 1.378 / 1.434 → 0.203 / 0.233 | -85.30% |
| tiny16 | tfidf | 1.362 / 1.444 → 0.218 / 0.311 | 1.377 / 1.450 → 0.204 / 0.235 | -85.17% |
| tiny-no-progress | bm25 | 1.260 / 1.333 → 0.086 / 0.178 | 1.264 / 1.313 → 0.076 / 0.113 | -93.96% |
| tiny-no-progress | tfidf | 1.252 / 1.304 → 0.087 / 0.124 | 1.261 / 1.294 → 0.075 / 0.091 | -94.03% |
| tiny4-no-progress | bm25 | 1.302 / 1.396 → 0.146 / 0.255 | 1.303 / 1.333 → 0.131 / 0.162 | -89.95% |
| tiny4-no-progress | tfidf | 1.294 / 1.342 → 0.143 / 0.187 | 1.302 / 1.347 → 0.131 / 0.173 | -89.95% |
| tiny16-no-progress | bm25 | 1.313 / 1.375 → 0.166 / 0.275 | 1.320 / 1.357 → 0.150 / 0.181 | -88.64% |
| tiny16-no-progress | tfidf | 1.318 / 1.402 → 0.166 / 0.274 | 1.322 / 1.351 → 0.152 / 0.187 | -88.53% |

| Case | Startup median ms: old → new | First TF-IDF median ms: old → new | Peak RSS median MiB: old → new |
| --- | --- | --- | --- |
| canonical | 4627.488 → 4753.569 | 20.595 → 20.939 | 1724.250 → 1724.219 |
| numpy | 126.774 → 126.877 | 1.460 → 1.426 | 57.203 → 56.953 |
| wide | 3.691 → 3.810 | 1.501 → 0.494 | 7.938 → 7.797 |
| tiny | 3.234 → 3.050 | 1.363 → 0.191 | 5.656 → 5.734 |
| tiny4 | 3.201 → 3.179 | 1.401 → 0.251 | 5.828 → 5.844 |
| tiny16 | 3.252 → 3.311 | 1.415 → 0.293 | 6.016 → 5.953 |
| tiny-no-progress | 3.278 → 3.274 | 1.302 → 0.124 | 5.703 → 5.688 |
| tiny4-no-progress | 3.304 → 3.190 | 1.325 → 0.187 | 5.812 → 5.766 |
| tiny16-no-progress | 3.284 → 3.238 | 1.350 → 0.232 | 6.000 → 5.906 |

### Second preplanned complete campaign

| Case | Ranker | Initial median / p95 ms: old → new | Repeated median / p95 ms: old → new | Repeated median change |
| --- | --- | --- | --- | --- |
| canonical | bm25 | 6.176 / 10.880 → 6.296 / 11.060 | 5.187 / 10.310 → 5.427 / 10.694 | +4.61% |
| canonical | tfidf | 17.683 / 30.567 → 17.200 / 31.322 | 5.131 / 9.940 → 5.220 / 10.770 | +1.74% |
| numpy | bm25 | 1.495 / 1.634 → 0.934 / 1.387 | 1.490 / 1.680 → 0.801 / 1.268 | -46.25% |
| numpy | tfidf | 1.522 / 3.416 → 0.999 / 2.926 | 1.512 / 1.936 → 0.743 / 1.221 | -50.87% |
| wide | bm25 | 1.454 / 1.568 → 0.516 / 0.637 | 1.466 / 1.589 → 0.418 / 0.662 | -71.46% |
| wide | tfidf | 1.470 / 1.549 → 0.478 / 0.805 | 1.479 / 1.564 → 0.413 / 0.770 | -72.06% |
| tiny | bm25 | 1.320 / 1.382 → 0.146 / 0.236 | 1.311 / 1.358 → 0.129 / 0.163 | -90.14% |
| tiny | tfidf | 1.318 / 1.354 → 0.142 / 0.182 | 1.310 / 1.348 → 0.129 / 0.147 | -90.13% |
| tiny4 | bm25 | 1.341 / 1.470 → 0.208 / 0.329 | 1.350 / 1.392 → 0.183 / 0.204 | -86.45% |
| tiny4 | tfidf | 1.342 / 1.405 → 0.210 / 0.244 | 1.355 / 1.395 → 0.185 / 0.228 | -86.35% |
| tiny16 | bm25 | 1.358 / 1.457 → 0.227 / 0.343 | 1.358 / 1.416 → 0.206 / 0.232 | -84.80% |
| tiny16 | tfidf | 1.371 / 1.464 → 0.226 / 0.312 | 1.363 / 1.412 → 0.206 / 0.241 | -84.86% |
| tiny-no-progress | bm25 | 1.273 / 1.331 → 0.093 / 0.190 | 1.262 / 1.300 → 0.078 / 0.121 | -93.85% |
| tiny-no-progress | tfidf | 1.270 / 1.301 → 0.088 / 0.145 | 1.264 / 1.298 → 0.076 / 0.089 | -93.95% |
| tiny4-no-progress | bm25 | 1.301 / 1.405 → 0.144 / 0.258 | 1.299 / 1.335 → 0.131 / 0.159 | -89.91% |
| tiny4-no-progress | tfidf | 1.308 / 1.348 → 0.150 / 0.206 | 1.298 / 1.341 → 0.130 / 0.165 | -90.01% |
| tiny16-no-progress | bm25 | 1.313 / 1.389 → 0.164 / 0.280 | 1.315 / 1.349 → 0.156 / 0.185 | -88.12% |
| tiny16-no-progress | tfidf | 1.321 / 1.423 → 0.174 / 0.274 | 1.320 / 1.363 → 0.159 / 0.192 | -87.98% |

| Case | Startup median ms: old → new | First TF-IDF median ms: old → new | Peak RSS median MiB: old → new |
| --- | --- | --- | --- |
| canonical | 4395.713 → 4407.396 | 19.580 → 19.874 | 1724.125 → 1724.094 |
| numpy | 133.127 → 137.657 | 1.564 → 1.592 | 57.203 → 57.031 |
| wide | 3.859 → 3.911 | 1.495 → 0.488 | 7.938 → 7.844 |
| tiny | 3.257 → 3.263 | 1.352 → 0.187 | 5.703 → 5.688 |
| tiny4 | 3.223 → 3.501 | 1.405 → 0.247 | 5.828 → 5.797 |
| tiny16 | 3.343 → 3.463 | 1.421 → 0.289 | 6.031 → 5.969 |
| tiny-no-progress | 3.268 → 3.311 | 1.294 → 0.142 | 5.688 → 5.688 |
| tiny4-no-progress | 3.435 → 3.425 | 1.339 → 0.208 | 5.828 → 5.812 |
| tiny16-no-progress | 3.436 → 3.252 | 1.359 → 0.230 | 6.000 → 5.969 |

## Retained Core controls

There is no Core algorithm change. Fresh controls detect drift and retain variation rather than crediting unrelated changes to the timer patch. Seven alternating pairs give fourteen plain processes per family. Norms are initialized for warm timing; two separate DHAT processes measure allocations. Instrumented durations are excluded from latency comparisons. Every result signature agrees and temporary-call retained deltas are zero.

### Small warm Core: 84,000 calls on 1/4/16 entries

| Fixture | Median change | Blocks/call: old → new | Allocated bytes/call: old → new |
| --- | --- | --- | --- |
| tiny16_bm25 | +0.56% | 32.0 → 32.0 | 2712.0 → 2712.0 |
| tiny16_tfidf | -1.74% | 32.0 → 32.0 | 2712.0 → 2712.0 |
| tiny4_bm25 | -2.08% | 16.0 → 16.0 | 1284.0 → 1284.0 |
| tiny4_tfidf | -1.20% | 16.0 → 16.0 | 1284.0 → 1284.0 |
| tiny_bm25 | -9.94% | 10.0 → 10.0 | 858.0 → 858.0 |
| tiny_tfidf | -4.44% | 10.0 → 10.0 | 858.0 → 858.0 |

### All 25 warm controls: 282,800 calls

| Fixture | Median change | Blocks/call: old → new | Allocated bytes/call: old → new |
| --- | --- | --- | --- |
| coverage_repeated | +0.02% | 1047.0 → 1047.0 | 92942.0 → 92942.0 |
| coverage_small | -1.77% | 784.0 → 784.0 | 78046.0 → 78046.0 |
| excerpt_ascii | -1.63% | 762.0 → 762.0 | 252438.0 → 252438.0 |
| excerpt_mixed | +1.30% | 569.0 → 569.0 | 417113.0 → 417113.0 |
| excerpt_unicode | +0.31% | 197.0 → 197.0 | 72254.0 → 72254.0 |
| long_repeated | -2.84% | 4.0 → 4.0 | 18798.0 → 18798.0 |
| mixed_unicode | +0.90% | 7.0 → 7.0 | 3155.0 → 3155.0 |
| punctuation | -3.73% | 1.0 → 1.0 | 768.0 → 768.0 |
| relevant_retrieval | +2.70% | 812.0 → 812.0 | 93562.0 → 93562.0 |
| repeated_search | -3.72% | 39.0 → 39.0 | 19736.0 → 19736.0 |
| repeated_short | +0.07% | 4.0 → 4.0 | 1656.0 → 1656.0 |
| sparse_bm25_128 | -2.23% | 359.0 → 359.0 | 29074.0 → 29074.0 |
| sparse_bm25_33 | -2.83% | 241.0 → 241.0 | 16309.0 → 16309.0 |
| sparse_bm25_512 | -2.69% | 1337.0 → 1337.0 | 119124.0 → 119124.0 |
| sparse_tfidf_128 | -0.27% | 474.0 → 474.0 | 35140.0 → 35140.0 |
| sparse_tfidf_33 | +3.64% | 241.0 → 241.0 | 16309.0 → 16309.0 |
| sparse_tfidf_512 | -0.44% | 1337.0 → 1337.0 | 119124.0 → 119124.0 |
| tfidf_union_32 | -2.32% | 244.0 → 244.0 | 19834.0 → 19834.0 |
| tfidf_union_33 | -2.15% | 296.0 → 296.0 | 27854.0 → 27854.0 |
| tfidf_warm | +4.47% | 71.0 → 71.0 | 19335.0 → 19335.0 |
| union_32 | +0.06% | 244.0 → 244.0 | 19834.0 → 19834.0 |
| union_33 | +0.44% | 296.0 → 296.0 | 27854.0 → 27854.0 |
| unique_long | +0.04% | 299.0 → 299.0 | 32672.0 → 32672.0 |
| unique_search | +0.22% | 72.0 → 72.0 | 19350.0 → 19350.0 |
| unique_short | -3.20% | 296.0 → 296.0 | 18548.0 → 18548.0 |

### Cold norm and heap controls

Six fixtures make twenty fresh-index-clone calls each across fourteen processes: 1,680 calls. Clone/setup is outside timing. Process/instruction/filesystem caches remain; cold means norm hints, not disks. Separate heap runs drop results before net retained measurements. The 0.4.14 inline/optional heap memo and persistent norm storage remain. Heap profiles exclude stack state/control fields and temporary frames.

| Fixture | Old median / p95 ns | New median / p95 ns |
| --- | --- | --- |
| index | 117729.500 / 133353.850 | 116916.500 / 129921.150 |
| wide | 36708.500 / 46637.500 | 35583.500 / 43093.400 |
| sparse | 489458.500 / 563127.100 | 492833.000 / 600963.150 |
| tiny | 604.500 / 2252.100 | 583.000 / 1541.000 |
| tiny4 | 792.000 / 1416.050 | 833.000 / 1333.000 |
| tiny16 | 2584.000 / 3791.050 | 2583.000 / 3671.150 |

| Cold fixture | Allocated bytes: old → new | Retained hint bytes: old → new |
| --- | --- | --- |
| index | 92660 → 92660 | 34904 → 34904 |
| wide | 19549 → 19549 | 4440 → 4440 |
| sparse | 140704 → 140704 | 8792 → 8792 |
| tiny | 1014 → 1014 | 156 → 156 |
| tiny4 | 1584 → 1584 | 224 → 224 |
| tiny16 | 3844 → 3844 | 632 → 632 |

Canonical first-pass heap records, all samples, blocks and cold tails remain in public JSON. Retained hints, heap traffic and whole-process RAM are different measures.

## Finite mixed control, documents and construction

The 6,000-entry mixed control retains four continuous warm readers while 100 initially uncached-group driver queries finish, then makes two quiescent probes. Five alternating pairs give 1,000 driver calls and twenty quiescent probes. Warm counts vary with the stop condition and new-group calls can become warm. This finite control is not fixed-load throughput, capacity or fairness evidence.

| Mixed work, ns | Old median / p95 | New median / p95 |
| --- | --- | --- |
| new_group_query_ns | 119812.500 / 197252.100 | 129687.500 / 170468.100 |
| post_readers_first_ns | 157125.000 / 174008.400 | 184792.000 / 256092.000 |
| post_readers_second_ns | 88208.000 / 98450.400 | 87833.000 / 117400.000 |

The private document corpus retains 72 Markdown files, 2,755,468 bytes and 1,910 chunks. All 342 authored questions (276 positive, 66 unsupported) are exposed regressions, not fresh holdouts or logged production queries. Literal BM25, prepared BM25 and coverage retain six passages, 100 candidates, 2,000 bytes/pass and 12,000 total bytes. All **1,026 complete original response JSON strings match byte-for-byte**, including scores, ordering, evidence, source text and positions. One warm-up and five repeats/question/configuration/version give **10,260 timed round trips**. These three configurations are BM25 controls; independent score oracles and actual protocol pairs cover TF-IDF. No new relevance judgment or model reply is produced.

All document timing aggregates and per-question results remain in the records. Construction uses forty plain processes on canonical data and three synthetic distributions, plus six separate profiles. Parsing/generation is outside construction timing; stats/signatures/live derived heap agree. The execution/shared-index worker checks all 167,942 entries through 128 changes/restoration, rebuilds, JSON round trips, retained clones, parallel readers and cancellation/deadline rejection. Portable exports and source hashes agree.

### Document round trips

| Case | Old median / p95 ms | New median / p95 ms |
| --- | --- | --- |
| baseline | 0.733 / 1.101 | 0.729 / 1.109 |
| prepared | 0.788 / 1.224 | 0.783 / 1.208 |
| evidence | 0.785 / 1.197 | 0.779 / 1.211 |

### Construction only

| Case | Old median / p95 ms | New median / p95 ms |
| --- | --- | --- |
| canonical | 4857.195 / 5918.075 | 4906.702 / 5662.630 |
| ascii | 15.606 / 16.995 | 14.186 / 14.751 |
| mixed | 35.077 / 39.894 | 33.861 / 34.445 |
| long_tokens | 1.659 / 1.748 | 1.520 / 1.566 |

## Large-query follow-up and observed load

The ordinary final campaigns do **not** establish preserved canonical query latency: repeated TF-IDF medians are 21.22% and 1.74% higher, and first-campaign tails also rise. This remains a limitation of this release. The following per-query medians keep the heterogeneity visible without publishing private query text. These are the same final samples, not new trials.

| Campaign | Query number | Old warm median ms | New warm median ms | Change |
| --- | --- | --- | --- | --- |
| first | 0 | 4.552 | 6.123 | +34.51% |
| first | 1 | 5.387 | 6.966 | +29.32% |
| first | 2 | 9.473 | 12.533 | +32.29% |
| first | 3 | 1.446 | 0.710 | -50.92% |
| first | 4 | 5.295 | 6.993 | +32.07% |
| first | 5 | 8.611 | 11.276 | +30.95% |
| first | 6 | 1.660 | 1.984 | +19.56% |
| first | 7 | 9.138 | 12.047 | +31.85% |
| first | 8 | 4.921 | 5.465 | +11.06% |
| first | 9 | 1.310 | 0.125 | -90.47% |
| second | 0 | 4.530 | 4.628 | +2.16% |
| second | 1 | 5.404 | 5.668 | +4.89% |
| second | 2 | 9.677 | 9.945 | +2.76% |
| second | 3 | 1.434 | 0.648 | -54.78% |
| second | 4 | 5.708 | 5.349 | -6.29% |
| second | 5 | 8.739 | 9.348 | +6.96% |
| second | 6 | 1.672 | 1.674 | +0.10% |
| second | 7 | 9.339 | 9.431 | +0.98% |
| second | 8 | 4.484 | 4.816 | +7.40% |
| second | 9 | 1.318 | 0.116 | -91.19% |

After both fixed campaigns, a **separate instrumented diagnostic** uses the same private source/build path, version 0.4.15, profile and dependency lock for two variants. One reinstates the initial wait; the other uses selected direct waiting. Both add the same worker wall-clock/stderr instrumentation. Each side is built before measurement. Five alternating pairs give ten further canonical servers; their complete structured payloads agree. These are additional diagnostic samples, not replacements or additions to the 180 ordinary servers or 13,880 schema-validation total.

Worker time measures the `execute` body after the blocking task begins, including scheduling interruptions. Remaining round-trip time includes IPC, dispatch/worker queue, notifications and result processing. Medians of components do not necessarily add to the median total. Instrumentation changes scheduling/cost and this same-version control differs from the actual old/new release comparison.

| Ranker | Variant | Warm samples | Round trip median ms | Worker median ms | Remaining round trip median ms |
| --- | --- | --- | --- | --- | --- |
| bm25 | wait | 100 | 6.805 | 6.269 | 0.459 |
| bm25 | direct | 100 | 6.847 | 6.528 | 0.336 |
| tfidf | wait | 100 | 7.099 | 6.644 | 0.482 |
| tfidf | direct | 100 | 7.037 | 6.534 | 0.345 |

The diagnostic does not reproduce a uniform 21% timer-induced slowdown; both variants are close for this large workload. It also does not prove absence of a large-query regression or establish why the ordinary campaigns differ. Their original slower results remain authoritative observations of those runs.

Stage-start snapshots record one other `rustc` process at the construction stage and one process above 50% CPU at the first MCP stage. The separate diagnostic also records other compiler activity in several starts. All compiler work managed for each measurement had completed; other workloads were neither stopped nor controlled. These snapshots cannot attribute a timing difference to a particular cause or prove continuously idle hardware. Read [load records](measurements/0.4.15/host-activity.json), [diagnostic summary](measurements/0.4.15/diagnostic-summary.json) and [all diagnostic samples](measurements/0.4.15/diagnostic-samples.json).

Further large-query optimization needs isolated per-query measurements under controlled load before making a claim that large-query speed is preserved. The demonstrated release benefit is small-call latency; no blanket speedup, sustained throughput or capacity claim is made.

## Limits and preservation

Measurements use one AC-powered Apple M5 Max with 64 GB, Rust 1.98.0 and Python 3.14.7. Stage-start CPU/compiler snapshots remain. Own compilers finish before timing; this does not establish continuously idle hardware. Randomized hashing, scheduling, one machine/corpus, exposed questions and finite process counts limit generalization. The pilot's large-query regression remains; assess final per-case medians and tails independently. No universal latency, total-memory, semantic-quality, token-saving or security-audit claim is made.

Canonical knowledge remains 167,942 entries from 35 PDFs and 49,086 physical pages, SHA-256 `13168d87872a137355fd8499c90a6084f52243a2ec7456f29b5fcb2a1c7423c3`. The PDF parser is unchanged; this is not a fresh large-PDF extraction campaign. Lexical overlap and exact quotes do not prove factual support or entailment. No model, daemon or DenkMaps integration is added.

Preserve PDFs, knowledge, snapshots, all pilot/final measurements, slower results, raw transcripts, profiles, exact small executables and locks. After verified publication archive package/proof, then delete only the named temporary Cargo/Miri targets. Default features remain empty.
