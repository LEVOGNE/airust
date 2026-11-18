#![cfg(feature = "feedback")]
use airust::{feedback::*, reranking::*, Context, KnowledgeEntry};
use serde_json::{json, Value};
use std::collections::BTreeMap;
#[derive(Default)]
struct Base {
    mode: u8,
    calls: usize,
    recipe: String,
}
impl Reranker for Base {
    fn recipe(&self) -> &str {
        if self.recipe.is_empty() {
            "neutral-v1"
        } else {
            &self.recipe
        }
    }
    fn score(&mut self, _: &str, docs: &[&str]) -> Result<Vec<f64>, RerankError> {
        self.calls += 1;
        match self.mode {
            1 => Err(RerankError::Provider("offline".into())),
            2 => Ok(vec![]),
            3 => Ok(vec![f64::NAN; docs.len()]),
            4 => {
                self.recipe = "changed".into();
                Ok(vec![0.; docs.len()])
            }
            _ => Ok(vec![0.; docs.len()]),
        }
    }
}
fn event(id: &str, ns: &str, q: &str, d: &str, r: Rating) -> FeedbackEvent {
    FeedbackEvent::new(id, ns, q, d, r).unwrap()
}
#[test]
fn duplicate_update_retract_preserve_revision_and_exact_document_version() {
    let mut l = FeedbackLedger::default();
    let e = event("1", "tenant", "  Straße  ", "Doc", Rating::Helpful);
    assert!(l.record(e.clone()).unwrap().changed);
    assert!(!l.record(e).unwrap().changed);
    assert_eq!(l.revision(), 1);
    assert_eq!(l.evidence("tenant", "straße", "Doc", 5).unwrap().helpful, 1);
    assert_eq!(l.evidence("other", "straße", "Doc", 5).unwrap().helpful, 0);
    assert_eq!(l.evidence("tenant", "straße", "doc", 5).unwrap().helpful, 0);
    l.record(event("1", "tenant", "straße", "Doc", Rating::Unhelpful))
        .unwrap();
    let v = l.evidence("tenant", "straße", "Doc", 0).unwrap();
    assert_eq!((v.helpful, v.unhelpful), (0, 1));
    assert!(v.truncated);
    assert!(l.retract("1").unwrap().changed);
    assert!(!l.retract("1").unwrap().changed);
    assert_eq!(l.revision(), 3);
    assert!(l.is_empty());
}
#[test]
fn namespace_fingerprint_and_strict_roundtrip() {
    let mut l = FeedbackLedger::default();
    l.record(event(
        "1",
        "a",
        "private question",
        "private document",
        Rating::Helpful,
    ))
    .unwrap();
    let f = l.profile_fingerprint("a").unwrap();
    let s = l.to_json().unwrap();
    assert!(!s.contains("private"));
    let back = FeedbackLedger::from_json(&s, FeedbackLimits::default()).unwrap();
    assert_eq!(f, back.profile_fingerprint("a").unwrap());
    l.record(event("2", "b", "q", "d", Rating::Helpful))
        .unwrap();
    assert_eq!(f, l.profile_fingerprint("a").unwrap());
    assert_eq!(l.events().count(), 2);
    let data: Value = serde_json::from_str(&s).unwrap();
    for bad in [
        json!({"schema_version":2,"revision":1,"events":data["events"]}),
        json!({"schema_version":1,"revision":0,"events":data["events"]}),
        json!({"schema_version":1,"revision":2,"events":[data["events"][0],data["events"][0]]}),
    ] {
        assert!(FeedbackLedger::from_json(&bad.to_string(), FeedbackLimits::default()).is_err());
    }
    let mut bad = data;
    bad["events"][0]["key"]["query"] = json!("fake");
    assert!(FeedbackLedger::from_json(&bad.to_string(), FeedbackLimits::default()).is_err());
}
#[test]
fn limits_and_revision_exhaustion_are_atomic() {
    let mut l = FeedbackLedger::with_limits(FeedbackLimits::new(1, 1024).unwrap());
    l.record(event("1", "a", "q", "d", Rating::Helpful))
        .unwrap();
    let s = l.to_json().unwrap();
    assert!(l
        .record(event("2", "a", "q", "d", Rating::Helpful))
        .is_err());
    assert_eq!(s, l.to_json().unwrap());
    let mut v: Value = serde_json::from_str(&s).unwrap();
    v["revision"] = json!(u64::MAX);
    let s = v.to_string();
    let mut max = FeedbackLedger::from_json(&s, FeedbackLimits::default()).unwrap();
    let s = max.to_json().unwrap();
    assert!(matches!(max.retract("1"), Err(FeedbackError::Exhausted)));
    assert!(matches!(
        max.record(event("1", "a", "q", "d", Rating::Unhelpful)),
        Err(FeedbackError::Exhausted)
    ));
    assert_eq!(max.to_json().unwrap(), s);
    assert!(FeedbackLimits::new(0, 1).is_err());
    assert!(FeedbackLimits::new(100001, 1).is_err());
    assert!(FeedbackEvent::new("\n", "n", "q", "d", Rating::Helpful).is_err());
    assert!(FeedbackEvent::new("a", "n", &"x".repeat(16385), "d", Rating::Helpful).is_err());
    assert!(FeedbackLedger::from_json(&s, FeedbackLimits::new(10, 1).unwrap()).is_err());
}
#[test]
fn seeded_updates_match_independent_full_scan() {
    let mut l = FeedbackLedger::default();
    let mut oracle = BTreeMap::<String, (String, String, String, bool)>::new();
    let mut seed = 73u64;
    for step in 0..1500 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let id = format!("{}", seed % 100);
        if seed.is_multiple_of(7) {
            l.retract(&id).unwrap();
            oracle.remove(&id);
        } else {
            let ns = format!("n{}", seed % 3);
            let q = format!("q{}", (seed >> 8) % 5);
            let d = format!("d{}", (seed >> 16) % 7);
            let positive = seed & 32 != 0;
            l.record(event(
                &id,
                &ns,
                &q,
                &d,
                if positive {
                    Rating::Helpful
                } else {
                    Rating::Unhelpful
                },
            ))
            .unwrap();
            oracle.insert(id, (ns, q, d, positive));
        }
        if step % 50 == 0 {
            for ns in 0..3 {
                for q in 0..5 {
                    for d in 0..7 {
                        let key = (format!("n{ns}"), format!("q{q}"), format!("d{d}"));
                        let records = oracle
                            .values()
                            .filter(|v| (&v.0, &v.1, &v.2) == (&key.0, &key.1, &key.2))
                            .collect::<Vec<_>>();
                        let helpful = records.iter().filter(|v| v.3).count();
                        let got = l.evidence(&key.0, &key.1, &key.2, 1000).unwrap();
                        assert_eq!(
                            (got.helpful, got.unhelpful),
                            (helpful, records.len() - helpful)
                        );
                    }
                }
            }
        }
    }
}
#[test]
fn smoothed_bias_is_inspectable_bounded_and_scoped() {
    let mut l = FeedbackLedger::default();
    for i in 0..3 {
        l.record(event(&i.to_string(), "n", "q", "answer", Rating::Helpful))
            .unwrap();
    }
    l.record(event("bad", "n", "q", "glossary", Rating::Unhelpful))
        .unwrap();
    let mut base = Base::default();
    let mut scorer =
        FeedbackReranker::new(&mut base, &l, "n", FeedbackOptions::new(2., 2).unwrap()).unwrap();
    let scores = scorer
        .score("q", &["answer", "glossary", "ANSWER"])
        .unwrap();
    assert_eq!(scores, [1.2, -2. / 3., 0.]);
    assert_eq!(scorer.last_adjustments()[0].helpful, 3);
    assert_eq!(scorer.last_adjustments()[0].batch_position, 1);
    assert!(scores.iter().all(|s| s.abs() <= 2.));
    assert_eq!(scorer.score("other", &["answer"]).unwrap(), [0.]);
    assert!(scorer.score("q", &[]).unwrap().is_empty());
    assert!(scorer.last_adjustments().is_empty());
}
#[test]
fn recipe_tracks_profile_options_and_base_and_errors_leave_no_partial_adjustments() {
    let mut l = FeedbackLedger::default();
    l.record(event("1", "n", "q", "d", Rating::Helpful))
        .unwrap();
    let mut base = Base::default();
    let recipe = FeedbackReranker::new(&mut base, &l, "n", FeedbackOptions::default())
        .unwrap()
        .recipe()
        .to_owned();
    l.retract("1").unwrap();
    assert_ne!(
        recipe,
        FeedbackReranker::new(&mut base, &l, "n", FeedbackOptions::default())
            .unwrap()
            .recipe()
    );
    for mode in 1..=4 {
        let mut b = Base {
            mode,
            ..Base::default()
        };
        let mut s = FeedbackReranker::new(&mut b, &l, "n", FeedbackOptions::default()).unwrap();
        let error = s.score("q", &["d", "e"]).unwrap_err();
        if mode == 4 {
            assert!(matches!(error, RerankError::RecipeChanged));
        }
        assert!(s.last_adjustments().is_empty());
    }
    for w in [f64::NAN, f64::INFINITY, -1., 101.] {
        assert!(FeedbackOptions::new(w, 2).is_err());
    }
}
#[test]
fn scope_filter_precedes_feedback_provider_and_retraction_restores_order() {
    let mut l = FeedbackLedger::default();
    l.record(event("1", "n", "q", "answer", Rating::Helpful))
        .unwrap();
    let docs = [
        KnowledgeEntry::new("private", "secret").with_metadata(json!({"scope":"private"})),
        KnowledgeEntry::new("a", "glossary").with_metadata(json!({"scope":"public"})),
        KnowledgeEntry::new("b", "answer").with_metadata(json!({"scope":"public"})),
    ];
    let mut base = Base::default();
    let options =
        RerankOptions::new(2).with_context(Context::new().with_metadata("scope", "public"));
    {
        let mut s = FeedbackReranker::new(&mut base, &l, "n", FeedbackOptions::default()).unwrap();
        let out = rerank("q", &docs, &mut s, &options).unwrap();
        assert_eq!(out.results()[0].entry().id, "b");
        assert_eq!(s.last_adjustments().len(), 2);
    }
    l.retract("1").unwrap();
    let mut s = FeedbackReranker::new(&mut base, &l, "n", FeedbackOptions::default()).unwrap();
    let out = rerank("q", &docs, &mut s, &options).unwrap();
    assert_eq!(out.results()[0].entry().id, "a");
}
