# AIRust 0.4.5 validation

The release adds optional BM25 evidence delivery and citation validation through the existing `retrieval` feature. It does not change the default Engine ranking or add a DenkMaps integration. Read [selection](../EVIDENCE-SELECTION.md), [citations](../CITATIONS.md) and the [release report](../CHANGES-0.4.5.md) for limits and measured results.

## Independent Rust consumer

Copy `consumer.toml` to a private directory as `Cargo.toml`, and copy `consumer.rs` to its `src/main.rs`. The manifest uses the exact registry version, not an application adapter. It tests actual public API access, distinct BM25 evidence, exact quotation positions, unknown-reference rejection, strict null handling and projection into a retained document revision.

```bash
cargo +1.89.0 run --manifest-path /private/work/consumer/Cargo.toml
```

Before publication the same consumer uses an explicit local path. After publication its path override is removed, a fresh registry lock is generated and package source, version, VCS commit and checksum are verified. The shipped example is `cargo run --locked --features retrieval --example checked_evidence`.

## Project-document comparison

`benchmark-worker.rs` is compiled twice: against exact published 0.4.4 with its `candidate` Cargo feature off, and against 0.4.5 with that feature on. The feature belongs only to the private benchmark consumer, not AIRust. It permits one source file to compile against both APIs. Every default old/new response is compared before selecting the new behavior.

`benchmark.py` reuses the retained local-only Ollama/SQLite harness from `validation-honest-0.4.4/run.py`. It requires NumPy 2.5.5 in a private environment and the frozen private inputs. It does not scan arbitrary folders or download a model. Start the isolated local server as described in [the earlier method](../validation-honest-0.4.4/README.md), with cloud disabled and existing model files. Private data remains excluded from Git.

The corpus is the unchanged 72-file, 1,910-chunk snapshot. There are 210 assistant-authored document questions, not independently adjudicated user questions: 162 exposed development/regression cases and 48 fresh holdout cases. The development set combines the previous 100 questions with 62 questions initially frozen before implementation. Exploratory retrieval exposed those 62 and showed worse evidence recall with aggressive paragraph compaction; they therefore became development data. All initial outputs and the failed implementation snapshot are retained. No model answers were generated for that exploratory round.

The final 48 questions (36 positive, 12 unsupported) were frozen before implementing the coverage-preservation fix or inspecting their rankings. They use the same source corpus, not unseen projects. The final implementation preserves the original window's distinct query-term coverage and every retained phrase hint, or keeps that window. A stratified 72-answer assistant audit (24 questions × three arms, half holdout and half development) was selected before final rankings or replies were examined. This is anonymized assistant review, not independent human validation.

Arms are A: 0.4.4 BM25/prefix; C: 0.4.5 BM25/paragraph preference/distinct delivery; S: SQLite FTS5/BGE-M3 exact-cosine equal-weight RRF. Every arm has six hits, 100 candidates, 2,000 bytes per excerpt and 12,000 total text bytes. The source text and scope are identical. SQLite uses the same document embeddings retained from the previous run and fresh measured query embeddings. A and C do not require embeddings.

All 210 questions use all six arm orders 35 times in total. Each arm occupies each position 70 times and each pair precedes the other 105 times. Orders are stratified: 27 repeats in development and eight in holdout. Prompt caching is left enabled and real cache counters are reported. The same installed Qwen, seed, temperature, context and JSON schema apply to each arm. The new common prompt requests exact quotations, so these fresh timings/token counts are not directly interchangeable with the earlier ID-only-answer experiment.

The benchmark's new checker validates all arms against their actual delivered text. Its adapter first checks every passage against its canonical entry and project, then constructs a temporary immutable delivered-response catalogue. Checked snippet coordinates are projected back into exact original-entry bytes. No guessed first-occurrence location is used for an ambiguous quotation. This is an external test adapter, not a source-authority or application integration.

```bash
export AIRUST_BENCH_DIR="$PWD/local-lab/evidence-0.4.5/final"
local-lab/honest-comparison-044/venv/bin/python docs/validation-0.4.5/benchmark.py prepare
local-lab/honest-comparison-044/venv/bin/python docs/validation-0.4.5/benchmark.py retrieve
local-lab/honest-comparison-044/venv/bin/python docs/validation-0.4.5/benchmark.py answer
```

The retained exact Cargo locks and input/source hashes identify the actual binaries. Retrieval timing includes the source, excerpt preparation, serialization/IPC and shared context construction; worker subphase timings are also retained and have different scopes across implementations. End-to-end measurements include fresh SQLite query embedding, generation and final citation admission/checking. Technical quote verification does not establish answer correctness; automatic fact/reference checks and assistant audits remain explicitly separate proxies. Failure cases, length stops and invalid output are retained without score-based retries.

## Build artifacts

Release checks use one explicit `CARGO_TARGET_DIR` outside the project, with `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0` and `CARGO_INCREMENTAL=0`. Targeted Miri uses a separate temporary target. Release packages and receipts are archived before those regenerable directories are removed. Original PDFs, knowledge, question/answer data, source snapshots and measurement receipts are preserved. The cleanup receipt records exact paths and sizes; it never uses a broad deletion pattern over project data.
