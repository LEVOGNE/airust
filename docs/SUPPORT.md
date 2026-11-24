# Feature and platform support

AIRust keeps a small default lexical library and explicit optional capabilities. This document records which interfaces receive automated checks, which require external assets, and which names remain compatibility flags. The machine-readable contract is [support-matrix.json](support-matrix.json); `scripts/check_support.py` rejects missing or stale feature rows.

## Platforms and validation

| Platform | Automated scope |
| --- | --- |
| Linux on GitHub-hosted `ubuntu-latest` | Default/all-feature tests, strict Clippy/docs, isolated features, Rust 1.89 provider-neutral checks, Rust 1.92 all-target checks and packaging. |
| macOS on GitHub-hosted `macos-latest` | Core, MCP, persistence, reference CLI/stdio, complete feature checks, strict Clippy/docs and packaging. |
| Local macOS ARM64 reference host | Explicit model-backed reference evaluation with recorded hardware, runtime, assets, raw results and process RSS. |
| Other targets, including Windows | No current CI validation claim. Consumers validate their own target and native integrations. |

Hosted runner labels identify the configured test environment, not a minimum operating-system deployment version. The actual compiler/host and job result belong to that run. A configured job is reported as passed only after its exact-commit run completes.

Provider-neutral features use Rust 1.89. The optional ONNX convenience adapters use Rust 1.92 and caller-supplied native libraries and model files. Ordinary CI checks their compilation and failure contracts without downloading model assets. The current model-backed reference campaign exercises external BGE-M3 vectors and the local MiniLMv2 reranker; it does not repeat historical E5 quality measurements.

## Complete feature matrix

Compatibility paths preserve existing callers. They are not silently removed. The `colors` and `plotting` flags are empty compatibility markers in this source; `async` enables Tokio without selecting a separate asynchronous Engine API. New integrations start with Engine, Retriever and explicit document snapshots.

