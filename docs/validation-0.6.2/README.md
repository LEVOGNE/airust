# AIRust 0.6.2 development validation

**Release note (2026-11-27):** 0.6.2 was authorized for publication. [RELEASE.md](RELEASE.md) governs the release gates and limits. The historical candidate text below is unchanged.

The native process-owner implementation and the local engineering checks are complete. **This is an unreleased candidate.** Memory ownership passed its finite controls; a repeated maximum-10% warm latency claim did not. The clean implementation commit/package and its separate applications passed. The remote push was rejected by automatic approval review before execution; candidate hosted CI has not started and is not a pass. Final source/package receipts are recorded separately in `local-lab/native-lifecycle-2026-11-25/final-closeout.json`; do not infer publication or a green hosted job from this document.

Read [scope](../CHANGES-0.6.2.md), [API/ownership](../LOCAL-MODEL-PROCESSES.md), [migration](../MIGRATION-0.6.2.md), [measurements and every regression](../measurements/native-lifecycle-0.6.2/README.md), [machine-readable local results](closeout.json) and [reproduction](REPRODUCE.md).

## Observed local checks

| Check | Observed result and scope |
| --- | --- |
| Complete `./test.sh` | Passed: **266 default / 940 all-feature**, including 40 doctests, before the final crash/recreation test was added. Strict Clippy/rustdoc, Python/real CLI/workflow/MCP, 40 separately checked features and package boundary passed. Counts overlap. |
| Final all-feature run | **941 passed**, including 40 doctests. Final all-target/all-feature Clippy and formatting passed. |
| Latest worker boundary/fault controls | **12 passed**, including large escaped requests (about 12 MiB on the wire), invalid-input recovery, blocked writes/reads/startup, malformed frames/IDs/counts/NaNs, crashes, bounded diagnostics, graceful/forced exit and repeated explicit recreation with original bytes preserved. This is a targeted rerun, not twelve additional unique tests. |
| Minimum compilers | Rust **1.89**, neutral features/all targets, and Rust **1.92**, all features/all targets, passed. The final escaped-frame assertion also compiled under 1.92. |
| Backend 0.3.5 | **14 regressions**, opt-in C ABI, provenance/license/helper formatting and Rust 1.89 passed. Source/lock unchanged; not republished. |
| Public API | Five static views against exact tag v0.6.1: default, MCP, local embeddings, local reranking, all features passed. Archive-selection setup failures remain retained. |
| Miri | **128 passed tests / 12 observed groups**, using TreeBorrows on recorded nightly. The initial full invocation remains failed because a new worker test did not compile; a corrected six-test worker subset and final twelve/eight-test example subsets supply explicit replacement receipts. This is an aggregate, not a relabeled successful original invocation. |
| Protocol libFuzzer + ASAN | Fresh **600-second campaign**, **45952362 executions**, **13047 instrumentation counters**, no crash/timeout/artifact detected; max observed ASAN RSS 724 MiB. All inputs/logs/artifacts and source/binary/runtime hashes retained. |
| Advisories | Fresh official RustSec clone, three lockfiles, **zero listed vulnerabilities**, no ignored IDs. Main/backend/fuzz maintenance warnings remain **3 / 4 / 2**. Registry yank-status checking was excluded explicitly. |
| Separate applications | Extracted preview crate: Rust 1.89 core application passed with model/HTTP/MCP/database dependencies absent. Rust 1.92 built the standalone worker and a separate application; **eight actual model cycles**, **160 baseline-exact result positions**, all workers reaped and private copies removed. Functional controls, not timed benchmarks or independent human onboarding. |
| Data/asset preservation | Original database, runtime, eleven existing model files / **19387512128 bytes**, backend, knowledge and frozen reference/fuzz fixtures matched retained identities. No model/runtime download or model-service startup; user processes untouched. |

The final all-feature invocation follows an additional crash/recreation regression. Its later large-wire extension was executed by the final twelve-test worker subset; unrelated successful tests are not presented as newly repeated at that point. Final documentation changes are checked by strict Rustdoc; Rust code blocks retain the tested APIs. Clean package extraction and source identity are separate closing controls.

## Failures and exclusions remain visible

The first full script found a missing `pilot`/`local-reranking` feature intersection in the new example worker arm. A corrected full attempt then caught a no-op explicit-drop Clippy violation in the benchmark harness. New validation tests initially had an array-reference coercion compile error; this was corrected without changing production score semantics. All failed logs are retained; complete successful retries and final targeted controls are separate. Package-preview inspection initially rejected the pre-existing `compat-0.2.30/legacy.db` fixture; its exact published bytes were verified before the separate consumer run. The original user database is excluded.

