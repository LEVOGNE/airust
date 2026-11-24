# AIRust 0.5.2

**A Modular Knowledge Intelligence Engine**

AIRust is a Rust library for finding information, comparing text, and suggesting possible connections. Give it entries with IDs, text, and optional metadata. Your application decides how to display and store the results.

Earlier local comparisons with the same Qwen model found no general AIRust hybrid quality or token-saving advantage over SQLite FTS5 plus embeddings. The [complete comparison](docs/HONEST-COMPARISON-0.4.4.md) preserves the measurements and limits; these are historical results from 0.4.4.

Release priorities are **stability, performance, reliability and clear API/MCP contracts**. New capabilities stay optional and must justify their measured cost.

Version 0.5.2 is a verification patch on the 0.5 line. Runtime behavior matches 0.5.0. A coverage-guided libFuzzer campaign checked lexical search against an independent BM25 and TF-IDF formula, rebuilds, retrieval and text spans, document JSON, damaged caches and bounded PDF import, then kept the minimized inputs as versioned seeds. It found no panic, timeout or score mismatch. No new API, runtime dependency, default feature or DenkMaps integration is included. `sha2` is a dev-dependency for the seed manifest only. Start with [upgrade](docs/MIGRATION-0.5.2.md), [API cookbook](docs/API-COOKBOOK.md), [MCP](docs/MCP.md) and [what the fuzzer covered](docs/CHANGES-0.5.2.md).

Version 0.5.0 remains the stability release for the existing knowledge core and optional MCP. It keeps the runtime fixes through 0.4.25, checks repeated changes against independent records, and tests large synthetic indexes and retained copies. Cargo users on 0.4 still select the 0.5 line explicitly. Your application still owns its data and permissions.

| Release | What it adds | Why it helps |
| --- | --- | --- |
| [0.5.2](docs/CHANGES-0.5.2.md) | Coverage-guided fuzzing with versioned seeds. | Replay the minimized lexical and PDF inputs without changing calls. |
| [0.5.1](docs/CHANGES-0.5.1.md) | Fuzz-check the unchanged 0.5.0 runtime. | Confirm scores, rebuilds and bounded PDF import without changing calls. |
| [0.5.0](docs/CHANGES-0.5.0.md) | Verify Core/MCP stability and large-index behavior. | Keep contracts explicit and check state over repeated changes. |
| [0.4.25](docs/CHANGES-0.4.25.md) | Release temporary raw JSON before indexing. | Avoid keeping the large input buffer during MCP startup. |
| [0.4.24](docs/CHANGES-0.4.24.md) | Keep final MCP answer checks inside the worker. | Report late size errors correctly and keep admission until work ends. |
| [0.4.23](docs/CHANGES-0.4.23.md) | Avoid extra copies for short repeated-word Core queries. | Reuse the word and its count while keeping the same results. |
| [0.4.22](docs/CHANGES-0.4.22.md) | Prepare bounded MCP answers with less duplicate work. | Reuse a private data check and move JSON while keeping the final reply bound. |

Read the [complete history](CHANGELOG.md) and [completed route](docs/ROADMAP-TO-0.5.0.md).

`./test.sh` on Rust 1.98.0 (macOS ARM64) passed: rustfmt, 253 default tests, 911 all-feature checks including 39 doctests (one ignored child, `persistence::tests::crash_writer_child`), strict Clippy and documentation, the unchanged isolated feature and example commands, and the package-boundary check (3,859 files). Nightly 1.102.0-nightly (2026-11-13) Tree Borrows Miri with `-Zmiri-no-extra-rounding-error` passed 71 tests: 41 `index::` tests (`shared-index,execution`; four large index tests were not part of this finite set and still pass natively), 25 `tests/core.rs` tests (`shared-index`), 1 miri-reduced `stability_state` test, and 4 synchronous MCP `response_tests` (two async admission tests were not run under Miri). Rust 1.89, Rust 1.92, the 0.5.0 patch-level API comparison and the independent consumer were not repeated. The 0.5.0 state test still checks 250 native mutation steps; its Miri form uses eight steps and a reduced matrix without native threads/I/O. Existing protocol, bounds, cancellation, deadline and shutdown tests remain in `test.sh`. This is not a proof that no bug exists. Read [0.5.0 scope](docs/CHANGES-0.5.0.md) and [0.5.2 fuzz scope](docs/CHANGES-0.5.2.md).

