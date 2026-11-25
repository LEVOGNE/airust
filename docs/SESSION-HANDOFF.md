# AIRust session handoff

Read this first, then [WORK-PROGRAM-NEXT.json](WORK-PROGRAM-NEXT.json), [ROADMAP.md](../ROADMAP.md), [AGENTS.md](../AGENTS.md) and [CONTRIBUTING.md](../CONTRIBUTING.md). Communicate in German; keep repository code/docs in English.

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

## Finish only pending publication stages

Read release PROGRESS.json and exact receipts before running a step. Prepare/test a clean release commit, inspect/dry-run the exact crate and immutable extracted consumers, fast-forward master, verify actual master/tag CI, publish one exact crate, compare anonymous registry/archive bytes/VCS/checksum, run real registry consumers and scoped installed MCP CLI, publish the GitHub payload, anonymously download/verify every asset and observe versioned docs.rs. Retain failures/unavailable jobs. Never interpret a queued/partial run or draft as a pass. Do not modify backend 0.3.5 or old release tags.

Preserve methods/locks/binaries/packages/logs/corpora before removing only known owned regenerable targets. Original knowledge, user databases/workspaces, models and retained evidence are not build artifacts. No unsolicited reviewer messages are authorized. After the final publication receipt is complete, the next conditional engineering step is **D1**: inventory current typed diagnostics/stats/preflight and scope an observed failure/cost problem with bounded output and measured overhead. Independent onboarding, genuine source-reviewed cases and external reproduction still need actual inputs. Do not restart unchanged C1 campaigns or implement/publish all roadmap versions blindly.

## Known limits and external gates

Native models exceed 1 GiB here. The small model-free host is measured separately on an M5 Max / 64-GiB Mac; an actual 1-GiB laptop is untested. Worker RSS is not total-system/shared-GPU accounting. Training sharing shifts work to the first write; all writers still need copies. Byte/diagnostic limits do not eliminate all parsed-heap/transient/native allocations. Cooperative cancellation cannot interrupt an executing native call.

Repeated native loading failed the RSS-growth budget twice under the default macOS allocator. Diagnostics showed retained free regions/fragmentation and zero unreachable malloc allocations, which is not an absence-of-leaks proof. Optional unchanged-binary child launch MallocSpaceEfficient=1 passed lifecycle growth; a separate same-final-binary request experiment passed its 10% p95-overhead gate (ratios 0.983 / 0.997), with every slower query retained. No global allocator/environment mutation is adopted. E5 asset rejection is tested; unavailable E5 assets do not establish successful inference.

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
