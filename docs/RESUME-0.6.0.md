# Resume AIRust 0.6.0 development

Historical pause checkpoint: development resumed at the user's request on 2026-11-24. Current results are in [the validation record](validation-0.6.0/README.md); the snapshot below describes the earlier pause.

Paused at the user's explicit request on 2026-11-24. This is a development checkpoint, not a completed or released 0.6.0 milestone. Resume only when the user asks to continue. Do not start background work during the pause.

## Objective and standing priorities

Complete the local pilot originally planned for 0.5.4, then the 0.6.0 work: explicit live search profiles, stronger stability evidence, Miri, reproducible benchmarks and honest handling of real search cases. The user extended the scope through 0.6.0 and asked for autonomous execution without routine questions, then explicitly requested this pause.

Memory safety and performance have first priority, together with stability, robustness and honesty. These instructions are persisted in the root [AGENTS.md](../AGENTS.md). Keep the default core small, preserve `forbid(unsafe_code)`, bound inputs/output/model work, retain failed measurements, and never invent independent human judgments or customer traffic. Follow [CONTRIBUTING.md](../CONTRIBUTING.md). Repository code and documentation stay in English; communication with the user is in German.

## Git and release boundary

- Working branch: `codex/pilot-0.6.0`.
- Parent before this checkpoint: `e38a384` (`Define the post-0.5.3 roadmap and milestone acceptance gates`), originally on `codex/roadmap-after-0.5.3`.
- Published baseline: `705803e34382be856c288c7c5c2eeb2507217330`, `master`, `origin/master`, tag `v0.5.3` at the last release verification.
- [GitHub release 0.5.3](https://github.com/LEVOGNE/airust/releases/tag/v0.5.3) and crates.io `airust 0.5.3` were verified in the preceding work. The crate checksum is `d558ec5a4b44bf7bd729308a3c1596d0d16022f7a93449f1b1e5b2106d6305e8`.
- Current manifests use `0.6.0-dev`. No 0.5.4 or 0.6.0 tag, release, registry publication or master merge has been performed for this work.
- Preserve the immutable 0.5.3 release and historical [work program](WORK-PROGRAM.json). Its seven completed stages describe 0.5.3, not the unfinished new milestone.
- The user's earlier explicit publication request applied to 0.5.3. A new publication still requires release authorization under CONTRIBUTING.md; do not infer it from an implementation or pause request.

The checkpoint commit is the commit that first adds this document. Find it with `git log -1 --format='%H %s' -- docs/RESUME-0.6.0.md`. It is local; do not assume it was pushed.

## Implemented so far

These changes compile in the targeted configurations and have the tests listed below. End-to-end integration, measurements and release gates remain unfinished.

| File | Current implementation |
| --- | --- |
| `Cargo.toml`, root/fuzz locks | Development version; new opt-in `pilot` feature enables existing MCP/hybrid/reranking and blocking reqwest. No dependency versions were upgraded. The root reqwest lock entry additionally lists its existing futures channel/util dependencies. |
| `src/mcp.rs` | Application-owned `McpSearchBackend` and `McpServer::with_search_backend`. The backend runs inside admitted blocking work. Custom results are checked against registered IDs, exact source text/spans, metadata, scope and output budgets before publication. Cancellation after backend work cannot publish partial success. Earlier server clones retain their previous backend. The five MCP tools and default lexical backend remain. |
| `examples/reference_support/mod.rs` | Deterministic workspace revision identity and shared bounded `query_using` path. Saved workspace schema remains version one. Preserve the typed `RetrievalError` cancellation contract fixed during targeted testing. |
| `examples/pilot_support/mod.rs` | Explicit local pilot initialization; bounded immutable capture, check, review and export; checksummed case envelopes; exact UTF-8 source/revision validation; declared origin/reviewer independence; manifest published last. No automatic recording by query/serve. |
| `examples/reference_profiles/mod.rs` | Explicit `bm25`, `hybrid`, `hybrid_rerank` configuration; bounded loopback Ollama embedding calls, digest checks before/after embedding, revision/model-bound vector cache; optional verified local ONNX reranker; visible strict or lexical-fallback policy; request control and cancellable model-lock waiting. Loading never downloads assets. |
| `examples/reference_workspace.rs` | CLI wiring for pilot commands and optional profiles in query and MCP serve. Existing build/query/serve commands remain available. |
| `AGENTS.md` | Durable engineering priorities requested by the user. |

Current CLI shapes:

```text
reference_workspace build INPUT_DIRECTORY NEW_WORKSPACE COLLECTION
reference_workspace query WORKSPACE QUESTION [--profile CONFIG]
reference_workspace serve WORKSPACE [--profile CONFIG]
reference_workspace profile-build WORKSPACE CONFIG NEW_VECTORS
reference_workspace pilot init INPUT_DIRECTORY NEW_PILOT COLLECTION
reference_workspace pilot capture PILOT REQUEST_JSON NEW_CASE_JSON [--profile CONFIG]
reference_workspace pilot review PILOT CASE_JSON REVIEW_JSON NEW_CASE_JSON
reference_workspace pilot check PILOT CASE_JSON
reference_workspace pilot export PILOT SELECTED_CASE_DIRECTORY NEW_EXPORT
```

Capture requests contain `case_id`, `query`, `language`, `origin` and `origin_evidence`. Origins distinguish `user_supplied`, `user_question_replay`, `assistant_authored`, `synthetic_test` and `unknown`. Captures begin unreviewed. Reviews preserve the request origin and create a new file linked to the previous payload hash; they do not modify the original case. Human/independent declarations are caller claims, not authenticated identities. Export rejects duplicate case IDs, so select one immutable revision per case in the export input directory.

Bounds already present: 128 KiB per case; 256 cases and 32 MiB per export; query 4096 bytes; at most six observed excerpts, 2000 bytes each and 12000 bytes total; at most 32 review source ranges; model config 16 KiB; vector cache 64 MiB; embedding batch at most 16 texts/64 KiB and response 2 MiB. Workspace import retains the existing 256-file, 4 MiB/file, 32 MiB total bounds. Typed source ranges refer to the original supplied text, including Unicode byte coordinates.

## Executed validation at the pause

Evidence directory: `local-lab/pilot-0.6.0/` (ignored, retained locally). Portable commands, counts, log hashes and missing gates are recorded in [checkpoint.json](validation-0.6.0/checkpoint.json).

- `cargo check --features mcp --lib`: passed; this first check updated the lock for the new feature union.
- `cargo check --locked --features pilot,local-reranking --example reference_workspace`: passed. This is a compile check, not native inference execution.
- `cargo test --locked --features mcp --example reference_workspace`: **11 passed**: six existing workspace tests plus five new pilot-format/export tests.
- `cargo test --locked --features mcp --example reference_workspace --lib search_backend_tests`: **3 backend tests passed**. The test-name filter selected zero example tests; the separate 11-test run above covers the example.
- `cargo fmt --package airust -- --check`: passed at the pause.
- `git diff --check`: passed at the pause.
- `cargo +nightly miri --version`: available, `miri 0.1.0 (5f4effd579 2026-11-13)`. **No new Miri test run has occurred.**

The initial workspace test log contains a cancellation error-type failure. It was fixed by mapping the new checkpoint error through `RetrievalError`; the later 11-test run passed. Preserve both logs.

**Not run for this source:** full `./test.sh`, new real CLI/MCP pilot/profile integration tests, actual live model queries, MSRV checks, scoped Miri, benchmarks, platform CI, package/publish dry run or registry consumer validation. Historical 0.5.3 counts and old measurements must not be reported as results for 0.6.0.

The repository is deliberately an unfinished checkpoint: `scripts/check_support.py` and therefore the first step of `./test.sh` will currently reject the new `pilot` feature because `docs/support-matrix.json` has not yet been updated. `docs/PILOT.md`, integration tests and current-version entry-point documentation also remain to be written. Do not call the full suite green.

## Local environment and retained inputs

- Workspace: `/Users/levogne/Desktop/airust-master`.
- Own build directory: `/tmp/airust-pilot-0.6.0/target`; it was retained for resumption. Temporary files may be removed by the OS; rebuild if absent. No source depends on their continued existence.
- Common build environment used for the targeted checks:

```bash
export CARGO_TARGET_DIR=/tmp/airust-pilot-0.6.0/target
export CARGO_PROFILE_DEV_DEBUG=0
export CARGO_PROFILE_TEST_DEBUG=0
export CARGO_INCREMENTAL=0
export CARGO_BUILD_JOBS=4
export CARGO_NET_OFFLINE=true
```

Use `python3 -B` for Python tooling; this host has a global bytecode-cache location outside the writable project. Use `cargo fmt --package airust`, never an indiscriminate all-workspace formatter that changes the separately maintained backend.

- Stable Rust is 1.98.0; toolchains 1.89.0, 1.92.0 and nightly are installed. Core/provider-neutral MSRV is 1.89; optional native adapters require 1.92.
- Existing local BGE model: `bge-m3:latest`, digest `7907646426070047a77226ac3e684fbbe8410524f7b4a74d02837e43f2146bab`, 1024 dimensions. Installed Ollama version from previous work: 0.40.17. Recheck identities when actually used.
- Verified MiniLM assets: `local-lab/reference-workflow/models/mmarco-minilm-v2`; pinned upstream revision `1427fd652930e4ba29e8149678df786c240d8825`.
- Existing runtime: `local-lab/pdf-stability/python-deps-0315/onnxruntime/capi/libonnxruntime.1.32.0.dylib`.
- The earlier `local-lab/reference-workflow/vectors.json` uses reference-v1 IDs, not current workspace passage IDs. Prepare a fresh profile cache against the actual workspace; do not reuse that file as if compatible.
- Previous release evidence: `local-lab/release-0.5.3-2026-11-24/`; frozen dataset/results: `tests/fixtures/reference-v1/`, `docs/measurements/reference-v1/`.
- At pause inspection, no matching AIRust build, Miri/evaluation job or `ollama serve` process was running. No background campaign has been scheduled. Preserve unrelated user processes.

Root `airust.db` is original user data. Only a read-only, immutable SQLite schema inspection was performed for this milestone; no chat content was extracted and no cases were inferred from it. SHA-256 before/after inspection and at the pause: `8048756bb34ea465ed4bdbc90537102e83e9690d360bba6c9d951b3b285bfbe8`. Schema evidence is local only in `local-lab/pilot-0.6.0/local-data-schema.json`. The existence of `chats`/`messages` tables does not establish usable genuine search traffic or independent judgments. Never replace this database to run a pilot.

## Remaining work, in execution order

1. **Complete pilot verification and documentation.** Add actual subprocess CLI/stdio tests covering init, explicit capture, immutable review, export/restore, original-source preservation and source coordinates. Independently verify exported hashes/UTF-8 ranges in Python. Cover tampering even after envelope rehash, stale workspace/model revisions, duplicate IDs, oversized/unknown fields, symlinks, existing destinations, invalid reviewer declarations and no automatic recording. Document that semantic validation precedes export creation, but an I/O failure can leave a partial new directory; the manifest is the completion marker. Never claim multi-file crash-atomic publication.
2. **Verify profile behavior before recommending it.** Add an explicitly synthetic loopback HTTP test service for cache creation and live CLI/MCP queries. Test strict failure versus visible lexical fallback, changed digest before/after embedding, wrong vector dimensions/zero/nonfinite values, corrupt/incompatible caches, wrong workspace, output bounds, scope and deadlines/cancellation. Native paths remain an opt-in asset-backed run. Consider explicitly disabling HTTP proxies on the loopback client (`ClientBuilder::no_proxy`) so environment proxy settings cannot redirect supplied local text. Review the current additional post-retrieval checkpoint on default MCP behavior and validate its cost, rather than claiming unchanged performance.
3. **Make collection auditable.** Implement an independent export validator/readiness report and, if justified by available genuine inputs, a separately versioned reference-v2 freeze path. An initial export always reports `evaluation_ready: false`; it cannot establish source-family splits, reviewer authenticity or a completed independent dataset. Distinguish tooling readiness from empirical quality readiness.
4. **Update support and interfaces.** Add `pilot` to `docs/support-matrix.json` (40 declared entries including default), a real `docs/PILOT.md` guide, `test.sh` integration checks and the Rust 1.89 provider-neutral CI feature list. Update README, STARTUP, API, MCP, change/migration notes and ROADMAP for the actual finished behavior. Keep v1 snapshot/data compatibility and literal lexical defaults. Do not retroactively alter historical receipt files or the frozen reference-v1 policy/results.
5. **Record maintenance decisions.** Retain reviewed reverse-dependency paths and compatible options for `chrono`, `paste`, `ttf-parser` and backend-lockfile `anymap2` notices. The previous audit snapshot found zero known vulnerabilities, but maintenance warnings remained; this is historical evidence, not a fresh audit pass. Direct/transitive ownership differs: chrono is selected through optional web/backend and MCP dependencies; paste through native tokenizers; ttf-parser through the backend; anymap2 is present in the backend comparison lock but was not selected in its default graph. Review exact paths rather than guessing or suppressing warnings. Do not change/publish `airust-pdf-backend 0.3.5` without its own provenance/compatibility gates.
6. **Freeze the measurement method before execution.** Use published 0.5.3 as baseline. The existing external identical-source consumer in `docs/validation-reference/refactor-consumer.rs` covers exact ordered IDs, score bits, scopes, Unicode, retained clones, replacement/removal and JSON restoration. Existing `scripts/build_index_control.py` / `verify_index_refactor.py` hard-code registry **0.5.2** and historical provenance; introduce an explicitly versioned/parameterized 0.5.3 campaign without silently changing old evidence. Record command/source/lock/binary/model/dataset hashes and all failures/regressions.
7. **Run scoped Miri on the completed implementation.** Historical 0.5.2 scope was 71 tests: 41 index tests with shared-index/execution, 25 core tests with shared-index, one scaled stability-state test and four synchronous MCP response tests. Excluded index tests: `empty_vocabulary_releases_its_hash_allocation_after_removal`, `sparse_vocabulary_shrinks_without_changing_retained_readers`, `token_ties_keep_id_order_and_character_scores_for_long_text`, `token_top_k_preserves_full_comparison_scores_and_unicode_explanations`. Excluded async MCP tests: `final_reply_work_keeps_admission_and_shutdown_until_cancelled_work_returns`, `shutdown_does_not_wait_for_other_server_clones_worker_permits`. Read the actual old command/flags in `docs/CHANGES-0.5.2.md` before reproducing. Add the three new pure backend tests and suitably small pilot tests. Record exact current interpreter flags, counts and exclusions; HTTP and native ONNX FFI execution are outside Miri's scope. Run excluded behavior natively; a finite Miri pass is not proof of absence of memory errors.
8. **Run actual performance comparisons.** Keep compiler/test work out of timed phases. Use two alternating paired baseline/candidate campaigns for the unchanged default Engine/MCP paths and retain slower cases. For each live profile measure complete warm MCP request latency including real query embedding, retrieval/reranking and final framing; record cold load separately. With six output hits, report Hit@6/MRR@6 rather than relabeling them nDCG@10. Use explicit `excerpt_bytes: 2000` in requests when comparing known quote presence; the normal MCP default is shorter. Reference-v1 IDs differ from workspace IDs, so compare the source/quote projection correctly. Measure worker and separate embedding-service memory, state unavailable GPU/shared-memory measurements, and do not add RSS values as if they were unique physical memory. Start/stop only a task-owned model service. Native inference must use the pinned verified assets. Freeze numerical budgets before running, retain all losses, and make no general speed/quality claim from one host or synthetic fixtures.
9. **Close the engineering gates.** After targeted checks pass, run complete `./test.sh`, Rust 1.89 provider-neutral and Rust 1.92 all-feature checks, clean packaging and publish dry run, meaningful package-consumer checks, and the actual supported-platform CI on the final candidate. Push only the authorized development branch for review/CI if needed; a version bump is not release authorization. Keep source, model, package and run identities explicit. Archive raw evidence before removing only owned rebuildable artifacts.

## Real-data gate and honesty boundary

Reference-v1 contains **150 assistant-authored questions**, not independent customer traffic. Its identity is `83595ee5603f3945f0aa8dbb07e0c79c024edf3fe14ef95d139ffcad73472ff7`. Its 90/60 split and reserved results are frozen and must not be retuned.

The 0.6.0 roadmap calls for **at least 50 genuine pilot questions with independent source review**, including unsupported cases, followed by frozen source-family splits and quality/resource gates. Those inputs have not been supplied or established. There is currently **no new genuine independently reviewed pilot dataset** and no new live-query capture collection from this milestone.

Three actual user questions in this conversation could be locally replayed later, explicitly marked `user_question_replay` and unreviewed: the project-assessment question, “wie gehts weiter” and “was ist der ROADMAP jetzt?!”. This is a possible smoke exercise, not work already performed, deployed traffic or independent evaluation. Do not publish private conversation/database material as fixtures. Do not promote replayed or assistant-reviewed cases into the 50-case gate.

The engineering implementation and its tests/benchmarks can be completed autonomously. If real human inputs remain absent, state that empirical gate as unmet and distinguish it from completed tooling. Never label the whole quality milestone passed merely to reach version 0.6.0.

## First actions when continuation is requested

Read this document, `AGENTS.md`, `CONTRIBUTING.md`, `ROADMAP.md` and `local-lab/pilot-0.6.0/PROGRESS.json`; inspect `git status` and the checkpoint commit. Check that retained logs/build inputs still exist. Continue at remaining-work item 1, then item 2. Do not repeat the already completed 0.5.3 release or start a benchmark before freezing its policy. No user clarification is needed for these implementation steps.
