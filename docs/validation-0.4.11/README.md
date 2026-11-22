# AIRust 0.4.11 validation

Compare exact registry 0.4.10 with candidate 0.4.11 using the same programs and inputs. This release changes norm-hint lookups and sparse broad scoring, not ranking semantics. Read the [report](../CHANGES-0.4.11.md), [performance contract](../CORE-PERFORMANCE.md), [API cookbook](../API-COOKBOOK.md) and [MCP boundaries](../MCP.md).

## Applications and frozen inputs

Copy each manifest to its private application as Cargo.toml and the paired Rust source alongside it. Before publication, add an explicit local path only to the candidate dependency. Keep exact locks, source hashes and executable hashes. After publication a fresh Rust 1.89 consumer must use the exact registry version without a path override. The consumer exercises retrieval, coverage, quotations, document positions and MCP/schema construction.

Retain the same private 72-file, 1,910-chunk snapshot and all 342 previously authored questions. They are exposed regression data, not a new holdout or real logged user questions. Compare complete original JSON strings for literal BM25, prepared BM25 and coverage: 1,026 comparisons. Use six passages, 100 candidates, 2,000 bytes per passage and 12,000 total text bytes. One warm-up and five timed repeats per query/configuration/version give 10,260 round trips, including serialization, IPC and Python parsing. Own compiler jobs must finish before timing.

## Query timing, allocation traffic and cold cache storage

Keep the prior sixteen fixtures. Add a warm TF-IDF query, 32/33-term TF-IDF over the existing 256-entry index, and BM25/TF-IDF sparse requests at 33, 128 and 512 terms over a separate 257-entry / 512-term corpus. There are 25 fixtures. Broad sparse fixtures use 200 timed calls per sample; the other 19 use 1,000. Seven alternating old/new process pairs give 282,800 timed calls in 14 plain processes. Build separately with heap instrumentation and run 100 calls per fixture in two processes, 5,000 calls total. DHAT durations are not latency measurements.

Run the heap executable with `norm_heap 1 PRIVATE_PROFILE.json` separately. Three cold queries record norm-hint live bytes after dropping result vectors; original sources and index setup are outside tracking. Record extra retained hint RAM as well as fewer allocations. Timed warm queries are initialized before timing. Signatures must match across versions, and temporary-call retained deltas must be zero.

The standard protected HashMap replaces the norm-hint BTreeMap. Invalidation, complete-value publication and nonblocking read/write/poison fallbacks remain. Wide candidate sets borrow existing Arc keys. If more than 32 known query terms exceed twice the document's unique words, scoring scans those words and binary-searches the query. Terms are still accumulated in the same sorted order. The independent oracle compares full score bits, explanations, scopes and cold/warm behavior; the checkpoint test interrupts sparse scoring.

## Actual MCP: initial and repeated requests

Run `python3 prepare_wide.py PRIVATE_DIRECTORY` to produce deterministic synthetic input and ten queries at 32, 33, 64 and 128 terms. Keep its extended-input hashes beside the unchanged original input hashes. Both preplanned campaigns use canonical 167,942-entry knowledge, the revision-bound NumPy snapshot and the synthetic wide input, five alternating old/new process pairs per input: 30 servers per campaign.

Each server first executes every query/algorithm once. Record these initial cold/partial-cache samples separately. Two later complete passes are timed and must match each initial full structured payload exactly. Discovery, schemas, source binding, exact snapshot readbacks, progress, concurrency, matching/relations, rejected arguments, health and EOF shutdown remain checked. Compare complete old/new tool payloads by JSON-RPC request ID, excluding only explicit status version and asserted release-qualified schema URNs. Concurrent replies need not arrive in request order.

Initial requests and warmed repetitions are separate scopes; do not compare their absolute values with older campaigns that mixed cache states. Snapshot readbacks occur outside search timing. Whole-child peak RSS includes input, parsing, index and transport. Source hashes must remain identical before/after every probe. Retain both complete campaigns, including slower observations. Stage-start host CPU/compiler snapshots do not guarantee continuously idle hardware.

## Construction and retained-knowledge controls

The construction application uses feature-free Core and excludes buffered file parsing/source generation from its Rust timer. Retain five old/new pairs for canonical knowledge and three synthetic distributions: 40 plain processes. Six separate profiles measure synthetic construction-derived live heap; already owned sources are excluded. Stats/signatures must match. This release does not target construction.

The execution/shared-index corpus worker separately checks updates/deletions/restoration, rebuilds, JSON round trips, concurrent readers, clone isolation and cancellation against all 167,942 entries. Source and portable export identities must agree. This is finite consistency validation, not a new PDF parser, semantic-quality or million-user capacity claim.

## Release gates and storage

Run the full default/all-feature/isolated-feature/Clippy/docs/Python matrix, minimum Rust versions, API comparison with 0.4.10, refreshed dependency audit, unchanged backend regressions, an independent consumer and ten targeted Tree Borrows Miri cases. Async/native/FFI work is outside those finite interpreter cases. Retain exact source and binary identities through measurement. Test the extracted package and require exact-commit CI; after publication verify registry source, checksum/VCS, tag and release asset.

Archive packages, logs, locks and small measured executables before deleting only the named temporary Cargo and Miri targets. Preserve PDFs, knowledge, source snapshots and all raw private results. No DenkMaps integration is performed.
