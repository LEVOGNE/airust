# AIRust 0.2.26

**A Modular Knowledge Intelligence Engine**

AIRust is a Rust library for finding information, comparing text, and suggesting possible connections. Give it entries with IDs, text, and optional metadata. Your application decides how to display and store the results.

Version 0.2.26 adds comparisons of identified retrieval runs, visible per-query regressions and explicit quality/resource criteria. See [API and limits](docs/QUALITY-GATES.md) and [validation report](docs/CHANGES-0.2.26.md). Extra capabilities remain optional. No DenkMaps or MCP integration is performed.

The [release history](CHANGELOG.md) explains every completed step through 0.2.26, with tests, measurements and limits. The [application guide](docs/APPLICATION-INTEGRATION.md) explains a later DenkMaps adapter and the real-data tests it still needs. No application dependency or data is changed here.

To use relevant excerpts, enable `retrieval` and pass `RetrievalOptions::new(10).with_relevant_excerpts(ExcerptOptions::default())`. Run `cargo run --features retrieval --example relevant_excerpts` for a complete example. A semantic result without literal word overlap reports that limitation rather than inventing a supporting passage.

To keep related data consistent, enable `snapshots` and acquire one `manager.snapshot()` per request. Use that view for search, vectors and source lookup. Run `cargo run --features snapshots,retrieval --example snapshots` for a complete example. These are RAM read views; an application still coordinates its own database transactions.

To assess evidence, enable `retrieval` and add `.with_evidence_policy(EvidencePolicy::default())`. Run `cargo run --features retrieval --example evidence_assessment`. The fixed twelve-case evaluation includes a false acceptance and a false rejection to show the limits of word coverage.

In the fixed 500,000-record/three-copy fixture, sharing reduces peak RAM by 65.3%. The first changed clone takes more time to detach its common posting data. See [measurements and tradeoffs](docs/CHANGES-0.2.20.md).

## What it does

| Operation | Result | Useful for |
| --- | --- | --- |
| `search()` | Relevant entries ranked with BM25. TF-IDF is also available. | Finding notes and document passages. |
| `find_matches()` | Exact matches, spelling similarity, or shared words. Rust also accepts `r#match()`. | Reviewing duplicates and similar content. |
| `relate()` | Suggested connections with the shared terms that explain them. | Helping users review possible links. |
| `SemanticIndex::search()` | Exact cosine ranking with an application-supplied embedding provider. | Finding related meanings, including across languages when the model supports them. |
| `SemanticIndex::hybrid_search()` | Combine lexical and semantic rank positions with visible evidence and optional fallback. | Queries mixing exact terminology and paraphrases. |
| `KnowledgeGraph` | Store explicit relationships, inspect neighbors, traverse within budgets, and detect cycles. | Application-owned hierarchies, dependencies, and references. |
| `ContextAssembler::query()` | Assemble ranked excerpts and optional graph neighbors with source evidence and budgets. | Preparing focused, inspectable context for an application or agent. |
| `reranking::rerank()` | Rescore a bounded candidate set while preserving its original retrieval evidence. | Improving the order of possible answers with an evaluated scorer. |
| `WorkControl` | Optional cancellation, monotonic deadlines and real phase/count notifications. | Keeping supported long-running work observable and interruptible. |
| `DatasetManager` | One immutable revision for search, optional vectors and sources; checked publication of prepared changes. | Keeping concurrent readers and updates consistent. |
| `Retriever::query()` | Compact owned excerpts, original ranking evidence and bounded JSON through interchangeable sources. | Feeding application transports without copying the whole corpus. |
| `AdaptiveSource` | Choose among sources you explicitly register and show the routing decision. | Keeping source selection visible and replaceable. |
| `FeedbackLedger` / `FeedbackReranker` | Record, correct or remove ratings and apply a visible preference adjustment. | Learning explicit preferences without automatic data collection. |
| `learned_ranking::train()` | Train a small model from labeled examples and inspect its features, weights and test results. | Testing learned ranking without building or loading an LLM. |
| `reasoning::infer()` | Apply explicit rules and return reviewable conclusions with proof chains. | Suggesting evidence-backed links without changing stored facts. |
| `StorageBackend` | Atomic original-record batches, revision fencing and bounded pages. | Sharing a storage contract without coupling to an application database. |
| `ProvenanceStore` | Share explicit source references and preserve them when entries are combined. | Showing where knowledge came from without inventing citations. |

