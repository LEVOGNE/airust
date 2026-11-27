# Optional local cross-encoder

Enable `local-reranking` for actual local query/document scoring. It includes `reranking`, FastEmbed and dynamically loaded ONNX Runtime. Rust 1.92 or newer is required. It does not enable semantic indexing. Enable `local-embeddings` separately when E5 retrieval is also wanted. The default library and provider-neutral reranking require neither.

`FastEmbedReranker::multilingual_minilm_v2(model_directory, runtime_library, threads)` loads the multilingual `cross-encoder/mmarco-mMiniLMv2-L12-H384-v1` model. It checks all five local assets against the packaged exact-size/SHA-256 [manifest](https://github.com/LEVOGNE/airust/blob/v0.2.9/docs/models/mmarco-minilm-v2.json), pinned to upstream revision `1427fd652930e4ba29e8149678df786c240d8825`. Missing, changed or truncated assets fail. This constructor deliberately accepts that verified model recipe; implement `Reranker` for a different model or existing inference service.

## Obtain assets explicitly

Builds, all-feature tests and initialization never download a model or native runtime. From a source checkout or extracted crate, Python 3.11 or newer can download the pinned assets explicitly:

```bash
python3 scripts/download_reranker.py /path/to/mmarco-minilm-v2
```

The helper downloads only those five revision-pinned files over HTTPS, checks sizes and hashes before publishing complete files atomically, and refuses to overwrite differing existing files. Rerunning it verifies existing files. Use an application-controlled model directory. The ONNX model is 470,883,696 bytes; the tokenizer is another 17,082,660 bytes, plus three small JSON files. Neither is bundled in the crate.

Supply the platform-specific ONNX Runtime shared library separately. Native evaluation used the official ONNX Runtime 1.32.0 macOS ARM64 release. Only load a trusted runtime library. A digest is a consistency check, not a signature or a sandbox for native code. The script does not install packages, edit environment variables or download a runtime.

## Use it

```toml
[dependencies]
airust = { version = "=0.6.2", default-features = false, features = ["local-reranking"] }
```

```rust,no_run
use airust::{Engine, KnowledgeEntry};
use airust::reranking::{local::FastEmbedReranker, rerank, RerankOptions};

let engine = Engine::from_entries([
    KnowledgeEntry::new("registration", "Kfz-Zulassung durchführen."),
    KnowledgeEntry::new("passport", "Einen Reisepass beantragen."),
])?;
let mut model = FastEmbedReranker::multilingual_minilm_v2(
    "/models/mmarco-minilm-v2",
    "/runtime/libonnxruntime.dylib",
    2,
)?;
let candidates: Vec<_> = engine.entries().collect();
let outcome = rerank(
    "Ein Fahrzeug bei der Behörde anmelden",
    &candidates,
    &mut model,
    &RerankOptions::new(2),
)?;
for hit in outcome.results() {
    println!("{} {:?}", hit.entry().id, hit.score);
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

This tiny example supplies both records so the model can compare them. In an actual larger application, use bounded retrieved candidates. Reranking BM25 alone cannot recover a semantic paraphrase that BM25 did not retrieve.

The constructor requires a positive CPU thread count and forces CPU execution. It scores query/document pairs with a maximum combined sequence length of 512 tokens, longest-first pair truncation, no E5 query/passage prefixes and one raw classifier logit. Longer model inputs can lose query or document tokens despite passing the pipeline's separate byte budgets. Prefer appropriately sized passages. Scores can be negative or greater than one; they are not relevance probabilities. FastEmbed's sorted native results are mapped back to input document order and checked for missing/duplicate indexes.

The local scorer uses internal batches of at most sixteen; the outer reranking pipeline can impose a smaller batch size. Reuse the loaded model. Model loading and pair inference add CPU and RAM cost, and reranking does not make the first-stage search faster.

`RerankOptions::with_batch_size` and `with_candidates` already expose those pipeline choices. For CPU workloads with mixed paragraph lengths, benchmark smaller outer batches with an unchanged candidate prefix: each native batch is padded to its longest tokenized pair. Changing the batch can change floating-point logits slightly even with the same model/text. Compare all scores, rankings, known-reference losses, complete request cost and peak memory; additional threads are an explicit CPU tradeoff. Do not infer an energy or general quality improvement from lower wall time.

The 0.6.1 reference profile makes these existing controls available as bounded `reranker.batch_size` and `reranker.candidates`, with configuration-only preflight. Existing profiles keep their old defaults. Read [configuration](CONFIGURATION.md), [migration](MIGRATION-0.6.1.md) and [the actual local-model investigation](measurements/profile-practical-0.6.0/README.md). Its small conversation probes and exposed authored regression corpus do not establish independent customer quality.

E5 and the reranker share one process-wide AIRust ONNX environment. Simultaneous local providers now work without the previous second-environment initialization error. Supply the same canonical runtime-library path for every AIRust local provider. A different path returns an error rather than silently loading another runtime. The shared environment remains alive for the process lifetime. If another library already owns an independently initialized ORT environment, supply your own `Reranker`/`EmbeddingProvider` around that setup; these convenience constructors do not adopt an arbitrary external environment.

## Reproduce native evaluation

```bash
cargo build --release --locked --features local-reranking,local-embeddings,hybrid,quality --example reranking_quality
python3 scripts/measure.py docs/measurements/local-rerank.json target/release/examples/reranking_quality /path/to/mmarco-minilm-v2 /path/to/libonnxruntime.dylib /path/to/e5-small
```

Omit the final E5 directory to evaluate BM25 reranking only; `local-embeddings,hybrid` are then unnecessary. The example warms the reranker, verifies an obvious English pair and native batch consistency, then evaluates the unchanged public 30-record/36-query fixture. It reranks actual top-5/top-20 retrieval pools, never injects missing positive answers, records all scores/latencies and preserves no-answer failures. It also verifies that E5 and reranker sessions coexist when the optional E5 path is supplied.

[Published measurements and limits](https://github.com/LEVOGNE/airust/blob/v0.2.9/docs/CHANGES-0.2.9.md) cover this developer-authored smoke set. No general multilingual accuracy, contradiction reasoning, production tail latency or token-saving claim follows from it. Regular tests use deterministic providers and asset validation without requiring downloads or a runtime.

The model is [Apache-2.0 licensed](https://huggingface.co/cross-encoder/mmarco-mMiniLMv2-L12-H384-v1); FastEmbed is [Apache-2.0 licensed](https://github.com/Anush008/fastembed-rs). Weights are external assets. Review their upstream licenses and the [ONNX Runtime distribution](https://github.com/microsoft/onnxruntime/releases) when distributing an application.

## Optional verified file loading in 0.6.1

The existing constructor remains memory-backed. `multilingual_minilm_v2_from_snapshot(model_directory,runtime_library,threads,snapshot_directory)` streams the exact pinned model into an exclusively created private temporary file, verifies it, then asks ONNX Runtime to load that copy. This avoids retaining a large Rust model buffer during native initialization without re-opening a potentially changed original after hashing. Normal/error paths clean up the copy after native state; forced termination can leave it behind. The application owns the directory and provides 470883696 bytes plus filesystem overhead of extra disk space.

The reference profile enables this with optional `reranker.model_snapshot_directory`. Missing/null preserves legacy behavior. The recipe, tokenizer, candidate prefix and raw-logit contract do not change. Measured memory/startup costs and native/Miri exclusions are in [resource ownership](RESOURCE-LIMITS.md) and [the retained study](measurements/profile-practical-0.6.0/README.md). The native profile still exceeds a 1-GiB device budget on this host; the model-free core is a separate option.

## Explicit process ownership in 0.6.2

For repeated MiniLM unload/reload, use the opt-in [process adapter](LOCAL-MODEL-PROCESSES.md) with a matching trusted worker and a verified private file snapshot. The parent retains bounded transport state and reaps its child before removing private copies. The reference pilot requires `reranker.isolated: true` and an explicit `model_snapshot_directory`; omitted/false keeps the old direct behavior. A malformed output, crash or timeout stops that scorer; recovery requires explicit recreation. There is no automatic retry/reload, global allocator setting or hard RAM quota.

[Native measurements](measurements/native-lifecycle-0.6.2/README.md) retain failed direct-load growth, rejected allocation-policy experiments and every startup/request regression. Stable unloaded-parent memory does not establish low active-model memory, uniform latency overhead, independent answer quality or actual 1-GiB-device support. Cancellation remains cooperative between native batches; per-call worker timeouts are a separate bound.
