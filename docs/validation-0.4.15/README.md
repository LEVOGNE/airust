# AIRust 0.4.15 validation

Compare freshly built exact registry 0.4.14 (VCS c0a718604ad36026a33c5a15a67ad7f49c5f41da) with candidate 0.4.15 using identical programs and inputs. Read the [executed report](../CHANGES-0.4.15.md), [API cookbook](../API-COOKBOOK.md) and [MCP contract](../MCP.md).

Copy each manifest as Cargo.toml beside its paired Rust source. For worker-baseline.toml and worker-candidate.toml, copy retrieval-worker.rs under the name worker.rs; the other programs keep their supplied filenames. Add a local path only to the candidate before publication. Preserve exact locks and library/program/executable hashes. After publication run the Rust 1.89 consumer against exact registry 0.4.15 without a path override. Finish all managed compiler jobs before final timing.

The only runtime change removes the initial awaited MCP timer tick. First periodic progress is scheduled one configured interval later and the handler enters the existing completion/cancellation/deadline selector immediately. Start/completion, worker admission, bounded notifications, original deadlines, schemas and exact outputs remain. Core algorithms and memory layout are unchanged.

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

## Actual MCP: nine input/progress cases and two campaigns

`prepare_wide.py PRIVATE_DIRECTORY` creates deterministic broad data; `prepare_tiny.py PRIVATE_DIRECTORY` creates 1/4/16-entry knowledge and extends hashes. Retain canonical knowledge and the revision-bound NumPy snapshot. Test canonical/snapshot/wide with progress and each small input both with and without requested progress. Use `mcp_probe.py --no-progress` for the latter; it disables notifications for sequential and concurrent searches and checks their absence. With progress, verify start/completion and increasing sequences.

Five alternating old/new pairs per nine cases give ninety servers per complete campaign, 180 across the two preplanned campaigns. Each executes ten queries with both rankers, two repeated passes, four concurrent searches, matching/relations, bad arguments and health. Snapshot readbacks check exact loaded UTF-8 ranges. Expected totals: 13,880 schema-valid replies and 6,940 complete old/new structured pairs, including 1,640 readbacks. Compare by request ID, excluding only status version and asserted version-qualified schema URNs. Inputs must retain their hashes. Require advertised/actual schema validation, source bounds, ordering and EOF shutdown.

Keep first TF-IDF calls separate from initial aggregates and repeats. Search timing includes Python parsing, serialization, stdio and requested progress; snapshot readbacks are outside it. Startup loads/indexes outside tool deadlines. Peak child RSS includes buffers/index/runtime; Core heap profiles exclude stack space. Stage-start CPU/compiler snapshots do not prove continuously idle hardware. Existing rate/admission limits still apply and these finite bursts do not establish sustained throughput.

## Pilot and release gates

Keep the separate three-pair tiny/canonical pilot, including its slower canonical results. It used a retained measured baseline executable and fresh candidate. Do not merge it into final data. Final campaigns freshly build both versions. Preserve all snapshots, binaries, locks, raw transcripts and slower outcomes.

Run the complete default/all-feature/isolated-feature/Clippy/docs/Python matrix, Rust minimum versions, API comparison with 0.4.14, refreshed audit, backend regressions/provenance and independent consumer. Sixteen finite Tree Borrows Miri cases cover retained Core invariants. New Tokio/protocol behavior runs natively; no security-audit claim is made. Construction controls retain forty plain processes and six profiles; the shared-index worker checks all canonical records through updates/restoration/rebuild/roundtrip/clones/readers/cancellation.

Test the extracted package, require exact-commit CI and verify fresh registry source/VCS/checksum, annotated tag and release asset. Archive proof/packages/locks and small measured workers, then remove only the two named temporary Cargo/Miri targets. Preserve PDFs, knowledge and every raw result. No DenkMaps integration.
