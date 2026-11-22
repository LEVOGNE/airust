# AIRust 0.4.4: fewer temporary allocations and reliable connection shutdown

AIRust needs less throwaway storage while building its knowledge index. Applications keep the same records and public methods. The completed index keeps its previous memory layout. A separate MCP defect found during review is fixed: closing one connection no longer waits for another connection's work. MCP stays optional in the same crate. No DenkMaps integration is included.

## What changed

A private TokenBatch holds at most 1,024 borrowed word/count pairs. It sorts them and combines repeated counts in place. The original number of tokens is saved before combining. The completed pairs are consumed into the existing ordered term map. The rest of a long entry is still processed as a stream. Empty token input reserves no buffer. A single very long word reserves one slot.

The compiler can reuse the consumed pair storage during collection. This is measured standard-library behavior, not an API guarantee or an unsafe memory conversion. No working buffer is retained between documents or inside completed index copies. Tree nodes, normalization, original records and other working data still allocate.

Engine is an alias for KnowledgeIndex. This remains the single owner of lexical state. Existing prepared insertion, shared vocabulary/IDs, normalized duplicates, lazy TF-IDF hints, clone isolation and sparse-vocabulary compaction remain. Cache restoration and advanced analyzers keep their existing path into that state. There is no second index or global mutable scratch pool. See [architecture and behavior](CORE-PERFORMANCE.md).

## A reproduced MCP defect and its fix

Previously, connection shutdown tried to acquire **all global worker permits**. Server clones share those permits. Work in connection B could therefore make healthy connection A fail its one-second shutdown grace. A waiting shutdown could also interfere with admission in other connections.

A deterministic regression holds an unrelated clone's permit throughout shutdown. It fails on the old code with the grace-period error and passes after the fix. A separate wire test closes A while B runs an actual large fuzzy comparison, verifies B still owns its slot, then cancels B and verifies cleanup.

Each connection now owns an existing tokio-util TaskTracker. It tracks the complete blocking closure, including work that outlives a cancelled async handler. Shutdown waits only for that connection's workers. The shared semaphore remains admission control. Global in-flight statistics, the cooperative cancellation boundary and the one-second grace remain. Admission and shutdown are synchronized: an owned worker lease registers its tracking token while acquiring the permit, before initial progress delivery. It moves into the blocking job. Closing rejects new reservations, while abandoned startup and panic paths release their leases. This prevents a late registration from escaping the shutdown wait. Four regressions cover those lifetime paths.

The five tool definitions are prepared once from each server data instance's immutable limits. Clones share that catalogue; calls select the requested definition without rebuilding all five. Returned definition/schema edits remain isolated, including across instances with different limits. This keeps a small fixed catalogue alive per shared data instance.

Missed progress ticks are skipped instead of replayed in a burst. The SDK rejects a probed JSON-RPC batch array with invalid-request error -32600; the next individual request succeeds. The suspected batch bypass was **not** reproduced. EOF cancellation is now explicit in the guide: keep stdin open until the responses you need have arrived. [MCP API](MCP.md) and [security boundaries](MCP-SECURITY.md) explain the exact contract.

Claude reviewed checksum-verified **public 0.4.3 source** with tools disabled. Local tests confirmed the shutdown finding and rejected the batch hypothesis. Unpublished source and private documents were not included. This review is a useful second reading, not an independent security audit.

## Executed checks

The complete local script passes **221 default tests, 811 all-feature checks including 36 documentation examples, 39 isolated declared feature entries and five Python tests**. Strict Clippy and documentation builds pass. The final changed MCP path also passes the unit/wire/executable/controlled-matching selection. Rust 1.89 checks provider-neutral features including MCP, and Rust 1.92 checks native ONNX adapters. Public API comparison against registry/tagged 0.4.3 passes 229 checks with 31 tool skips. A separate public API consumer actually runs.

Four new token-batch regressions cover zero/single-token storage, sorted Unicode frequencies, pre-dedup totals, the 1,024-token boundary, full streaming-tail preservation, vocabulary Arc reuse and retained readers. Existing independent BM25 tests cover 840 parameter/scope/limit/query combinations, including extreme valid parameters and score-bit equality. Further cases continue through 12,000-token streams, mutations, ties and original JSON preservation.

An external program compares registry 0.4.3 and current 0.4.4 on 1,000, 10,000 and all 167,942 retained entries. All **2,520 ordered-ID/score-bit/explanation comparisons** agree before and after 49 mutations per prefix, on retained readers and through bidirectional public cache loading. Cache analysis/metadata payloads agree after the version-specific producer label. Whole-file checksums differ because that label changes. Matching/relations remain equal on both tested prefixes.

