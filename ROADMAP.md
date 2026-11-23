# AIRust roadmap

## Release priorities

Every release prioritizes stability, performance, reliability and a clear public API, including optional MCP. Preserve the small optional architecture. Run compatibility and feature checks, test cancellation/budgets/scope where affected, and report measured costs and regressions honestly. Keep original knowledge and evidence; remove regenerable build artifacts after verified publication. New capabilities require a demonstrated use and must not silently change defaults. Model training and agent expansion are possible long-term research directions, not committed version milestones.

0.4.17 targets the measured small Core regressions by avoiding a merge for one already sorted candidate list. Keep the large-query frequency cursors, original checkpoints, exact ranking and public contracts. Compare fresh 0.4.15/0.4.16/0.4.17 small controls, a broad query/filter matrix and full MCP round trips under recorded host load. Preserve every slower outcome. Following releases should improve a demonstrated usage path before adding capabilities.


AIRust's direction is a universal Rust knowledge engine. Any application can supply records and consume ranked evidence. The engine should remain independent of a particular document editor, mind-map format, database, or agent transport.

The 0.1.9 baseline remains available. The 0.2.0 release adds optional provider-neutral embeddings and an explicitly configured local E5 adapter. Later releases are implemented and validated one at a time. External application integrations remain separate work.

| Target | Scope | Status |
| --- | --- | --- |
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
