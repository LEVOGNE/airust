#![cfg(feature = "quality-gates")]
use airust::{quality::*, quality_gates::*};
use std::time::Duration;
fn case(
    id: &str,
    ranking: &[&str],
    query: &str,
    grades: &[(&str, u8)],
    time: Option<u64>,
) -> CaseMeasurement {
    CaseMeasurement::evaluate(
        id,
        query,
        ranking,
        &Judgments::new(grades.iter().copied()).unwrap(),
        2,
        UnjudgedPolicy::NonRelevant,
        time.map(Duration::from_millis),
    )
    .unwrap()
}
fn run(cases: Vec<CaseMeasurement>, rss: Option<u64>) -> EvaluationRun {
    EvaluationRun::new("fixture", "0".repeat(64), "recipe", cases, rss).unwrap()
}
#[test]
fn identical_runs_pass_and_fingerprints_are_order_independent() {
    let a = case("a", &["x"], "q1", &[("x", 1)], None);
    let b = case("b", &["y"], "q2", &[("y", 1)], None);
    let x = run(vec![a.clone(), b.clone()], None);
    let y = run(vec![b, a], None);
    assert_eq!(x.fingerprint().unwrap(), y.fingerprint().unwrap());
    assert_eq!(
        compare_runs(&x, &y, &GatePolicy::default())
            .unwrap()
            .verdict(),
        GateVerdict::Passed
    );
}
#[test]
fn a_mean_improvement_does_not_hide_a_query_regression() {
    let a = run(
        vec![
            case("a", &["x"], "q1", &[("x", 1)], None),
            case("b", &[], "q2", &[("y", 1)], None),
        ],
        None,
    );
    let b = run(
        vec![
            case("a", &["z", "x"], "q1", &[("x", 1)], None),
            case("b", &["y"], "q2", &[("y", 1)], None),
        ],
        None,
    );
    let r = compare_runs(&a, &b, &GatePolicy::default()).unwrap();
    assert_eq!(r.verdict(), GateVerdict::Failed);
    assert_eq!(r.regressed_queries(), 1);
    let policy = GatePolicy::default()
        .with_quality_limits(None, 0., 0., 1)
        .unwrap();
    assert_eq!(
        compare_runs(&a, &b, &policy).unwrap().verdict(),
        GateVerdict::Passed
    );
}
#[test]
fn changed_labels_queries_corpus_or_cutoffs_cannot_be_compared() {
    let a = run(vec![case("a", &["x"], "q", &[("x", 1)], None)], None);
    for b in [
        run(
            vec![case("a", &["x"], "different", &[("x", 1)], None)],
            None,
        ),
        run(vec![case("a", &["x"], "q", &[("x", 2)], None)], None),
        run(vec![case("b", &["x"], "q", &[("x", 1)], None)], None),
    ] {
        assert!(compare_runs(&a, &b, &GatePolicy::default()).is_err());
    }
    let c = a.cases()["a"].clone();
    let b = EvaluationRun::new("fixture", "1".repeat(64), "recipe", vec![c], None).unwrap();
    assert!(compare_runs(&a, &b, &GatePolicy::default()).is_err());
    for (k, policy) in [
        (1, UnjudgedPolicy::NonRelevant),
        (2, UnjudgedPolicy::RequireJudged),
    ] {
        let c = CaseMeasurement::evaluate(
            "a",
            "q",
            &["x"],
            &Judgments::new([("x", 1)]).unwrap(),
            k,
            policy,
            None,
        )
        .unwrap();
        assert!(compare_runs(&a, &run(vec![c], None), &GatePolicy::default()).is_err());
    }
}
#[test]
fn undefined_metrics_and_unmeasured_resources_never_pass() {
    let a = run(vec![case("a", &[], "q", &[("x", 0)], None)], None);
    assert_eq!(
        compare_runs(&a, &a, &GatePolicy::default())
            .unwrap()
            .verdict(),
        GateVerdict::Undetermined
    );
    let a = run(vec![case("a", &["x"], "q", &[("x", 1)], None)], None);
    let policy = GatePolicy::default()
        .with_resource_limits(Some(2.), Some(100))
        .unwrap();
    assert_eq!(
        compare_runs(&a, &a, &policy).unwrap().verdict(),
        GateVerdict::Undetermined
    );
}
#[test]
fn resource_and_floor_limits_are_explicit() {
    let a = run(vec![case("a", &["x"], "q", &[("x", 1)], Some(1))], Some(10));
    let b = run(vec![case("a", &["x"], "q", &[("x", 1)], Some(3))], Some(30));
    assert_eq!(
        compare_runs(
            &a,
            &b,
            &GatePolicy::default()
                .with_resource_limits(Some(2.), Some(20))
                .unwrap()
        )
        .unwrap()
        .verdict(),
        GateVerdict::Failed
    );
    let policy = GatePolicy::new(Metric::Precision)
        .with_quality_limits(Some(0.75), 1., 1., 100)
        .unwrap();
    assert_eq!(
        compare_runs(&a, &b, &policy).unwrap().verdict(),
        GateVerdict::Failed
    );
}
#[test]
fn malformed_runs_and_nonfinite_limits_are_rejected() {
    assert!(GatePolicy::default()
        .with_quality_limits(Some(f64::NAN), 0., 0., 0)
        .is_err());
    assert!(GatePolicy::default()
        .with_resource_limits(Some(f64::INFINITY), None)
        .is_err());
    assert!(EvaluationRun::new("x", "bad", "r", vec![], None).is_err());
    let c = case("a", &[], "q", &[("x", 1)], None);
    assert!(EvaluationRun::new("x", "0".repeat(64), "r", vec![c.clone(), c], None).is_err());
}
#[test]
fn every_metric_detects_a_lost_relevant_document() {
    for metric in [
        Metric::Precision,
        Metric::Recall,
        Metric::ReciprocalRank,
        Metric::Ndcg,
    ] {
        let a = run(vec![case("a", &["x"], "q", &[("x", 1)], None)], None);
        let b = run(vec![case("a", &[], "q", &[("x", 1)], None)], None);
        assert_eq!(
            compare_runs(&a, &b, &GatePolicy::new(metric))
                .unwrap()
                .verdict(),
            GateVerdict::Failed
        );
    }
}
