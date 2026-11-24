# Configuration ownership and validation

AIRust keeps configuration with the component that owns the work. Core retrieval, evidence output, immutable documents, transport admission and native inference have different units and failure rules. A shared `max_bytes` or mandatory global `Config` would obscure those boundaries. The default library remains model/server/PDF/database-free.

This inventory starts from published 0.6.0, source `8c098f15a39f6085f748c56ac52e5737abdff423`. The practical-profile investigation supplies the integration problem for C1: the published example exposes native threads but fixes reranking batches at 16 and defaults to a 20-candidate prefix. The unpublished 0.6.1 candidate reuses validated library batching/prefix controls instead of introducing a parallel inference framework. Current validation/measurement status is in [the session handoff](SESSION-HANDOFF.md) and [work program](WORK-PROGRAM-NEXT.json). An inventory or tunable setting does not itself pass a performance or usability gate.

## Core and evidence settings

| Domain / feature | Existing types and defaults | Validation and ownership |
| --- | --- | --- |
| Lexical core / default | `Engine`, `SearchOptions`: BM25, limit 10, no scope. `Bm25Config`: k1=1.2, b=0.75. | `Bm25Config::new` requires finite k1>0 and finite b in [0,1]; fields are private. Search limit zero is valid. Applications bound their query/input/entry workload; example/MCP limits are not generic Engine capacity guarantees. |
| Text matching / default | `MatchOptions`: fuzzy, similarity 0.8, limit 10. | Public fields are checked by matching operations: finite similarity in [0,1], typed `MatchError`. Zero results is valid. Legacy `FuzzyOptions` uses distance/factor rules and is not a second spelling of similarity. |
| Evidence envelope / `retrieval` | `RetrievalOptions`: 10 results, 100 candidates; query/text 16 KiB each, excerpt 2048 B, output/evidence 64 KiB, ID 4096 B; metadata disabled, optional metadata bound 4096 B. | `new(limit)` uses max(limit,100) candidates. Builders validate positive candidates>=limit, positive query/excerpt/output/ID/evidence/metadata bounds. **Total text zero is valid.** Budgets count bytes in different parts of the response; do not merge them. Failures retain `Configuration`, `Budget`, `Output` and `Source` categories. |
| Excerpt/query/assessment/citations / `retrieval` | `ExcerptOptions`, `LexicalQueryOptions`, `LexicalCoverageOptions`, `AssessmentLimits`, `EvidencePolicy`, `CitationLimits`. | Dedicated typed builders bound analysis/evidence work. Relevant excerpts are opt-in. An assessment or matching quote does not prove an answer is true; validate generated quotations against exact revision-bound source spans. Read [retrieval](RETRIEVAL.md), [excerpts](EXCERPTS.md), [assessment](ASSESSMENT.md) and [citations](CITATIONS.md). |
| Documents / `documents` | `DocumentLimits`: source 128 MiB, text 256 MiB, 50000 pages, ID 1024 B, recipe 16 KiB, JSON 512 MiB. | Public fields are validated by fingerprint/snapshot construction/restoration; all six bounds must be positive. Stream reads, original hashes, UTF-8 coordinates and immutable revisions have their own contracts. A successful limit check does not authenticate a source or establish an OS memory quota. |
| Execution / `execution` | `CancellationToken`, `WorkControl`; no deadline unless requested. | Timeout addition is checked; an overflow is `InvalidBudget`. Combining deadlines retains the earlier absolute deadline. Cancellation/deadlines are cooperative. Observer failures and progress accounting are separate controls; never replace a caller deadline with a fresh longer timeout. |

## Retrieval models and transport

