# AIRust 0.4.14 validation

Compare exact registry 0.4.13 with candidate 0.4.14 using identical programs and inputs. Read the [report](../CHANGES-0.4.14.md), [performance contract](../CORE-PERFORMANCE.md), [API cookbook](../API-COOKBOOK.md) and [MCP contract](../MCP.md).

## Source and applications

Copy each manifest as Cargo.toml and its paired Rust source alongside it. Before publication, add a local path only to the candidate. Preserve exact locks, library/program hashes and measured executable hashes. After publication a fresh Rust 1.89 consumer must use exact registry 0.4.14 without a path override. It exercises retrieval, coverage, quotations, document positions and MCP/schema construction. Managed compiler jobs must finish before final timing.

The selected implementation batches successful norm-hint reads for at most 128 hits, releasing on misses, progress callbacks and before explanations. Cold word weights use eight inline slots, promoting to the existing 256-slot heap memo after 128 terms from earlier completed norms. Both tiers compare full keys and preserve exact IDF values. Inline/control state uses stack space that heap profiles exclude. Hint publication remains nonblocking/best effort; other readers can defer it without changing answers.

## Retained document and warm controls

Keep the same private 72-file, 1,910-chunk snapshot and 342 exposed authored questions. These are not new holdouts or logged production traffic. Literal BM25, prepared BM25 and coverage retain six passages, 100 candidates, 2,000 bytes per passage and 12,000 total bytes. Compare 1,026 complete original JSON strings and all repeated payloads. One warm-up and five samples per question/configuration/version yield 10,260 round trips, including serialization, IPC and parsing.

Retain all 25 warm query fixtures. Seven alternating pairs give fourteen plain processes and 282,800 timed calls: nineteen cases use 1,000 calls/sample, six broad cases use 200. Two separately instrumented processes use 100 calls/fixture, 5,000 total. Norms are initialized first. Signatures must match and temporary-call retained deltas must be zero. Instrumented durations are excluded from latency comparisons.

`small_time 1000` separately measures six warmed cases: 1/4/16 entries with both rankers. Fourteen processes give 84,000 calls. Two `small_heap 100 PRIVATE_PROFILE.json` processes give 1,200 separately instrumented calls, with result signatures and retained deltas checked. These additions apply equally to both versions; historical absolute tables with other programs are not directly interchangeable.

## Cold and heap controls

`cold_time 1` makes twenty fresh-cache calls per fixture using clones of a never-queried Engine. Six cases cover the original 1,000-entry index, 256-entry wide index, 257-entry sparse/vocabulary index and 1/4/16-entry controls. Seven alternating pairs give 1,680 calls. Clone/setup is outside the timer; process/instruction/filesystem caches remain, so this is cold norm state rather than cold disks.

`norm_heap 1 PRIVATE_PROFILE.json` records six first-query allocation/retained-hint cases after dropping results, excluding source/index setup. `corpus_norm_heap 1 PRIVATE_PROFILE.json CANONICAL.json` loads/indexes 167,942 records before profiling ten initial TF-IDF calls, once per version. Allocation traffic, net retained hint growth, stack state and process RSS are separate quantities.

## Finite mixed-work control

`mixed 1` creates 6,000 synthetic entries: 5,000 warm-group and 1,000 new-group records. An independent Engine computes fixed reference signatures. Prime only the measured Engine's warm group; start four continuous warm readers and execute 100 new-group queries, then stop/join readers. Check every result and make two quiescent probes. New-group calls can become warm during the loop.

Five alternating pairs yield ten processes, 1,000 driver calls and twenty quiescent probes. Warm work counts vary with the stop-on-driver-completion rule. Keep all counts/samples; do not present this as a fixed-load throughput or capacity benchmark. The native held-reader regression separately forces deferred hint publication, checks nonblocking exact output and verifies recovery afterward.

## Actual MCP: four inputs and two campaigns

`prepare_wide.py PRIVATE_DIRECTORY` creates the deterministic broad corpus/queries; `prepare_tiny.py PRIVATE_DIRECTORY` adds a one-entry corpus and extends its hashes. The final campaigns use canonical knowledge, a revision-bound NumPy snapshot, synthetic wide input and one-entry input. Five alternating old/new pairs per input give forty servers per campaign, eighty total.

Each server first executes ten queries with both rankers, then two repeated passes which must reproduce each initial structured payload. Retain the first TF-IDF call separately from the initial aggregate and warm passes. Compare complete old/new payloads by JSON-RPC request ID, excluding only explicit status version and asserted version-qualified schema URNs. Validate actual replies against advertised schemas. Expected totals across both campaigns are 7,080 schema-valid replies and 3,540 complete pairs, including 1,640 exact snapshot readbacks. Discovery, source binding, progress, concurrent ordering, bounds, matching/relations, rejected arguments, health and EOF shutdown remain checked.

Search timing includes progress/stdio transport. Initial snapshot follow-up reads are outside it. Startup includes synchronous loading/indexing outside request deadlines; no norm prewarming is added. Peak process RSS includes buffers, index and runtime. Hash all inputs before/after probes. Stage-start CPU/compiler snapshots do not establish continuously idle hardware.

## Development and release gates

Two separate pilots are retained privately. The first direct-delay variant regressed some small cold cases and was not selected; the inline variant's promising small timings were not substituted for final samples. A separate mixed-work check preceded the final paired control. Preserve all snapshots, binaries and slower outcomes.

Run the complete default/all-feature/isolated-feature/Clippy/docs/Python matrix, Rust minimum versions, API comparison with 0.4.13, refreshed audit, backend regressions/provenance, independent consumer and sixteen targeted Tree Borrows Miri cases. Native/async/FFI work is outside the finite interpreter scope. Construction controls retain forty plain processes and six profiles. The execution/shared-index worker checks updates, restoration, rebuilds, JSON round trips, clones, parallel readers and cancellation on all canonical entries.

Test the extracted package and require exact-commit CI, then verify fresh registry source/VCS/checksum, annotated tag and release asset. Archive packages/proof/locks and small measured programs before removing only the two named temporary Cargo/Miri targets. Preserve PDFs, knowledge, source snapshots and every raw result. No DenkMaps integration is performed.
