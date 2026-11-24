# Assessing retrieval evidence

Enable `retrieval`. Assessment is off by default. It adds no model, network call, dependency or default feature. It judges the returned evidence against an explicit policy; it does not answer a question or prove source truth.

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use airust::{Engine, KnowledgeEntry, retrieval::*};
let index = Engine::from_entries([KnowledgeEntry::new("a", "Rust ownership prevents memory errors")])?;
let options = RetrievalOptions::new(3)
    .with_relevant_excerpts(ExcerptOptions::default())
    .with_evidence_policy(EvidencePolicy::default());
let response = Retriever::new(LexicalSource::new(&index)).query("Rust ownership", &options)?;
assert_eq!(response.assessment().unwrap().verdict(), EvidenceVerdict::MeetsPolicy);
for rank in response.assessment().unwrap().qualified_ranks() {
    let evidence = &response.hits()[rank - 1];
    assert!(evidence.excerpt.contains("ownership"));
}
# Ok(()) }
```

## Decisions

| Verdict | Meaning |
| --- | --- |
| `MeetsPolicy` | At least the configured number of delivered excerpts meets every configured criterion. |
| `Insufficient` | Too few delivered excerpts meet the policy, even counting undecided excerpts as possible successes. |
| `Undetermined` | Calibration, informative query terms or other necessary evidence is missing, or a provider fallback was used. |
| `NotRequested` | The requested result limit is zero. No source is called. |

The default policy requires all distinct informative normalized query terms in one nonblank delivered excerpt and at least one qualifying excerpt. It uses the existing small English/German stopword list; that list is a heuristic. Coverage is per excerpt, never a union of unrelated documents. Repeated query terms count once. Actual excerpt text is analyzed, rather than trusting a provider's claimed matched terms. Configured German analysis is forwarded by TextSource. Metadata is not assessed.

`EvidencePolicy::lexical(fraction)` sets a coverage threshold in (0,1]. Builders `with_min_hits`, `with_min_text_bytes`, `with_ignore_stopwords`, `with_label` and `with_limits` make the policy explicit. Labels identify caller policies; they do not automatically record the corpus or configuration. Keep those in your application evaluation manifest. `without_evidence_policy()` restores the previous response shape. Existing hit/evidence/batch struct literals remain compatible.

## Semantic scores and calibration

The default does not guess whether cosine, fused RRF or custom scores imply enough evidence. Such results are undecided. `with_nonlexical_coverage(true)` explicitly applies literal coverage to those results; that still does not measure semantic understanding.

For evaluated thresholds use `ScoreGate::new(ScoreScale::Retrieval(RankingKind::Cosine), recipe, minimum)` or `ScoreScale::Rerank`. `EvidencePolicy::scored(gate)` requires only the selected score and nonblank text. `scored_and_lexical(gate, fraction)` requires both score and literal coverage in the same excerpt. The exact recipe and score kind must match; a missing/mismatched recipe or rerank score remains undecided. Threshold equality passes. Numeric values must be finite and within the declared scale; malformed values are errors. No score is converted to a probability.

Calibration belongs to the complete model, corpus, query distribution and pipeline. Generic recipes such as BM25 do not fingerprint corpus statistics or every ranking parameter. Even an exact recipe match cannot detect all changes. Re-evaluate thresholds when those inputs change. A source or reranker fallback makes the overall decision Undetermined, including an empty fallback. A technical error remains an error; it is never silently converted into insufficient evidence.

## Output and resource bounds

The assessment records policy, criteria, counts, verdict and reasons. Each observation has a one-based returned rank, optional qualification, coverage and observed gated score. `qualified_ranks()` indexes `response.hits()[rank - 1]`. Reasons are diagnostic concerns across all delivered hits; a successful report may still mention another hit's low coverage. They are not a list of asserted reasons to reject every hit.

Assessment runs on actual delivered excerpts after text budgets. Prefix excerpts can miss a useful later passage; explicitly select relevant excerpts where appropriate. Complete JSON includes the report in the output byte budget. Removing tail hits recomputes the verdict over the retained prefix. If even the empty envelope cannot fit, retrieval returns a budget error. No assessment survives from discarded hits.

Default AssessmentLimits: query 16 KiB, 256 distinct terms, 100 requested hits, 64 KiB per excerpt, 256 KiB normalized analysis and 8192 tokens. Builders validate positive limits. Queries and requested counts are checked before source/provider work; bounded analysis errors do not become partial favorable decisions. Limits bound this layer, not arbitrary provider allocation or total process memory.

`assess_response(query, response, analyzer, policy)` assesses an existing response without changing it. Supply its original query and analyzer. A separately serialized report is outside the original response's JSON budget. Zero result limits intentionally skip query analysis.

## Evidence and limitations

Run `cargo run --features retrieval --example evidence_assessment` and the fixed, authored `assessment_quality` example. Its twelve cases compare the default policy with accepting any hit. It deliberately includes two failures: a glossary containing all keywords passes despite not answering, and a paraphrased answer fails literal coverage. These are documented counterexamples, not bugs hidden by the benchmark. See [release report](CHANGES-0.2.15.md) and [raw measurements](measurements/0.2.15/).

No result means no qualifying delivered result from the selected source, scope, limits and corpus. It does not establish global absence of knowledge. Word coverage cannot detect negation, factual correctness, completeness, contradictions or an answer's logical support. Applications should show uncertainty, ask for more evidence or use an independently evaluated semantic assessor where required.
