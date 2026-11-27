# AIRust API cookbook

Start with the operation your application needs and enable its smallest feature set. This guide covers AIRust 0.6.2. Generic record APIs and saved formats remain compatible. The complete reference workflow is [a local Markdown search application](REFERENCE-APPLICATION.md).

## Choose an entry point

| Application task | API | Minimum features |
| --- | --- | --- |
| Search or compare application records | `Engine`, `KnowledgeEntry`, `SearchOptions`, `MatchOptions` | None |
| Return owned excerpts and bounded JSON | `Retriever`, `LexicalSource`, `RetrievalOptions` | `retrieval` |
| Retain document revision and byte coordinates | `DocumentSnapshot`, `Passage` | `documents` |
| Serve a selected collection to an agent | `McpServer`, `McpLimits` or `airust-mcp` | `mcp` |
| Match quoted phrases or German word forms | `TextIndex`, `AnalyzerConfig`, `TextSource` | `advanced-search`; add `retrieval` for `TextSource` |
| Use application-owned vectors | `EmbeddingProvider`, `SemanticIndex` | `embeddings` |
| Combine lexical and semantic ranks | `HybridSource`, `HybridSearchOptions` | `hybrid`; add `retrieval` for `HybridSource` |
| Rescore a selected candidate prefix | `Reranker`, `RerankOptions` | `reranking` |
| Publish consistent search, vector and source views | `DatasetManager` | `snapshots` |

The [support matrix](SUPPORT.md) covers every declared feature, including storage, graph, learning and compatibility adapters. Provider-neutral APIs require Rust 1.89; optional local ONNX adapters require Rust 1.92 and explicitly supplied files.

## Index general records

Assign stable application IDs and supply original text, with optional JSON metadata. `upsert` validates before changing an entry. `remove` returns the removed record. `Engine::from_entries` rejects duplicate IDs. `to_json` and `from_json` retain generic original records and rebuild derived search state.

`search` returns borrowed entries with scores and matching terms. Hold the index/read guard while using these borrows. Equal scores use ascending IDs. Empty matches are an empty list. Choose BM25 or TF-IDF through `SearchOptions`; scores are not probabilities. [Executable core example](../examples/core.rs), [core behavior](CORE-PERFORMANCE.md).

Historical `KnowledgeBase` stores `input`/`output` training examples. It is a compatibility adapter, not the generic Engine entry format. In AIRust 0.6.1, `KnowledgeBase::load_bounded(path,max_bytes)` checks file bytes and regular-file input before loading; read-only snapshots privately share examples and copy on mutation. [Resource ownership](RESOURCE-LIMITS.md) explains first-write costs and parsed-memory limits. The historical `TfidfAgent` uses BM25. New integrations should name the actual algorithm they select.

## Apply the application's scope before retrieval

Attach collection/project metadata to original entries and pass a `Context` to search or retrieval. Source and request scopes intersect before top-K. An empty Context ID set means unrestricted; an application whose authorization result is empty must stop before calling search.

```rust
use airust::Context;
let scope = Context::new().with_metadata("project", "manuals");
```

A Context filters supplied data. It is not a user-account or authorization service. The calling application owns that decision. [Scope and retrieval contract](RETRIEVAL.md).

## Deliver a small owned response

Enable `retrieval` and wrap a borrowed Engine with `LexicalSource`. The response owns selected IDs, excerpts and evidence so it can outlive the source read guard.

```rust
use airust::{Engine, KnowledgeEntry, retrieval::{LexicalSource, RetrievalOptions, Retriever}};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let index = Engine::from_entries([KnowledgeEntry::new("a", "Rust protects memory.")])?;
    let options = RetrievalOptions::new(6)
        .with_candidates(100)?
        .with_byte_budgets(4096, 12000, 2000, 65536)?;
    let reply = Retriever::new(LexicalSource::new(&index)).query("Rust memory", &options)?;
    println!("{}", reply.to_json());
    Ok(())
}
```

The budgets above cover query bytes, total excerpt bytes, bytes per excerpt and the complete JSON response. Escaping, IDs, metadata and evidence count toward the final output cap. Insufficient space removes tail hits or returns a budget error if even the empty envelope cannot fit. A transport adding its own wrapper must bound that complete wrapper too.

Use `with_relevant_excerpts(ExcerptOptions::default())` for query-related windows. `with_paragraph_boundaries()` prefers complete blocks; `with_distinct_evidence()` avoids returning identical delivered text repeatedly. Original UTF-8 entry-relative spans remain available. [Excerpt rules](EXCERPTS.md), [evidence selection](EVIDENCE-SELECTION.md), [checked citation example](../examples/checked_evidence.rs).

## Bind a result to an exact document revision