The default `search()`, matching, and `relate()` methods compare words and characters. The optional semantic index uses model embeddings instead. Its quality depends on the provider and corpus. AIRust has no required model, cloud account, HTTP client, or LLM dependency.

`relate()` explains its suggestions with shared words. The optional rule engine explains its proposals with explicit premises and sources. Your application reviews these results before storing a hierarchy, prerequisite or contradiction.

## Try the source checkout

Use Rust 1.89 or newer for the core and provider-neutral embeddings. The optional local ONNX adapters need Rust 1.92. From the project directory:

```bash
cargo run --locked --example core
```

The example demonstrates search, text comparison, duplicates, relation suggestions, filters, and JSON round trips.

## Add AIRust to an application

For the current lexical library, add this to your `Cargo.toml`:

```toml
[dependencies]
airust = { version = "=0.2.26", default-features = false }
```

For local source development, use the checkout path instead:

```toml
[dependencies]
airust = { path = "../airust-master" }
```

A complete example:

```rust
use airust::{Engine, KnowledgeEntry};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut knowledge = Engine::new();
    knowledge.upsert(KnowledgeEntry::new(
        "note-1",
        "Rust provides memory safety without a garbage collector.",
    ))?;
    knowledge.upsert(KnowledgeEntry::new(
        "note-2",
        "Python is useful for scripting.",
    ))?;

    let results = knowledge.search("Rust memory", 5);
    assert_eq!(results[0].entry.id, "note-1");
    println!("{}", results[0].entry.text);

    let matches = knowledge.find_matches(
        "Rust provides memory safety without a garbage collector.",
        5,
    );
    assert_eq!(matches[0].score, 1.0);

    knowledge.upsert(KnowledgeEntry::new("note-2", "Python helps with automation."))?;
    knowledge.remove("note-2");

    let restored = Engine::from_json(&knowledge.to_json()?)?;
    assert_eq!(restored.len(), 1);
    Ok(())
}
```

Your application owns the IDs, storage model, and presentation. AIRust accepts the same entry format from document tools, graph applications, and agent services.

Search results contain the original entry, a relevance score, and matching terms. A query without a match returns an empty list. BM25 scores are relevance values, not probabilities. Equal scores use ascending entry IDs for a stable order.

## Optional semantic search

Enable `features = ["embeddings"]` for validated vectors, the provider interface, and `SemanticIndex`. This feature adds no dependencies. Supply an `EmbeddingProvider` or precomputed vectors. Each model space records a complete model recipe identifier and dimension; incompatible spaces are rejected.

```bash
cargo run --locked --features embeddings --example semantic
```

The example uses hand-written vectors to demonstrate the API. For real local inference, enable `local-embeddings` and supply the E5 model files and an ONNX Runtime shared library explicitly. Compiling the crate downloads neither model weights nor runtime binaries. See the [local-model guide](docs/LOCAL-EMBEDDINGS.md).

The fixed release smoke corpus contains twelve public records and sixteen German, English, French, and Spanish queries. Actual model results and their limits are recorded in the [0.2.0 change report](docs/CHANGES-0.2.0.md). The [embedding API guide](docs/SEMANTIC.md) explains updates, batches, filtering, and exact search costs.

Semantic retrieval scans all eligible vectors. The `hybrid` feature adds reciprocal rank fusion over the existing lexical and semantic results. Configure weights, candidate limits, thresholds, metadata scope, and explicit provider-error fallback through `HybridSearchOptions`. Results explain their component ranks and scores. See [HYBRID.md](docs/HYBRID.md).

