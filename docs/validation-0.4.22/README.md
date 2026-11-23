# AIRust 0.4.22 validation

Compare freshly built exact registry 0.4.21 (VCS 8ce456be12218f38cffc32ed1737accbe24f8668) with candidate 0.4.22 using identical programs/inputs. The separate earlier reference is exact 0.4.20 (VCS e30a055fab3dae32c8e59047fc01a2e32226dbed). Read the [report](../CHANGES-0.4.22.md), [Core performance](../CORE-PERFORMANCE.md), [API cookbook](../API-COOKBOOK.md) and [MCP contract](../MCP.md).

Copy each manifest as Cargo.toml beside its paired Rust source. For worker-baseline.toml/worker-candidate.toml copy retrieval-worker.rs as worker.rs. Other programs keep supplied source filenames. Add a local path only to the candidate before publication. Preserve locks and hashes of actual private manifests/source copies, public method, library and measured executables. After publication use the Rust 1.89 consumer against exact registry 0.4.22 without a path override. Finish all managed compiler jobs before timing.

The selected change carries the once-checked worker payload in a private immutable value/budget witness. Reuse that check only at the same budget; recheck if it changes. Move the owned JSON value into its envelope, preserving insertion order and full wire bytes. Always retain the final encoded-reply bound, including escaped text and duplicated structured content. No public schema, permission, dependency or saved field changes. Core source is byte-for-byte unchanged from 0.4.21. Existing progress/admission/cancellation remain; final response construction still follows worker search completion and is not newly covered by its deadline.

## Primary small-MCP comparison

Run small_mcp_control.py standard first and confirmation after all retained controls. Each uses seven rotating triples of fresh exact 0.4.20/0.4.21/0.4.22 on tiny/tiny4/tiny16 inputs, each with/without progress: 126 servers, 8,568 schema-valid actual replies and 5,712 complete structured comparisons per campaign. Combined totals are 252, 17,136 and 11,424. Compare discovery, advertised schemas and complete RPC-ID responses, normalizing only asserted status version and version-qualified schema URNs. Check input hashes, requested/absent progress and EOF shutdown. Retain all initial/repeated ranker/per-query samples, process sums, startup/RSS and coarse host observations. Keep two campaigns separate, including slower samples. These are fixed synthetic controls, not production or semantic holdouts.

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

Run additional.py reference with exact fresh 0.4.20/0.4.21/0.4.22 MCP executables. Five rotating triples give 15 canonical servers. Run the same complete probe and actual schema validation. Expect 1,020 schema-valid replies and 680 complete structured comparisons against the candidate, excluding only status version/checked URNs. Keep initial/warm/per-query/startup/RAM observations separate. Do not merge these samples into either ordinary campaign or use them to replace a slower result.

## Load, development and release gates

host_monitor.py records coarse ps CPU/compiler counts every two seconds across final stages, alongside stage-start snapshots. Active benchmark workers are included; other workloads are not stopped. Samples do not prove continuous idleness or identify a timing cause. Preserve every sample and do not discard slow trials based on load afterward.

The separate development pilot uses three rotating triples on standard-wide, extended-wide, canonical and tiny MCP plus six Core cases. Complete structured responses agree. Preserve its MCP/Core source snapshots, locks, workers and all observations separately from final distributions. Earlier releases retain their own selected/rejected variants in their original archives. No pilot or faster rerun replaces final samples.

Run the complete default/all-feature/isolated-feature/Clippy/docs/Python matrix, Rust minimum versions, API comparison with 0.4.21, refreshed audit, unchanged backend regressions/provenance and independent consumer. Thirty-three finite Tree Borrows Miri cases include the previous 30 cases and three new pure MCP reply tests; the native full-scan oracle covers 1,025 records, query widths/scopes/limits/cache states. Tokio/PDF/FFI execution is outside that finite scope. Construction retains 40 plain processes and six profiles; canonical shared-index checks cover updates/restoration/rebuild/roundtrip/clones/readers/cancellation.

Test the extracted package, require exact-commit CI and verify fresh registry source/VCS/checksum, annotated tag and release asset. Archive proof/packages/locks/small measured programs before removing only the two named temporary Cargo/Miri targets. Preserve PDFs, knowledge and every raw result. No DenkMaps integration.


## Retained small Core target and confirmation

Build matrix-reference/baseline/candidate from small_matrix.rs and query-reference from query_work.rs, alongside the baseline/candidate workers above. There are fifteen worker source/manifest directories in total. Public manifests use exact registry versions; only the candidate gets a local path before publication. Verify actual source copies, manifests, locks and executable hashes, not just a public method filename.

