# AIRust 0.4.17 validation

Compare freshly built exact registry 0.4.16 (VCS 869a814934737ded051e9e2ec64a73395e3285a5) with candidate 0.4.17 using identical programs/inputs. The separate earlier reference is exact 0.4.15 (VCS 6ac8f081b46bc63cbc9c999e0b88d944524fc62e). Read the [report](../CHANGES-0.4.17.md), [Core performance](../CORE-PERFORMANCE.md), [API cookbook](../API-COOKBOOK.md) and [MCP contract](../MCP.md).

Copy each manifest as Cargo.toml beside its paired Rust source. For worker-baseline.toml/worker-candidate.toml copy retrieval-worker.rs as worker.rs. Other programs keep supplied source filenames. Add a local path only to the candidate before publication. Preserve locks and hashes of actual private manifests/source copies, public method, library and measured executables. After publication use the Rust 1.89 consumer against exact registry 0.4.17 without a path override. Finish all managed compiler jobs before timing.

The 0.4.17 Core change directly collects one known term's sorted unique posting IDs, initially reserving min(32, posting count) borrowed slots. Keep every posting checkpoint/progress count and original scoring/filter order. Multiple known-term collection, large-query frequency cursors, the small MCP timer behavior and public APIs remain.

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

## Actual MCP: two complete ordinary campaigns

Use prepare_wide.py/prepare_tiny.py to retain broad and 1/4/16-entry synthetic inputs and hashes. Test canonical knowledge, revision-bound NumPy snapshot and wide input with progress; test each small input with and without progress. mcp_probe.py --no-progress disables sequential/concurrent notifications and checks absence. Other calls verify requested start/completion and increasing progress.

Five alternating old/new pairs per nine cases give 90 servers per campaign, 180 across two fixed campaigns. Each makes 20 initial searches, 40 repeats, four concurrent searches, matching/relations, rejected arguments and health; snapshot cases also check exact UTF-8 passage readbacks. Expected ordinary totals: 13,880 schema-valid replies and 6,940 complete old/new structured pairs including 1,640 readbacks. Compare by RPC ID excluding only status version and asserted version-qualified schema URNs. Check discovery, real advertised/actual schemas, bounds, ordering, input hashes and EOF shutdown. Separate first TF-IDF, aggregate initial and repeated calls. Search timing includes serialization/stdio/Python/requested progress; passage follow-ups are outside it. Startup indexing is outside tool deadlines.

## Direct controlled search

Run additional.py controlled after building controlled-baseline/candidate. Each controlled_work.rs process loads/warms the canonical Engine, checks 20 complete original responses across plain/deadline/progress modes and reports candidate counts. Readiness contains **private full response text** and must not be published. After RUN, ten rounds rotate three control modes across ten queries and both rankers. Seven alternating pairs give 14 processes, 8,400 timed calls and 140 cross-version complete original-response comparisons. Compare per-query signatures and report mode/ranker/per-query medians and tails. Setup, readiness, transport and profiler are outside call timing; controls add the same query allocation/observer/time-budget setup as defined by the program.

## Fresh three-version reference

Run additional.py reference with exact fresh 0.4.15/0.4.16/0.4.17 MCP executables. Five rotating triples give 15 canonical servers. Run the same complete probe and actual schema validation. Expect 1,020 schema-valid replies and 680 complete structured comparisons against the candidate, excluding only status version/checked URNs. Keep initial/warm/per-query/startup/RAM observations separate. Do not merge these samples into either ordinary campaign or use them to replace a slower result.

## Load, development and release gates

host_monitor.py records coarse ps CPU/compiler counts every two seconds across final stages, alongside stage-start snapshots. Active benchmark workers are included; other workloads are not stopped. Samples do not prove continuous idleness or identify a timing cause. Preserve every sample and do not discard slow trials based on load afterward.

Keep the separate seven-triple small Core development pilot, actual sources/locks/binaries, raw samples and all setup/export logs. Pilot timing is not merged into either final campaign. Historical 0.4.16 profiles and rejected prototypes stay in their earlier archive. No fresh holdout, model answer, semantic-quality or native PDF extraction claim is made.

Run the complete default/all-feature/isolated-feature/Clippy/docs/Python matrix, Rust minimum versions, API comparison with 0.4.16, refreshed audit, unchanged backend regressions/provenance and independent consumer. Twenty finite Tree Borrows Miri cases include scaled cursor identity/seek/cancellation cases; the native full-scan oracle covers 1,025 records, query widths/scopes/limits/cache states. Tokio/PDF/FFI execution is outside that finite scope. Construction retains 40 plain processes and six profiles; canonical shared-index checks cover updates/restoration/rebuild/roundtrip/clones/readers/cancellation.

Test the extracted package, require exact-commit CI and verify fresh registry source/VCS/checksum, annotated tag and release asset. Archive proof/packages/locks/small measured programs before removing only the two named temporary Cargo/Miri targets. Preserve PDFs, knowledge and every raw result. No DenkMaps integration.


## Primary small Core target and confirmation

Build matrix-reference/baseline/candidate from small_matrix.rs and query-reference from query_work.rs, alongside the baseline/candidate workers above. There are fifteen worker source/manifest directories in total. Public manifests use exact registry versions; only the candidate gets a local path before publication. Verify actual source copies, manifests, locks and executable hashes, not just a public method filename.

Run small_reference.py small-reference before the ordinary retained stages, and small_reference.py small-confirmation afterward. Each uses seven rotating triples of exact fresh 0.4.15/0.4.16/0.4.17, running old-six and matrix as separate programs: 42 processes, 2,142,000 timed calls and 1,344 full original matrix-response comparisons. Together: 84 processes, 4,284,000 calls and 2,688 complete comparisons. Keep campaigns separate; do not replace slow samples. Original six cases construct options inside their timer; the matrix reuses fixed options, so absolute values are not interchangeable.

The extension has 96 cases: 1/4/16/33 entries, both rankers, single/repeated/unknown-added/multiple/Unicode terms, unknown-only and punctuation queries, matching/disjoint scopes, limit-one/zero and wide queries. Warm-up and original full-result JSON comparison are outside timing. Actual full matrix outputs stay private; summaries expose every fixture median/p95 and raw timing sample, including slower cases. The original-six comparisons additionally require equal signatures. This is a fixed synthetic control, not fresh production or relevance holdout evidence.

The two new Miri cases cover single-list interruption and hand-calculated small-index score/tie/scope/limit/Unicode behavior. Native checks exercise larger sizes and the existing 1,025-record full-scan oracle. Keep all knowledge/PDFs, failed logs, pilot/final samples and exact small executables. Remove only the explicitly named 0.4.17 Cargo/Miri targets after verified publication.
