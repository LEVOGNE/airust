# AIRust 0.4.16: large queries through bounded posting-frequency cursors

This release targets large MCP search regressions through the shared lexical Core. After enough matching candidates, small known-term queries can reuse frequencies from sorted postings rather than repeatedly searching each document's word dictionary. The small-call timer improvement from 0.4.15 remains. Public APIs, exact ranking, evidence, source positions, schemas and saved knowledge remain. No dependency, feature flag, model, startup prewarming or DenkMaps integration is added.

Across two preplanned ordinary campaigns, repeated canonical MCP TF-IDF median changes are **-45.84% and -45.57%** against freshly built exact registry 0.4.15. One-entry repeated medians change by **+1.17% and -15.38%**. Negative means shorter observed time. All slower medians, tails, initial calls, startup, allocations and RAM remain below and in [sanitized records](measurements/0.4.16/README.md). A separate direct controlled-search matrix and fresh three-version 0.4.14/0.4.15/0.4.16 control follow these tables. No result establishes universal improvement or statistical significance.

## Execution and resource contract

The existing candidate union remains sorted, and document lookup, ranking arithmetic and contribution order remain. When there are at least 1,024 candidates and 1–4 known query terms, the query waits until 128 candidates have passed its context/custom-scope filters. At the next admitted candidate it creates one private frequency cursor per known term. Narrow scopes that never reach that point, smaller candidate sets and wider queries retain the prior lookup strategy. Eligibility/accounting code can still affect their costs; all small controls remain reported.

Each cursor starts at a lower bound in the term's existing sorted postings and reads the same term frequency for that document as the document word map. Matching uses a live Arc ID identity shortcut, with exact string comparison when identities differ. Nothing is persisted or accepted as a pointer from a client. Skipping 32 posting entries triggers an additional checkpoint and lower-bound seek, limiting linear work across filtered gaps. All original posting, candidate, ranking and norm checkpoints remain; no cancellation/deadline check is removed or sampled away. Partial work is not returned on error.

Term callbacks still run in the original normalized query-term order, preserving floating-point additions, exact TF-IDF/BM25 scores, deterministic ties and explanations. The cursor state belongs to one immutable query and is dropped on completion or failure. A bounded temporary Vec adds allocation traffic on eligible large queries. It adds no index field, per-document cache or retained corpus memory. Heap traffic, stack state, retained hints and process RSS are separate measures; this is not a hard memory quota.

MCP still launches admitted work on blocking workers and immediately selects completion, cancellation or the original absolute deadline. The first progress timer wait removed in 0.4.15 stays removed. Start/periodic/completion notifications, delivery deadlines, worker leases, admission, disconnect drain and frame/rate/result budgets remain. Faster requests do not raise configured request/message allowances. Startup loading/indexing remains outside tool deadlines. Read [MCP](MCP.md), [security scope](MCP-SECURITY.md), [API](API.md), [cookbook](API-COOKBOOK.md) and [Core performance](CORE-PERFORMANCE.md).

## Executed validation

The complete matrix passes **236 default tests, 886 all-feature checks including 39 doctests, 39 isolated feature entries and five Python tests**. Strict Clippy/docs, Rust 1.89 provider-neutral and Rust 1.92 all-feature checks pass. API comparison with tagged 0.4.15 reports **229 pass, 31 skip**. An independent Rust 1.89 consumer exercises retrieval, coverage, quotes, document positions and MCP/schema construction. The unchanged PDF backend passes 14 regressions and its 381-source-file provenance check. A refreshed audit reports zero known vulnerabilities in the checked database; existing maintenance notices remain in the resource record.

New cursor regressions verify exact frequencies, a separately allocated equal ID, gaps, missing IDs and cancellation before seeking. The independent full-scan score oracle grows from 49 to 1,025 native records including the vocabulary control and checks small/wide queries, context filters, top-100/full results, both rankers and cold/warm states. Its old small-data assumption about result limits is corrected. Existing mutation, retained-clone, reused-ID, metadata, source/span, hostile MCP input, cancellation/suppressed-response, absolute-deadline, stalled-reader, busy/duplicate-token, disconnect and worker-drain tests pass.