Run small_reference.py small-reference first and small_reference.py small-confirmation after all large/broad controls, keeping both distributions separate. Each uses seven rotating triples of exact fresh 0.4.20/0.4.21/0.4.22, running old-six and matrix as separate programs: 42 processes, 2,142,000 timed calls and 1,344 full original matrix-response comparisons. Together: 84 processes, 4,284,000 calls and 2,688 complete comparisons. Keep campaigns separate; do not replace slow samples. Original six cases construct options inside their timer; the matrix reuses fixed options, so absolute values are not interchangeable.

The extension has 96 cases: 1/4/16/33 entries, both rankers, single/repeated/unknown-added/multiple/Unicode terms, unknown-only and punctuation queries, matching/disjoint scopes, limit-one/zero and wide queries. Warm-up and original full-result JSON comparison are outside timing. Actual full matrix outputs stay private; summaries expose every fixture median/p95 and raw timing sample, including slower cases. The original-six comparisons additionally require equal signatures. This is a fixed synthetic control, not fresh production or relevance holdout evidence.

Three new native pure-response tests compare exact encoded lengths at acceptance/rejection boundaries, entire legacy wire bytes, changed-budget checks and rejection of oversized escaped/duplicated/enveloped replies. They run under Miri with the mcp feature, without Tokio I/O. Preserve all 30 previous finite Miri cases and their scaled scope, native full-scan 1,025-record checks and native 2,048-record clustered/gap regression. Existing full RPC bounds, hostile inputs, timeout, disconnect and drain checks remain.


## Retained broad MCP controls and protected bounds

Run wide_control.py standard before retained controls, then confirmation and extended after them. Each campaign has seven rotating 0.4.20/0.4.21/0.4.22 triples, 21 fresh servers, 1,428 actual schema replies and 952 complete cross-version structured comparisons. Totals: 63 servers, 4,284 schema-valid replies and 2,856 complete comparisons. Keep three distributions separate. Standard/confirmation repeat the unchanged 257-record, 512-word-vocabulary fixture and original ten queries. Extended adds 32/33/64/65/128/129/192/255/256 known terms and a duplicate/unknown case, staying within 256 unique normalized assessment terms and 4,096 query bytes. The input includes sparse 16-term documents and one dense vocabulary document. For each ranker retain initial/repeated per-query medians, p95, arithmetic means, process totals, startup, RSS and coarse host samples. No slow sample is removed.

Run wide_limits.py separately: three fresh servers reject257/512 known-term queries for both rankers (twelve errors), report error progress, expose no partial result and accept later health requests. This checks the existing256 unique assessment-term bound; it does not relax it or benchmark rejected queries. Limits remain cooperative, not OS CPU/RAM quotas.

Before final timing compare every cached baseline/reference source file byte-for-byte with its preserved checksum-verified published crate archive. Freeze public method, actual private worker source/manifests/locks, binaries, runtime and inputs. Export only sanitized records; source text, questions, full responses and raw transcripts stay private. After all gates, test the packaged crate and require exact-commit CI, annotated tag, registry checksum/VCS and release-asset agreement before deleting temporary builds.

## Retained large-corpus MCP controls

Run large_control.py standard after the first small Core campaign and large_control.py confirmation before the last small Core campaign. Each uses seven rotating triples of exact 0.4.20/0.4.21/0.4.22 with the canonical 167,942-entry knowledge input and the same ten queries. Each has 21 fresh servers, 1,428 actual schema replies and 952 complete cross-version comparisons. Together: 42 servers, 2,856 schema replies and 1,904 complete comparisons. Check schema URNs, asserted version, complete structured results by RPC ID, progress, bounds, concurrency, EOF shutdown and unchanged inputs. Only status version/checked schema URNs are normalized; no score, ordering, matched term, source text or position is removed from comparison.

Keep these two distributions separate from the ordinary 90-server campaigns, the 15-server canonical reference and the three 21-server broad guards. Preserve initial/repeated ranker and per-query means/medians/p95, per-process totals, startup/RSS and host samples. Timings include stdio/serialization/Python/progress; source/index startup is separate. Every slower observation remains. Candidate counts range from 0 to 29,600 in the retained ten queries; this is one corpus/workload, not a proof for every sparse distribution.

## Registry source-package boundary

Keep every raw measurement JSON file in the tagged GitHub repository and local archives. Exclude docs/measurements/**/*.json only from the registry source package; retain API/Markdown guides, method source, examples and required fixtures. scripts/check_package.py rejects measurement JSON in the package listing. Verify the extracted committed package with all features before publishing. Record the actual package bytes/checksum; the earlier dirty preview is size/boundary evidence, not the published checksum. This follows [Cargo packaging guidance](https://doc.rust-lang.org/stable/cargo/reference/publishing.html). No source knowledge, rejected trial or measurement is deleted.
