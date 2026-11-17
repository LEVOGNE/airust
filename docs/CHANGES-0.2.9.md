# AIRust 0.2.9: intelligent reranking

AIRust can now evaluate retrieved candidates a second time. The optional provider-independent pipeline preserves original results and applies a bounded query/document scorer. A separate optional local adapter uses an actual multilingual MiniLMv2 cross-encoder through FastEmbed. Existing search methods, entry fields, features and formats retain their behavior. This release performs no DenkMaps or MCP integration.

## API and implementation

- `reranking = []`: `Reranker`, `Rerankable`, `rerank`, options, typed errors and borrowed results/outcome. It adds no dependency and does not enable embeddings or a model.
- `local-reranking`: explicitly loaded, exact-size/SHA-256-verified model/tokenizer assets and a trusted dynamic ONNX Runtime library. No build or initialization downloads. CPU threads are explicit; raw single logits, 512-token longest-first pairs, no E5 prefixes.
- Scope before candidate selection and scoring, one captured entry per candidate, complete selected-input validation before any model call, paired-text byte and batch budgets, visible candidate counts and deterministic input-order ties.
- Default strict errors; explicit provider-only original-order fallback discards partial scores. Invalid outputs/recipes and configured score thresholds cannot be bypassed by unscored fallback.
- Original candidate/component evidence remains borrowed. Reordered IDs can be assembled into scoped context, with the separate rerank evidence retained. Source entries are never modified.

See the [complete contract](RERANKING.md), [native setup](LOCAL-RERANKING.md), [API cookbook](API-COOKBOOK.md) and [migration notes](MIGRATION-0.2.9.md). The generic `reranking` and combined `knowledge_pipeline` executables use explicitly handwritten API scorers; they are not native model quality evidence.

## Bugs found and fixed

The first real E5-plus-reranker run failed with ORT's “only one environment is allowed per process” error. The convenience adapters previously created separate environments. Both now use one synchronized shared environment with a consistent canonical runtime path. Native combined retrieval/reranking succeeds; the application may still supply its own providers when another library owns its runtime setup.

A custom candidate accessor could otherwise return a different entry after its scope check. The pipeline now captures each entry reference once and uses that exact checked text and result reference. Tests cover a deliberately stateful adapter. Wrong counts, missing native score indexes, non-finite values, changed recipes, selected duplicate IDs and native output-shape/type errors remain errors. A later-batch failure cannot present earlier scores mixed with untouched candidates, and an unscored fallback cannot bypass a score threshold. Signed-zero ties preserve input order.

## Real native comparison

The fixture is unchanged from 0.2.8: 30 developer-authored public documents and 36 questions, including German/multilingual paraphrases, literal identifiers, distractors and two no-answer cases. Unlisted document grades are explicit zero only in this closed fixture. Precision/MRR average all 36 questions; recall/nDCG average the 34 with positive judgments. Cutoff is three.

The retrievers are the original BM25, exact multilingual E5 and equal-weight hybrid RRF with default smoothing/component-candidate limits. Each reranker receives the actual first-stage top-5 or top-20 prefix. No relevant document is injected. Candidate Recall@N is the maximum recall available to the reranker. No minimum logit or no-answer threshold was selected.

| Retriever | Candidates | nDCG@3 before | nDCG@3 after | Recall@3 before | Recall@3 after | MRR@3 before | MRR@3 after |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| BM25 | 5 | 0.394 | 0.439 | 0.426 | 0.456 | 0.412 | 0.458 |
| BM25 | 20 | 0.394 | 0.439 | 0.426 | 0.456 | 0.412 | 0.458 |
| Exact E5 | 5 | 0.858 | 0.925 | 0.971 | 1.000 | 0.806 | 0.861 |
| Exact E5 | 20 | 0.858 | 0.914 | 0.971 | 1.000 | 0.806 | 0.847 |
| Equal hybrid | 5 | 0.598 | 0.737 | 0.676 | 0.765 | 0.579 | 0.722 |
| Equal hybrid | 20 | 0.598 | 0.914 | 0.676 | 1.000 | 0.579 | 0.847 |

These are descriptive smoke results, not an independent relevance benchmark or proof of logical understanding. A recall of 1.000 means the closed fixture's positively judged documents were in these top-three outputs. It is not a universal accuracy or answer-quality claim.

Two exact-E5 queries regress at both window sizes: `de-registration` moves the office-that-does-not-register-vehicles distractor above the actual registration note; `de-backup` puts a Rust note above the backup instruction. Their nDCG@3 falls from 1.000 to 0.631. Average gains do not remove these errors. Five E5 candidates give a better aggregate ranking than twenty here; a larger pool is not automatically better. Native model behavior is recorded rather than corrected by fixture-specific rules.

The no-answer lexical queries remain empty. Semantic/hybrid retrieval and reranking still return three unrelated candidates for each no-answer question without a calibrated threshold, even though their logits are negative. Reranking does not prove truth, identify contradiction reliably or decide whether an answer exists. BM25's missing semantic candidates cannot be repaired by reranking; a different first-stage retriever is needed for that case.