**Measured 0.5.0 result:** all 30 finite scale/clone processes pass across exact 0.4.21, 0.4.25 and 0.5.0. The million-entry case checks search, 100 updates, 20 deletes and four readers; the 500,000-entry case preserves three retained clones when one changes. Independent mutation/cache tests and all retained RPC/startup/heap/source controls pass. Read [every timing, memory observation and limit](docs/CHANGES-0.5.0.md).

**0.5.2 limits:** runtime source is unchanged from 0.5.0, so the 0.5.0 timings above were not measured again. The campaign is coverage-guided libFuzzer on one macOS ARM64 host, with versioned seeds replayed by `cargo test`. It is not a proof that no defect remains. Saved inputs did not grow past the largest starter seed. MCP framing, embeddings, disk, graph, hybrid, reranking and ONNX were outside that harness and stay covered by the existing feature tests. A cache whose payload and checksum are both replaced is still trusted when its structure matches. Read [the exact fuzz scope](docs/CHANGES-0.5.2.md).

**0.5.0 limits:** Core/MCP runtime source is unchanged from 0.4.25. Small-MCP repeated median changes still range from -1.63% to +2.31% and -0.79% to +1.56%; ordinary canonical TF-IDF changes are +1.75% / +0.67%. All slower outcomes remain. Shared-host variation is not a new algorithmic speedup. Synthetic records do not prove real-document capacity or uptime. Finite tests, exposed questions and one host establish no zero-bug, semantic-quality, OS-quota or independent security-audit guarantee.