## Miri and longer stress

Miri completes **61 targeted test executions** plus **80 additional executions** over several seeds, including explicit Tree Borrows checks. It covers token ownership, sharing, core ranking, norm-hint locks, poison recovery and independent score calculations. No memory/aliasing/race diagnostic occurs in those passing runs.

An initial large Tree Borrows resource attempt printed eight successful test lines, but its outer Cargo watchdog expired before a verified interpreter exit. That attempt is preserved and excluded from completed campaign counts. The complete unchanged eight-test group was repeated with a three-hour watchdog covering the entire process group; the counted rerun has a verified normal exit. No fixture was shortened and no memory, alias, race or leak check was disabled.

Exact numeric comparisons use `-Zmiri-no-extra-rounding-error`. A retained seed-17 reproducer differs by about 3e-16 with Miri's extra rounding injection and passes when only that injection is disabled. AIRust scoring and Miri memory/race checks are unchanged. Native PDF/ONNX and asynchronous MCP are outside this interpreter campaign. These finite executions do not prove soundness or all possible schedules. See [Miri's documented behavior](https://github.com/rust-lang/miri#readme).

Four fixed-seed cases use a **90-minute work budget each**, six hours in total. Each epoch mixes updates, deletes, metadata changes, rejected invalid replacements and long/Unicode/punctuation-only records. It compares complete ranking signatures against 0.4.3 and a fresh rebuild, checks retained readers and four parallel readers, and verifies cancellation/deadlines. Periodic cross-version cache loading and deliberately truncated caches check recovery without consuming original inputs.

| Mode | Initial records | Seed | Epochs | Mutation operations | Exact ranking comparisons |
| --- | ---: | ---: | ---: | ---: | ---: |
| shared | 4,000 | 202611210441 | 21,793 | 2,789,504 | 1,743,440 |
| default | 4,000 | 202611210442 | 23,910 | 3,060,480 | 1,912,800 |
| shared | 16,000 | 202611210443 | 6,687 | 855,936 | 534,960 |
| default | 16,000 | 202611210444 | 5,813 | 744,064 | 465,040 |

Totals are **7,449,984 mutation operations**, **4,656,240 exact ranking comparisons**, **989,451 concurrent-reader checks** and **230 cache cross-load pairs**. Final mutated test copies and caches remain in the private lab. Original knowledge is unchanged. The mutation counter includes rejected and no-op attempts. These are fixed-seed consistency checks, not human relevance judgments or a race-freedom proof.

The original long-run reporter reused the final epoch's progress timestamp. One row therefore records 5,399.9989 seconds: its progress-file write crosses the final budget check. Those raw values are preserved. The loop uses a monotonic 5,400-second budget, and separately retained outer timestamps span at least that long for every case, including setup and final save. The public reproduction reporter now captures loop-completion time before final serialization; an authored short smoke run checks the correction. Library code and correctness counters are unchanged.

Fresh release processes also pass the existing one-million/two-million short-record scale fixture and a 500,000-entry fixture with three retained clones in both default/shared modes. Four concurrent readers and update/delete checks run in scale mode; clone mode verifies isolated mutation. The six processes run under deliberate concurrent test load. Their times/RSS are published separately and are not the quiet latency baseline or a claim that arbitrary two-million-record corpora fit the same RAM.

## Quiet ordinary-call measurements

All paired table values below are **0.4.3 / 0.4.4**. Twelve fresh Rust 1.89 release children cover two sizes and three repeats, alternating order. Both versions enable cache, execution and shared-index. Construction excludes full JSON decoding and owned-record preparation. Seventy fixed literal queries run three passes; repeated-pass medians contain 420 samples per algorithm/version/size. First-pass queries can warm overlapping candidates. These probes measure work, not independent relevance.

| Records | Build median ms | Repeated BM25 median ms | Repeated TF-IDF median ms |
| --- | ---: | ---: | ---: |
| 10,000 | 291.8 / 294.5 | 0.459 / 0.471 | 0.467 / 0.482 |
| 167,942 | 5545.5 / 5568.0 | 14.026 / 12.990 | 15.352 / 13.981 |

Query scoring is unchanged by the allocation work. Warm files, random hash seeds, desktop activity and only three repeats limit interpretation. No deliberate competing builds/checks run during the quiet samples. All first-pass samples, per-query times/hits, maxima and whole-child RSS remain available. Whole-child RSS includes source parsing and is not isolated index heap. No p95/p99 or universal speed guarantee follows.

## Allocation traffic and retained memory

A separate Rust 1.89 DHAT consumer profiles default indices without shared-index. It preloads source outside profiling, then clones owned records under the profiler. One trace per version/size records construction, ten required-positive TF-IDF probes over two passes, a default clone and complete release. Instrumentation timings are not performance measurements.

| Records | Live index bytes, old/new | Build peak bytes, old/new | DHAT build allocation count, old/new | Cumulative build bytes, old/new |
| --- | ---: | ---: | ---: | ---: |
| 1,000 | 9,694,935 / 9,694,935 | 9,776,726 / 9,777,750 | 51,656 / 41,042 | 20,375,898 / 16,400,154 |
| 10,000 | 86,102,837 / 86,102,837 | 86,903,698 / 86,904,722 | 451,911 / 344,054 | 198,377,266 / 150,976,970 |

At 10,000 records, allocation count falls by **23.87%** and cumulative allocated bytes by **23.89%**. Completed live bytes and live allocation counts are exactly equal to 0.4.3 in both traces. Build peak is 1 KiB higher; it is not claimed as a peak-memory reduction. All tracked allocations return to zero after the index and retained copy are released. Source preloading, profiler overhead and OS RSS are outside this boundary. Earlier prototypes remain retained and are excluded from final comparisons.

## Authored allocation edge distributions

The same final heap consumer also runs eight authored 64-entry collections, one trace per version/case. Every positive TF-IDF probe returns hits, completed live byte/allocation counts are equal, and all tracked heap is released. The table shows **current minus previous** construction costs; positive byte deltas are increases.

| Distribution | Allocation-count delta | Cumulative-byte delta | Peak-byte delta |
| --- | ---: | ---: | ---: |
| long-single | -64 | -8,704 | +0 |
| long-two | -64 | +1,562,624 | +8,168 |
| long-unicode-two | -64 | +1,562,624 | +0 |
| mixed-long-prefix | -832 | +272,384 | +14,336 |
| repeated-short | -576 | -526,336 | +0 |
| short-two | -64 | -4,096 | -64 |
| unique-prefix | -1,088 | -3,659,776 | -16,384 |
| unique-tail | -1,088 | -3,659,776 | +0 |

Few very long words expose a limit of the byte-based reservation estimate: it can reserve more token/count slots than needed. Both long-two-word cases request 1,562,624 additional cumulative bytes across 64 entries despite 64 fewer allocation calls. A mixed long prefix also uses more cumulative bytes. The buffer remains bounded to 1,024 pairs, and these traces show no additional completed index storage. Fewer allocation calls do not imply fewer allocated bytes or a lower peak for every input. The real-document improvement above is specific to its measured collection. [Fixtures and reproduction](validation-0.4.4/README.md) are public; instrumented times are not latency measurements.

## Same-process connection lifetimes and MCP allocations

A separate Rust 1.89 DHAT probe reuses one 1,000-record synthetic server across 1,000 connections. Each connection initializes, searches with progress, requests status and closes. Serial runs compare registry 0.4.3 and current 0.4.4. The measured window excludes instrumented warmup counters. Allocation count falls by **24.75%** and cumulative allocated bytes by **20.17%** in that window. This measures definition reuse and worker-lifetime changes together; it does not isolate one instruction's benefit.

A separate current run completes all 1,000 connections with four concurrent connections and checks zero occupied worker slots after every batch. The first quiescent snapshot at 100 connections has zero tracked live bytes; subsequent snapshots at 200 through 1,000 stay at 64 bytes in this run. The profiler excludes preloaded index/runtime/tool catalogue and uses preallocated numeric telemetry. The fixed cached catalogue adds startup storage outside this scope. This is not a whole-process RAM or leak-freedom claim, and instrumented timings are not latency benchmarks.

The old parallel run reproduces a false busy response during another connection's shutdown. Its failed partial totals are retained and excluded from allocation comparisons. The deterministic red/green unit regression remains the non-random proof of the defect. Raw traces/prototypes stay private; the [connection probe](validation-0.4.4/README.md) is public.

## Actual MCP objects, schemas and costs

The exact measured executable serves all 35 retained document snapshots plus the full canonical collection in separate stdio connections. Complete retrieval objects match separate direct Rust API oracles and all 36 old 0.4.3 oracle files. This covers **424 searches, 624 exact passage readbacks and 72 matching/relation calls**. Source/revision/span checks, progress tokens/sequences, concurrent requests, denied fields and unchanged source hashes are verified.

An independent executable with no AIRust dependency validates the five advertised schemas and all **1,192 actual structured replies**, including error shapes, text-copy equality and isError agreement. No remote schema references are resolved. Schema shape does not establish source truth, authorization or relevance.

Thirty fresh release servers compare five cases with three repeats per version. Both measured binaries use Rust 1.98, thin LTO and one codegen unit. Search includes local request/progress/result transport and excludes passage follow-up. Startup ends at initialized, before tools/list. Transcript/general-schema validation is outside timers. No deliberate competing test/build work runs during samples.

| Case | Startup median ms | BM25 median ms | TF-IDF median ms | Server peak RSS MiB |
| --- | ---: | ---: | ---: | ---: |
| canonical | 5268.5 / 5240.5 | 14.45 / 17.50 | 19.29 / 24.43 | 1730.7 / 1730.7 |
| vkspec | 376.7 / 379.7 | 3.99 / 4.00 | 9.45 / 9.43 | 132.1 / 132.2 |
| scipy | 243.0 / 242.9 | 4.36 / 4.34 | 9.04 / 9.03 | 97.1 / 96.9 |
| racket-reference | 89.3 / 90.4 | 2.05 / 2.12 | 3.01 / 3.18 | 39.3 / 39.5 |
| pdf-12 | 40.2 / 40.6 | 1.66 / 1.64 | 2.21 / 2.16 | 21.2 / 21.1 |

Canonical samples include 210 sequential queries per algorithm/version; each snapshot has six. Canonical records are older chunks, while snapshots index nonblank whole pages. RSS covers the single child server's lifetime including startup/index/snapshot/runtime/transport, excluding the Python validator. Four concurrent requests have separate elapsed fields. Files are warm and unrelated desktop activity remains uncontrolled.

## Canonical MCP latency follow-up

The original first current canonical process is an outlier: startup takes 7,925.6 ms and its BM25/TF-IDF process medians are 26.74/38.54 ms. The next two current processes have medians 13.75/18.43 and 13.94/18.37 ms. All original samples, including that first process, remain in the table and raw artifacts above.

One fixed follow-up uses **three additional old/current pairs**, reversing the starting order and keeping the same binaries, source, queries and direct oracle. It performs no build or intentional concurrent check. The table below summarizes the median of the three per-process medians; this differs from pooling all heterogeneous query timings.

| Algorithm | Original process medians, old/new ms | Follow-up process medians, old/new ms |
| --- | ---: | ---: |
| BM25 | 14.45 / 13.94 | 14.30 / 13.84 |
| TFIDF | 19.20 / 18.43 | 19.69 / 19.87 |

The follow-up does not reproduce the large pooled slowdown as a consistent per-process shift. TF-IDF process medians are close, and the small BM25 difference remains subject to desktop noise and three repeats. No universal MCP latency improvement is claimed. The [complete fixed follow-up](measurements/0.4.4/mcp-canonical-followup.json) includes every process and both summaries; no run is removed or repeated until a preferred outcome appears.

## Retention, dependencies and release scope

All eight canonical exports, all 35 original PDF hashes and all 35 existing revisioned snapshots remain unchanged. The collection stays **167,942 entries from 35 PDFs / 49,086 physical pages**, with SHA-256 `13168d87872a137355fd8499c90a6084f52243a2ec7456f29b5fcb2a1c7423c3`. Snapshots retain 48,983 supplied text pages. Original documents, knowledge, raw private replies, experiments, failed attempts, binaries and traces remain available. Public artifacts contain source-free counts/hashes/times.

PDF backend 0.3.5 is source-identical, with 14 fresh regression checks and 381-file provenance verification. Dependency audits find zero known vulnerabilities; three main/four backend maintenance warnings remain documented. Dependency versions do not change. Optional MCP explicitly requires the already-resolved tokio-util0.7.20 and enables rt for task tracking; Cargo.lock adds its futures-util edge. The default library remains feature-free and model/server/database independent.

No PDF re-extraction, OCR, new model, Docker run, live third-party client UI, application database or DenkMaps integration is performed. This is not an independent penetration audit or a guarantee of defect-free behavior. Client authorization, source authenticity, prompt handling and hard OS resource limits remain host responsibilities.

Publication is gated on the clean package, extracted all-feature/Python checks, dry-run and Linux CI for the exact commit. Registry-only Rust 1.89 API/MCP/schema/corpus consumers plus checksum/VCS/tag/three-asset verification are retained in the release verification artifact. The final documentation rebuild must match the measured MCP executable hash.

[Reproduction](validation-0.4.4/README.md) · [API cookbook](API-COOKBOOK.md) · [All measurements](measurements/0.4.4/)
