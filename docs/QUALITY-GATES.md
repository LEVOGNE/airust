# Comparable quality gates

Enable `quality-gates`. Existing `quality` metrics remain available. This extension compares measured runs and reports Passed, Failed or Undetermined. It does not decide whether human relevance labels are correct.

```rust
# fn main()->Result<(),Box<dyn std::error::Error>> {
use airust::{quality::*, quality_gates::*};
let labels=Judgments::new([("answer",1)])?;
let case=CaseMeasurement::evaluate("q1","question",&["answer"],&labels,10,UnjudgedPolicy::NonRelevant,None)?;
let baseline=EvaluationRun::new("dataset-v1","0".repeat(64),"baseline",vec![case.clone()],None)?;
let candidate=EvaluationRun::new("dataset-v1","0".repeat(64),"candidate",vec![case],None)?;
assert_eq!(compare_runs(&baseline,&candidate,&GatePolicy::default())?.verdict(),GateVerdict::Passed);
# Ok(()) }
```

CaseMeasurement::evaluate binds an exact query, full sorted relevance judgments, cutoff and explicit unjudged policy to its identity. It retains the evaluated ranked IDs, hashes and metrics, not original queries/documents. EvaluationRun::new adds an identified corpus SHA-256, recipe, case set and optional caller-measured peak RSS. Cases are sorted by ID. Comparing mismatched corpora, queries, judgments, case IDs, cutoffs or policies is an error. Recipe changes are permitted because comparing methods is the purpose. Fingerprints identify supplied data, not authenticity.

GatePolicy::new selects Precision, Recall, ReciprocalRank or Ndcg. with_quality_limits configures an optional mean floor, allowed mean loss, per-query loss tolerance and allowed regressed-query count. Defaults use nDCG with zero allowed losses/regressions. Every regressed query is visible even when the mean improves. Regression findings are diagnostic observations; regression_count decides whether their configured count is acceptable. Floating comparisons tolerate 1e-12 numerical noise. These are descriptive checks, not statistical-significance or confidence claims.

with_resource_limits optionally sets a mean-latency ratio and absolute peak-RSS cap. Every case must have latency if that criterion is selected. Missing measurements or undefined relevance metrics yield Undetermined, never Passed. Any failed criterion makes the overall result Failed even if another criterion is undetermined. Zero baseline latency permits only zero candidate latency. Caller measurements must come from comparable hardware/configuration; AIRust cannot authenticate or infer missing measurements. RSS can include unrelated phases if measured that way; record scope.

Runs cap at 10,000 cases and 64 MiB retained ranked-ID bytes; cutoffs at 1,000; judgments at 100,000 and 16 MiB encoded ID bytes per case; ranked IDs at 4,096 bytes and queries at 16 KiB. Labels are bounded. Input structures are caller owned; these checks are not exact process-memory or wall-time quotas. Evaluation is synchronous. Judgments::iter adds read-only sorted access. Case identities include grade-zero judgments and the full evaluation policy.

Run `quality_gate` for a small self-contained check. `beir_evaluation DATASET_DIR` reads caller-supplied BEIR corpus.jsonl, queries.jsonl and qrels/test.tsv and compares BM25 against TF-IDF at K=10. It accepts no network instruction and downloads nothing. Files/rows/case counts are bounded. It emits case IDs, metrics and hashes without original corpus/query text. Third-party data is not bundled with AIRust. Independent SciFact and GermanQuAD runs, identities and limitations are recorded in the release report. No-answer, scope and malformed-input regression cases remain in the ordinary test suite; those authored cases do not become independent relevance judgments.

No model, database, application integration or automatic release policy is enabled. An application/CI must explicitly choose predeclared thresholds and treat Undetermined according to its own policy.
