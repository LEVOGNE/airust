# AIRust 0.6.1 candidate validation

This guide records an unpublished source candidate, not a registry release. Receipts are retained under `local-lab/overnight-0.6.1-2026-11-24/`; earlier configuration/practical evidence keeps its original paths. Production source was frozen at 2026-11-24 23:40:39 UTC; the 23:51:37 revision changes two training-test fixtures/scopes only. All selected long fuzz scopes and separate final-binary controls completed by **2026-11-25 07:16 UTC**. Clean package/extracted-consumer and exact-source hosted checks follow the final source commit; do not infer their outcome from an earlier preview. Read actual receipts before treating a gate as passed.

## Observed candidate checks

| Check | Observed result | Scope / retained receipt |
| --- | --- | --- |
| Full `./test.sh`, Rust 1.98.0 | Passed, 688.058 s; repeated stream controls 138.468 s; latest reconstruction-control run 137.426 s, finished 05:31 UTC | All complete logs/results retained; later runs reuse compiler cache, not test results |
| Default `cargo test --workspace --locked` | 266 passed | Count for this invocation; overlaps all-feature tests |
| All-feature workspace tests | 928 passed, including 39 doctests | One deliberately ignored crash-writer subprocess helper; its parent test passes |
| Reference example, `mcp` / `pilot` | 13 / 20 Rust tests passed | Seven new configuration controls are in the pilot invocation |
| Small stdio example | 2 Rust / 6 Python tests passed | Actual process, recovery after discarded frames, true quota close with open client pipe, input caps and FIFO controls |
| Frozen tokenizer-input reconstruction | 4 Python CLI tests passed | Valid source/rank reconstruction, corruption/revision checks, input bounds and retained original destinations |
| Python dataset/workflow/pilot/profile/evaluation/download checks | 5 / 6 / 6 / 10 / 4 / 5 passed | Counts per invocation, not a new unique combined total |
| Isolated declared features | All 40 passed, all targets | Supplements interaction/all-feature checks; not every feature subset |
| Strict Clippy / rustdoc / examples / targeted interactions | Passed | Commands retained in the complete native log |
| Rust 1.89 all provider-neutral targets | Passed | Includes pilot, excludes supplied native ONNX adapters |
| Rust 1.92 all features / all targets | Passed | Compile controls, separate from actual asset-backed inference |
| Static API compatibility against exact 0.6.0 | All five views passed | Default, MCP, local embeddings, local reranking, all features; cargo-semver-checks 0.51.0 |
| Backend 0.3.5 | 14 regressions, C ABI compile, provenance and Rust 1.89 compile passed | Backend source/lock unchanged; official upstream comparison and license checks retained |
| Both-lock advisory audit | Zero known vulnerabilities at recorded revision | All maintenance notices retained; see below |
| Package boundary during latest native script | 4018 intended files | Earlier counts predate public evidence/protocol additions; actual final archive still requires clean packaging |
| Selected long libFuzzer scopes | Lexical 90 min / PDF recovery 90 min / training 60 min passed | ASAN only lexical/training; original PDF corpus-I/O failure retained |
| Final-source training confirmation | 16 workers passed | All signatures match; first-write/all-writer costs retained |
| Final-source model-free host | 8 workers / 8000 timed + 256 warmup requests passed | Includes 393216-term stress case; all 512-MiB peak / 32-MiB growth budgets pass |
| Final-source complete MCP requests | Both paired campaigns passed | 600 timed + 600 initial requests; unchanged top six/non-logit data/known quotes; both p95 gain gates pass |
| Optional allocator request cost | Both declared 10% overhead gates passed | Same binary/profile, ratios 0.983 / 0.997; all 159 slower per-query means retained; not a library default |

Static API comparison uses official tool archive SHA-256 `3cec5da41eee9a3336d75ad43d1f30a5c9680818f4ba209024786f452cabab5b` and extracted binary `30493e32d77a4a9f90ea6695cf67765a4c9106c46a7db4603f2026043d469bb8`. The baseline is an immutable archive of released source `8c098f15a39f6085f748c56ac52e5737abdff423`. These are current-host public-signature checks, not behavior, data-format or all-platform proofs.

`cargo tree --locked --offline --no-default-features --edges normal` also matches the baseline's complete normal dependency tree, including proc macros, apart from the AIRust package identity. Default features remain empty. The optional native snapshot reuses the already declared `tempfile` dependency; no new dependency version is introduced.

Both lockfiles were audited with cargo-audit 0.23 against official RustSec revision `10db6998b5d74ec438bcf0763125bd425a068a8f`. No advisory was ignored. Retained maintenance notices: chrono `RUSTSEC-2026-0543`, paste `RUSTSEC-2024-0436`, ttf-parser `RUSTSEC-2026-0192`, and backend comparison dependency anymap2 `RUSTSEC-2026-0319`. This is a dated advisory check, not an absence-of-vulnerabilities guarantee.

## Required native and compiler controls