Miri does **not** execute ONNX/HTTP/native parser FFI, OS worker processes or the two declared asynchronous MCP cases. Four large index controls and the large independently generated corpus-parity case run natively. Filesystem groups explicitly disable Miri isolation for temporary fixtures. Existing directory-sync exclusions remain; native durability tests passed. Mutation/training/asset fixtures have their recorded smaller Miri workloads. The pure worker mutation loop is eight rounds under Miri versus 2048 natively. The twelve native worker tests are deliberately excluded from interpretation.

Fuzzing executes the **production protocol source**, shared by path rather than copied. It covers bounded framing, strict serde shapes, raw score-bit serialization and UTF-8 diagnostic truncation, with a fuzz-selected frame maximum 0..4080 bytes and input maximum 65536. Oversized reads are exercised; this is not a campaign of valid 16-MiB requests. The separate native wire-boundary test covers large successful escaping/backpressure. The fuzzer starts no model/process and opens no caller path. LeakSanitizer is off; standard library, C/system and native inference are not fully instrumented. Earlier lexical/training/PDF campaigns remain historical evidence, not fresh 0.6.2 campaigns.

Source comparison with the frozen alternating run confirms that the production library and manifests are unchanged. Only test code and benchmark handshake/formatting cleanup follow that run; detailed source differences and original binary/source identities are preserved. The 314-cycle measurements are not reassigned to rebuilt artifacts. The eight extracted-worker cycles supply additional actual final-API integration evidence without new timing claims.

The old in-process adapter still retains substantial allocator RSS after repeated drop. The selected process/file path observes stable unloaded-parent RSS, but active model memory exceeds 1 GiB and verified copies need disk space. One serial warm-p95 confirmation fails at **1.213**; the subsequent alternating diagnostic **0.986** does not erase it. A failed cost gate is not converted to a pass by successful safety/correctness tests. Startup, cooperative cancellation, OS cleanup/termination, trusted-executable permissions and multiple-instance admission limits remain explicit in the API contract.

Actual E5 inference, Windows/Intel native execution, unique physical/system/GPU accounting and actual 1-GiB hardware are unestablished. RustSec maintenance notices remain: chrono, paste and ttf-parser in the main graph, plus anymap2 in the backend comparison graph. Audits do not certify the separately supplied native runtime. **Zero of fifty required independent questions** and no independent timed onboarding are established. This work does not publish 0.6.2 or close those product-readiness gates.

## Exact local source and external continuation

Implementation commit: `a5725fcc0614982b016216679b7891546d0384c2`. Its clean archive has 4075 files, 6866544 bytes and SHA-256 `0c516d93e03cdc13ed3a6881727bcaa9385a449067bdec7f8a3121cd8da9952c`; packaged Rust/numeric artifact bytes match. Separate core/native applications also pass against this exact clean archive, including a second eight-cycle/160-result standalone-worker cohort. These are two separate functional cohorts, not additional thirty-cycle growth gates.

Automatic approval review rejected pushing `codex/stabilize-native-model-lifecycle` to `https://github.com/LEVOGNE/airust.git` because explicit authorization for that concrete external source transfer was not established. No candidate source was uploaded and hosted CI was not started. A new explicit branch-push approval is required before retrying; no alternate upload or bypass is authorized. Local closeout documentation does not change runtime or measurement inputs. The release remains 0.6.1; a new release needs its own instruction.

## Post-closeout review and maintenance (2026-11-26)

This section adds later evidence. It does not change the observations above, and none of it is a release.

**Remote state.** After the rejected attempt described above, the branch was pushed at `1cbd70e` (documentation-only follow-up to `a5725fc`), and hosted CI run `45669021076` ("Rust") passed. The source review fixes and the test control are in a later local commit, `9e792bc`, which has not been pushed. The maintenance changes below are uncommitted working-tree changes on top of `9e792bc`.

**Review fixes (`9e792bc`).** A six-reviewer source review, with every finding checked against the code, led to these fixes:

- A worker request rejected before transmission no longer consumes the sequence ID.
- A provider `NonFiniteScore` position is reported as the original input rank across batches. A regression test covers this.
- Web file writes are atomic.
- A Tatoeba import is one database transaction and reports its errors.
- Web PDF uploads apply the default import limits.
- `add_message` is transactional.
- Vector search sorts with `total_cmp`.
- The cache checks the file type and length on the opened handle.
- The shared local runtime tolerates a poisoned lock.
- CLI `stop` ignores an already removed record.
- A failed detached start kills and reaps its child.

Findings that would change the schema, the public behaviour or the security policy were not implemented. They remain open user decisions: CSP and inline handlers in the web frontend, admin mode without a token, web size limits, database indexes, duplicate hashes and labels, and the worker environment and runtime hash.