| Feature | Tier | Minimum Rust | Purpose and contract |
| --- | --- | --- | --- |
| `adaptive` | optional | 1.89 | [Explicit routing among registered retrieval sources.](ADAPTIVE.md) |
| `advanced-search` | optional | 1.89 | [Positional phrases and optional German stemming.](TEXT-SEARCH.md) |
| `async` | compatibility-marker | 1.89 | [Enables the Tokio dependency; no separate async Engine API is selected.](../STARTUP.md) |
| `bots` | compatibility | 1.89 | [Alias enabling the optional web integration and its crawler.](../STARTUP.md) |
| `cache` | optional | 1.89 | [Rebuildable derived lexical and optional vector caches.](CACHE.md) |
| `cli` | compatibility | 1.89 | [Historical agent and training-file command-line tools.](../STARTUP.md) |
| `colors` | compatibility-marker | 1.89 | [Retained empty feature flag; no standalone behavior is switched by this flag.](../STARTUP.md) |
| `context-assembly` | optional | 1.89 | [Bounded ranked context and explicit graph neighbors.](CONTEXT.md) |
| `default` | core | 1.89 | [Lexical search, matching and relation suggestions.](../README.md) |
| `disk-index` | optional | 1.89 | [Own-format SQLite original records and full-text index.](DISK-INDEX.md) |
| `documents` | optional | 1.89 | [Immutable text revisions and checked UTF-8 source ranges.](DOCUMENTS.md) |
| `embedded-knowledge` | compatibility | 1.89 | [Explicitly embeds the sample training knowledge through build.rs.](../STARTUP.md) |
| `embeddings` | optional | 1.89 | [Provider-neutral validated vectors and exact cosine retrieval.](SEMANTIC.md) |
| `evidence-freshness` | optional | 1.89 | [Explicit evidence revision and freshness assessments.](EVIDENCE-FRESHNESS.md) |
| `execution` | optional | 1.89 | [Cooperative cancellation, monotonic deadlines and progress.](EXECUTION.md) |
| `feedback` | optional | 1.89 | [Explicit local feedback ledger and bounded preference adjustment.](FEEDBACK.md) |
| `graph` | optional | 1.89 | [Application-owned relationships and bounded traversal.](GRAPH.md) |
| `hnsw` | optional | 1.89 | [Explicit approximate vector retrieval with exact fallback.](ANN.md) |
| `hybrid` | optional | 1.89 | [Reciprocal-rank fusion over lexical and exact semantic candidates.](HYBRID.md) |
| `learned-ranking` | optional | 1.89 | [Explicit small linear ranker trained on supplied labels.](LEARNED_RANKING.md) |
| `local-embeddings` | native-model | 1.92 | [Explicit verified local E5 ONNX adapter.](LOCAL-EMBEDDINGS.md) |
| `local-reranking` | native-model | 1.92 | [Explicit verified local multilingual MiniLMv2 ONNX adapter.](LOCAL-RERANKING.md) |
| `mcp` | optional | 1.89 | [Bounded read-only local stdio transport with five tools.](MCP.md) |
| `pdf` | optional | 1.89 | [Maintained bounded PDF import and supervised CLI path.](PDF-STABILITY.md) |
| `pilot` | optional | 1.89 | [Explicit pilot capture/export and live reference profiles.](PILOT.md) |
| `plotting` | compatibility-marker | 1.89 | [Retained empty feature flag; no standalone plotting API is switched by this flag.](../STARTUP.md) |
| `provenance` | optional | 1.89 | [Explicit shared source definitions and checked locations.](PROVENANCE.md) |
| `quality` | optional | 1.89 | [Inspectable ranking metrics from supplied judgments.](QUALITY.md) |
| `quality-gates` | optional | 1.89 | [Comparable runs and explicit quality/resource policies.](QUALITY-GATES.md) |
| `rank-explanations` | optional | 1.89 | [Inspectable learned and feedback ranking contributions.](RANK-EXPLANATIONS.md) |
| `reasoning` | optional | 1.89 | [Bounded explicit rule proposals and proof chains.](REASONING.md) |
| `reranking` | optional | 1.89 | [Provider-neutral scoring of actual candidate prefixes.](RERANKING.md) |
| `retrieval` | optional | 1.89 | [Owned bounded excerpts, evidence and complete JSON.](RETRIEVAL.md) |
| `shared-index` | optional | 1.89 | [Shared immutable payloads across cloned indexes.](SHARED-INDEX.md) |
| `snapshots` | optional | 1.89 | [Checked publication of coherent immutable dataset views.](SNAPSHOTS.md) |
| `sqlite-storage` | optional | 1.89 | [Own-format SQLite original-record adapter.](STORAGE.md) |
| `storage` | optional | 1.89 | [Atomic original-record batches, fenced revisions and pages.](STORAGE.md) |
| `training-pipeline` | optional | 1.89 | [Explicit grouped training and evaluation orchestration.](TRAINING-PIPELINE.md) |
| `versioning` | optional | 1.89 | [Conflict-checked record revisions and bounded history.](VERSIONING.md) |
| `web` | compatibility | 1.89 | [Optional dashboard, HTTP/WebSocket, crawler and administration.](../STARTUP.md) |

## Boundaries and maintenance

`./test.sh` checks every declared feature separately with all targets, the all-feature combination and meaningful interaction paths. The explicit `mcp,hybrid` Clippy check protects the execution/embedding combination without requiring snapshots. These checks do not establish every possible feature subset or an independent security audit.

The application owns durable storage, source acquisition and authorization. Scope filters do not implement accounts. MCP exposes its complete loaded collection to the connected client. Native providers and damaged-document parsing retain the limits documented by their own contracts.

The separately maintained PDF backend stays pinned at 0.3.5. Its exact source provenance, owned regressions, C ABI boundary and upstream replacement gates are in [PDF-MAINTENANCE.md](PDF-MAINTENANCE.md). Original knowledge, rejected trials and raw measurements are retained. Regenerable build output can be removed after its evidence is archived.

The 0.6 milestone preserves the existing public calls and readable versioned formats. Changes to an exposed format or incompatible call require a separately identified compatibility boundary and migration instructions. A development branch and CI results do not publish a new registry version.

### Release CI architecture

The macOS job selects the officially supported `macos-15` ARM64 image, limits Cargo to two build jobs and bounds the job at 45 minutes. Cache keys and restore prefixes include architecture. Earlier macOS 26 ARM64 attempts were cancelled before any step because no runner was acquired; GitHub reported capacity constraints. An Intel macOS 26 attempt then remained in the full script beyond an hour, did not acknowledge normal cancellation promptly, and required force cancellation; its job log was unavailable afterward. Its cause remains unestablished and it is not a pass. The release receipts distinguish each hosted result from the complete local ARM64 checks. No unavailable platform job is counted as passing.
