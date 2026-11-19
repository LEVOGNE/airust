use airust::{quality::*, quality_gates::*};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let labels = Judgments::new([("answer", 1), ("other", 0)])?;
    let case = CaseMeasurement::evaluate(
        "q1",
        "question",
        &["answer", "other"],
        &labels,
        2,
        UnjudgedPolicy::RequireJudged,
        None,
    )?;
    let baseline =
        EvaluationRun::new("demo", "0".repeat(64), "baseline", vec![case.clone()], None)?;
    let candidate = EvaluationRun::new("demo", "0".repeat(64), "candidate", vec![case], None)?;
    let report = compare_runs(&baseline, &candidate, &GatePolicy::default())?;
    assert_eq!(report.verdict(), GateVerdict::Passed);
    println!("{}", report.to_json()?);
    Ok(())
}
