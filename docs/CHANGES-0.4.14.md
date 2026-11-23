# AIRust 0.4.14: warm and small TF-IDF work

This release changes private norm-hint reading and request-local scratch. Exact scoring arithmetic, deterministic ties, public Rust APIs, knowledge formats, source positions and MCP schemas remain. No dependency, feature flag, model, startup prewarming or DenkMaps integration is added.

The two preplanned canonical MCP campaigns show repeated TF-IDF median changes of **-1.66% and -1.05%** against exact registry 0.4.13. Initial-pass median changes are **+1.60% and -0.99%**. One-entry MCP repeated medians change by **-0.14% and +0.17%**. Negative percentages mean shorter observed times. Small Core, cold work, mixed traffic, heap and process memory are separate scopes below; slower outcomes are retained.

## Implementation and resource contract

One request-local NormQuery shares a successful try_read guard for up to 128 norm-hint hits. The guard is released at the boundary, on a miss, before a progress callback and before winner explanations. Cache-hit dispatch remains small and exact cold computation is a separate non-inlined helper. No guard or query state is stored in the index or returned with a result.

Misses release the query's read guard before computation and best-effort try_write publication. Other readers or busy/poisoned locks may still defer caching a complete norm. The exact answer is computed and returned without waiting for a hint write; a later quiescent query can populate it. New native regressions cover missed publication under a deliberately held reader and recovery after release. This is not a fairness or lock-free guarantee.

Word weights first use an eight-slot inline memo, 128 payload bytes on this 64-bit build. After at least 128 terms from earlier completed cold norms, a later norm may promote to the existing 256-slot / 4,096-byte heap memo. Surviving exact weights are copied; full live interned-term identities are checked before reuse, and collisions recompute the original IDF. There is no new unsafe code, public identity or persisted memo. Warm hits add no cold-work credit and leave the word memos unused; aborted work adds no completed credit. Both tiers are released with the query.

Inline state/control fields and temporary construction can increase stack frames. Heap profiles do not measure stack space. A removed 4,096-byte heap request does not imply an equal process-RAM reduction. Persistent norm storage, mutation invalidation, metadata reuse and shared-clone ownership remain. Sorted term order, floating-point operations and norm checkpoints every 128 inspected terms stay exact. Startup still loads/indexes original data without prewarming norms.

Use existing `Engine::search_with_options` with `SearchAlgorithm::Tfidf`, or MCP `airust_search` with `algorithm: "tfidf"`. No setting is added. Context/scope checks, original IDs, source binding, spans, progress, cancellation, deadlines, worker admission and argument/result budgets remain. Read [API](API.md), [cookbook](API-COOKBOOK.md), [performance](CORE-PERFORMANCE.md), [MCP](MCP.md) and [security scope](MCP-SECURITY.md).

## Executed validation

The complete matrix passes **234 default tests, 882 all-feature checks including 39 documentation examples, 39 isolated feature entries and five Python tests**. Strict Clippy/docs, Rust 1.89 provider-neutral and Rust 1.92 all-feature checks pass. API comparison against tagged 0.4.13 reports **229 pass, 31 skip**. A separate Rust 1.89 consumer exercises retrieval, coverage, quotations, document positions and MCP/schema construction. The unchanged PDF backend passes 14 regressions and its 381-source-file provenance check. A refreshed audit reports zero known vulnerabilities in the checked database, retaining existing informational chrono/paste/ttf-parser maintenance notices.

New regressions cover bounded read release, release before computing/publishing a miss, scratch promotion after completed cold work, and correct nonblocking cold computation under a held warm reader followed by hint recovery. Both 8/256-slot collision paths and exact-weight promotion are checked. Existing independent score-bit oracles, scopes, metadata/clone/reused-ID cases, poison/contention, incomplete-value rejection, hostile MCP cases and the prior deterministic snapshot-conflict example regression remain.

