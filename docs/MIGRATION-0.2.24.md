# Migrating to AIRust 0.2.24

Enable `learned-ranking` explicitly. Existing default behavior, storage data and retrieval JSON remain unchanged. an optional small linear model with replaceable bounded features, explicit training, grouped holdout evaluation and versioned artifacts.

The authored lexical-pattern fixture holds out complete query groups, records losses, weights and zero-threshold classification against a neutral baseline, and checks a new-query ranking. It does not measure real semantic or independent corpus quality. Full-batch training uses bounded examples/features/epochs; logits are not calibrated relevance probabilities. Custom feature semantics, label trust, leakage grouping and model authenticity remain application responsibilities.

See [full API contract](LEARNED_RANKING.md). No DenkMaps/MCP integration is performed.