Eighteen finite Tree Borrows Miri cases include the two new cursor regressions; scaled cases exercise cursor identity, seeking and cancellation, while the native oracle covers strategy thresholds and larger data. Only extra floating-point rounding injection is disabled; memory/alias checks remain. Tokio/PDF/FFI/ONNX execution is outside the finite interpreter scope. These checks are not independent security or soundness audits. Extracted-package tests, exact-commit CI, fresh registry consumer and checksum/VCS/tag matching gate publication.

## Inputs, sampling and development

The [programs/method](validation-0.4.16/README.md) compare exact registry 0.4.15 (VCS 6ac8f081b46bc63cbc9c999e0b88d944524fc62e) and candidate 0.4.16 with identical programs and inputs. Both final sides are freshly built before timing. A separate reference uses exact 0.4.14 (VCS c0a718604ad36026a33c5a15a67ad7f49c5f41da). All programs, locks, actual manifest/source copies, measured executable hashes and input hashes are preserved.

A separate macOS sample of 0.4.15 controlled retrieval showed many string comparisons and clock checks inside scoring. It suggested dictionary lookup costs worth testing; it did not independently establish why the earlier MCP campaigns differed. A first ordered-document sweep was rejected because direct controlled medians became slower even though some MCP observations improved. Its complete code, samples and negative outcomes remain. The selected posting-frequency design passed exact full-response checks in the next pilot. No pilot or sampled duration is merged with final timing.

Selected-pilot repeated canonical TF-IDF medians changed from 5.666 to 2.957 ms, and tiny repeated medians from 0.128 to 0.127 ms. Initial tiny calls and other pilot variations are retained too. These are development observations, not replacements for final campaigns or new holdouts.

Final measurements retain canonical knowledge, a revision-bound NumPy snapshot and deterministic wide corpus. Small MCP inputs cover 1/4/16 entries with and without requested progress. Each of two fixed ordinary campaigns uses five alternating old/new pairs for nine cases: **90 servers per campaign, 180 total**. Additional controlled-search and three-version trials are reported separately. Every managed compiler job finishes before timing. Coarse stage snapshots and two-second host sampling expose other load without stopping it or proving continuous idleness. Historical tables with other programs/loads are not directly interchangeable. The exposed authored questions and tuned workload are not fresh holdouts.

## Actual MCP: two complete ordinary campaigns

Each server makes ten queries with both rankers, two repeated passes, four concurrent searches, matching/relations, rejected arguments and health. Snapshot inputs also check exact UTF-8 spans and passage readbacks. Across both ordinary campaigns **13,880 actual result/error replies validate against advertised schemas**, and **6,940 complete old/new structured pairs match by request ID**, excluding only status version and asserted version-qualified schema URNs. Counts: 3,600 initial searches, 7,200 repeats, 720 concurrent searches, 1,640 readbacks, 360 matching/relations, 180 rejected arguments and 180 health calls. Discovery, progress/no-progress, source bounds, concurrent ordering, input hashes and EOF shutdown pass.

`tiny`, `tiny4`, `tiny16` request progress; `-no-progress` cases do not. Initial passes include cold/partly warm hints and differ from the first TF-IDF call and repeats. Search timing includes serialization, stdio, Python parsing and requested notifications; passage follow-ups are outside it. Startup includes loading/indexing. Whole-child RSS includes buffers/index/runtime and differs from allocation traffic. Read every slower case from its own row.

### First complete campaign

