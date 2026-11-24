# Explicit lexical query preparation

Enable the existing `retrieval` feature. `LexicalSource::with_query_analysis(LexicalQueryOptions::default())` explicitly prepares natural-language questions before the existing BM25 or TF-IDF search. Ordinary Engine and LexicalSource calls keep their previous behavior. No model, translation service, network, application adapter or new dependency is added.

Preparation removes a fixed English/German list of common function/question words, preserves negation and original technical identifiers, and adds parts of ASCII CamelCase, acronym and digit boundaries. `ProvenanceStore` keeps `provenancestore` and adds `provenance` and `store`; `HTTP2Server` adds `http`, `2` and `server`. Tokens inside ASCII double quotes/backticks and directly joined by `::` or `_` are protected from the word filter. An unmatched quote conservatively protects following terms. Quotation does not introduce phrase-search semantics into ordinary Engine search; use the existing advanced text-search source for required phrases. Non-ASCII identifiers retain whole Unicode-normalized tokens rather than guessing language-specific word boundaries.

Inspect `LexicalQueryOptions::analyze(query)` before use. `LexicalQueryAnalysis` exposes `effective_query`, `removed_terms`, `added_terms` and `original_fallback`. Terms are unique in first-seen order. If filtering would remove every original term, the original normalized terms remain. Punctuation-only input has no searchable terms. There is no cross-language meaning, stemming or inferred synonym expansion. An identifier can also be an ordinary word; measure the actual workload before enabling preparation.

`without_word_filter` and `without_identifier_parts` disable each step. `with_limits(input_bytes, effective_bytes, terms)` sets positive byte/count budgets, defaulting to 16 KiB input, 64 KiB normalized/effective text and 256 unique terms. Exceeding a limit returns a budget error instead of silently dropping terms. Limits bound format/analysis work, not exact process RAM or hard elapsed time. Source scope intersection and cooperative cancellation checks still apply. Preparation is computed independently for each retrieval; no previous query is retained in the source. Its explicit recipe describes flags and limits. Ranking evidence contains the actual prepared query's matched terms and BM25 scores, not scores for the unmodified input.

The existing evidence-assessment policy still evaluates the caller's original question. Query preparation does not silently redefine its coverage policy. If an application deliberately assesses prepared terms instead, pass the inspected effective query explicitly and retain the original question separately. Citation validation continues against the exact final delivered response.

```rust
use airust::{Engine, KnowledgeEntry, retrieval::{
    ExcerptOptions, LexicalQueryOptions, LexicalSource, RetrievalOptions, Retriever,
}};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let engine = Engine::from_entries([
    KnowledgeEntry::new("limits", "Provenance store import limits.\n\nThese limits do not establish source truth."),
])?;
let preparation = LexicalQueryOptions::default();
let trace = preparation.analyze("Welche ProvenanceStore limits?")?;
assert!(trace.added_terms().iter().any(|term| term == "provenance"));
let source = LexicalSource::new(&engine).with_query_analysis(preparation);
let options = RetrievalOptions::new(3)
    .with_distinct_evidence()
    .with_relevant_excerpts(ExcerptOptions::default().with_leading_context());
let result = Retriever::new(source).query("Welche ProvenanceStore limits?", &options)?;
assert_eq!(result.hits()[0].id, "limits");
assert!(result.hits()[0].excerpt.contains("do not establish source truth"));
# Ok(()) }
```