Run `./test.sh` with a dedicated owned target directory and debug info/incremental builds disabled to bound build storage. It includes the original feature/platform/Clippy/documentation/package controls, the new pilot-feature Rust example tests and Python real-CLI no-network preflight tests. Default and all-feature totals overlap; record counts per invocation rather than adding them as unique tests. Keep the original crash-writer subprocess helper's ignored marker and its passing parent distinct.

Check the provider-neutral/pilot public path on Rust 1.89 and all features/all targets on Rust 1.92. Retain an independent consumer over an immutable extracted package, a model/server-free nine-line core consumer and a real profile-check invocation over the packaged example. Run the backend provenance/owned regressions separately; backend 0.3.5 and its lock/source remain unchanged. Audit both locks with the observed advisory revision and all maintenance notices retained.

The external [consumer source](consumer.rs) additionally checks training snapshot isolation, bounded loading/retained save paths, multiline Unicode diagnostic positions, original files, document spans, typed cancellation, the application search trait and all five versioned MCP schemas. This is automated package-integration evidence, not an independently timed human onboarding attempt. [REPRODUCE.md](REPRODUCE.md) supplies the measurement procedures and the formatted training probe.

The dirty package preview (4008 files / 5886388 bytes, archive SHA-256 `093614053810dd047f4f0559b02f5c9fc30b566826d5e9aa796bf1ef8bfe0452`) was copied to a separate immutable extraction. Three external Rust-1.89 programs pass: the nine-line default consumer, the public API/document/MCP/diagnostic consumer and a small functional invocation of the training probe. This preview is explicitly not the final clean package; later protocol/test additions need the final archive/consumer receipts.

Retained non-passing orchestration attempts include a sandbox run that could not bind the fake loopback HTTP fixture, plus an initial stream test with the wrong expectation that every discarded malformed frame must close the connection. The installed primary SDK source and existing recovery tests disproved that expectation. Original and experimental binaries both closed actual quotas correctly with the client pipe open; the proposed runtime change was reverted. Six corrected small-host tests and the repeated full matrix pass on byte-identical production Rust/manifest inputs. No failed attempt is relabelled a product fix or passed validation.

The final-source controller's initial archive guard also stopped before any archive/build/timing: the original database is historically tracked, so Git visibility alone was an incorrect archive-selection rule. The corrected task controller explicitly excludes it and private laboratory evidence without modifying measured Rust/method inputs. The failed attempt/log remains retained, with zero discarded timing observations. An initial ancillary listener-output parser likewise stopped before signalling anything; its corrected ownership check accepts lsof's descriptor records while retaining exact PID/start-time/port verification.

Before claiming package readiness, use a clean source commit, `cargo package --locked`, `cargo publish --locked --dry-run`, inspect the archive and retain exact VCS/archive identities and consumer locks. A dry run is not publication. No 0.6.1 tag/GitHub/crates.io release is asserted by these instructions.

## Interpreter controls

`scripts/run_miri.py NEW_DIRECTORY --toolchain INSTALLED_NIGHTLY` defines eleven finite groups. The observed revised scope has **121 passed tests** on nightly `1.102.0-nightly (5f4effd57 2026-11-13)`, Miri `0.1.0 (5f4effd579 2026-11-13)`, aarch64-apple-darwin. It loads no native model/runtime or inference service.

| Group | Passed |
| --- | ---: |
| Private regular-file / allocation / diagnostic input | 6 |
| Private pinned asset / snapshot helper | 2 |
| Training ownership / bounded load | 18 |
| Training property controls | 2 |
| Index | 41 |
| Core | 25 |
| Reduced stability state | 1 |
| Synchronous MCP response | 4 |
| MCP search-backend validation | 3 |
| Small workspace/pilot | 12 |
| Profile configuration | 7 |

This is an aggregation of observed groups, not a claim that the initial full invocation passed. Initial training groups stopped at Miri's unsupported macOS directory `F_FULLFSYNC`; the original report remains failed. Production persistence was not bypassed. The explicitly revised training subset reran and passed 18+2 tests after changing only test setup/successful-save scoping. Nine unaffected groups completed in the original invocation. `miri-aggregate.json` records both receipt hashes, the two changed test inputs, all original failures and the reconciliation. Successful save/durability paths, including the omitted `test_save_load_roundtrip`, passed natively.

Flags are `-Zmiri-tree-borrows -Zmiri-no-extra-rounding-error`, with `-Zmiri-disable-isolation` only for temporary filesystem fixture groups. The training properties use eight Miri iterations versus 128 native iterations. The asset checksum fixture uses 71 Miri bytes; its native version crosses the 64-KiB chunk boundary with 65543 bytes. `--groups` runs a named subset and explicitly labels it as a subset.

Retain actual commands, source, flags, counts and each log. Exclusions remain: successful macOS directory-sync publication, four large index tests, two asynchronous MCP shutdown tests, full independently generated corpus parity and native HTTP/Tokio/ONNX/Metal/PDF execution/million-entry workloads. The stability test keeps its declared reduced cfg(miri) workload. A missing/partial/failed interpreter group is not passed evidence. Finite Miri success does not prove absence of all undefined behavior or native-dependency defects.

## Instrumented fuzzing

