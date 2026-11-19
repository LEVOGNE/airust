# AIRust 0.3.0 application integration guide

For this stability release, read the [migration notes](MIGRATION-0.3.0.md), [support contract](STABILITY.md), and [executed validation report](CHANGES-0.3.0.md). No new application integration is performed.

AIRust finds candidate knowledge, compares text, assembles bounded evidence and offers optional ranking and rule evaluation. Your application owns users, permissions, durable data and accepted decisions. This document describes a future application adapter, including DenkMaps. This release changes no DenkMaps or MCP code and does not establish production readiness for a real application dataset.

## 1. Start with the smallest dependency

```toml
[dependencies]
airust = { version = "=0.3.0", default-features = false,
           features = ["retrieval", "advanced-search", "execution"] }
```

Use Engine for existing BM25 search/matching, or TextIndex with AnalyzerConfig::german() for optional German forms and analyzed phrases. Choose one owner for each projection. Enable snapshots when coordinated lexical/vector/source/text views are actually needed. Add shared-index only after measuring retained-reader/update costs. Nothing loads a model automatically.

Core Rust requirement is 1.89. Optional local inference adapters require 1.92 plus explicitly verified runtime/model assets. Select features deliberately instead of enabling all of them in production.

## 2. Keep durable data authoritative

Re-read the current application's tables and APIs before implementation. No current DenkMaps schema audit or performance comparison is performed by this release. Preserve the existing SQLite owner, stable IDs, transactions, revisions and rollback. Do not place SqliteStore into an unrelated application database; it owns a distinct AIRust schema.

Map an authorized application record to KnowledgeEntry::new(opaque_id, text).with_metadata(...). IDs must identify the same record across updates. Decide which title/note fields form the exact indexed text and retain that projection recipe. Use only needed metadata; permissions and sensitive source paths must not leak through results, duplicates, graph neighbors or diagnostics.

Read records and a committed revision in one consistent transaction. After an authorized database commit, apply only changed/deleted records to the derived index. A revision journal/outbox can support durable replay; AIRust does not create one for an external database. On missed revisions or failed derived updates, invalidate/rebuild from the authoritative database. Never return an index publication as proof that SQLite committed.

## 3. Publish consistent derived generations

With snapshots, DatasetManager owns immutable DatasetSnapshot views. Read a base snapshot/revision, create DatasetRecord values with complete SourceReference lists, prepare a DatasetUpdate and publish the prepared result against the expected revision. For full loading use prepare_replace. Keep expensive vector work outside long application locks. A concurrent publication conflict requires a fresh base and retry/reconciliation.

Acquire one snapshot per request, and borrow its lexical, semantic, positional and provenance views together. Old readers retain a coherent generation while a new generation is prepared. A source marker is a caller assertion, not a database watcher or distributed transaction. Recheck the authoritative revision/access rules before serving if application policy requires strict freshness.

Metadata-only edits reuse analysis; unchanged source-only edits can reuse the entire positional projection. Changed original text always refreshes offsets even when normalization produces equivalent tokens. PreparationStats reports actual reused/rebuilt work. A Put supplies the complete new source list: an omitted list clears the previous references. Persist source IDs/revisions in the authoritative application data, and rebuild their bindings when needed.

See [snapshots](SNAPSHOTS.md), [incremental updates](INCREMENTAL.md) and [memory tradeoffs](SHARED-INDEX.md).

## 4. Authorize before retrieving

Build an authorized index/view or intersect a validated allowed-ID scope before search and before reranking/model calls. An empty Context ID set is unrestricted; if authorization produces no permitted IDs, return an empty authorized result before querying. Do not translate denial into an unrestricted empty context.

Retriever accepts LexicalSource, TextSource, SemanticSource or HybridSource. RetrievalOptions selects a hit/candidate limit, scope, byte budgets, relevant excerpts, optional EvidencePolicy and WorkControl. Use an explicitly supplied reranker only when evaluated. The owned RetrievalResponse can leave the read/compute boundary after the snapshot guard is released.

Relevant excerpts retain UTF-8 byte offsets into the original indexed entry text. These are not character counts, DOM ranges, PDF offsets or coordinates into a separately transformed note. Slice the same original text version and verify the entry/source revision before highlighting or citing. Include the dataset/application revision in your outer response envelope.

EvidencePolicy detects whether retained hits meet explicit coverage/score rules. MeetsPolicy does not prove answer correctness. Insufficient means the requested policy lacks enough evidence; Undetermined means calibration/evidence is incomplete. Treat technical errors and cancellation as errors. A smaller final JSON budget can remove hits and change the assessment. Keep the complete serialized envelope, citations and transport framing inside your own final budget.