| Domain / feature | Existing types and defaults | Validation and ownership |
| --- | --- | --- |
| Semantic / `embeddings` | `EmbeddingSpace`, `Embedding`, `SemanticSearchOptions`, provider-owned identity/preprocessing. | Validated spaces/vectors and finite score controls. The provider supplies actual assets and preprocessing. No default model/service is started by the library. |
| Fusion / `hybrid` | `HybridSearchOptions`: limit 10, candidates 100, BM25, RRF k=60, weights 0.5/0.5, min cosine -1, no scope, strict `ReturnError`. | Candidates positive>=limit, finite nonnegative RRF smoothing, finite nonnegative nonzero combined weights (normalized safely), bounded cosine gate. `LexicalOnly` is explicit failure policy. The fusion result window does not add candidates that the first stage never returned. |
| Cross-encoder / `reranking` | `RerankOptions`: limit 10, candidates 100, batch 16, query 16 KiB, document 64 KiB, total query/document pairs 2 MiB, no minimum score, strict `ReturnError`. | Validated positive batch/candidate/byte bounds; finite minimum score; exact selected-prefix ownership, IDs, scopes, recipe and score shape checked. `KeepOriginalOrder` is an explicit provider-failure policy, not recovery from invalid configuration/output. Batch controls exist already. |
| Native adapters / `local-reranking`, `local-embeddings` | Supplied pinned assets/runtime, provider identity, caller-selected CPU threads. Native MiniLM pair recipe uses longest-first truncation, maximum 512 tokens. | Optional Rust 1.92 path. Assets are size/hash checked. Native runtime environment and library ownership remain explicit. Ordinary tests do not download assets or execute native inference. Native FFI is outside Miri. A batch/CPU change must be measured with the same candidates and model/token recipe; floating-point and ranking differences must remain visible. |
| MCP / `mcp` | `McpLimits`: 4 admitted workers, 64 pending; default timeout 10000 ms, maximum 60000 ms, progress 100 ms; query 4096 B, 50 hits, excerpt 4096 B, result/frame 64 KiB; dataset 512 MiB, file 256 MiB, entry 1 MiB. | Public `validate()` is reused by constructors and file loaders. All budgets positive; workers<=64, pending<=1024 and workers<=pending; result 4..1024 KiB, frame<=1 MiB, request IDs<=1024 B, session requests<=1m; default timeout<=maximum<=300000 ms; progress>=10 ms. Dataset/document/ID/metadata and inbound/session/rate limits remain distinct. Visibility is not an ACL. |

## Application policy and profile files

The reference application owns tighter limits than generic library code: 4 MiB per source file, 32 MiB total source text, 64 MiB encoded workspace/vector buffers and at most 100000 passages. Its query wrapper returns at most six passages, with a 4096-byte query, 12000-byte total text, 2000-byte excerpts, 48 KiB inner output and 64 KiB full reply. Pilot capture is explicit, with 128 KiB cases, at most 256 selected cases and manifest-last export publication. Read [PILOT.md](PILOT.md); queries do not automatically become user traffic records.

The optional `pilot` example profile is schema-one JSON capped at 16 KiB, with unknown fields rejected. Relative model/cache/runtime paths resolve against the profile file directory. It is example application policy, not a new library-wide persistence format.

| Profile | Required | Forbidden / failure behavior |
| --- | --- | --- |
| `bm25` | Explicit profile/schema/failure policy. | Embedding, vectors and reranker must be absent. No model/service is required. |
| `hybrid` | Embedding identity/budgets and vector cache. | No reranker. |
| `hybrid_rerank` | Embedding, vector cache and native reranker paths/threads. | Native adapter feature and supplied runtime/assets must exist for execution. |

Embedding endpoints must be explicit loopback HTTP, with no credentials/query/fragment/non-root path, redirects or proxies. Model digest is 64 lowercase hexadecimal characters; dimensions 1..4096; HTTP timeout 1..30000 ms. Native example threads are 1..64. Preparation/model/cache failures are errors even when lexical fallback is selected. Runtime fallback is visible and cannot turn cancellation, deadlines, malformed output or configuration failure into success.