| Case | Ranker | Initial median / p95 ms: old → new | Repeated median / p95 ms: old → new | Repeated median change |
| --- | --- | --- | --- | --- |
| canonical | bm25 | 6.578 / 11.885 → 3.191 / 5.343 | 5.885 / 11.689 → 2.772 / 5.148 | -52.89% |
| canonical | tfidf | 19.387 / 35.943 → 18.177 / 32.279 | 5.741 / 13.226 → 3.110 / 6.472 | -45.84% |
| numpy | bm25 | 0.886 / 1.226 → 0.868 / 1.127 | 0.752 / 1.029 → 0.750 / 1.044 | -0.21% |
| numpy | tfidf | 0.970 / 2.773 → 0.968 / 2.589 | 0.707 / 1.086 → 0.707 / 1.195 | +0.04% |
| wide | bm25 | 0.510 / 0.611 → 0.511 / 0.631 | 0.420 / 0.684 → 0.419 / 0.660 | -0.16% |
| wide | tfidf | 0.452 / 0.776 → 0.453 / 0.799 | 0.409 / 0.761 → 0.422 / 0.770 | +3.32% |
| tiny | bm25 | 0.142 / 0.230 → 0.141 / 0.241 | 0.126 / 0.164 → 0.128 / 0.168 | +1.60% |
| tiny | tfidf | 0.142 / 0.175 → 0.137 / 0.187 | 0.126 / 0.140 → 0.128 / 0.152 | +1.17% |
| tiny4 | bm25 | 0.204 / 0.307 → 0.196 / 0.321 | 0.180 / 0.201 → 0.179 / 0.202 | -0.58% |
| tiny4 | tfidf | 0.196 / 0.261 → 0.195 / 0.248 | 0.181 / 0.219 → 0.177 / 0.213 | -2.23% |
| tiny16 | bm25 | 0.214 / 0.335 → 0.220 / 0.346 | 0.198 / 0.226 → 0.201 / 0.224 | +1.15% |
| tiny16 | tfidf | 0.217 / 0.305 → 0.225 / 0.309 | 0.203 / 0.233 → 0.204 / 0.234 | +0.53% |
| tiny-no-progress | bm25 | 0.084 / 0.171 → 0.091 / 0.179 | 0.073 / 0.112 → 0.075 / 0.115 | +3.41% |
| tiny-no-progress | tfidf | 0.082 / 0.132 → 0.085 / 0.126 | 0.074 / 0.100 → 0.075 / 0.094 | +1.78% |
| tiny4-no-progress | bm25 | 0.145 / 0.252 → 0.142 / 0.258 | 0.129 / 0.163 → 0.134 / 0.154 | +3.47% |
| tiny4-no-progress | tfidf | 0.142 / 0.198 → 0.144 / 0.190 | 0.130 / 0.173 → 0.131 / 0.176 | +0.91% |
| tiny16-no-progress | bm25 | 0.167 / 0.281 → 0.157 / 0.272 | 0.150 / 0.177 → 0.149 / 0.186 | -0.89% |
| tiny16-no-progress | tfidf | 0.167 / 0.260 → 0.163 / 0.258 | 0.152 / 0.187 → 0.152 / 0.187 | -0.01% |

| Case | Startup median ms: old → new | First TF-IDF median ms: old → new | Peak RSS median MiB: old → new |
| --- | --- | --- | --- |
| canonical | 4831.187 → 4642.909 | 21.082 → 20.736 | 1724.062 → 1724.234 |
| numpy | 129.579 → 127.283 | 1.457 → 1.441 | 57.031 → 57.141 |
| wide | 3.744 → 3.721 | 0.479 → 0.461 | 7.766 → 7.734 |
| tiny | 3.155 → 3.253 | 0.176 → 0.188 | 5.750 → 5.781 |
| tiny4 | 3.242 → 3.232 | 0.261 → 0.252 | 5.844 → 5.906 |
| tiny16 | 3.195 → 3.188 | 0.285 → 0.283 | 5.953 → 6.125 |
| tiny-no-progress | 3.217 → 3.324 | 0.133 → 0.131 | 5.672 → 5.797 |
| tiny4-no-progress | 3.232 → 3.214 | 0.201 → 0.196 | 5.812 → 5.906 |
| tiny16-no-progress | 3.195 → 3.266 | 0.229 → 0.224 | 5.953 → 6.125 |

### Second preplanned complete campaign

