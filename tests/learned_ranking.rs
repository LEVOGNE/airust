#![cfg(feature = "learned-ranking")]
use airust::{learned_ranking::*, reranking::*};
use serde_json::{json, Value};
use std::collections::BTreeSet;
fn data() -> Vec<TrainingExample> {
    (0..60)
        .flat_map(|i| {
            let q = format!("topic{i} detail{i}");
            [
                TrainingExample::new(&q, format!("{q} explanation"), true).unwrap(),
                TrainingExample::new(&q, format!("unrelated{i} archive"), false).unwrap(),
            ]
        })
        .collect()
}
#[test]
fn deterministic_loss_falls_roundtrip_and_unseen_query_ranking() {
    let d = data();
    let m = train(&d, &mut LexicalFeatures, &TrainingOptions::default()).unwrap();
    let m2 = train(&d, &mut LexicalFeatures, &TrainingOptions::default()).unwrap();
    assert_eq!(m.to_json().unwrap(), m2.to_json().unwrap());
    assert!(m.report().final_loss() < m.report().initial_loss());
    let restored = LinearModel::from_json(&m.to_json().unwrap()).unwrap();
    assert_eq!(m.fingerprint().unwrap(), restored.fingerprint().unwrap());
    let mut f = LexicalFeatures;
    let mut s = LinearReranker::new(&restored, &mut f).unwrap();
    let scores = s
        .score(
            "unseen detail",
            &["unrelated archive", "unseen detail explanation"],
        )
        .unwrap();
    assert!(scores[1] > scores[0]);
    assert!(!m.to_json().unwrap().contains("topic0"));
}
#[test]
fn grouped_holdout_is_disjoint_deterministic_and_measured() {
    let d = data();
    let (t, h) = split_grouped(&d, 25, 42).unwrap();
    assert_eq!((t.clone(), h.clone()), split_grouped(&d, 25, 42).unwrap());
    let groups = t.iter().map(|i| d[*i].group()).collect::<BTreeSet<_>>();
    assert!(h.iter().all(|i| !groups.contains(d[*i].group())));
    let trainset = t.iter().map(|i| d[*i].clone()).collect::<Vec<_>>();
    let hold = h.iter().map(|i| d[*i].clone()).collect::<Vec<_>>();
    let m = train(&trainset, &mut LexicalFeatures, &TrainingOptions::default()).unwrap();
    let e = evaluate(&m, &hold, &mut LexicalFeatures).unwrap();
    assert_eq!(e.correct_at_zero, e.examples);
    assert!(e.loss < m.report().initial_loss());
    let same = [
        TrainingExample::new("q", "a", true).unwrap(),
        TrainingExample::new(" Q ", "b", false).unwrap(),
    ];
    assert!(split_grouped(&same, 50, 1).is_err());
}
#[test]
fn limits_invalid_config_and_degenerate_training_are_rejected() {
    for c in [
        (0, 0.1, 0., 1.),
        (201, 0.1, 0., 1.),
        (5, f64::NAN, 0., 1.),
        (5, 0.1, -1., 1.),
        (5, 0.1, 0., 0.),
    ] {
        assert!(TrainingConfig::new(c.0, c.1, c.2, c.3).is_err());
    }
    assert!(TrainingExample::new(" ", "d", true).is_err());
    assert!(TrainingExample::new("q", "x".repeat(65537), true).is_err());
    assert!(train(&[], &mut LexicalFeatures, &TrainingOptions::default()).is_err());
    let one = [TrainingExample::new("q", "d", true).unwrap()];
    assert!(train(&one, &mut LexicalFeatures, &TrainingOptions::default()).is_err());
    assert!(FeatureSchema::new("x", vec!["a".into(), "a".into()]).is_err());
}
struct Bad {
    mode: u8,
    changed: bool,
}
impl FeatureExtractor for Bad {
    fn schema(&self) -> FeatureSchema {
        FeatureSchema::new(if self.changed { "v2" } else { "v1" }, vec!["bias".into()]).unwrap()
    }
    fn extract(&mut self, _: &str, _: &str) -> Result<Vec<f64>, LearningError> {
        match self.mode {
            1 => Ok(vec![f64::NAN]),
            2 => Ok(vec![2.]),
            3 => Ok(vec![]),
            4 => {
                self.changed = true;
                Ok(vec![1.])
            }
            _ => Ok(vec![1.]),
        }
    }
}
#[test]
fn bad_or_stateful_extractors_never_create_models() {
    for mode in 1..=4 {
        assert!(train(
            &data(),
            &mut Bad {
                mode,
                changed: false
            },
            &TrainingOptions::default()
        )
        .is_err());
    }
    let m = train(&data(), &mut LexicalFeatures, &TrainingOptions::default()).unwrap();
    assert!(LinearReranker::new(
        &m,
        &mut Bad {
            mode: 0,
            changed: false
        }
    )
    .is_err());
}
#[test]
fn malformed_artifacts_unknown_fields_dimensions_and_extremes_are_rejected() {
    let m = train(&data(), &mut LexicalFeatures, &TrainingOptions::default()).unwrap();
    let original: Value = serde_json::from_str(&m.to_json().unwrap()).unwrap();
    for (path, value) in [
        (vec!["schema_version"], json!(2)),
        (vec!["weights"], json!([])),
        (vec!["weights"], json!([1e100, 0, 0, 0, 0, 0])),
        (vec!["report", "examples"], json!(0)),
        (vec!["report", "manifest"], json!("bad")),
        (vec!["report", "final_loss"], json!(-1)),
        (vec!["unknown"], json!(1)),
    ] {
        let mut v = original.clone();
        let mut target = &mut v;
        for p in &path[..path.len() - 1] {
            target = &mut target[*p];
        }
        target[path[path.len() - 1]] = value;
        assert!(LinearModel::from_json(&v.to_string()).is_err());
    }
    assert!(LinearModel::from_json(&" ".repeat(1024 * 1024 + 1)).is_err());
}
#[test]
fn custom_schema_is_trainable_and_serialized_recipe_binds_features() {
    let mut f = Bad {
        mode: 0,
        changed: false,
    };
    let m = train(&data(), &mut f, &TrainingOptions::default()).unwrap();
    assert_eq!(m.weights().len(), 1);
    let s = LinearReranker::new(&m, &mut f).unwrap();
    assert!(s.recipe().starts_with("airust-linear-v1/"));
    let mut lexical = LexicalFeatures;
    assert!(LinearReranker::new(&m, &mut lexical).is_err());
}
#[test]
fn lexical_features_are_bounded_with_unicode_and_original_inputs_unchanged() {
    let query = "STRASSE Straße Ａ";
    let doc = "Straße A erklärt Straße";
    let mut f = LexicalFeatures;
    let values = f.extract(query, doc).unwrap();
    assert_eq!(values.len(), 6);
    assert!(values
        .iter()
        .all(|v| v.is_finite() && (0.0..=1.).contains(v)));
    assert_eq!(query, "STRASSE Straße Ａ");
    assert_eq!(doc, "Straße A erklärt Straße");
}
#[cfg(feature = "execution")]
#[test]
fn cancellation_deadline_and_progress_prevent_partial_training() {
    use airust::execution::*;
    use std::sync::{Arc, Mutex};
    let token = CancellationToken::new();
    token.cancel();
    assert!(matches!(
        train(
            &data(),
            &mut LexicalFeatures,
            &TrainingOptions::default().with_control(WorkControl::new(token))
        ),
        Err(LearningError::Execution(ExecutionError::Cancelled))
    ));
    let c = WorkControl::default()
        .with_timeout(std::time::Duration::ZERO)
        .unwrap();
    assert!(matches!(
        train(
            &data(),
            &mut LexicalFeatures,
            &TrainingOptions::default().with_control(c)
        ),
        Err(LearningError::Execution(ExecutionError::DeadlineExceeded))
    ));
    let token = CancellationToken::new();
    let cancel = token.clone();
    let c = WorkControl::new(token).with_observer(move |e: &ProgressEvent| {
        if e.phase() == WorkPhase::Training && e.completed() == 1 {
            cancel.cancel();
        }
    });
    assert!(matches!(
        train(
            &data(),
            &mut LexicalFeatures,
            &TrainingOptions::default().with_control(c)
        ),
        Err(LearningError::Execution(ExecutionError::Cancelled))
    ));
    #[derive(Clone)]
    struct Observer(Arc<Mutex<Vec<ProgressEvent>>>);
    impl ProgressObserver for Observer {
        fn on_progress(&self, e: &ProgressEvent) {
            self.0.lock().unwrap().push(*e);
        }
    }
    let observer = Observer(Arc::new(Mutex::new(Vec::new())));
    let control = WorkControl::default().with_observer(observer.clone());
    train(
        &data(),
        &mut LexicalFeatures,
        &TrainingOptions::default().with_control(control),
    )
    .unwrap();
    let events = observer.0.lock().unwrap();
    assert!(events.iter().any(|e| e.phase() == WorkPhase::Features));
    assert!(events
        .iter()
        .any(|e| e.phase() == WorkPhase::Training && e.completed() == 100));
}