Sixteen finite Tree Borrows Miri cases pass, including the new serial read-release and scratch cases. The overlapping native-thread test runs in the ordinary matrices. Only extra floating-point rounding injection is disabled; memory/alias checks remain. Native/async/PDF/ONNX execution is outside the finite interpreter scope. These checks are not an independent security or soundness audit. Extracted-package tests, exact-commit CI, fresh exact-registry consumer and checksum/VCS/tag matching gate publication.

## Fixed inputs and development steps

The [programs/method](validation-0.4.14/README.md) compare exact registry 0.4.13 (VCS 6ad1fc98bc95af93e234624976c3c19587b2db5d) with the candidate using identical updated programs and inputs. Compiler jobs finish before final timing. Frozen original/extended inputs, source/program/executable hashes and locks are retained.

Two separate development pilots are retained privately. A first direct-computation variant removed early heap allocation but made four/sixteen-entry cold cases slower; it was not selected. The selected inline-memo variant showed better small Core results in that small pilot, while tiny MCP remained flat. A further mixed read/new-group check supported the selected bounded-read design. None of these pilot observations is merged into final samples. Sanitized [direct-delay](measurements/0.4.14/development-direct-summary.json) and [inline](measurements/0.4.14/development-inline-summary.json) summaries are retained alongside the final data. Their code snapshots, logs, binaries and slower values remain.

The method adds three sizes of warm Core controls, four/sixteen-entry cold controls, one-entry actual MCP input and a finite mixed-work control. Both crate versions receive these additions. Historical absolute tables with fewer/different controls are not interchangeable with these observations. No new relevance holdout or model answer is produced.

## Small warm Core controls

Prebuilt one/four/sixteen-entry engines execute both BM25 and TF-IDF. Seven alternating pairs give fourteen plain processes, six fixtures and **84,000 timed calls** (1,000 per fixture/sample). Two separately instrumented processes run 100 calls per fixture, 1,200 total. Norms are initialized before timing/profiling. Signatures agree, with zero temporary retained-byte/block deltas. Instrumented durations are not timing measurements.

| Warm fixture | Median latency change | Allocation blocks/call, old → new | Bytes/call, old → new |
| --- | ---: | ---: | ---: |
| `tiny16_bm25` | +3.76% | 32 → 32 | 2712 → 2712 |
| `tiny16_tfidf` | +0.24% | 32 → 32 | 2712 → 2712 |
| `tiny4_bm25` | +1.41% | 16 → 16 | 1284 → 1284 |
| `tiny4_tfidf` | -0.12% | 16 → 16 | 1284 → 1284 |
| `tiny_bm25` | +3.69% | 10 → 10 | 858 → 858 |
| `tiny_tfidf` | +3.39% | 10 → 10 | 858 → 858 |

## Retained warm-query controls

All 25 prior fixtures remain: fourteen plain processes and **282,800 timed calls**. Nineteen fixtures use 1,000 calls per sample; six sparse broad cases use 200. Two separate DHAT runs use 100 calls per fixture, 5,000 total. Complete result signatures match and temporary-call retained deltas are zero. Do not treat unchanged-path variation as a measured mechanism or discard its positive percentages.

