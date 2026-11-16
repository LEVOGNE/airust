# Changelog

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