| Case | Ranker | Initial median / p95 ms: old → new | Repeated median / p95 ms: old → new | Repeated median change |
| --- | --- | --- | --- | --- |
| canonical | bm25 | 6.387 / 11.326 → 3.118 / 8.160 | 5.388 / 10.732 → 2.646 / 8.351 | -50.90% |
| canonical | tfidf | 17.835 / 31.513 → 19.520 / 44.566 | 5.380 / 11.026 → 2.928 / 9.993 | -45.57% |
| numpy | bm25 | 1.326 / 4.170 → 1.411 / 11.320 | 1.179 / 1.921 → 1.184 / 1.866 | +0.46% |
| numpy | tfidf | 1.700 / 5.374 → 1.760 / 9.333 | 1.071 / 2.796 → 1.079 / 1.895 | +0.80% |
| wide | bm25 | 0.578 / 0.744 → 0.537 / 0.737 | 0.459 / 1.787 → 0.452 / 0.857 | -1.53% |
| wide | tfidf | 0.519 / 0.958 → 0.500 / 0.892 | 0.470 / 0.872 → 0.453 / 0.904 | -3.71% |
| tiny | bm25 | 0.199 / 0.318 → 0.176 / 0.305 | 0.158 / 0.261 → 0.143 / 0.258 | -9.37% |
| tiny | tfidf | 0.195 / 0.292 → 0.165 / 0.272 | 0.154 / 0.257 → 0.131 / 0.252 | -15.38% |
| tiny4 | bm25 | 0.231 / 0.384 → 0.197 / 0.361 | 0.190 / 0.290 → 0.183 / 0.207 | -3.83% |
| tiny4 | tfidf | 0.213 / 0.299 → 0.192 / 0.287 | 0.189 / 0.269 → 0.181 / 0.230 | -4.59% |
| tiny16 | bm25 | 0.217 / 0.336 → 0.220 / 0.341 | 0.199 / 0.239 → 0.198 / 0.247 | -0.77% |
| tiny16 | tfidf | 0.219 / 0.325 → 0.216 / 0.341 | 0.199 / 0.255 → 0.200 / 0.251 | +0.52% |
| tiny-no-progress | bm25 | 0.079 / 0.169 → 0.088 / 0.170 | 0.072 / 0.144 → 0.070 / 0.108 | -1.91% |
| tiny-no-progress | tfidf | 0.079 / 0.120 → 0.083 / 0.125 | 0.072 / 0.109 → 0.072 / 0.091 | +0.38% |
| tiny4-no-progress | bm25 | 0.142 / 0.279 → 0.138 / 0.266 | 0.125 / 0.160 → 0.125 / 0.148 | -0.20% |
| tiny4-no-progress | tfidf | 0.144 / 0.189 → 0.132 / 0.200 | 0.127 / 0.170 → 0.124 / 0.158 | -2.50% |
| tiny16-no-progress | bm25 | 0.153 / 0.280 → 0.154 / 0.284 | 0.148 / 0.169 → 0.146 / 0.172 | -1.44% |
| tiny16-no-progress | tfidf | 0.156 / 0.248 → 0.162 / 0.281 | 0.148 / 0.189 → 0.148 / 0.185 | -0.52% |

| Case | Startup median ms: old → new | First TF-IDF median ms: old → new | Peak RSS median MiB: old → new |
| --- | --- | --- | --- |
| canonical | 4515.078 → 4470.095 | 20.583 → 20.155 | 1724.062 → 1724.172 |
| numpy | 199.748 → 190.398 | 2.350 → 3.139 | 57.922 → 58.469 |
| wide | 4.127 → 4.128 | 0.543 → 0.550 | 7.719 → 7.812 |
| tiny | 4.195 → 4.017 | 0.230 → 0.235 | 5.781 → 5.812 |
| tiny4 | 3.723 → 3.764 | 0.298 → 0.289 | 5.797 → 5.828 |
| tiny16 | 3.401 → 3.487 | 0.290 → 0.275 | 5.906 → 6.047 |
| tiny-no-progress | 3.483 → 3.455 | 0.122 → 0.131 | 5.641 → 5.766 |
| tiny4-no-progress | 3.473 → 3.554 | 0.190 → 0.208 | 5.812 → 5.891 |
| tiny16-no-progress | 3.515 → 3.635 | 0.209 → 0.214 | 5.984 → 6.094 |

## Retained Core controls

