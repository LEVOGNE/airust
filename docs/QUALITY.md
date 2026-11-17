# Optional retrieval-quality measurements

Enable `quality` for validated `Judgments`, explicit `UnjudgedPolicy`, `evaluate_ranking`, per-query `QueryMetrics` and macro `QualitySummary` through `summarize`. It adds no dependency, model or server. Supply ranked entry IDs and relevance labels; the engine cannot determine whether those human or authored labels are correct.

Grades are integers 0..31. Zero is judged nonrelevant; positive grades are relevant. Judgments reject repeated or empty IDs and out-of-range grades. A cutoff must be positive. Only the first K ranked IDs are evaluated and validated. Duplicates in that window are errors, so repeated results cannot inflate metrics.

Choose the unjudged policy explicitly: `RequireJudged` rejects an evaluated ID absent from the judgment set; `NonRelevant` scores it as zero and records the unjudged count. Incomplete judgments are not evidence that unknown records are actually irrelevant. Grade-zero records remain distinct from unjudged records.

Precision@K is the number of relevant retrieved IDs divided by K, including when fewer than K results were returned. Recall@K divides by all positively judged IDs. Reciprocal rank at K is the inverse one-based position of the first relevant result, or zero. nDCG@K uses gain `2^grade - 1`, discounted by `log2(rank + 1)`, divided by the ideal descending-grade DCG at the same cutoff. This explicit gain variant follows the TREC Web formulation. If there are no positive judgments, recall and nDCG are `None`, not perfect scores. Precision and reciprocal rank remain zero; returned/judged/unjudged counts preserve the context.

`summarize` requires a nonempty query set with matching cutoffs and policies. Every query has equal weight. Precision and reciprocal rank average all supplied queries. Recall and nDCG average only defined values and expose their respective query counts; all-undefined means stay `None`. No confidence interval, statistical-significance claim or production answer-quality guarantee follows from these descriptive numbers.

Primary metric references: [TREC precision/recall methodology](https://trec.nist.gov/pubs/trec30/papers/Overview-2021.pdf), [TREC exponential-gain nDCG](https://trec.nist.gov/pubs/trec20/papers/WEB.OVERVIEW.pdf), and [reciprocal rank](https://trec.nist.gov/presentations/TREC8/removed/tsld051.htm). This API implements the stated formulas; it does not claim every metric/configuration matches every trec_eval variant. No third-party source is copied.

## Minimal usage

```rust
use airust::quality::{evaluate_ranking, summarize, Judgments, UnjudgedPolicy};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let labels = Judgments::new([("a", 3), ("b", 1), ("c", 0)])?;
let result = evaluate_ranking(&["a", "c", "b"], &labels, 3,
    UnjudgedPolicy::RequireJudged)?;
assert_eq!(result.recall_at_k(), Some(1.0));
let summary = summarize(&[result])?;
assert_eq!(summary.queries(), 1);
# Ok(())
# }
```

`examples/retrieval_quality.rs` runs the separately versioned public reference corpus. Without explicit native model paths it evaluates BM25/TF-IDF; with `local-embeddings` and both paths it also evaluates exact E5 and two hybrid weights. Results retain all query IDs, grades, ranks, scores and descriptive metrics. The fixture's authored labels include difficult/no-answer cases and do not establish independent application relevance or logical reasoning. Earlier release fixtures remain unchanged.
