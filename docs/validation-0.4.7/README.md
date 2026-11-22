# AIRust 0.4.7 validation and reproduction

Read [coverage scoring](../LEXICAL-COVERAGE.md) and the [executed report](../CHANGES-0.4.7.md). The new source is explicit under `retrieval`; literal and prepared-only sources, ordinary Engine ranking, citation checks and MCP defaults remain unchanged.

## Independent Rust application

Copy `consumer.toml` to a private directory as `Cargo.toml` and `consumer.rs` to `src/main.rs`. Run `cargo +1.89.0 run --manifest-path /absolute/private/directory/Cargo.toml`. It exercises coverage inspection/delivery, old prepared query behavior, paragraph selection, bounded quotation checks and retained document-coordinate projection. The prepublication run uses a local path. A fresh postpublication consumer uses exact registry 0.4.7, without a path override; package version, registry checksum and VCS commit are checked.

## Frozen private inputs

The comparison retains the unchanged 72-file, 2,755,468-byte snapshot with 1,910 chunks from AIRust, Flasm and Vecasm. This is Markdown retrieval, not a new large-PDF import benchmark. Original PDFs and the canonical 167,942-entry knowledge store remain.

All 282 earlier questions are exposed development data. Thirty rank-fusion parameter combinations on their actual components, earlier approximate variants, reserved-head experiments and 44 coverage/original-score blends are retained, including failures. Rank fusion did not improve top-six reference coverage. The selected recipe uses one prepared lexical candidate search and a fixed coverage weight of 0.5. These development results guide design and are not independent test evidence.

Before inspecting new rankings, 60 new questions were frozen: 48 answerable and 12 intentionally unsupported, 20 per project. There are 16 English and 44 German questions. Positive labels cover ten known files: four AIRust and four Flasm files with four questions each; two Vecasm files with eight questions each. All questions/labels come from the same assistant, not independent human judges or production user logs. These are new questions about known source files, not unseen domains. File clustering and possible conceptual overlap with earlier questions limit independence. No parameter is changed from these new results.

Input SHA-256 manifests, full questions, sources, attempted designs, selected audit questions, exact locks, complete contexts and all local replies remain private. Public sanitized counters cannot independently reproduce a private corpus. The executable harness also works with a caller-supplied corpus in the same format.

## Three paired configurations

- A: exact published registry 0.4.6 literal BM25 and ordinary prefix excerpts.
- P: exact published registry 0.4.6 with explicit default query preparation, leading context and exact distinct excerpts.
- C: candidate 0.4.7 with explicit default query preparation, half normalized BM25/half distinct-term coverage, leading context and exact distinct excerpts.

All use the same project scope, six returned passages, 100 source candidates, 2,000 bytes per passage and 12,000 total source-text bytes. The original case-preserved question is passed unchanged to each source and the model. Coverage changes source candidate order; it does not add candidates, enlarge context budgets or enable a model in the library. Citation validation checks the final delivered immutable passages against scoped canonical entries and original byte positions. Recipe-specific score values are not interchangeable.

The two old configurations are compared byte-for-byte against candidate implementations on all 342 questions: 684 complete response comparisons. Retrieval timing uses one untimed warm-up and five repeated responses for every question/arm, 5,130 samples. Full outputs must remain deterministic. The private component trace exposes the actual formula; query-level sanitized rank/coverage counts are published. Own compilation finishes before timed retrieval and generation.

Copy each `worker-*.toml` to its own private folder as `Cargo.toml`, copy `benchmark-worker.rs` as `worker.rs`, and build in one dedicated temporary target. For the candidate before publication, add the local crate path to its dependency. After publication the supplied candidate manifest pins registry 0.4.7.

```bash
CARGO_TARGET_DIR=/absolute/temporary/airust-build cargo build --release \
  --manifest-path /absolute/private/baseline/Cargo.toml --features candidate
CARGO_TARGET_DIR=/absolute/temporary/airust-build cargo build --release \
  --manifest-path /absolute/private/candidate/Cargo.toml --features candidate,coverage
```

Set `AIRUST_BENCH_DIR` to the private input folder and `AIRUST_BENCH_TARGET` to that build target. With NumPy installed, run `python -B docs/validation-0.4.7/benchmark.py retrieve`.

## Fresh answer and citation checks

Only the 60 new questions receive fresh replies from all three configurations, for 180 replies. Earlier 846 replies are retained as historical evidence and are not counted again. This focused release does not rerun SQLite/embeddings; its earlier comparison remains separately reported.

Use the installed Qwen3.6 35B Q4_K_M model on a private loopback Ollama daemon at port 11464, cloud disabled, no pruning, parallelism one and context 16,384. The original user daemon remains untouched. The client rejects proxies and redirects. No remote inference or model downloads are used. Run `python -B docs/validation-0.4.7/benchmark.py answer`, then `python -B docs/validation-0.4.7/analyze.py`.

Generation uses the same prompt/schema across configurations, temperature zero, seed 471107, thinking disabled and 512 output tokens. All six arm orders occur ten times; every arm occupies each position twenty times and precedes each other arm thirty times. Prompt caching remains enabled and real cache counters are retained. No failed quotation, incorrect reply, abstention or length stop is retried for quality. Complete pipeline time includes retrieval, context building, local generation and quotation checking; it is not isolated BM25 time.

Reference quote presence/MRR are authored-reference metrics, not complete relevance judgments. Word-pattern answer checks and literal quotation validity are separate descriptive proxies, not proof of truth or entailment. A correct alternative source can fail the authored-ID metric. Paired question bootstrap intervals, where reported, describe this sample and do not correct for source-file clustering or multiple development trials. Small assistant semantic reviews are not independent human evaluations.

Worker RSS and isolated Ollama process-family samples are whole-process observations, not index-only heap or full GPU/unified-memory accounting. No quality-equivalent speed, token or RAM saving follows from different answers or contexts. Exact model, host, toolchain, hashes and validation receipts are retained.

## Artifact discipline

Only `/private/tmp/airust-relevance-047-build` and `/private/tmp/airust-relevance-047-miri` contain new regenerable build trees. Preserve release crate, logs, measured workers and exact locks before removing those trees after successful registry/CI verification. Never broadly clean the private knowledge or measurement folders.