Three final-source libFuzzer smoke probes verify nonzero coverage counters/PC tables and execution of all retained seeds. Lexical and training binaries additionally have observed AddressSanitizer initialization symbols; PDF is explicitly coverage-only. Original campaigns started 2026-11-24 23:51:52 UTC: 5400 seconds lexical, 5400 seconds PDF, 3600 seconds training, sequentially with one worker and fixed seeds.

Lexical completed its full scope: 509913 executions, 5409.654 seconds observed, 183377 instrumentation counters, no artifacts. Training completed: 405218 executions, 3605.818 seconds observed, no artifacts. These are mutation executions, not unique tests or production throughput.

The original PDF attempt **did not pass**: exit 1 after 3741.057 seconds, without an artifact, ending `Interrupted system call: .../corpus; exiting`. The exact diagnostic is in libfuzzer-sys 0.4.13's `FuzzerIOPosix.cpp`, where failed `opendir` terminates corpus-directory enumeration. `FuzzerLoop.cpp`/`FuzzerFlags.def` identify periodic shared-corpus reload. The source hashes and failed report/log remain retained. This is a confirmed framework directory-I/O exit, not a completed fuzzing result or proof that the PDF target is defect-free.

A separate **full 5400-second PDF recovery passed** at 2026-11-25 06:56:13 UTC: 224254 executions, 5402.934 seconds observed, 780807 coverage counters, exit 0 and no artifacts. It uses the same target binary/input/timeout/RSS/coverage guards, a fixed new seed, every original/generated seed copied into an independent directory and `-reload=0`. One worker does not require other-worker corpus exchange; no target code or guard is disabled. The earlier 62-minute attempt is not stitched to a short retry or relabelled successful. `fuzz-reconciled.json` selects the two complete original ASAN scopes plus this separate complete coverage-only recovery and preserves the original overall-false report.

Inputs are copied from historical corpora into new owned directories. Per-input limits are 4096 / 65536 / 4096 bytes, native timeout 8 seconds and RSS limit 4096 MiB; the controller also limits corpus, artifacts, logs and wall time. AddressSanitizer leak detection is disabled and C/standard-library coverage is incomplete. Completed campaigns provide bounded finding evidence, not a leak or safety proof. Preserve any future crash/timeout/artifact and minimize it before fixing. The training target checks arbitrary bounded UTF-8/JSON, retained snapshots, independent values and failed publication without replacing original evidence.

## Actual model and compatibility measurements

Use the [practical measurement record](../measurements/profile-practical-0.6.0/README.md). The baseline uses published 0.6.0 source, all three real profiles and existing BGE-M3/MiniLM/Qwen3.8 assets. Preserve the four conversation inputs privately and distinguish them from the unchanged 150 authored regression questions. Freeze source/model/workspace/query identities and numerical/ranking/resource criteria before candidates.

Compare compatible baseline/candidate binary features, build flags and locks. Keep 20 actual first-stage candidates and maximum pair length 512 when evaluating batching/threads. Retain exact score differences, every top-six/known-quote loss and every slower observation. Separate native scoring, complete MCP request, startup and complete answer generation. Do not benchmark while compilers/Miri or unrelated model work run. Process RSS and service-reported model sizes do not establish unique GPU/shared physical memory.

The [final-source cohort](../measurements/configuration-0.6.1-final/README.md) finished after fuzz completion, with all required and optional declared gates passed. Full-request p95 was 895.581 / 900.440 ms for baseline versus 229.610 / 228.034 ms for 8-thread/batch1/snapshot candidate, with the same model/twenty candidates/max512 and no changed top-six/non-logit evidence. Worker peak fell from about 3.395 GB to 1.429 GB; extra CPU and startup costs remain visible. Sixteen COW workers and eight model-free workers separately confirm resource behavior. Twelve public CSV files retain 12073 numerical rows, including all 159 slower optional-allocator query means; these are not unique tests or independent customer cases. Precompilation/cache reuse and the zero-observation initial archive-guard failure are recorded explicitly.

The actual LLM attempts retain one invalid generated quotation and a confident historical-roadmap answer. A native timing improvement does not repair those failures or establish independent quality. Source/review provenance gates remain at zero eligible independently reviewed cases against minimum 50. Native default-allocator repeated-load growth still failed twice; E5 asset rejection is checked but actual E5 inference remains unavailable without its supplied assets.

## Preservation and resumption

Original `knowledge/`, `airust.db`, user workspaces, frozen reference-v1, prior raw evidence and supplied models/runtime are retained. Model manifests/assets and source texts must match before/after the study. Stop only the recorded owned service at 127.0.0.1:11437 after all model work. Preserve binaries/locks/methods/raw failures before cleaning `/tmp/airust-profile-practical-0.6.0` or separate validation targets. Never clean evidence as build output.

Update [SESSION-HANDOFF.md](../SESSION-HANDOFF.md), [WORK-PROGRAM-NEXT.json](../WORK-PROGRAM-NEXT.json) and this guide with observed final results. Future publication requires its own authorization; keep published 0.6.0 immutable.