In the 0.6.1 candidate, native profile batch size is explicitly configurable in 1..16, default 16. An optional explicit candidate prefix is 1..100; absent/null retains the legacy max(20, requested result limit), preserving custom callers asking for more than 20. Explicit budgets are not silently increased. `profile-check CONFIG` requires `pilot`, validates this same configuration/path logic without loading data/models or contacting services, and returns a bounded report. It does not verify cache identity, asset existence or service readiness; actual execution still does. Older 0.6.0 readers reject new fields. See [migration](MIGRATION-0.6.1.md).

Precedence is deliberate: library defaults initialize their own options; explicit application builders/profile settings replace only those local defaults; per-request settings remain subject to transport and application bounds; the earlier cooperative deadline wins. A profile cannot broaden scopes or waive server validation. The reference model profile supports BM25 fusion; an explicit TF-IDF request is a configuration error rather than silently ignored.

## Other optional configuration domains

These interfaces remain feature-local; do not pull their dependencies into the default core or replace their validation rules with generic positivity checks.

| Existing configuration | Feature / contract |
| --- | --- |
| `AnalyzerConfig`, `TextSearchOptions` | `advanced-search`; tokenizer/stemming recipe and bounded analysis, [text search](TEXT-SEARCH.md). |
| `HnswConfig`, `AnnSearchOptions` | `hnsw`; validated construction/search widths and explicit exact fallbacks, [ANN](ANN.md). |
| `ContextOptions`, `TraversalOptions` | `context-assembly` / `graph`; scope, node/text/neighborhood limits, [context](CONTEXT.md), [graph](GRAPH.md). |
| `CacheLimits`, `CacheIdentity` | `cache`; file/entry/dimension/binding limits and identity, [cache](CACHE.md). |
| `StorageLimits`, `SnapshotOptions`, `DatasetConfig` | `storage` / `snapshots`; records/atomic publication/retained readers and snapshot identities, [storage](STORAGE.md), [snapshots](SNAPSHOTS.md). |
| `DiskOptions`, `DiskSearchOptions` | `disk-index`; on-disk index/query work and recovery, [disk index](DISK-INDEX.md). |
| `FeedbackLimits`, `FeedbackOptions`, `TrainingConfig`, `TrainingOptions` | `feedback` / `learned-ranking`; bounded feedback/training with validated provenance, [feedback](FEEDBACK.md), [learning](LEARNED_RANKING.md). |
| `ReasoningOptions`, `FreshnessOptions`, `ExplanationOptions` and quality/training gates | `reasoning`, `evidence-freshness`, `rank-explanations`, `quality-gates`, `training-pipeline`; explicit rules, evidence/quality policies and bounded explanations, dedicated feature guides. |
| `PdfLoaderConfig`, `PdfImportLimits` | `pdf`; validated chunk ranges/overlap/weight and bounded files/pages/text/chunks. This is the AIRust wrapper; separately maintained backend source/configuration remains unchanged, [PDF parser](PDF-PARSER.md). |
| Legacy agent/training/web settings | Keep compatibility through their existing constructors/validation and supported feature boundaries; they are not a reason to make web/PDF/database/model dependencies mandatory. |

## Evidence required before recommending a setting

The current practical study uses four exposed local inputs and a current-source documentation corpus. Its historical 150-case reference controls are assistant-authored. They can reveal defects, slower cases and numerical/ranking changes; neither set establishes independent customer quality. Valid source ranges and valid quotations are necessary but do not prove semantic answer completeness. The study already found a generated quote mismatch and a confident historical-roadmap answer.

Compare compatible baseline/candidate builds, identical source/model identities and candidate/token bounds. Keep raw samples, first/setup/full-request costs, all losses and native FFI limitations. A tunable option is not an optimization until measured. Independent 15-minute onboarding, genuine source-reviewed cases (minimum 50; currently zero established) and external reproduction remain open. No future release is authorized merely by adding this document.
