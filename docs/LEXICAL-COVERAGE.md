# Lexical evidence coverage

Enable `retrieval`. `LexicalCoverageSource` explicitly prepares a question and balances the existing lexical score with the fraction of its distinct terms found in each candidate. It loads no model, changes no index and leaves `LexicalSource` and MCP defaults unchanged.

```rust
use airust::{Engine, KnowledgeEntry, retrieval::*};
let index = Engine::from_entries([
    KnowledgeEntry::new("a", "ProvenanceStore retains the original source."),
    KnowledgeEntry::new("b", "Provenance Store revision checks."),
])?;
let question = "Which ProvenanceStore revision checks?";
let options = RetrievalOptions::new(2).with_candidates(20)?;
let source = LexicalCoverageSource::new(&index);
let trace = source.inspect(question, &options)?;
assert!(trace.analysis().unwrap().effective_query().contains("provenance store"));
for hit in trace.hits() {
    assert!((0.0..=1.0).contains(&hit.score()));
    assert!((0.0..=1.0).contains(&hit.query_coverage()));
}
let response = Retriever::new(source).query(question, &options)?;
assert!(response.recipe().starts_with("airust-lexical-coverage-v1:"));
# Ok::<(), Box<dyn std::error::Error>>(())
```

`LexicalCoverageOptions::default()` uses `LexicalQueryOptions::default()` and coverage weight 0.5. `with_coverage_weight(weight)` accepts finite values from zero to one inclusive. `with_query_options` changes the explicit preparation contract. Getters are `query_options`, `coverage_weight` and `recipe`. `LexicalCoverageSource::with_options(SearchOptions)` sets the lexical algorithm, BM25 parameters and base scope; `with_coverage_options` sets the preparation/scoring configuration. Request candidate count overrides `SearchOptions::limit`.

One prepared lexical search produces at most the request candidate count. For a candidate with lexical score s, maximum returned lexical score m and distinct-term fraction c, the new score is `(1-weight)*(s/m) + weight*c`. A zero maximum gives normalized score zero. The fraction counts unique effective query terms, including identifier parts, not repeated occurrences or their rarity. Query preparation already bounds and deduplicates these terms. Scores sort descending; exact ties use stable entry ID. This bounds retained candidates, not all postings visited during lexical search. Reranking cannot recover a record outside those candidates.

`inspect(question, &RetrievalOptions)` returns an immutable `LexicalCoverageBatch` with `hits`, optional `analysis` and `recipe` getters. Each borrowed `LexicalCoverageHit` exposes `entry`, final `score`, one-based `lexical_rank`, raw `lexical_score`, `normalized_score`, `matched_query_terms`, `query_terms`, `query_coverage` and original `matched_terms`. `analysis()` is `None` for disjoint scope, where analysis/search are skipped. No previous query is cached. Traces contain query/source information; apply your own access and logging policy.

The ordinary response uses existing `custom` ranking evidence for the new score, with prepared lexical rank/score and matched terms retained. `source_rank` is its position after coverage ordering; `evidence.lexical_rank` is its position before coverage ordering. Semantic fields remain `None`. Use `inspect` for the complete calculation. The recipe includes preparation, weight, algorithm and BM25 settings. It excludes scope, candidate count and index identity, so it is not a complete cache key. The final score is query-local and not a calibrated confidence or semantic evidence probability. Existing BM25 score thresholds must not be reused for it. The existing evidence-assessment policy still analyzes the original caller question; this scorer does not silently bypass that policy.

Scope intersection happens before analysis/search. Scopes are caller assertions, not authentication; reject an empty authorized ID list before constructing an unrestricted Context. Query byte/preparation limits apply. Under `execution`, cancellation/time budgets are checked before/after analysis, inside controlled search, while rescoring and after sorting. Cancellation returns an error, never a silently partial result. A sort/normalization/allocation cannot be forcibly interrupted. Original source text, excerpt byte budgets, UTF-8 spans, distinct-evidence selection and citation checks retain their contracts.

Weight zero retains prepared lexical score ordering except scores that round to an identical normalized value; exact ties always use ID. Weight one uses only term coverage. More query terms do not prove that a passage answers the question, and rare important words can still be missed. Query preparation, finite top-K and excerpt budgets can each remove useful evidence. No universal recall, rank preservation, semantic understanding or answer-accuracy improvement is promised. Measure your workload; literal BM25 remains the default. See [query preparation](LEXICAL-QUERY.md), [excerpts](EXCERPTS.md) and [citations](CITATIONS.md).
