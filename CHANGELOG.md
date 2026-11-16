# Changelog

## 0.2.4

- Add independent optional source provenance with validated document/page/text/file/code locations.
- Intern reference definitions, bind stable entry IDs, preserve merged sources and reject conflicts before mutation.
- Add checked versioned JSON, pruning, missing/omitted states and borrowed context-source references.
- Preserve existing entry schemas, algorithms and default features; document coordinate and freshness limits.


## 0.2.3

- Add optional borrowed context assembly over existing lexical, semantic, hybrid and graph APIs.
- Preserve ranked source IDs, metadata, graph evidence and UTF-8 excerpt offsets within explicit node/text budgets.
- Add fail-closed scope intersection and predicate-filtered BFS without hidden bridges.
- Retain distinct duplicate source references; cover streaming, multilingual budgets, fallback and validation regressions.
- Record 1k/10k/100k assembly measurements, API contracts and migration notes without adding an application integration.

## 0.2.2 — Optional generic graph — November 16, 2026

- Reuse petgraph's stable storage, union-find, and iterative topological sorting behind an independent `graph` feature.
- Add shared stable IDs, typed directed/undirected relations, metadata, neighbors, and coordinated node/edge removal.
- Add deterministic iterative BFS/DFS with depth/node budgets, relation filters, and explicit allowed-ID sets.
- Detect mixed cycles and support checking selected relation types.
- Fix a long-chain stack overflow found in the initial upstream recursive cycle-check path by selecting iterative topological sorting.
- Add exhaustive small-graph reference checks, mutation tests, long-chain/high-degree regressions, and measurements through 100,000 nodes.

See the [0.2.2 report](docs/CHANGES-0.2.2.md) and [graph API](docs/GRAPH.md).

## 0.2.1 — Optional hybrid retrieval — November 16, 2026

- Reuse BM25, TF-IDF, exact semantic ranking, metadata filters, source entries, and deterministic top-result selection.
- Add configurable weighted reciprocal rank fusion with per-component candidate budgets and score/rank evidence.
- Keep IDs unique across component results and preserve original entry references and metadata.
- Offer an explicit provider-error fallback with a visible reason; model and vector integrity failures remain errors.
- Add rank-reference, failure, scope, extreme-configuration, and large-candidate tests and a real-model comparison.
- Preserve 0.2.0 APIs, feature defaults, and minimum toolchain boundaries.

See the [0.2.1 report](docs/CHANGES-0.2.1.md) and [hybrid API](docs/HYBRID.md).

## 0.2.0 — Optional embeddings — November 16, 2026

- Add the optional `embeddings` feature with provider-neutral batch inference, model-space validation, normalized vectors, and robust cosine similarity.
- Reuse the existing lexical index, entry storage, metadata filters, and bounded deterministic ranking for exact semantic retrieval.
- Validate and stage provider-backed batches before applying any source or vector updates.
- Add an optional local FastEmbed/E5 adapter that reads explicit files and fingerprints the complete model recipe; disable upstream downloads.
- Preserve Rust 1.89 for the core and general provider feature. The optional current ONNX adapter needs Rust 1.92.
- Add public multilingual reference cases, deterministic failure tests, examples, measured evaluation, and migration documentation.

See the [0.2.0 change report](docs/CHANGES-0.2.0.md). Hybrid fusion and the remaining roadmap stages are not part of this release.

## Repository documentation update — November 16, 2026

- Add the [follow-up 0.1.9 validation report](docs/VALIDATION-0.1.9.md), aggregate measurements, a separate registry test consumer, and public fixtures.
- Summarize the 187 published-package tests, real-corpus index checks, SQLite replay checks, and common-query scale measurements in the README.
- State the limits of lexical matching, application synchronization, scope handling, relevance evaluation, and the read-only DenkMaps investigation.
- Keep AIRust at 0.1.9. This documentation update introduces no library changes, new crate release, or DenkMaps/MCP integration.

## 0.1.9 — English and hardening — November 16, 2026

- Move project documentation and user-facing project text to English.
- Give documentation files English names and keep API contracts outside source comments.
- Remove source comments as requested.
- Validate training weights and replace saved collections atomically through private temporary files.
- Bound retained top results and prune edit-distance work while preserving deterministic ordering.
- Use private CLI runtime state, verified process identity, strict parsing, and explicit PDF overwrite.
- Add API authentication, Host/Origin checks, administrative opt-in, path confinement, and crawler limits.
- Add a configurable index benchmark and a clearly separated future roadmap.

See the [0.1.9 change report](docs/CHANGES-0.1.9.md) for implementation details and validation results.

Version artifacts: [GitHub release](https://github.com/LEVOGNE/airust/releases/tag/v0.1.9), [crates.io](https://crates.io/crates/airust/0.1.9).

## 0.1.8 — Knowledge library core — November 16, 2026

- Add generic knowledge entries with IDs, text, and metadata.
- Share an incrementally maintained index between BM25 and TF-IDF retrieval.
- Add text comparison, duplicate groups, and explained lexical relation suggestions.
- Add explicit search scope by entry ID and metadata.
- Disable PDF, HTTP, SQLite, and bots in the default build.
- Preserve existing agents and training formats as adapters.
- Correct Unicode limits, confidence forwarding, metadata handling, and PDF chunk boundaries.
- Make training-file merging repeatable without reading its output back as an input.
- Update dependencies within their compatible version ranges.
- Revise documentation, startup commands, dashboard text, container configuration, and tests.

See the [0.1.8 change report](docs/CHANGES-0.1.8.md) and [migration guide](docs/MIGRATION-0.1.8.md).

Published artifacts: [GitHub release](https://github.com/LEVOGNE/airust/releases/tag/v0.1.8), [crates.io](https://crates.io/crates/airust/0.1.8).

## 0.1.7 — Preserved original baseline

The supplied source contained the previous agents, PDF import, CLI, dashboard, SQLite storage, and bots. Before the core extraction, 158 unit tests and ten integration tests passed. The [original analysis](docs/ANALYSIS-0.1.7.md) describes that baseline.
