# AIRust 0.4.2: Leaner core and checked MCP responses

AIRust stores less repeated index text and reuses completed TF-IDF calculations. Every optional MCP tool now advertises a response schema. Existing public core signatures, original text, deterministic ranking and saved knowledge formats remain compatible with 0.4.1. There is no DenkMaps integration. MCP stays optional in this same crate; the default library remains feature-free.

## Core changes and safeguards

Immutable vocabulary words, document-map/posting/duplicate-group identifiers and identical normalized text share allocations. Normal insertion counts borrowed tokens directly into the interned term map instead of constructing an intermediate owned-string map. Public entries still own their original ID/text/metadata. The default clone copies containers while sharing immutable strings; `shared-index` additionally retains its existing sharing of complete document/posting blocks.

TF-IDF lazily retains complete document norms for visited candidates. Repeated queries reuse them while corpus statistics match. Lexical edits, insertions, successful removals and clear invalidate the changed index's hints. Metadata-only updates preserve them. Retained clones stay correct when another clone changes. BM25 creates no norm hints.

Testing the new optimization covers hazards that would otherwise make a cache unsafe: stale document frequencies after an edit, divergence between retained readers, a poisoned lock, lock contention and interruption partway through a norm. Hints use nonblocking lock attempts and exact recomputation as fallback. Checkpoints also occur inside the cold term loop, and partial norms are never published. These are safeguards for the new derived state, not claims that 0.4.1 contained this cache. See [full API behavior and costs](CORE-PERFORMANCE.md).

## MCP response contracts

The five tools advertise self-contained JSON Schema 2020-12 `outputSchema` objects. One bundled source defines typed nested search evidence, excerpts/positions, document bindings, matching, relation and status fields. Each tool accepts its own success envelope or the shared structured-error envelope. Payload `schema_version` remains **1**. Clients validate `result.structuredContent` and separately check `isError`; JSON-RPC errors and cancelled requests follow their own lifecycle.

Schemas are cached with only their required local definitions. No general validator or reference-fetching service is added to runtime dependencies. The independent test validator has HTTP/file resolver features disabled. Schema validity does not establish source truth, authorization or cross-field byte/hash correctness; runtime binding checks and client trust rules remain essential. Registered snapshots are fully readable even if only selected passages are indexed. See [installation, Rust API, schemas and errors](MCP.md), [security](MCP-SECURITY.md) and [API cookbook](API-COOKBOOK.md).

## Executed compatibility and failure checks

The full local script passes **212 default tests, 795 all-feature checks including 36 documentation examples, 39 isolated declared feature entries and five Python tests**, with strict Clippy/documentation checks. Four new core integration tests and six internal optimization regressions cover exact scores, updates, clones, parallel first/warm queries, allocation sharing, no-cache paths and the interruption/lock cases. Seventeen MCP protocol tests validate actual structured successes/errors and matching textual JSON, retain all earlier attack-oriented cases, and reject malformed nested/cross-tool/schema data. Four executable tests and three controlled matching integrations remain.

Rust 1.89 checks provider-neutral features including MCP and the dev validator; Rust 1.92 checks all optional native ONNX adapters. The independent public API consumer runs on Rust 1.89. Patch-level API comparison against 0.4.1 passes **229 checks with 31 tool skips**.

An external old/new core program parses the complete retained collection and indexes prefixes of **1,000, 10,000 and all 167,942 entries**. Each performs 840 BM25/TF-IDF comparisons: ordered IDs, exact floating-point score bits and matched terms before/after 49 mutations, with retained readers and caches cross-loaded in both directions. All **2,520 comparisons** agree. Cache metadata/analysis agrees after the expected version-specific producer label; whole-file checksums differ because of that label and the header checksum. No cache format migration is needed, and private norm hints are not serialized.

Matching comparisons at 1,000/10,000 entries each check 72 full ordinary/controlled results and 12 relation cases against 0.4.1. Every matching probe returns hits. Exact/token probes use whole entries; fuzzy probes edit available 24..128-character entries. The smaller prefix has one short source reused with distinct edits; the larger has twelve. The retained legacy corpus consumer also executes one checked 1,000-entry epoch with rebuild/update/concurrent/retained-reader/cancellation/deadline checks. This is not a fresh million-record stress or human relevance campaign.