```bash
cargo run --locked --features hybrid --example hybrid
```

Hybrid fusion can improve retrieval for some workloads and make it worse for others; the release report compares the fixed reference corpus without claiming universal improvement.

## Optional approximate vector retrieval

Enable `hnsw`, move an existing semantic index into `HnswIndex`, and choose `AnnSearchOptions` explicitly. The established semantic, hybrid and context APIs retain exact retrieval. Shared vector buffers avoid duplicating their contents. Configurable construction/search widths trade CPU and memory for recall; candidate scores use the existing cosine function.

```bash
cargo run --locked --features hnsw --example ann
```

Small indexes and explicit scopes use exact fallback. Changed entries are searched exactly in an overlay, stale graph points are excluded, and `rebuild` compacts them. Excessive pending changes switch to exact retrieval until compaction. The [ANN guide](docs/ANN.md) explains mode reporting, scopes, updates and measured recall limits. In the difficult 100k independent-random-vector test, width 100 recovered only 14.5% of exact top-ten neighbors on average; width 1000 recovered 68.5%. Serial topology construction took almost nine minutes. These synthetic results are reasons to measure the corpus and retain exact retrieval when needed, not production quality guarantees.

## Optional relationships and graph queries

Enable `graph` for `KnowledgeGraph`. Supply stable node IDs and explicit parent-child, dependency, reference, related, or custom relationships. Choose directed or undirected edges, inspect multiple relations between the same nodes, and preserve relation metadata. The graph stores IDs and relationships rather than a second copy of knowledge text.

```bash
cargo run --locked --features graph --example graph
```

Breadth-first and depth-first queries have depth and node budgets, relation filters, and optional allowed-ID sets. Use breadth-first traversal for shortest-hop neighborhoods. Mixed-graph cycle checks use union-find and iterative topological sorting; the long-chain regression catches recursive-stack failures. See the [graph API](docs/GRAPH.md) for exact semantics and costs.

Graph relationships come from the application. They do not establish truth, and lexical or semantic suggestions are not automatically stored as facts.

## Optional advanced text search

Enable `advanced-search` and construct a `TextIndex` with `AnalyzerConfig::default()` or `AnalyzerConfig::german()`. Quoted phrases require consecutive analyzed words in order. German stemming can match Haus/Häusern while original UTF-8 spans preserve what the document actually says. Select Unicode word boundaries or alphanumeric splitting, apostrophe policy and explicit budgets.

```bash
cargo run --locked --features advanced-search --example advanced_search
```

This is explicit positional/word-form search, not full German lemmatization or semantic understanding. Irregular verbs and compounds are not automatically resolved; stemming can merge different meanings. Existing Engine methods and caches retain their original analysis. See [TEXT-SEARCH.md](docs/TEXT-SEARCH.md).

## Optional intelligent reranking

Enable `reranking` for a provider-neutral second ranking step. Pass lexical, semantic or hybrid candidates to `reranking::rerank`. Choose candidate, batch and byte limits; apply a Context before scoring. The result preserves original candidates/ranks and adds a raw reranker score. Existing search methods keep their behavior.

```bash
cargo run --locked --features reranking --example reranking
```

That executable uses a handwritten API scorer. For actual model inference, enable `local-reranking` and explicitly supply the pinned multilingual MiniLMv2 model files and ONNX Runtime library. Builds do not download either. The [API contract](docs/RERANKING.md) and [local-model guide](docs/LOCAL-RERANKING.md) explain setup and compiled examples. Local E5 and reranker providers now share one runtime environment, fixing their previous second-environment initialization failure.

On the unchanged authored 30-record/36-query reference, reranking twenty hybrid candidates raised nDCG@3 from 0.598 to 0.914. Median added reranking time was 65.6 ms on the recorded host. Two E5 queries became worse despite a better average. The combined two-model evaluation peaked at 2721 MiB. These are smoke measurements, not production guarantees. See every query and cost in the [0.2.9 report](docs/CHANGES-0.2.9.md).

