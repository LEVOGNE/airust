# Changelog

## 0.4.12 — Compact cold TF-IDF hints

Store norm hints by a compact identity of the ID allocation already owned by the central index. Keep standard randomized hashing, exact scores, invalidation, complete-value publication and nonblocking contention/poison fallback. Avoid extra ID reference retention and string hashing without adding per-document fields, dependencies or public options. Add identifier-ownership, concurrent cold-query and retained-clone/reused-ID regressions. Measure cold latency and hint heap separately from warmed queries and process RSS. Preserve sources, knowledge and MCP schemas. No DenkMaps integration. See [API/performance](docs/CORE-PERFORMANCE.md) and [honest measurements](docs/CHANGES-0.4.12.md).

## 0.4.11 — TF-IDF hints and sparse broad scoring

Use a protected hash dictionary for private TF-IDF norm hints with unchanged invalidation and nonblocking lock/poison fallback. Borrow Arc keys in wide candidate sets and intersect sparse document vocabularies with broad prepared queries while retaining contribution order, arithmetic and checkpoints. Add exact full-scan score/cold-warm/scope oracles and a sparse-scoring interruption test. Preserve signatures, schemas, sources and knowledge. Benchmark initial and repeated MCP requests separately, report cold hint memory, and retain broader-query fixtures. No dependencies or DenkMaps integration. See [API/performance](docs/CORE-PERFORMANCE.md), [MCP](docs/MCP.md) and [honest report](docs/CHANGES-0.4.11.md).

## 0.4.10 — Index normalization and ordered candidate merge

Use direct ASCII normalization with the same case/Unicode-whitespace rules. Merge at most 32 already sorted known-term posting lists into unique sorted borrowed IDs using a bounded heap; retain the prior ordered-set route for wider queries. Keep checkpoints for every posting, score arithmetic/order, filtering, winner ties, persistent state and MCP contracts. Add independent normalization and candidate-set/filter/update oracles. Repeat full validation, construction process pairs, allocation profiles and two complete MCP campaigns. No new dependency or DenkMaps integration. See [API/performance](docs/CORE-PERFORMANCE.md), [MCP](docs/MCP.md) and [executed report](docs/CHANGES-0.4.10.md).

## 0.4.9 — Faster complete retrieval and bounded MCP serialization

Use bounded direct ASCII excerpt tokenization and avoid Unicode normalization for ASCII graphemes in mixed text. Share retained JSON serialization and size-only counting through one bounded writer; avoid discarded buffers in candidate, metadata and MCP guards. Preserve public signatures, ranking, source positions, byte limits, schemas and error codes. Add exact escaped-budget/session-recovery and ASCII/Unicode/CRLF/span regressions. Repeat full validation and exact-registry old/new benchmarks. No new dependencies or DenkMaps integration. See [API/performance](docs/CORE-PERFORMANCE.md), [MCP](docs/MCP.md) and [honest report](docs/CHANGES-0.4.9.md).

## 0.4.8 — Query working allocations and MCP progress deadlines

Reuse normalized term keys for repeated occurrences and existing lexical uniqueness guarantees for coverage counts. Keep scorer arithmetic and retrieval responses unchanged. Apply the original request deadline and connection cancellation to start/heartbeat/final MCP progress waits, retain typed deadline errors and avoid starting expired work. No new dependencies or DenkMaps integration. See [API/performance](docs/CORE-PERFORMANCE.md), [MCP](docs/MCP.md) and [executed report](docs/CHANGES-0.4.8.md).


## 0.4.7 — Inspectable lexical evidence coverage

Add optional `LexicalCoverageSource` and bounded preparation/scoring options. Retain prepared lexical scores/ranks and expose distinct-term coverage diagnostics. Require patched optional `url >=2.5.9` and exercise nested-origin regressions. Fresh comparison retains 180 local replies, 5,130 timed retrieval samples and exposed/fresh quality limits. Default lexical search, index formats, citation validation and MCP behavior remain. See [API](docs/LEXICAL-COVERAGE.md) and [executed report](docs/CHANGES-0.4.7.md).


## 0.4.6 — Inspectable lexical questions and preserved context