These controls retain positive and negative timing changes. Most small/sparse fixtures do not activate posting cursors. Their timings still include changes around the fallback route and must be judged from their own rows; slower results are not dismissed as unrelated variation. Seven alternating pairs give fourteen plain processes per family. Norms are initialized for warm timing; two separate DHAT processes measure allocations. Instrumented durations are excluded from latency comparisons. Every result signature agrees and temporary-call retained deltas are zero.

### Small warm Core: 84,000 calls on 1/4/16 entries

| Fixture | Median change | Blocks/call: old → new | Allocated bytes/call: old → new |
| --- | --- | --- | --- |
| tiny16_bm25 | +4.18% | 32.0 → 32.0 | 2712.0 → 2712.0 |
| tiny16_tfidf | +1.74% | 32.0 → 32.0 | 2712.0 → 2712.0 |
| tiny4_bm25 | +0.87% | 16.0 → 16.0 | 1284.0 → 1284.0 |
| tiny4_tfidf | +1.64% | 16.0 → 16.0 | 1284.0 → 1284.0 |
| tiny_bm25 | +7.57% | 10.0 → 10.0 | 858.0 → 858.0 |
| tiny_tfidf | +5.03% | 10.0 → 10.0 | 858.0 → 858.0 |

The six small Core medians are all higher in this run (0.87–7.57%). Their allocation counts are unchanged. They do not activate posting cursors, but still include surrounding code changes and measured host variation; these regressions are retained rather than dismissed.

### All 25 warm controls: 282,800 calls

| Fixture | Median change | Blocks/call: old → new | Allocated bytes/call: old → new |
| --- | --- | --- | --- |
| coverage_repeated | +0.63% | 1047.0 → 1047.0 | 92942.0 → 92942.0 |
| coverage_small | -1.25% | 784.0 → 784.0 | 78046.0 → 78046.0 |
| excerpt_ascii | +0.86% | 762.0 → 762.0 | 252438.0 → 252438.0 |
| excerpt_mixed | -0.31% | 569.0 → 569.0 | 417113.0 → 417113.0 |
| excerpt_unicode | +0.96% | 197.0 → 197.0 | 72254.0 → 72254.0 |
| long_repeated | -16.57% | 4.0 → 4.0 | 18798.0 → 18798.0 |
| mixed_unicode | +0.55% | 7.0 → 7.0 | 3155.0 → 3155.0 |
| punctuation | +0.36% | 1.0 → 1.0 | 768.0 → 768.0 |
| relevant_retrieval | -12.57% | 812.0 → 812.0 | 93562.0 → 93562.0 |
| repeated_search | +6.99% | 39.0 → 39.0 | 19736.0 → 19736.0 |
| repeated_short | -13.90% | 4.0 → 4.0 | 1656.0 → 1656.0 |
| sparse_bm25_128 | -1.76% | 359.0 → 359.0 | 29074.0 → 29074.0 |
| sparse_bm25_33 | +1.01% | 241.0 → 241.0 | 16309.0 → 16309.0 |
| sparse_bm25_512 | -0.38% | 1337.0 → 1337.0 | 119124.0 → 119124.0 |
| sparse_tfidf_128 | +1.80% | 474.0 → 474.0 | 35140.0 → 35140.0 |
| sparse_tfidf_33 | +2.55% | 241.0 → 241.0 | 16309.0 → 16309.0 |
| sparse_tfidf_512 | -2.04% | 1337.0 → 1337.0 | 119124.0 → 119124.0 |
| tfidf_union_32 | -0.83% | 244.0 → 244.0 | 19834.0 → 19834.0 |
| tfidf_union_33 | -1.15% | 296.0 → 296.0 | 27854.0 → 27854.0 |
| tfidf_warm | -0.25% | 71.0 → 71.0 | 19335.0 → 19335.0 |
| union_32 | +1.57% | 244.0 → 244.0 | 19834.0 → 19834.0 |
| union_33 | +0.18% | 296.0 → 296.0 | 27854.0 → 27854.0 |
| unique_long | -6.11% | 299.0 → 299.0 | 32672.0 → 32672.0 |
| unique_search | +0.64% | 72.0 → 72.0 | 19350.0 → 19350.0 |
| unique_short | +0.25% | 296.0 → 296.0 | 18548.0 → 18548.0 |