Reranking cannot recover a missing first-stage candidate, prove a statement true or guarantee a better answer. Raw logits are not probabilities. No default threshold detects unanswerable questions. It is optional and does not enable itself in existing applications.

## Optional context assembly

Enable `context-assembly` for `ContextAssembler` and `ContextOptions`. Borrow an existing lexical or semantic index, optionally attach its graph, and choose lexical, semantic, or hybrid search. Search seeds take priority over neighbors. The bundle preserves selected source IDs, metadata, ranking evidence, and a graph path when used.

```bash
cargo run --locked --features context-assembly --example context
```

Set limits for unique source IDs, total returned text bytes, and each excerpt. Duplicate normalized content shares text while retaining separate source references. Clipping preserves UTF-8 and original byte offsets. Metadata and serialization overhead are outside the text budget; applications measure their final prompt with the chosen tokenizer. Scope intersections fail closed, and excluded or missing entries cannot become graph bridges. See [CONTEXT.md](docs/CONTEXT.md) for the API, priority rules, and excerpt limits. No prompt format, LLM, or application integration is required.

## Optional source references

Enable `provenance` for `SourceReference`, validated UTF-8 byte/code positions, and `ProvenanceStore`. The store holds each reference definition once and associates its ID with any number of knowledge entries. It supports multiple or missing sources, conflict-checked merging, removal, pruning, and versioned JSON without changing the existing entry format.

```bash
cargo run --locked --features provenance --example provenance
cargo run --locked --features provenance,context-assembly --example provenance_context
```

When both features are enabled, attach `ContextAssembler::with_provenance` to retain each selected entry's references, including duplicate content with distinct sources. Missing and intentionally omitted sources are separate states. Source locations stay in their original coordinates. References do not prove truth or freshness; callers update bindings when text changes and verify exact source bytes/revisions when needed. See [PROVENANCE.md](docs/PROVENANCE.md).

## Optional derived-index cache

Enable `cache` for `IndexCache`. Save derived lexical analysis, or add `embeddings` separately to preserve model-specific vectors. Load requires the authoritative original entries. Source fingerprints include text and metadata; dataset identity, analysis and model recipes must also match. Failed loads leave the originals available for rebuilding without a corpus clone.

```bash
cargo run --locked --features cache --example cache
```

Files use atomic replacement, exact lengths, SHA-256 and explicit limits. Checksums detect damage, not a hostile writer with directory access. Loading reconstructs posting trees and validates source fingerprints, so a cache is not guaranteed faster for every workload. See [CACHE.md](docs/CACHE.md) for recovery, permissions and concurrency limits.

## Optional revisions and history

Enable `versioning` for `VersionedIndex`. Each write supplies an expected revision, so a stale caller receives a conflict. Deleted IDs retain a revision; recreating them needs that current value. Optional actor/reason/metadata describe a change without claiming authenticated audit evidence.

```bash
cargo run --locked --features versioning --example versioning
```

History defaults to zero retained snapshots. Opt into a global record limit when earlier content must remain available. Compare states, restore live or deleted states into new revisions, and export/import a validated versioned snapshot. Record limits are not byte limits; original application transactions, permissions and durable history remain caller-owned. See [VERSIONING.md](docs/VERSIONING.md).

## Measure retrieval quality

Enable `quality` for judgments, Precision@K, Recall@K, reciprocal rank and exponential-gain nDCG@K. Choose how to handle unjudged IDs explicitly. Duplicate ranked IDs are rejected. Missing positive judgments leave recall/nDCG undefined; summaries show how many queries have defined denominators.

```bash
cargo run --locked --features quality --example quality
cargo run --locked --features quality,hybrid --example retrieval_quality
```

The new authored reference has 30 public records and 36 queries, including literal identifiers, multilingual paraphrases, graded sources, distractors and no-answer cases. It is not an independent application benchmark or proof of logical understanding. Explicit local E5 runs and scale checks are documented in [BENCHMARKS.md](docs/BENCHMARKS.md). Raw samples, costs and unfavorable results remain visible. No token-saving percentage or production speed guarantee is inferred.

