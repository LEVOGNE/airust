# AIRust 0.2.24: Inspectable trainable ranking

an optional small linear model with replaceable bounded features, explicit training, grouped holdout evaluation and versioned artifacts.

## Validation

The complete suite passes **194 default and 590 all-feature Rust tests**, including compiled documentation, plus five Python tests. Eight integration cases plus a numerical-gradient unit test cover deterministic learning, held-out groups, unseen-query ordering, loss reduction, malformed artifacts, feature/schema changes, Unicode, budgets and controlled cancellation/deadlines/progress.

Formatting, strict default/all-feature/isolated Clippy, rustdoc, minimum Rust 1.89/1.92, patch compatibility against 0.2.23, fresh audit, local/registry consumers, extracted-package tests, dry-run and exact-commit GitHub CI gate publication. Known dependency notices are recorded in [audit metadata](measurements/0.2.24/audit.json); these checks are not an independent security audit.

## Limits and measurements

The authored lexical-pattern fixture holds out complete query groups, records losses, weights and zero-threshold classification against a neutral baseline, and checks a new-query ranking. It does not measure real semantic or independent corpus quality. Full-batch training uses bounded examples/features/epochs; logits are not calibrated relevance probabilities. Custom feature semantics, label trust, leakage grouping and model authenticity remain application responsibilities.

No default model, automatic telemetry, DenkMaps/MCP integration or unmeasured universal quality claim is introduced. See [API contract](LEARNED_RANKING.md), [API cookbook](API-COOKBOOK.md) and [raw measurements](measurements/0.2.24/).

A regression test found that the default JSON float parser could change a weight by one representable value on reload, changing model identity. The optional learned-ranking feature now enables exact floating-point round-trip parsing, and the artifact/recipe equality test is retained.

## Recorded fixture output

The executed release fixture returned the following measurements. These values describe this run and the explicitly synthetic setup above.

```json
{
  "fixture": "authored lexical patterns, grouped query holdout, no semantic quality claim",
  "holdout": {
    "correct_at_zero": 48,
    "examples": 48,
    "loss": 0.11167818690001896
  },
  "model_fingerprint": "c3cb33c58f68bb8bbfca36c6e68d343deb96d12edaa4ec3dc3d3cf12b936942d",
  "model_json_bytes": 568,
  "neutral_zero_baseline_correct": 24,
  "new_query_first": "b",
  "schema": 1,
  "schema_features": {
    "names": [
      "bias",
      "query_coverage",
      "document_density",
      "exact_phrase",
      "length_balance",
      "ordered_bigram_coverage"
    ],
    "recipe": "airust-lexical-features-v1"
  },
  "train_examples": 192,
  "train_ms": 1.287625,
  "training_report": {
    "config": {
      "epochs": 100,
      "gradient_clip": 5.0,
      "l2": 0.01,
      "learning_rate": 0.2
    },
    "examples": 192,
    "final_loss": 0.11167818690001868,
    "initial_loss": 0.6931471805599464,
    "manifest": "5b37818ad223ab7be7bd10ac30cdb6640953b347f9e0b1e1114660a553bfe855",
    "positive": 96
  },
  "weights": [
    -1.0554663453812294,
    1.2944668561629493,
    0.43148895205431603,
    1.2944668561629493,
    -1.1351331823084698,
    1.2944668561629493
  ]
}
```

Raw child-process timing, CPU and peak RSS are in the adjacent `.resources.json`; timings are descriptive, not an SLA.

## Native inference boundary

These releases exercise provider-neutral algorithms and authored fixtures. The complete suite retains 1 ignored native-model test; native assets/inference are not revalidated by these runs. Synthetic features, vectors and symbolic rules do not establish real-model semantic quality.
