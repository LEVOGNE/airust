# AIRust test plan and resource-aware test control

This plan decides which checks run after a change, which deep checks are due, and which earlier results may be reused. It complements the release gates in [CONTRIBUTING.md](../CONTRIBUTING.md) and the measurement rules in [AGENTS.md](../AGENTS.md); it does not replace release validation receipts.

`docs/testplan.json` is the single machine-readable source of areas, inputs, triggers, commands, budgets and maximum coverage ages. `scripts/testctl.py` executes it. The area table below is generated from the JSON (`testctl.py plan --write-doc`); the `controller` area fails when it drifts.

## Inventory (recorded 2026-11-26)

| Item | State |
| --- | --- |
| Crates | `airust` (library, CLI/MCP/model-worker binaries, 78 examples, 57 Rust integration-test files, 8 Python workflow tests); separate `crates/airust-pdf-backend` 0.3.5 with its own lock; separate `fuzz/` crate (4 libFuzzer targets, own lock). |
| Module dependencies | One library crate with 40 Cargo features and an empty default set. Feature edges are declared in `Cargo.toml` and checked by `scripts/check_support.py`, `scripts/check_features.py` and `scripts/check_mcp_dependencies.py`. Changes are therefore scoped conservatively at crate level: any source, test, example or fixture change invalidates every check of the main crate. |
| Platforms | Local: macOS arm64 (`aarch64-apple-darwin`), stable, nightly (Miri, ASAN fuzzing), 1.89.0 (MSRV), 1.92.0 (native local models). Hosted CI (`.github/workflows/ci.yml`): Ubuntu + macOS 15 arm64 run `test.sh`, MSRV, Rust 1.92 and backend jobs on every push. Windows and Intel are not available. |
| `unsafe` | Owned library code uses `#![forbid(unsafe_code)]`; the fuzz model-worker target also forbids it. Native FFI lives in dependencies (ONNX Runtime, SQLite, PDF backend C ABI) and is outside Miri. |
| Parsers / decoders / import | Knowledge/training JSON, bounded input reader, documents, lexical cache files, disk index, PDF import and page tree, MCP framing, model-worker protocol. |
| Persistence | `src/persistence.rs` (atomic publish), caches, snapshots, SQLite storage, disk index, web database. |
| Own synchronisation | Shared index, snapshots publication lock, execution/cancellation, MCP transport/session state, worker process transport, local runtime environment lock. No Loom models exist; repeated release-mode runs vary interleavings instead. |
| Property-style tests | Seeded/permutation tests (`tests/knowledge_snapshots.rs`, reranking permutation and seeded reference cases, stability state). There is no `proptest` dependency; none is added without a concrete purpose. |
| Fuzz regressions | Minimized seeds in `tests/fixtures/fuzz-seeds/v0.5.2/` are replayed by stable `cargo test` (`tests/fuzz_seeds.rs`, `tests/fuzz_seeds_pdf.rs`). |
| Not available | `cargo-fuzz` (targets are built directly with nightly sanitizer flags), Loom, `cargo-audit` (advisories were checked from a cloned RustSec database during releases). |

## Profiles

All commands run from the repository root. Python runs through uv; the controller itself uses only the standard library.

```bash
uv run --no-project scripts/testctl.py quick              # after every finished change
uv run --no-project scripts/testctl.py quick --candidate  # before merging: clean tree, whole quick suite
uv run --no-project scripts/testctl.py nightly            # due deep checks, 90 min budget, one focus
uv run --no-project scripts/testctl.py full               # weekly and before releases
uv run --no-project scripts/testctl.py status             # last results and next due dates
uv run --no-project scripts/testctl.py plan --check       # validate plan and this document
```