Add explicit bounded English/German function-word filtering with protected identifiers/quoted terms and ASCII CamelCase/acronym/digit expansion to LexicalSource. Keep literal Engine and ordinary retrieval defaults. Add optional leading-context preference that preserves all term coverage and explicit phrase positions of the relevant window. Retain the 0.4.5 citation checks. Compare against published 0.4.5 BM25 and SQLite hybrid using exposed development data and newly frozen questions; report actual quality/cost tradeoffs. No mandatory models, new dependencies or DenkMaps integration. See [query API](docs/LEXICAL-QUERY.md) and [release report](docs/CHANGES-0.4.6.md).


## 0.4.5 — Explicit evidence selection and citation checks

Keep Engine BM25 and default retrieval/MCP behavior. Add optional complete-paragraph selection and exact delivered-excerpt deduplication with bounded refill, original positions/ranks and selection diagnostics. Add provider-neutral strict bounded cited-answer parsing and reference/quote/range validation against immutable delivered context. Distinguish ID existence, exact quotations and semantic truth. Add independent overlapping-quote/paragraph oracles, scope/revision/budget regressions and a separate Rust consumer. Compare the unchanged published 0.4.4 baseline, the explicit new selection and SQLite hybrid on exposed and newly authored questions. No DenkMaps integration or mandatory model. See [API/limits](docs/CITATIONS.md) and [tests and honest measurements](docs/CHANGES-0.4.5.md).

## 0.4.4 — Less temporary allocation traffic

Added a private bounded token batch that aggregates repeated word counts in place before interning and ordered collection. Empty/single-token input avoids oversized reservations. The centralized KnowledgeIndex, streaming tail, stable ranking, retained memory sharing, norm invalidation and vocabulary compaction remain. Added prefix/Unicode/lifetime regressions, interpreter checks and a reproducible differential stress consumer. Fixed cross-connection MCP shutdown interference with per-connection worker tracking, skipped stale progress heartbeats and verified batch-array rejection. Reused immutable per-server tool definitions with caller-edit/configuration isolation and a reproducible connection-allocation probe. Public APIs, knowledge/cache formats, optional MCP schemas and PDF backend 0.3.5 remain. No DenkMaps integration. See [behavior and limits](docs/CORE-PERFORMANCE.md) and [executed validation](docs/CHANGES-0.4.4.md).

Eight authored allocation distributions also measure long-word over-reservation: fewer allocation calls can still mean more working bytes. Build, warm-query, clone and release phases retain equal live heap in those comparisons. The real-document improvements and these limits are reported separately. Current API, support and future application-adapter guides are updated in English.

## 0.4.3 — Faster construction and BM25

Count a bounded prefix of borrowed tokens through sorted grouping, stream long-entry tails and use the standard randomly seeded hash dictionary for shared vocabulary lookups. Prepare query IDF once, compute BM25 normalization once per candidate and retain borrowed scores until top-K winners are selected. Allocate matched-term explanations only for winners. Preserve exact arithmetic order, deterministic ties/scopes, shared index memory, TF-IDF hints, knowledge/cache formats and MCP response version one. Add independent extreme-parameter/full-scan, batch-boundary/Unicode and deferred-explanation regressions. Recheck retained knowledge, real MCP/schema replies, minimum Rust, compatibility and memory/build/query costs. Backend remains 0.3.5; no DenkMaps integration. See [API behavior](docs/CORE-PERFORMANCE.md) and [results/limits](docs/CHANGES-0.4.3.md).

## 0.4.2 — Leaner core and MCP response schemas

Share immutable vocabulary, identifiers and normalized duplicate text internally. Avoid temporary owned token maps during normal insertion. Lazily reuse complete TF-IDF document norms while corpus statistics remain unchanged; invalidate on lexical changes and fall back to exact computation on cache contention/poisoning. Add cancellation checks inside cold norm calculations. Every MCP tool advertises a self-contained JSON Schema 2020-12 response schema for success and structured errors. Add exact old/new ranking/cache tests, adversarial cache cases, real-document schema validation and measured memory/latency comparisons. Public signatures, knowledge formats, MCP payload version one and PDF backend 0.3.5 remain. No DenkMaps integration. See [API behavior](docs/CORE-PERFORMANCE.md) and [executed results/limits](docs/CHANGES-0.4.2.md).

## 0.4.1 — Optional bounded MCP in the same crate