| Fixture | Blocks per call, old → new | Bytes per call, old → new | Median time change |
| --- | ---: | ---: | ---: |
| `coverage_repeated` | 1047 → 1047 | 92942 → 92942 | -0.74% |
| `coverage_small` | 784 → 784 | 78046 → 78046 | -2.23% |
| `excerpt_ascii` | 762 → 762 | 252438 → 252438 | -1.52% |
| `excerpt_mixed` | 569 → 569 | 417113 → 417113 | +0.33% |
| `excerpt_unicode` | 197 → 197 | 72254 → 72254 | -1.74% |
| `long_repeated` | 4 → 4 | 18798 → 18798 | +20.89% |
| `mixed_unicode` | 7 → 7 | 3155 → 3155 | -0.46% |
| `punctuation` | 1 → 1 | 768 → 768 | +0.48% |
| `relevant_retrieval` | 812 → 812 | 93562 → 93562 | -0.39% |
| `repeated_search` | 39 → 39 | 19736 → 19736 | -0.36% |
| `repeated_short` | 4 → 4 | 1656 → 1656 | +10.24% |
| `sparse_bm25_128` | 359 → 359 | 29074 → 29074 | +2.13% |
| `sparse_bm25_33` | 241 → 241 | 16309 → 16309 | +1.96% |
| `sparse_bm25_512` | 1337 → 1337 | 119124 → 119124 | +1.68% |
| `sparse_tfidf_128` | 474 → 474 | 35140 → 35140 | +2.23% |
| `sparse_tfidf_33` | 241 → 241 | 16309 → 16309 | +1.36% |
| `sparse_tfidf_512` | 1337 → 1337 | 119124 → 119124 | +0.03% |
| `tfidf_union_32` | 244 → 244 | 19834 → 19834 | +0.71% |
| `tfidf_union_33` | 296 → 296 | 27854 → 27854 | -0.16% |
| `tfidf_warm` | 71 → 71 | 19335 → 19335 | -1.30% |
| `union_32` | 244 → 244 | 19834 → 19834 | -0.33% |
| `union_33` | 296 → 296 | 27854 → 27854 | +0.58% |
| `unique_long` | 299 → 299 | 32672 → 32672 | +8.28% |
| `unique_search` | 72 → 72 | 19350 → 19350 | -0.65% |
| `unique_short` | 296 → 296 | 18548 → 18548 | +2.53% |

The final small warm results do not reproduce the pilot's broad speed gains: five of six medians are higher, including tiny TF-IDF (+3.39%). Unchanged normalization controls also have slower long-repeated (+20.89%), repeated-short (+10.24%) and unique-long (+8.28%) samples. No mechanism or statistical significance is established for those differences; all rows remain.

## Cold norm calls and heap

The six cold fixtures are the prior 1,000-entry index, 256-entry wide index, 257-entry sparse/vocabulary index and one/four/sixteen-entry controls. Twenty fresh-index-clone queries per fixture/process across seven alternating pairs give **1,680 calls in fourteen plain processes**. Clone/setup work is outside timing; instruction/process/filesystem caches are not flushed. These are cold norm hints rather than cold disks.

| Cold norm-cache call, ns | 0.4.13 median / p95 | 0.4.14 median / p95 |
| --- | ---: | ---: |
| index | 145521.000 / 172655.950 | 146645.500 / 169335.700 |
| wide | 43645.500 / 55818.150 | 46437.500 / 59450.350 |
| sparse | 630104.000 / 776583.650 | 628188.000 / 775100.300 |
| tiny | 1625.000 / 6674.400 | 1500.000 / 6544.050 |
| tiny4 | 1500.000 / 2216.350 | 1375.000 / 2044.100 |
| tiny16 | 4166.000 / 5344.350 | 4020.500 / 6544.050 |

Two instrumented processes record six first-query heap cases after dropping results, excluding source/index setup. Allocated bytes include query scratch; net retained bytes measure persistent hint growth. Inline memory is absent from this heap accounting.

| Cold hint fixture | Allocation blocks, old → new | Allocated bytes, old → new | Retained hint bytes, old → new |
| --- | ---: | ---: | ---: |
| index | 50 → 50 | 92660 → 92660 | 34904 → 34904 |
| wide | 56 → 56 | 19549 → 19549 | 4440 → 4440 |
| sparse | 1347 → 1347 | 140704 → 140704 | 8792 → 8792 |
| tiny | 13 → 12 | 5110 → 1014 | 156 → 156 |
| tiny4 | 20 → 19 | 5680 → 1584 | 224 → 224 |
| tiny16 | 38 → 37 | 7940 → 3844 | 632 → 632 |

Two further processes load/index all 167,942 canonical entries before profiling the same ten initial TF-IDF queries. Folded signatures agree. Results are dropped before net deltas.

