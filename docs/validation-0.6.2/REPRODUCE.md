# Reproduce the 0.6.2 development checks

Use the checkout containing these files. It is not a published registry 0.6.2 release. Keep outputs in new task-owned directories; retain failures and raw evidence. Original models, databases and historical laboratory data are not build artifacts.

## Native and compatibility checks

Run `./test.sh` with Rust 1.92 or newer (the recorded stable build is 1.98.0) and Python 3.11 or newer. It runs actual CLI/Python/workflow checks, native tests, strict Clippy/rustdoc, every declared feature separately and package boundaries. Default and all-feature counts overlap. `cargo fmt --package airust` preserves the separately maintained backend.

Run `cargo +1.92.0 check --locked --all-features --all-targets`. For Rust 1.89, check all targets with the declared neutral features while excluding `local-embeddings`/`local-reranking`. Run the unchanged backend's fourteen regressions, C ABI, Rust 1.89 and `python3 -B scripts/check_pdf_backend.py` separately as required by CONTRIBUTING.md. Check API views against exact tag v0.6.1; the original source has embedded schemas/manifests/STARTUP dependencies, not only `.rs` files.

## Finite Miri

With the recorded installed nightly and Miri component:

```bash
python3 -B scripts/run_miri.py /tmp/new-airust-miri-results --toolchain nightly
```

The method freezes source identities, commands, flags, counts, failures and exclusions. It never downloads models. Full observed scope is twelve groups. A failed group remains failed; use a separate new directory for a corrected subset with `--groups model-worker` or `--groups pilot profile-config`. Preserve the original report and combine actual successful receipts explicitly. The reported result here is 128 tests across twelve groups, with the initial worker compile failure and replacement runs retained.

## Actual model lifecycle

Follow [the supported process example/controller](../LOCAL-MODEL-PROCESSES.md). Existing pinned MiniLM/runtime assets are required explicitly. The library does not download them. Keep two CPU threads, twenty candidates, outer batch one, thirty cycles, two warmup cycles and all slow observations. Run before/after serial controls without competing compiler/Miri/fuzz work. Never treat startup as a cold-cache result without an actual cache policy.

The public [prototype patch](../measurements/native-lifecycle-0.6.2/paired-prototype-from-v0.6.1.patch) plus exact v0.6.1 source reconstructs all 86 primary executable-input hashes. Its memory-worker path and the three allocation-policy patches are retained rejected experiments. The frozen [alternating controller](../measurements/native-lifecycle-0.6.2/alternating-method.py.txt) uses the recorded lab layout; restore it there with matching binary/fixture/model/runtime identities. It alternates first backend each cycle and holds two parents resident. Its passed cost diagnostic is separate from the failed reversed serial confirmation. All raw CSV and figure bytes are checked by SHA256SUMS.

Use `python3 -B scripts/export_model_lifecycle.py --lab /owned/retained-lab --output /owned/export` for the allowlisted numeric export. The plot source uses Matplotlib 3.10.8; the recorded local plotting environment is Python 3.14 with resolved wheel hashes retained privately. Plot generation is not a new measurement. No private replay questions/answers, passages or service inventory are exported.

## Protocol fuzzing

The target is `fuzz/fuzz_targets/model_worker.rs`, built with `--no-default-features --bin model_worker --target aarch64-apple-darwin` in the recorded nightly environment. Its real compiler flags are:

```text
--cfg fuzzing -Zsanitizer=address -Cpasses=sancov-module
-Cllvm-args=-sanitizer-coverage-level=4
-Cllvm-args=-sanitizer-coverage-inline-8bit-counters
-Cllvm-args=-sanitizer-coverage-pc-table
-Cllvm-args=-sanitizer-coverage-trace-compares
```

Build in a separate target directory. Preserve the ASAN dylib from that toolchain before deleting build outputs, verify its SHA, linked ASAN initializer and nonzero inline counters with a small probe. Then run one worker with `-seed=612025 -max_total_time=600 -timeout=8 -rss_limit_mb=1024 -max_len=65536 -reload=0 -detect_leaks=0 -print_final_stats=1`, a new corpus and an explicit artifact directory. The initial corpus includes valid Init/Request/Ready/Scores/Failure frames, malformed UTF-8, unknown fields, oversized data and concatenated frames. Bound retained corpus/log/artifact bytes and preserve the complete campaign without stitching partial durations. This is protocol fuzzing, not random native-model execution or a leak-absence proof.

## Extracted application controls and package

The preserved [core application](consumers/core/src/main.rs) and [native application](consumers/native/src/main.rs) use only public APIs and have separate Cargo manifests/locks. Restore `manifest.toml` as `Cargo.toml` and `consumer.lock` as `Cargo.lock`, then restore the recorded layout in a new directory: `core/`, `native/`, `extracted/airust-0.6.2/`. Extract the candidate crate into `extracted/`; the existing relative manifest paths then select those exact bytes. The native application also retains its public synthetic cases and baseline bit oracle. These are automated consumer controls, not independent human onboarding.

Run the core application on Rust 1.89 and inspect its normal dependency graph: no native model, HTTP, MCP or database. Build `airust-model-worker` from the extracted package on Rust 1.92 with only `local-reranking`. Build the separate native application and pass its worker executable, existing model directory, existing runtime library and a new owned snapshot parent. It executes eight load/score/drop cycles and checks all 160 baseline positions/bits, process reaping and private-copy removal. Its elapsed functional runtime is not a timing benchmark.

After all source checks, create a clean source commit and run `cargo package --locked` (offline if all dependencies are already cached). Inspect the archive and `.cargo_vcs_info.json`. Default features stay empty; backend dependency normalizes to exact `=0.3.5`. Exclude original databases/models/runtimes/lab data; the historical `tests/fixtures/compat-0.2.30/legacy.db` is an intentional unchanged test fixture. Verify packaged numeric-artifact checksums and source identity. No `cargo publish`, new tag, master merge or GitHub Release belongs to this engineering request.