Added `mcp`, the `airust-mcp` stdio executable and an immutable public `McpServer` API. Five read-only tools expose lexical search, exact revision-bound passage reads, matching, shared-term suggestions and status. Added client-requested progress, cancellation, absolute time budgets, bounded workers/frames/results and connection flood/ID guards. Controlled matching/relations check expensive edit-distance work. Added raw-protocol, attack-oriented and executable tests, retained-knowledge checks and detailed [MCP](docs/MCP.md) / [security](docs/MCP-SECURITY.md) guides. Existing APIs, knowledge and PDF backend 0.3.5 remain. No DenkMaps integration. See [results and limits](docs/CHANGES-0.4.1.md).

## 0.4.0 — Document revisions and reliable text positions

Added optional `documents`: source fingerprints, immutable text snapshots, explicit physical page maps and revision-bound UTF-8 passages. Added bounded checked JSON, deterministic version-one identities, exact entry/retrieval projection and existing-provenance composition. Source/text/recipe/page changes invalidate old positions; old knowledge remains untouched. Core signatures and PDF backend 0.3.5 remain. See [API and upgrade](docs/DOCUMENTS.md) and [validation with limits](docs/CHANGES-0.4.0.md). No DenkMaps/MCP integration.


## 0.3.15 — Compact PDF page-tree validation

Reduced temporary pending-work memory for wide PDF trees without changing structural checks or public APIs. Added a deterministic 128-tree nested-count property regression, three official large-manual checks (one retained control), independent extraction/search probes and reproducible isolated memory measurements. All previous knowledge is retained. This is the final planned 0.3.x stability release; further patches require concrete defects. See [executed tests, benchmarks and limits](docs/CHANGES-0.3.15.md) and [API/upgrade](docs/PDF-PAGE-TREE-MEMORY.md). No DenkMaps/MCP integration.


## 0.3.14

Independently validate the reachable PDF page tree before extraction. Compare every present root/subtree count with actual leaves, reject duplicate/cyclic or malformed structures, bound validation work and enforce configured page limits on actual leaves. Require the parser count to agree. Preserve old output on direct/supervised failure. Public APIs/formats and backend 0.3.5 remain unchanged. See [results](docs/CHANGES-0.3.14.md) and [API/upgrade/limits](docs/PDF-PAGE-COUNTS.md). No DenkMaps/MCP integration.

## 0.3.13

Reject covered native Form depth/processing exhaustion and incomplete fallback page counts. Restore local Form state before propagating errors and avoid caching failed span results as successful extraction. Preserve existing public APIs/formats and earlier knowledge. Pin optional backend 0.3.5, with maintained provenance and four new backend regressions; add public library/direct-worker output-preservation checks. See [results](docs/CHANGES-0.3.13.md) and [API/upgrade/limits](docs/PDF-STRUCTURE-LIMITS.md). No DenkMaps/MCP integration.

## 0.3.12

Reuse immutable entry IDs in word lists and normalized duplicate groups, including cache reconstruction. Reduce repeated ID allocations without changing public entry types, ordering, query/matching results, clone isolation or JSON/cache formats. Validate real-knowledge and synthetic resource costs separately. Source data and all earlier exports remain retained. See [results](docs/CHANGES-0.3.12.md) and [API/upgrade/limits](docs/SHARED-IDENTIFIERS.md). Backend stays 0.3.4; no DenkMaps/MCP integration.

## 0.3.11

Defer character-distance explanations until token-overlap winners are selected. Preserve complete result IDs, ordering, scores and explanations while avoiding expensive distances for discarded candidates. Add full-comparison regressions and real-knowledge stability/benchmark evidence. Existing PDF behavior, backend 0.3.4 and retained knowledge remain. See [results](docs/CHANGES-0.3.11.md), [API and limits](docs/TOKEN-MATCHING.md) and [0.3.x gates](docs/STABILITY-0.3.md). No DenkMaps/MCP integration.

## 0.3.10

Expand the two missing st presentation ligatures and combine the initial NFC/Latin-ligature/control-cleaning stage into one pass. Avoid repeated whole-text replacement copies while preserving later composition and existing controls. Public methods/JSON fields remain compatible; corrected text can change derived chunks and indexes. Further official large PDFs, an independent oracle, malformed inputs and separate helper/PDF measurements support the change. Knowledge and evidence remain retained. See [results](docs/CHANGES-0.3.10.md) and [API/upgrade/limits](docs/PDF-CLEANING-PASS.md). Optional backend stays 0.3.4. No DenkMaps/MCP integration.

## 0.3.9