## Real MCP and independent schema checks

The exact measured executable serves all **35 retained document snapshots** in separate stdio connections, plus the complete canonical collection. Fixed literal probes use both algorithms, then each connection submits four concurrent searches. Complete retrieval objects equal a separate direct Rust API oracle, including ranking evidence, excerpts, assessment and budgets. Snapshot excerpts are checked against exact UTF-8 bytes and read back through MCP.

The final campaign performs **424 searches, 624 exact passage readbacks and 72 matching/relation calls**, with progress/token checks, read-only status, denied fields and unchanged input hashes. A separate executable with **no AIRust dependency** validates the advertised schemas against their metaschema and checks **all 1,192 captured structured results**, including error replies and textual-copy equality. It does not retrieve remote references.

The corpus remains **167,942 entries from 35 PDFs / 49,086 physical pages**. Existing 0.4.0 snapshots cover 48,983 supplied text pages. This release restores their retained text; it does not claim fresh PDF extraction, OCR, independent glyph coverage or semantic relevance labels.

## Allocator-instrumented memory

A separate Rust 1.89 DHAT program measures owned index records, with source preloading outside profiling. Both engines use their default features, without `shared-index`. It records build, ten positive TF-IDF probes over two passes, default clone, drop-clone and drop-all phases. Each version/prefix has one deterministic allocation trace. Every positive probe must return hits; all tracked index heap is released at the end. Preloaded source, profiler overhead and OS memory are excluded. Instrumented execution times are not latency measurements.

| Entries | 0.4.1 live index MiB | 0.4.2 live index MiB | Less live heap | Fewer build allocations |
| --- | ---: | ---: | ---: | ---: |
| 1,000 | 11.75 | 9.80 | 16.57% | 79.51% |
| 10,000 | 106.86 | 87.28 | 18.32% | 83.44% |

The 10,000-entry case also has roughly 66.9% fewer live allocation blocks after construction. Norm hints add a small amount of memory when TF-IDF is used; complete phase values remain in [index-heap.json](measurements/0.4.2/index-heap.json). Early no-hit probe attempts and the incorrect initial whole-cache-file assertion were harness issues; they remain retained and are excluded from accepted measurements. The corrected cache oracle cross-loads both versions and accounts explicitly for producer metadata.

## Ordinary core costs

Twelve fresh Rust 1.89 release children compare both versions with the same `cache`, `execution` and `shared-index` features, 70 fixed probes and three query passes. Three repeats alternate old/new order. Build timers exclude full-file parsing and owned-record preparation; query timers exclude result serialization. The first pass is partly warmed across overlapping candidates. The table uses the two repeated passes: 420 samples per algorithm/version/prefix. Every timed query returns at least one hit in both versions/prefixes. A hit from a smaller prefix need not be the intended original document; these are lexical performance probes rather than relevance labels. Positive counts/medians and first-pass values are retained separately.

All paired values below are **0.4.1 / 0.4.2**.

| Indexed entries | Build median ms | Repeated BM25 median ms | Repeated TF-IDF median ms |
| --- | ---: | ---: | ---: |
| 10,000 | 419.3 / 404.5 | 0.7941 / 0.7812 | 5.0336 / 0.7182 |
| 167,942 | 7080.0 / 8150.1 | 19.5382 / 20.5764 | 167.5044 / 22.8107 |

In the full collection, repeated TF-IDF median time falls from **167.50 to 22.81 ms** (about **7.34 times faster**). Index construction increases from **7.08 to 8.15 s** (about **15.1% longer**), and repeated BM25 rises from **19.54 to 20.58 ms** (about **5.3%**). These measured costs accompany the lower memory footprint and TF-IDF reuse; applications rebuilding frequently or using only BM25 should account for them. The smaller-prefix build improves slightly, so the effect is workload-dependent.

