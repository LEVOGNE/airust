# AIRust roadmap

For a new session, start with [SESSION-HANDOFF.md](docs/SESSION-HANDOFF.md), then inspect [the next work program](docs/WORK-PROGRAM-NEXT.json). The roadmap defines direction and acceptance evidence; it does not authorize blindly publishing every listed version. Follow [engineering priorities](AGENTS.md) and [release requirements](CONTRIBUTING.md).

## Current baseline and priorities

AIRust 0.6.0 completes the engineering work for the bounded local pilot and explicit CLI/MCP search profiles. It includes the pilot originally planned for 0.5.4; no intermediate release was omitted from testing. The [release receipts](https://github.com/LEVOGNE/airust/releases/tag/v0.6.0), [validation](docs/validation-0.6.0/README.md) and [measurement record](docs/measurements/pilot-0.6.0/README.md) establish the actual source, checks and publication state. Original evidence is not rewritten when final documentation changes.

Prioritize memory safety and performance together with stability, robustness and honesty. Plan roughly 70% of effort for quality, documentation, integration and actual usage, and 30% for justified new capabilities. This is a planning preference, not measured time accounting. Keep default features empty. Extend existing configuration, cancellation, deadlines, admission, storage and model interfaces before adding another layer. Do not add a database, tensor framework or search algorithm without an observed workload need and comparative evidence.

The next product question is which concrete local-document/evidence task makes AIRust useful to a developer. Compare appropriate existing tools, such as SQLite, Tantivy or LanceDB, with comparable source data, scopes, models and cost accounting before claiming an advantage. The historical [0.4.4 comparison](docs/HONEST-COMPARISON-0.4.4.md) remains evidence, including its negative results.

## Five product-readiness gates

| Gate | Required observed result | Current evidence and open work |
| --- | --- | --- |
| G1: developer onboarding | A developer independent of this implementation completes the documented default-library example and a real query in at most 15 minutes, without manual source fixes. Record starting prerequisites, elapsed time, help and failures. | Engine/KnowledgeEntry work without MCP/server/models. A nine-line Rust 1.89 consumer is part of the 0.6.0 release checks. No independent timed onboarding session is established. |
| G2: complete reference application | Follow the documented import/index/query/MCP path on a clean supported environment without undocumented edits; preserve original files and exact source revisions. Record an actual external LLM-client integration separately from protocol tests. | The reference application and automated real CLI/stdio tests exist. Independent clean-machine execution and external-client experience remain open. |
| G3: independent retrieval quality | At least 50 genuine independently source-reviewed questions, including unsupported cases; freeze source-family splits, judgments, policy and numerical budgets before comparison. On answerable cases require nDCG@10 >= 0.75 and Hit@6 >= 0.80, zero range mismatches/scope leaks and predeclared optional-profile gain/cost criteria. | Zero eligible cases are established at the 0.6.0 closeout. The 150 assistant-authored reference-v1 cases and three unreviewed historical-question replays do not meet this gate. No new production-profile recommendation is made. |
| G4: reproducible operating costs | Retain compatible baseline/candidate builds, locks, source/model identities and every raw sample/regression. Separate retrieval, complete warm requests, cold startup, worker memory and external-service memory; publish independent reproduction and unavailable observations. | Finite Core/MCP/scale/live-model campaigns exist. Unique GPU/shared physical accounting and independent reproduction remain unestablished. Same-host archive reproducibility does not prove cross-host binary reproducibility. |
| G5: compatibility contract | Review supported public APIs, error categories, saved formats and migration policy; demonstrate old-data restoration and independent consumers on supported compilers/platforms. A stable 1.0 contract needs operational evidence outside the authored reference set. | Pre-1.0 Cargo boundaries, version-one source/workspace formats and release consumers are documented. A reviewed 1.0 contract and external operational experience remain open. |

The [independent-validation protocol](docs/INDEPENDENT-VALIDATION.md) specifies the attempts and evidence to collect. Automated agent reviews, synthetic fixtures and caller-declared human roles cannot supply independent human validation. Do not mark a gate passed because its tooling exists.

## Conditional version direction

The third version component is a patch; 0.5 to 0.6 is a minor boundary. The 0.6.x entries below are compatible work targets, not mandatory one-feature releases. Combine coherent work when appropriate. Move a public break to a new minor line and document the migration. Every release retains its applicable checks; version count is not test coverage.

| Target | Intended work | Acceptance and boundaries | Status |
| --- | --- | --- | --- |
| **0.6.0** | Developer entry point, bounded pilot and explicit live profiles | Complete release checks and source/tag/registry identities; publish measured limits and keep G1–G5 external requirements explicit. | Released and verified; authoritative receipts retained. |
| 0.6.1 | Configuration clarity and measured resource controls | Reuse existing configuration; bounded reranking batches/prefixes and preflight; shared training snapshots, bounded input/diagnostics and optional verified native snapshots. Practical local LLM/profile evidence precedes changes. | Validated unpublished candidate: native/compiler/five API views, 121 finite Miri cases, full selected fuzz scopes, final-binary controls, clean engineering-source package/consumers and all five hosted source jobs passed. Documentation closure reuses only verified unchanged code. Independent human gates and explicit publication authorization remain open. |
| 0.6.2 | Diagnostics | Reproducible structured error/cost information for observed failure paths; no sensitive source leakage, hidden work or unstable error-category changes. Measure overhead. | Planned conditionally. |
| 0.6.3 | Observability | Opt-in timing/resource/quality observations with bounded storage/cardinality and measured overhead. Distinguish request cost from retrieval and reference labels from actual user outcomes. | Planned conditionally. |
| 0.6.4 | Resource management | Extend existing admission, queue, byte/count limits, cancellation and absolute deadlines with adversarial/recovery controls. State cooperative limits accurately; hard CPU/RAM isolation needs a separately verified mechanism. | Planned conditionally. |
| 0.6.5 | Security hardening | Review actual import/MCP/backend trust boundaries, malformed inputs, scopes, floods and recovery. Retain audit/provenance/feature gates; security checks also apply to every earlier release. | Planned conditionally. |
| **0.7.0** | Knowledge pipeline | Import/update/index source revisions atomically with retained old readers and failure recovery; scope from real pilot failures and reuse existing storage/snapshot APIs. | Await pilot evidence. |
| 0.7.1 | Incremental sync | Reprocess changed content only, with deterministic revision invalidation, deletion/rename/restart recovery and compatible data. | Conditional on pipeline contract. |
| 0.7.2 | Retrieval quality | Publish the independent comparison under G3/G4 and all individual losses. Start collection now; do not delay it until this version or retune frozen v1. | Await independent inputs. |
| **0.8.0** | Model integration | Extend existing provider/local-model interfaces only for a demonstrated integration need; explicit assets/identities, bounded work, strict/fallback controls and no mandatory core model. | Conditional; existing interfaces already exist. |
| **0.9.0** | SDK and ecosystem | Examples/adapters/tools used by actual external consumers, with measured onboarding, supported versions and ownership boundaries. Split crates only for demonstrated dependency/release benefit. | Await actual consumers. |
| **1.0.0** | Stable release | G1–G5 observed, reviewed API/data guarantees and supported-platform operational experience. No fixed date or feature-count shortcut. | Evidence gate remains open. |

## Start the next work here

1. **R0 — preserve the verified 0.6.0 closeout.** Publication and closing checks passed; never republish or move the tag. [Practical local LLM/profile evidence](docs/measurements/profile-practical-0.6.0/README.md) is complete. The current engineering task is the [unpublished 0.6.1 candidate closeout](docs/SESSION-HANDOFF.md), with observed gates and remaining work in the handoff.
2. **DX1 — independent onboarding and application attempts.** Use the prepared [protocol](docs/INDEPENDENT-VALIDATION.md). Keep failure reports and assistance levels. The agent can prepare/verify artifacts, but cannot act as an independent human participant.
3. **Q1 — genuine pilot collection and source review.** Use [PILOT.md](docs/PILOT.md); retain originals/private data outside Git. Freeze reference-v2 only after the independently reviewed minimum, source-family split and policy exist. Never substitute conversation replays.
4. **C1 — preserve the validated unpublished candidate.** Configuration/resource changes, native/compiler/API/Miri/fuzz/final-binary controls and clean engineering-source package/consumer/CI checks passed. Read the final closeout receipt before repeating work. The next conditional D1 step inventories existing diagnostics and actual observed failures; no new facade or automatic publication.
5. **QA1 — independent reproduction and compatible alternatives.** Reproduce the public artifacts on another supported environment; compare a concrete workload with existing tools under a frozen policy. Retain all failures and source/compiler/model/lock differences.

External human-input tasks may remain awaiting inputs while independent engineering work progresses. They are not complete and must not be fabricated. Update [WORK-PROGRAM-NEXT.json](docs/WORK-PROGRAM-NEXT.json) and the handoff after each coherent step with commands, source identities, results and the next action. Publishing a later version requires its own explicit release authorization.

## Component boundaries

AIRust is the project and knowledge-engine library. `airust` is the small core crate. MCP transport, import/search/check CLI workflows and optional local-model adapters are conceptual components. The current `airust-mcp` executable is shipped in the main crate. Separate `airust-cli` and `airust-model` crates are not asserted to exist. Keep default builds independent of HTTP, PDF, databases and models; do not split working modules merely to match a product diagram.

## Completed engineering foundation: 0.6.0

The [seven-stage engineering work program](docs/WORK-PROGRAM-0.6.0.json) covers immutable pilot capture/review/export, independent export validation, explicit live profiles, bounded trusted MCP backends, native/feature/MSRV/Miri controls, paired complete-request/scale/model measurements and retained publication evidence. The [migration guide](docs/MIGRATION-0.6.0.md) lists the 100000-passage example bound and preserved ordinary formats. The empirical real-data gate remains separate and unmet; no checklist below changes that fact.

## Completed foundation: 0.5.3

The 0.5.3 work program turns the verified 0.5.2 baseline into a reproducible reference application: local Markdown search with exact source coordinates, bounded evidence and optional MCP. The [reference use case and frozen acceptance policy](docs/REFERENCE-APPLICATION.md) define completion. The [150-question comparison](docs/measurements/reference-v1/README.md) retains failed targets, local model costs and individual losses. [WORK-PROGRAM.json](docs/WORK-PROGRAM.json) records the seven implementation stages and their evidence.

Further quality work requires a new dataset/version, genuine user questions or independent judgments. Version-one reserved results are not a tuning set. Release 0.5.3 publishes this completed milestone under the existing 0.5.x compatibility contract. Keep provider-neutral defaults small; real external-application integration remains separate.

## Release priorities

Every release prioritizes stability, performance, reliability and a clear public API, including optional MCP. Preserve the small optional architecture. Run compatibility and feature checks, test cancellation/budgets/scope where affected, and report measured costs and regressions honestly. Keep original knowledge and evidence; remove regenerable build artifacts after verified publication. New capabilities require a demonstrated use and must not silently change defaults. Model training and agent expansion are possible long-term research directions, not committed version milestones.

0.5.0 completes the scoped [stability route](docs/ROADMAP-TO-0.5.0.md). Retain the existing Core/MCP runtime, independently check mutation/cache/clone state, rerun real-corpus retrieval and finite million-entry controls, and document the new Cargo line. 0.5.1 and 0.5.2 follow that rule: the fuzz checks found no defect to change, so the runtime stays at 0.5.0 and no new measurement campaign was invented. Future 0.5.x work should follow demonstrated defects and representative application measurements. Stability, performance, reliability and API/MCP remain priorities; new optional capabilities need separate evidence and scope.

The verified 0.5.2 baseline is commit `6031fe5f5fda4727e13d89497aa238797cf6474f`. Its [post-release closeout record](docs/validation-0.5.2/closeout.json) binds the published archive, exact-commit CI, repeated native checks and a pinned Rust 1.89 registry consumer to that baseline. The consumer manifest, lock and source are retained beside the record; its reproduction steps use the published crate. Raw evidence is retained locally and dedicated build directories were removed. These subsequent audit files are separate from the immutable published 0.5.2 archive.

AIRust's direction is a universal Rust knowledge engine. Any application can supply records and consume ranked evidence. The engine should remain independent of a particular document editor, mind-map format, database, or agent transport.

The 0.1.9 baseline remains available. The 0.2.0 release adds optional provider-neutral embeddings and an explicitly configured local E5 adapter. Later releases are implemented and validated one at a time. External application integrations remain separate work.

| Target | Scope | Status |
| --- | --- | --- |
| 0.5.3 | Reference application, frozen evaluation and measured maintenance | Implemented and released. Read [scope](docs/CHANGES-0.5.3.md), [migration](docs/MIGRATION-0.5.3.md) and [release verification](docs/validation-0.5.3/README.md). |
| 0.5.2 | Coverage-guided fuzzing with versioned seeds | Implemented; read [scope](docs/CHANGES-0.5.2.md) and [migration](docs/MIGRATION-0.5.2.md). |
| 0.5.1 | Fuzz-check the unchanged 0.5.0 runtime | Implemented; read [scope](docs/CHANGES-0.5.1.md) and [migration](docs/MIGRATION-0.5.1.md). |
| 0.5.0 | Existing Core/MCP stability and explicit upgrade | Implemented; read [validation](docs/CHANGES-0.5.0.md) and [migration](docs/MIGRATION-0.5.0.md). |
| 0.4.25 | Release raw startup JSON before indexing | Implemented; read [memory, time and compatibility observations](docs/CHANGES-0.4.25.md). |
| 0.4.24 | Bounded final MCP reply work and correct error progress | Implemented; read [MCP/API](docs/MCP.md) and [measurements](docs/CHANGES-0.4.24.md). |
| 0.4.23 | Short repeated-word Core preparation | Implemented; read [API](docs/API-COOKBOOK.md), [performance](docs/CORE-PERFORMANCE.md) and [measurements](docs/CHANGES-0.4.23.md). |
| 0.4.22 | Less duplicate MCP response work | Implemented; read [API](docs/API-COOKBOOK.md), [MCP](docs/MCP.md) and [measurements](docs/CHANGES-0.4.22.md). |
| 0.4.21 | Borrowed single-word preparation for small queries | Implemented; read [API](docs/API-COOKBOOK.md), [performance](docs/CORE-PERFORMANCE.md) and [measurements](docs/CHANGES-0.4.21.md). |
| 0.4.20 | Bounded ordered document lookup for large searches | Implemented; read [API](docs/API-COOKBOOK.md), [performance](docs/CORE-PERFORMANCE.md) and [all measurements](docs/CHANGES-0.4.20.md). |
| 0.4.19 | Shared exact broad-query explanations | Implemented; read [API](docs/API-COOKBOOK.md), [performance](docs/CORE-PERFORMANCE.md) and [measurements](docs/CHANGES-0.4.19.md). |
| 0.4.18 | Direct ordered merge of two candidate lists | Implemented; read [API](docs/API-COOKBOOK.md), [performance](docs/CORE-PERFORMANCE.md) and [all comparisons](docs/CHANGES-0.4.18.md). |
| 0.4.17 | Direct collection for one known query term | Implemented; read [API](docs/API-COOKBOOK.md), [performance](docs/CORE-PERFORMANCE.md) and [all comparisons](docs/CHANGES-0.4.17.md). |
| 0.4.16 | Large-query posting-frequency cursors | Implemented; read [performance](docs/CORE-PERFORMANCE.md), [API](docs/API-COOKBOOK.md) and [all comparisons](docs/CHANGES-0.4.16.md). |
| 0.4.15 | Small MCP execution without the initial timer wait | Implemented; read [MCP](docs/MCP.md) and [measurements](docs/CHANGES-0.4.15.md). |
| 0.4.14 | Warm cache-read batches and adaptive inline scratch | Implemented; see [API](docs/CORE-PERFORMANCE.md) and [measurements](docs/CHANGES-0.4.14.md). |
| 0.4.13 | Bounded cold-query TF-IDF weight reuse | Implemented; see [API](docs/CORE-PERFORMANCE.md) and [measurements](docs/CHANGES-0.4.13.md). |
| 0.4.12 | Compact transient TF-IDF cache | Implemented; see [API](docs/CORE-PERFORMANCE.md) and [measurements](docs/CHANGES-0.4.12.md). |
| 0.4.11 | TF-IDF hints and sparse broad-query scoring | Implemented; see [API](docs/CORE-PERFORMANCE.md) and [validation](docs/CHANGES-0.4.11.md). |
| 0.4.10 | Index normalization and bounded ordered posting-list merge | Implemented; see [API](docs/CORE-PERFORMANCE.md) and [validation](docs/CHANGES-0.4.10.md). |
| 0.4.9 | Faster complete retrieval and bounded MCP serialization | Implemented; see [API](docs/CORE-PERFORMANCE.md) and [validation](docs/CHANGES-0.4.9.md). |
| 0.4.8 | Query working allocations and deadline-aware MCP progress | Implemented; see [API](docs/CORE-PERFORMANCE.md) and [validation](docs/CHANGES-0.4.8.md). |
| 0.4.7 | Explicit lexical evidence coverage scoring | Implemented; see [API](docs/LEXICAL-COVERAGE.md) and [validation](docs/CHANGES-0.4.7.md). |
| 0.4.6 | Explicit lexical question preparation and retained context | Implemented; see [API](docs/LEXICAL-QUERY.md) and [validation](docs/CHANGES-0.4.6.md). |
| 0.4.5 | Explicit BM25 evidence delivery and bounded citation validation | Implemented; see [selection](docs/EVIDENCE-SELECTION.md), [citation boundaries](docs/CITATIONS.md) and [measured validation](docs/CHANGES-0.4.5.md). |
| 0.4.4 | Less temporary construction allocation traffic | Implemented; see [core behavior](docs/CORE-PERFORMANCE.md) and [validation](docs/CHANGES-0.4.4.md). |
| 0.4.3 | Faster construction and BM25 with retained memory sharing | Implemented; see [core behavior](docs/CORE-PERFORMANCE.md) and [validation](docs/CHANGES-0.4.3.md). |
| 0.4.2 | Leaner core, reusable TF-IDF norms and MCP output schemas | Implemented; see [core behavior](docs/CORE-PERFORMANCE.md) and [validation](docs/CHANGES-0.4.2.md). |
| 0.4.1 | Optional read-only MCP in the same crate, progress, cancellation and bounded work | Implemented; see [API/security](docs/MCP.md) and [validation](docs/CHANGES-0.4.1.md). |
| 0.4.0 | Document revisions and reliable UTF-8 positions | Implemented; see [document contract](docs/DOCUMENTS.md). |
| 0.2.0 | Optional embeddings and exact semantic retrieval | Implemented; see the release report for validation. |
| 0.2.1 | Hybrid lexical and semantic retrieval | Implemented; see the release report for validation. |
| 0.2.2 | Generic graph model and bounded algorithms | Implemented; see the release report for validation. |
| 0.2.3 | Structured context assembly and budgets | Implemented; see the release report for validation. |
| 0.2.4 | Optional typed source references | Implemented; see the release report for validation. |
| 0.2.5 | Versioned persistent derived-index cache | Implemented; see the release report for validation. |
| 0.2.6 | Optional HNSW with recall measurements | Implemented; see the release report for validation. |
| 0.2.7 | Generic revisions and conflict detection | Implemented; see the release report for validation. |
| 0.2.8 | Reproducible performance and quality suite | Implemented; see the release report for validation. |
| 0.2.9 | Optional intelligent reranking over retrieved candidates | Implemented and measured; see [release report](docs/CHANGES-0.2.9.md). |
| 0.2.10 | Phrase queries, optional German morphology and configurable text analysis | Implemented and validated; see [report](docs/CHANGES-0.2.10.md). |
| 0.2.11 | Interchangeable original-record storage API | Implemented and measured; see [report](docs/CHANGES-0.2.11.md). |
| 0.2.12 | Unified compact retrieval API for applications and agent transports | Implemented and validated; see [report](docs/CHANGES-0.2.12.md). |
| 0.2.13 | Optional relevant excerpts with original positions and bounded highlights | Implemented; see [report](docs/CHANGES-0.2.13.md). |
| 0.2.14 | Coherent immutable search/vector/source snapshots and checked publication | Implemented; see [report](docs/CHANGES-0.2.14.md). |
| 0.2.15 | Explicit retrieval evidence assessment | Implemented; see [report](docs/CHANGES-0.2.15.md). |
| 0.2.16 | Cooperative cancellation | Implemented; see [report](docs/CHANGES-0.2.16.md). |
| 0.2.17 | Cooperative time budgets | Implemented; see [report](docs/CHANGES-0.2.17.md). |
| 0.2.18 | Observable work phases and progress | Implemented; see [report](docs/CHANGES-0.2.18.md). |
| 0.2.19 | Reproducible million-record and concurrent stress validation | Implemented; see [report](docs/CHANGES-0.2.19.md). |
| 0.2.20 | Measured memory optimization | Implemented as opt-in shared-index; see [report](docs/CHANGES-0.2.20.md). |
| 0.2.21 | Incremental recomputation of changed analysis | Implemented; see [report](docs/CHANGES-0.2.21.md). |
| 0.2.22 | Explicit adaptive retrieval routing | Implemented; see [report](docs/CHANGES-0.2.22.md). |
| 0.2.23 | Bounded local feedback learning | Implemented; see [report](docs/CHANGES-0.2.23.md). |
| 0.2.24 | Optional small trainable ranking model | Implemented; see [report](docs/CHANGES-0.2.24.md). |
| 0.2.25 | Bounded rule-based inference proposals with evidence | Implemented; see [report](docs/CHANGES-0.2.25.md). |
| 0.2.26 | Comparable quality gates | Implemented; see [report](docs/CHANGES-0.2.26.md). |
| 0.2.27 | Evidence freshness | Implemented; see [report](docs/CHANGES-0.2.27.md). |
| 0.2.28 | Exact ranking explanations | Implemented; see [report](docs/CHANGES-0.2.28.md). |
| 0.2.29 | Controlled training pipeline | Implemented; see [report](docs/CHANGES-0.2.29.md). |
| 0.2.30 | Optional disk search index | Implemented; see [report](docs/CHANGES-0.2.30.md). |

The target 0.2.x sequence requires additive, Cargo-compatible APIs. A breaking change needs a separately justified compatibility boundary. Publishing and verifying the current release precedes work on the next.

The 0.2.10–0.2.12 sequence is implemented with separate validation and releases. Reranking remains workload-dependent. Phrase/German analysis is explicit. Storage adapters own original records and do not persist the RAM search index or alter existing application schemas. Compact retrieval stays transport-independent; optional MCP was subsequently added in 0.4.1 in this same crate. AIRust composes maintained libraries when justified instead of duplicating every subsystem.

## 1. Maintain the tested 0.1.9 baseline

Keep the English-language source and documentation consistent, retain API contracts in external documentation, and preserve Unicode behavior and existing supported data formats. Source comments were removed in 0.1.9.

Repeat validation of the default library, all optional features, independent feature combinations, the minimum Rust version, packaging, and the real HTTP/WebSocket boundaries when these areas change. Record changed behavior and migration steps. Publish changes under a new version; existing releases remain immutable.

Acceptance: repeatable checks, explicit limits, reproducible artifacts, and a change report that distinguishes executed checks from untested deployment environments.

## 2. Add an optional semantic provider boundary

The optional boundary is implemented in 0.2.0; see the [semantic API](docs/SEMANTIC.md) and [local-model guide](docs/LOCAL-EMBEDDINGS.md). Providers accept borrowed application text and return vectors tied to a stable model recipe. The caller owns external-service policy.

The contract must define vector dimension, finite values, empty-input handling, normalization, timeouts, cancellation, and failure reporting. Index metadata must record the provider and model fingerprint. Vectors from incompatible models must not silently share a search space.

Hybrid ranking is separately implemented in 0.2.1; see the [hybrid guide](docs/HYBRID.md). Preserve a useful lexical path when a provider is unavailable. Do not add mandatory model downloads, cloud accounts, or HTTP dependencies to the default core.

Acceptance: interchangeable test providers, vector validation, model-mismatch detection, stable result ordering, and measured retrieval quality on an identified corpus. Spelling similarity must remain distinct from semantic similarity.

## 3. Extend relation proposals with typed evidence

Build typed proposals on top of shared retrieval candidates. Possible future relation types include broader or narrower topics, prerequisites, and contradictions. Each type needs its own evidence requirements.

A proposal should identify source and target versions, supporting passages or offsets, the proposing method, and uncertainty. A numeric similarity score alone must not become a claim of causality, dependency, or contradiction. Explicit application-provided relationships should remain distinguishable from inferred proposals.

Use a proposal lifecycle such as suggested, reviewed, accepted, and rejected. The calling application owns review and persistence. Evaluate each relation type against labeled examples, including unrelated and contradictory statements that share vocabulary.

Acceptance: inspectable evidence, provenance, uncertainty, and per-type quality measurements. No automatic conversion of an unreviewed inference into stored fact.

## 4. Introduce a versioned persistent index

Choose a persistence strategy from measured requirements. Source entries, derived indexes, and model-specific vectors need explicit format versions and integrity checks. Record tokenizer and normalization versions so changes cannot leave stale derived state in use.

Define atomic snapshots or transactional updates, crash recovery, corruption detection, and upgrade or rebuild behavior. Preserve a portable source-entry export even if an optimized internal format changes.

Test interrupted writes, partial updates, incompatible formats, damaged files, and large rebuilds. Multi-process access requires an explicit locking and consistency policy.

Acceptance: documented recovery rules, deterministic rebuilds, compatibility tests, and measured startup and update costs. A persisted file is not automatically a database concurrency guarantee.

## 5. Accept structural context through generic adapters

Allow applications to provide optional structural evidence: parent IDs, typed links, paths, collections, or bounded graph neighborhoods. Use application-owned identifiers and generic structures.

Keep structural context separate from lexical relevance and semantic-provider scores. Document how an adapter influences candidate selection and ranking. Prevent content excluded by an application scope from re-entering results through related entries.

The core should not contain a built-in mind-map model or assume a specific rendering system. An external application may map its nodes and edges into the adapter only after the base engine's contracts are tested.

Acceptance: useful adapters for more than one data shape, explicit scope behavior, cycle and missing-ID handling, and controlled limits on neighborhood expansion.

## 6. Measure scale at 10k, 100k, and 1m entries

Use separate corpus sizes of 10,000, 100,000, and 1,000,000 entries. A graph application may call these records nodes; the engine benchmark should still describe entry count, text length, metadata size, and term distribution.

Record a corpus generator or dataset version and random seed. Include multilingual Unicode samples, duplicates, common terms, rare terms, long texts, misses, and skewed term distributions. Define the query set independently of the indexed corpus generator.

Measure:

- Initial build time, incremental insertion, replacement, and deletion.
- Cold and warm query latency, including median, p95, and p99.
- Peak resident memory, final index size, and allocations where available.
- Search, matching, and relation proposals with several result limits and thresholds.
- Correctness against reference calculations or an independent full rebuild.
- Retrieval quality using labeled relevance judgments, with recall and ranking metrics.

Report hardware, operating system, Rust version, dependency lockfile, build profile, repetitions, and background load. Preserve raw samples and the exact command. State failures and incomplete runs rather than extrapolating a capacity result from a smaller corpus.

The existing 1,000-entry example is a historical smoke benchmark. It does not establish performance at these larger sizes.

## 7. Evaluate context and token savings honestly

When a calling application supplies retrieved passages to a language model, measure its complete baseline prompt and its retrieved-context prompt with the same identified tokenizer and model configuration. Include instructions, metadata, references, and formatting overhead in both counts.

Token reduction is `1 - retrieved_prompt_tokens / baseline_prompt_tokens`. Report the actual counts, the query workload, and answer-quality or evidence-recall changes. A smaller prompt is useful only if the required information remains available.

An example ratio derived from assumed input sizes is illustrative. It must not be labeled a measurement. No 99.8% reduction, universal cost saving, or fixed answer-quality claim is established by the current repository.

Acceptance: reproducible token counts, comparable prompts, identified tokenizer, query-level distributions, and a clear separation between measured results and illustrative calculations.

0.2.11 delivers the optional record Storage Backend API. 0.2.12 adds the compact transport-neutral Knowledge Retrieval API. Application and MCP integration remain separate.

Permanent principles: optional capabilities, reproducible measurements and inspectable decisions. No optimization is accepted on an unsupported speed or quality claim. Learning is explicit and local; rule conclusions remain proposals. No mandatory model or application integration.

## 0.3.0 stability release

Validate existing behavior and formats before adding capabilities. See the [release report](docs/CHANGES-0.3.0.md) and [stability contract](docs/STABILITY.md). Further changes require measured benefits and explicit compatibility notes.

## 0.3.x document hardening before 0.4.0

0.3.1 fixes measured PDF failures and preserves the local document lab. 0.3.2 adds opt-in import budgets and fixes CLI argument and terminal handling. 0.3.3 adds supervised worker execution for the measured blocking native parser path, with validated parent publication and explicit real-time/RAM/disk boundaries. Further changes require reproducible failures or measured benefits. Continue with findings-driven stability patches, reproducible real-document comparisons and bounded adversarial campaigns. Do not create versions merely to reach a count. General document/passages features for 0.4.0 remain deferred until the stability gates are met. Known parser latency and resource-boundary limitations must remain visible. See [PDF stability](docs/PDF-STABILITY.md).

0.3.4 optimizes the measured long-line object-loading bottleneck through an optional maintained backend, with raw-text equivalence and paired CLI measurements. Structure-tree ordering remains complete. Remaining fallback costs and hard resource isolation retain their documented limits; 0.4.0 feature expansion remains separate. See [parser results](docs/PDF-PARSER.md).


0.3.5 continues stabilization with larger official PDFs, authored format cases and early pagewise chunk-budget rejection. Parser caches are released before duplicating training text. Measurements, retained knowledge and explicit limits precede any 0.4.0 expansion. No application or MCP integration is included.


0.3.6 continues stabilization by correcting PDF text fidelity and testing further official large references. Old knowledge remains intact, changed text is audited, and callers explicitly rebuild derived data. No 0.4.0 feature expansion or application/MCP integration is included.


0.3.7 limits PDF previews to useful prefixes, verifies more official references and preserves current knowledge, full-import text and ranking behavior. Native opening/resource limits remain explicit. General 0.4.0 expansion and application/MCP integration remain separate. See [preview contract](docs/PDF-PREVIEW.md).


0.3.8 corrects measured chunk-text fidelity while retaining native extraction, cleanup controls and resource limits. Current imports and old snapshots are kept separately, with explicit derived-index rebuilding. Further changes remain findings-driven; no 0.4.0 expansion or application/MCP integration is included. See [chunk contract](docs/PDF-CHUNKS.md).


0.3.9 reduces measured overlap-processing waste while keeping source-excerpt fidelity and existing APIs/budget semantics. Real imports and synthetic extracted-text stress measurements remain separate. Further changes stay findings-driven; 0.4.0 expansion and application/MCP integration remain deferred. See [overlap controls and limits](docs/PDF-OVERLAP.md).


0.3.10 completes the limited Latin-ligature cleaning policy and reduces intermediate text copies. Independent normalization checks, actual document comparisons and separate helper/import measurements precede further changes. Old knowledge remains retained; 0.4.0 expansion and application/MCP integration stay deferred. See [cleaning contract](docs/PDF-CLEANING-PASS.md).


0.3.11 reduces unnecessary character-distance work in token-overlap matching while preserving complete results. Real-knowledge update/rebuild, JSON, concurrency and resource checks strengthen the [0.3.x stability gates](docs/STABILITY-0.3.md). The next proposed scope is a small optional document/passage contract with explicit source revisions and truthful position semantics; implement and measure it separately before a 0.4.x release. No application/MCP integration is included.


0.3.12 addresses repeated entry-ID allocations identified by the real-corpus profile. Complete results, caches, raw IDs and retained views must remain compatible; isolated prototypes precede final paired resource checks. Further targeted malformed-structure tests and the [0.3.x gates](docs/STABILITY-0.3.md) precede 0.4.x expansion. No application/MCP integration is included.


0.3.13 closes the demonstrated native Form/fallback-page-count omissions with explicit errors at existing work limits. Retain supported complete results, old output on failure and cross-API cache correctness. Fresh real-document, malformed-input and publication checks belong to its report. Next, use the [0.4 preparation gates](docs/PREPARING-0.4.md) to evaluate a small optional document/passage contract against existing source, snapshot and excerpt APIs. No application or transport integration is included.


0.3.14 adds a bounded, independent PDF page-tree/count check before extraction, closing the positive root-count omission and checking intermediate counts as well. Keep backend 0.3.5 unchanged and measure complete conversion plus bounded-preview costs separately. Future text snapshots should reuse this structural check while keeping extraction coverage, byte identity and Unicode coordinate spaces separate. See [results](docs/CHANGES-0.3.14.md) and [0.4 preparation](docs/PREPARING-0.4.md).


## Close the planned 0.3.x cycle

0.3.15 is the final planned stability release. It reduces measured PDF validation memory and expands real-document evidence and retained knowledge. Additional 0.3.x patches require concrete defects; there is no target patch count. The next planned scope is the small optional document/revision/position API described in [0.4 preparation](docs/PREPARING-0.4.md). Keep source identity, Unicode coordinates and extracted-text offsets explicit; validate the prototype before publishing 0.4.0. No DenkMaps or MCP integration belongs to this work.


## 0.4.0: implemented document contract

The optional document feature promotes the tested prototype to a bounded, versioned text-snapshot/passage API. Source/text/recipe/page identity, UTF-8 bounds, explicit coordinate spaces and checked restoration precede application adapters. Existing core APIs and knowledge formats remain. See [API and migration](docs/DOCUMENTS.md) and [executed evidence](docs/CHANGES-0.4.0.md). Future work requires concrete measurements and retained failures; semantic/OCR improvements and transport integrations remain separate.
