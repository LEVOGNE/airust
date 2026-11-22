# AIRust 0.4.12: compact cold TF-IDF hints

This release changes the private norm-cache key. Existing public signatures, scoring arithmetic, deterministic ties, source positions, knowledge formats and MCP schemas remain. No dependency, feature flag, model, database migration or DenkMaps integration is added.

On the retained canonical first-pass heap profile, net norm-hint storage changes from **3,276,888 to 2,228,312 bytes (-32.00%)** after result vectors are dropped. That is a cache-component measurement, not a comparable reduction in total process RAM. Cold and warm times, initial/repeated MCP requests, slower cases and whole-process RSS are reported separately below. No universal gain is claimed.

## Implementation and ownership

The standard randomly seeded HashMap now stores a machine-word identity of each ID allocation already owned by the central KnowledgeIndex. On this 64-bit build it replaces a two-word Arc string key, avoids extra ID reference retention and hashes one integer instead of the ID text. There is no custom unprotected hasher and no new unsafe code. The integer is never dereferenced, persisted, exposed as an application ID or accepted from callers. Original public string IDs and tie order remain.

Unchanged clones share valid hints while retaining their ID allocations. Metadata-only changes keep those allocations. Text/corpus changes, removals and clear discard the changing view's cache before its next query; retained views remain valid. Rebuild/load creates fresh hints. Completed norms are published only after checkpoints. Contended or poisoned locks fall back to exact nonblocking computation. BM25 and unknown queries allocate no hint cache. There is no added per-document field, eager corpus-sized reservation or second index. The broad scoring and candidate improvements from 0.4.11 remain unchanged.

Existing `Engine::search_with_options` with `SearchAlgorithm::Tfidf` and MCP `airust_search` with `algorithm: "tfidf"` use the optimization automatically. Public source binding, progress, cancellation, absolute deadlines, worker admission and argument/output budgets remain. See [API](API.md), [cookbook](API-COOKBOOK.md), [performance](CORE-PERFORMANCE.md), [MCP](MCP.md) and [security scope](MCP-SECURITY.md).

## Executed validation

The full matrix passes **229 default tests, 877 all-feature checks including 39 documentation examples, 39 isolated feature entries and five Python tests**. Strict Clippy/docs, Rust 1.89 provider-neutral and Rust 1.92 all-feature checks pass. API comparison against tagged 0.4.11 reports **229 pass, 31 skip**. A separate Rust 1.89 consumer exercises retrieval, coverage, quotations, source positions and MCP/schema construction. The unchanged PDF backend passes fourteen regressions and its 381-source-file provenance check. The refreshed dependency audit reports zero known vulnerabilities in its checked database, retaining existing informational chrono/paste/ttf-parser maintenance notices.

New regressions verify that hints add no ID allocation ownership, concurrent first queries return exact scores, and warmed retained clones remain correct through metadata updates, repeated removals/reinsertions with reused IDs, text/corpus edits, clear and JSON reload. Existing independent full-scan score-bit oracles, scopes, cold/warm cache states, interruption-before-publication, poison/contention, hostile MCP arguments and cancellation/deadline cases pass.

Twelve targeted Tree Borrows Miri cases pass, including identifier ownership and retained-clone/reused-ID regressions. The latter uses three mutation rounds under Miri and 64 normally. Only extra floating-point rounding injection is disabled; memory/alias checks remain. Async/native PDF/ONNX work is outside these finite cases. This is not an independent security or soundness audit. Extracted-package tests, exact-commit CI and a fresh exact-registry consumer with matching VCS/tag/checksum are publication gates.

## CI-found snapshot stress example regression