Track trimmed boundaries as PDF chunk windows advance, skip empty windows without temporary strings and validate chunk/UTF-8 quotas before retaining text. Keep public APIs, successful knowledge formats, boundary choices and budget errors compatible. Further official large PDFs, generated Unicode windows, malformed inputs and explicit synthetic/real cost comparisons support the change. Knowledge and raw evidence remain retained. See [results](docs/CHANGES-0.3.9.md) and [API/controls/limits](docs/PDF-OVERLAP.md). The optional backend stays 0.3.4. No DenkMaps/MCP integration.

## 0.3.8

Preserve original separators and indentation during PDF sentence/paragraph segmentation instead of trimming and inserting whitespace. Long-page knowledge chunks become contiguous excerpts of the extracted/cleaned source; outer trimming and chunk budgets remain. Corrected imports can change chunk IDs and ranking, so older knowledge is retained separately. Further official large PDFs, authored format cases and damaged inputs extend validation. See [results](docs/CHANGES-0.3.8.md) and [API/upgrade/limits](docs/PDF-CHUNKS.md). Optional backend remains 0.3.4. No DenkMaps/MCP integration.

## 0.3.7

Limit PDF previews to the requested usable prefix and use that API for the verbose CLI. Avoid duplicate full extraction and preserve early chunk-budget failures. Whole-file/page budgets remain enforced; a preview is not validation of later pages. Successful corpus imports, rankings and JSON remain identical to 0.3.6. More official large PDFs, difficult formats and damaged inputs are checked; knowledge and raw evidence remain local. See [results](docs/CHANGES-0.3.7.md) and [API/commands/limits](docs/PDF-PREVIEW.md). The optional backend remains 0.3.4. No DenkMaps/MCP integration.

## 0.3.6

Correct PDF text cleaning: keep digit boundaries and blank paragraphs, join chained wraps without losing match boundaries, normalize Unicode after transformations and retain internal numeric lines. Existing API/JSON formats remain compatible; corrected text can change chunks, offsets and rankings. The expanded 21-document corpus, independent cleanup oracle, fixed literal probes, difficult formats and malformed inputs validate the changes. Old knowledge and raw evidence remain local. See [results](docs/CHANGES-0.3.6.md) and [API/controls/upgrade/limits](docs/PDF-TEXT-FIDELITY.md). The optional backend stays 0.3.4. No DenkMaps/MCP integration is added.

## 0.3.5

PDF conversion now checks chunk count and overlap-expanded text budgets while visiting each page, avoiding needless later-page extraction after a failure. Validated chunks are staged once and the parser is released before training text is duplicated. Public signatures, successful JSON and the optional backend version remain unchanged. Additional official large documents and authored format tests extend validation; original knowledge, fresh imports and raw evidence remain local. See [release results](docs/CHANGES-0.3.5.md) and [API/behavior/limits](docs/PDF-IMPORT-PIPELINE.md). No DenkMaps/MCP integration is added.

## 0.3.4

Optimize optional PDF object loading with a bounded complete-nonstream-object fast path on long physical lines. Preserve source bytes, complete structure traversal, decryption, caches and original fallback. Fix UTF-8 predecessor slicing and default C-symbol collisions in the independently named optional backend. Five matched CLI runs reduce the known 494-page case from 23.36 to 0.58 seconds in the median; fifteen-document raw text/knowledge/rankings remain unchanged. See [API, results and limits](docs/PDF-PARSER.md) and [validation](docs/CHANGES-0.3.4.md). No DenkMaps/MCP integration or default-core dependency change.

## 0.3.3

Add optional supervised PDF conversion with time budgets, bounded diagnostics, bounded accepted JSON and parent-controlled atomic publication. The real slow-case deadline preserves prior output; all fifteen successful corpus imports retain checked text and provenance. Stateful real-PDF index soak results, frozen release sources and reusable knowledge remain retained. See [commands and limits](docs/PDF-WORKERS.md) and [validation](docs/CHANGES-0.3.3.md). Library/ranking APIs remain unchanged; no DenkMaps or MCP integration.

## 0.3.2

Add opt-in PDF budgets for file bytes, pages, raw/cleaned/chunk text and total chunks. Reject chunk amplification before retaining an oversized collection, handle OS-string CLI paths without argument panics, and escape controls in the CLI's own terminal output. Exclude retained lab data from the Docker context. All fifteen corpus imports match 0.3.1; native-call latency and hard-memory isolation remain open. See [API and limits](docs/PDF-LIMITS.md) and [validation](docs/CHANGES-0.3.2.md).

