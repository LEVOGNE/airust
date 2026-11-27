#![doc = include_str!("../docs/EVIDENCE-FRESHNESS.md")]
use crate::reasoning::{Fact, Pattern, PremiseRef, ReasoningResult, Rule, Statement, Term};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum FreshnessError {
    #[error("invalid evidence receipt/view: {0}")]
    Invalid(&'static str),
    #[error("evidence receipt budget exceeded")]
    Budget,
    #[cfg(feature = "execution")]
    #[error(transparent)]
    Execution(#[from] crate::execution::ExecutionError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
#[derive(Debug, Clone)]
pub struct SourceVersion {
    id: String,
    version: String,
}
impl SourceVersion {
    pub fn new(id: impl Into<String>, version: impl Into<String>) -> Result<Self, FreshnessError> {
        let s = Self {
            id: id.into(),
            version: version.into(),
        };
        if !crate::text::Identifier::label(&s.id, 128)
            || !crate::text::Identifier::label(&s.version, 256)
        {
            return Err(FreshnessError::Invalid("source version"));
        }
        Ok(s)
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn version(&self) -> &str {
        &self.version
    }
}
pub struct EvidenceView<'a> {
    revision: &'a str,
    facts: &'a [Fact],
    rules: &'a [Rule],
    sources: &'a [SourceVersion],
    allowed: Option<&'a BTreeSet<String>>,
}
impl<'a> EvidenceView<'a> {
    pub fn new(
        revision: &'a str,
        facts: &'a [Fact],
        rules: &'a [Rule],
        sources: &'a [SourceVersion],
    ) -> Result<Self, FreshnessError> {
        if !crate::text::Identifier::label(revision, 256)
            || facts.len() > 100000
            || rules.len() > 256
            || sources.len() > 100000
        {
            return Err(FreshnessError::Invalid("view identity/count"));
        }
        Ok(Self {
            revision,
            facts,
            rules,
            sources,
            allowed: None,
        })
    }
    pub fn with_allowed_facts(mut self, ids: &'a BTreeSet<String>) -> Result<Self, FreshnessError> {
        if ids.len() > 100000 || ids.iter().any(|s| !crate::text::Identifier::label(s, 128)) {
            return Err(FreshnessError::Invalid("allowed IDs"));
        }
        self.allowed = Some(ids);
        Ok(self)
    }
}
#[derive(Debug, Clone)]
pub struct FreshnessOptions {
    max_bytes: usize,
    #[cfg(feature = "execution")]
    control: Option<crate::execution::WorkControl>,
}
impl Default for FreshnessOptions {
    fn default() -> Self {
        Self {
            max_bytes: 16 * 1024 * 1024,
            #[cfg(feature = "execution")]
            control: None,
        }
    }
}
impl FreshnessOptions {
    pub fn with_byte_limit(mut self, bytes: usize) -> Result<Self, FreshnessError> {
        if !(1024..=64 * 1024 * 1024).contains(&bytes) {
            return Err(FreshnessError::Budget);
        }
        self.max_bytes = bytes;
        Ok(self)
    }
    #[cfg(feature = "execution")]
    pub fn with_control(mut self, c: crate::execution::WorkControl) -> Self {
        self.control = Some(c);
        self
    }
    fn check(&self) -> Result<(), FreshnessError> {
        #[cfg(feature = "execution")]
        if let Some(c) = &self.control {
            c.checkpoint()?;
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {
    id: String,
    value: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProofRecord {
    id: usize,
    conclusion: String,
    facts: Vec<Binding>,
    rules: Vec<Binding>,
    sources: Vec<Binding>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReceiptData {
    schema_version: u32,
    revision: String,
    closure_complete: bool,
    proofs: Vec<ProofRecord>,
}
#[derive(Debug, Clone)]
pub struct EvidenceReceipt {
    data: ReceiptData,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    checksum: String,
    data: ReceiptData,
}
impl EvidenceReceipt {
    pub fn len(&self) -> usize {
        self.data.proofs.len()
    }
    pub fn is_empty(&self) -> bool {
        self.data.proofs.is_empty()
    }
    pub fn revision(&self) -> &str {
        &self.data.revision
    }
    pub fn fingerprint(&self) -> Result<String, FreshnessError> {
        Ok(hash(&serde_json::to_vec(&self.data)?))
    }
    pub fn to_json(&self, options: &FreshnessOptions) -> Result<String, FreshnessError> {
        options.check()?;
        let json = serde_json::to_string(&Envelope {
            checksum: self.fingerprint()?,
            data: self.data.clone(),
        })?;
        if json.len() > options.max_bytes {
            return Err(FreshnessError::Budget);
        }
        Ok(json)
    }
    pub fn from_json(json: &str, options: &FreshnessOptions) -> Result<Self, FreshnessError> {
        options.check()?;
        if json.len() > options.max_bytes {
            return Err(FreshnessError::Budget);
        }
        let wire: Envelope = serde_json::from_str(json).map_err(crate::input::json_error)?;
        validate_data(&wire.data)?;
        if hash(&serde_json::to_vec(&wire.data)?) != wire.checksum {
            return Err(FreshnessError::Invalid("checksum"));
        }
        options.check()?;
        Ok(Self { data: wire.data })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum FreshnessState {
    Current,
    Stale,
    Missing,
    NotAllowed,
}
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct FreshnessFinding {
    pub kind: String,
    pub id: String,
}
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct ProofFreshness {
    pub proposal_id: usize,
    pub state: FreshnessState,
    pub findings: Vec<FreshnessFinding>,
}
#[derive(Debug, Clone, Serialize)]
pub struct FreshnessReport {
    receipt_revision: String,
    current_revision: String,
    closure_was_complete: bool,
    proofs: Vec<ProofFreshness>,
}
impl FreshnessReport {
    pub fn proofs(&self) -> &[ProofFreshness] {
        &self.proofs
    }
    pub fn all_current(&self) -> bool {
        !self.proofs.is_empty()
            && self
                .proofs
                .iter()
                .all(|p| p.state == FreshnessState::Current)
    }
    pub fn to_json(&self) -> Result<String, FreshnessError> {
        Ok(serde_json::to_string(self)?)
    }
}
struct Indexed<'a> {
    facts: BTreeMap<&'a str, &'a Fact>,
    rules: BTreeMap<&'a str, &'a Rule>,
    sources: BTreeMap<&'a str, &'a str>,
}
fn index<'a>(
    view: &EvidenceView<'a>,
    options: &FreshnessOptions,
) -> Result<Indexed<'a>, FreshnessError> {
    let mut ix = Indexed {
        facts: BTreeMap::new(),
        rules: BTreeMap::new(),
        sources: BTreeMap::new(),
    };
    for f in view.facts {
        options.check()?;
        if ix.facts.insert(f.id(), f).is_some() {
            return Err(FreshnessError::Invalid("duplicate fact"));
        }
    }
    for r in view.rules {
        if ix.rules.insert(r.id(), r).is_some() {
            return Err(FreshnessError::Invalid("duplicate rule"));
        }
    }
    for s in view.sources {
        if ix.sources.insert(&s.id, &s.version).is_some() {
            return Err(FreshnessError::Invalid("duplicate source"));
        }
    }
    Ok(ix)
}
pub fn capture_evidence(
    result: &ReasoningResult,
    view: &EvidenceView<'_>,
    options: &FreshnessOptions,
) -> Result<EvidenceReceipt, FreshnessError> {
    options.check()?;
    if result.revision() != view.revision {
        return Err(FreshnessError::Invalid("capture revision mismatch"));
    }
    let ix = index(view, options)?;
    let mut records = BTreeMap::<usize, (Statement, ProofRecord)>::new();
    for p in result.proposals() {
        options.check()?;
        if p.id != records.len() + 1 {
            return Err(FreshnessError::Invalid("proposal order"));
        }
        let rule = ix
            .rules
            .get(p.rule_id.as_str())
            .ok_or(FreshnessError::Invalid("missing capture rule"))?;
        if rule
            .fingerprint()
            .map_err(|_| FreshnessError::Invalid("rule hash"))?
            != p.rule_fingerprint
            || rule.body().len() != p.premises.len()
        {
            return Err(FreshnessError::Invalid("capture rule identity/arity"));
        }
        let mut facts = BTreeMap::new();
        let mut sources = BTreeMap::new();
        let mut rules = BTreeMap::from([(p.rule_id.clone(), p.rule_fingerprint.clone())]);
        let mut bindings = BTreeMap::new();
        for (pattern, reference) in rule.body().iter().zip(&p.premises) {
            let statement = match reference {
                PremiseRef::Fact(id) => {
                    if view.allowed.is_some_and(|a| !a.contains(id)) {
                        return Err(FreshnessError::Invalid("capture scope"));
                    }
                    let f = ix
                        .facts
                        .get(id.as_str())
                        .ok_or(FreshnessError::Invalid("missing capture fact"))?;
                    if f.sources().is_empty() {
                        return Err(FreshnessError::Invalid("unsupported capture fact"));
                    }
                    facts.insert(id.clone(), fact_hash(f)?);
                    for sid in f.sources() {
                        sources.insert(
                            sid.clone(),
                            ix.sources
                                .get(sid.as_str())
                                .ok_or(FreshnessError::Invalid("missing source version"))?
                                .to_string(),
                        );
                    }
                    f.statement().clone()
                }
                PremiseRef::Proposal(id) => {
                    let (s, r) = records
                        .get(id)
                        .filter(|_| *id < p.id)
                        .ok_or(FreshnessError::Invalid("forward/missing proof"))?;
                    merge(&mut facts, &r.facts);
                    merge(&mut rules, &r.rules);
                    merge(&mut sources, &r.sources);
                    s.clone()
                }
            };
            if !matches(pattern, &statement, &mut bindings) {
                return Err(FreshnessError::Invalid("premise replay"));
            }
        }
        if resolve(rule.head().subject(), &bindings) != Some(p.statement.subject())
            || resolve(rule.head().object(), &bindings) != Some(p.statement.object())
            || rule.head().predicate() != p.statement.predicate()
            || rule.head().negative() != p.statement.negative()
            || facts.keys().cloned().collect::<BTreeSet<_>>() != p.fact_ids
            || sources.keys().cloned().collect::<BTreeSet<_>>() != p.source_ids
        {
            return Err(FreshnessError::Invalid("conclusion/dependency replay"));
        }
        let record = ProofRecord {
            id: p.id,
            conclusion: hash(&serde_json::to_vec(&p.statement)?),
            facts: bindings_vec(facts),
            rules: bindings_vec(rules),
            sources: bindings_vec(sources),
        };
        records.insert(p.id, (p.statement.clone(), record));
    }
    let receipt = EvidenceReceipt {
        data: ReceiptData {
            schema_version: 1,
            revision: view.revision.into(),
            closure_complete: result.complete(),
            proofs: records.into_values().map(|(_, r)| r).collect(),
        },
    };
    validate_data(&receipt.data)?;
    receipt.to_json(options)?;
    options.check()?;
    Ok(receipt)
}
pub fn verify_evidence(
    receipt: &EvidenceReceipt,
    view: &EvidenceView<'_>,
    options: &FreshnessOptions,
) -> Result<FreshnessReport, FreshnessError> {
    options.check()?;
    validate_data(&receipt.data)?;
    let ix = index(view, options)?;
    let mut proofs = Vec::new();
    for p in &receipt.data.proofs {
        options.check()?;
        let mut state = FreshnessState::Current;
        let mut findings = Vec::new();
        let denied = p
            .facts
            .iter()
            .filter(|b| view.allowed.is_some_and(|a| !a.contains(&b.id)))
            .collect::<Vec<_>>();
        if !denied.is_empty() {
            for b in denied {
                findings.push(FreshnessFinding {
                    kind: "not_allowed".into(),
                    id: b.id.clone(),
                });
            }
            state = FreshnessState::NotAllowed;
        } else {
            for b in &p.facts {
                let actual = ix
                    .facts
                    .get(b.id.as_str())
                    .map(|f| fact_hash(f))
                    .transpose()?;
                check_binding(b, actual.as_deref(), "fact", &mut state, &mut findings);
            }
            for b in &p.rules {
                let actual = ix
                    .rules
                    .get(b.id.as_str())
                    .map(|r| {
                        r.fingerprint()
                            .map_err(|_| FreshnessError::Invalid("rule hash"))
                    })
                    .transpose()?;
                check_binding(b, actual.as_deref(), "rule", &mut state, &mut findings);
            }
            for b in &p.sources {
                check_binding(
                    b,
                    ix.sources.get(b.id.as_str()).copied(),
                    "source",
                    &mut state,
                    &mut findings,
                );
            }
        }
        proofs.push(ProofFreshness {
            proposal_id: p.id,
            state,
            findings,
        });
    }
    options.check()?;
    Ok(FreshnessReport {
        receipt_revision: receipt.data.revision.clone(),
        current_revision: view.revision.into(),
        closure_was_complete: receipt.data.closure_complete,
        proofs,
    })
}
fn check_binding(
    b: &Binding,
    actual: Option<&str>,
    kind: &str,
    state: &mut FreshnessState,
    out: &mut Vec<FreshnessFinding>,
) {
    let issue = match actual {
        None => Some((FreshnessState::Missing, "missing")),
        Some(v) if v != b.value => Some((FreshnessState::Stale, "changed")),
        _ => None,
    };
    if let Some((s, message)) = issue {
        *state = (*state).max(s);
        out.push(FreshnessFinding {
            kind: format!("{kind}_{message}"),
            id: b.id.clone(),
        });
    }
}
fn matches(p: &Pattern, s: &Statement, b: &mut BTreeMap<String, String>) -> bool {
    p.predicate() == s.predicate()
        && p.negative() == s.negative()
        && unify(p.subject(), s.subject(), b)
        && unify(p.object(), s.object(), b)
}
fn unify(t: &Term, v: &str, b: &mut BTreeMap<String, String>) -> bool {
    match t {
        Term::Constant(c) => c == v,
        Term::Variable(k) => match b.get(k) {
            Some(old) => old == v,
            None => {
                b.insert(k.clone(), v.into());
                true
            }
        },
    }
}
fn resolve<'a>(t: &'a Term, b: &'a BTreeMap<String, String>) -> Option<&'a str> {
    match t {
        Term::Constant(c) => Some(c),
        Term::Variable(k) => b.get(k).map(String::as_str),
    }
}
fn merge(out: &mut BTreeMap<String, String>, bindings: &[Binding]) {
    for b in bindings {
        out.insert(b.id.clone(), b.value.clone());
    }
}
fn bindings_vec(map: BTreeMap<String, String>) -> Vec<Binding> {
    map.into_iter()
        .map(|(id, value)| Binding { id, value })
        .collect()
}
fn fact_hash(f: &Fact) -> Result<String, FreshnessError> {
    Ok(hash(&serde_json::to_vec(&(f.statement(), f.sources()))?))
}
fn hash(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(b"airust-evidence-v1");
    h.update(bytes);
    format!("{:x}", h.finalize())
}
fn validate_data(data: &ReceiptData) -> Result<(), FreshnessError> {
    if data.schema_version != 1
        || !crate::text::Identifier::label(&data.revision, 256)
        || data.proofs.len() > 10000
    {
        return Err(FreshnessError::Invalid("receipt schema/count"));
    }
    let mut ids = BTreeSet::new();
    for p in &data.proofs {
        if p.id == 0
            || !ids.insert(p.id)
            || !crate::text::Identifier::sha256(&p.conclusion)
            || p.facts.is_empty()
            || p.rules.is_empty()
            || p.sources.is_empty()
        {
            return Err(FreshnessError::Invalid("proof identity/support"));
        }
        for (bindings, cap, hashed) in [
            (&p.facts, 256, true),
            (&p.rules, 256, true),
            (&p.sources, 64, false),
        ] {
            if bindings.len() > cap {
                return Err(FreshnessError::Budget);
            }
            let mut seen = BTreeSet::new();
            for b in bindings {
                if !crate::text::Identifier::label(&b.id, 128)
                    || !seen.insert(&b.id)
                    || if hashed {
                        !crate::text::Identifier::sha256(&b.value)
                    } else {
                        !crate::text::Identifier::label(&b.value, 256)
                    }
                {
                    return Err(FreshnessError::Invalid("dependency binding"));
                }
            }
        }
    }
    Ok(())
}
