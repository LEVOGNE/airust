# Reproduce 0.3.0 validation

Run from a source checkout with Rust 1.92 or later and Python 3.11 or later:

```bash
./test.sh
python3 -B scripts/check_features.py --report /tmp/airust-features.json
```

The second command repeats only the feature checks and records each exit status. `test.sh` already includes them. Minimum-Rust checks also use 1.89 for the provider-neutral feature union and 1.92 for all features; see the checked-in CI workflow.

`consumer.rs` and `consumer.toml` are a standalone application, not another workspace member. Copy them to an empty temporary project's `src/main.rs` and `Cargo.toml`, then run `cargo +1.89.0 run --manifest-path PATH/Cargo.toml`. The exact `=0.3.0` dependency tests the published registry package. A local check adds `path = "/absolute/path/to/airust-master"` to the dependency temporarily. The consumer covers every provider-neutral release capability, including storage, snapshots, excerpts, ranking, reasoning, freshness, gates, training and disk retrieval.

The compatibility fixtures in `tests/fixtures/compat-0.2.30` were produced by unchanged v0.2.30 source, commit `33b3a0a4c5e4ab35b7dc0a47c99f1c8359d9d245`. `legacy-producer.rs` and `legacy-producer.toml` show the generator as a separate application. The actual run used a `git archive v0.2.30` checkout as its dependency. The registry manifest here pins the same published version for reproduction. Run it with a fresh output directory argument. Disk revisions include a fresh random identity, so regeneration does not reproduce the database bytes; the original fixture hashes and identities are preserved in the manifest. Release tests consume the existing files and never regenerate them.

## Native and scale measurements

Native assets are supplied explicitly and checked against the adapters' pinned manifests. Builds do not fetch them. With E5, the cross-encoder and a compatible ONNX runtime available, run:

```bash
cargo build --release --locked --features local-reranking,local-embeddings,hybrid,quality-gates,snapshots,execution,disk-index --example semantic_quality --example reranking_quality --example beir_evaluation --example stress_index --example stress_snapshots --example disk_benchmark
python3 scripts/measure.py /tmp/semantic.json target/release/examples/semantic_quality E5_DIRECTORY ONNX_RUNTIME_LIBRARY
python3 scripts/measure.py /tmp/reranking.json target/release/examples/reranking_quality RERANKER_DIRECTORY ONNX_RUNTIME_LIBRARY E5_DIRECTORY
python3 scripts/measure.py /tmp/scale.json target/release/examples/stress_index scale 2000000
python3 scripts/measure.py /tmp/snapshots.json target/release/examples/stress_snapshots 100000
python3 scripts/measure.py /tmp/disk.json target/release/examples/disk_benchmark 100000 disk
```

For independent lexical evaluation, provide the original SciFact or MTEB GermanQuAD mirror files in `corpus.jsonl`, `queries.jsonl`, `qrels/test.tsv` form. The [original source manifests](../measurements/0.2.26/) record download revisions and hashes. Run `python3 scripts/measure.py /tmp/quality.json target/release/examples/beir_evaluation DATASET_DIRECTORY`. The mirror has 474 German contexts, not the full Wikipedia collection. Compare per-query case identities, rankings and metrics against the recorded reference; latency is descriptive and excluded from identity equality.

Measurement binaries are optimized builds. The build record contains toolchain, feature set, binary hashes and ONNX runtime hashes. RSS is the peak of a single child including its measured phases, not OS file-cache usage. Runs performed alongside other validation are not controlled latency experiments.

## Publication gates

After local checks, create a clean release commit, run `cargo package --locked`, test the extracted crate, and run `cargo publish --locked --dry-run`. GitHub CI must pass for that exact commit. After publication, run the registry consumer with no path override and compare the registry SHA-256 and packaged VCS commit with the annotated release tag and GitHub crate asset. CI completion and registry verification happen after the source commit; they are recorded in the external release verification rather than falsely embedded as completed in an earlier commit.