The [complete change history](CHANGELOG.md) preserves earlier performance and model comparisons. The [API cookbook](docs/API-COOKBOOK.md) explains optional documents, source positions, snapshots, evidence checks and other features. Raw benchmark JSON stays in the [tagged GitHub repository](https://github.com/LEVOGNE/airust/tree/v0.5.0/docs/measurements) and local lab; it is excluded from the registry source package. Read the [power metadata errata](docs/BENCHMARK-ERRATA.md) when comparing historical campaigns.

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
| `DocumentSnapshot` / `Passage` | Identify source/text revisions, validate UTF-8 ranges and project entry excerpts. | Keeping document citations attached to their exact text and optional physical page. |
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

## Optional MCP for agents

Install the executable from the same crate:

```bash
cargo install airust --version 0.5.2 --locked --no-default-features --features mcp --bin airust-mcp
airust-mcp --knowledge /absolute/path/knowledge.json
```

Use `--document /absolute/path/snapshot.json` for exact document revision/range citations. The five tools search, read a loaded passage, find matches, suggest relations and report status. They cannot write knowledge, run shell commands, fetch URLs or open client-supplied paths. All loaded data is readable by the connected client, so select the collection deliberately.

Every tool advertises an `outputSchema` for its version-one structured response, including errors. Clients can validate the shape while still checking `isError`, source bindings and revisions. A schema does not establish that source content is true or authorized.

The server sends progress when the client supplies a token. A loading indicator and spoken explanations depend on the host client/agent; server instructions ask the agent to explain its work but cannot force it. MCP alone adds no HTTP listener, database or model. Read [installation, schemas, examples and every limit](docs/MCP.md) and [attack coverage and remaining responsibilities](docs/MCP-SECURITY.md). These protections are tested; they are not a guarantee against every attack.

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
airust = { version = "=0.5.2", default-features = false }
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

The fixed authored reference has 30 public records and 36 queries, including literal identifiers, multilingual paraphrases, graded sources, distractors and no-answer cases. It is not an independent application benchmark or proof of logical understanding. Explicit local E5 runs and scale checks are documented in [BENCHMARKS.md](docs/BENCHMARKS.md). Raw samples, costs and unfavorable results remain visible. No token-saving percentage or production speed guarantee is inferred.

The assessment fixture reduces false acceptances from five to one compared with accepting any hit, while rejecting one paraphrased answer. These twelve authored cases do not establish production answer quality; see [results and limitations](docs/CHANGES-0.2.15.md).

Cancellation validation includes provider failures, concurrent signalling, preserved ranking and cancelled publication. See [the 0.2.16 report](docs/CHANGES-0.2.16.md).

The 0.3.0 stability baseline passed **197 default tests and 694 all-feature tests**, plus compiled examples, five Python tests, minimum-Rust checks and all 37 declared feature entries checked independently. Fresh native E5/cross-encoder runs, two independent lexical evaluations and stress tests through two million records are recorded with costs and limits in the [0.3.0 stability report](docs/CHANGES-0.3.0.md). Historical measurements remain in the [0.2.9 report](docs/CHANGES-0.2.9.md). The unchanged 0.2.8 baseline reference has Recall@3 of 0.426 for BM25, 0.971 for exact E5 and 0.676 for the tested hybrids, over 34 positively judged queries. Reranking changes ordering, with the separate improvements and failures reported above. Both neural stages can still return unrelated candidates for no-answer queries. These results measure the fixture, not reliable logical understanding. Historical scale measurements remain in the [0.2.8 report](docs/CHANGES-0.2.8.md).


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
| `evidence-freshness` | Evidence freshness; optional. |
| `rank-explanations` | Exact ranking explanations; optional. |
| `training-pipeline` | Controlled training pipeline; optional. |
| `disk-index` | Optional disk search index; optional. |
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
cargo run --locked --features cli,pdf --bin pdf2kb -- data/pdf/sample.pdf /tmp/airust-sample.json --timeout-seconds 60
```

Start the optional dashboard:

```bash
cargo run --locked --features web --bin airust
```

The default dashboard address is `http://127.0.0.1:7070`. The console uses port 7071. Administration is disabled by default; network-facing operation requires a configured API token. Follow [STARTUP.md](STARTUP.md) and the [security guide](docs/SECURITY.md) for configuration.

## Package contents

Raw benchmark JSON archives stay in the [tagged 0.5.0 GitHub tree](https://github.com/LEVOGNE/airust/tree/v0.5.0/docs/measurements) and local lab. Version 0.5.2 adds no new measurement JSON. Those archives are excluded from the crates.io source package. API guides, Markdown reports, method source, examples and required test fixtures remain included. Open measurement links in the tagged repository when reading an older report from an unpacked crate.

## Project layout

| Location | Purpose |
| --- | --- |
| `src/execution.rs` | Optional execution controls shared across threads and supported operations. |
| `src/snapshots.rs` | Common revisions, prepared updates, vector reuse and atomic index/source publication. |
| `src/feedback.rs` | Optional local rating ledger and explicit scorer wrapper; no automatic telemetry. |
| `src/learned_ranking.rs` | Optional trainable linear ranker with replaceable features and inspectable artifacts. |
| `src/reasoning.rs` | Optional caller-defined rules and evidence-backed proposals; no automatic knowledge writes. |
| `src/quality_gates.rs` | comparisons of identified retrieval runs, visible per-query regressions and explicit quality/resource criteria. |
| `src/evidence_freshness.rs` | verification of captured proof dependencies against current facts, rules, source versions and allowed scope. |
| `src/rank_explanations.rs` | per-document score explanations using the actual learned features and feedback adjustments. |
| `src/training_pipeline.rs` | trusted feedback curation, versioned datasets, grouped training and explicit quality-gated model promotion with rollback. |
| `src/disk_index.rs` | a dedicated disk-backed full-text index with atomic revisions, scoped retrieval, bounded original-record export and integrity checks. |
| `src/adaptive.rs` | Optional query-shape routing across explicitly configured sources, preserving scopes and recipes. |
| `src/retrieval/assessment.rs` | Optional evidence policies, explicit uncertainty and auditable per-excerpt observations. |
| `src/retrieval/excerpts.rs` | Bounded query-related windows, Unicode-safe highlights and original byte coordinates. |
| `src/retrieval.rs` | Transport-neutral source adapters, scope checks, optional reranking and exact JSON output budgets. |
| `src/storage.rs` | Optional record-backend contract, memory adapter and snapshot loading; `storage/sqlite.rs` supplies standalone SQLite. |
| `src/text_search.rs` | Optional configurable analysis, bounded required phrases and original UTF-8 evidence over shared ranking storage. |
| `src/text.rs` | Shared tokenization and Unicode normalization. |
| `src/matching.rs` | Character and token comparisons. |
| `src/mcp.rs`, `src/mcp/transport.rs`, `src/bin/airust-mcp.rs` | Optional read-only MCP, bounded transport/workers, progress, cancellation and the same-crate executable. |
| `src/documents.rs` | Immutable source/text revisions and checked UTF-8 passage positions. |
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
| `crates/airust-pdf-backend/` | Optional maintained parser backend with exact upstream provenance and separate regression checks. |
| `src/pdf_loader.rs`, `src/bin/`, `src/web/` | Import and user-interface integrations. |
| `src/pdf_page_tree.rs` | Private PDF-only page-tree/count validation before extraction. |
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

Benchmark power labels and their limits are corrected in [the measurement errata](docs/BENCHMARK-ERRATA.md). Raw timing samples remain unchanged.
