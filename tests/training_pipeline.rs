#![cfg(feature = "training-pipeline")]
use airust::{feedback::*, learned_ranking::*, quality::*, quality_gates::*, training_pipeline::*};
fn dataset() -> (FeedbackLedger, Vec<TrainingItem>, CurationPolicy) {
    let mut l = FeedbackLedger::default();
    let mut items = Vec::new();
    let mut ids = Vec::new();
    for i in 0..40 {
        for positive in [true, false] {
            let id = format!("{i}-{positive}");
            let q = format!("topic{i}");
            let doc = if positive {
                format!("topic{i} answer")
            } else {
                format!("other{i}")
            };
            l.record(
                FeedbackEvent::new(
                    &id,
                    "n",
                    &q,
                    &doc,
                    if positive {
                        Rating::Helpful
                    } else {
                        Rating::Unhelpful
                    },
                )
                .unwrap(),
            )
            .unwrap();
            items.push(TrainingItem::new(&id, q, doc).unwrap());
            ids.push(id);
        }
    }
    (
        l,
        items,
        CurationPolicy::new(ids)
            .unwrap()
            .with_minimums(1, 1)
            .unwrap(),
    )
}
fn model(epochs: usize) -> LinearModel {
    train(
        &[
            TrainingExample::new("q", "q answer", true).unwrap(),
            TrainingExample::new("q", "other", false).unwrap(),
        ],
        &mut LexicalFeatures,
        &TrainingOptions::new(TrainingConfig::new(epochs, 0.2, 0.01, 5.).unwrap()),
    )
    .unwrap()
}
fn evaluation(recipe: &str) -> EvaluationRun {
    let c = CaseMeasurement::evaluate(
        "q",
        "q",
        &["a"],
        &Judgments::new([("a", 1)]).unwrap(),
        1,
        UnjudgedPolicy::RequireJudged,
        None,
    )
    .unwrap();
    EvaluationRun::new("fixture", "0".repeat(64), recipe, vec![c], None).unwrap()
}
#[test]
fn explicit_trust_is_required_and_unjudged_items_do_not_become_negatives() {
    let (l, items, p) = dataset();
    let empty = CurationPolicy::new(Vec::<String>::new()).unwrap();
    assert!(curate_feedback("n", &l, &items, &empty).unwrap().is_empty());
    let d = curate_feedback("n", &l, &items, &p).unwrap();
    assert_eq!(d.len(), 80);
    assert!(curate_feedback("other", &l, &items, &p).unwrap().is_empty());
}
#[test]
fn feedback_retraction_marks_dataset_stale_and_rebuild_removes_the_label() {
    let (mut l, items, p) = dataset();
    let d = curate_feedback("n", &l, &items, &p).unwrap();
    l.retract("0-true").unwrap();
    assert!(!d.is_current(&l).unwrap());
    assert_eq!(curate_feedback("n", &l, &items, &p).unwrap().len(), 79);
}
#[test]
fn grouped_training_has_no_query_leakage_and_reports_dataset_identity() {
    let (l, items, p) = dataset();
    let d = curate_feedback("n", &l, &items, &p).unwrap();
    let result = train_grouped(
        &d,
        &mut LexicalFeatures,
        &TrainingOptions::default(),
        25,
        42,
    )
    .unwrap();
    let groups = result
        .training_indices()
        .iter()
        .map(|i| d.examples()[*i].group())
        .collect::<std::collections::BTreeSet<_>>();
    assert!(result
        .holdout_indices()
        .iter()
        .all(|i| !groups.contains(d.examples()[*i].group())));
    assert_eq!(result.dataset_fingerprint(), d.fingerprint().unwrap());
    assert!(result.evaluation().loss.is_finite());
}
#[test]
fn dataset_roundtrip_is_versioned_and_tampering_is_rejected() {
    let (l, items, p) = dataset();
    let d = curate_feedback("n", &l, &items, &p).unwrap();
    let json = d.to_json().unwrap();
    assert!(json.contains("topic0"));
    let copy = TrainingDataset::from_json(&json).unwrap();
    assert_eq!(copy.fingerprint().unwrap(), d.fingerprint().unwrap());
    let mut v: serde_json::Value = serde_json::from_str(&json).unwrap();
    v["data"]["rows"][0]["relevant"] = serde_json::json!(false);
    assert!(TrainingDataset::from_json(&v.to_string()).is_err());
}
#[test]
fn duplicate_content_is_not_multiplied_and_conflicting_groups_fail() {
    let (l, mut items, p) = dataset();
    let copy = TrainingItem::new("duplicate", "topic0", "topic0 answer").unwrap();
    items.push(copy);
    assert_eq!(curate_feedback("n", &l, &items, &p).unwrap().len(), 80);
    items.push(
        TrainingItem::new("conflict", "topic0", "topic0 answer")
            .unwrap()
            .with_group("other-group")
            .unwrap(),
    );
    assert!(curate_feedback("n", &l, &items, &p).is_err());
}
#[test]
fn promotion_checks_gate_model_identity_and_registry_revision() {
    let mut registry = ModelRegistry::new("r", 4).unwrap();
    let m = model(50);
    let b = evaluation("baseline");
    let c = evaluation(&format!("airust-linear-v1/{}", m.fingerprint().unwrap()));
    let gate = compare_runs(&b, &c, &GatePolicy::default()).unwrap();
    let empty = registry.revision().unwrap();
    registry.promote(&empty, m.clone(), &b, &c, &gate).unwrap();
    assert!(matches!(
        registry.promote(&empty, m, &b, &c, &gate),
        Err(PipelineError::Conflict)
    ));
    let m = model(100);
    let wrong = evaluation("not-the-model");
    let gate = compare_runs(&c, &wrong, &GatePolicy::default()).unwrap();
    let rev = registry.revision().unwrap();
    assert!(matches!(
        registry.promote(&rev, m, &c, &wrong, &gate),
        Err(PipelineError::NotApproved)
    ));
    assert_eq!(rev, registry.revision().unwrap());
}
#[test]
fn model_history_roundtrips_and_rollback_prevents_aba() {
    let mut registry = ModelRegistry::new("r", 2).unwrap();
    let a = model(40);
    let b = model(100);
    let ea = evaluation(&format!("airust-linear-v1/{}", a.fingerprint().unwrap()));
    let eb = evaluation(&format!("airust-linear-v1/{}", b.fingerprint().unwrap()));
    let initial = evaluation("baseline");
    let gate = compare_runs(&initial, &ea, &GatePolicy::default()).unwrap();
    let rev = registry.revision().unwrap();
    let a_rev = registry.promote(&rev, a, &initial, &ea, &gate).unwrap();
    let gate = compare_runs(&ea, &eb, &GatePolicy::default()).unwrap();
    let b_rev = registry.promote(&a_rev, b, &ea, &eb, &gate).unwrap();
    let mut copy = ModelRegistry::from_json(&registry.to_json().unwrap()).unwrap();
    let returned = copy.rollback(&b_rev).unwrap();
    assert_eq!(returned.model_fingerprint(), a_rev.model_fingerprint());
    assert_ne!(returned.generation(), a_rev.generation());
    assert!(matches!(
        copy.rollback(&a_rev),
        Err(PipelineError::Conflict)
    ));
}
#[test]
fn ties_and_vote_margins_are_excluded() {
    let mut l = FeedbackLedger::default();
    l.record(FeedbackEvent::new("a", "n", "q", "d", Rating::Helpful).unwrap())
        .unwrap();
    l.record(FeedbackEvent::new("b", "n", "q", "d", Rating::Unhelpful).unwrap())
        .unwrap();
    let p = CurationPolicy::new(["a".into(), "b".into()]).unwrap();
    assert!(
        curate_feedback("n", &l, &[TrainingItem::new("d", "q", "d").unwrap()], &p)
            .unwrap()
            .is_empty()
    );
}
