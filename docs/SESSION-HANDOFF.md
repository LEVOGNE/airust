# AIRust session handoff

Read this file first when resuming roadmap/release work. Then read [the roadmap](../ROADMAP.md), [WORK-PROGRAM-NEXT.json](WORK-PROGRAM-NEXT.json) and [CONTRIBUTING.md](../CONTRIBUTING.md). Communicate with the user in German; keep repository code/docs in English. Do not restart completed benchmark campaigns or ask the user to restate the project.

## Current work: practical profiles before 0.6.1

The user first authorized C1 configuration work, then explicitly prioritized testing the three profiles with an existing local answer LLM and actual user questions before implementing 0.6.1. They also requested a substantial, measured reranking performance investigation. Follow that order. Branch: `codex/configuration-0.6.1`; library/example implementation is initially unchanged from published 0.6.0. Do not infer authorization to publish 0.6.1.

Live evidence and continuation: `local-lab/profile-practical-0.6.0-2026-11-24/PROGRESS.json`, `frozen-policy.json`, executed Python methods and raw baseline results. The new immutable workload has 18 current published-source documents / 729 passages, three verbatim earlier user-question replays and one current user improvement request. This is a small nonrepresentative exposed local workload, not four independent customer questions or human judgments. Original replays/frozen reference-v1 and release evidence remain unchanged.

The fresh baseline binary is compiled from exact source `8c098f15a39f6085f748c56ac52e5737abdff423`, SHA-256 `45abd1eef5cae133762514f7ecdf61c133f22f3ff73d73170348c307f9b665ad`. Complete MCP baseline measurements completed with 20 timed requests per profile; all source ranges matched. Their p95 values are approximately 1.285 / 32.089 / 474.982 ms for bm25 / hybrid / hybrid_rerank. These are not speedups against the historical 150-question chart: workload identities differ. Local Qwen3.8:27b answer attempts and no-context controls follow separately, with exact quote validation and retained failures.

The task owns an Ollama service at `127.0.0.1:11437`; read `owned-service.json` for its actual PID/liveness. It uses existing BGE-M3 and Qwen3.8:27b, cloud/pruning disabled, one request at a time, at most two loaded models and an 8192-token answer context. No models were downloaded. The service home/log, questions and answers are private ignored evidence, not publication assets. Stop only this recorded owned process after the run. Temporary builds: `/tmp/airust-profile-practical-0.6.0`; retain binary/locks/methods before cleanup.

Next: inspect all actual local answers and errors, then test existing bounded thread/batch controls with unchanged model, 20-candidate prefix and max512 pair tokenization. Freeze comparable paired campaigns, preserve numerical/ranking/quality losses and avoid timings during compilation or unrelated model work. Only adopt a measured improvement. Use the resulting integration problems to scope C1; no global configuration layer is justified by the initial inventory alone. Independent quality/adoption gates remain unmet.

## First action: verify the actual release state

**AIRust 0.6.0 is released.** Its final local receipt `local-lab/release-0.6.0-2026-11-24/final-closeout.json` records clean source/tag/registry identities, all five exact-source candidate/master/tag CI jobs, 13 anonymous payload downloads plus two separate public receipt files, docs.rs and completed owned-build cleanup. The public GitHub release and crates.io archive agree. Verify authoritative receipts if needed; do not repeat publication or move the tag. The earlier release authorization applied to finishing 0.6.0, not automatically publishing a later version.

```bash
git status --short --branch
git log -6 --oneline
git rev-parse 'v0.6.0^{commit}'
gh api repos/LEVOGNE/airust/releases/tags/v0.6.0 --jq '{tag_name,draft,prerelease,html_url,published_at}'
```

