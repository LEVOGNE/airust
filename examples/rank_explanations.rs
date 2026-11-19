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
            doc.components()
                .iter()
                .map(|c| c.contribution())
                .sum::<f64>()
        );
    }
    println!("{}", result.to_json()?);
    Ok(())
}
