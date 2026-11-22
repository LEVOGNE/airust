# AIRust 0.4.6 validation and reproduction

The new source options are explicit under `retrieval`; Engine BM25, literal source behavior and MCP defaults are unchanged. Read [query preparation](../LEXICAL-QUERY.md), [excerpt positions](../EXCERPTS.md) and the [release report](../CHANGES-0.4.6.md).

## Separate Rust application

Copy `consumer.toml` to a private folder as `Cargo.toml` and `consumer.rs` to its `src/main.rs`. The manifest pins exact registry 0.4.6 with only `retrieval,documents`. Run `cargo +1.89.0 run --manifest-path /absolute/private/folder/Cargo.toml`. Before publication the same application uses a local path; afterwards a fresh registry lock verifies actual package source, version, VCS commit and checksum. It exercises query inspection, explicit source preparation, surrounding evidence, old paragraph behavior, bounded quotation checks and projection into retained document snapshots.

## Frozen document comparison

The corpus is the unchanged private snapshot of 72 Markdown files, 2,755,468 bytes and 1,910 chunks from AIRust, Flasm and Vecasm. This is document retrieval, not a new large-PDF import benchmark. Original PDFs and canonical knowledge remain intact.

All 210 questions used in 0.4.5 are now exposed development data. Development probes compare query filtering/identifier splitting and diversity penalties. Diversity produced little improvement and was not added. A separate set of 72 questions was frozen before 0.4.6 implementation or inspection of its new rankings: 54 positive and 18 intentionally unsupported, 24 per project. There are 12 English and 60 German questions in that new set; previous questions are German. Authorship and reference labels come from the same assistant, not independent human adjudication or real user logs. These are new questions about known source files, not unseen projects. Full frozen inputs and all failed attempts are retained privately.

Three arms receive the same project scope, six passages, 100 source candidates, 2,000 bytes per passage and 12,000 total text bytes:

- A: exact published 0.4.5 BM25 with ordinary prefix delivery.
- C: 0.4.6 BM25 with explicit question/function-word filtering, protected quoted/scoped terms, bounded ASCII identifier expansion, conservative leading context and exact-excerpt deduplication.
- S: SQLite FTS5 plus BGE-M3 exact cosine, equal-weight reciprocal-rank fusion with smoothing 60.

Rust receives the original case-preserved question in all arms; ordinary tokenization of A is checked against the common normalized lexical question. S uses that common normalization for FTS5 and the original question for embeddings. C intentionally changes its effective lexical query and exposes the rule/limits in its recipe; this is a comparison of complete configurations, not an isolated algorithm attribution. Default old/new Rust retrieval JSON is also compared independently for every question. SQLite's retained document vectors have verified model and file digests; timed end-to-end S replies include fresh query embeddings.

The worker is compiled twice from `benchmark-worker.rs`: use `worker-baseline.toml` against 0.4.5 and `worker-candidate.toml` against 0.4.6. Copy each manifest to a private folder as `Cargo.toml` beside the source renamed `worker.rs`. The `candidate` feature belongs to the benchmark consumer only. Set `CARGO_TARGET_DIR=/private/tmp/airust-relevance-046-build`; build the baseline with `cargo build --release` and candidate with `cargo build --release --features candidate`. Retain exact Cargo locks; the measured candidate uses an explicit path before publication, the public reconstruction pins the registry version. Measured binaries use the recorded compiler and default consumer release profile. Another machine/compiler will not reproduce exact timing.

`benchmark.py` reuses the public local-only API/SQLite harness in `validation-honest-0.4.4/run.py`. The original 0.4.5 files and measurements are unchanged. It requires the retained private frozen `plan.json`, `questions.json`, `chunks.json`, `inventory.json`, `frozen-sha256.json`, existing model files, prior verified document vectors/embedding manifest and the NumPy environment. No arbitrary directory scan or model download occurs.

```bash
export AIRUST_BENCH_DIR="$PWD/local-lab/relevance-0.4.6"
local-lab/honest-comparison-044/venv/bin/python -B docs/validation-0.4.6/benchmark.py prepare
local-lab/honest-comparison-044/venv/bin/python -B docs/validation-0.4.6/benchmark.py retrieve
local-lab/honest-comparison-044/venv/bin/python -B docs/validation-0.4.6/benchmark.py answer
local-lab/honest-comparison-044/venv/bin/python -B docs/validation-0.4.6/analyze.py
```

Use the same installed Qwen3.6 35B Q4_K_M and BGE-M3 F16 as before, with cloud disabled, private loopback port 11464, parallelism one and at most two resident models. The original daemon on 11434 is untouched. All 282 questions run all three arms, for 846 replies, with the identical quote-object prompt/schema, temperature zero, seed 461106, thinking disabled, 16,384 context tokens and 512 output tokens. Each of six arm orders occurs 47 times; each arm occupies each position 94 times and precedes each other arm 141 times. Orders are stratified into 35 repeats on development and 12 on holdout. Prompt caching remains enabled and returned cache counters are retained. Do not compare absolute reply timings against earlier different seeds or prompts as if they were paired runs.

All arms use the same literal citation checker against their actual delivered passages. The temporary adapter first verifies source ID, project and exact canonical range, then checks the immutable catalogue and projects verified coordinates into original entry bytes. Exact quotation occurrence does not prove entailment, source authenticity, freshness or permissions. Admission/check subphase and full adapter/IPC wall times have separate records.

Five warm retrieval repeats per question/arm measure source work, excerpt construction, serialization/IPC and shared context construction. The S retrieval-only repeats use prepared vectors and exclude query inference; end-to-end reply measurements include fresh S query embedding, retrieval, generation and quote checking. Worker internal subphase scopes differ. Whole-worker RSS is not isolated index heap; Ollama RSS and loaded GPU/unified-memory residency must not be added as independent allocations.

A 72-reply same-assistant audit over 24 stratified questions is selected before implementation/rankings. Half are development and half holdout, with three positive and one unsupported question per project/partition. Variant labels are masked while judging; all judgments are fixed and hashed before unmasking. Automatic fact/reference matching and assistant support checks are distinct limited proxies, not independent human accuracy. Retain invalid output, abstentions, unsupported claims, quote failures and length stops without quality-based retries. Full private documents/questions/answers are excluded from Git and crates.io; public hashes/counters cannot independently reconstruct the private corpus.

## Artifact discipline

Release checks use the explicit temporary Cargo target with dev/test debug disabled and incremental compilation off. Small Tree Borrows Miri probes use a second named temporary target. Only those regenerable trees are removed after package, CI, registry consumer, VCS and checksum verification. Archive the release package, small measured workers, exact locks and receipts first. Preserve PDFs, snapshots, knowledge, frozen questions, full answers and measurements; no broad deletion over project data is used.