The assessment fixture reduces false acceptances from five to one compared with accepting any hit, while rejecting one paraphrased answer. These twelve authored cases do not establish production answer quality; see [results and limitations](docs/CHANGES-0.2.15.md).

Cancellation validation includes provider failures, concurrent signalling, preserved ranking and cancelled publication. See [the 0.2.16 report](docs/CHANGES-0.2.16.md).

The current release passes **189 default tests and 539 all-feature tests**, plus compiled examples and minimum-Rust checks. Historical native E5/cross-encoder measurements remain in the [0.2.9 report](docs/CHANGES-0.2.9.md). The unchanged 0.2.8 baseline reference has Recall@3 of 0.426 for BM25, 0.971 for exact E5 and 0.676 for the tested hybrids, over 34 positively judged queries. Reranking changes ordering, with the separate improvements and failures reported above. Both neural stages can still return unrelated candidates for no-answer queries. These results measure the fixture, not reliable logical understanding. Historical scale measurements remain in the [0.2.8 report](docs/CHANGES-0.2.8.md).


## Tested lexical baseline and limits

The [additional 0.1.9 validation report](docs/VALIDATION-0.1.9.md) records checks of the published crates.io package. Its complete default-feature suite passed again: **187 tests**, including the README example. The earlier release checks, including all optional features, are recorded separately in the [0.1.9 change report](docs/CHANGES-0.1.9.md).

| Check | Observed result |
| --- | --- |
| Real records | Indexed 722 exported entries from three local maps. Private text and databases are excluded from this repository. |
| Incremental changes | After 200 replacements or deletions, checked search scores, statistics, duplicate groups, fuzzy results, and relation proposals against a fresh index. |
| SQLite replay | A separate in-memory test database passed 300 steps with committed changes and rollbacks. The test caller updated the index. |
| German text | Equivalent Unicode umlauts matched exactly. The checked spelling-error example scored 0.92 character similarity. |

A synthetic scale check also measured frequent queries. Each size used short five-token records, a result limit of ten, and twelve warm samples per query on macOS ARM64 with 64 GiB memory and Rust 1.98.0:

| Entries | Rare query median (ms) | Common query median (ms) | Peak process memory (MiB) |
| ---: | ---: | ---: | ---: |
| 10,000 | 0.000604 | 2.737 | 25.62 |
| 100,000 | 0.001896 | 28.509 | 187.50 |
| 1,000,000 | 0.018708 | 313.901 | 1,914.19 |

The rare query matched about 1, 10, and 100 candidates; the common query matched every entry. These single-process results describe this workload. They do not establish application speedups, production capacity, or tail latency. The report includes raw samples and a runnable public test consumer.

Keep these limits in mind:

- Default search, matching, and relation suggestions are lexical. Optional semantic retrieval can find model-based similarities, but does not prove logical relationships. Opposing statements can share words and receive a `Related` suggestion.
- Normalization preserves accents. `Straße` and `Strasse`, or `Führerschein` and `Fuehrerschein`, are not exact duplicates. Search has no stemming and can return entries matching only some query terms.
- The core index is in memory. JSON loading rebuilds it. Applications own durable storage, committed-change replay, crash recovery, and permissions. AIRust does not automatically monitor SQLite.
- `Context` filters search. Matching, duplicate grouping, and relation suggestions require the caller to supply an appropriately scoped index or otherwise enforce the authorized record set.
- These checks do not establish semantic retrieval quality, token savings, or a completed production security audit. The real-corpus queries had no labeled relevance judgments.

During the 0.1.9 baseline validation, DenkMaps was inspected read-only and its exported records were tested in a separate consumer. No DenkMaps or MCP integration was implemented. A direct comparison with the application's existing search, including relevance and end-to-end performance, remains future work.

## Optional integrations

The default build is the library. PDF processing, HTTP, SQLite, bots, command-line programs, and embedded sample knowledge are enabled explicitly.