### Cold norm and heap controls

Six fixtures make twenty fresh-index-clone calls each across fourteen processes: 1,680 calls. Clone/setup is outside timing. Process/instruction/filesystem caches remain; cold means norm hints, not disks. Separate heap runs drop results before net retained measurements. The existing inline/optional heap word memo and persistent norm storage remain. Large small-term requests may add up to four temporary posting cursors; canonical allocation traffic below records that cost. Heap profiles exclude stack state/control fields and temporary frames.

| Fixture | Old median / p95 ns | New median / p95 ns |
| --- | --- | --- |
| index | 114771.000 / 131408.350 | 118541.500 / 134194.100 |
| wide | 34312.500 / 43137.500 | 38020.500 / 49793.750 |
| sparse | 486333.500 / 558956.250 | 488041.500 / 563871.450 |
| tiny | 625.000 / 1591.350 | 542.000 / 1877.050 |
| tiny4 | 834.000 / 1336.050 | 916.000 / 1671.150 |
| tiny16 | 2625.000 / 3583.000 | 2750.000 / 3794.050 |

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
| new_group_query_ns | 115208.000 / 131014.550 | 117750.000 / 141756.250 |
| post_readers_first_ns | 124334.000 / 129866.400 | 123917.000 / 131274.800 |
| post_readers_second_ns | 87625.000 / 91999.400 | 92709.000 / 94783.600 |

The private document corpus retains 72 Markdown files, 2,755,468 bytes and 1,910 chunks. All 342 authored questions (276 positive, 66 unsupported) are exposed regressions, not fresh holdouts or logged production queries. Literal BM25, prepared BM25 and coverage retain six passages, 100 candidates, 2,000 bytes/pass and 12,000 total bytes. All **1,026 complete original response JSON strings match byte-for-byte**, including scores, ordering, evidence, source text and positions. One warm-up and five repeats/question/configuration/version give **10,260 timed round trips**. These three configurations are BM25 controls; independent score oracles and actual protocol pairs cover TF-IDF. No new relevance judgment or model reply is produced.

All document timing aggregates and per-question results remain in the records. Construction uses forty plain processes on canonical data and three synthetic distributions, plus six separate profiles. Parsing/generation is outside construction timing; stats/signatures/live derived heap agree. The execution/shared-index worker checks all 167,942 entries through 128 changes/restoration, rebuilds, JSON round trips, retained clones, parallel readers and cancellation/deadline rejection. Portable exports and source hashes agree.

### Document round trips

| Case | Old median / p95 ms | New median / p95 ms |
| --- | --- | --- |
| baseline | 0.945 / 1.799 | 0.959 / 1.914 |
| prepared | 1.056 / 2.004 | 1.061 / 2.091 |
| evidence | 1.044 / 1.957 | 1.058 / 2.057 |

### Construction only

| Case | Old median / p95 ms | New median / p95 ms |
| --- | --- | --- |
| canonical | 5187.002 / 6668.524 | 5289.992 / 7232.417 |
| ascii | 14.954 / 17.112 | 16.470 / 20.960 |
| mixed | 34.399 / 38.208 | 37.202 / 44.131 |
| long_tokens | 1.582 / 1.892 | 1.718 / 2.310 |

## Direct canonical search with explicit control modes

Fourteen freshly built-program processes (seven alternating pairs) retain one warmed Engine each. Ten rounds rotate plain, deadline and progress controls within each query/ranker pair. This gives **8,400 timed Retriever calls**, excluding loading, warm-up/readiness, stdio and profiling. The progress mode includes an observer and the same deadline/cancellation setup; plain mode has no WorkControl. Every timing signature agrees, and **140 complete original response JSON pairs match**. Each program also checks its control-mode responses against plain output before timing. The readiness contains private source text and remains private.

Aggregate medians combine different queries; use per-query values as well. These are controlled Core retrieval boundaries, not MCP round trips or capacity measurements.

