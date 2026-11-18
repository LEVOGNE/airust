# Explicit trainable ranking

Enable `learned-ranking`. No neural framework, LLM, model download, server or training service is required. Training and inference happen only when called.

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use airust::{learned_ranking::*, reranking::Reranker};
let examples = vec![
    TrainingExample::new("Rust ownership", "Rust ownership prevents memory errors", true)?,
    TrainingExample::new("Rust ownership", "Unrelated archive headings", false)?,
];
let mut features = LexicalFeatures;
let model = train(&examples, &mut features, &TrainingOptions::default())?;
let restored = LinearModel::from_json(&model.to_json()?)?;
let mut scorer = LinearReranker::new(&restored, &mut features)?;
let scores = scorer.score("Rust ownership", &["Rust ownership prevents memory errors", "Unrelated archive headings"])?;
assert!(scores[0] > scores[1]);
# Ok(()) }
```

## Features and model

FeatureExtractor exposes a validated FeatureSchema and extract(query, document). The built-in LexicalFeatures provides six inspectable values in 0..1: bias, distinct query-term coverage, distinct document-term density, normalized substring phrase presence, query/document token-count balance, and ordered query-bigram coverage. It uses core normalized tokenization, without stemming, embeddings or semantic understanding. Repeated query bigrams count as repeated observations; substring phrase presence is not the advanced-search whole-token phrase contract.

Replace the extractor explicitly for another task. Schemas have a recipe and 1..64 unique, nonblank/control-free feature names. Custom values must be finite and in -1..1. Schema identity is checked before and after every extraction, during training and scoring. Custom provider work cannot be preempted or validated for truth by this layer.

The model is a small linear scorer trained using binary logistic loss. Inference returns raw logits, not calibrated answer probabilities, and leaves original retrieval evidence unchanged through the existing Reranker interface. It ranks only supplied candidates and cannot recover a missed document. LinearReranker::new binds a model to an exactly matching extractor schema. Its recipe fingerprints the full validated model. Inspect weights(), schema(), report() and fingerprint(). The feature meaning/recipe is a caller assertion for custom implementations.

## Bounded explicit training

TrainingExample::new(query, document, relevant) validates text and sets its group to a normalized-query fingerprint; with_group sets an explicit group so paraphrases/near duplicates can be kept together. Training requires both labels. Defaults: 100 full-batch gradient steps, learning rate 0.2, L2=0.01, per-coordinate gradient clipping=5. TrainingConfig::new validates 1..200 epochs, finite learning rate in (0,1], L2 in 0..10 and clipping in (0,100]. TrainingOptions::new selects a config.

Training accepts 1..100,000 examples and at most 64 MiB of query/document/group bytes. Each query/document is at most 16 KiB/64 KiB; built-in normalized text also has expansion caps. Feature matrices are cached once, so feature extraction does not repeat every epoch. Matrix memory is bounded by example and feature counts, not an exact process RAM quota. Every example uses the same order and deterministic zero initialization/full-batch updates; repeated runs with the same data/extractor/settings on the same toolchain produce the same artifact. Floating-point portability across platforms is not a bit-identical guarantee.

Stable sigmoid/logistic-loss arithmetic avoids overflowing exp for large logits. Configured updates are bounded and checked for finite weights. Report contains example/positive counts, a length-framed SHA-256 training manifest, config and initial/final unregularized logistic losses. Loss improvement is measured, not universally guaranteed for arbitrary custom features/configurations. A training result is returned only after completion.

With `execution`, TrainingOptions::with_control supports cooperative cancellation, deadlines and actual Features/Training progress counts. Checks surround feature calls and occur during gradient batches/epochs. A blocking custom extractor needs its own timeout. There is no partial model on interruption. Evaluation, splitting and JSON import/export are synchronous bounded operations without a claimed hard time limit.

## Evaluation and artifacts

split_grouped(examples, holdout_percent, seed) deterministically hashes group IDs into disjoint training/holdout index lists. Empty partitions are rejected. Split before training, fit only training examples and evaluate the held-out records with evaluate. Group paraphrases explicitly; hashes cannot identify all leakage. Evaluation returns mean logistic loss and classification count at logit zero, not a ranking-quality guarantee. Report ranking metrics on realistic candidate lists separately. Feedback labels should be trusted/curated and balanced before constructing examples; feedback is not automatically converted into training data.

LinearModel::to_json/from_json uses strict versioned JSON, validates schema/dimensions/config/finite bounded weights/report, and rejects more than 1 MiB input. This feature enables serde_json/float_roundtrip so finite IEEE-754 values and model identity survive JSON parsing exactly. Artifact contains weights and hashes, no raw training query/document strings. Group names and model content hashes are not authentication/anonymization; custom schema labels can themselves contain sensitive data. Caller must authenticate distributed models and preserve training provenance externally. Schema/version validation detects malformed data, not dishonest metrics or poisoned training.

Run learned_ranking for authored lexical patterns split by query group, actual losses/weights/resources and a new-query ranking check. This is a synthetic generalization fixture; it demonstrates learning lexical patterns, not semantic understanding, independent corpus quality or a universal replacement for BM25. No existing knowledge entries, models or datasets are changed automatically. No DenkMaps/MCP integration is performed.
