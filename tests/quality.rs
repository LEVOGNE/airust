#![cfg(feature = "quality")]

use airust::quality::{
    evaluate_ranking, summarize, Judgments, QualityError, UnjudgedPolicy as Policy,
};
fn close(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-12, "{a} != {b}");
}
#[test]
fn hand_calculated_graded_ranking_has_defined_precision_recall_rr_and_ndcg() {
    let judgments = Judgments::new([("a", 3), ("b", 2), ("c", 1), ("d", 0)]).unwrap();
    let metrics = evaluate_ranking(&["b", "a", "d"], &judgments, 3, Policy::RequireJudged).unwrap();
    close(metrics.precision_at_k(), 2.0 / 3.0);
    close(metrics.recall_at_k().unwrap(), 2.0 / 3.0);
    close(metrics.reciprocal_rank_at_k(), 1.0);
    close(
        metrics.ndcg_at_k().unwrap(),
        (3.0 + 7.0 / 3f64.log2()) / (7.0 + 3.0 / 3f64.log2() + 0.5),
    );
    assert_eq!(metrics.judged_retrieved(), 3);
    assert_eq!(metrics.unjudged_retrieved(), 0);
    assert_eq!(metrics.relevant_retrieved(), 2);
    let perfect = evaluate_ranking(&["a", "b", "c"], &judgments, 3, Policy::RequireJudged).unwrap();
    close(perfect.ndcg_at_k().unwrap(), 1.0);
}
#[test]
fn cutoff_controls_the_denominator_and_ignores_ids_beyond_the_window() {
    let judgments = Judgments::new([("a", 1), ("b", 0)]).unwrap();
    let short = evaluate_ranking(&["a"], &judgments, 5, Policy::RequireJudged).unwrap();
    close(short.precision_at_k(), 0.2);
    close(short.recall_at_k().unwrap(), 1.0);
    assert_eq!(short.returned(), 1);
    let beyond = evaluate_ranking(
        &["b", "a", "a", "unknown"],
        &judgments,
        1,
        Policy::RequireJudged,
    )
    .unwrap();
    assert_eq!(beyond.returned(), 1);
    close(beyond.reciprocal_rank_at_k(), 0.0);
    close(beyond.recall_at_k().unwrap(), 0.0);
    assert!(matches!(
        evaluate_ranking(&["a"], &judgments, 0, Policy::RequireJudged),
        Err(QualityError::Cutoff)
    ));
}
#[test]
fn missing_positive_judgments_are_undefined_and_empty_rankings_are_not_perfect() {
    let none = Judgments::new([("a", 0)]).unwrap();
    let metrics = evaluate_ranking(&["a"], &none, 3, Policy::RequireJudged).unwrap();
    assert_eq!(metrics.recall_at_k(), None);
    assert_eq!(metrics.ndcg_at_k(), None);
    close(metrics.precision_at_k(), 0.0);
    let positive = Judgments::new([("a", 1)]).unwrap();
    let empty = evaluate_ranking::<&str>(&[], &positive, 3, Policy::RequireJudged).unwrap();
    assert_eq!(empty.returned(), 0);
    close(empty.recall_at_k().unwrap(), 0.0);
    close(empty.ndcg_at_k().unwrap(), 0.0);
    assert!(Judgments::new(Vec::<(String, u8)>::new())
        .unwrap()
        .is_empty());
}
#[test]
fn unjudged_results_need_an_explicit_policy_and_remain_visible_in_reports() {
    let judgments = Judgments::new([("a", 1), ("b", 0)]).unwrap();
    assert!(matches!(
        evaluate_ranking(&["unknown", "a"], &judgments, 2, Policy::RequireJudged),
        Err(QualityError::Unjudged(_))
    ));
    let metrics = evaluate_ranking(&["unknown", "a"], &judgments, 2, Policy::NonRelevant).unwrap();
    close(metrics.precision_at_k(), 0.5);
    close(metrics.reciprocal_rank_at_k(), 0.5);
    assert_eq!(metrics.unjudged_retrieved(), 1);
    assert_eq!(metrics.judged_retrieved(), 1);
    assert_eq!(judgments.grade("b"), Some(0));
    assert_eq!(judgments.grade("unknown"), None);
}
#[test]
fn duplicate_ids_bad_grades_and_empty_ids_cannot_inflate_results() {
    assert!(Judgments::new([("a", 1), ("a", 2)]).is_err());
    assert!(Judgments::new([(" ", 1)]).is_err());
    assert!(Judgments::new([("a", 32)]).is_err());
    assert!(Judgments::new([("a", 255)]).is_err());
    let judgments = Judgments::new([("a", 31)]).unwrap();
    assert!(matches!(
        evaluate_ranking(&["a", "a"], &judgments, 2, Policy::NonRelevant),
        Err(QualityError::DuplicateId(_))
    ));
    assert!(evaluate_ranking(&[""], &judgments, 1, Policy::NonRelevant).is_err());
    assert!(
        evaluate_ranking(&["a"], &judgments, 1, Policy::RequireJudged)
            .unwrap()
            .ndcg_at_k()
            .unwrap()
            .is_finite()
    );
}
#[test]
fn macro_means_expose_defined_denominators_and_reject_mixed_configuration() {
    let positive = Judgments::new([("a", 1), ("b", 0)]).unwrap();
    let none = Judgments::new([("a", 0)]).unwrap();
    let a = evaluate_ranking(&["a"], &positive, 2, Policy::RequireJudged).unwrap();
    let b = evaluate_ranking(&["b"], &positive, 2, Policy::RequireJudged).unwrap();
    let c = evaluate_ranking(&["a"], &none, 2, Policy::RequireJudged).unwrap();
    let summary = summarize(&[a.clone(), b, c]).unwrap();
    assert_eq!(summary.queries(), 3);
    assert_eq!(summary.recall_queries(), 2);
    assert_eq!(summary.ndcg_queries(), 2);
    close(summary.mean_precision_at_k(), 1.0 / 6.0);
    close(summary.mean_recall_at_k().unwrap(), 0.5);
    close(summary.mean_reciprocal_rank_at_k(), 1.0 / 3.0);
    close(summary.mean_ndcg_at_k().unwrap(), 0.5);
    assert!(summarize(&[]).is_err());
    let other = evaluate_ranking(&["a"], &positive, 1, Policy::RequireJudged).unwrap();
    assert!(summarize(&[a.clone(), other]).is_err());
    let other = evaluate_ranking(&["a"], &positive, 2, Policy::NonRelevant).unwrap();
    assert!(summarize(&[a, other]).is_err());
    let zero = evaluate_ranking(&["a"], &none, 2, Policy::RequireJudged).unwrap();
    let summary = summarize(&[zero]).unwrap();
    assert_eq!(summary.mean_recall_at_k(), None);
    assert_eq!(summary.mean_ndcg_at_k(), None);
}
#[test]
fn all_small_permutations_keep_metrics_bounded_and_complete_recall() {
    let judgments = Judgments::new([("a", 3), ("b", 2), ("c", 1), ("d", 0)]).unwrap();
    let ids = ["a", "b", "c", "d"];
    for a in 0..4 {
        for b in 0..4 {
            for c in 0..4 {
                for d in 0..4 {
                    if [a, b, c, d]
                        .into_iter()
                        .collect::<std::collections::BTreeSet<_>>()
                        .len()
                        != 4
                    {
                        continue;
                    }
                    let ranking = [ids[a], ids[b], ids[c], ids[d]];
                    for k in 1..=5 {
                        let m = evaluate_ranking(&ranking, &judgments, k, Policy::RequireJudged)
                            .unwrap();
                        for value in [
                            m.precision_at_k(),
                            m.recall_at_k().unwrap(),
                            m.reciprocal_rank_at_k(),
                            m.ndcg_at_k().unwrap(),
                        ] {
                            assert!((0.0..=1.0).contains(&value));
                        }
                        if k >= 4 {
                            close(m.recall_at_k().unwrap(), 1.0);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn public_reference_literal_identifiers_have_manual_expected_top_hits() {
    use airust::{Engine, KnowledgeEntry};
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/quality-reference-v1.json")).unwrap();
    let documents: Vec<KnowledgeEntry> =
        serde_json::from_value(fixture["documents"].clone()).unwrap();
    let index = Engine::from_entries(documents).unwrap();
    let mut checked = 0;
    for query in fixture["queries"].as_array().unwrap() {
        if let Some(expected) = query["lexical_top1"].as_str() {
            assert_eq!(
                index.search(query["text"].as_str().unwrap(), 1)[0].entry.id,
                expected
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 8);
}
