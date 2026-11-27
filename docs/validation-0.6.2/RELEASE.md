# AIRust 0.6.2 release verification

On 2026-11-27 the user explicitly authorized publishing **0.6.2**: merging the validated branch into master, an annotated tag, a GitHub Release and the crates.io package. The published 0.6.1 and 0.6.0 crates and the separately versioned PDF backend 0.3.5 remain immutable. The backend is not republished.

## Authoritative publication receipts

The [GitHub release](https://github.com/LEVOGNE/airust/releases/tag/v0.6.2) carries the exact crate, its checksum and the release manifest. The [registry version](https://crates.io/crates/airust/0.6.2) must identify the same crate checksum and packaged Git commit as the annotated tag. The local continuation record is `local-lab/release-0.6.2-2026-11-27/PROGRESS.json`. `final-closeout.json` exists only after the publication, registry, asset, consumer and CI checks are complete. If identities disagree, inspect the recorded stage. Do not repeat publication or move a tag.

## Source and release gates

- **Engineering source.** Implementation `a5725fc`, documentation closeout `1cbd70e` (hosted CI `45669021076` passed), review fixes and test control `9e792bc`, and maintenance evidence `7dda87c` (hosted CI `45948393659`).
- **Release preparation.** It changes only version examples, current-release wording, this record and continuation metadata. Rust source, manifests and locks, knowledge, frozen datasets, model manifest, MCP schemas and benchmark methods are unchanged.
- **Fresh checks on the release commit.** The release runs the complete `full` profile of the [test control](../TESTPLAN.md): `test.sh` with the native, feature, Clippy, rustdoc, example, Python and package checks; Rust 1.89 and 1.92; and the backend. It also runs a clean `cargo package`/dry-run with a content inspection, extracted-package consumers with Rust 1.89 and 1.92, exact-source master and tag CI, an anonymous registry download compared byte for byte, and registry consumers.
- **Reused results.** Miri (128 tests in 12 groups, twice) and fuzzing (105 to 140 minutes of cumulative ASAN search per target) come from the 2026-11-26/27 runs recorded in the [maintenance section](README.md#post-closeout-review-and-maintenance-2026-11-26). They are reused because the Rust source is unchanged, and they are not relabelled as fresh release runs.

## Measured behavior and limits

- **Memory.** The explicit process owner keeps the unloaded parent stable across repeated MiniLM load/use/drop. Three thirty-cycle post-warm parent-RSS ranges were 0.328, 0.406 and 0.375 MiB, and ten later worker runs stayed between 0.219 and 0.328 MiB. Direct in-process loading still retains about 200 MiB or more of allocator RSS under the default macOS allocator, and its growth gate still fails. Active native models exceed 1 GiB. Verified copies need disk space.
- **Latency: the release does not pass this gate.** The declared warm-recovery cost gate (worker p95 at most 1.10 x direct) **failed in 4 of 11 serial pair comparisons** (1.213, 1.169, 1.622 and 2.516). The steady-state boundary cost was small, a median of 1.2 to 3.2 ms per 20-call rerank, but p95 tails were not stable on the measuring host, and a foreign one-core load was present during all 2026-11-26 measurements. The process mode is therefore released as an **opt-in memory-ownership option without a latency guarantee**. Worker load p95 was higher than direct load in every recorded pair. See the [measurements](../measurements/native-lifecycle-0.6.2/README.md) and the [IPC record](../measurements/ipc-overhead-0.6.2/README.md).
- **Scope exclusions.** Miri excludes native ONNX, HTTP, PDF parser FFI, OS worker processes and the declared asynchronous cases. Fuzzing found nothing within its budgets with LeakSanitizer off; that is not a correctness proof. Startup, cooperative cancellation, OS cleanup and termination, trusted-executable permissions and multiple-instance admission limits remain explicit API contracts.
- **Unestablished.** Actual E5 inference, Windows and Intel native execution, actual 1-GiB hardware and unique system/GPU accounting are not established.
- **Independent gates.** **Zero of the fifty required independently reviewed questions** and no independent timed onboarding are established. Synthetic resource cases, conversation replays and automated consumers do not count.
- **Open review decisions.** CSP and inline handlers in the web frontend, admin mode without a token, web size limits, database indexes, duplicate hashes and labels, and the worker environment and runtime hash remain user decisions. They are not changed by this release.

Continue from [SESSION-HANDOFF.md](../SESSION-HANDOFF.md). Later versions need their own evidence and publication authorization.