## 0.3.1

Harden optional PDF imports: exclusive atomic output publication, refusal of empty conversions, parser-panic containment, linear parenthesis tracking and explicit package isolation for retained local lab data. Real-document comparisons, retained private knowledge and adversarial regression cases support the change. See [results and limits](docs/CHANGES-0.3.1.md).

## 0.3.0

Stability release: consistent validation for direct model deserialization, a single bounded disk-state read, compatibility fixtures from 0.2.30, and a manifest-driven isolated-feature matrix. Existing algorithms and data-format versions are retained. See [results and limits](docs/CHANGES-0.3.0.md) and [migration](docs/MIGRATION-0.3.0.md).

## 0.2.30

a dedicated disk-backed full-text index with atomic revisions, scoped retrieval, bounded original-record export and integrity checks. See [report](docs/CHANGES-0.2.30.md).

## 0.2.29

trusted feedback curation, versioned datasets, grouped training and explicit quality-gated model promotion with rollback. See [report](docs/CHANGES-0.2.29.md).

## 0.2.28

per-document score explanations using the actual learned features and feedback adjustments. See [report](docs/CHANGES-0.2.28.md).

## 0.2.27

verification of captured proof dependencies against current facts, rules, source versions and allowed scope. See [report](docs/CHANGES-0.2.27.md).

## 0.2.26

comparisons of identified retrieval runs, visible per-query regressions and explicit quality/resource criteria. See [report](docs/CHANGES-0.2.26.md).

## 0.2.25

Evidence-backed rule proposals. an optional bounded rule interpreter that proposes conclusions with explicit premises, sources, revision, conflicts and incomplete-result status. See [report](docs/CHANGES-0.2.25.md).

## 0.2.24

Inspectable trainable ranking. an optional small linear model with replaceable bounded features, explicit training, grouped holdout evaluation and versioned artifacts. See [report](docs/CHANGES-0.2.24.md).

## 0.2.23

Explicit feedback learning. a bounded local feedback ledger and inspectable smoothed ranking adjustments. See [report](docs/CHANGES-0.2.23.md).

## 0.2.22

Optional adaptive retrieval: bounded explicit backend registration, replaceable query-shape policy, frozen analyzer/route planning and bounded decision traces. Original recipes/scopes survive routing; failures do not cause hidden retries. No model auto-loading. See [report](docs/CHANGES-0.2.22.md).

## 0.2.21

Incremental lexical/positional updates: metadata-only term/token reuse, clonable text indices, source-only projection reuse and changed-record reanalysis with fresh original coordinates. Diagnostic work receipts explain rebuilt/reused data. Serialized metadata representation is preserved exactly. See [report](docs/CHANGES-0.2.21.md).

## 0.2.20

Optional shared-index storage: immutable analyzed records, posting maps and normalized duplicate bindings are shared across clones and detached on mutation. Default storage, APIs and wire formats remain unchanged. Fixed before/after workloads expose both copy savings and single-owner tradeoffs. See [report](docs/CHANGES-0.2.20.md).

## 0.2.19

Reproducible guarded stress workloads: two-million-record lexical scale, retained-copy memory baseline and concurrent snapshot/source consistency with four readers/two writers. Seeded update and concurrent-reader regressions preserve existing APIs/defaults. See [report](docs/CHANGES-0.2.19.md).

## 0.2.18

Optional transport-neutral progress observers: actual phase/count events, known/unknown totals, sequenced delivery and callback diagnostics. Panics are isolated, busy/reentrant notifications are counted and dropped, and publication callbacks run outside locks. See [report](docs/CHANGES-0.2.18.md).

## 0.2.17

Optional monotonic time budgets on the shared execution controls. Earlier parent deadlines survive composition; late results remain typed deadline failures and never become provider fallbacks or partial publication. See [report](docs/CHANGES-0.2.17.md).

## 0.2.16

Optional execution control: shared one-way cancellation, controlled lexical/phrase/semantic/hybrid search, reranking and compact retrieval, plus cancelled dataset staging/publication without partial commits. Provider interruption remains cooperative. See [report](docs/CHANGES-0.2.16.md).

## 0.2.15

