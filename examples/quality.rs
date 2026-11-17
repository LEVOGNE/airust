use airust::quality::{evaluate_ranking, summarize, Judgments, UnjudgedPolicy};
use std::error::Error;
fn main() -> Result<(), Box<dyn Error>> {
    let labels = Judgments::new([("a", 3), ("b", 1), ("c", 0)])?;
    let result = evaluate_ranking(&["a", "c", "b"], &labels, 3, UnjudgedPolicy::RequireJudged)?;
    assert_eq!(result.recall_at_k(), Some(1.0));
    println!("{}", serde_json::to_string_pretty(&summarize(&[result])?)?);
    Ok(())
}
