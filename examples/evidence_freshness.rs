use airust::{evidence_freshness::*, reasoning::*};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let facts = [
        Fact::new("ab", Statement::new("a", "edge", "b")?)?.with_sources(["s".into()])?,
        Fact::new("bc", Statement::new("b", "edge", "c")?)?.with_sources(["s".into()])?,
    ];
    let rules = [Rule::new(
        "transitive",
        vec![
            Pattern::new(Term::variable("a"), "edge", Term::variable("b"))?,
            Pattern::new(Term::variable("b"), "edge", Term::variable("c"))?,
        ],
        Pattern::new(Term::variable("a"), "edge", Term::variable("c"))?,
    )?];
    let result = infer(&facts, &rules, &ReasoningOptions::new("r1", ["s".into()])?)?;
    let sources = [SourceVersion::new("s", "version-1")?];
    let view = EvidenceView::new("r1", &facts, &rules, &sources)?;
    let options = FreshnessOptions::default();
    let receipt = capture_evidence(&result, &view, &options)?;
    assert!(verify_evidence(&receipt, &view, &options)?.all_current());
    let changed = [SourceVersion::new("s", "version-2")?];
    let report = verify_evidence(
        &receipt,
        &EvidenceView::new("r2", &facts, &rules, &changed)?,
        &options,
    )?;
    assert_eq!(report.proofs()[0].state, FreshnessState::Stale);
    println!("{}", report.to_json()?);
    Ok(())
}
