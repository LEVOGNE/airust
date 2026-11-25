# Reproduce the configuration and resource controls

Use a new owned output directory for every attempt. Retain failed commands and every slower result. Build all compared binaries before timing and stop compilers, Miri, fuzzers and unrelated inference for the measured phase. Record OS/CPU/RAM/power, exact source/lock/binary/method hashes and model/runtime identities. These procedures reproduce engineering probes; they are not independent human quality review.

## Complete native matrix and interpreter scope

From the candidate checkout, use a dedicated target directory and run:

```bash
export CARGO_TARGET_DIR=/tmp/airust-candidate-checks
export CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0
export CARGO_PROFILE_TEST_DEBUG=0
export CARGO_INCREMENTAL=0
./test.sh
python3 -B scripts/run_miri.py /tmp/new-airust-miri --toolchain nightly
```

Use an explicitly installed dated nightly for exact interpreter reproduction when available. The observed candidate uses nightly commit `5f4effd579500175af844241eb179e756190f1e7`. Keep all eleven group logs and the explicit native/large-fixture exclusions. A subset (`--groups input model-assets`) is useful for a changed helper but is labelled a subset. macOS directory synchronization remains native-only; do not change production durability code to make Miri accept it.

Run minimum-compiler/backend/audit controls described in [the validation record](README.md) and [CONTRIBUTING](../../CONTRIBUTING.md). Use `cargo fmt --package airust`; backend provenance and formatting have their own checker. Clean packaging and extracted-consumer checks belong after the final source commit. A `cargo publish --dry-run` does not authorize publication.

## Paired complete MCP reranking

Build one immutable published-0.6.0 source archive and the candidate with the same `pilot,local-reranking` features, release thin LTO, one codegen unit and compatible dependency locks. Copy the resulting `reference_workspace` binaries before another build can replace them. The baseline source is `8c098f15a39f6085f748c56ac52e5737abdff423`. New candidate settings must not be passed to the old reader, which rejects unknown fields.

Verify the frozen corpus and import the original source directory into a new workspace:

```bash
python3 -B scripts/reference_dataset.py
/absolute/candidate-reference_workspace build \
  tests/fixtures/reference-v1/sources /tmp/new-reference-workspace airust-reference
```

