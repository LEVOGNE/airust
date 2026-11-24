# AIRust 0.6.0 engineering validation

The source milestone implements the local pilot originally planned for 0.5.4 and explicit live CLI/MCP inference profiles. It is prepared on `codex/pilot-0.6.0`; no master merge, 0.6.0 tag, GitHub release or registry publication is asserted. The last verified published baseline remains `705803e34382be856c288c7c5c2eeb2507217330` / 0.5.3.

The engineering checks and empirical real-data readiness are separate. The new collection/validation/freeze tools work, but **zero genuine independently reviewed pilot questions have been established** against the required minimum of 50. Three historical user questions were locally replayed, explicitly unreviewed and excluded from that gate. No new independent quality recommendation is made.

## Executed checks

| Check | Observed result |
| --- | --- |
| Complete local `./test.sh`, Rust 1.98.0 | 253 standard checks; 914 all-feature checks including 39 doctests; strict Clippy/docs, examples, feature interactions and package boundary pass. |
| Feature isolation | All 40 declared entries checked separately with all targets. |
| Reference workspace Rust tests | 13 pass, including import/source identity, immutable capture/export, malformed/stale UTF-8 evidence, paragraph expansion and encoded JSON budgets. |
| Real pilot CLI/stdio Python workflow | 6 pass, including independent document framing/source checks and declared-role freeze/split controls. Synthetic declarations remain test fixtures. |
| Synthetic loopback model protocol/failure suite | 7 pass on the release binary, including malformed/digest/byte/dimension/zero/nonfinite/count responses, strict/fallback behavior, cancellation, deadline and recovery. No model-quality claim follows from this fixture. |
| Rust 1.89 provider-neutral all-target matrix | Pass, including `pilot`. |
| Rust 1.92 all-feature/all-target matrix | Pass. |
| Rust 1.89 independent packaged-crate consumer | Pass for core/Unicode, retained clones, rejected-update rollback, JSON/document restoration, typed cancellation, five versioned MCP schemas and the new application-owned search hook. |
| Maintained backend | 14 owned regressions, provenance, C ABI check and Rust 1.89 check pass. Backend source/lock are unchanged. |
| Packaging and registry publish dry run | Pass. The initial offline dry run could not perform its registry check; the repeated online `--dry-run` passed and explicitly aborted upload. This is not publication. |
| Dependency audit | cargo-audit 0.23.0, RustSec `1d328b6bb9f2f350bf65c1c81ab68781cb9e018a`: zero known vulnerabilities in both locks. Three root/four backend maintenance notices remain; none suppressed. |

The full local script's initial profile invocation ran six cases; the release-binary follow-up adds the seventh cancellation case and validates final request-control release. The complete platform CI subsequently runs all seven cases. Source/model/compiler/lock identities, exact command logs and raw outputs remain under ignored `local-lab/pilot-0.6.0/`.

