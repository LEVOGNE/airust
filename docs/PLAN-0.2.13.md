# AIRust 0.2.13 acceptance plan

Add explicit query-related original-text windows over the existing retrieval API. Preserve prefix defaults, source ranking, default features and existing public struct literal construction. Prefer complete phrase evidence, then distinct terms/occurrences with deterministic ties. Keep extended graphemes intact and expose checked original/excerpt-relative byte positions.

Bound source/query analysis, normalized text, tokens, terms, matching occurrences and returned highlights. Make absent/too-wide localization and exhausted text budgets explicit. Keep details aligned when reranking or JSON tail removal changes result order/count. Preserve source scope and analyzer configuration through dynamic/borrowed adapters.

Compare the optimized sweep to exhaustive windows, generate mixed Unicode round trips, test German stems/phrases, source permissions, reranking inputs/order, semantic no-overlap, malformed evidence and independent byte/count budgets. Run full/default/isolated tests, Clippy/rustdoc, minimum Rust, static patch compatibility, advisory audit, independent consumer, reproducible long-prefix measurements, extracted package and exact-commit CI before publishing/verifying registry/tag/asset digests. No application integration.