| Canonical first-pass work | 0.4.13 | 0.4.14 |
| --- | ---: | ---: |
| Net retained norm-hint bytes | 2228312 | 2228312 |
| Net retained norm-hint blocks | 2 | 2 |
| Total allocated query bytes | 7216195 | 7216195 |
| Total query allocation blocks | 443 | 443 |

Heap traffic, retained hints and whole-process RSS are different quantities. The small inline payload/control state still uses stack space, while a promoted memo still requests the 4 KiB payload. Workload shape, hash seeds and table growth affect the observed costs.

## Finite mixed read/new-group control

One synthetic 6,000-entry engine has 5,000 warm-group entries and 1,000 initially uncached-group entries. Prime only the warm group, start four continuous warm readers, then make 100 new-group queries and stop the readers. Five alternating pairs yield ten processes and **1,000 new-group calls**, followed by two quiescent queries per process. Every result matches the fixed signature/reference. New-group calls can become partly/warm cached during the loop; they are not all cold requests.

| Mixed control, ns | 0.4.13 median / p95 | 0.4.14 median / p95 |
| --- | ---: | ---: |
| Initially uncached group under four warm readers | 525833.000 / 595281.250 | 128229.000 / 145897.950 |
| First query after readers stop | 92375.000 / 98541.600 | 128917.000 / 165000.200 |
| Second query after readers stop | 82750.000 / 86875.200 | 90250.000 / 91325.200 |

Warm reader call totals are 424 for 0.4.13 and 591 for 0.4.14. The stop condition means these counts and offered work vary with completion speed. These finite observations are not a fixed-load throughput or capacity benchmark. All per-process samples remain in the records. Best-effort hint publication may be deferred while readers are active; later quiescent behavior is shown separately. Both post-reader medians are higher for the candidate, so the concurrent improvement must not be generalized to every later call.

## Actual MCP: four inputs, initial and repeated passes

Both preplanned campaigns use canonical 167,942-entry knowledge, the revision-bound NumPy snapshot, deterministic synthetic wide input and the new one-entry knowledge control. Five alternating old/new pairs per input give **forty servers per campaign, eighty total**. Each executes the ten queries with both rankers once, then two repeated passes. Every repeated payload must match its initial structured value exactly.

Across both campaigns **7,080 actual result/error replies validate against advertised schemas**, and **3,540 complete old/new structured pairs match by JSON-RPC request ID**, excluding only status version and asserted version-qualified schema URNs. Counts are 1,600 initial searches, 3,200 repeated searches, 320 concurrent requests, 1,640 exact snapshot readbacks, 160 matching/relationship calls, 80 rejected arguments and 80 health calls. Discovery/schema/source binding, progress, concurrent ordering, bounds, health and EOF shutdown pass. Input hashes remain unchanged.

### First complete campaign