| Cargo feature | Enables |
| --- | --- |
| None | Knowledge index, BM25, TF-IDF, matching, duplicates, filters, and relation suggestions. |
| `embeddings` | Provider-neutral validated embeddings, atomic updates, and exact semantic retrieval. No added dependencies; Rust 1.89. |
| `hnsw` | Optional approximate semantic candidates with exact re-scoring, update overlay and visible exact fallbacks. Enables embeddings and hnsw_rs; Rust 1.89. |
| `hybrid` | Weighted reciprocal rank fusion, component evidence, filters, and explicit provider-error fallback. Enables `embeddings`; no added dependencies. |
| `graph` | Stable IDs, typed directed/undirected relationships, bounded queries, and cycle checks using petgraph. Independent of embeddings; Rust 1.89. |
| `context-assembly` | Borrowed ranked excerpts, graph evidence, deduplication, scope intersection, and text budgets. Enables `hybrid` and `graph`; Rust 1.89. |
| `provenance` | Shared document/page/text/file/code source references, checked merges and versioned export. No added dependencies; Rust 1.89. |
| `cache` | Validated atomic derived-analysis/vector snapshots using Postcard and SHA-256. Original records remain authoritative; Rust 1.89. |
| `versioning` | Revision heads/tombstones, expected-version conflicts, optional bounded history, comparison and restoration. No new dependency; Rust 1.89. |
| `quality` | Validated graded judgments, explicit unknown-result policy, retrieval metrics and macro summaries. No added dependencies; Rust 1.89. |
| `execution` | Cooperative cancellation, deadlines and phase/count observers; no new dependency. |
| `shared-index` | Optional sharing of immutable record, posting and duplicate buffers across cloned lexical indices; no new dependency. |
| `snapshots` | Coordinated immutable dataset views and conflict-checked publication; includes provider-neutral embeddings, provenance and record-validation contracts. |
| `feedback` | Explicit feedback learning; explicitly enabled. |
| `learned-ranking` | Inspectable trainable ranking; explicitly enabled. |
| `reasoning` | Evidence-backed rule proposals; explicitly enabled. |
| `quality-gates` | Comparable quality gates; optional. |
| `adaptive` | Explicit bounded source registry, replaceable routing policy and inspectable route traces; includes retrieval. |
| `retrieval` | Compact owned query responses and optional relevant excerpts; includes Unicode segmentation. |
| `storage` / `sqlite-storage` | Bounded atomic record backends with revision fencing; SQLite stays separate from web. |
| `advanced-search` | Explicit positional phrase search, configurable Unicode analysis and optional German Snowball stemming. Rust 1.89. |
| `reranking` | Bounded provider-neutral candidate rescoring with preserved original evidence. No added dependencies; Rust 1.89. |
| `local-reranking` | Explicit verified multilingual cross-encoder using FastEmbed and a shared dynamic ONNX Runtime environment. Rust 1.92. |
| `local-embeddings` | Explicit local E5 inference using FastEmbed and a separately supplied dynamic ONNX Runtime library. Rust 1.92. |
| `cli` | The existing command-line programs and embedded sample knowledge. |
| `pdf` | PDF text extraction and chunking. |
| `web` | The existing dashboard, HTTP, SQLite, bots, PDF, and CLI. |
| `bots` | A compatibility name for the existing web and bot integration. |
| `embedded-knowledge` | Sample data from `knowledge/train.json` at build time. |

The existing `MatchAgent`, `TfidfAgent`, `ContextAgent`, and `KnowledgeBase` adapters remain available. The historical `TfidfAgent` name uses the shared BM25 index. The actual TF-IDF search mode is `SearchAlgorithm::Tfidf`.

## Run the tools

Run a stored-answer query:

```bash
cargo run --locked --features cli --bin airust -- cli query tfidf "What is AIRust?"
```

Convert a sample PDF into training data:

```bash
cargo run --locked --features cli,pdf --bin pdf2kb -- data/pdf/sample.pdf /tmp/airust-sample.json
```

Start the optional dashboard:

```bash
cargo run --locked --features web --bin airust
```

The default dashboard address is `http://127.0.0.1:7070`. The console uses port 7071. Administration is disabled by default; network-facing operation requires a configured API token. Follow [STARTUP.md](STARTUP.md) and the [security guide](docs/SECURITY.md) for configuration.

## Project layout

| Location | Purpose |
| --- | --- |
| `src/execution.rs` | Optional execution controls shared across threads and supported operations. |
| `src/snapshots.rs` | Common revisions, prepared updates, vector reuse and atomic index/source publication. |
| `src/feedback.rs` | Optional local rating ledger and explicit scorer wrapper; no automatic telemetry. |
| `src/learned_ranking.rs` | Optional trainable linear ranker with replaceable features and inspectable artifacts. |
| `src/reasoning.rs` | Optional caller-defined rules and evidence-backed proposals; no automatic knowledge writes. |
| `src/quality_gates.rs` | comparisons of identified retrieval runs, visible per-query regressions and explicit quality/resource criteria. |
| `src/adaptive.rs` | Optional query-shape routing across explicitly configured sources, preserving scopes and recipes. |
| `src/retrieval/assessment.rs` | Optional evidence policies, explicit uncertainty and auditable per-excerpt observations. |
| `src/retrieval/excerpts.rs` | Bounded query-related windows, Unicode-safe highlights and original byte coordinates. |
| `src/retrieval.rs` | Transport-neutral source adapters, scope checks, optional reranking and exact JSON output budgets. |
| `src/storage.rs` | Optional record-backend contract, memory adapter and snapshot loading; `storage/sqlite.rs` supplies standalone SQLite. |
| `src/text_search.rs` | Optional configurable analysis, bounded required phrases and original UTF-8 evidence over shared ranking storage. |
| `src/text.rs` | Shared tokenization and Unicode normalization. |
| `src/matching.rs` | Character and token comparisons. |
| `src/search.rs` | Search modes, BM25 settings, and result types. |
| `src/context.rs` | Search filters for IDs and metadata. |
| `src/knowledge.rs` | Generic knowledge entries and the existing training collection. |
| `src/index.rs` | Shared storage and retrieval operations. |
| `src/semantic.rs`, `src/semantic/local.rs` | Optional semantic index, provider contract, and local E5 adapter. |
| `src/ann.rs` | Optional shared-vector HNSW topology, update overlay and explicit fallback policy. |
| `src/hybrid.rs` | Optional rank fusion over the shared lexical and semantic index. |
| `src/reranking.rs`, `src/reranking/local.rs` | Optional bounded candidate reranking and explicit local cross-encoder inference. |
| `src/graph.rs` | Optional ID-based graph, relationship metadata, bounded traversals, and cycle checks. |
| `src/context_engine.rs` | Optional structured context selection over the existing index and graph. |
| `src/cache.rs` | Optional versioned, bounded derived-index snapshots and validated restoration. |
| `src/versioning.rs` | Optional revision ownership, conflict checks and retained snapshots over the existing lexical engine. |
| `src/quality.rs` | Optional descriptive retrieval-quality metrics and query-summary contracts. |
| `src/provenance.rs` | Optional validated source locations and shared entry-to-source bindings. |
| `src/agent.rs`, `src/*_agent.rs` | Compatibility with the existing agent API. |
| `src/pdf_loader.rs`, `src/bin/`, `src/web/` | Import and user-interface integrations. |
| `examples/` | Runnable examples, a small runtime comparison, and a scalable index benchmark. |
| `scripts/measure.py` | Development-only CPU, memory and wall-time wrapper for timed executables. |
| `tests/` | Cross-module tests and regression cases. |
| `docs/` | API contracts, architecture, migration, change reports, and follow-up validation with reproducible checks. |
| `knowledge/`, `data/pdf/` | Sample knowledge and a PDF fixture. |
| `airust.db` | Existing local dashboard database, excluded from the crate package. |
| `backups/` | The preserved original archive, excluded from the crate package. |
| `build.rs` | Copies optional sample knowledge into the build directory. |
| `Dockerfile`, `.dockerignore` | Container configuration for the dashboard. |
| `test.sh`, `.github/workflows/ci.yml` | Repeatable validation. |

