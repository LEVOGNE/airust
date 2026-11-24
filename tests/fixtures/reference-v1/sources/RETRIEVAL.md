# Knowledge Retrieval API

Enable `retrieval` for a bounded, transport-neutral query/response API. It enables no server, database or model. The optional retrieval feature uses Unicode segmentation for relevant excerpts; default crate features remain empty. Existing specialized search APIs and default features are unchanged. Rust 1.89 is supported; optional native adapters retain their separate requirements.

```rust
use airust::{Engine, KnowledgeEntry, retrieval::{LexicalSource, Retriever, RetrievalOptions}};
let index = Engine::from_entries(vec![KnowledgeEntry::new("a", "Rust memory safety")])?;
let response = Retriever::new(LexicalSource::new(&index))
    .query("Rust", &RetrievalOptions::new(5))?;
assert_eq!(response.hits()[0].id, "a");
let wire = response.to_json();
assert_eq!(serde_json::from_str::<serde_json::Value>(wire)?["schema_version"], 1);
# Ok::<(), Box<dyn std::error::Error>>(())
```

The response owns only selected IDs, excerpts and evidence. It can outlive the source index/read guard. `to_json()` returns the already validated complete JSON string. Immutable accessors and Serialize produce the same JSON. Schema version 1 uses named snake_case ranking/status values; tolerate additive fields. Responses are not authoritative records, model answers or a persistence format for rebuilding indexes.

## Sources and composition

- `LexicalSource::new(&Engine)` uses existing BM25/TF-IDF. `with_options(SearchOptions)` retains algorithm, BM25 settings and base scope. Request budgets control final/candidate counts.
- `LexicalCoverageSource::new(&Engine)` explicitly rescales prepared lexical candidates by score and distinct-term coverage. [API, diagnostics and limits](LEXICAL-COVERAGE.md).
- With `advanced-search`, `TextSource` uses TextIndex and TextSearchOptions, including German analysis and required phrases. It preserves original UTF-8 phrase spans, flattened across required phrases, and a phrase-evidence truncation flag. Use the specialized API when grouped phrase terms/occurrences are needed. This does not combine a positional index with a separately owned semantic index automatically.
- With `embeddings`, `SemanticSource::new(&SemanticIndex, &mut provider)` uses existing exact cosine retrieval and its explicit threshold.
- With `hybrid`, `HybridSource` uses existing rank fusion. Its source options retain weights, RRF smoothing, algorithm, threshold, scope and explicit provider-failure policy. Component ranks/scores and any lexical fallback reason remain visible. Request candidate count bounds each component and final fused candidates; it does not bound all vectors/documents visited by exact search.
- Implement `RetrievalSource` for an application-owned index or another library. Return borrowed `RetrievalCandidate` entries with `RankingEvidence`, a nonempty recipe up to 1,024 bytes and an optional fallback reason up to 4,096 bytes. Enforce request scope before selecting/scoring candidates and cap returned candidates. The pipeline rejects excessive counts, duplicate IDs, blank records, non-finite scores, zero ranks, invalid phrase spans and oversized evidence. It rejects an out-of-scope custom result before any reranker sees it. These checks cannot bound work or allocations already performed inside a custom provider.

The effective scope is the intersection of source configuration and request Context, checked before top-K. Disjoint intersections return an empty batch without calling an embedding provider. As in existing APIs, an empty Context ID set is unrestricted. Applications must return no results for an empty authorized set before constructing that Context. Scopes are caller assertions, not an authentication system.

With `reranking`, call `Retriever::with_reranker(&mut scorer, RerankOptions)` before query. Source candidates are validated first. Reranking retains its own input/batch/byte/threshold/failure contract; the smaller configured return limits apply. Results retain source evidence and one-based source rank separately from optional rerank scores. Summary fields disclose actual selected/scored counts, recipe, use and fallback reason. Explicit KeepOriginalOrder fallback has no invented scores; default provider failures remain errors. No model is loaded or called implicitly without a supplied provider.

Graph traversal and source provenance remain explicit sidecars. Feed response hit IDs into ContextAssembler::assemble with the same authorized scope and optional ProvenanceStore to obtain sourced/neighbor context; retain this response's original scores separately. The compact response does not invent sources or infer causal relationships. Generic callers can return it from their own transport. The retrieval feature itself enables no server or endpoint; the separately optional [MCP feature](MCP.md) exposes existing operations in this same crate.

## Explicit evidence delivery and citation checking (0.4.5)