| MCP observation, ms unless stated | 0.4.13 | 0.4.14 |
| --- | ---: | ---: |
| Canonical startup median | 4387.05 | 4472.31 |
| Canonical first TF-IDF call median | 19.523 | 20.013 |
| Canonical initial pass TFIDF, median / p95 | 16.966 / 31.242 | 17.238 / 31.600 |
| Canonical repeated BM25, median / p95 | 5.225 / 10.012 | 5.033 / 9.972 |
| Canonical repeated TFIDF, median / p95 | 5.418 / 10.334 | 5.328 / 9.847 |
| Canonical whole-child peak RSS, median MiB | 1724.17 | 1724.20 |
| Snapshot startup median | 123.81 | 125.02 |
| Snapshot first TF-IDF call median | 1.579 | 1.576 |
| Snapshot initial pass TFIDF, median / p95 | 1.593 / 3.464 | 1.629 / 3.409 |
| Snapshot repeated BM25, median / p95 | 1.635 / 1.774 | 1.628 / 1.713 |
| Snapshot repeated TFIDF, median / p95 | 1.660 / 1.910 | 1.645 / 1.822 |
| Snapshot whole-child peak RSS, median MiB | 57.28 | 57.34 |
| Synthetic wide startup median | 3.77 | 3.83 |
| Synthetic wide first TF-IDF call median | 1.619 | 1.604 |
| Synthetic wide initial pass TFIDF, median / p95 | 1.591 / 1.661 | 1.590 / 1.658 |
| Synthetic wide repeated BM25, median / p95 | 1.602 / 1.719 | 1.599 / 1.727 |
| Synthetic wide repeated TFIDF, median / p95 | 1.614 / 1.700 | 1.614 / 1.694 |
| Synthetic wide whole-child peak RSS, median MiB | 8.03 | 7.98 |
| One-entry startup median | 3.29 | 3.27 |
| One-entry first TF-IDF call median | 1.492 | 1.495 |
| One-entry initial pass TFIDF, median / p95 | 1.446 / 1.496 | 1.440 / 1.489 |
| One-entry repeated BM25, median / p95 | 1.448 / 1.499 | 1.449 / 1.490 |
| One-entry repeated TFIDF, median / p95 | 1.447 / 1.479 | 1.445 / 1.478 |
| One-entry whole-child peak RSS, median MiB | 5.80 | 5.72 |

### Second preplanned campaign

| MCP observation, ms unless stated | 0.4.13 | 0.4.14 |
| --- | ---: | ---: |
| Canonical startup median | 4370.33 | 4397.15 |
| Canonical first TF-IDF call median | 19.854 | 20.352 |
| Canonical initial pass TFIDF, median / p95 | 17.591 / 31.728 | 17.416 / 31.575 |
| Canonical repeated BM25, median / p95 | 5.156 / 10.034 | 5.320 / 10.235 |
| Canonical repeated TFIDF, median / p95 | 5.352 / 9.985 | 5.296 / 10.818 |
| Canonical whole-child peak RSS, median MiB | 1724.16 | 1724.09 |
| Snapshot startup median | 123.97 | 124.70 |
| Snapshot first TF-IDF call median | 1.561 | 1.562 |
| Snapshot initial pass TFIDF, median / p95 | 1.599 / 3.458 | 1.599 / 3.406 |
| Snapshot repeated BM25, median / p95 | 1.646 / 1.750 | 1.631 / 1.734 |
| Snapshot repeated TFIDF, median / p95 | 1.648 / 1.845 | 1.649 / 1.835 |
| Snapshot whole-child peak RSS, median MiB | 57.28 | 57.17 |
| Synthetic wide startup median | 3.76 | 3.82 |
| Synthetic wide first TF-IDF call median | 1.635 | 1.593 |
| Synthetic wide initial pass TFIDF, median / p95 | 1.604 / 1.679 | 1.587 / 1.659 |
| Synthetic wide repeated BM25, median / p95 | 1.598 / 1.719 | 1.602 / 1.717 |
| Synthetic wide repeated TFIDF, median / p95 | 1.618 / 1.716 | 1.608 / 1.718 |
| Synthetic wide whole-child peak RSS, median MiB | 8.02 | 7.91 |
| One-entry startup median | 3.29 | 3.26 |
| One-entry first TF-IDF call median | 1.472 | 1.469 |
| One-entry initial pass TFIDF, median / p95 | 1.447 / 1.487 | 1.450 / 1.490 |
| One-entry repeated BM25, median / p95 | 1.452 / 1.490 | 1.448 / 1.490 |
| One-entry repeated TFIDF, median / p95 | 1.450 / 1.475 | 1.453 / 1.482 |
| One-entry whole-child peak RSS, median MiB | 5.81 | 5.73 |

Canonical warm median gains are modest (about 1–2%). The one-entry MCP medians are essentially flat, with a slightly higher second-campaign p95. The second canonical warm TF-IDF p95 rises from 9.985 to 10.818 ms; its first TF-IDF call is slower in both campaigns. Whole-process RSS is essentially unchanged. These data do not establish a general tiny-query or tail-latency gain.