Start with the [API cookbook](docs/API-COOKBOOK.md) for practical recipes and ownership choices. The [API guide](docs/API.md) describes the public contracts. The [architecture guide](docs/ARCHITECTURE.md) explains how the modules work together. Source code is kept free of comments; explanations live in these documents and executable examples.

## Changes and compatibility

The [0.2.8 report](docs/CHANGES-0.2.8.md) and [migration guide](docs/MIGRATION-0.2.8.md) complete this release sequence with quality tools. Existing retrieval APIs, defaults and source/cache/history formats remain unchanged.

The [0.2.7 report](docs/CHANGES-0.2.7.md) and [migration guide](docs/MIGRATION-0.2.7.md) describe additive revision tracking. Existing source entries, retrieval and cache formats stay unchanged.

The [0.2.6 report](docs/CHANGES-0.2.6.md) and [migration guide](docs/MIGRATION-0.2.6.md) describe opt-in HNSW. Existing exact retrieval and default features remain unchanged.

The [0.2.5 report](docs/CHANGES-0.2.5.md) and [migration guide](docs/MIGRATION-0.2.5.md) describe the additive cache. Default features, original entries and existing retrieval APIs remain unchanged.

The [0.2.4 report](docs/CHANGES-0.2.4.md) and [migration guide](docs/MIGRATION-0.2.4.md) describe optional provenance. Source entries keep their original fields and JSON; references live in a separate caller-owned sidecar.

The [0.2.3 report](docs/CHANGES-0.2.3.md) and [migration guide](docs/MIGRATION-0.2.3.md) describe context assembly. Existing filters, graph, source formats, and retrieval APIs are preserved.

The [0.2.2 report](docs/CHANGES-0.2.2.md) and [migration guide](docs/MIGRATION-0.2.2.md) describe the additive graph feature. Existing source entries and lexical, semantic, and hybrid APIs are preserved.

The [0.2.1 report](docs/CHANGES-0.2.1.md) and [migration guide](docs/MIGRATION-0.2.1.md) describe the additive hybrid feature. Existing 0.2.0 APIs, embeddings, and source formats are preserved.

See the [0.2.0 change report](docs/CHANGES-0.2.0.md) and [migration guide](docs/MIGRATION-0.2.0.md) for optional embedding features. Existing lexical APIs and default features are preserved. Cargo dependencies on 0.1.x must explicitly select 0.2.0 to use the new release.

See the [0.1.9 change report](docs/CHANGES-0.1.9.md) and [0.1.9 migration guide](docs/MIGRATION-0.1.9.md) for the English-language and hardening work.

The [additional validation report](docs/VALIDATION-0.1.9.md) adds measurements and records remaining limits without changing the library version or introducing an application integration.

The [roadmap](ROADMAP.md) records implemented optional embeddings, hybrid retrieval, graph, context assembly and provenance. Quality evaluation, reranking, advanced text analysis and record storage are implemented. The compact Retrieval API is implemented in 0.2.12. External application integrations remain separate.

The published 0.1.8 baseline is described in the [0.1.8 change report](docs/CHANGES-0.1.8.md). The [original analysis](docs/ANALYSIS-0.1.7.md) records what existed in 0.1.7. [CHANGELOG.md](CHANGELOG.md) lists the versions.

## Validate a change

```bash
./test.sh
cargo doc --locked --no-deps --open
```

The script checks formatting, the default library, optional integrations, documentation examples, and Clippy. Validation results for a particular version belong in its change report.

## License

MIT. See [LICENSE](./LICENSE).

For a complete future application adapter, see the [application integration guide](docs/APPLICATION-INTEGRATION.md). It covers all new APIs, revision/permission boundaries, real progress, explicit learning and reviewed rule proposals. No application integration is performed.
