#![doc = include_str!("../docs/REASONING.md")]
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ReasoningError {
    #[error("invalid reasoning input/configuration: {0}")]
    Invalid(&'static str),
    #[error("reasoning input limit exceeded: {0}")]
    Budget(&'static str),
    #[cfg(feature = "execution")]
    #[error(transparent)]
    Execution(#[from] crate::execution::ExecutionError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Statement {
    subject: String,
    predicate: String,
    object: String,
    negative: bool,
}
impl Statement {
    pub fn new(
        subject: impl Into<String>,
        predicate: impl Into<String>,
        object: impl Into<String>,
    ) -> Result<Self, ReasoningError> {
        let s = Self {
            subject: subject.into(),
            predicate: predicate.into(),
            object: object.into(),
            negative: false,
        };
        s.validate()?;
        Ok(s)
    }
    pub fn with_negative(mut self, negative: bool) -> Self {
        self.negative = negative;
        self
    }
    pub fn subject(&self) -> &str {
        &self.subject
    }
    pub fn predicate(&self) -> &str {
        &self.predicate
    }
    pub fn object(&self) -> &str {
        &self.object
    }
    pub fn negative(&self) -> bool {
        self.negative
    }
    fn validate(&self) -> Result<(), ReasoningError> {
        if [&self.subject, &self.predicate, &self.object]
            .iter()
            .any(|v| !label(v, 256))
        {
            return Err(ReasoningError::Invalid("statement label"));
        }
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct Fact {
    id: String,
    statement: Statement,
    sources: BTreeSet<String>,
}
impl Fact {
    pub fn new(id: impl Into<String>, statement: Statement) -> Result<Self, ReasoningError> {
        let f = Self {
            id: id.into(),
            statement,
            sources: BTreeSet::new(),
        };
        f.validate()?;
        Ok(f)
    }
    pub fn with_sources(
        mut self,
        sources: impl IntoIterator<Item = String>,
    ) -> Result<Self, ReasoningError> {
        self.sources = sources.into_iter().collect();
        self.validate()?;
        Ok(self)
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn statement(&self) -> &Statement {
        &self.statement
    }
    pub fn sources(&self) -> &BTreeSet<String> {
        &self.sources
    }
    fn validate(&self) -> Result<(), ReasoningError> {
        self.statement.validate()?;
        if !label(&self.id, 128)
            || self.sources.len() > 64
            || self.sources.iter().any(|s| !label(s, 128))
        {
            return Err(ReasoningError::Invalid("fact identity/sources"));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum Term {
    Constant(String),
    Variable(String),
}
impl Term {
    pub fn constant(value: impl Into<String>) -> Self {
        Self::Constant(value.into())
    }
    pub fn variable(value: impl Into<String>) -> Self {
        Self::Variable(value.into())
    }
    fn validate(&self) -> bool {
        match self {
            Self::Constant(s) => label(s, 256),
            Self::Variable(s) => label(s, 64),
        }
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct Pattern {
    subject: Term,
    predicate: String,
    object: Term,
    negative: bool,
}
impl Pattern {
    pub fn new(
        subject: Term,
        predicate: impl Into<String>,
        object: Term,
    ) -> Result<Self, ReasoningError> {
        let p = Self {
            subject,
            predicate: predicate.into(),
            object,
            negative: false,
        };
        p.validate()?;
        Ok(p)
    }
    pub fn with_negative(mut self, negative: bool) -> Self {
        self.negative = negative;
        self
    }
    pub fn subject(&self) -> &Term {
        &self.subject
    }
    pub fn predicate(&self) -> &str {
        &self.predicate
    }
    pub fn object(&self) -> &Term {
        &self.object
    }
    pub fn negative(&self) -> bool {
        self.negative
    }
    fn validate(&self) -> Result<(), ReasoningError> {
        if !self.subject.validate() || !self.object.validate() || !label(&self.predicate, 256) {
            return Err(ReasoningError::Invalid("rule pattern"));
        }
        Ok(())
    }
    fn variables(&self) -> impl Iterator<Item = &String> {
        [&self.subject, &self.object]
            .into_iter()
            .filter_map(|t| match t {
                Term::Variable(v) => Some(v),
                _ => None,
            })
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct Rule {
    id: String,
    body: Vec<Pattern>,
    head: Pattern,
}
impl Rule {
    pub fn new(
        id: impl Into<String>,
        body: Vec<Pattern>,
        head: Pattern,
    ) -> Result<Self, ReasoningError> {
        let r = Self {
            id: id.into(),
            body,
            head,
        };
        r.validate()?;
        Ok(r)
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn body(&self) -> &[Pattern] {
        &self.body
    }
    pub fn head(&self) -> &Pattern {
        &self.head
    }
    pub fn to_json(&self) -> Result<String, ReasoningError> {
        Ok(serde_json::to_string(self)?)
    }
    pub fn fingerprint(&self) -> Result<String, ReasoningError> {
        let mut hash = Sha256::new();
        hash.update(b"airust-rule-v1");
        hash.update(self.to_json()?.as_bytes());
        Ok(format!("{:x}", hash.finalize()))
    }
    fn validate(&self) -> Result<(), ReasoningError> {
        if !label(&self.id, 128) || self.body.is_empty() || self.body.len() > 8 {
            return Err(ReasoningError::Invalid("rule identity/premise count"));
        }
        self.head.validate()?;
        for p in &self.body {
            p.validate()?;
        }
        let bound = self
            .body
            .iter()
            .flat_map(Pattern::variables)
            .collect::<BTreeSet<_>>();
        if bound.len() > 16 || self.head.variables().any(|v| !bound.contains(v)) {
            return Err(ReasoningError::Invalid("unsafe/unbounded rule variables"));
        }
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct ReasoningOptions {
    revision: String,
    known_sources: BTreeSet<String>,
    allowed: Option<BTreeSet<String>>,
    max_rounds: usize,
    max_joins: usize,
    max_proposals: usize,
    max_json_bytes: usize,
    #[cfg(feature = "execution")]
    control: Option<crate::execution::WorkControl>,
}
impl Default for ReasoningOptions {
    fn default() -> Self {
        Self {
            revision: "unspecified".into(),
            known_sources: BTreeSet::new(),
            allowed: None,
            max_rounds: 16,
            max_joins: 1000000,
            max_proposals: 1000,
            max_json_bytes: 1024 * 1024,
            #[cfg(feature = "execution")]
            control: None,
        }
    }
}
impl ReasoningOptions {
    pub fn new(
        revision: impl Into<String>,
        known_sources: impl IntoIterator<Item = String>,
    ) -> Result<Self, ReasoningError> {
        let o = Self {
            revision: revision.into(),
            known_sources: known_sources.into_iter().collect(),
            ..Self::default()
        };
        o.validate()?;
        Ok(o)
    }
    pub fn with_allowed_facts(
        mut self,
        ids: impl IntoIterator<Item = String>,
    ) -> Result<Self, ReasoningError> {
        self.allowed = Some(ids.into_iter().collect());
        self.validate()?;
        Ok(self)
    }
    pub fn with_limits(
        mut self,
        rounds: usize,
        joins: usize,
        proposals: usize,
        json_bytes: usize,
    ) -> Result<Self, ReasoningError> {
        self.max_rounds = rounds;
        self.max_joins = joins;
        self.max_proposals = proposals;
        self.max_json_bytes = json_bytes;
        self.validate()?;
        Ok(self)
    }
    #[cfg(feature = "execution")]
    pub fn with_control(mut self, control: crate::execution::WorkControl) -> Self {
        self.control = Some(control);
        self
    }
    fn validate(&self) -> Result<(), ReasoningError> {
        if !label(&self.revision, 256)
            || self.known_sources.len() > 100000
            || self.known_sources.iter().any(|s| !label(s, 128))
            || self
                .allowed
                .as_ref()
                .is_some_and(|v| v.len() > 100000 || v.iter().any(|s| !label(s, 128)))
            || !(1..=100).contains(&self.max_rounds)
            || !(1..=10000000).contains(&self.max_joins)
            || !(1..=10000).contains(&self.max_proposals)
            || !(1024..=16 * 1024 * 1024).contains(&self.max_json_bytes)
        {
            return Err(ReasoningError::Invalid("reasoning options"));
        }
        Ok(())
    }
    fn check(&self) -> Result<(), ReasoningError> {
        #[cfg(feature = "execution")]
        if let Some(c) = &self.control {
            c.checkpoint()?;
        }
        Ok(())
    }
    fn progress(&self, round: usize) {
        #[cfg(feature = "execution")]
        if let Some(c) = &self.control {
            use crate::execution::*;
            c.report(ProgressEvent::new(WorkPhase::Reasoning, round, None).expect("unknown total"));
        }
        #[cfg(not(feature = "execution"))]
        let _ = round;
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum PremiseRef {
    Fact(String),
    Proposal(usize),
}
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct Proposal {
    pub id: usize,
    pub statement: Statement,
    pub rule_id: String,
    pub rule_fingerprint: String,
    pub premises: Vec<PremiseRef>,
    pub fact_ids: BTreeSet<String>,
    pub source_ids: BTreeSet<String>,
}
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct Conflict {
    pub positive: PremiseRef,
    pub negative: PremiseRef,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum StopReason {
    FixedPoint,
    RoundLimit,
    JoinLimit,
    ProposalLimit,
    ProofLimit,
    OutputLimit,
}
#[derive(Debug, Clone, Serialize)]
pub struct ReasoningResult {
    schema_version: u32,
    revision: String,
    complete: bool,
    stop: StopReason,
    rounds: usize,
    join_trials: usize,
    missing_support_count: usize,
    missing_support: Vec<String>,
    proposals: Vec<Proposal>,
    conflicts: Vec<Conflict>,
}
impl ReasoningResult {
    pub fn revision(&self) -> &str {
        &self.revision
    }
    pub fn complete(&self) -> bool {
        self.complete
    }
    pub fn stop_reason(&self) -> StopReason {
        self.stop
    }
    pub fn rounds(&self) -> usize {
        self.rounds
    }
    pub fn join_trials(&self) -> usize {
        self.join_trials
    }
    pub fn missing_support_count(&self) -> usize {
        self.missing_support_count
    }
    pub fn missing_support(&self) -> &[String] {
        &self.missing_support
    }
    pub fn proposals(&self) -> &[Proposal] {
        &self.proposals
    }
    pub fn conflicts(&self) -> &[Conflict] {
        &self.conflicts
    }
    pub fn to_json(&self) -> Result<String, ReasoningError> {
        Ok(serde_json::to_string(self)?)
    }
}
#[derive(Clone)]
struct Node {
    statement: Statement,
    reference: PremiseRef,
    facts: BTreeSet<String>,
    sources: BTreeSet<String>,
}
pub fn infer(
    facts: &[Fact],
    rules: &[Rule],
    options: &ReasoningOptions,
) -> Result<ReasoningResult, ReasoningError> {
    options.validate()?;
    options.check()?;
    if facts.len() > 100000 || rules.len() > 256 {
        return Err(ReasoningError::Budget("fact/rule count"));
    }
    let mut ids = BTreeSet::new();
    let mut bytes = 0usize;
    for f in facts {
        options.check()?;
        f.validate()?;
        if !ids.insert(&f.id) {
            return Err(ReasoningError::Invalid("duplicate fact ID"));
        }
        bytes += f.id.len()
            + f.statement.subject.len()
            + f.statement.predicate.len()
            + f.statement.object.len()
            + f.sources.iter().map(String::len).sum::<usize>();
    }
    let mut rule_ids = BTreeSet::new();
    for r in rules {
        r.validate()?;
        if !rule_ids.insert(&r.id) {
            return Err(ReasoningError::Invalid("duplicate rule ID"));
        }
        bytes += r.id.len()
            + r.body
                .iter()
                .chain([&r.head])
                .map(|p| p.predicate.len() + term_len(&p.subject) + term_len(&p.object))
                .sum::<usize>();
    }
    if bytes > 16 * 1024 * 1024 {
        return Err(ReasoningError::Budget("input bytes"));
    }
    let mut nodes = Vec::new();
    let mut missing = Vec::new();
    let mut missing_count = 0;
    let mut ordered = facts.iter().collect::<Vec<_>>();
    ordered.sort_by(|a, b| a.id.cmp(&b.id));
    for (position, f) in ordered.into_iter().enumerate() {
        if position.is_multiple_of(256) {
            options.check()?;
        }
        if options.allowed.as_ref().is_some_and(|a| !a.contains(&f.id)) {
            continue;
        }
        if f.sources.is_empty() || f.sources.iter().any(|s| !options.known_sources.contains(s)) {
            missing_count += 1;
            if missing.len() < 100 {
                missing.push(f.id.clone());
            }
            continue;
        }
        nodes.push(Node {
            statement: f.statement.clone(),
            reference: PremiseRef::Fact(f.id.clone()),
            facts: BTreeSet::from([f.id.clone()]),
            sources: f.sources.clone(),
        });
    }
    let mut result = ReasoningResult {
        schema_version: 1,
        revision: options.revision.clone(),
        complete: false,
        stop: StopReason::RoundLimit,
        rounds: 0,
        join_trials: 0,
        missing_support_count: missing_count,
        missing_support: missing,
        proposals: Vec::new(),
        conflicts: Vec::new(),
    };
    let mut seen = nodes
        .iter()
        .map(|n| n.statement.clone())
        .collect::<BTreeSet<_>>();
    let mut ordered_rules = rules.iter().collect::<Vec<_>>();
    ordered_rules.sort_by(|a, b| a.id.cmp(&b.id));
    options.progress(0);
    for round in 1..=options.max_rounds {
        options.check()?;
        let mut index = BTreeMap::<(&str, bool), Vec<usize>>::new();
        for (i, n) in nodes.iter().enumerate() {
            if i.is_multiple_of(256) {
                options.check()?;
            }
            index
                .entry((&n.statement.predicate, n.statement.negative))
                .or_default()
                .push(i);
        }
        let mut pending = Vec::new();
        let mut stop = None;
        for rule in &ordered_rules {
            let mut walk = Walk {
                nodes: &nodes,
                index: &index,
                options,
                result: &mut result,
                pending: &mut pending,
                seen: &mut seen,
            };
            if let Some(s) = walk.apply(rule, 0, &BTreeMap::new(), &[])? {
                stop = Some(s);
                break;
            }
        }
        let added = pending.len();
        result.rounds = round;
        nodes.extend(pending);
        options.progress(round);
        if let Some(s) = stop {
            result.stop = s;
            break;
        }
        if added == 0 {
            result.complete = missing_count == 0;
            result.stop = StopReason::FixedPoint;
            break;
        }
    }
    let mut by_statement = BTreeMap::<Statement, PremiseRef>::new();
    for (position, n) in nodes.iter().enumerate() {
        if position.is_multiple_of(256) {
            options.check()?;
        }
        by_statement
            .entry(n.statement.clone())
            .or_insert_with(|| n.reference.clone());
    }
    for (position, (s, reference)) in by_statement.iter().enumerate() {
        if position.is_multiple_of(256) {
            options.check()?;
        }
        if s.negative {
            continue;
        }
        let mut opposite = s.clone();
        opposite.negative = true;
        if let Some(negative) = by_statement.get(&opposite) {
            result.conflicts.push(Conflict {
                positive: reference.clone(),
                negative: negative.clone(),
            });
        }
    }
    options.check()?;
    if !fits_json(&result, options.max_json_bytes)? {
        result.complete = false;
        result.stop = StopReason::OutputLimit;
        result.missing_support.clear();
        result.conflicts.clear();
        let proposals = std::mem::take(&mut result.proposals);
        let mut bytes = result.to_json()?.len();
        if bytes > options.max_json_bytes {
            return Err(ReasoningError::Budget("response bytes"));
        }
        let mut retained = 0;
        for proposal in &proposals {
            options.check()?;
            let addition = serde_json::to_vec(proposal)?.len() + usize::from(retained > 0);
            if addition > options.max_json_bytes - bytes {
                break;
            }
            bytes += addition;
            retained += 1;
        }
        result.proposals = proposals;
        result.proposals.truncate(retained);
        if !fits_json(&result, options.max_json_bytes)? {
            return Err(ReasoningError::Budget("response bytes"));
        }
    }
    options.check()?;
    Ok(result)
}
struct Walk<'a> {
    nodes: &'a [Node],
    index: &'a BTreeMap<(&'a str, bool), Vec<usize>>,
    options: &'a ReasoningOptions,
    result: &'a mut ReasoningResult,
    pending: &'a mut Vec<Node>,
    seen: &'a mut BTreeSet<Statement>,
}
impl Walk<'_> {
    fn apply(
        &mut self,
        rule: &Rule,
        depth: usize,
        bindings: &BTreeMap<String, String>,
        premises: &[usize],
    ) -> Result<Option<StopReason>, ReasoningError> {
        self.options.check()?;
        if depth == rule.body.len() {
            let statement = Statement {
                subject: resolve(&rule.head.subject, bindings).expect("safe head"),
                predicate: rule.head.predicate.clone(),
                object: resolve(&rule.head.object, bindings).expect("safe head"),
                negative: rule.head.negative,
            };
            if self.seen.contains(&statement) {
                return Ok(None);
            }
            if self.result.proposals.len() >= self.options.max_proposals {
                return Ok(Some(StopReason::ProposalLimit));
            }
            let mut facts = BTreeSet::new();
            let mut sources = BTreeSet::new();
            for i in premises {
                facts.extend(self.nodes[*i].facts.iter().cloned());
                sources.extend(self.nodes[*i].sources.iter().cloned());
            }
            if facts.len() > 256 || sources.len() > 64 {
                return Ok(Some(StopReason::ProofLimit));
            }
            let id = self.result.proposals.len() + 1;
            let refs = premises
                .iter()
                .map(|i| self.nodes[*i].reference.clone())
                .collect();
            self.result.proposals.push(Proposal {
                id,
                statement: statement.clone(),
                rule_id: rule.id.clone(),
                rule_fingerprint: rule.fingerprint()?,
                premises: refs,
                fact_ids: facts.clone(),
                source_ids: sources.clone(),
            });
            self.pending.push(Node {
                statement: statement.clone(),
                reference: PremiseRef::Proposal(id),
                facts,
                sources,
            });
            self.seen.insert(statement);
            return Ok(None);
        }
        let pattern = &rule.body[depth];
        let index = self.index;
        let candidates = index
            .get(&(pattern.predicate.as_str(), pattern.negative))
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        for &i in candidates {
            if self.result.join_trials >= self.options.max_joins {
                return Ok(Some(StopReason::JoinLimit));
            }
            self.result.join_trials += 1;
            if self.result.join_trials.is_multiple_of(256) {
                self.options.check()?;
            }
            let n = &self.nodes[i];
            let mut next = bindings.clone();
            if !unify(&pattern.subject, &n.statement.subject, &mut next)
                || !unify(&pattern.object, &n.statement.object, &mut next)
            {
                continue;
            }
            let mut selected = premises.to_vec();
            selected.push(i);
            if let Some(s) = self.apply(rule, depth + 1, &next, &selected)? {
                return Ok(Some(s));
            }
        }
        Ok(None)
    }
}
fn unify(term: &Term, value: &str, bindings: &mut BTreeMap<String, String>) -> bool {
    match term {
        Term::Constant(c) => c == value,
        Term::Variable(v) => {
            if let Some(old) = bindings.get(v) {
                old == value
            } else {
                bindings.insert(v.clone(), value.into());
                true
            }
        }
    }
}
fn resolve(term: &Term, bindings: &BTreeMap<String, String>) -> Option<String> {
    match term {
        Term::Constant(c) => Some(c.clone()),
        Term::Variable(v) => bindings.get(v).cloned(),
    }
}
fn term_len(term: &Term) -> usize {
    match term {
        Term::Constant(s) | Term::Variable(s) => s.len(),
    }
}
fn label(s: &str, max: usize) -> bool {
    !s.trim().is_empty() && s.len() <= max && !s.chars().any(char::is_control)
}

fn fits_json(value: &ReasoningResult, limit: usize) -> Result<bool, ReasoningError> {
    struct Counter {
        remaining: usize,
        exceeded: bool,
    }
    impl std::io::Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > self.remaining {
                self.exceeded = true;
                return Err(std::io::Error::other("JSON limit"));
            }
            self.remaining -= bytes.len();
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut counter = Counter {
        remaining: limit,
        exceeded: false,
    };
    let result = serde_json::to_writer(&mut counter, value);
    if counter.exceeded {
        return Ok(false);
    }
    result?;
    Ok(true)
}
