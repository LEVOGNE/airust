# AIRust 0.6.1 candidate validation

This guide records an unpublished source candidate, not a registry release. Current command results and source identities are retained under `local-lab/configuration-0.6.1/` and the practical study under `local-lab/profile-practical-0.6.0-2026-11-24/`. Read actual result files before treating a gate as passed. The final handoff/work program supplies the current step.

## Required native and compiler controls

Run `./test.sh` with a dedicated owned target directory and debug info/incremental builds disabled to bound build storage. It includes the original feature/platform/Clippy/documentation/package controls, the new pilot-feature Rust example tests and Python real-CLI no-network preflight tests. Default and all-feature totals overlap; record counts per invocation rather than adding them as unique tests. Keep the original crash-writer subprocess helper's ignored marker and its passing parent distinct.

Check the provider-neutral/pilot public path on Rust 1.89 and all features/all targets on Rust 1.92. Retain an independent consumer over an immutable extracted package, a model/server-free nine-line core consumer and a real profile-check invocation over the packaged example. Run the backend provenance/owned regressions separately; backend 0.3.5 and its lock/source remain unchanged. Audit both locks with the observed advisory revision and all maintenance notices retained.

Before claiming package readiness, use a clean source commit, `cargo package --locked`, `cargo publish --locked --dry-run`, inspect the archive and retain exact VCS/archive identities and consumer locks. A dry run is not publication. No 0.6.1 tag/GitHub/crates.io release is asserted by these instructions.

## Interpreter controls

`scripts/run_miri.py NEW_DIRECTORY --toolchain INSTALLED_NIGHTLY` adds the seven pure `reference_profiles::configuration_tests` to the existing six finite groups. The new group uses `pilot` without `local-reranking`, and explicitly disables Miri isolation only for temporary profile-file fixtures. It loads no native runtime, model, HTTP client/service or inference state.

Retain actual commands, source, flags, counts and each log. Existing exclusions remain: four large index tests, two asynchronous MCP shutdown tests, full independently generated corpus parity and native HTTP/Tokio/ONNX/Metal/PDF execution/million-entry workloads. The reduced stability test keeps its existing cfg(miri) workload. A missing/partial/failed interpreter group is not passed evidence. Finite Miri success does not prove absence of all undefined behavior or native-dependency defects.

## Actual model and compatibility measurements

Use the [practical measurement record](../measurements/profile-practical-0.6.0/README.md). The baseline uses published 0.6.0 source, all three real profiles and existing BGE-M3/MiniLM/Qwen3.8 assets. Preserve the four conversation inputs privately and distinguish them from the unchanged 150 authored regression questions. Freeze source/model/workspace/query identities and numerical/ranking/resource criteria before candidates.

Compare compatible baseline/candidate binary features, build flags and locks. Keep 20 actual first-stage candidates and maximum pair length 512 when evaluating batching/threads. Retain exact score differences, every top-six/known-quote loss and every slower observation. Separate native scoring, complete MCP request, startup and complete answer generation. Do not benchmark while compilers/Miri or unrelated model work run. Process RSS and service-reported model sizes do not establish unique GPU/shared physical memory.

The actual LLM attempts retain one invalid generated quotation and a confident historical-roadmap answer. A native timing improvement does not repair those failures or establish independent quality. Source/review provenance gates remain at zero eligible independently reviewed cases against minimum 50.

## Preservation and resumption

Original `knowledge/`, `airust.db`, user workspaces, frozen reference-v1, prior raw evidence and supplied models/runtime are retained. Model manifests/assets and source texts must match before/after the study. Stop only the recorded owned service at 127.0.0.1:11437 after all model work. Preserve binaries/locks/methods/raw failures before cleaning `/tmp/airust-profile-practical-0.6.0` or separate validation targets. Never clean evidence as build output.

Update [SESSION-HANDOFF.md](../SESSION-HANDOFF.md), [WORK-PROGRAM-NEXT.json](../WORK-PROGRAM-NEXT.json) and this guide with observed final results. Future publication requires its own authorization; keep published 0.6.0 immutable.