| Control | Ranker | Old median / p95 ms | New median / p95 ms | Median change |
| --- | --- | --- | --- | --- |
| plain | Bm25 | 4.595 / 10.653 | 1.941 / 3.961 | -57.76% |
| plain | Tfidf | 4.502 / 11.134 | 2.155 / 4.771 | -52.13% |
| deadline | Bm25 | 4.708 / 10.863 | 2.218 / 4.586 | -52.88% |
| deadline | Tfidf | 4.647 / 11.675 | 2.490 / 5.611 | -46.42% |
| progress | Bm25 | 4.624 / 10.890 | 2.227 / 4.639 | -51.84% |
| progress | Tfidf | 4.651 / 11.476 | 2.472 / 5.534 | -46.84% |

| Query number | Candidates | Progress BM25 median ms: old → new | Progress TF-IDF median ms: old → new |
| --- | --- | --- | --- |
| 0 | 12365 | 4.249 → 2.061 | 4.471 → 2.326 |
| 1 | 13864 | 5.298 → 2.337 | 5.126 → 2.652 |
| 2 | 24299 | 9.932 → 3.910 | 10.268 → 4.299 |
| 3 | 596 | 0.304 → 0.290 | 0.291 → 0.295 |
| 4 | 15403 | 5.133 → 2.385 | 5.097 → 2.669 |
| 5 | 24140 | 8.897 → 3.896 | 9.032 → 4.302 |
| 6 | 3358 | 1.321 → 0.721 | 1.285 → 0.756 |
| 7 | 29600 | 10.056 → 4.207 | 9.911 → 4.892 |
| 8 | 11504 | 3.672 → 1.849 | 3.312 → 2.084 |
| 9 | 0 | 0.002 → 0.002 | 0.001 → 0.001 |

## Separate fresh three-version canonical reference

Five rotating triples compare **fresh registry 0.4.14, fresh registry 0.4.15 and the candidate**, with the same program/input/profile and ordinary requested progress. Fifteen servers validate **1,020 actual schema replies**. Their **680 complete structured comparisons** against the candidate match by RPC ID, excluding only checked status version and expected version-qualified schema URNs. Tool definitions/source ranges and inputs agree. Initial/repeated/startup/RAM scopes remain separate.

These trials are separate from both ordinary campaigns; no sample replaces an earlier slower result or is merged into another distribution. A faster result here does not by itself identify why the original 0.4.15 campaigns differed.

| Version | Ranker | Initial median / p95 ms | Repeated median / p95 ms | Peak child RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.14 | bm25 | 7.101 / 12.673 | 5.938 / 11.063 | 1724.156 |
| 0.4.14 | tfidf | 19.019 / 34.520 | 5.729 / 11.927 | 1724.156 |
| 0.4.15 | bm25 | 6.763 / 12.606 | 6.044 / 11.443 | 1724.047 |
| 0.4.15 | tfidf | 21.515 / 35.902 | 5.886 / 13.381 | 1724.047 |
| 0.4.16 | bm25 | 3.254 / 5.424 | 2.740 / 5.058 | 1724.172 |
| 0.4.16 | tfidf | 17.754 / 31.507 | 3.016 / 5.483 | 1724.172 |

| Query number | 0.4.14 warm TF-IDF median ms | 0.4.15 warm TF-IDF median ms | 0.4.16 warm TF-IDF median ms |
| --- | --- | --- | --- |
| 0 | 5.266 | 5.211 | 2.730 |
| 1 | 6.165 | 6.209 | 3.079 |
| 2 | 10.830 | 11.244 | 4.855 |
| 3 | 1.371 | 0.676 | 0.648 |
| 4 | 6.153 | 5.950 | 3.111 |
| 5 | 10.346 | 10.500 | 4.913 |
| 6 | 1.819 | 1.824 | 1.180 |
| 7 | 10.055 | 10.681 | 5.435 |
| 8 | 5.077 | 5.612 | 3.247 |
| 9 | 1.243 | 0.118 | 0.106 |

## Allocation scope on canonical first-pass work