Enable `documents`. A `SourceFingerprint` identifies supplied original bytes. `DocumentSnapshot::new` preserves unpaged text; `from_pages` retains explicitly supplied physical page numbers. Include extraction and cleanup settings in the recipe and choose `Supplied`, `Extracted` or `Cleaned` honestly.

1. Keep one immutable snapshot for the exact text revision.
2. Create a checked `passage(start_byte, end_byte)` and call `to_entry`.
3. Preserve its `airust_document` metadata while adding application scope metadata.
4. Validate the original entry, then project a retrieval excerpt with `passage.subspan(snapshot, hit.span.start, hit.span.end)`.
5. Resolve against the same revision before displaying or serving the citation.

All ranges use zero-based UTF-8 bytes with an exclusive end. They are not character positions, PDF file offsets or graphical coordinates. Restoring a passage from an old or different revision fails. The [complete workspace example](REFERENCE-APPLICATION.md) demonstrates import, persistence, retrieval and MCP. [Document contract](DOCUMENTS.md), [provenance](PROVENANCE.md).

## Verify quotations after generation

If another component generates an answer, retain the exact RetrievalResponse used as its context. `CitationRegistry` checks bounded IDs, exact quotations and optional ranges against that response. Missing, overlapping, malformed and unsupported references remain explicit. A matching quotation does not prove semantic support or source truth. [Citation contract](CITATIONS.md).

The reference workspace itself generates no answer. Empty delivery reports no evidence; nonempty delivery leaves answerability unassessed. Optional `EvidencePolicy` evaluates stated coverage/score conditions and can report uncertainty. Model scores need separately calibrated gates. [Assessment contract](ASSESSMENT.md).

## Expose the collection through MCP

Enable `mcp` to include document, retrieval and execution support. `McpServer::new(entries, snapshots, limits)` validates the complete collection and exact source bindings. `from_entries` serves generic records; `from_documents` indexes supplied snapshots. `serve(reader, writer)` uses the calling application's Tokio runtime. An external Rust application using Tokio APIs declares Tokio directly.

The standalone executable accepts one knowledge file or one or more snapshot files at startup. Its five read-only tools and schema-shaped responses have explicit byte, request, concurrency and timeout budgets. Clients refresh version-qualified schema IDs when the crate version changes. All loaded documents are readable by the connected client. [MCP examples and schemas](MCP.md), [trust boundaries](MCP-SECURITY.md).

## Add ranking controls deliberately

| Need | Existing control | Evaluation boundary |
| --- | --- | --- |
| Remove question words and expand technical identifiers | `LexicalSource::with_query_analysis(LexicalQueryOptions::default())` | Inspect added/removed terms; literal Engine behavior remains unchanged. |
| Balance BM25 score and distinct prepared-term coverage | `LexicalCoverageSource` | It reranks only the retrieved candidates and its normalized scale needs its own thresholds. |
| Require phrases or compare German word forms | `TextIndex` / `TextSource` | German stemming does not translate English source text or provide full lemmatization. |
| Compare meaning, including across languages | `SemanticIndex` with an `EmbeddingProvider` | Provider recipe/dimensions must match; quality depends on the actual model and corpus. |
| Fuse lexical and semantic candidates | `HybridSource` / `hybrid_search` | Component ranks/scores remain visible; candidates below both cutoffs cannot appear. |
| Score actual first-stage candidates again | `Retriever::with_reranker` | Preserve first-stage evidence and measure per-query losses and added inference cost. |

Provider-neutral embeddings and reranking add no inference model. Real local adapters use explicitly supplied, verified ONNX assets and runtime paths. Approximate HNSW retrieval is separately selected and needs recall measurements; exact retrieval remains available. [Question preparation](LEXICAL-QUERY.md), [coverage](LEXICAL-COVERAGE.md), [semantic retrieval](SEMANTIC.md), [hybrid](HYBRID.md), [reranking](RERANKING.md), [ANN](ANN.md).

The [reference evaluation](measurements/reference-v1/README.md) compares fixed configurations on a frozen dataset with development-only selection, reserved evaluation and recorded resource costs. Its known reference paragraphs are not exhaustive human judgments. [Quality metrics and gates](QUALITY-GATES.md).

## Coordinate updates and concurrent readers

Use `DatasetManager` when search, optional vectors and source bindings must publish together. Acquire one immutable `DatasetSnapshot` per request. Prepare bounded changes outside the publication lock and publish only if the complete expected revision still matches. Old readers retain the previous complete view.

Supply the entire desired source-reference list on each Put, including metadata-only changes. Changed text requires matching replacement vectors in semantic mode. Application database transactions, file acquisition and authorization remain application-owned. [Snapshots](SNAPSHOTS.md), [incremental analysis](INCREMENTAL.md), [shared index payloads](SHARED-INDEX.md).

