#![cfg(feature = "evidence-freshness")]
use airust::{evidence_freshness::*, reasoning::*};
use std::collections::BTreeSet;
fn facts() -> Vec<Fact> {
    [('a', 'b'), ('b', 'c'), ('c', 'd')]
        .iter()
        .enumerate()
        .map(|(i, (a, b))| {
            Fact::new(
                format!("f{i}"),
                Statement::new(a.to_string(), "edge", b.to_string()).unwrap(),
            )
            .unwrap()
            .with_sources(["s".into()])
            .unwrap()
        })
        .collect()
}
fn rules() -> Vec<Rule> {
    vec![Rule::new(
        "transitive",
        vec![
            Pattern::new(Term::variable("a"), "edge", Term::variable("b")).unwrap(),
            Pattern::new(Term::variable("b"), "edge", Term::variable("c")).unwrap(),
        ],
        Pattern::new(Term::variable("a"), "edge", Term::variable("c")).unwrap(),
    )
    .unwrap()]
}
fn receipt() -> EvidenceReceipt {
    let f = facts();
    let r = rules();
    let s = [SourceVersion::new("s", "v1").unwrap()];
    let result = infer(&f, &r, &ReasoningOptions::new("r1", ["s".into()]).unwrap()).unwrap();
    capture_evidence(
        &result,
        &EvidenceView::new("r1", &f, &r, &s).unwrap(),
        &FreshnessOptions::default(),
    )
    .unwrap()
}
#[test]
fn roundtrip_and_unrelated_generation_change_preserve_valid_dependencies() {
    let p = receipt();
    let o = FreshnessOptions::default();
    let copy = EvidenceReceipt::from_json(&p.to_json(&o).unwrap(), &o).unwrap();
    assert_eq!(p.fingerprint().unwrap(), copy.fingerprint().unwrap());
    let f = facts();
    let r = rules();
    let s = [SourceVersion::new("s", "v1").unwrap()];
    assert!(verify_evidence(
        &copy,
        &EvidenceView::new("unrelated-new-revision", &f, &r, &s).unwrap(),
        &o
    )
    .unwrap()
    .all_current());
}
#[test]
fn changed_sources_invalidate_dependent_proposals_without_mutation() {
    let p = receipt();
    let before = p.fingerprint().unwrap();
    let f = facts();
    let r = rules();
    let s = [SourceVersion::new("s", "v2").unwrap()];
    let report = verify_evidence(
        &p,
        &EvidenceView::new("r2", &f, &r, &s).unwrap(),
        &FreshnessOptions::default(),
    )
    .unwrap();
    assert!(report
        .proofs()
        .iter()
        .all(|p| p.state == FreshnessState::Stale));
    assert_eq!(before, p.fingerprint().unwrap());
}
#[test]
fn deletion_and_scope_revocation_are_distinct_and_scoped_checks_stop_early() {
    let p = receipt();
    let mut f = facts();
    f.remove(0);
    let r = rules();
    let s = [SourceVersion::new("s", "v1").unwrap()];
    let view = EvidenceView::new("r2", &f, &r, &s).unwrap();
    let report = verify_evidence(&p, &view, &FreshnessOptions::default()).unwrap();
    assert!(report
        .proofs()
        .iter()
        .any(|p| p.state == FreshnessState::Missing));
    let allowed = BTreeSet::new();
    let report = verify_evidence(
        &p,
        &view.with_allowed_facts(&allowed).unwrap(),
        &FreshnessOptions::default(),
    )
    .unwrap();
    assert!(report
        .proofs()
        .iter()
        .all(|p| p.state == FreshnessState::NotAllowed
            && p.findings.iter().all(|f| f.kind == "not_allowed")));
}
#[test]
fn altered_fact_and_rule_content_are_detected() {
    let p = receipt();
    let mut f = facts();
    f[0] = Fact::new("f0", Statement::new("other", "edge", "b").unwrap())
        .unwrap()
        .with_sources(["s".into()])
        .unwrap();
    let s = [SourceVersion::new("s", "v1").unwrap()];
    let r = rules();
    assert!(!verify_evidence(
        &p,
        &EvidenceView::new("r2", &f, &r, &s).unwrap(),
        &FreshnessOptions::default()
    )
    .unwrap()
    .all_current());
    let f = facts();
    let newrule = Rule::new(
        "transitive",
        r[0].body().to_vec(),
        Pattern::new(Term::variable("a"), "changed", Term::variable("c")).unwrap(),
    )
    .unwrap();
    assert!(verify_evidence(
        &p,
        &EvidenceView::new("r2", &f, &[newrule], &s).unwrap(),
        &FreshnessOptions::default()
    )
    .unwrap()
    .proofs()
    .iter()
    .all(|p| p.state == FreshnessState::Stale));
}
#[test]
fn capture_replays_proof_and_rejects_mismatched_views() {
    let f = facts();
    let r = rules();
    let result = infer(&f, &r, &ReasoningOptions::new("r1", ["s".into()]).unwrap()).unwrap();
    let s = [SourceVersion::new("s", "v1").unwrap()];
    assert!(capture_evidence(
        &result,
        &EvidenceView::new("wrong", &f, &r, &s).unwrap(),
        &FreshnessOptions::default()
    )
    .is_err());
    let mut changed = f.clone();
    changed[0] = Fact::new("f0", Statement::new("x", "edge", "y").unwrap())
        .unwrap()
        .with_sources(["s".into()])
        .unwrap();
    assert!(capture_evidence(
        &result,
        &EvidenceView::new("r1", &changed, &r, &s).unwrap(),
        &FreshnessOptions::default()
    )
    .is_err());
    assert!(capture_evidence(
        &result,
        &EvidenceView::new("r1", &f, &r, &[]).unwrap(),
        &FreshnessOptions::default()
    )
    .is_err());
}
#[test]
fn tampered_unknown_duplicate_and_oversized_receipts_fail() {
    let o = FreshnessOptions::default();
    let p = receipt();
    let json = p.to_json(&o).unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&json).unwrap();
    v["data"]["revision"] = serde_json::json!("tampered");
    assert!(EvidenceReceipt::from_json(&v.to_string(), &o).is_err());
    assert!(EvidenceReceipt::from_json(&" ".repeat(16 * 1024 * 1024 + 1), &o).is_err());
    assert!(FreshnessOptions::default().with_byte_limit(1).is_err());
    assert!(SourceVersion::new("s", "\n").is_err());
}
#[cfg(feature = "execution")]
#[test]
fn interruption_is_an_error_without_partial_freshness_success() {
    use airust::execution::*;
    let t = CancellationToken::new();
    t.cancel();
    let o = FreshnessOptions::default().with_control(WorkControl::new(t));
    let f = facts();
    let r = rules();
    let s = [SourceVersion::new("s", "v1").unwrap()];
    assert!(matches!(
        verify_evidence(
            &receipt(),
            &EvidenceView::new("r2", &f, &r, &s).unwrap(),
            &o
        ),
        Err(FreshnessError::Execution(ExecutionError::Cancelled))
    ));
}