**Resource-aware test control.** [TESTPLAN.md](../TESTPLAN.md) defines 34 area configurations. Every run below was selected and recorded by `scripts/testctl.py`. The exported status and run reports, including the failed run, are in [maintenance-2026-11-26/testctl-receipt.json](maintenance-2026-11-26/testctl-receipt.json), with absolute paths normalized. Raw logs and fuzz corpora stay in the ignored local `.testctl/` directory and are not published.

| Check | Source | Result |
| --- | --- | --- |
| Miri, 12 groups, TreeBorrows, nightly `5f4effd579`, isolated worktree | `9e792bc` | **128 passed tests**, 0 failed (core 25, index 41, training-knowledge 18, pilot 12, profile-config 8, input 6, model-worker 6, mcp-response 4, mcp-backend 3, model-assets 2, training-properties 2, stability-state 1); 1471 s total |
| libFuzzer + ASAN, `knowledge`, isolated worktree | `9e792bc` | 1200 s, **149717** executions, 1152 new units, peak RSS 479 MiB, no crash/timeout/artifact |
| libFuzzer + ASAN, `lexical` | `9e792bc` | 1200 s, **102971** executions, 1088 new units, peak RSS 587 MiB, no artifact |
| libFuzzer + ASAN, `model_worker` (production protocol source) | `9e792bc` | 900 s, **69380168** executions, 14202 new units, peak RSS 740 MiB, no artifact |
| libFuzzer + ASAN, `pdf` | `9e792bc` | 1500 s, **69684** executions, 2898 new units, peak RSS 1659 MiB, no artifact |
| libFuzzer + ASAN, `lexical` short | `1cbd70e` working tree | 60 s, 5563 executions, no artifact |
| Concurrency short (triggered) | `1cbd70e` working tree | 93 passed, 1 ignored |
| Concurrency deep, release, 25 repetitions | `9e792bc` + working tree | **2325 passed**, 0 failed, 25 ignored, 337 s |
| libFuzzer + ASAN, extended round 1 outside testctl, 2700 s per target on the retained corpora ([runs](maintenance-2026-11-26/fuzz-extended/round-1-runs.txt), [script](maintenance-2026-11-26/fuzz-extended/run.sh.txt)) | binaries from the testctl builds above (`9e792bc` + working tree; model_worker from the isolated `9e792bc` build) | knowledge **326716** executions (corpus 1004 to 1808), lexical **255499** (1715 to 2720), model_worker **257206100** (1461 to 4664), pdf **88292** (4107 to 6139); exit 0, no artifact |
| libFuzzer + ASAN, extended round 2, 1800 s per target, same binaries and corpora ([runs](maintenance-2026-11-26/fuzz-extended/round-2-runs.txt)) | as round 1 | knowledge **218200** (corpus 1808 to 2331), lexical **173808** (2720 to 3343), model_worker **187719459** (4664 to 5750), pdf **54403** (6139 to 7262); exit 0, no artifact |
| libFuzzer + ASAN, extended round 3, 900 s per target, after the foreign load had ended ([runs](maintenance-2026-11-26/fuzz-extended/round-3-runs.txt)) | as round 1 | knowledge 103870 (corpus to 2590), lexical 93068 (to 3667), model_worker 99497112 (to 6357), pdf 23395 (to 7656); exit 0, no artifact |
| **Cumulative fuzz search, 2026-11-26/27** | | knowledge 7803 s / 946468 executions; lexical 7866 s / 740285; model_worker 6302 s / 613802839; pdf 8406 s / 288985. No crash, timeout or ASAN report within these budgets. This is not a correctness proof. Corpora are retained locally in `.testctl/fuzz/corpus` and are not published. |
| Concurrency, 100 additional release repetitions of the same two deep commands, outside testctl ([results](maintenance-2026-11-26/concurrency-100/results.txt), [script](maintenance-2026-11-26/concurrency-100/run.sh.txt)) | `9e792bc` + working tree | 200 invocations, **9300 passed**, 0 failed, 22:08-22:12 UTC; varied interleavings, not a proof of absence of races |
| Concurrency, 500 further release repetitions, same script ([results](maintenance-2026-11-26/concurrency-500/results.txt)) | `9e792bc` + working tree | 1000 invocations, **46500 passed**, 0 failed, 03:14-03:32 UTC |
| Stress deep: index scale/clone and concurrent snapshots, 10000 synthetic entries each, release | `9e792bc` + working tree | Passed, 37.5 s; functional bounds only, not a timed benchmark |
| full (run `20261126T194753Z-e9adae35`): `test.sh`, Rust 1.89 CI features and 1.92 all features, backend full, quick suite | `9e792bc` + working tree | `test.sh` **1504 passed**, 1 ignored (416 s); MSRV, backend and quick suite passed |
| Miri again within the same full run (inputs changed after `9e792bc`) | `9e792bc` + working tree | **128 passed tests** / 12 groups again, 0 failed, 1547 s |
| libFuzzer + ASAN, second round on the retained corpora (same full run) | `9e792bc` + working tree | knowledge 1200 s / 147965 executions (corpus 599 to 1004), lexical 1200 s / 109376 (1062 to 1715), pdf 1500 s / 53211 (2579 to 4107); no artifact. Cumulative recorded search: knowledge 2403 s, lexical 2406 s (+60 s short), pdf 3006 s, model_worker 902 s |
| quick: fmt, Clippy default/all features, tests default/all features, contracts, controller, backend | `9e792bc` + working tree | Default 266, all-feature **942** passed / 1 ignored, backend 14; all passed |

