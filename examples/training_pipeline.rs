use airust::{feedback::*, learned_ranking::*, training_pipeline::*};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ledger = FeedbackLedger::default();
    for (id, doc, rating) in [
        ("yes", "q answer", Rating::Helpful),
        ("no", "unrelated", Rating::Unhelpful),
    ] {
        ledger.record(FeedbackEvent::new(id, "demo", "q", doc, rating)?)?;
    }
    let policy = CurationPolicy::new(["yes".into(), "no".into()])?.with_minimums(1, 1)?;
    let items = [
        TrainingItem::new("answer", "q", "q answer")?,
        TrainingItem::new("other", "q", "unrelated")?,
    ];
    let data = curate_feedback("demo", &ledger, &items, &policy)?;
    let restored = TrainingDataset::from_json(&data.to_json()?)?;
    assert_eq!(restored.fingerprint()?, data.fingerprint()?);
    let model = train(
        data.examples(),
        &mut LexicalFeatures,
        &TrainingOptions::default(),
    )?;
    println!(
        "{} examples; dataset {}; model {}",
        data.len(),
        data.fingerprint()?,
        model.fingerprint()?
    );
    Ok(())
}