The exact existing norm memo/cache remains, while eligible large requests can add posting-cursor scratch. Two separate instrumented processes load/index all records before profiling the same ten initial TF-IDF calls. Results are dropped before net deltas. Allocated traffic includes cursor lifetime; retained bytes reflect persistent hints. These are not stack measurements or process-RAM bounds.

| Canonical first pass | 0.4.15 | 0.4.16 |
| --- | --- | --- |
| Retained hint bytes | 2228312 | 2228312 |
| Retained hint blocks | 2 | 2 |
| Total allocated query bytes | 7216195 | 7217203 |
| Total query allocation blocks | 443 | 451 |

## Development record and host-load limits

The [sampled baseline summary](measurements/0.4.16/baseline-profile-summary.json), [rejected document-lookup pilot](measurements/0.4.16/development-document-lookup-summary.json) and [selected posting pilot](measurements/0.4.16/development-posting-summary.json) are preserved separately. The sampler records top frames, not a standalone attribution of an entire latency regression. The first prototype was rejected despite some favorable MCP observations because direct controls became slower. The second kept exact complete responses and improved both direct/ordinary large pilot observations. Initial tiny calls were slower in that pilot. Pilots occurred at different times, with their own baselines and overheads; their absolute values are not interchangeable and none is a final benchmark sample.

| Pilot | Direct progress median ms: old → new | Canonical warm MCP median ms | Tiny initial MCP median ms | Tiny warm MCP median ms |
| --- | --- | --- | --- | --- |
| Rejected document sweep | 4.459 → 5.005 | 8.332 → 7.131 | 0.162 → 0.163 | 0.139 → 0.157 |
| Selected posting cursors | 4.200 → 2.176 | 5.666 → 2.957 | 0.140 → 0.154 | 0.128 → 0.127 |

The final two-second monitor records **340 load snapshots**, including **97 snapshots with other rustc processes** (maximum 3). Managed compiler work completed before measurement; other workloads were not stopped. CPU counts include active benchmark workers. These snapshots have gaps, do not prove continuous idleness and cannot establish a timing cause. Read [all sampled load](measurements/0.4.16/host-samples.json) and [stage-start observations](measurements/0.4.16/host-activity.json).

The following boundaries assign the same coarse observations to stages; they are not new timing trials.

| Stage | Load snapshots | Snapshots with other rustc processes |
| --- | ---: | ---: |
| retrieval | 7 | 7 |
| query | 86 | 43 |
| build | 34 | 20 |
| controlled | 54 | 7 |
| mcp | 45 | 0 |
| corpus | 18 | 0 |
| mcp-confirmation | 46 | 15 |
| reference | 49 | 4 |

All observed regressions and tails stay in their own tables. The measured benefit concerns the tested workload, not every query distribution. Sparse/custom filters can pay seek costs; wider/small requests retain prior lookup. Further performance work must preserve exact outputs and explicit resource/cancellation bounds.

## Limits and preservation

Measurements use one AC-powered Apple M5 Max with 64 GB, Rust 1.98.0 and Python 3.14.7. Stage-start CPU/compiler snapshots remain. Own compilers finish before timing; this does not establish continuously idle hardware. Randomized hashing, scheduling, one machine/corpus, exposed questions and finite process counts limit generalization. Both selected and rejected pilot outcomes remain; assess final per-case medians and tails independently. Host-load sampling records observations rather than continuous idleness. No universal latency, total-memory, semantic-quality, token-saving or security-audit claim is made.

Canonical knowledge remains 167,942 entries from 35 PDFs and 49,086 physical pages, SHA-256 `13168d87872a137355fd8499c90a6084f52243a2ec7456f29b5fcb2a1c7423c3`. The PDF parser is unchanged; this is not a fresh large-PDF extraction campaign. Lexical overlap and exact quotes do not prove factual support or entailment. No model, daemon or DenkMaps integration is added.

Preserve PDFs, knowledge, snapshots, all pilot/final measurements, slower results, raw transcripts, profiles, exact small executables and locks. After verified publication archive package/proof, then delete only the named temporary Cargo/Miri targets. Default features remain empty.
