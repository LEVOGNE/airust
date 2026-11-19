#![cfg(feature = "rank-explanations")]
use airust::{feedback::*, learned_ranking::*, rank_explanations::*, reranking::*};
fn model() -> LinearModel {
    train(
        &[
            TrainingExample::new("q", "q answer", true).unwrap(),
            TrainingExample::new("q", "other", false).unwrap(),
        ],
        &mut LexicalFeatures,
        &TrainingOptions::default(),
    )
    .unwrap()
}
#[test]
fn contributions_reproduce_real_scores_and_do_not_retain_raw_text() {
    let m = model();
    let mut f = LexicalFeatures;
    let mut s = LinearReranker::new(&m, &mut f).unwrap();
    let docs = ["q answer", "other"];
    let expected = s.score("q", &docs).unwrap();
    let explained = s
        .score_explained("q", &docs, &ExplanationOptions::default())
        .unwrap();
    assert_eq!(explained.scores(), expected);
    assert_eq!(explained.documents()[0].input_position(), 1);
    for doc in explained.documents() {
        assert_eq!(
            doc.score(),
            doc.components()
                .iter()
                .map(|c| c.value() * c.weight())
                .sum::<f64>()
        );
        assert_eq!(doc.components().len(), 6);
    }
    assert!(!explained.to_json().unwrap().contains("q answer"));
}
struct Counting {
    calls: usize,
    changed: bool,
}
impl FeatureExtractor for Counting {
    fn schema(&self) -> FeatureSchema {
        FeatureSchema::new(
            if self.changed { "two" } else { "one" },
            vec!["bias".into()],
        )
        .unwrap()
    }
    fn extract(&mut self, _: &str, _: &str) -> Result<Vec<f64>, LearningError> {
        self.calls += 1;
        Ok(vec![1.])
    }
}
#[test]
fn explained_scoring_extracts_each_document_exactly_once() {
    let mut f = Counting {
        calls: 0,
        changed: false,
    };
    let m = train(
        &[
            TrainingExample::new("q", "a", true).unwrap(),
            TrainingExample::new("q", "b", false).unwrap(),
        ],
        &mut f,
        &TrainingOptions::default(),
    )
    .unwrap();
    f.calls = 0;
    {
        let mut s = LinearReranker::new(&m, &mut f).unwrap();
        s.score_explained("q", &["a", "b", "c"], &ExplanationOptions::default())
            .unwrap();
    }
    assert_eq!(f.calls, 3);
}
struct Neutral;
impl Reranker for Neutral {
    fn recipe(&self) -> &str {
        "neutral"
    }
    fn score(&mut self, _: &str, docs: &[&str]) -> Result<Vec<f64>, RerankError> {
        Ok(vec![0.; docs.len()])
    }
}
#[test]
fn feedback_explains_base_and_adjustment_without_claiming_hidden_features() {
    let mut ledger = FeedbackLedger::default();
    ledger
        .record(FeedbackEvent::new("vote", "n", "q", "answer", Rating::Helpful).unwrap())
        .unwrap();
    let mut base = Neutral;
    let mut s = FeedbackReranker::new(&mut base, &ledger, "n", FeedbackOptions::default()).unwrap();
    let x = s
        .score_explained("q", &["answer"], &ExplanationOptions::default())
        .unwrap();
    assert_eq!(x.documents()[0].components()[0].name(), "base_score");
    assert_eq!(x.documents()[0].components()[1].contribution(), 1. / 3.);
    assert_eq!(x.scores(), s.score("q", &["answer"]).unwrap());
}
#[test]
fn duplicate_names_nonfinite_values_empty_and_oversize_inputs_fail() {
    assert!(ScoreComponent::new("bad", f64::NAN, 1.).is_err());
    let c = ScoreComponent::new("same", 1., 1.).unwrap();
    assert!(ExplainedScores::from_components(
        "r",
        "q",
        &["d"],
        vec![vec![c.clone(), c]],
        &ExplanationOptions::default()
    )
    .is_err());
    assert!(ExplanationOptions::new(0, 1024).is_err());
    let m = model();
    let mut f = LexicalFeatures;
    let mut s = LinearReranker::new(&m, &mut f).unwrap();
    assert!(s
        .score_explained(
            "q",
            &["x".repeat(65537).as_str()],
            &ExplanationOptions::default()
        )
        .is_err());
    assert!(s
        .score_explained(" ", &["d"], &ExplanationOptions::default())
        .is_err());
}
#[test]
fn output_budget_is_an_error_instead_of_a_partial_explanation() {
    let m = model();
    let mut f = LexicalFeatures;
    let mut s = LinearReranker::new(&m, &mut f).unwrap();
    assert!(matches!(
        s.score_explained(
            "q",
            &["q answer"; 10],
            &ExplanationOptions::new(10, 1024).unwrap()
        ),
        Err(RerankError::Budget(_))
    ));
}
#[test]
fn custom_component_api_checks_algebra_and_identity() {
    let x = ExplainedScores::from_components(
        "custom-v1",
        "q",
        &["d"],
        vec![vec![ScoreComponent::new("signal", 0.5, 2.).unwrap()]],
        &ExplanationOptions::default(),
    )
    .unwrap();
    assert_eq!(x.scores(), [1.]);
    assert_eq!(x.recipe(), "custom-v1");
    assert_eq!(x.documents()[0].document_fingerprint().len(), 64);
}
