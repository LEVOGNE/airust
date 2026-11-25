# AIRust session handoff

Read this first, then [WORK-PROGRAM-NEXT.json](WORK-PROGRAM-NEXT.json), [ROADMAP.md](../ROADMAP.md), [AGENTS.md](../AGENTS.md) and [CONTRIBUTING.md](../CONTRIBUTING.md). Communicate in German; keep code/docs in English. Resume observed work rather than restarting completed campaigns or asking the user to restate it.

## Current state: C1 validated, unpublished

The tested engineering source is **b875707dea03fa16a24eb7e0a489de9741d32f1b** on **codex/configuration-0.6.1**. Native/compiler/API/backend/advisory controls, 121 finite Miri cases, full selected fuzz scopes, quiet final-source measurements, clean package/extracted consumers and all five source-CI jobs passed. [The public closeout](validation-0.6.1/closeout.json) records observed results and honest limits. Independent human/readiness gates remain open. No 0.6.1 publication, master merge or tag is authorized.

This documentation closure follows the tested code commit. **First inspect local-lab/overnight-0.6.1-2026-11-24/final-closeout.json**: verify its final HEAD equals the current Git HEAD and its clean archive/CI/cleanup fields are actually complete. If it is absent/incomplete, finish only those post-documentation gates below; do not restart completed native/Miri/fuzz/benchmark campaigns. The original checkpoint 4cfbe9 remains in history.

Actual receipts in that laboratory: source-commit.json, clean-package-controls/report.json, ci-b875707dea03/report.json, fuzz-reconciled.json, miri-aggregate.json, final-source-controls/report.json and preservation-and-owned-service-closeout.json. Every retained initial failure remains separate. The task-owned model server/runner are stopped; original database/runtime/model bytes match after inference. Do not restart services automatically.

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

## Resume only actual remaining work

If final-closeout.json matches the current clean HEAD with all five final hosted checks and archive/preservation complete, C1 is closed. Proceed from WORK-PROGRAM-NEXT.json; do not repeat completed campaigns merely because a new session started.

If that final receipt is missing, finish the documentation commit, verify every frozen implementation/method/runtime input and packaged code byte against the tested engineering source, create a clean documentation-head package/dry-run and verify numerical checksums, then inspect the exact documentation-head hosted CI. Retain all observed failures/unavailable jobs. Complete the local final-closeout receipt and remove only recorded owned regenerable targets after retaining source/binaries/locks/packages/raw evidence. No master merge/tag/registry publication.

Next external work is the prepared independent onboarding/source-review/reproduction protocol; actual eligible human cases remain zero. The next conditional engineering step is D1: inventory existing typed diagnostics/stats/preflight and scope an observed failure/cost problem, with bounded output and measured overhead. Do not add a new telemetry/configuration framework or publish future versions blindly. Use the recorded native allocator, startup and model-answer failures as inputs; independent evidence cannot be manufactured.

The earlier six-hour request's minimum elapsed before the final engineering commit. It does not establish uninterrupted active-time accounting or start a new six-hour block for later sessions.

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
