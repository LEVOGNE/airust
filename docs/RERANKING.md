# Intelligent reranking

Reranking is a second ranking step. First retrieve a manageable list of possible answers. Then score each query/document pair with a reranker and return the best candidates. The optional `reranking` feature provides this pipeline without requiring a model or network service. The optional [local cross-encoder](https://github.com/LEVOGNE/airust/blob/v0.2.9/docs/LOCAL-RERANKING.md) supplies actual multilingual model inference.

A reranker can change the order of candidates. It cannot recover an answer that the first search did not retrieve. It can also make a good ranking worse. Evaluate the complete pipeline on fixed labeled queries before enabling it in an application.

## Add the feature

```toml
[dependencies]
airust = { version = "=0.2.9", default-features = false, features = ["reranking"] }
```

The provider-independent API supports Rust 1.89. It adds no dependency and does not enable embeddings, HTTP, databases or an LLM. The default features stay empty. Existing search methods do not automatically rerank.

## Score candidates

Implement `Reranker::recipe()` and `Reranker::score(query, documents)`. Return exactly one finite `f64` score per document in the supplied document order. Higher means more relevant. Negative values are valid. Do not return already sorted scores without restoring document positions. Include weights/revision, tokenizer, truncation and postprocessing in a stable recipe ID of 1–1024 bytes.

The following scorer only illustrates the API. It is handwritten and is not semantic inference. Run `cargo run --features reranking --example reranking` for the complete executable demonstration. Use `FastEmbedReranker` or another evaluated provider for real query/document scoring.

```rust
use airust::{Engine, KnowledgeEntry};
use airust::reranking::{rerank, RerankError, RerankOptions, Reranker};

struct Demo;
impl Reranker for Demo {
    fn recipe(&self) -> &str { "handwritten-demo-v1" }
    fn score(&mut self, _query: &str, documents: &[&str])
        -> Result<Vec<f64>, RerankError>
    {
        Ok(documents.iter().map(|text|
            if text.contains("ownership") { 1.0 } else { 0.0 }
        ).collect())
    }
}

let engine = Engine::from_entries([
    KnowledgeEntry::new("safe", "Rust ownership helps prevent memory errors."),
    KnowledgeEntry::new("manual", "Rust raw pointers require manual memory invariants."),
])?;
let candidates = engine.search("Rust memory", 20);
let outcome = rerank("Rust memory", &candidates, &mut Demo, &RerankOptions::new(2))?;
assert_eq!(outcome.results()[0].entry().id, "safe");
assert!(outcome.reranker_used());
# Ok::<(), Box<dyn std::error::Error>>(())
```

Pass a slice of `KnowledgeEntry`, references to entries, `SearchResult`, or, with their separate features, `SemanticResult` and `HybridResult`. Custom candidate wrappers implement `Rerankable::entry()`. The operation captures each candidate's entry reference once; scope checks, scored text and returned entry therefore refer to that same record even if a custom wrapper's accessor is stateful.

Results borrow the original candidate and entry. `candidate` retains all original fields, including BM25, cosine or hybrid component scores and matched terms. `original_rank` is the one-based position in the supplied slice, before scope filtering. `score` is the new raw score, or `None` for an explicit unscored fallback. Do not add scores from unrelated scales. Equal reranker scores retain input order, including positive/negative zero. The source index is unchanged.

The candidate slice and its index/read guard must outlive the outcome. Serialize or copy a small selected result while those borrows are valid. Metadata and provenance stay with the existing candidate; they are not sent to the scorer.

## Bound work and scope

`RerankOptions::new(limit)` chooses the final result limit. The default is ten results, one hundred selected candidates and batches of sixteen. `new(limit)` raises the candidate cap when necessary to cover a larger requested result limit. Choose a small explicit cap with `with_candidates`; it must be positive and at least the result limit. A zero result limit skips validation/scoring of candidate text and provider access, while still counting input/scope-eligible candidates. The query must still be nonempty and within its byte limit.

`with_context` rechecks IDs/metadata before selecting the candidate prefix and before any provider call. The existing Context convention applies: an empty ID set is unrestricted, not deny-all. An application with no authorized IDs must return an empty result itself. An incompatible scope intersection stays denied. Reranking is not an authentication layer; retrieve an authorized candidate set first and apply the same boundary to later context/graph expansion.

All supplied candidates are visited to count eligibility. Only the first capped eligible candidates are selected, validated and scored. Selected IDs/text must be nonempty and selected IDs unique. Omitted or excluded records are not scored or validated as selected records. `received_candidates`, `eligible_candidates`, `selected_candidates` and `scored_candidates` make these limits visible. The omitted eligible tail is not returned as if it had been scored. Input scanning still costs time proportional to the supplied slice; do not materialize an entire large corpus as candidates.

Default byte budgets are 16 KiB per query, 64 KiB per selected document and 2 MiB across selected query/document pairs. The total includes the query again for every selected document. `with_byte_budgets(query, document, pairs)` replaces these positive limits. Bytes mean UTF-8 bytes, not characters or model tokens. Original UTF-8 and whitespace are passed through unchanged. All selected records and budgets are checked before the first inference batch. Oversized inputs fail instead of being silently shortened by the pipeline.

`with_batch_size` accepts a positive batch size. Providers receive only the original query and document texts. They must preserve the stable recipe and return a complete score vector. The pipeline rejects wrong counts, NaN, infinities and recipe changes, including a recipe change accompanying an error.

## Handle errors and no-answer cases

The default policy returns an error on provider failure. `with_failure_policy(RerankFailurePolicy::KeepOriginalOrder)` explicitly permits fallback only for `RerankError::Provider`. The complete selected prefix returns in its original order, limited by the requested output count. `reranker_used` is false, every output score is `None` and `fallback_reason` states why. Scores from earlier successful batches are discarded; `scored_candidates` still records how many candidates completed valid scoring before failure. No partly scored mixture is presented as a final ranking.

Invalid inputs, changed recipes, malformed output and non-finite scores always fail. `with_min_score` accepts any finite raw-score threshold. If a threshold is configured, provider failure remains an error even with fallback enabled: unscored results cannot bypass the threshold.

There is no default relevance threshold or automatic no-answer detector. Cross-encoder logits are not calibrated probabilities. A chosen threshold depends on the model, corpus and task and needs separate calibration/evaluation. Without one, unrelated candidates can still be returned.

Provider calls are synchronous. The application owns trusted model/service selection, deadlines, cancellation, authentication, retries and external-data policy. Bound blocking model work in a worker pool, reuse model instances and avoid holding database/index locks during remote or expensive inference. Byte/count limits do not promise an exact RAM, token or latency limit inside a provider. Provider error strings are caller/provider supplied; redact them before exposing them through an external service when appropriate.

## Compose with context

Retrieve lexical, semantic, hybrid or explicitly selected ANN candidates, rerank that slice, then pass the selected IDs to `ContextAssembler::assemble` when `context-assembly` is enabled. Preserve the separate rerank outcome as ranking evidence; assembly's Provided origin does not invent or embed reranker scores. Context still enforces its own source, excerpt and authorization budgets. Source references and full entry metadata are not reranker evidence of truth.

The [0.2.9 report](https://github.com/LEVOGNE/airust/blob/v0.2.9/docs/CHANGES-0.2.9.md) compares real local-model reranking against unchanged retrieval baselines on the frozen 0.2.8 public reference. It preserves query-level regressions, candidate recall ceilings and additional cost. This authored smoke set is not an independent production benchmark.

Primary design references: [Sentence Transformers retrieve-and-rerank](https://www.sbert.net/examples/sentence_transformer/applications/retrieve_rerank/README.html), [FastEmbed implementation](https://github.com/Anush008/fastembed-rs) and the [selected model card](https://huggingface.co/cross-encoder/mmarco-mMiniLMv2-L12-H384-v1). AIRust reuses FastEmbed rather than implementing another transformer runtime; no upstream model weights are bundled.