This tests a stable retained index. Frequent lexical changes invalidate norms and reduce reuse. Busy hint locks may repeat calculations. These results do not promise a universal speedup, isolated cold-query latency or a fixed RAM ceiling. Process RSS includes full-file parsing before prefix selection; it is not the isolated heap measurement above. See [complete core costs/samples](measurements/0.4.2/core-cost.json).

## MCP round-trip costs

Thirty fresh release server processes compare five cases with three repeats per version. Both server binaries use Rust 1.98, thin LTO and one codegen unit. No deliberate builds/checks run during these samples; files are warm and unrelated desktop activity is uncontrolled. Search includes request/progress/response transport and excludes later passage readbacks. No transcript/schema-validator work runs inside these timers. Startup-to-initialized includes spawn, decode/restore, index construction and lifecycle, ending before `tools/list`. The fixed query sequence becomes partly warmed across candidates.

| Case | Startup median ms, old/new | BM25 median ms, old/new | TF-IDF median ms, old/new | Server peak RSS MiB, old/new |
| --- | ---: | ---: | ---: | ---: |
| canonical | 6933.8 / 8061.4 | 21.01 / 21.30 | 171.84 / 35.93 | 2269.5 / 1837.8 |
| vkspec | 450.7 / 500.3 | 5.86 / 4.94 | 32.26 / 26.57 | 166.0 / 138.4 |
| scipy | 306.7 / 317.9 | 6.22 / 5.86 | 41.17 / 24.77 | 119.6 / 101.1 |
| racket-reference | 109.9 / 124.3 | 2.51 / 2.53 | 9.81 / 6.78 | 48.2 / 41.5 |
| pdf-12 | 49.0 / 51.2 | 1.90 / 1.76 | 5.23 / 3.64 | 24.7 / 21.9 |

The canonical MCP TF-IDF median falls from **171.84 to 35.93 ms** (about **4.78 times faster**). Its startup median rises from **6.93 to 8.06 s**, while server peak RSS falls from **2,269.5 to 1,837.8 MiB**. BM25 round trips stay close on this case.

Canonical medians have 210 sequential samples per algorithm/version; each snapshot case has six. Snapshot indexing uses nonblank whole pages while canonical entries are older chunks, so granularity differs. RSS is the single child's lifetime maximum including startup JSON buffers, snapshots/index/runtime/transport; it excludes the Python oracle data. Four concurrent requests per connection have separate elapsed fields and are not mixed into sequential medians. Three warm repeats establish neither p95/p99 nor hard CPU/RAM capacity. The configured 512 MiB dataset accounting remains a payload limit, not a 512 MiB process-memory quota.

## Retention and release boundaries

All eight canonical exports, all 35 original PDF hashes and all 35 revisioned snapshots are rechecked unchanged. The canonical SHA-256 remains `13168d87872a137355fd8499c90a6084f52243a2ec7456f29b5fcb2a1c7423c3`. Knowledge, failed attempts, raw traces, scripts and binaries remain in the ignored private lab. Public artifacts contain counts, hashes and timing samples without document/query text.

The PDF backend remains exactly **0.3.5**, with 14 fresh regressions and the 381-file provenance check. Fresh dependency audits find zero known vulnerabilities; three main and four backend maintenance warnings are retained explicitly. The validator is dev-only; the profiler is external-only. Runtime dependency boundaries exclude both from core/MCP, along with HTTP, SQLite, PDF and native-model adapters unless their own features are requested. The lockfile's existing regex dependency advances one patch as required by the dev validator; no core tokenizer change is made.

Clean packaging, extracted-package tests, dry-run, exact-commit Linux CI and actual registry API/MCP/schema consumers gate/verify publication. Immutable checksums, VCS/tag identity and three GitHub assets are recorded in the release verification asset. A documentation-only final edit must rebuild to the same measured MCP executable hash. No Docker execution, live third-party client UI test or independent security-audit claim is made. Read-only tools do not sandbox a compromised host, and schemas do not replace authorization or safe source handling.

See [all public measurements](measurements/0.4.2/), [reproduction programs](validation-0.4.2/README.md), [core API](CORE-PERFORMANCE.md) and [MCP security responsibilities](MCP-SECURITY.md).