## Choose persistence by what it owns

| API | Owned state | Application responsibility |
| --- | --- | --- |
| Engine JSON | Portable original records | Durable publication and synchronization with the application's store. |
| `StorageBackend` / SQLite adapter | Original records, atomic batches and fenced revisions | Database lifecycle, source authorization and index refresh. |
| `IndexCache` | Rebuildable lexical data and optional model-specific vectors | Supply original records, protect the cache directory and rebuild on miss/error. |
| `DiskIndex` | Its own SQLite records and FTS index | Its own database format and scoring recipe; coordinate external stores explicitly. |

Cache checksums detect accidental damage; a caller able to rewrite payload and checksum can supply new internally consistent derived state. Disk scores use native SQLite FTS5 and are not interchangeable with Core BM25. [Storage](STORAGE.md), [cache](CACHE.md), [disk index](DISK-INDEX.md).

## Bound work and handle errors

Enable `execution` and attach a `WorkControl` to supported options. Cancellation and monotonic deadlines are typed execution errors, not successful empty searches or evidence decisions. Cancellation wins when both conditions apply. Cooperative checkpoints cannot forcibly stop a blocked provider, allocation or lock wait; a late result is rejected after control returns. [Execution and progress](EXECUTION.md).

The maintained PDF path has its own import budgets and supervised worker commands. Original knowledge is retained when validation or publication fails. [PDF limits](PDF-LIMITS.md), [workers](PDF-WORKERS.md), [parser maintenance](PDF-PARSER.md).

## Additional modules and compatibility

Explicit graphs store application relationships. Provenance identifies supplied source locations. Feedback, learned ranking and reasoning operate on explicit caller inputs; no automatic telemetry or fact publication is enabled. [Graph](GRAPH.md), [feedback](FEEDBACK.md), [learned ranking](LEARNED_RANKING.md), [reasoning](REASONING.md), [training pipeline](TRAINING-PIPELINE.md).

Release-specific details remain in [CHANGELOG.md](../CHANGELOG.md) and the immutable [0.5.2 cookbook](https://github.com/LEVOGNE/airust/blob/v0.5.2/docs/API-COOKBOOK.md). Existing callers upgrading from 0.4 use [the 0.5 migration](MIGRATION-0.5.0.md). For startup commands and compatibility tools, use [STARTUP.md](../STARTUP.md).


## Capture pilot cases and select a live profile

The [local pilot guide](PILOT.md) provides runnable initialization, capture, immutable review, export, independent verification and optional model configuration. Normal queries do not record cases. The format preserves explicit origin, original UTF-8 source coordinates and caller-declared review status; export never turns synthetic/replayed cases into independently reviewed traffic.

Use [CONFIGURATION.md](CONFIGURATION.md) to select the settings owned by each component. Existing typed constructors/builders already validate BM25, retrieval, model and work budgets; MCP has public `McpLimits::validate()`. Keep application policy separate from generic library limits. The 0.6.1 reference example adds `profile-check CONFIG` and explicit bounded reranking batch/candidate fields, reusing `RerankOptions`. Preflight validates configuration without checking asset/service availability. See [migration and compatibility](MIGRATION-0.6.1.md).

Applications can implement `airust::mcp::McpSearchBackend` and attach it with `McpServer::with_search_backend`. Return an owned bounded RetrievalResponse over the supplied Engine and honor its RetrievalOptions control/scope. The server revalidates registered source integrity before publication; the application owns model/network policy. See the reference profile implementation and [MCP contract](MCP.md).


The source also includes `bounded_mcp`, a model-free stdio example with tighter existing `McpLimits` and one admitted CPU worker. Optional native reranking may use an explicitly writable `model_snapshot_directory`; absence preserves memory-backed loading. This disk/memory tradeoff does not make a large native model suitable for a 1-GiB device. Follow [resource contracts](RESOURCE-LIMITS.md) and [actual measurements](measurements/profile-practical-0.6.0/README.md), including first requests, startup and all regressions.

## Repeated MiniLM unload/reload (0.6.2)

Reuse the existing `Reranker` and `rerank` calls with `ProcessReranker::multilingual_minilm_v2_from_snapshot`. Build an application-owned matching worker, supply explicit trusted assets and a private snapshot parent, then drop the scorer to close, stop and reap its native process. See the executable Rust example and full failure/timeout contract in [local model processes](LOCAL-MODEL-PROCESSES.md). The ordinary direct constructor remains compatible and retains its measured allocator limitation. Process isolation adds startup/IPC/disk costs and does not bound the active model to 1 GiB.