Use an already installed, explicitly chosen loopback embedding service and supplied model/runtime assets. This procedure does not download models or launch a service. Follow [profile preparation](../PILOT.md#explicit-live-profiles) to create the workspace vector cache. Keep the original source, BGE-M3 digest, 1024 dimensions, MiniLM manifest/revision and max512 pair recipe. Record all identities; a differently generated cache/workspace is a new workload, not the historical samples.

Create two full schema-one strict `hybrid_rerank` profiles. Both must use the same vectors, endpoint, digest, model/runtime and 20-candidate prefix. In the baseline omit the new candidate/batch/snapshot fields and select 2 threads. In the candidate explicitly select `batch_size: 1`, `candidates: 20` and either 2 or 8 threads. Optional `model_snapshot_directory` must be an existing application-owned directory with at least 470883696 bytes plus filesystem overhead available. Preflight the candidate using `profile-check`; this checks configuration, not asset/service readiness.

```bash
python3 -B scripts/compare_rerank_profiles.py \
  --baseline /absolute/baseline-reference_workspace \
  --candidate /absolute/candidate-reference_workspace \
  --workspace /tmp/new-reference-workspace \
  --baseline-profile /absolute/baseline.json \
  --candidate-profile /absolute/candidate.json \
  --output /tmp/new-paired-request-evidence
```

The script runs two sequential campaigns with reversed build order, one initial request and two timed requests per each of 150 authored cases. `--repetitions 1` is a separately identified shorter confirmation. It preserves raw samples, source-checked results, startup, peak worker RSS, every slower query and every nonzero returned-logit difference. Gates are unchanged top-six IDs/order and all non-logit response fields, no known-quote losses, abs `1e-4` + relative `1e-5` logit tolerance and at least 20% p95 improvement in **both** campaigns. Keep a failed campaign; do not select only successful settings/runs. Check that normal scorer drop removes every owned snapshot. External service memory and answer-generation costs require separate accounting.

## Static tokenizer and padding reproduction

The token-size analysis needs the four installed pinned tokenizer/config files, without ONNX weights, runtime or an embedding service. Candidate order is taken from the observed public pair-coordinate CSV; this procedure reproduces that structural analysis, not the retrieval step or latency measurement. Verify the published checksums first.

Create an unpublished external Cargo project from [token-padding-probe.rs](token-padding-probe.rs) as `src/main.rs`, [token-padding-probe.toml](token-padding-probe.toml) as `Cargo.toml` and [token-padding-probe.lock](token-padding-probe.lock) as `Cargo.lock`. Build with `cargo build --release --locked` in a dedicated target directory. The observed toolchain is Rust 1.98.0; this optional analysis tool is separate from the library's minimum-compiler commitment.

Import the frozen reference source directory as shown above. Then reconstruct candidate text from the CSV's document/revision/byte coordinates, using the independently validated workspace:

```bash
python3 -B scripts/prepare_padding_prefixes.py \
  docs/measurements/profile-practical-0.6.0/token-padding-pairs.csv \
  /tmp/new-reference-workspace /tmp/new-padding-prefixes.json
/absolute/airust-padding-probe /tmp/new-padding-prefixes.json \
  /absolute/mmarco-minilm-v2 docs/models/mmarco-minilm-v2.json \
  /tmp/new-token-padding-evidence
```

The preparer validates all 150 frozen queries, exactly twenty distinct candidate positions each, immutable source bytes/revisions, passage-ID framing and query/text hashes before creating a new JSON file. It intentionally omits unused entry metadata, so that file's SHA differs from the historical full-prefix file. Pair/batch output observations match byte-for-byte. The probe validates asset lengths/SHA-256, bounds input/pair sizes, preserves 512-token longest-first truncation and compares every batch's non-padding IDs/masks with single-pair encoding. It produces no inference timing. The squared-length proxy is not a FLOP/latency/energy/quality measurement.

## Training snapshot sharing and first writes

[training-snapshot-probe.rs](training-snapshot-probe.rs) is the measured harness with formatting only. Create a small unpublished external Rust consumer for each source archive, with `default-features = false` on the `airust` dependency and only `sha2 = "0.10"` / `serde_json = "1"` as probe dependencies. Copy the probe to `src/main.rs`. Use `[profile.release] lto = "thin"` and `codegen-units = 1`. Keep both generated locks and verify the same non-AIRust dependency versions before comparing. Build and retain both release binaries before timing.

Arguments are `RECORDS TEXT_WIDTH SNAPSHOTS EDITED_SNAPSHOTS`:

| Scenario | Arguments |
| --- | --- |
| Read-only | `6000 1536 24 0` |
| One first write | `6000 1536 24 1` |
| Every snapshot written | `6000 1536 8 8` |
| Small input | `200 128 24 0` |

On macOS, capture `/usr/bin/time -l /absolute/probe 6000 1536 24 0` stdout/stderr separately. On another OS use its exact-child peak-memory tool and document units/accounting rather than equating unlike counters. Run two campaigns with five fresh processes per build/scenario and alternate/randomize build order before execution. Compare framed signatures for every pair. Report clone, first-edit and combined latency, plus peak RSS; do not hide higher first-write cost or the all-writers case. The declared read-only candidate observation budget is 128 MiB worker peak on this host, not an OS quota.

## Retained rejection errors

The public `examples/retained_errors.rs` works as an external consumer of both 0.6.0 and the candidate with `mcp`, without a model. Keep its source identical across builds; select each of `utf8`, `mcp-json`, `training-json`, `engine-json` and `document-json`. Each fresh process retains 128 errors from a 2-MiB input and checks typed categories, JSON positions and unchanged original bytes. Run three processes per mode/build; retain the captured diagnostic lengths and kernel peak RSS. The fixed-candidate observation budget is 32 MiB. Historical pre-fix negative controls are evidence, not a source tree to recommend for use. Returned diagnostic bounds do not eliminate transient parser allocations.

## Small model-free stdio application

```bash
cargo build --locked --release --no-default-features --features mcp --example bounded_mcp
python3 -B scripts/benchmark_small_host.py \
  "$CARGO_TARGET_DIR/release/examples/bounded_mcp" /tmp/new-small-host-evidence
```

Default probes use two campaigns, three deterministic 2048-entry datasets, 32 initial/warmup requests and 1000 timed requests per worker. Keep cold-start/first-request values and all sampled RSS. Budgets are 512 MiB worker peak and 32 MiB observed post-warm growth, with the exact admission/input/output/thread limits in [RESOURCE-LIMITS](../RESOURCE-LIMITS.md). The script uses `ps` for samples and the MCP client's exact-child exit/peak receipt. This does not impose a kernel memory limit. A real 1-GiB device, OS overhead and long-lived production workload are separate unestablished checks.

For the denser-vocabulary follow-up, explicitly pass `--workloads unicode high-vocabulary long-token short-unique`. The added dataset has 393216 distinct four-character terms in 2048 entries and stays within the unchanged 2-MiB dataset-byte bound. It is a separately identified stress case; do not rewrite the earlier three-workload samples or treat input-byte limits as a heap-size guarantee.

## Optional macOS allocator launch policy

The repeated-load default-allocator failures and the separate `MallocSpaceEfficient=1` lifecycle experiment are retained in [the resource guide](../RESOURCE-LIMITS.md). Measure its steady request/startup tradeoff separately with the **same binary and profile**:

```bash
python3 -B scripts/benchmark_native_allocator.py \
  --binary /absolute/candidate-reference_workspace \
  --workspace /tmp/new-reference-workspace \
  --profile /absolute/candidate.json \
  --output /tmp/new-allocator-request-evidence
```

This macOS-only script launches child processes through `/usr/bin/env`, clearing the flag for the default arm and setting it only for the explicit experiment. The library and parent environment are unchanged. Two campaigns reverse launch order, retaining the same 150 authored cases, initial requests, source/rank checks, peak worker RSS, every slower query and logit difference. The predeclared optional-policy threshold is at most 10% warm p95 overhead in both campaigns; a failed threshold stays failed and does not stop unrelated candidate validation. This request experiment does not replace the repeated-drop lifecycle controls or establish whole-system/GPU/energy accounting.

## Evidence interpretation

The [numeric exports and figures](../measurements/profile-practical-0.6.0/README.md) preserve every sample from their declared cohorts. Private replayed questions and local LLM answers are intentionally excluded from public exports and remain retained locally. Frozen authored questions, generated resource probes, exact source checks and self-assessed model answers do not become genuine independent customer traffic or human labels. Independent onboarding, at least 50 eligible independently reviewed real questions, clean-machine integration and external reproduction remain open.

Use the packaged [SHA256SUMS](../measurements/profile-practical-0.6.0/SHA256SUMS) to verify CSV and figure bytes. The detailed JSON measurement manifest is retained in Git and intentionally excluded from the crate, following the existing package policy. The six SVG/PNG figures reproduce byte-for-byte in two renders on the recorded host/dependency/font setup; that is not a cross-host Rust-binary reproducibility claim.
