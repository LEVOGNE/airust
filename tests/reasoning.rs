#![cfg(feature = "reasoning")]
use airust::reasoning::*;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
fn fact(id: &str, a: &str, b: &str) -> Fact {
    Fact::new(id, Statement::new(a, "edge", b).unwrap())
        .unwrap()
        .with_sources(["source".into()])
        .unwrap()
}
fn transitive() -> Rule {
    Rule::new(
        "transitive",
        vec![
            Pattern::new(Term::variable("a"), "edge", Term::variable("b")).unwrap(),
            Pattern::new(Term::variable("b"), "edge", Term::variable("c")).unwrap(),
        ],
        Pattern::new(Term::variable("a"), "edge", Term::variable("c")).unwrap(),
    )
    .unwrap()
}
fn options() -> ReasoningOptions {
    ReasoningOptions::new("revision-7", ["source".into()]).unwrap()
}
#[test]
fn proofs_are_acyclic_complete_and_inputs_unchanged() {
    let facts = [
        fact("ab", "a", "b"),
        fact("bc", "b", "c"),
        fact("cd", "c", "d"),
    ];
    let result = infer(&facts, &[transitive()], &options()).unwrap();
    assert!(result.complete());
    assert_eq!(result.revision(), "revision-7");
    let ad = result
        .proposals()
        .iter()
        .find(|p| p.statement.subject() == "a" && p.statement.object() == "d")
        .unwrap();
    assert_eq!(
        ad.fact_ids,
        BTreeSet::from(["ab".into(), "bc".into(), "cd".into()])
    );
    assert_eq!(ad.source_ids, BTreeSet::from(["source".into()]));
    for p in result.proposals() {
        for r in &p.premises {
            match r {
                PremiseRef::Fact(id) => assert!(facts.iter().any(|f| f.id() == id)),
                PremiseRef::Proposal(id) => assert!(*id < p.id),
            }
        }
    }
    assert_eq!(facts.len(), 3);
    assert_eq!(facts[0].statement().object(), "b");
    assert_eq!(
        result.to_json().unwrap(),
        infer(&facts, &[transitive()], &options())
            .unwrap()
            .to_json()
            .unwrap()
    );
    let rule = transitive();
    assert_eq!(rule.body().len(), 2);
    assert_eq!(rule.head().predicate(), "edge");
    for proposal in result.proposals() {
        assert_eq!(proposal.rule_fingerprint, rule.fingerprint().unwrap());
    }
}
#[test]
fn seeded_cyclic_graph_closure_matches_independent_bfs() {
    let mut seed = 19u64;
    for _ in 0..30 {
        let mut facts = Vec::new();
        let mut edges = BTreeSet::new();
        for a in 0..7 {
            for b in 0..7 {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                if seed.is_multiple_of(5) {
                    edges.insert((a, b));
                    facts.push(fact(&format!("e{a}-{b}"), &a.to_string(), &b.to_string()));
                }
            }
        }
        let result = infer(&facts, &[transitive()], &options()).unwrap();
        assert!(result.complete());
        let actual = edges
            .iter()
            .copied()
            .chain(result.proposals().iter().map(|p| {
                (
                    p.statement.subject().parse().unwrap(),
                    p.statement.object().parse().unwrap(),
                )
            }))
            .collect::<BTreeSet<(usize, usize)>>();
        let mut expected = BTreeSet::new();
        for start in 0..7 {
            let mut q = VecDeque::new();
            for &(a, b) in &edges {
                if a == start {
                    q.push_back(b);
                }
            }
            let mut seen = BTreeSet::new();
            while let Some(v) = q.pop_front() {
                if !seen.insert(v) {
                    continue;
                }
                expected.insert((start, v));
                for &(a, b) in &edges {
                    if a == v {
                        q.push_back(b);
                    }
                }
            }
        }
        assert_eq!(actual, expected);
    }
}
#[test]
fn hidden_bridge_never_enters_any_proof_or_output() {
    let facts = [
        fact("ab", "a", "b"),
        fact("hidden", "b", "secret"),
        fact("cd", "secret", "d"),
    ];
    let o = options()
        .with_allowed_facts(["ab".into(), "cd".into()])
        .unwrap();
    let r = infer(&facts, &[transitive()], &o).unwrap();
    assert!(r.proposals().is_empty());
    assert!(!r.to_json().unwrap().contains("hidden"));
    let empty = options().with_allowed_facts(Vec::<String>::new()).unwrap();
    assert!(infer(&facts, &[transitive()], &empty)
        .unwrap()
        .proposals()
        .is_empty());
}
#[test]
fn absent_or_unregistered_sources_are_explicit_and_excluded() {
    let unsourced = Fact::new("bc", Statement::new("b", "edge", "c").unwrap()).unwrap();
    let unknown = Fact::new("cd", Statement::new("c", "edge", "d").unwrap())
        .unwrap()
        .with_sources(["unknown".into()])
        .unwrap();
    let r = infer(
        &[fact("ab", "a", "b"), unsourced, unknown],
        &[transitive()],
        &options(),
    )
    .unwrap();
    assert_eq!(r.missing_support_count(), 2);
    assert_eq!(r.missing_support(), ["bc", "cd"]);
    assert!(!r.complete());
    assert!(r.proposals().is_empty());
}
#[test]
fn explicit_negative_conflict_is_reported_without_explosion() {
    let negative = Fact::new(
        "noac",
        Statement::new("a", "edge", "c")
            .unwrap()
            .with_negative(true),
    )
    .unwrap()
    .with_sources(["source".into()])
    .unwrap();
    let r = infer(
        &[fact("ab", "a", "b"), fact("bc", "b", "c"), negative],
        &[transitive()],
        &options(),
    )
    .unwrap();
    assert_eq!(r.conflicts().len(), 1);
    assert!(matches!(r.conflicts()[0].positive, PremiseRef::Proposal(_)));
    assert_eq!(r.conflicts()[0].negative, PremiseRef::Fact("noac".into()));
    assert_eq!(r.proposals().len(), 1);
}
#[test]
fn unsafe_rules_duplicates_and_invalid_limits_fail() {
    let p = Pattern::new(Term::variable("a"), "edge", Term::variable("b")).unwrap();
    let head = Pattern::new(Term::variable("unbound"), "edge", Term::variable("b")).unwrap();
    assert!(Rule::new("bad", vec![p.clone()], head).is_err());
    assert!(Rule::new("bad", vec![], p).is_err());
    assert!(options().with_limits(0, 10, 10, 1024).is_err());
    let f = fact("same", "a", "b");
    assert!(infer(&[f.clone(), f], &[], &options()).is_err());
    assert!(infer(&[], &[transitive(), transitive()], &options()).is_err());
    assert!(Statement::new("a\n", "p", "b").is_err());
}
#[test]
fn round_join_and_proposal_limits_never_claim_complete_closure() {
    let facts = [
        fact("ab", "a", "b"),
        fact("bc", "b", "c"),
        fact("cd", "c", "d"),
        fact("de", "d", "e"),
    ];
    for (rounds, joins, proposals, stop) in [
        (1, 10000, 100, StopReason::RoundLimit),
        (16, 1, 100, StopReason::JoinLimit),
        (16, 10000, 1, StopReason::ProposalLimit),
    ] {
        let o = options()
            .with_limits(rounds, joins, proposals, 1024 * 1024)
            .unwrap();
        let r = infer(&facts, &[transitive()], &o).unwrap();
        assert!(!r.complete());
        assert_eq!(r.stop_reason(), stop);
        assert!(r.join_trials() <= joins);
        assert!(r.proposals().len() <= proposals);
    }
}
#[test]
fn json_budget_trims_only_tail_proofs_and_remains_valid() {
    let facts = (0..10)
        .map(|i| fact(&format!("e{i}"), &i.to_string(), &(i + 1).to_string()))
        .collect::<Vec<_>>();
    let r = infer(
        &facts,
        &[transitive()],
        &options().with_limits(16, 1000000, 1000, 1024).unwrap(),
    )
    .unwrap();
    assert!(!r.complete());
    assert_eq!(r.stop_reason(), StopReason::OutputLimit);
    assert!(r.to_json().unwrap().len() <= 1024);
    let available = r.proposals().iter().map(|p| p.id).collect::<BTreeSet<_>>();
    for p in r.proposals() {
        for r in &p.premises {
            if let PremiseRef::Proposal(id) = r {
                assert!(available.contains(id) && *id < p.id);
            }
        }
    }
}
#[test]
fn constants_and_repeated_variables_use_safe_unification() {
    let p = Pattern::new(Term::variable("x"), "edge", Term::variable("x")).unwrap();
    let head = Pattern::new(Term::variable("x"), "cycle", Term::constant("yes")).unwrap();
    let rule = Rule::new("self", vec![p], head).unwrap();
    let r = infer(
        &[fact("ab", "a", "b"), fact("aa", "a", "a")],
        &[rule],
        &options(),
    )
    .unwrap();
    assert_eq!(r.proposals().len(), 1);
    assert_eq!(r.proposals()[0].fact_ids, BTreeSet::from(["aa".into()]));
}
#[cfg(feature = "execution")]
#[test]
fn controlled_reasoning_reports_work_and_interruption_has_no_partial_success() {
    use airust::execution::*;
    use std::sync::{Arc, Mutex};
    let token = CancellationToken::new();
    token.cancel();
    assert!(matches!(
        infer(
            &[fact("ab", "a", "b")],
            &[transitive()],
            &options().with_control(WorkControl::new(token))
        ),
        Err(ReasoningError::Execution(ExecutionError::Cancelled))
    ));
    let c = WorkControl::default()
        .with_timeout(std::time::Duration::ZERO)
        .unwrap();
    assert!(matches!(
        infer(&[], &[], &options().with_control(c)),
        Err(ReasoningError::Execution(ExecutionError::DeadlineExceeded))
    ));
    let events = Arc::new(Mutex::new(Vec::new()));
    let copy = events.clone();
    let c = WorkControl::default().with_observer(move |e: &ProgressEvent| {
        copy.lock().unwrap().push(*e);
    });
    infer(
        &[fact("ab", "a", "b"), fact("bc", "b", "c")],
        &[transitive()],
        &options().with_control(c),
    )
    .unwrap();
    assert!(events
        .lock()
        .unwrap()
        .iter()
        .any(|e| e.phase() == WorkPhase::Reasoning));
}
#[test]
fn every_derived_proof_has_registered_sources_and_real_base_facts() {
    let facts = [
        fact("1", "a", "b"),
        fact("2", "b", "c"),
        fact("3", "c", "a"),
    ];
    let map = facts
        .iter()
        .map(|f| (f.id(), f))
        .collect::<BTreeMap<_, _>>();
    let r = infer(&facts, &[transitive()], &options()).unwrap();
    assert!(r.complete());
    for p in r.proposals() {
        assert!(!p.fact_ids.is_empty());
        for id in &p.fact_ids {
            assert!(map.contains_key(id.as_str()));
        }
        assert_eq!(p.source_ids, BTreeSet::from(["source".into()]));
    }
}
