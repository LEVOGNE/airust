# AIRust 0.4.16 validation

Compare freshly built exact registry 0.4.15 (VCS 6ac8f081b46bc63cbc9c999e0b88d944524fc62e) with candidate 0.4.16 using identical programs/inputs. The separate earlier reference is exact 0.4.14 (VCS c0a718604ad36026a33c5a15a67ad7f49c5f41da). Read the [report](../CHANGES-0.4.16.md), [Core performance](../CORE-PERFORMANCE.md), [API cookbook](../API-COOKBOOK.md) and [MCP contract](../MCP.md).

Copy each manifest as Cargo.toml beside its paired Rust source. For worker-baseline.toml/worker-candidate.toml copy retrieval-worker.rs as worker.rs. Other programs keep supplied source filenames. Add a local path only to the candidate before publication. Preserve locks and hashes of actual private manifests/source copies, public method, library and measured executables. After publication use the Rust 1.89 consumer against exact registry 0.4.16 without a path override. Finish all managed compiler jobs before timing.

The selected Core change uses at most four query-local posting-frequency cursors after 128 admitted candidates, at least 1,024 total candidates and 1–4 known terms. After 32 skipped postings checkpoint before seeking. Keep original scoring order and checkpoints, source/API contracts and small-call timer behavior. Extra temporary scratch is distinct from retained memory.

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

Run additional.py reference with exact fresh 0.4.14/0.4.15/0.4.16 MCP executables. Five rotating triples give 15 canonical servers. Run the same complete probe and actual schema validation. Expect 1,020 schema-valid replies and 680 complete structured comparisons against the candidate, excluding only status version/checked URNs. Keep initial/warm/per-query/startup/RAM observations separate. Do not merge these samples into either ordinary campaign or use them to replace a slower result.

## Load, development and release gates

host_monitor.py records coarse ps CPU/compiler counts every two seconds across final stages, alongside stage-start snapshots. Active benchmark workers are included; other workloads are not stopped. Samples do not prove continuous idleness or identify a timing cause. Preserve every sample and do not discard slow trials based on load afterward.

Keep the original sampled baseline profile, rejected ordered-document prototype, selected posting-cursor pilot, all failed setup/test logs and slower pilot outcomes. Their sources/locks/binaries stay private. No profiler/pilot timing is merged into final data. No fresh holdout, model answer, semantic-quality or native PDF extraction claim is made.

Run the complete default/all-feature/isolated-feature/Clippy/docs/Python matrix, Rust minimum versions, API comparison with 0.4.15, refreshed audit, unchanged backend regressions/provenance and independent consumer. Eighteen finite Tree Borrows Miri cases include scaled cursor identity/seek/cancellation cases; the native full-scan oracle covers 1,025 records, query widths/scopes/limits/cache states. New Tokio/PDF/FFI work is outside that finite scope. Construction retains 40 plain processes and six profiles; canonical shared-index checks cover updates/restoration/rebuild/roundtrip/clones/readers/cancellation.

Test the extracted package, require exact-commit CI and verify fresh registry source/VCS/checksum, annotated tag and release asset. Archive proof/packages/locks/small measured programs before removing only the two named temporary Cargo/Miri targets. Preserve PDFs, knowledge and every raw result. No DenkMaps integration.