See [retrieval](RETRIEVAL.md), [excerpts](EXCERPTS.md) and [assessment](ASSESSMENT.md). File names and available examples are also indexed in [API cookbook](API-COOKBOOK.md).

## 5. Report real work and support cancellation

Create a CancellationToken and WorkControl per operation. with_timeout sets a monotonic deadline from construction; child controls cannot extend a parent deadline. Pass the same control through retrieval, dataset preparation, training or reasoning APIs that support it. Propagate typed cancellation/deadline errors to the application. Blocking providers and network calls still need provider-specific timeouts.

with_observer receives ProgressEvent values with phase, sequence, completed units and an optional known total. Emit a visible start message before dispatch. Map phases to plain status text such as “Searching permitted notes”, “Ranking candidates” or “Selecting evidence”. Show a per-phase percentage only when total is known and nonzero; otherwise use an activity indicator and actual completed count. Counts from different phases cannot be added into a trustworthy global percentage. Sequence numbers help concurrent consumers; callback arrival order can differ.

Keep observer work quick and nonblocking. Send status through a bounded application channel, coalesce repeated counts and handle disconnected subscribers. The engine isolates observer panics and exposes observer_failures; telemetry failure must not undo a completed publication. Do not report success until the operation's Result confirms it. If an external AI agent is still writing after retrieval, show that as a separate application phase; AIRust cannot predict its answer arrival time or force it to narrate. Put start/update/end communication rules in the agent's own instructions later.

See [execution](EXECUTION.md), [deadlines](EXECUTION.md) and [progress](EXECUTION.md). This release adds no MCP progress transport or UI loading bar.

## 6. Add optional routing, feedback and training separately

Enable adaptive and register already configured sources with names and SourceRole labels. The default policy selects from available sources using query shape; replace RoutingPolicy for a measured corpus-specific choice. Quotes require declared text capability. last_route explains the selection and actual backend recipe. Provider errors do not secretly retry another backend. Use views from one snapshot; registration alone cannot establish that arbitrary sources share a dataset. See [adaptive retrieval](ADAPTIVE.md).

Enable feedback for a local FeedbackLedger. Use authenticated, authorized, stable event IDs and explicit namespaces. Event IDs are global upsert keys within one ledger; namespace separates preference profiles, not event-ID ownership. Prefer separate tenant ledgers or globally prefixed event IDs, and authorize replacements/retractions against the existing event. Record Helpful/Unhelpful, update or retract the same ID and persist strict JSON. Feedback matches normalized query plus exact document text, not an immutable application record ID; identical texts share feedback within a namespace. Choose a namespace/version policy accordingly. FeedbackReranker wraps an explicit base scorer and adds an inspectable bounded preference bias. Calibrate weight to base-score scale. No ratings are collected or uploaded automatically. Hashes do not anonymize data. See [feedback](FEEDBACK.md).

Enable learned-ranking only for explicit curated training. Construct TrainingExample values from reviewed labels, group related/paraphrased questions, split_grouped before training and train only on the training partition. LexicalFeatures is a replaceable six-value extractor; a custom FeatureExtractor declares its exact schema/recipe. TrainingOptions bounds epochs/settings and can carry WorkControl. Inspect TrainingReport, held-out Evaluation, weights and regressions. Save LinearModel::to_json with external label provenance and authenticate artifacts before loading. LinearReranker returns logits, not answer probabilities. Neither feedback nor training changes source records. See [learned ranking](LEARNED_RANKING.md).

## 7. Review rule proposals before saving

Enable reasoning for explicit domain rules over signed subject/predicate/object statements. Construct Fact values with registered source IDs, safe Rule patterns and ReasoningOptions containing the application revision and allowed facts. infer returns proposals with immediate premises, transitive fact/source IDs and conflicts. Check complete, stop_reason and missing_support_count before describing the result as a complete closure. Inspect each proposal's evidence even when complete is true.

Rules and source registration are assertions supplied by the application; the engine does not establish truth from prose. Accepting a proposal is a separate authorized application transaction. Save its rule/source/premise revisions, and invalidate/review it when the premises change. Do not automatically turn text similarity into a prerequisite, contradiction or accepted edge. See [reasoning](REASONING.md).

## 8. Validate a real adapter before rollout

Freeze representative authorized queries and relevance judgments before tuning. Include German forms, quoted phrases, paths/IDs, duplicates, paraphrases, empty/no-answer cases and private maps. Compare the current application search against the candidate in shadow mode. Measure relevance per query and end-to-end latency/RAM, including model load, serialization and synchronization. Include regressions rather than reporting only the average improvement.