- Public closeout: [v0.6.0 assets](https://github.com/LEVOGNE/airust/releases/tag/v0.6.0), especially `release-manifest.json`, `SHA256SUMS` and the separate postpublication public-asset verification receipt. Verify the crates.io checksum against the same `.crate`, VCS commit and annotated tag.
- Local live/final closeout: `local-lab/release-0.6.0-2026-11-24/PROGRESS.json`, `release-manifest.json`, `public-release-verification.json` and `README.md` when present. A missing final receipt means inspect the recorded pending steps; do not infer publication.
- Completed engineering evidence: [validation](validation-0.6.0/README.md), [measurements](measurements/pilot-0.6.0/README.md), [work program](WORK-PROGRAM-0.6.0.json), ignored `local-lab/pilot-0.6.0/`.
- The earlier `RESUME-0.6.0.md` is an explicitly historical pause snapshot. Its `0.6.0-dev`, old branch and pause statements are not current instructions.

## Standing decisions

Memory safety and performance come first, together with stability, robustness and honesty. Preserve `#![forbid(unsafe_code)]`, empty default features, bounded input/output/model work, original data on failure, typed errors, scopes/revisions and retained readers. Use small coherent modules and existing validated APIs. Plan approximately 70% quality/docs/integration/actual validation and 30% justified capabilities. Do not add another database, tensor framework or search algorithm without evidence. MCP/CLI/model components are conceptual boundaries; do not assert separate crates exist or split merely for branding.

The 0.5.4 pilot was integrated into 0.6.0. Patch counts are not test counts. 0.6.x is a compatibility line; breaking changes require a new minor line. The roadmap's future version numbers are conditional targets, not instructions to implement/publish every version. Publication of a later version needs its own explicit release authorization; the current 0.6.0 authorization remains sufficient to finish an interrupted 0.6.0 release.

## Implemented baseline and engineering evidence

- Bounded immutable local pilot capture/check/review/export and independent Python format/source validator/freezer. Query/serve do not record cases automatically.
- Explicit bm25/hybrid/hybrid_rerank CLI/MCP profiles, loopback/digest/cache/recipe checks, visible strict/fallback policy and cooperative cancellation/recovery. Trusted application-owned `McpSearchBackend` results are checked before publication.
- Existing Engine/KnowledgeEntry entry point works without MCP/server/models. A nine-line Rust 1.89 package/registry consumer is included in release checks; an independent timed usability test is a different gate.
- Native release checks: 253 default; 914 all-feature including 39 doctests; 40 isolated feature entries; 13 reference-example Rust tests; six pilot/seven profile Python cases; strict Clippy/docs; package/default/1.89-consumer/1.92-all-feature gates. Counts overlap and must not be summed as unique tests. Exact rerun commits/logs are in release receipts.
- Finite Miri scope: 86 tests across index/core/reduced stability/synchronous MCP/new backend/small pilot groups. Four large index tests, two asynchronous MCP shutdown tests, full corpus parity, HTTP/native Tokio/ONNX/Metal/PDF execution and million-entry workloads are outside the interpreter scope. Filesystem pilot tests explicitly disable isolation. Original commands, flags, exclusions and source identities remain recorded. A partial run is never passed.
- Two Core, two matched-feature complete-MCP and two scale/retained-clone campaigns, plus actual BGE-M3/MiniLMv2 complete-request costs. Raw regressions and failed baseline-feature/package-harness attempts remain. No general speed, throughput, token-saving or production-capacity claim follows. Do not time new campaigns while compilers/Miri run.
- Both-lock RustSec audit has zero known vulnerabilities at the recorded revision, with all maintenance notices for chrono/paste/ttf-parser and backend-lock anymap2 retained. Check current receipts rather than claiming a past audit covers future advisories. Hosted macOS availability has previously failed before steps; actual unavailable jobs remain separate from passing complete local macOS checks.

## Unmet external gates

**Zero of at least 50 genuine independently reviewed pilot questions are established at this closeout.** The frozen 150 reference-v1 cases are assistant-authored; three local historical conversation replays are unreviewed and do not count as independent traffic/review. `evaluation_ready` remains false and no new production-profile recommendation exists. Independent 15-minute onboarding, clean-machine application/real external LLM-client use and independent benchmark reproduction remain unestablished. Do not ask another automated agent to pretend to be a human reviewer.

Use [INDEPENDENT-VALIDATION.md](INDEPENDENT-VALIDATION.md) for the prepared protocol, empty evidence template, quality floors and failure retention. A role declaration/valid export or frozen input set does not pass measured quality/adoption gates. Continue independent engineering work while human-input tasks await actual inputs.

## Next concrete work

1. R0: finish or verify release receipts as above. If publication checks already passed, preserve them and start the next task without repeating upload/tag operations.
2. DX1/Q1/QA1: prepare actual independent attempts, genuine case/source review and external reproduction using the existing protocol. Only record observed participants/results. No unsolicited messages to potential reviewers are authorized.
3. C1: inventory current configuration types, defaults, bounds, ownership and feature interactions in Engine/search, RetrievalOptions, documents, McpLimits and profile JSON. Tie any proposed facade to an observed integration problem. Reuse validated controls and prepare a small compatible change; do not create a new configuration layer merely to fill 0.6.1.
4. Then use the conditional 0.6.x diagnostics/observability/resources/security tasks in the roadmap. Update the handoff and machine work program after each coherent step with actual commands, evidence paths, changed behavior, limitations and next action.

For implementation, inspect current user changes first, branch under `codex/`, and use a new unpublished version when changing an immutable published package. Run meaningful affected tests and the required release matrix. Use `cargo fmt --package airust`, never `cargo fmt --all`.

## Data and evidence that must survive

- Original `knowledge/`, source documents, user workspaces and `airust.db` are not build artifacts. Database SHA-256 at closeout: `8048756bb34ea465ed4bdbc90537102e83e9690d360bba6c9d951b3b285bfbe8`.
- Preserve frozen `tests/fixtures/reference-v1/`, questions/judgments/policy/results and every historical source/model/release identity. Reference-v2 must be a separate dataset/version.
- Keep ignored `local-lab/pilot-0.6.0/` and `local-lab/release-0.6.0-2026-11-24/`, prior release evidence, raw failures, executed methods, locks and retained binaries.
- Shareable benchmark archive: `local-lab/pilot-0.6.0/engineering-evidence.tar.gz`, SHA-256 `cc5c1787bd1983a557a0dd1259a584b48c0b746c064f5b31c538d880eed830cc`, 481 verified files. Its tracked manifest is [artifact-manifest.json](measurements/pilot-0.6.0/artifact-manifest.json).
- Do not publish `engineering-evidence-private.tar.gz`, `question-replays/`, `ollama.log` or `local-data-schema.json`. The public archive excludes private questions/service environment output.
- Existing model assets remain under `local-lab/reference-workflow/models/mmarco-minilm-v2/`; trusted runtime under `local-lab/pdf-stability/python-deps-0315/onnxruntime/capi/`. Do not download/restart models automatically. The earlier task-owned Ollama service was stopped; no continuing model service is asserted.
- `airust-pdf-backend` stays separately versioned at 0.3.5, untouched. Backend source/provenance/C ABI/MSRV requirements are in CONTRIBUTING and the parser guide. Never reformat or republish it incidentally.
- Only dedicated regenerable build directories may be cleaned after preserving binaries, locks, packages and receipts. Earlier `/tmp/airust-pilot-0.6.0` was already removed; historical paths in logs are not current files. Release-owned `/tmp/airust-release-0.6.0` may be removed only after final archival and process completion. Never kill unrelated user services.

## Useful verification commands

```bash
./test.sh
python3 -B scripts/check_pdf_backend.py
python3 -B scripts/reference_dataset.py
python3 -B scripts/run_miri.py NEW_EVIDENCE_DIRECTORY --toolchain INSTALLED_NIGHTLY
```

Use separate owned target directories with debug info/incremental builds disabled when bounding build storage. Package/publish operations may refresh Cargo's verifier extraction; consume a separate immutable extraction and do not race those mutations. The [validation guide](validation-0.6.0/README.md) gives model/control reproduction with exact retained locks. Missing models, external data, reviewers or hosted runners are unavailable evidence, not passes. Preserve a usable next-action record before any future pause.

The release macOS CI job explicitly selects the supported `macos-15` ARM64 image, uses two Cargo build jobs and a 45-minute job bound, with architecture-specific cache keys. Earlier macOS 26 ARM64 capacity cancellations and an unresponsive/log-unavailable Intel attempt remain separate non-passing evidence. The Intel attempt does not establish Intel support or a proven library failure. Read actual final job conclusions in the release manifest; keep complete local ARM64 checks and scoped Miri separately recorded.
