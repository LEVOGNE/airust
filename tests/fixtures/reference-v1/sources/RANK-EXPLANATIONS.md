# Exact ranking explanations

Enable `rank-explanations`. It enables the lightweight learned-ranking and feedback interfaces, without loading any model or provider automatically. Existing scoring APIs remain unchanged.

`ExplainableReranker::score_explained(query, documents, options)` returns input positions, exact-text fingerprints, recipe, scores and additive ScoreComponent values. For LinearReranker each component is the actual feature value multiplied by its learned weight. Features are extracted once per document; explanations do not rerun a stateful provider to guess its earlier values. Component sums reproduce the score in the same order.

FeedbackReranker exposes two components: the base provider's aggregate score and the actual feedback adjustment. It does not invent explanations of an opaque base provider's internal features. Existing retrieval evidence retains matched terms, source recipes, original scores and positions separately. Join document positions/IDs while the application's authorized candidate mapping is available. Input positions start at one; this API explains scores in input order and performs no independent sorting.

ScoreComponent::new and ExplainedScores::from_components let custom providers supply validated additive explanations. The algebra is checked; custom feature meanings remain provider assertions. Names must be unique, bounded and control-free; all values/products/sums must be finite. An additive component is not a causal attribution, calibrated probability or proof of factual relevance.

ExplanationOptions defaults to 1,000 documents and 1 MiB JSON. Supported JSON caps are 1 KiB..16 MiB. Inputs cap query/document bytes at 16 KiB/64 KiB, combined document bytes at 16 MiB, and components at 64 per document. A too-small output budget returns an error; it never silently drops components to present an incomplete score explanation. Scoring may already have run when output size is checked. Provider failures propagate. Arbitrary provider allocation/blocking still requires its own constraints.

No original query/document text is retained in the explanation; fingerprints can still be guessed and are not anonymization. These direct scorer APIs do not enforce authorization. Supply scoped candidates before calling, and apply outer execution controls where needed. Explanations are not automatically added to RetrievalResponse JSON; budget any combined envelope yourself. No application integration or telemetry collection is performed.

## Complete example

```rust
use airust::{learned_ranking::*, rank_explanations::*};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = [
        TrainingExample::new("Rust ownership", "Rust ownership prevents errors", true)?,
        TrainingExample::new("Rust ownership", "unrelated archive", false)?,
    ];
    let mut features = LexicalFeatures;
    let model = train(&data, &mut features, &TrainingOptions::default())?;
    let mut scorer = LinearReranker::new(&model, &mut features)?;
    let result = scorer.score_explained(
        "Rust ownership",
        &["Rust ownership prevents errors", "unrelated archive"],
        &ExplanationOptions::default(),
    )?;
    for doc in result.documents() {
        assert_eq!(
            doc.score(),
            doc.components().iter().map(|c| c.contribution()).sum::<f64>()
        );
    }
    println!("{}", result.to_json()?);
    Ok(())
}
```
