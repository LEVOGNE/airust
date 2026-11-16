# AIRust 0.2.1 implementation plan

Baseline: published and verified 0.2.0, commit `c78ff74`. This release adds hybrid retrieval only. Graphs, context assembly, provenance, persistence, ANN indexes, and revisions remain later stages.

Reuse `SemanticIndex` and its owned lexical `Engine`, both existing lexical rankers, semantic cosine ranking, metadata filters, borrowed source entries, and bounded `TopK`. Add a dependency-free `hybrid` feature that enables `embeddings`.

`HybridSearchOptions` configures result limit, per-component candidate limit, BM25 or TF-IDF, BM25 parameters, nonnegative reciprocal-rank smoothing, relative lexical/semantic weights, cosine threshold, context, and an explicit failure policy. Normalize weights safely so even extreme finite inputs cannot overflow fused scores.

`SemanticIndex::hybrid_search()` merges by stable entry ID and returns an outcome containing borrowed entries, fused score, component ranks and scores, matched terms, and any explicitly requested provider-failure fallback. Model mismatches and invalid vector output remain errors. A fallback must be visible, not silently represented as semantic success. Filters apply before component ranking and fusion.

Test rank calculations against an independent full reference, ID deduplication, ties, weighting, independent component selection, zero limits, empty queries, metadata scope, candidate budgets, provider failures, model mismatch, invalid output, TF-IDF preservation, and large result sets. Compare lexical, semantic, and hybrid rankings on the fixed public multilingual corpus using the already pinned real model. Record quality and latency without assuming hybrid always improves quality.

Run the complete default/all-feature, feature-isolation, formatting, Clippy, rustdoc, minimum-Rust, package, audit, extracted-package, independent-consumer, dry-run, and publication-verification sequence. Preserve every 0.2.0 public API and introduce no new variants into exhaustive existing enums.

Primary ranking reference: [Cormack, Clarke, and Buettcher, SIGIR 2009](https://plg.uwaterloo.ca/~gvcormac/cormacksigir09-rrf.pdf). RRF combines ranks rather than assuming BM25 and cosine scores share a scale.