The five-job [runtime-source CI](https://github.com/LEVOGNE/airust/actions/runs/45418692340) passed at `857758a`: Linux, macOS, Rust 1.89, Rust 1.92 and the maintained backend. Later code commits concern measurement/reporting tools; the final code-CI result is recorded separately in the local closeout record. Existing source/tests must be compared before reusing a prior run as evidence for another commit.

## Actual finite Miri scope

Nightly 1.102.0-nightly / `5f4effd579` (2026-11-13), Tree Borrows, passed **86 tests**: 41 index tests, 25 core tests, one existing `cfg(miri)` reduced mutation-state test, four synchronous MCP reply tests, three new backend source/cancellation tests and 12 small workspace/pilot tests. Exact commands, flags, log hashes and exclusions are retained in [the interpreter report](../measurements/pilot-0.6.0/miri-report.json) and [method provenance](../measurements/pilot-0.6.0/miri-provenance.json).

All jobs use `-Zmiri-tree-borrows -Zmiri-no-extra-rounding-error` and one test thread. The small filesystem example tests additionally use `-Zmiri-disable-isolation`. Alias/memory checking remains enabled. The helper now keeps partial progress explicitly non-passing until every planned job finishes and accepts a dated installed `--toolchain` for reproduction. The recorded 86-test run used the archived helper source at `cad0409` with the same default test commands/flags.

Four large index cases remain native-only: `empty_vocabulary_releases_its_hash_allocation_after_removal`, `sparse_vocabulary_shrinks_without_changing_retained_readers`, `token_ties_keep_id_order_and_character_scores_for_long_text` and `token_top_k_preserves_full_comparison_scores_and_unicode_explanations`. Two asynchronous MCP shutdown cases remain native-only: `final_reply_work_keeps_admission_and_shutdown_until_cancelled_work_returns` and `shutdown_does_not_wait_for_other_server_clones_worker_permits`. They pass in the native suite. The full frozen-corpus parity example also runs natively; its smaller format/source checks run under Miri.

HTTP transport, actual Tokio/ONNX/Metal/PDF native execution and million-entry workloads are outside this interpreter scope. Real loopback integration, actual model-backed requests, native backend regressions and finite-scale controls provide separate evidence. Neither this finite Miri scope nor native success proves absence of undefined behavior, bugs or leaks in every dependency/environment.

## Actual measurements and retained failures

The [measurement report](../measurements/pilot-0.6.0/README.md) records exact Core compatibility, two paired query/RSS campaigns, two full MCP campaigns with matched feature graphs, two 10k/100k/1m/retained-clone campaigns and all three actual live inference profiles. Numerical budgets were fixed before execution; all completed campaign resource/regression gates pass. Every individual slower query and known-quote loss remains.

The initial MCP exact-data comparison failed because the external baseline omitted the example's `serde_json/float_roundtrip` feature unification. The corrected build repeats the full comparison without relaxing expected data or budgets; the failed trial and 276 float-only differences are retained. Full request time includes query embedding and final framing; startup and separately sampled model-service memory are not hidden inside a retrieval-only number. Unique GPU/shared physical memory was unavailable and is not invented.

Original knowledge and `airust.db` are preserved; database SHA-256 stayed `8048756bb34ea465ed4bdbc90537102e83e9690d360bba6c9d951b3b285bfbe8`. Private question replays remain local. The task-owned loopback model service was stopped after measurement. Rebuildable target directories are separate from retained evidence/model assets and may be cleaned only after their binaries, locks and results are archived.

Read [the machine validation record](validation.json), [work program](../WORK-PROGRAM-0.6.0.json), [pilot guide](../PILOT.md), [migration](../MIGRATION-0.6.0.md) and [maintenance decisions](../DEPENDENCY-DECISIONS-0.6.0.md). A new publication still needs explicit release authorization and final source/tag/package/registry verification.


## Reproduce with explicit inputs

Run `./test.sh` first with Rust 1.98 (or the documented minimum toolchains). `python3 -B scripts/run_miri.py NEW_OUTPUT --toolchain INSTALLED_DATED_NIGHTLY` repeats the finite interpreter matrix and records its actual version/flags; a different nightly is a new run, not the recorded one.

Validate/export the unchanged dataset with `python3 -B scripts/reference_dataset.py --output NEW_DATASET`. Build the external Core control with `python3 -B scripts/build_index_control.py NEW_CONTROLS --baseline-version 0.5.3 --offline`, then invoke `scripts/verify_index_refactor.py --binary-directory NEW_CONTROLS --dataset NEW_DATASET --output NEW_RESULTS`. Use `--source examples/stress_index.rs` in a separate control build for `scripts/benchmark_scale.py`. Reuse the retained exact [baseline](core-baseline.lock)/[candidate](core-candidate.lock) locks and the recorded compiler before reporting a comparable build; a newly resolved dependency graph is a separate configuration.

For the complete MCP baseline, take `examples/reference_workspace.rs` and `examples/reference_support/mod.rs` from the checksum-verified published 0.5.3 archive (`d558ec5a4b44bf7bd729308a3c1596d0d16022f7a93449f1b1e5b2106d6305e8`). Copy them into a standalone consumer as `src/main.rs` and `src/reference_support/mod.rs`, use [the matching-feature manifest](mcp-baseline-consumer.toml) as Cargo.toml and [its lock](mcp-baseline.lock) as Cargo.lock, then `cargo build --release --locked`. This uses the registry library and published example source, not a modified baseline implementation. Compare it with the current source-built `--features mcp` example using `scripts/benchmark_pilot.py compatibility --binary CANDIDATE --baseline BASELINE --workspace WORKSPACE --output NEW_RESULTS`.

Prepare actual profile assets/configs and the workspace/cache as documented in [PILOT.md](../PILOT.md), build with `--release --locked --features pilot,local-reranking`, then run `scripts/benchmark_pilot.py profiles --binary BINARY --workspace WORKSPACE --profiles CONFIG_DIRECTORY --service-pid OWNED_PID --service-url LOOPBACK_URL --output NEW_RESULTS`. The directory contains `bm25.json`, `hybrid.json`, `hybrid_rerank.json` and the compatible vectors file. Own and record the local service process, observe memory separately and stop it afterward. Do not run compilers/interpreters concurrently with native timings. Preserve every raw result, failure and source/model/lock/method identity before cleaning only owned build artifacts.