- **quick** runs the quick suite (format, both Clippy sets, default and all-feature tests, contracts, controller, PDF backend). Checks whose relevant inputs and configuration are byte-identical to their last success are reported as **reused**, never as executed. Changed paths are compared with the last fully successful quick baseline (or with `HEAD` when there is none); matching risk triggers run immediately: short fuzz runs for parser/decoder/import changes, the concurrency suite for synchronisation/persistence changes, and Miri groups for any `unsafe` (or lifted `unsafe_code` lint) in a covered file. An `unsafe` change outside every Miri group is reported as a gap. Target: 1 to 5 minutes once the dedicated target directory is warm.
- **nightly** works on a detached git worktree of a fixed commit (default `HEAD`); local uncommitted changes are neither used nor modified. It ranks due deep checks: failed/incomplete first, then starved coverage (older than twice the maximum age), then new risk (inputs, configuration or toolchain changed) and never-run checks, then expired coverage. It picks one compute-intensive focus (Miri, fuzzing, concurrency or stress) and runs that kind's due items while their measured (or estimated) duration fits the remaining budget of 90 minutes including compilation. Everything else stays visibly **DUE** and gains priority with age. With nothing due it exits without an expensive run.
- **full** is the independent control: every quick and full check executes fresh (no reuse), including the complete `test.sh`, MSRV 1.89/1.92 and backend ABI/provenance checks, followed by due deep checks across kinds within 120 minutes. Benchmarks are not part of any profile; measure them separately under quiet, comparable conditions per [AGENTS.md](../AGENTS.md).

For `BIN WEG <duration>` sessions run `full`, then `nightly --budget-minutes <remaining>` repeatedly; the 90-minute limit does not apply there.

## Reuse and invalidation rules

- An area's fingerprint covers the plan definition of that area and configuration, the controller, the content hashes of every relevant input file (crate-level groups in the JSON, including tests, fixtures, examples, `build.rs`, `Cargo.toml`, `Cargo.lock`, toolchain files), the `rustc`/`cargo`/Miri/Python versions actually used, the host triple and relevant environment variables (`RUSTFLAGS`, `RUSTDOCFLAGS`, …). File names, timestamps or a global commit ID alone never decide reuse. Toolchain or dependency updates therefore invalidate the affected results automatically.
- Only deterministic checks are reused, and only when the last attempt is the last success with an identical fingerprint and its coverage is younger than the maximum age. Fuzzing, stress and the repeated concurrency suite are variation checks: they are never reused; they become due after their maximum age or on input change, and fuzz runs always use a new random seed while the per-target corpus in `.testctl/fuzz/corpus/` accumulates.
- A failed, incomplete or unavailable check is never green. Quick checks are re-executed every time until they pass. A deep check that failed on unchanged inputs is reported as **FAILING** (known failure) instead of burning the budget again; any relevant change makes it top priority.
- A missing or damaged status file preserves the damaged copy and reuses nothing, so the broad quick suite runs.
- Missing tools (nightly, Miri, a pinned toolchain) are reported as **unavailable** gaps; the run fails.
- A test area that executes zero tests is a failure (filter/target mismatch guard).
- Bounded fuzzing reports "no crash found within the search budget", never correctness. Cumulative fuzz seconds are recorded per target and per fingerprint.

## Run status and logs

`.testctl/` (git-ignored, never committed) holds `status.json` (atomic write + fsync), deduplicated input manifests, one JSON report per run in `runs/`, per-area logs in `logs/<run>/`, fuzz corpora/artifacts, dedicated target directories and the lock that refuses parallel duplicate runs. Each status entry records last attempt, last success, fingerprint, input manifest, uncommitted relevant inputs and whether an isolated worktree was used, configuration and environment, commands, tool versions, timestamps, duration, result, test counts, log path and next due date. Hosted CI does not run the controller yet; to retain status between CI runs, cache `.testctl/status.json` and `.testctl/manifests/` with `actions/cache` keyed per branch.

## Regressions and findings

- Every fixed bug keeps a regression test in the matching test file.
- A fuzz crash keeps its artifact in `.testctl/fuzz/artifacts/<target>/<run>/`. Reproduce with the logged binary command and the artifact path, fix the cause, then copy the minimized input into `tests/fixtures/fuzz-seeds/` (update the seed manifest) so stable `cargo test` replays it forever.
- Never relax an assertion or a requirement to make a check green.

## Scheduling

No automatic nightly trigger is installed. Until one is set up explicitly, start `nightly` manually (or during `BIN WEG`). Options: a user `launchd` agent on this Mac calling `uv run --no-project scripts/testctl.py nightly`, or a scheduled GitHub workflow with a cached `.testctl/` (Linux needs no `DYLD_LIBRARY_PATH` for the ASAN runtime). Neither is active.

