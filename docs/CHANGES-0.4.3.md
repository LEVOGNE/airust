# AIRust 0.4.3: Faster construction and BM25, with retained memory sharing

Version 0.4.2 reduced retained index memory and accelerated repeated TF-IDF queries, but its measured large-index construction and BM25 costs increased. AIRust 0.4.3 targets those two costs. It keeps shared immutable strings, valid TF-IDF norm hints, public methods, source text and versioned knowledge/cache formats. Optional MCP remains in the same crate with its existing response schema version one. No DenkMaps integration is included.

## What changed and why

The first 1,024 borrowed tokens of an entry are sorted and counted together. The remaining tokens continue through a streaming counter. This reduces repeated per-word tree searches without buffering an entire long entry. The buffer holds at most 1,024 string references; tree/collection construction uses additional bounded temporary allocations. Original text and Unicode normalization rules remain unchanged.

The global vocabulary uses the Rust standard library's randomly seeded `HashMap`, preserving shared `Arc<str>` keys. No custom unprotected fast hasher is installed. The dictionary is used for lookups; ordered document IDs, postings IDs, per-entry terms, duplicate groups, score contributions and tie ordering still determine output. Random seeds do not define cache bytes or ranking. See [Rust's map contract](https://doc.rust-lang.org/std/collections/struct.HashMap.html).

BM25 corpus-frequency logarithms are prepared once per known query word. Document normalization is calculated once per candidate. Score multiplication/division and sorted addition order remain the same, including the existing large-`k1` branch. Unknown/repeated words retain existing behavior. A compact heap keeps borrowed candidates and scores, then creates matched-term strings only for selected winners. Asking for all results still pays for all returned explanations.

TF-IDF uses the same prepared terms and winner selection while retaining its 0.4.2 norm policy: lazy completed norms, lexical-statistics invalidation, metadata-only reuse, clone isolation and nonblocking exact fallback on busy/poisoned hints. Controlled scoring and explanation loops add cancellation/deadline checkpoints within long term sequences. No new public option, required feature, dependency or persistence migration is introduced. See [core/API behavior](CORE-PERFORMANCE.md) and the [API cookbook](API-COOKBOOK.md).

## Resource defect found and fixed

The first hash-dictionary prototype kept a large empty bucket allocation after deleting its final entry. A targeted regression fails before the fix. The released implementation tracks dictionary capacity independently of deleted-slot bookkeeping, releases the allocation completely when empty, and compacts a large dictionary after its word count falls below a quarter of the retained capacity. Small tables keep bounded headroom. Retained cloned readers keep their original contents and storage. Two regressions cover empty release and sparse dictionaries. Compaction can add an occasional rehash cost and temporarily hold old/replacement tables during deletion; it is not part of ordinary lookup latency.

## Executed tests and compatibility

The full local script passes **217 default tests, 800 all-feature checks including 36 documentation examples, 39 isolated declared feature entries and five Python tests**. Strict Clippy and documentation builds pass. Rust 1.89 covers provider-neutral features including MCP; Rust 1.92 covers all native ONNX adapters. Patch-level public API comparison against 0.4.2 passes **229 checks with 31 tool skips**. A separate Rust 1.89 public API program runs successfully.

Three new tests use an independent full-scan BM25 oracle for **840 parameter/scope/limit/query combinations**. They include extremely small/large valid `k1`, `b` endpoints, unknown/repeated terms, empty queries, zero/large limits, ID/metadata scopes and exact score bits. Further cases cross the 1,024-token boundary, continue through 12,000-token Unicode streams, replace/remove records and verify ties, complete explanations and JSON preservation. Existing mutation, sharing, parallel-reader, cache, cancellation, deadline, progress and MCP attack-oriented suites remain passing. No functional divergence from 0.4.2 is observed in these checks.

An external program compares 0.4.3 against actual registry 0.4.2 on prefixes of **1,000, 10,000 and all 167,942 retained entries**. Each performs 840 comparisons of ordered IDs, floating-point score bits and matched terms: initial/repeated queries, 49 mutations, retained readers and bidirectional public cache loading. All **2,520 comparisons** agree. Cache analysis/metadata remains identical after the expected version-specific producer label; whole-file hashes differ because of that label and header checksum.

Matching checks at 1,000/10,000 entries each compare 72 positive ordinary/controlled results and 12 relation cases against 0.4.2. A retained legacy corpus consumer executes a checked 1,000-entry epoch after full-file parsing. It covers incremental/rebuild equality, serialization, concurrent signatures, retained readers, cancellation and deadlines. This is not a fresh million-record or semantic-relevance evaluation.

## Actual MCP and complete retrieval objects

The exact measured executable serves all **35 retained document snapshots** in separate stdio connections plus the complete canonical collection. Fixed probes exercise BM25/TF-IDF and four concurrent requests per connection. Complete retrieval objects match a separate direct Rust API oracle. The 36 direct-oracle files also agree completely with the retained 0.4.2 oracles.

This is **424 searches, 624 exact snapshot passage readbacks and 72 matching/relation calls**. Byte spans, source bindings, revisions, progress tokens/sequences, denied fields and unchanged input hashes are checked. An independent executable with no AIRust dependency validates the five advertised schemas against their metaschema and checks **all 1,192 actual structured replies**, including errors, text-copy equality and `isError` agreement. No remote schema references are resolved. Existing argument/output/worker/transport protections remain; see [MCP setup](MCP.md) and [security boundaries](MCP-SECURITY.md).

## Ordinary core timings

Twelve fresh Rust 1.89 release children cover two prefix sizes, two versions and three repeats, alternating old/new order. Both versions enable `cache`, `execution` and `shared-index`. Build timing starts after full JSON parsing and owned-record preparation. Seventy fixed literal probes run three passes per process. The first pass is partly warmed by overlapping candidate sets; repeated-pass medians below contain 420 samples per algorithm/version/prefix. All timed queries return hits, but a smaller-prefix hit need not be the intended source document. These are performance probes rather than human relevance labels.

All paired values are **0.4.2 / 0.4.3**.

| Indexed entries | Build median ms | Repeated BM25 median ms | Repeated TF-IDF median ms |
| --- | ---: | ---: | ---: |
| 10,000 | 362.1 / 259.2 | 0.637 / 0.434 | 0.620 / 0.440 |
| 167,942 | 7366.6 / 5194.0 | 17.634 / 12.402 | 19.052 / 13.703 |

For the complete collection, construction takes **29.5% less time** and the repeated BM25 median is **29.7% lower** in this workload. TF-IDF reuse remains available. First-pass values, per-query hits/times, maxima and whole-child RSS are retained separately. Whole-child RSS includes full-file decoding before prefix selection; it is not isolated index heap. Three warm repeats with uncontrolled desktop activity do not establish cold-query, p95/p99 or universal speed/capacity guarantees.

## Retained index memory and allocation traffic

A separate external-only Rust 1.89 DHAT program profiles both default indices without `shared-index`. It preloads source outside profiling, then clones owned records under the profiler. One trace per version/prefix records construction, ten required-positive TF-IDF probes over two passes, a default clone, drop-clone and drop-all. All tracked index allocations return to zero after release. Instrumented timings are not latency measurements.

| Entries | Live index MiB, old/new | Less live heap | Build peak MiB, old/new | Build allocations, old/new | Cumulative allocated MiB, old/new |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1,000 | 9.80 / 9.25 | 5.67% | 9.88 / 9.32 | 42,738 / 51,656 | 10.65 / 19.43 |
| 10,000 | 87.28 / 82.11 | 5.92% | 88.04 / 82.88 | 356,279 / 451,911 | 96.60 / 189.19 |

The 10,000-entry live index uses **5.92% less heap than 0.4.2**, preserving and extending that release's retained-memory gain. Its construction peak also falls. The new grouping/hash construction creates **more short-lived allocations and cumulative allocation traffic**; those costs are shown explicitly rather than treating every memory metric as equivalent. They are bounded working allocations and are freed, while the completed index is smaller. The ordinary build measurements include their CPU cost. Preloaded originals, profiler overhead, OS RSS and other application processes are excluded from the heap figures.

The initial sorted-tree prototype and hash-dictionary experiments remain retained privately. Their timings are excluded from the final three-repeat results. Small entries, long unusual words, vocabulary shape, hash seeds, query width, requested limit and frequent updates can change the trade-offs.

## MCP startup and round-trip costs

Thirty fresh release servers compare five cases with three repeats per version. Both server binaries use Rust 1.98, thin LTO and one codegen unit. No deliberate competing builds/checks run during these measurements. Files are warm and unrelated desktop activity is uncontrolled. Search includes request/progress/result transport and excludes later passage readbacks. No transcript/general-schema-validation work runs inside timers. Startup-to-initialized ends before `tools/list`.

| Case | Startup median ms, old/new | BM25 median ms, old/new | TF-IDF median ms, old/new | Server peak RSS MiB, old/new |
| --- | ---: | ---: | ---: | ---: |
| canonical | 7271.9 / 5308.0 | 19.06 / 13.73 | 31.62 / 19.33 | 1837.7 / 1730.5 |
| vkspec | 458.6 / 382.8 | 4.85 / 4.07 | 21.35 / 8.72 | 137.9 / 132.1 |
| scipy | 315.8 / 247.2 | 5.80 / 4.46 | 22.47 / 8.47 | 101.3 / 97.0 |
| racket-reference | 115.0 / 91.4 | 2.21 / 1.98 | 6.12 / 3.17 | 41.6 / 39.5 |
| pdf-12 | 50.1 / 40.5 | 1.66 / 1.63 | 3.53 / 2.13 | 21.7 / 21.2 |

Canonical medians have 210 sequential samples per algorithm/version; each snapshot case has six. Snapshot indexing uses nonblank whole pages while canonical entries are older chunks. RSS is the single server child's lifetime peak including startup/index/snapshots/runtime/transport, excluding Python's expected results. Four concurrent calls have separate elapsed fields. The configured dataset accounting remains a payload limit, not a process-RAM quota. See [raw source-free samples](measurements/0.4.3/).

## Retention, scope and publication

All eight canonical exports, all 35 original PDF hashes and all 35 0.4.0 snapshots remain unchanged. The corpus stays **167,942 entries from 35 PDFs / 49,086 physical pages**, with canonical SHA-256 `13168d87872a137355fd8499c90a6084f52243a2ec7456f29b5fcb2a1c7423c3`. The snapshots retain 48,983 supplied text pages. Knowledge, failed setup attempts, prototypes, raw traces, binaries and scripts are kept in the ignored private lab. Public measurements contain counts/hashes/timings without private document or query text.

PDF backend **0.3.5** remains source-identical, with 14 fresh regressions and the 381-file provenance check. Fresh dependency audits find zero known vulnerabilities; three main and four backend maintenance warnings remain documented. Cargo.lock changes only the AIRust version. This release does not re-extract PDFs, add OCR/models, integrate DenkMaps, run Docker or test a third-party MCP client UI. It is not an independent security audit.

Publication requires a clean package, extracted all-feature/Python checks, dry-run and successful Linux CI for the exact commit. Actual registry API/MCP/schema/corpus consumers and checksum/VCS/tag/three-asset verification are recorded in the release verification artifact. The final documentation-only rebuild must match the measured MCP executable hash. Original formats, source positions, client authorization responsibilities and cooperative resource limits retain their documented boundaries.

[Reproduction programs](validation-0.4.3/README.md) · [Core API](CORE-PERFORMANCE.md) · [All measurements](measurements/0.4.3/)