Each fuzz campaign started from an empty local corpus plus the versioned seeds in `tests/fixtures/fuzz-seeds/v0.5.2`. The deep campaigns ran one after another in one focus run with fixed libFuzzer seeds, all recorded in the receipt. LeakSanitizer was off. The standard library and C/system code are not fully instrumented. The PDF target runs the Rust parser under ASAN, unlike the earlier coverage-only PDF campaigns. These campaigns found nothing within their budgets. That is not a correctness proof.

**Failures retained.** The first `quick` run on the working tree failed `clippy-all` with `collapsible_match` in the new web reset code (run `20261126T193722Z-d7237845`). The code was corrected, and the next run passed (`20261126T193919Z-2c0832cf`). Both runs are in the receipt. While the test control was being built, its own unit tests found three controller defects: a counted echo line, invisible `.rs` changes outside the planned inputs, and a 0.0-second duration read as "unknown". They were fixed before the commit.

**Test-control defect found after these runs (working tree).** Inspecting the status after the full run showed that the three deep fuzz records of run `20261126T194753Z-e9adae35` (knowledge, lexical, pdf) list `uncommitted_inputs: []` and reference input manifests that no longer exist.

The cause: the deep stage stores every planned manifest up front, and `StatusStore.save()` garbage-collected the manifests of areas that had not executed yet. The Miri records survived only because all twelve groups share one manifest.

Those three fuzz campaigns actually ran in place on `9e792bc` plus the uncommitted `src/web/mod.rs` change. They are not clean-commit results, and their status rows are left unedited.

The fix keeps manifests stored by the running process out of garbage collection. Regression test `test_in_place_records_uncommitted_inputs_for_every_area_of_one_run` failed before the fix and passes after it, and all 23 controller tests pass.

**Measurements.**

- [IPC overhead](../measurements/ipc-overhead-0.6.2/README.md): the steady-state process-boundary cost is 1.3 to 3.2 ms per 20-call rerank. A buffered-stdin candidate was rejected by its frozen rule and reverted.
- [Lifecycle repeat on `9e792bc`](../measurements/native-lifecycle-0.6.2/README.md#additional-repeat-on-commit-9e792bc-2026-11-26): the recovery ratios are 0.986 and 1.000, and the worker growth gates passed.

- [Eight-pair unattended lifecycle repeat](../measurements/native-lifecycle-0.6.2/README.md#eight-pair-unattended-repeat-on-commit-9e792bc-2026-11-26-2129-2155-utc): under its frozen rule, the recovery gate **failed in 3 of 8 pairs** (1.169, 1.622, 2.516). The cost failure is therefore reproduced, not refuted. Across all serial pair comparisons, 4 of 11 have now failed.
- IPC set 3, with the unchanged baseline only: the per-case p95 gate failed in all four blocks. The median boundary cost stayed at 1.2 to 1.5 ms.
- A foreign Miri process from another session ran at about 96 % of one core on the host during **all** 2026-11-26 measurements. It was not quantified and was not stopped.

The original serial confirmation failure at **1.213** remains recorded and is not replaced. On this host, the worker's warm-recovery tail is not established as within 10 %.

**Working-tree maintenance.** The `src/web/mod.rs` server reset was consolidated into one `ServerReset` type, because the agent construction and the reset logic were repeated. A database that cannot be opened during a console reset is now logged, and the server stays stopped. Before this change, that case panicked. There is no automated test for the console reset path; it is covered by compilation, Clippy and the full-suite runs only. `scripts/testctl.py nightly --focus` selects one heavy area explicitly.

Because `src/` changed after `9e792bc`, the controller now marks the Miri and fuzz deep checks as due again, under its conservative crate-level rule. Their `9e792bc` results are not transferred to the working tree.