## Cost and reproduction

Measurements use release builds with Rust 1.98.0 on macOS ARM64, Apple M5 Max, 18 logical CPUs and 64 GiB RAM. Each local model uses two CPU threads. The exact model recipe is `cross-encoder/mmarco-mMiniLMv2-L12-H384-v1:sha256:56a64759ca42ec108da29086f27f4f76de65514550826b866fc97dff6314317e`, backed by the pinned [asset manifest](models/mmarco-minilm-v2.json). E5 retains its existing `bdeaf1db1a21c209f990af1bd3faa13d439d214ebc80e4e3dc73d76c9fa79456` recipe. Native runtime is the official ONNX Runtime 1.32.0 macOS ARM64 library.

| Retriever | Candidates | Added rerank median (ms) | Added rerank p95 (ms) |
| --- | ---: | ---: | ---: |
| BM25 | 5 | 6.923 | 20.994 |
| BM25 | 20 | 6.767 | 62.705 |
| exact-semantic | 5 | 16.249 | 19.387 |
| exact-semantic | 20 | 68.237 | 73.616 |
| hybrid-equal | 5 | 16.094 | 23.085 |
| hybrid-equal | 20 | 65.633 | 71.138 |

Reranker loading: 2179.7 ms; warmup/native consistency probe: 13.4 ms. Entire combined evaluation: 11.100 seconds, 18.643 user CPU seconds and 0.949 system CPU seconds. Peak RSS: 2720.78 MiB.

Each series has 36 measured queries after an explicit reranker warmup; empty candidate sets still contribute their near-zero pipeline time. These medians and nearest-rank p95 values describe one small shared-development-host run. They are not statistical confidence intervals or production p95 guarantees. The resource wrapper includes both models, loading, index creation, scoring, result buffers and all evaluation phases; it is not minimal steady-state core memory. Core-only reranking requires no loaded model.

The first native probe checks that a clear Mars query scores its relevant passage above a bread passage. Separate-versus-batched scores agree within 0.001. The combined E5/reranker evaluation exercises the shared environment. Regular tests need no models or runtime, and models/native libraries remain outside the package.

Build first, then measure without concurrent AIRust compilation:

```bash
python3 scripts/download_reranker.py MODEL_DIRECTORY
cargo build --release --locked --features local-reranking,local-embeddings,hybrid,quality --example reranking_quality
python3 scripts/measure.py /tmp/reranking.json target/release/examples/reranking_quality MODEL_DIRECTORY ONNX_RUNTIME_LIBRARY E5_DIRECTORY
```

The [raw query-level JSON](measurements/0.2.9/reranking-reference.json), [resource counters](measurements/0.2.9/reranking-reference.resources.json), [stderr](measurements/0.2.9/reranking-reference.stderr.txt) and [method guide](BENCHMARKS.md) preserve rankings, original positions, logits, before/after metrics, candidate recall and separate retrieval/rerank/end-to-end latency. Source/corpus/lockfile fingerprints and release validation are summarized in [validation metadata](measurements/0.2.9/validation.json).

## Validation and release boundaries

The final validation gate covers **189 default tests and 377 all-feature tests**, strict Clippy/rustdoc/formatting, independently selected features, Rust 1.89 provider-independent APIs and Rust 1.92 complete-feature APIs. New coverage includes 125 exhaustive three-score permutations and 500 seeded scope/prefix/batch cases in addition to protocol/fallback/native-asset regressions. Five additional Python helper tests verify checked atomic downloads, no unexpected network access, refusal to overwrite differing/concurrently created files and cleanup after corrupt/short/oversized responses. The combined example runs with and without reranking. An independent Rust 1.89 consumer compiles both old APIs and the new rerank interface.

Cargo-semver-checks 0.51.0 compares all-feature public APIs against the 0.2.8 source: **229 checks pass, 31 skipped, no SemVer update required** when assuming a patch release. This is a static API compatibility check, not proof that model rankings or every application's behavior are unchanged. Existing retrieval defaults remain unchanged because reranking is opt-in.

The fresh RustSec audit finds zero known vulnerabilities against 1509 advisories, database commit `02e2517f457a359fc6d3440ea4560c8ea188fee8`, updated 15 November 2026. The same optional unmaintained warnings remain: [paste](https://rustsec.org/advisories/RUSTSEC-2024-0436.html) through tokenizers/local models and [ttf-parser](https://rustsec.org/advisories/RUSTSEC-2026-0192.html) through PDF support. They are not suppressed. [Audit metadata](measurements/0.2.9/audit.json) is retained. Dependencies are unchanged apart from the root package version. This work is not a completed independent security audit.

Clean packaging/extracted-crate tests, publish dry run, GitHub CI and registry/tag/release-asset verification gate publication. Published versions remain immutable. Artifacts: [GitHub v0.2.9](https://github.com/LEVOGNE/airust/releases/tag/v0.2.9), [crates.io 0.2.9](https://crates.io/crates/airust/0.2.9). DenkMaps and its existing MCP deployment are not changed by this release.