Exercise committed changes/deletes, rollback, restart/rebuild, stale prepared updates, concurrent publication, permission changes, cancelled/late providers, corrupted cache/model files and output caps. Verify that no private entry, intermediate fact, source path or feedback identifier crosses the boundary. Test actual map sizes and your application's configured limits. Public synthetic million-entry or authored quality fixtures do not establish production capacity or semantic quality for your data.

Roll out one capability at a time behind a configuration switch, with the previous search and dependency rollback tested. MCP/TOON, UI progress and a future DenkMaps adapter remain separate application work after these checks.

## New API reference by release

| Release | API | Purpose and contract |
| --- | --- | --- |
| 0.2.13 | ExcerptOptions, ExcerptSpan | Relevant original-text byte ranges |
| 0.2.14 | DatasetManager, DatasetSnapshot, DatasetUpdate | One coherent derived generation |
| 0.2.15 | EvidencePolicy, EvidenceVerdict | Explicit insufficient/undetermined evidence |
| 0.2.16 | CancellationToken, WorkControl | Cooperative interruption |
| 0.2.17 | WorkControl::with_timeout/with_deadline | Monotonic time budgets |
| 0.2.18 | ProgressObserver, ProgressEvent, WorkPhase | Actual phases and units |
| 0.2.19 | stress_index, stress_snapshots examples | Reproducible bounded stress workloads |
| 0.2.20 | shared-index feature | Optional retained-reader memory tradeoff |
| 0.2.21 | PreparationStats reuse counters | Incremental text/payload updates |
| 0.2.22 | AdaptiveSource, RoutingPolicy, RouteTrace | Explicit policy-selected sources |
| 0.2.23 | FeedbackLedger, FeedbackReranker | Retractable local preference evidence |
| 0.2.24 | FeatureExtractor, train, LinearModel, LinearReranker | Inspectable small trained ranking model |
| 0.2.25 | Fact, Rule, infer, ReasoningResult | Bounded evidence-backed rule proposals |
| 0.2.26 | CaseMeasurement, EvaluationRun, GatePolicy, compare_runs | Explicit comparable quality/resource checks |
| 0.2.27 | EvidenceView, EvidenceReceipt, capture_evidence, verify_evidence | Detect changed, missing or forbidden proof dependencies |
| 0.2.28 | ExplainableReranker, ExplainedScores | Exact additive score explanations |
| 0.2.29 | CurationPolicy, TrainingDataset, train_grouped, ModelRegistry | Reviewed training data and quality-gated model transitions |
| 0.2.30 | DiskIndex, DiskSource, DiskPage | Dedicated disk-backed search with revision-fenced updates/export |

The [API cookbook](API-COOKBOOK.md), focused module contracts and runnable examples contain exact signatures. [Release reports](../CHANGELOG.md) link actual tests, measurements and limitations for each version. This is an integration plan, not a completed DenkMaps integration or a claim of perfection.

## Comparable quality gates (0.2.26)

comparisons of identified retrieval runs, visible per-query regressions and explicit quality/resource criteria. Enable `quality-gates` explicitly. See [contract](QUALITY-GATES.md) and [release report](CHANGES-0.2.26.md).

## Evidence freshness (0.2.27)

verification of captured proof dependencies against current facts, rules, source versions and allowed scope. Enable `evidence-freshness` explicitly. See [contract](EVIDENCE-FRESHNESS.md) and [release report](CHANGES-0.2.27.md).

## Exact ranking explanations (0.2.28)

per-document score explanations using the actual learned features and feedback adjustments. Enable `rank-explanations` explicitly. See [contract](RANK-EXPLANATIONS.md) and [release report](CHANGES-0.2.28.md).

## Controlled training pipeline (0.2.29)

trusted feedback curation, versioned datasets, grouped training and explicit quality-gated model promotion with rollback. Enable `training-pipeline` explicitly. See [contract](TRAINING-PIPELINE.md) and [release report](CHANGES-0.2.29.md).

## Optional disk search index (0.2.30)

a dedicated disk-backed full-text index with atomic revisions, scoped retrieval, bounded original-record export and integrity checks. Enable `disk-index` explicitly. See [contract](DISK-INDEX.md) and [release report](CHANGES-0.2.30.md).

The disk adapter has its own coherent record/FTS revision. Existing DatasetManager RAM snapshots do not cover it automatically. Coordinate external source generations before combining disk hits with vectors/graphs/provenance. Apply explicit quality gates, recheck proof receipts, inspect explanations and review curated training data before model promotion. ModelRegistry is a caller-owned slot; persist/authenticate its state and keep source/database ownership in the application.