Optional retrieval evidence policies: explicit sufficient-by-policy, insufficient and uncertain decisions; per-excerpt coverage; caller-calibrated recipe-bound score gates; JSON-budget-aware reports. Defaults remain unchanged. See [report](docs/CHANGES-0.2.15.md).

## 0.2.14

Optional coherent dataset snapshots: one revision for search, vectors and source references, off-lock preparation, conflict-checked publication, full reconciliation, exact-text vector reuse and explicit model rebuilding. Optional positional search joins the same boundary. See [report](docs/CHANGES-0.2.14.md).


## 0.2.13

Optional query-related excerpts over existing retrieval results: original UTF-8 byte spans, grapheme-safe bounded highlights, phrase/term selection, analyzer forwarding and explicit fallback reasons. Prefix defaults and old public struct literals remain compatible. See [report](docs/CHANGES-0.2.13.md).


## 0.2.12

Optional Knowledge Retrieval API: interchangeable lexical/text/semantic/hybrid sources, compact owned excerpts with original ranking evidence, explicit reranking/fallbacks and exact complete-JSON budgets. Existing specialized APIs/defaults remain unchanged. See [report](docs/CHANGES-0.2.12.md).


## 0.2.11

Optional Storage Backend API: memory and standalone SQLite implementations, atomic revision-checked batches, bounded pages, consistent snapshot loading and explicit identity renewal. No default-feature or application integration changes. See [report](docs/CHANGES-0.2.11.md).


## 0.2.10

- Add explicitly selected Unicode word analysis, optional German Snowball stemming and required quoted token phrases.
- Reuse shared Engine postings/BM25/TF-IDF/top-K without changing default analysis or duplicating source records.
- Preserve original UTF-8 evidence across normalization/compatibility expansions; bound input/query/occurrence work and stage updates before mutation.
- Test ordering, overlaps, malformed queries, scope-before-top-K, Unicode spans, stem limitations and incremental/rebuilt agreement.
- Add executable recipes, measured scale checks, compatibility validation and English API guidance. No DenkMaps/MCP integration.

## 0.2.9

- Add optional provider-neutral candidate reranking with bounded batches/bytes, pre-inference scope checks, stable ties and preserved retrieval evidence.
- Add an explicitly loaded, size/SHA-256-verified multilingual MiniLMv2 cross-encoder; no automatic model/runtime downloads or new default dependencies.
- Fix local E5/reranker coexistence by sharing one ONNX environment. Reject a different runtime path instead of silently mixing libraries.
- Reject malformed scores/recipes and partial-result mixtures; opt-in original-order fallback cannot bypass a configured score threshold.
- Compare actual native reranking against the unchanged 0.2.8 fixture, retaining per-query regressions, no-answer failures and added CPU/RAM/latency.
- Extend API guides, executable examples, compatibility checks and CI. No DenkMaps or MCP integration.

## 0.2.8

- Add independent retrieval-quality metrics with graded judgments, explicit unjudged policy and defined-denominator summaries.
- Preserve old fixtures and add public literal/paraphrase/distractor/no-answer reference cases plus context/source/access contracts.
- Add reproducible scale/resource harnesses and a compiled generic knowledge pipeline.
- Document all new APIs, ownership choices and a proposed shadow-tested DenkMaps upgrade without integrating it.
- Update CI checkout to verified v7.0.1, pinned by commit, with Node 24 and read-only credentials policy.


## 0.2.7

- Add independent revision heads and deletion tombstones with explicit expected-version writes.
- Keep historical snapshots opt-in, bounded globally, and separate from application durability.
- Compare available states and restore them into new revisions without rewinding the sequence.
- Validate versioned JSON and standalone records, including sequence exhaustion and malformed histories.


## 0.2.6

- Add optional HNSW over shared semantic-vector buffers with configurable construction/search widths.
- Preserve exact final cosine scores, access-filter fallback, small-index fallback and stable result ordering.
- Track changed/deleted IDs with an exact live overlay and explicit topology compaction.
- Compare recall, build/query time and memory against exact search; retain existing exact APIs.


## 0.2.5

- Add optional versioned derived-index caches with source, analysis and model validation.
- Stream private atomic snapshots; preserve source ownership on every returned load error.
- Reuse lexical analysis and optional vector bits without inference; retain portable source JSON.
- Add corruption, malformed-format, crash and repeated-restart regression checks plus measured startup costs.


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
