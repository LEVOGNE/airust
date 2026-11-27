# AIRust session handoff

Read this first, then [WORK-PROGRAM-NEXT.json](WORK-PROGRAM-NEXT.json), [ROADMAP.md](../ROADMAP.md), [AGENTS.md](../AGENTS.md) and [CONTRIBUTING.md](../CONTRIBUTING.md). Communicate in German; keep repository code/docs in English.

## Alternating agents (Claude Code and ChatGPT/Codex)

**Release 0.6.2 is published and verified (2026-11-27).**

- Tag `v0.6.2` points to `5b69258`.
- The crates.io checksum is `549117704f977a45b7b34ecda49f075295cfd414ecd6e67c26e52124aab8bf0c`, and the anonymous download is byte-identical.
- Branch, master and tag CI each passed 5 of 5 jobs.
- The GitHub Release assets and docs.rs were verified.
- The complete receipt is `local-lab/release-0.6.2-2026-11-27/final-closeout.json`, and the public manifest is on the [release page](https://github.com/LEVOGNE/airust/releases/tag/v0.6.2).
- Do not republish or move the tag.

**Next work, if the user asks for it:**

1. Investigate the process-mode warm-recovery latency failure, which failed 4 of 11 comparisons. Freeze a policy first and measure only on a host checked for foreign load.
2. Take the open review decisions listed below.

Miri, fuzz and concurrency become due again from 2026-12-04 (`testctl status`).

**0.6.3 operational robustness (2026-11-27 evening, uncommitted on top of local commit `0eceabc`, which is not pushed):**

- The criteria are frozen in `docs/validation-0.6.3/CRITERIA.md`, with amendment 1 recorded before any counted run. Results are in `docs/validation-0.6.3/RESULTS.md`.
- **R1, the 3-hour soak, failed on R1.6 only.** The p95 drift was 1.32, against a limit of 1.10, under heavy foreign load. R1.1 to R1.5 passed: 3897 cycles, 1169 injected faults, 0 leftovers, RSS range 0.81 MiB. Do not relabel it. A quiet-host repeat would be additional evidence only.
- **R2.** `ProcessPool` with `RerankError::Busy` passes its mock test. Real-model throughput was 154 requests/min with 1 worker and 242 with 2.
- **R3.** The `native-worker-linux` CI job is prepared but has never run. It needs a push, which requires user approval.
- The user's next separate package is a local-LLM answer comparison via Ollama: Qwen3.8:27b on the 150 authored questions with deterministic checks, labelled as authored rather than independent.

**Session 2026-11-27 afternoon (committed locally as `0eceabc`, not pushed):**

- The user decided all six open review items, see CHANGELOG `Unreleased`. These are implemented with tests: admin mode requires a token, the worker environment is reduced, `ProcessOptions::with_runtime_identity` is added, web field limits with HTTP 413, indexes are created only for new databases, and the duplicated byte-label and SHA-256 checks are shared in `text::Identifier`.
- I recommended counting labels in characters, but implemented the byte count instead. Characters would loosen the memory bounds and break the compatibility of saved receipts. Tell the user if this needs revisiting.
- CSP is only partly done: a `Content-Security-Policy-Report-Only` header is sent. Enforcement needs a rewrite of `src/web/static/index.html`, which has an inline script with 110 free functions, alert/confirm/prompt and 79 inline handlers. The user's design guard rightly blocks moving that script unchanged. This is a separate user decision.
- Latency: an eight-pair monitored repeat on a user-freed host passed 1.10 in all 8 pairs (0.981 to 1.025). The frozen quiet-host claim is still not made, because the monitor saw brief foreign activity in every pair. The total is now 4 failures in 19 serial comparisons. See `docs/measurements/native-lifecycle-0.6.2/README.md`.

**Unattended block on 2026-11-27, 10:02 to 11:47 UTC (uncommitted, awaiting user review):**

- `testctl status` now reports never-run trigger-only `short` checks as "on matching risk trigger in quick" instead of "now". A regression test was added, 24/24 controller tests pass, and the CHANGELOG has an `Unreleased` entry.
- A fourth extended ASAN fuzz round ran outside testctl, 1200 s per target on the retained corpora: knowledge 151161 executions, lexical 117571, model_worker 117183122, pdf 29371. There was no artifact. Raw runs are in `local-lab/bin-weg-2026-11-27/fuzz-extended-4/`.
- Cumulative search since 2026-11-26 is about 140 to 186 minutes per target. Nothing was found within these budgets, which is not a correctness proof.
- No latency measurement was taken, because another project's compilation was running on the host. The latency item stays open.

The user alternates between Claude Code and ChatGPT/Codex on this repository. Whoever finishes a block updates this section. Last update: Claude Code, unattended maintenance 2026-11-26 20:13 to 2026-11-27 05:00 UTC.

- **Git state:**
  - `master` and `codex/stabilize-native-model-lifecycle` contain everything up to the release commit `5b69258` (tag `v0.6.2`). This handoff update follows on `master` only.
  - There are no uncommitted changes.
- **Test control:**
  - Use `uv run --no-project scripts/testctl.py status|quick|nightly|full` rather than ad hoc command lists.
  - Only one heavy run at a time; `.testctl/lock` enforces this.
  - The status in `.testctl/` is local and gitignored.
  - The exported receipt is in `docs/validation-0.6.2/maintenance-2026-11-26/testctl-receipt.json`.
- **Do not repeat:** the 2026-11-26/27 Miri (2 x 128 tests), fuzz, full, concurrency and stress runs, unless inputs change. Check `testctl status` first.
  - Fuzz: cumulative ASAN search of 105 to 140 min per target, 0.29 to 614 million executions, no artifact.
  - Concurrency: 25 + 100 + 500 release repetitions, 0 failures.
  - Further fuzzing should continue on the retained corpora in `.testctl/fuzz/corpus` rather than starting fresh.
- **Measurement caution:**
  - Another Claude Code session runs a long Miri job for an unrelated project. It has been at about 96 % of one core since 2026-11-26 00:17 UTC.
  - Check `ps` for foreign load before any timing measurement, and record it.
  - Do not stop other sessions' processes.
- **Honesty state:**
  - The warm-recovery 1.10 gate failed in 4 of 11 serial pair comparisons, including the original 1.213. It remains an open cost failure, not a pass.
  - IPC set 3 failed its p95 gate in all blocks under load.
  - Never re-run until a preferred result appears. Freeze a policy before any new measurement.
- **Open user decisions:** commit and push of the uncommitted work; CSP and inline handlers; admin mode without a token; web size limits; DB indexes (these change the `airust.db` checksum); duplicate hashes and labels; worker environment and runtime hash; a test scheduler (launchd or GitHub Action; none is installed).
- **Language:** talk to the user in German; repository text and code stay in English.

## Current engineering candidate: ML1, unreleased 0.6.2

**Update 2026-11-26 (supersedes the push status in the next paragraph):**

- The branch was pushed at `1cbd70e`, and hosted CI run `45669021076` passed.
- Local commit `9e792bc` holds the source review fixes and the resource-aware test control ([TESTPLAN.md](TESTPLAN.md)). It is **not pushed**.
- An unattended maintenance session then left uncommitted working-tree changes: the `ServerReset` consolidation in `src/web/mod.rs`, `testctl --focus`, the [IPC overhead record](measurements/ipc-overhead-0.6.2/README.md), a lifecycle repeat and the [maintenance validation section](validation-0.6.2/README.md#post-closeout-review-and-maintenance-2026-11-26) with its testctl receipt.
- Review and commit these first. Run `uv run --no-project scripts/testctl.py status` to see what is due.
- The 1.213 serial failure remains recorded.
- Open user decisions from the review: CSP and inline handlers, admin mode without a token, web size limits, database indexes, duplicate hashes and labels, worker environment and runtime hash, and a test scheduler (none is installed).

**Local implementation and clean-package/application closeout are complete at `a5725fcc0614982b016216679b7891546d0384c2`. The one remaining external step is blocked:** automatic approval review rejected the push of `codex/stabilize-native-model-lifecycle` to `https://github.com/LEVOGNE/airust.git` before execution. The candidate has not been uploaded or checked by hosted CI. Do not retry or use another transfer path without new explicit user approval for this concrete branch push. Read the final local receipt for the latest documentation-only commit/archive binding.

The user explicitly requested point 1 on 2026-11-25, then asked to finish everything without questions: reproduce/fix repeated local-model memory growth, measure before/after, test and document. Branch **codex/stabilize-native-model-lifecycle**. A new-version master merge/tag/GitHub Release/crates.io publication is not part of this instruction. The published release remains 0.6.1.

Start with **local-lab/native-lifecycle-2026-11-25/PROGRESS.json** and **final-closeout.json**, then [0.6.2 validation](validation-0.6.2/README.md), [API/ownership](LOCAL-MODEL-PROCESSES.md) and [all measurements](measurements/native-lifecycle-0.6.2/README.md). If the final receipt is complete, preserve the implementation/evidence and continue at actual external Q1/DX1 inputs or a separately scoped observed D1 problem. If incomplete, finish only the recorded clean-source package/CI/commit closing steps; do not restart unchanged campaigns or invent a release. Check current Git/remote receipts before repeating work.

The direct published loader/runtime remain byte-identical and still retain native allocator RSS. Three allocation ablations failed and are archived. The selected new `ProcessReranker` reuses the pinned scorer in one trusted process, requires verified private file snapshots, bounds transport/input/diagnostics and stops/reaps before releasing its parent-owned snapshot guard. `reranker.isolated` is opt-in and requires an explicit snapshot parent; native pilot builds use `pilot,local-reranking`. Model-free pilot/preflight builds stay independent of native inference.

There are 314 measured cycles across thirteen controls, including every failure. Three selected thirty-cycle post-warm parent-RSS ranges are **0.328 / 0.406 / 0.375 MiB**, with zero surviving model workers; the small unloaded test parent is about 4.2 MiB, not a full application footprint. Active model RSS exceeds 1 GiB. The serial warm-p95 confirmation **failed at 1.213**; the later alternating diagnostic 0.986 does not erase it. The memory-backed worker prototype also failed its cost gate and was removed. All raw samples, 309 slower comparison observations, score bits and reproducible negative patches remain public without private text.

Local checks passed: full test.sh (266 default / 940 all-feature) before the last crash test; final 941 all-feature including 40 doctests; latest 12 worker controls including 12 MiB escaped frames; 40 features; Rust 1.89 neutral and 1.92 all-target; backend 14 / C ABI/provenance/MSRV; five API views versus exact v0.6.1. Miri aggregate is 128 tests / 12 observed groups with the original worker-test compile failure retained and explicit successful replacement/final-example receipts. Fresh 600-second ASAN protocol fuzz executed 45952362 inputs with no artifact; FFInative/subprocess/leak-instrumentation exclusions remain explicit. Three RustSec locks have no listed vulnerability and 3 / 4 / 2 maintenance warnings; yanks are outside that check.

Separate extracted-package applications pass: model-free Rust 1.89 core, and Rust 1.92 standalone native helper with eight actual cycles / 160 baseline-exact results, all workers reaped/private copies removed. These are functional automated consumers, not timed benchmarks or independent human onboarding. Eleven retained model files / 19387512128 bytes, runtime, original database, backend, knowledge and frozen datasets matched identities. No model/runtime download or model-service startup occurred; user processes were not signalled. Preserve every model/database/evidence file and only remove marked task-owned regenerable targets after preserving artifacts.

Zero of fifty genuine independently source-reviewed cases is established. The four synthetic resource cases, four private exposed replay inputs and 150 authored reference cases never count as independent traffic/review. Actual E5 inference, Windows/Intel native runs, actual 1-GiB devices, hard OS quotas and unique system/GPU accounting remain unestablished. Private replay text stays outside Git. The original 0.6.1 macOS queue/capacity observations are separate history.

## Current release: 0.6.1, receipt-driven continuation

On **2026-11-25**, the user explicitly authorized merging the validated candidate into master and publishing **AIRust 0.6.1** with an annotated tag, GitHub Release and crates.io package, including all closing checks without further questions. This supersedes the earlier candidate-only authorization. Later versions still need their own release instruction.

First inspect **local-lab/release-0.6.1-2026-11-25/final-closeout.json**, **PROGRESS.json** and the [public release manifest](https://github.com/LEVOGNE/airust/releases/download/v0.6.1/release-manifest.json). The release source, annotated tag target, packaged Git commit, registry checksum and GitHub crate must agree. A completed receipt means preserve the release and proceed to D1; do not republish or move a tag. An absent/incomplete receipt means resume only its actual pending stage. [RELEASE.md](validation-0.6.1/RELEASE.md) defines the checks and limits. The manifest and separate anonymous public-verification receipt are authoritative publication evidence; a prepared version header is not itself a publication receipt.

The validated candidate is **2ef04aa7c005b5a6f315d5ebe649358a96fcd199**, with engineering implementation **b875707dea03fa16a24eb7e0a489de9741d32f1b** and earlier checkpoint 4cfbe9. Its complete receipt remains at local-lab/overnight-0.6.1-2026-11-24/final-closeout.json. Both candidate commits passed all five hosted jobs; clean archives and external consumers passed. Do not overwrite that historical evidence.

Release preparation changes documentation/version examples and continuation metadata only. Verify all 228 Rust/manifest/data files and nine measured method inputs against source-freeze-public-padding-tests.json. Model manifest, MCP schema and embedded static HTML also remain unchanged. README Rustdoc and STARTUP web-help text deliberately change for release usage; these are separately recorded documentation differences. The release repeats complete native/feature/compiler/backend/package/consumer/current-advisory controls. Existing finite Miri/fuzz/quiet benchmark results are reused with exact executable-input proof and explicit exclusions, not described as new runs.

## Completed candidate engineering scope

- Native 266 default  / 928 all-feature including 39 doctests, 40 isolated features; strict Clippy/rustdoc/real CLI/Python boundaries. Counts overlap. Rust 1.89 neutral/all targets, Rust 1.92 all-feature/all targets, five static API views versus exact 0.6.0, unchanged backend 0.3.5 regressions/C ABI/provenance/MSRV passed.
- Finite Miri: 121 tests  / 11 observed groups on dated nightly 1.102.0-nightly5f4effd579, TreeBorrows. Initial unsupported macOS directory-sync attempts remain failed, successful durability checked natively. Native FFI/HTTP/Tokio/ONNX/Metal/PDF and declared large/async scopes are excluded.
- Full selected fuzz: lexical 90 min/509913 units ASAN, training 60 min/405218 units ASAN, separate PDF 90 min/224254 units coverage-only. The original PDF 62 min corpus opendir(EINTR) exit remains failed. Recovery keeps the same binary/guards, copies every original/generated seed, disables unnecessary single-worker reload, and never stitches partial durations. Leak detection is off; C/std/native instrumentation incomplete.
- Final-source request/resource confirmation: 16 COW workers; 8 model-free workers  / 8000 timed + 256 warmup requests including 393216 terms; 600 timed + 600 initial paired MCP requests. Source/rank/known-quote controls match. Same-binary optional allocator comparison passes its declared 10% p95 cost gate; all 159 slower query means retained. Source/model/lock/binary identities and all primary regressions are public CSV with checksums.
- Actual local LLM attempts include a generated quotation mismatch and a confident incomplete historical-roadmap answer. The four exposed conversation inputs and 150 authored regression cases do not establish independent customer quality. [Primary study](measurements/profile-practical-0.6.0/README.md), [final cohort](measurements/configuration-0.6.1-final/README.md), [historical candidate closeout](validation-0.6.1/closeout.json).
- Original database/runtime and eleven model files  / 19387512128 bytes matched after inference. The task-owned Ollama server/runner are stopped. No automatic service/model restart. Thirteen owned Cargo targets were removed after preserving artifacts; opaque API/Miri/rustdoc outputs and all raw evidence remain retained. Existing ASAN runtime bytes were copied and three one-input replay/link controls passed after cleanup; these are not new campaigns.

## Historical 0.6.1 publication procedure — completed

The 0.6.1 final publication receipt is complete; the following procedure is historical. Do not repeat it for ML1. Read release PROGRESS.json and exact receipts before running a step. Prepare/test a clean release commit, inspect/dry-run the exact crate and immutable extracted consumers, fast-forward master, verify actual master/tag CI, publish one exact crate, compare anonymous registry/archive bytes/VCS/checksum, run real registry consumers and scoped installed MCP CLI, publish the GitHub payload, anonymously download/verify every asset and observe versioned docs.rs. Retain failures/unavailable jobs. Never interpret a queued/partial run or draft as a pass. Do not modify backend 0.3.5 or old release tags.

Preserve methods/locks/binaries/packages/logs/corpora before removing only known owned regenerable targets. Original knowledge, user databases/workspaces, models and retained evidence are not build artifacts. No unsolicited reviewer messages are authorized. After the final publication receipt is complete, the next conditional engineering step is **D1**: inventory current typed diagnostics/stats/preflight and scope an observed failure/cost problem with bounded output and measured overhead. Independent onboarding, genuine source-reviewed cases and external reproduction still need actual inputs. Do not restart unchanged C1 campaigns or implement/publish all roadmap versions blindly.

## Known limits and external gates

Native models exceed 1 GiB here. The small model-free host is measured separately on an M5 Max / 64-GiB Mac; an actual 1-GiB laptop is untested. Worker RSS is not total-system/shared-GPU accounting. Training sharing shifts work to the first write; all writers still need copies. Byte/diagnostic limits do not eliminate all parsed-heap/transient/native allocations. Cooperative cancellation cannot interrupt an executing native call.

Historical 0.6.1 direct native loading failed the RSS-growth budget twice under the default macOS allocator. The new explicit process mode has its own separate 0.6.2 observations above. Diagnostics showed retained free regions/fragmentation and zero unreachable malloc allocations, which is not an absence-of-leaks proof. Optional unchanged-binary child launch MallocSpaceEfficient=1 passed lifecycle growth; a separate same-final-binary request experiment passed its 10% p95-overhead gate (ratios 0.983 / 0.997), with every slower query retained. No global allocator/environment mutation is adopted. E5 asset rejection is tested; unavailable E5 assets do not establish successful inference.

**Zero of the required fifty genuine independently reviewed pilot questions are established.** Frozen 150 reference-v1 questions are assistant-authored. Three historical conversation replays plus one current request are exposed local probes; sixteen local-LLM attempts include an invalid quotation and a confident incomplete historical-roadmap answer. Valid quotations do not establish semantic completeness. Keep these actual questions/answers private; they are not independent customer traffic or human review.

Independent timed onboarding, clean-machine/external-client usage and independent benchmark/alternatives reproduction remain open. Use [INDEPENDENT-VALIDATION.md](INDEPENDENT-VALIDATION.md) and [PILOT.md](PILOT.md). Do not contact reviewers without authorization or fabricate participants. After C1 closeout, conditional diagnostics/observability/resources/security work starts from observed failures and existing APIs; version numbers do not authorize blind implementation/publication.

## Previous immutable release: 0.6.0

**0.6.0 remains released and verified**: historical source **8c098f15a39f6085f748c56ac52e5737abdff423**, annotated tag object **b80140d414a030585809da3ac98a2483f547e47f**, crates.io SHA-256 **5de243422a16f5bb6e8ecc425c657fc5104156d1e6324a8e22593afc445b104b**.

Authoritative [release assets](https://github.com/LEVOGNE/airust/releases/tag/v0.6.0) and local-lab/release-0.6.0-2026-11-24/final-closeout.json bind package/tag/registry, all five exact-source CI jobs and public-asset verification. Do not repeat publication, move tags or reinterpret historical RESUME-0.6.0.md as a current pause. The 0.5.4 pilot was integrated into 0.6.0; patch counts are not test counts.

## Preservation and working rules

- Memory safety/performance, stability, robustness and honesty are release criteria. Preserve owned-library forbid(unsafe_code), empty default features, original data on failure, typed errors, scopes/revisions and retained readers. Plan about 70% quality/docs/integration/validation and 30% justified capabilities; do not claim measured time accounting.
- Preserve original knowledge/, airust.db, sources, workspaces, frozen tests/fixtures/reference-v1/, historical releases/corpora and raw evidence. Original database SHA-256: **8048756bb34ea465ed4bdbc90537102e83e9690d360bba6c9d951b3b285bfbe8**.
- Keep local-lab/pilot-0.6.0/, release-0.6.0-2026-11-24/, profile-practical-0.6.0-2026-11-24/ and overnight-0.6.1-2026-11-24/. Never publish private replayed questions, LLM answers, service logs/environment/inventory or engineering-evidence-private.tar.gz.
- Existing MiniLM assets and ONNX runtime remain under local-lab/reference-workflow/models/mmarco-minilm-v2/ and local-lab/pdf-stability/python-deps-0315/onnxruntime/capi/. Model/runtime preservation receipts are in the overnight lab; no model downloads occurred.
- Backend **0.3.5** and its separate source/lock stay untouched. Use cargo fmt --package airust, never --all; backend provenance/regressions/MSRV have separate gates. Audit both locks and retain maintenance notices.
- Use dedicated owned target directories with debug info/incremental builds disabled. Reuse unchanged measurements only after verifying source/inputs; state scopes/exclusions and unavailable observations. Preserve evidence before build cleanup.
- Detailed public measurement JSON stays in Git and is intentionally excluded from crates; packaged CSV/figures/SHA256SUMS remain usable.
- Hosted CI uses Ubuntu and explicit macos-15 ARM64 with two Cargo jobs / a 45-minute macOS bound. Earlier capacity cancellations/Intel-unavailable attempts remain unavailable evidence, not Windows/Intel validation.

Inspect git status --short --branch and git log -6 --oneline before edits. No force/reset or cleanup of unknown changes. Update this handoff/work program after observed closeout so the next session starts at the actual remaining task.
