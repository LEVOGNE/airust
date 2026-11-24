# Index maintenance boundaries

This development change isolates two private mechanisms from `src/index.rs`. It does not change public types, scoring arithmetic, serialized records, cache formats, storage defaults or result ownership.

| Module | Owns | Contract |
| --- | --- | --- |
| `src/index/cursors.rs` | Ordered document and posting-frequency lookups | Forward-only increasing IDs; exact pointer/string comparisons and range boundaries; checkpoint before the existing gap seek. |
| `src/index/query_weights.rs` | Small direct-mapped query-local IDF memo | Nonzero live interned-term keys, power-of-two capacities 8/256, recompute on collision, transfer occupied slots in their original order. |
| `src/index.rs` | Index generation, mutation, posting union, scoring and norm-cache coordination | Validate before publishing edits; invalidate derived norms on generation changes; release read guards before writers need progress. |

The norm cache stays beside index generations and mutation because its identity and lifetime are coupled to them. Moving it behind broader private field visibility would weaken that ownership boundary. Posting union and score accumulation stay together, preserving traversal and floating-point addition order. The change extracts small coherent mechanisms without introducing a new public abstraction or changing heuristics.

Concise source comments now document invariants and lock-lifetime choices. Detailed contracts remain in the guides and executable examples. The preserved PDF backend's original source, notices and patch are unchanged.

## Behavior control

The same independent consumer is compiled against the exact published registry crate 0.5.2 and the current checkout, each with default and `shared-index` storage. All use `execution,retrieval`, one compiler, equivalent direct dependency resolution, thin LTO and one codegen unit. Each compilation has its own retained manifest/lock and binary identity.

The signature runs all 150 frozen questions plus eight controls (single and repeated terms, a 320-term query, Unicode normalization, a miss, empty input and a broad query). Both BM25 and TF-IDF run under unrestricted, metadata and explicit-ID scopes. It compares ordered IDs, every f64 score bit, matching terms, portable original records and metadata across original, edited, retained-clone and JSON-restored stages. Invalid replacement must retain the original records. The synthetic records exercise long term dictionaries and query memo promotion alongside the frozen corpus.

The controls are finite regression evidence, not a proof for every possible input. Existing independent-oracle, cancellation, concurrency, insertion, replacement, removal and Unicode tests remain in the ordinary suite.

## Paired cost control

The measured corpus has eight copies of all 1,143 paragraph rows plus 160 long synthetic records (9,304 entries). Every worker warms all 158 queries for both algorithms, then measures seven repetitions. Construction and output-file serialization are outside the query timer. Two campaigns alternate baseline/candidate order across seven process pairs for each storage mode. No model inference or compilation runs alongside the paired workers.

For each campaign and storage mode, p95 pools individual warm query samples over seven workers. RSS is exact-child process peak, using native macOS bytes or Linux KiB converted to bytes. Compare the median peak across seven workers; retain the maximum too. The frozen 10% aggregate budget fails when the same storage/metric exceeds it in both campaigns. Every slower per-query mean remains recorded even when the pooled gate passes. Tiny-query ratios can reflect timer/host noise, so the report does not interpret them as universal algorithmic costs.

[Measured signatures, all slower cases and paired results](measurements/index-refactor/README.md).

## Reproduce in isolation

The baseline package must be available from the registry or local Cargo cache. The explicit builder creates four external consumers without changing either source checkout. It can access the registry unless `--offline` is supplied. Use a new output directory for every command:

```bash
python3 -B scripts/reference_dataset.py --output /tmp/airust-index-dataset
python3 -B scripts/build_index_control.py /tmp/airust-index-controls --offline
python3 -B scripts/verify_index_refactor.py --binary-directory /tmp/airust-index-controls --dataset /tmp/airust-index-dataset --output /tmp/airust-index-pairs
```

The consumer source and manifest template live in `docs/validation-reference/`. Preserve the resulting locks, build receipt, binaries and campaign output before removing the builder's `target` subdirectory. Normal tests do not fetch or rebuild the older registry crate; this is an explicit maintenance control.