First calls, aggregate initial passes and repeated passes are distinct scopes; later initial queries can already reuse earlier norms. Snapshot follow-up reads are outside search timing. Search includes progress and stdio transport; startup includes synchronous loading/indexing outside request deadlines. Tiny Core changes can be hidden by transport/runtime scheduling. Whole-child peak RSS includes input buffers, index, hints and runtime. Judge slower medians and tails from their own rows.

## Complete document responses and construction controls

The private document corpus remains 72 Markdown files, 2,755,468 source bytes and 1,910 chunks. All 342 authored questions (276 positive, 66 unsupported) are exposed regressions, not fresh holdout or logged production questions. Literal BM25, prepared BM25 and coverage retain six passages, 100 candidates, 2,000 bytes per passage and 12,000 total bytes.

All **1,026 complete original response JSON strings match byte-for-byte**, including scores, IDs, order, terms, source text/positions and recipes. Every timed repeat matches. One warm-up plus five repeats per question/configuration/version gives **10,260 timed round trips**, including evidence/source work, serialization, IPC and Python parsing. These three configurations use BM25; TF-IDF is checked by independent score oracles and actual protocol pairs. Previous misses remain, without a new semantic judgment/model reply.

| Document context, ms | 0.4.13 median / p95 | 0.4.14 median / p95 |
| --- | ---: | ---: |
| baseline | 0.748 / 1.126 | 0.740 / 1.107 |
| prepared | 0.798 / 1.238 | 0.797 / 1.233 |
| evidence | 0.788 / 1.219 | 0.785 / 1.212 |

Construction remains a control: five pairs for canonical data and three synthetic distributions, forty plain processes, parsing/input generation outside the timer. Six separate profiles check derived live heap; stats/signatures/live derived bytes match.

| Construction only, ms | 0.4.13 median / p95 | 0.4.14 median / p95 |
| --- | ---: | ---: |
| canonical | 4524.603 / 4553.984 | 4524.069 / 4582.939 |
| ascii | 13.851 / 13.925 | 13.650 / 13.815 |
| mixed | 32.376 / 32.750 | 32.395 / 32.642 |
| long_tokens | 1.476 / 1.507 | 1.440 / 1.464 |

The execution/shared-index worker also validates all 167,942 entries through 128 changes/restoration, rebuilds, JSON round trips, four readers with forty queries each, retained clones and cancellation/deadline rejection. Source and portable export identities agree. This is finite consistency validation, not a new PDF parser or million-user capacity test.

## Conditions and preservation

Measurements use one AC-powered Apple M5 Max with 64 GB, Rust 1.98.0 and Python 3.14.7. Stage-start CPU/compiler counts are retained; they do not establish continuously idle hardware. One corpus, authored/exposed questions, randomized hashing and small process counts limit generalization. Positive and negative timing changes are observations, not significance tests or a universal latency guarantee. Read [all sanitized records](measurements/0.4.14/README.md). Raw data, transcripts, profiles, source snapshots, exact small executables/locks and logs stay private and preserved.

Lexical scores and exact quotations do not establish truth or entailment. There is no new semantic-quality, token-saving, local-model or fresh PDF extraction claim. No daemon is started. Canonical knowledge remains **167,942 entries from 35 PDFs and 49,086 physical pages**, SHA-256 `13168d87872a137355fd8499c90a6084f52243a2ec7456f29b5fcb2a1c7423c3`.

Preserve PDFs, knowledge, snapshots and every measurement. After verified publication, archive package/proof/locks/small programs before deleting only the named temporary Cargo/Miri targets. Defaults remain empty; no DenkMaps integration is performed.

Remaining work should isolate the tiny request transport/JSON path and repeat the slower normalization controls before changing them. Preserve exact-output checks and separate stack/heap/whole-process measurements as the gates for further optimization.