The [first exact-commit CI run](https://github.com/LEVOGNE/airust/actions/runs/45054044739) failed in the existing snapshot stress example. Two writers can race between acquiring a snapshot and preparing a candidate. The library correctly returns `DatasetError::Conflict` at preparation, but the example unwrapped it and retried only publication conflicts. The failed writer then left readers running until their guard fired. Publication stopped before tagging or crates.io upload.

The example now returns preparation and publication errors through one attempt function and retries expected conflicts against a fresh snapshot. Other errors still fail; operation/conflict guards and the deadline remain. A deterministic stale-preparation regression is part of `test.sh`. The complete local matrix is repeated after the fix, followed by 24 retained stress processes across default/shared-index configurations and 1,000/10,000 entries, four readers, two writers and ten commits each. Current and retained snapshots remain consistent. See [failure/fix provenance](measurements/0.4.12/ci-example-fix.json) and [all repetitions](measurements/0.4.12/stress-example-repeats.json).

Executable code changes after measurement are limited to the example and test runner. Library source hashes, benchmark programs and exact measured executable hashes remain identical; the retained performance comparisons therefore still use the published library implementation. Failed CI logs and the earlier package/commit are preserved. The corrected package and exact commit must pass CI and all publication gates again.

## Cold norm-cache calls and retained hint memory

Three fixed prebuilt engines retain the prior 1,000-entry index, 256-entry wide index and 257-entry sparse/vocabulary index. In seven alternating old/new process pairs, each plain process makes twenty queries per fixture using fresh clones of a never-queried Engine. Clone/setup work is outside the timer. There are **840 cold-cache calls across fourteen processes**, with equal signatures. These are cold norm hints in an ordinary warmed process; instruction/filesystem caches are not flushed. No instrumented duration is used as latency data.

| Cold norm-cache call, ns | 0.4.11 median / p95 | 0.4.12 median / p95 |
| --- | ---: | ---: |
| index | 178562.500 / 194674.700 | 171791.500 / 188277.100 |
| wide | 87125.000 / 94419.050 | 85667.000 / 95050.300 |
| sparse | 523166.500 / 577445.850 | 514625.000 / 576427.400 |

Two separate instrumented processes measure first-query allocation traffic and retained hint bytes after dropping results; original sources/index construction are excluded. The retained deltas isolate hint storage. Allocation traffic includes the complete query's temporary work.

| Cold hint fixture | Allocation blocks, old → new | Allocated bytes, old → new | Retained hint bytes, old → new |
| --- | ---: | ---: | ---: |
| index | 49 → 49 | 121300 → 88564 | 51288 → 34904 |
| wide | 55 → 55 | 19517 → 15453 | 6488 → 4440 |
| sparse | 1346 → 1346 | 144768 → 136608 | 12888 → 8792 |

A further two instrumented processes load all 167,942 canonical entries before profiling and execute the same ten fixed queries used for MCP. Folded score signatures agree. The net per-query deltas below exclude returned result vectors and input/index setup. They are hint growth, not total process memory.

| Canonical first pass | 0.4.11 | 0.4.12 |
| --- | ---: | ---: |
| Net retained hint bytes | 3276888 | 2228312 |
| Net retained hint blocks | 2 | 2 |
| Total allocated query bytes | 9280547 | 7183427 |
| Total query allocation blocks | 435 | 435 |

The compact keys target this cache component. Existing source buffers, normalized terms, postings, document entries, runtime and transport remain. Peak process RSS can differ or remain effectively unchanged despite a smaller cache. Hash-table growth/load factors and hash seeds still affect allocation sizes. Do not interpret allocation counts as a latency or whole-application RAM guarantee.

## Retained warm-query controls

The [public programs/method](validation-0.4.12/README.md) compare exact registry 0.4.11 (VCS 5f729eb985d2726eff6bd9e75facb6bcf459cc8b) with candidate 0.4.12 using identical source inputs and program configuration. All 25 prior fixtures remain. Seven alternating old/new pairs give fourteen plain processes and **282,800 timed calls**. Two separate DHAT builds execute 100 calls per fixture, 5,000 calls total. Warm-ups initialize hints before measurement. Signatures agree and temporary-call retained-byte/block deltas are zero in both versions. Instrumented timing is excluded.

| Fixture | Blocks per call, old → new | Bytes per call, old → new | Median time change |
| --- | ---: | ---: | ---: |
| `coverage_repeated` | 1047 → 1047 | 92942 → 92942 | +6.00% |
| `coverage_small` | 784 → 784 | 78046 → 78046 | +1.82% |
| `excerpt_ascii` | 762 → 762 | 252438 → 252438 | +2.15% |
| `excerpt_mixed` | 569 → 569 | 417113 → 417113 | +5.98% |
| `excerpt_unicode` | 197 → 197 | 72254 → 72254 | +6.02% |
| `long_repeated` | 4 → 4 | 18798 → 18798 | +3.69% |
| `mixed_unicode` | 7 → 7 | 3155 → 3155 | -0.37% |
| `punctuation` | 1 → 1 | 768 → 768 | +1.84% |
| `relevant_retrieval` | 812 → 812 | 93562 → 93562 | +2.42% |
| `repeated_search` | 39 → 39 | 19736 → 19736 | +3.19% |
| `repeated_short` | 4 → 4 | 1656 → 1656 | -2.83% |
| `sparse_bm25_128` | 359 → 359 | 29074 → 29074 | -1.10% |
| `sparse_bm25_33` | 241 → 241 | 16309 → 16309 | +0.79% |
| `sparse_bm25_512` | 1337 → 1337 | 119124 → 119124 | -0.64% |
| `sparse_tfidf_128` | 474 → 474 | 35140 → 35140 | +1.00% |
| `sparse_tfidf_33` | 241 → 241 | 16309 → 16309 | -0.76% |
| `sparse_tfidf_512` | 1337 → 1337 | 119124 → 119124 | -1.11% |
| `tfidf_union_32` | 244 → 244 | 19834 → 19834 | +5.03% |
| `tfidf_union_33` | 296 → 296 | 27854 → 27854 | +3.37% |
| `tfidf_warm` | 71 → 71 | 19335 → 19335 | -0.52% |
| `union_32` | 244 → 244 | 19834 → 19834 | +3.27% |
| `union_33` | 296 → 296 | 27854 → 27854 | +0.37% |
| `unique_long` | 299 → 299 | 32672 → 32672 | +1.82% |
| `unique_search` | 72 → 72 | 19350 → 19350 | +0.10% |
| `unique_short` | 296 → 296 | 18548 → 18548 | +1.23% |

Negative percentages are lower observed latency; positive rows are slower and remain visible. Most of these paths are unchanged by this release. Binary layout, randomized hashing, caches, scheduling and small process-pair counts are not isolated; these are not significance tests or a universal speed guarantee. No isolated broad-query gain is presented as an end-to-end MCP gain.

## Actual MCP: first pass and repeated passes

Both complete campaigns are preplanned, using canonical 167,942-entry knowledge, the revision-bound NumPy snapshot and deterministic synthetic broad input. Five alternating old/new process pairs per input give thirty servers per campaign, sixty total. Synthetic requests use 32/33/64/128 terms within MCP limits. Each server executes every query/algorithm once, then two repeated passes. Every repeated full payload must match its initial one.

Across both campaigns **5,720 actual result/error responses validate against their schemas**, and **2,860 complete old/new structured pairs match by JSON-RPC request ID**, excluding only explicit status version and asserted release-qualified schema URNs. Discovery/source binding, initial exact snapshot readbacks, progress, concurrent queries, matching/relations, rejected arguments, health and EOF shutdown pass. Counts are 1,200 initial searches, 2,400 repeated searches, 240 concurrent requests, 1,640 readbacks, 120 matching/relationship calls, 60 rejected arguments and 60 health calls. Input hashes remain unchanged.

### First complete campaign

| MCP observation, ms unless stated | 0.4.11 | 0.4.12 |
| --- | ---: | ---: |
| Canonical startup median | 5380.60 | 5472.29 |
| Canonical first TF-IDF call median | 29.308 | 29.091 |
| Canonical initial pass TFIDF, median / p95 | 26.047 / 48.448 | 27.224 / 47.944 |
| Canonical repeated BM25, median / p95 | 7.180 / 13.483 | 6.883 / 12.773 |
| Canonical repeated TFIDF, median / p95 | 8.382 / 16.037 | 8.129 / 14.824 |
| Canonical whole-child peak RSS, median MiB | 1725.45 | 1724.53 |
| Snapshot startup median | 175.69 | 174.29 |
| Snapshot first TF-IDF call median | 2.707 | 2.061 |
| Snapshot initial pass TFIDF, median / p95 | 1.748 / 4.463 | 1.757 / 5.114 |
| Snapshot repeated BM25, median / p95 | 1.535 / 1.956 | 1.504 / 1.999 |
| Snapshot repeated TFIDF, median / p95 | 1.576 / 2.203 | 1.524 / 2.692 |
| Snapshot whole-child peak RSS, median MiB | 58.67 | 57.22 |
| Synthetic wide startup median | 4.47 | 4.55 |
| Synthetic wide first TF-IDF call median | 1.444 | 1.460 |
| Synthetic wide initial pass TFIDF, median / p95 | 1.464 / 1.659 | 1.451 / 1.602 |
| Synthetic wide repeated BM25, median / p95 | 1.416 / 1.729 | 1.409 / 1.641 |
| Synthetic wide repeated TFIDF, median / p95 | 1.435 / 1.624 | 1.425 / 1.558 |
| Synthetic wide whole-child peak RSS, median MiB | 8.05 | 7.98 |

### Second preplanned campaign

| MCP observation, ms unless stated | 0.4.11 | 0.4.12 |
| --- | ---: | ---: |
| Canonical startup median | 5184.16 | 5178.57 |
| Canonical first TF-IDF call median | 28.971 | 26.730 |
| Canonical initial pass TFIDF, median / p95 | 27.166 / 46.772 | 24.601 / 44.516 |
| Canonical repeated BM25, median / p95 | 6.459 / 11.736 | 6.309 / 11.748 |
| Canonical repeated TFIDF, median / p95 | 7.648 / 14.069 | 7.388 / 13.372 |
| Canonical whole-child peak RSS, median MiB | 1725.39 | 1724.28 |
| Snapshot startup median | 143.69 | 146.78 |
| Snapshot first TF-IDF call median | 1.849 | 1.806 |
| Snapshot initial pass TFIDF, median / p95 | 1.562 / 4.206 | 1.527 / 4.007 |
| Snapshot repeated BM25, median / p95 | 1.453 / 1.726 | 1.467 / 1.639 |
| Snapshot repeated TFIDF, median / p95 | 1.468 / 1.756 | 1.477 / 1.702 |
| Snapshot whole-child peak RSS, median MiB | 57.30 | 57.11 |
| Synthetic wide startup median | 4.39 | 4.41 |
| Synthetic wide first TF-IDF call median | 1.464 | 1.481 |
| Synthetic wide initial pass TFIDF, median / p95 | 1.453 / 1.543 | 1.467 / 1.547 |
| Synthetic wide repeated BM25, median / p95 | 1.439 / 1.797 | 1.436 / 1.594 |
| Synthetic wide repeated TFIDF, median / p95 | 1.451 / 1.573 | 1.452 / 1.604 |
| Synthetic wide whole-child peak RSS, median MiB | 7.92 | 7.97 |

Canonical repeated TF-IDF median changes are -3.02% and -3.40%. Canonical initial-pass TF-IDF medians change by +4.52% and -9.44%; cold transport latency therefore does not improve consistently. Eighteen of the 25 warm microfixture medians are slower, including TF-IDF union32 (+5.03%) and union33 (+3.37%). Whole canonical process RSS stays close to 1.7 GiB despite the smaller cache. First calls, initial aggregate requests and repeated requests remain distinct measurements. Small inputs, broad transport and whole-process RSS must be judged from their own rows, including slower results. Startup includes synchronous loading/indexing outside request deadlines. Search timing includes progress and stdio transport; initial snapshot readbacks occur outside that timer. Whole-child peak RSS includes parsing, buffers, index, hints and transport. These results do not establish a hard memory/time quota.

## Complete document responses and construction controls

The retained private corpus contains 72 Markdown files, 2,755,468 source bytes and 1,910 chunks. All 342 authored questions (276 positives, 66 unsupported) are exposed regressions. They are not a fresh holdout or logged user questions. Literal BM25, prepared BM25 and coverage retain six passages, 100 candidates, 2,000 bytes per passage and 12,000 total text bytes.

All **1,026 complete original response JSON strings match byte-for-byte**, including scores, IDs, order, terms, source text/spans and selection recipes. Repeated responses also match. Previous misses remain; no new semantic judgment/model reply is added. One warm-up and five samples per query/configuration/version give **10,260 timed round trips**, including IPC, source/evidence work, serialization and Python parsing. These three configurations use BM25; TF-IDF correctness is covered by independent score oracles and actual protocol comparisons.

| Document context, ms | 0.4.11 median / p95 | 0.4.12 median / p95 |
| --- | ---: | ---: |
| baseline | 0.833 / 1.570 | 0.842 / 1.505 |
| prepared | 0.937 / 1.637 | 0.931 / 1.640 |
| evidence | 0.928 / 1.632 | 0.939 / 1.671 |

Construction remains a control. Five alternating pairs for full canonical knowledge and three synthetic distributions give 40 plain processes; parsing/input generation is outside the construction timer. Six separate synthetic profiles check derived live heap. Stats/signatures and live derived bytes agree.

| Construction only, ms | 0.4.11 median / p95 | 0.4.12 median / p95 |
| --- | ---: | ---: |
| canonical | 5111.986 / 5306.101 | 5283.618 / 5384.225 |
| ascii | 14.857 / 15.309 | 15.261 / 15.375 |
| mixed | 34.456 / 36.386 | 34.625 / 34.900 |
| long_tokens | 1.654 / 1.706 | 1.639 / 1.729 |

A separate execution/shared-index worker validates all 167,942 entries through 128 changes/restoration, rebuilds, JSON round trips, four readers with forty queries each, retained clones and cancellation/deadline rejection. Source and portable export identities agree. This is finite consistency validation, not a new PDF parser or million-user capacity test.

## Conditions, limits and preservation

Measurements use one AC-powered Apple M5 Max with 64 GB, Rust 1.98.0 and Python 3.14.7. Managed compilers finish before timed stages. Stage-start host CPU/compiler snapshots are retained; they do not establish continuously idle hardware or attribute every difference. One corpus, authored/exposed queries, randomized hash seeds and small process counts limit generalization. Read [all sanitized records](measurements/0.4.12/README.md). Private raw sources/transcripts, profiles, locks, exact measured executables and logs remain preserved. Public aggregates do not reconstruct private documents; synthetic programs/generators are published.

Lexical scores and exact quotations do not establish truth or entailment. There is no new LLM, semantic-quality, token-saving or PDF extraction claim. The inherited 0.4.8 input plan is retained privately; the current 0.4.12 plan separately records this release and the fixed scope.

Canonical knowledge remains **167,942 entries from 35 PDFs and 49,086 physical pages**, SHA-256 `13168d87872a137355fd8499c90a6084f52243a2ec7456f29b5fcb2a1c7423c3`. PDFs, knowledge, snapshots and every measurement remain. After verified publication, archive release proof, locks, package and small measured programs before deleting only the named temporary Cargo/Miri targets. No model daemon is started, defaults remain empty and no DenkMaps integration is performed.
