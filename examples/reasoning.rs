use airust::reasoning::*;
use std::time::Instant;
fn transitive() -> Result<Rule, ReasoningError> {
    Rule::new(
        "transitive",
        vec![
            Pattern::new(Term::variable("a"), "requires", Term::variable("b"))?,
            Pattern::new(Term::variable("b"), "requires", Term::variable("c"))?,
        ],
        Pattern::new(Term::variable("a"), "requires", Term::variable("c"))?,
    )
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut facts = Vec::new();
    for i in 0..12 {
        facts.push(
            Fact::new(
                format!("edge-{i}"),
                Statement::new(format!("step-{i}"), "requires", format!("step-{}", i + 1))?,
            )?
            .with_sources(["manual-source".into()])?,
        );
    }
    let start = Instant::now();
    let result = infer(
        &facts,
        &[transitive()?],
        &ReasoningOptions::new("demo-revision-1", ["manual-source".into()])?,
    )?;
    assert!(result.complete());
    assert!(result
        .proposals()
        .iter()
        .any(|p| p.statement.subject() == "step-0" && p.statement.object() == "step-12"));
    assert_eq!(facts.len(), 12);
    println!(
        "{}",
        serde_json::json!({"schema":1,"fixture":"explicit synthetic prerequisite chain, not inferred natural-language truth","elapsed_ms":start.elapsed().as_secs_f64()*1000.,"input_facts":facts.len(),"response_bytes":result.to_json()?.len(),"result":result})
    );
    Ok(())
}