## Areas

<!-- testctl:areas:start -->
| Area | Config | Kind | Profiles | Max age | Trigger paths | Commands | Purpose |
| --- | --- | --- | --- | --- | --- | --- | --- |
| fmt | default | check | full, quick | input change | - | `cargo fmt --package airust -- --check` | Formatting of the main crate (never --all; the backend has its own gate). |
| clippy-default | default | check | full, quick | input change | - | `cargo clippy --workspace --all-targets --locked -- -D warnings` | Strict lints for the empty default feature set and all targets. |
| clippy-all | default | check | full, quick | input change | - | `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | Strict lints with every feature enabled. |
| test-default | default | test | full, quick | input change | - | `cargo test --workspace --locked` | Unit, integration, regression, fuzz-seed replay and doctests with default features. |
| test-all | default | test | full, quick | input change | - | `cargo test --workspace --all-features --locked` | Unit, integration, regression, property-style, fuzz-seed replay and doctests with every feature. |
| contracts | default | check | full, quick | input change | - | `{python} -B scripts/check_support.py`<br>`{python} -B scripts/check_mcp_dependencies.py` | Architecture and dependency rules: feature/support matrix, entry-point links, MCP dependency boundary. |
| controller | default | check | full, quick | input change | - | `{python} -B tests/testctl_test.py`<br>`{python} -B scripts/testctl.py plan --check` | Selection, status persistence and failure handling of this test controller. |
| pdf-backend | default | test | full, quick | input change | - | `cargo test --manifest-path crates/airust-pdf-backend/Cargo.toml --test regressions --locked` | Maintained PDF backend 0.3.5 regressions (separate crate and lock). |
| concurrency | short | concurrency | full, nightly, quick (trigger) | input change | `src/persistence.rs`, `src/snapshots.rs`, `src/storage.rs`, `src/storage/**`, `src/execution.rs`, `src/mcp.rs`, `src/mcp/**`, `src/reranking/process/**`, `src/local_runtime.rs`, `src/index.rs`, `src/index/**`, `src/cache.rs`, `src/web/state.rs`, `tests/stability_state.rs`, `tests/shared_index.rs`, `tests/snapshots.rs`, `tests/persistence.rs`, `tests/execution.rs`, `tests/progress.rs`, `tests/stress.rs` | `cargo test --all-features --locked --test stability_state --test shared_index --test snapshots --test persistence --test execution --test progress --test stress`<br>`cargo test --all-features --locked --lib -- persistence:: mcp:: reranking::process:: snapshots:: local_runtime::` | Own synchronisation: snapshots, shared index, persistence, execution, MCP transport and worker process lifecycle. Deep config repeats to vary interleavings. |
| concurrency | deep | concurrency | full, nightly | 7 d | `src/persistence.rs`, `src/snapshots.rs`, `src/storage.rs`, `src/storage/**`, `src/execution.rs`, `src/mcp.rs`, `src/mcp/**`, `src/reranking/process/**`, `src/local_runtime.rs`, `src/index.rs`, `src/index/**`, `src/cache.rs`, `src/web/state.rs`, `tests/stability_state.rs`, `tests/shared_index.rs`, `tests/snapshots.rs`, `tests/persistence.rs`, `tests/execution.rs`, `tests/progress.rs`, `tests/stress.rs` | `cargo test --release --all-features --locked --test stability_state --test shared_index --test snapshots --test persistence --test execution --test progress --test stress`<br>`cargo test --release --all-features --locked --lib -- persistence:: mcp:: reranking::process:: snapshots:: local_runtime::` (x25) | Own synchronisation: snapshots, shared index, persistence, execution, MCP transport and worker process lifecycle. Deep config repeats to vary interleavings. |
| stress | deep | stress | full, nightly | 14 d | - | `cargo run --release --locked --example stress_index -- scale 10000`<br>`cargo run --release --locked --example stress_index -- clone 10000`<br>`cargo run --release --locked --features snapshots,execution --example stress_snapshots -- 10000` | Load and duration: existing stress examples at ten times the CI scale. |
| miri-input | deep | miri | full, nightly | 7 d | `src/input.rs` | `{python} -B scripts/run_miri.py {log_dir}/receipt --groups input` | Miri: bounded input reader. |
| miri-model-assets | deep | miri | full, nightly | 7 d | `src/model_assets.rs` | `{python} -B scripts/run_miri.py {log_dir}/receipt --groups model-assets` | Miri: model asset size/checksum verification. |
| miri-model-worker | deep | miri | full, nightly | 7 d | `src/reranking/process/**`, `src/reranking.rs` | `{python} -B scripts/run_miri.py {log_dir}/receipt --groups model-worker` | Miri: worker protocol, input and timeout validation (no OS processes, no FFI). |
| miri-training-knowledge | deep | miri | full, nightly | 7 d | `src/knowledge.rs`, `src/agent.rs` | `{python} -B scripts/run_miri.py {log_dir}/receipt --groups training-knowledge` | Miri: knowledge base loading and copy-on-write training data. |
| miri-training-properties | deep | miri | full, nightly | 7 d | `src/knowledge.rs`, `tests/knowledge_snapshots.rs` | `{python} -B scripts/run_miri.py {log_dir}/receipt --groups training-properties` | Miri: knowledge snapshot property workload (reduced under cfg(miri)). |
| miri-index | deep | miri | full, nightly | 7 d | `src/index.rs`, `src/index/**`, `src/text.rs`, `src/text_search.rs` | `{python} -B scripts/run_miri.py {log_dir}/receipt --groups index` | Miri: lexical index, cursors and query weights. |
| miri-core | deep | miri | full, nightly | 7 d | `src/index.rs`, `src/index/**`, `src/text.rs`, `src/search.rs`, `src/matching.rs`, `src/ranking.rs`, `tests/core.rs` | `{python} -B scripts/run_miri.py {log_dir}/receipt --groups core` | Miri: core engine integration tests. |
| miri-stability-state | deep | miri | full, nightly | 7 d | `src/index.rs`, `src/index/**`, `src/execution.rs`, `tests/stability_state.rs` | `{python} -B scripts/run_miri.py {log_dir}/receipt --groups stability-state` | Miri: shared-index state stability (reduced under cfg(miri)). |
| miri-mcp-response | deep | miri | full, nightly | 7 d | `src/mcp.rs`, `src/mcp/**` | `{python} -B scripts/run_miri.py {log_dir}/receipt --groups mcp-response` | Miri: MCP response shaping and byte budgets. |
| miri-mcp-backend | deep | miri | full, nightly | 7 d | `src/mcp.rs`, `src/mcp/**` | `{python} -B scripts/run_miri.py {log_dir}/receipt --groups mcp-backend` | Miri: MCP search backend hook. |
| miri-pilot | deep | miri | full, nightly | 7 d | `examples/reference_workspace.rs`, `examples/reference_workspace/**` | `{python} -B scripts/run_miri.py {log_dir}/receipt --groups pilot` | Miri: reference workspace and pilot small-fixture tests. |
| miri-profile-config | deep | miri | full, nightly | 7 d | `examples/reference_workspace.rs`, `examples/reference_workspace/**` | `{python} -B scripts/run_miri.py {log_dir}/receipt --groups profile-config` | Miri: reference profile configuration tests. |
| fuzz-lexical | short | fuzz | full, nightly, quick (trigger) | input change | `src/index.rs`, `src/index/**`, `src/text.rs`, `src/text_search.rs`, `src/search.rs`, `src/retrieval.rs`, `src/retrieval/**`, `src/documents.rs`, `src/cache.rs`, `src/execution.rs`, `tests/support/fuzz_lexical.rs`, `fuzz/**` | libFuzzer `lexical` for 60 s (ASAN, nightly) | Fuzz: lexical index, retrieval, documents and damaged caches. |
| fuzz-lexical | deep | fuzz | full, nightly | 7 d | `src/index.rs`, `src/index/**`, `src/text.rs`, `src/text_search.rs`, `src/search.rs`, `src/retrieval.rs`, `src/retrieval/**`, `src/documents.rs`, `src/cache.rs`, `src/execution.rs`, `tests/support/fuzz_lexical.rs`, `fuzz/**` | libFuzzer `lexical` for 1200 s (ASAN, nightly) | Fuzz: lexical index, retrieval, documents and damaged caches. |
| fuzz-knowledge | short | fuzz | full, nightly, quick (trigger) | input change | `src/knowledge.rs`, `src/agent.rs`, `src/input.rs`, `tests/support/fuzz_knowledge.rs`, `fuzz/**` | libFuzzer `knowledge` for 60 s (ASAN, nightly) | Fuzz: bounded training/knowledge JSON input. |
| fuzz-knowledge | deep | fuzz | full, nightly | 7 d | `src/knowledge.rs`, `src/agent.rs`, `src/input.rs`, `tests/support/fuzz_knowledge.rs`, `fuzz/**` | libFuzzer `knowledge` for 1200 s (ASAN, nightly) | Fuzz: bounded training/knowledge JSON input. |
| fuzz-model-worker | short | fuzz | full, nightly, quick (trigger) | input change | `src/reranking/process/protocol.rs`, `fuzz/**` | libFuzzer `model_worker` for 60 s (ASAN, nightly) | Fuzz: production worker framing, strict serde and diagnostic truncation. |
| fuzz-model-worker | deep | fuzz | full, nightly | 7 d | `src/reranking/process/protocol.rs`, `fuzz/**` | libFuzzer `model_worker` for 900 s (ASAN, nightly) | Fuzz: production worker framing, strict serde and diagnostic truncation. |
| fuzz-pdf | short | fuzz | full, nightly, quick (trigger) | input change | `src/pdf_loader.rs`, `src/pdf_page_tree.rs`, `tests/support/fuzz_pdf.rs`, `tests/support/pdf_fixture.rs`, `fuzz/**` | libFuzzer `pdf` for 60 s (ASAN, nightly) | Fuzz: bounded PDF import and page tree. |
| fuzz-pdf | deep | fuzz | full, nightly | 7 d | `src/pdf_loader.rs`, `src/pdf_page_tree.rs`, `tests/support/fuzz_pdf.rs`, `tests/support/pdf_fixture.rs`, `fuzz/**` | libFuzzer `pdf` for 1500 s (ASAN, nightly) | Fuzz: bounded PDF import and page tree. |
| suite | default | suite | full | input change | - | `bash test.sh` | Complete existing CI suite (test.sh): examples, Python workflows, isolated feature checks, rustdoc and package boundary. |
| msrv | default | check | full | input change | - | `cargo +1.89.0 check --all-targets --features web,bots,async,colors,plotting,embeddings,hybrid,graph,context-assembly,provenance,cache,hnsw,versioning,quality,reranking,advanced-search,storage,sqlite-storage,retrieval,snapshots,execution,shared-index,adaptive,feedback,learned-ranking,reasoning,quality-gates,evidence-freshness,rank-explanations,training-pipeline,disk-index,documents,mcp,pilot --locked`<br>`cargo +1.92.0 check --all-features --all-targets --locked` | Minimum supported Rust 1.89 with the CI feature list; Rust 1.92 for native local models. |
| pdf-backend-full | default | check | full | input change | - | `cargo check --manifest-path crates/airust-pdf-backend/Cargo.toml --features c-api --lib --locked`<br>`{python} -B scripts/check_pdf_backend.py`<br>`cargo +1.89.0 check --manifest-path crates/airust-pdf-backend/Cargo.toml --lib --locked` | PDF backend C ABI, provenance and its own minimum Rust. |
<!-- testctl:areas:end -->

## Remaining coverage gaps

- No Loom models; synchronisation relies on repeated release-mode tests and existing stress examples.
- Miri excludes native FFI, OS subprocesses, HTTP/Tokio paths and large workloads (see `scripts/run_miri.py` exclusions).
- Fuzzing is instrumented with ASAN only; LeakSanitizer is off on macOS and native libraries are not instrumented.
- Windows and Intel targets are unavailable; Linux runs only in hosted CI.
- Web UI behaviour has no browser end-to-end tests.