`with_distinct_evidence()` skips byte-identical delivered excerpts and fills slots from the existing candidate list without changing BM25 scores. The optional `selection` diagnostics count considered candidates, skipped duplicates and kept hits before final JSON-budget truncation. `without_distinct_evidence()` restores the unchanged default delivery. Empty text budgets and exhausted candidate sets remain explicit limits. See [evidence selection](EVIDENCE-SELECTION.md).

Combine it with `ExcerptOptions::with_paragraph_boundaries()` to prefer complete matching blocks, with the original window as a bounded fallback. After generation, `citations::CitedAnswer` parses bounded strict JSON and `CitationRegistry` checks IDs, exact quotations and original-entry byte ranges against the immutable delivered response. ID-only references never become quote proof, and matching quotes never become semantic truth verification. See the [citation contract and example](CITATIONS.md). No model is invoked by these checks.

## Byte budgets and truncation

Defaults: ten hits, one hundred candidates, 16 KiB query, 16 KiB total excerpt text, 2 KiB per excerpt, 64 KiB **complete encoded JSON**, 4,096-byte IDs and 64 KiB encoded evidence per candidate. `with_candidates`, `with_byte_budgets` and `with_evidence_limits` validate positive limits, except total text may be zero. Limit zero returns an empty envelope without source/model work, after validating the query. Blank/oversized queries fail before a source call.

By default excerpts remain deterministic original-text prefixes, clipped at UTF-8 code-point boundaries. In 0.2.13, opt into `with_relevant_excerpts(ExcerptOptions::default())` for query-related original windows and grapheme-safe highlights. See [EXCERPTS](EXCERPTS.md) for scoring, checked byte positions, analyzer forwarding, explicit fallback reasons and additional analysis budgets. Neither mode synthesizes a summary. Span start/end are zero-based original-text byte offsets; spans and excerpts agree exactly. Phrase evidence refers to the **full original record**, so a phrase can lie beyond the returned excerpt. Metadata is omitted by default. `with_metadata(max_bytes)` opts in; Missing, Included, Omitted and BudgetExceeded are distinct statuses. Encoded bytes including escaping, depth 64 and 100,000 JSON nodes are checked before cloning metadata. Over-budget metadata is omitted explicitly.

The output cap includes envelope, IDs, evidence, escaped text, metadata and rerank diagnostics. If it does not fit, remove lowest-priority tail hits until a complete JSON response fits and set `truncated=true`. If even the empty diagnostic envelope cannot fit, return a budget error; never return partial JSON. Query, ID and evidence-limit violations are errors rather than silent evidence corruption. A true truncation flag also reports fewer hits than received candidates, clipped text, budget-omitted metadata or omitted relevant-excerpt highlights. It is not a count of all relevant documents outside the candidate window.

Byte limits are not LLM-token counts, exact total heap limits, deadlines, cancellation or promises about relevance/recall. Exact search can still inspect many entries. Model tokenization and reranking pair truncation have separate budgets. Measure the complete serialized prompt with the receiving model. Raw BM25/TF-IDF/cosine/RRF/rerank scores are different ranking scales, not calibrated probabilities or reliable no-answer judgments.

## Freshness and application boundaries

Retriever borrows an immutable index through each source and does not mutate records. It does not synchronize a database, refresh vectors, authorize users or automatically retry stale work. A StoredSnapshot/LoadedIndex revision can be carried by the caller alongside the response; the current store may already have advanced. Recheck authoritative source/access revisions before publishing prepared context when required. Do model work outside long database locks and use bounded workers in async applications.

See the [API cookbook](API-COOKBOOK.md), executable `retrieval`, [release report](CHANGES-0.2.13.md) and proposed application upgrade guide. No DenkMaps or MCP integration is performed by this release.

Optional adaptive routing selects only explicitly registered sources, freezes analyzer selection during metadata-only preflight and preserves actual recipes/scopes. Source failures never trigger hidden retries. Route traces are separate diagnostics. See [routing contract](ADAPTIVE.md).


For explicit 0.4.6 lexical question preparation, configure `LexicalSource::with_query_analysis(LexicalQueryOptions::default())` and inspect `LexicalQueryOptions::analyze` independently. Matched terms and source scores describe that effective query. Literal sources remain unchanged. `ExcerptOptions::with_leading_context` preserves more surrounding text when it retains the relevant window’s matched terms and phrase positions. See [query preparation](LEXICAL-QUERY.md) and [excerpt selection](EXCERPTS.md).
