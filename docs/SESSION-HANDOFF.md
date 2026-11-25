# AIRust session handoff

Read this first, then [WORK-PROGRAM-NEXT.json](WORK-PROGRAM-NEXT.json), [ROADMAP.md](../ROADMAP.md), [AGENTS.md](../AGENTS.md) and [CONTRIBUTING.md](../CONTRIBUTING.md). Communicate in German; keep code/docs in English. Resume observed work rather than restarting completed campaigns or asking the user to restate it.

## Current state: locally validated 0.6.1, post-commit gates next

As of **2026-11-25 07:24 UTC**, the unpublished candidate is on **codex/configuration-0.6.1**, with checkpoint **4cfbe9ade3056239c5c79331a567be8871cb0473** plus tested uncommitted hardening. All selected long fuzz scopes, finite Miri and final-source resource/request controls passed. The six-hour minimum elapsed before this closeout. The remaining steps are source commit, clean package/extracted consumers and exact-source hosted CI. No 0.6.1 publication, master merge or tag is authorized.

Read actual receipts in **local-lab/overnight-0.6.1-2026-11-24/**:

1. PROGRESS.json, fuzz-reconciled.json, miri-aggregate.json and final-source-controls/report.json. Final controls finished **07:16 UTC**; all required and optional declared gates passed, source inputs unchanged, normal snapshots clean.
2. preservation-and-owned-service-closeout.json: original database/runtime, **eleven model files / 19387512128 bytes** fully rehashed after inference, and 242 frozen source/method/runtime inputs match. The owned server PID 86497 / port 11437 and its recorded runner are **stopped**. No fuzz/model job remains; do not restart services/campaigns automatically.
3. clean-package-controls/report.json, ci-COMMITPREFIX/report.json and final-closeout.json when present. Missing/incomplete/failed receipts are not passes. Post-commit receipts must identify the actual source/package/CI commit.

The first final-controller attempt stopped before any archive/build/timing because the historically tracked original database was included in a Git-visible selection. The corrected controller excludes it explicitly; the failed attempt remains in final-source-controls-input-selection-failed/. The initial service-listener parser also stopped before any signal; corrected PID/start-time/port validation and shutdown passed. These are retained task-orchestration failures, not discarded numerical observations or product fixes.

## Completed engineering evidence

- C1 inventory and compatible profile settings: bounded outer batches/candidate prefixes, offline profile-check, legacy defaults/cancellation/failure policies retained. [Configuration](CONFIGURATION.md), [changes](CHANGES-0.6.1.md), [migration](MIGRATION-0.6.1.md).
- Private copy-on-write training snapshots, regular-file/byte/allocation checks, bounded retained JSON/UTF-8 diagnostics, opt-in verified native file snapshots and a model-free bounded MCP host. Use Rust composition/traits/private modules; no inheritance framework or duplicate configuration facade. [Architecture](ARCHITECTURE.md), [resource contracts](RESOURCE-LIMITS.md).
- Latest complete native matrix finished **05:31 UTC**, exit 0, 137.426 s: **266 default / 928 all-feature including 39 doctests / 40 isolated features**. Counts overlap. Four actual CLI corruption/bounds/preservation controls for tokenizer-input reconstruction are included. Receipts: full-native-with-padding-tests-result.json and native-with-padding-counts.json.
- Rust 1.89 provider-neutral/all targets, Rust 1.92 all features/all targets, five static API views against exact 0.6.0, and unchanged backend 0.3.5 regressions/C ABI/provenance/MSRV passed. Both-lock advisory audit has zero known vulnerabilities at its recorded RustSec revision, with maintenance notices retained. [Validation](validation-0.6.1/README.md).
- Miri: **121 passing tests in eleven observed groups**, miri-aggregate.json. Two initial macOS directory-sync attempts remain unsupported/failed; revised training scopes exclude that native-only successful-save path. Native durability controls pass. ONNX/Metal/HTTP/Tokio/PDF/large-scale execution remains outside this finite interpreter scope.
- Selected long fuzz scopes passed: lexical 5400 s / 509913 units and training 3600 s / 405218 units with ASAN; separate PDF recovery 5402.934 s / 224254 units, coverage-only, no artifacts. Original PDF exit after 3741 s at framework corpus opendir(EINTR) remains failed. Recovery keeps the identical target/guards, all original/generated seeds and reload=0; no stitched durations. fuzz-reconciled.json binds selected receipts and the retained original overall-false report. Never call PDF memory-sanitized.
- Actual local-model study, all 32 native options, six 150-query controls, 1920 checkpoint and 2560 hardened complete-MCP timed requests, 80 training-snapshot workers, 45 retained-error workers, small-host probes and three native-lifecycle cohorts are retained. Public numbers/figures and every regression: [primary measurements](measurements/profile-practical-0.6.0/README.md).
- [Final-source confirmation](measurements/configuration-0.6.1-final/README.md): sixteen COW workers; eight model-free workers  / 8000 timed + 256 warmup requests including 393216 terms, all memory budgets pass; complete-MCP p95 baseline 895.581 / 900.440 ms versus 8-thread/batch1/snapshot 229.610 / 228.034 ms, unchanged top six/non-logit data/known quotes. Optional same-binary 2-thread allocator comparison passes 10% p95-overhead gate, all 159 slower query means retained. Twelve new CSV files / 12073 numerical rows remain separate from primary cohorts.
- Static tokenizer check: 3000 pairs / 600 query-batch observations. Batch16 has 904224 padded positions versus 431059 real positions; non-padding tokens/masks are identical, and 41 pairs still truncate at 512. Locked public probe/source-coordinate reconstruction reproduces observations. This is structural evidence, not additional neural timing or independent quality review.

Production Rust/manifest/input **228 files remain frozen**. The latest freeze is source-freeze-public-padding-tests.json; only Python/method controls changed after the earlier accepted Rust freeze. Do not change measured inputs without recording the change and rerunning affected gates. The suspected cached MCP JSON copy and malformed-frame shutdown hypothesis were ruled out. Rejected experiments/failures remain retained; the proposed production runtime change was reverted.

## Finish the current authorized task

1. Complete final source/docs/private-public/numeric-integrity review and package-boundary listing. Rust/manifest/input 228 files and frozen methods remain unchanged; do not repeat native/Miri/fuzz/benchmark campaigns without an affected-input change. Commit tested source, preserving the checkpoint.
2. run_clean_package_controls.py requires that clean new commit: cargo package --locked, cargo publish --locked --dry-run, separate immutable extraction, three Rust-1.89 external consumers, packaged preflight/import/query/stdio and Rust-1.92 all-feature/all-target controls. It never uploads. Do not race Cargo's mutable verifier extraction with consumers.
3. Push only the candidate branch and inspect all five exact-source hosted jobs. observe_candidate_ci.py is a finite read-only current-task observer, not a recurring automation. Preserve unavailable/cancelled/failed jobs. A documentation closeout commit may explicitly reuse unchanged-code evidence; verify its clean archive/code identities and actual hosted conclusions separately.
4. Bind final HEAD/archive/CI/preservation/owned-service cleanup in final-closeout.json. Update this handoff and WORK-PROGRAM-NEXT.json from actual results. Preserve binaries/locks/methods/packages/raw failures before removing only recorded owned regenerable targets. Original data/laboratory evidence are not cleanup targets. No master merge/tag/0.6.1 publication is authorized.

The user's six-hour request began **2026-11-24 21:55:59 UTC**, minimum **2026-11-25 03:55:59 UTC  / 04:55:59 Europe/Berlin**. That minimum has elapsed; it does not itself pass a gate or establish uninterrupted active-time accounting. No new six-hour block or repeat campaign is implied for a later session.

## Known limits and external gates

Native models exceed 1 GiB here. The small model-free host is measured separately on an M5 Max / 64-GiB Mac; an actual 1-GiB laptop is untested. Worker RSS is not total-system/shared-GPU accounting. Training sharing shifts work to the first write; all writers still need copies. Byte/diagnostic limits do not eliminate all parsed-heap/transient/native allocations. Cooperative cancellation cannot interrupt an executing native call.

Repeated native loading failed the RSS-growth budget twice under the default macOS allocator. Diagnostics showed retained free regions/fragmentation and zero unreachable malloc allocations, which is not an absence-of-leaks proof. Optional unchanged-binary child launch MallocSpaceEfficient=1 passed lifecycle growth; a separate same-final-binary request experiment passed its 10% p95-overhead gate (ratios 0.983 / 0.997), with every slower query retained. No global allocator/environment mutation is adopted. E5 asset rejection is tested; unavailable E5 assets do not establish successful inference.

**Zero of the required fifty genuine independently reviewed pilot questions are established.** Frozen 150 reference-v1 questions are assistant-authored. Three historical conversation replays plus one current request are exposed local probes; sixteen local-LLM attempts include an invalid quotation and a confident incomplete historical-roadmap answer. Valid quotations do not establish semantic completeness. Keep these actual questions/answers private; they are not independent customer traffic or human review.

Independent timed onboarding, clean-machine/external-client usage and independent benchmark/alternatives reproduction remain open. Use [INDEPENDENT-VALIDATION.md](INDEPENDENT-VALIDATION.md) and [PILOT.md](PILOT.md). Do not contact reviewers without authorization or fabricate participants. After C1 closeout, conditional diagnostics/observability/resources/security work starts from observed failures and existing APIs; version numbers do not authorize blind implementation/publication.

## Immutable released baseline

**0.6.0 is released and verified**: master/source **8c098f15a39f6085f748c56ac52e5737abdff423**, annotated tag object **b80140d414a030585809da3ac98a2483f547e47f**, crates.io SHA-256 **5de243422a16f5bb6e8ecc425c657fc5104156d1e6324a8e22593afc445b104b**.

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
