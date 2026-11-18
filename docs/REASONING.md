# Bounded explicit reasoning proposals

Enable `reasoning`. The feature adds no model, database, parser, network client or mandatory dependency. Optional SHA-256 binds rule identities; defaults remain unchanged. Applications supply signed facts, registered source IDs and rules; the engine does not extract truth or rules from natural language.

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use airust::reasoning::*;
let facts = vec![
    Fact::new("ab", Statement::new("a", "requires", "b")?)?.with_sources(["manual".into()])?,
    Fact::new("bc", Statement::new("b", "requires", "c")?)?.with_sources(["manual".into()])?,
];
let rule = Rule::new("transitive", vec![
    Pattern::new(Term::variable("x"), "requires", Term::variable("y"))?,
    Pattern::new(Term::variable("y"), "requires", Term::variable("z"))?,
], Pattern::new(Term::variable("x"), "requires", Term::variable("z"))?)?;
let options = ReasoningOptions::new("database-revision-7", ["manual".into()])?;
let result = infer(&facts, &[rule], &options)?;
assert!(result.complete());
assert_eq!(result.proposals()[0].statement.subject(), "a");
assert_eq!(result.proposals()[0].statement.object(), "c");
assert_eq!(facts.len(), 2);
# Ok(()) }
```

## Explicit semantics and evidence

Statement contains subject/predicate/object and an explicit negative flag. Constants are opaque case-sensitive strings, not normalized text. Pattern has constant predicate, constant/variable subject/object and explicit sign. Rule has an ID, 1..8 premises and one head. At most sixteen variables are allowed, and every head variable must occur in a premise. There are no function symbols, implicit negation, probabilistic truth claims or automatic relation types. Repeated variables require equality.

Fact::new(id, statement).with_sources(...) supplies immutable caller facts. ReasoningOptions::new(revision, known_sources) supplies the explicitly registered source-ID set. A fact without sources or referencing an unknown source is excluded from deduction, counted in missing_support_count and listed up to 100 IDs in missing_support (possibly fewer after output budgeting). Registered source IDs establish traceability only: this function cannot verify source content, reliability, ownership or whether a statement follows from quoted text. Use your own authoritative provenance store and evidence review.

infer returns proposals, never updates the facts or a knowledge database. Each Proposal has its statement, rule ID and canonical rule fingerprint, typed immediate premise references (Fact ID or earlier Proposal ID), transitive base fact IDs and source IDs. Rule::body/head/to_json expose the exact rule; fingerprint identifies its canonical representation, not its authenticity. Preserve that representation with the application audit record. Only one deterministic proof is retained per new statement. Inputs/rules are sorted by ID; rounds use only already committed local proof nodes. Proposal references always point backward, so the proof graph remains acyclic even if input relationships contain cycles. Cycles can still imply self-relations under a caller's transitive rule; decide whether such a rule makes sense for your domain.

Explicit positive/negative opposites appear as Conflict reference pairs, including conflicts involving a derived proposal. Both statements remain visible; a contradiction does not prove unrelated statements or select a winner. A complete result means the finite supported/allowed closure reached a fixed point and no input support was missing. It does not establish real-world truth, rule correctness or proof uniqueness. A missing-support result may stop at FixedPoint while complete remains false. Never treat absence of a proposal as proof of falsehood.

## Scope, bounds and interruption

with_allowed_facts explicitly restricts premise IDs. All intermediates and proposals derive only from those facts; hidden facts cannot act as bridges. This is not an authorization service: filter/authorize facts and node endpoints before calling, and ensure their source references are also safe to display. Without an allowed-ID set every supplied supported fact is eligible.

with_limits(rounds, joins, proposals, json_bytes) sets actual limits. Defaults: 16 rounds, 1,000,000 join trials, 1,000 proposals and 1 MiB serialized result. Supported maxima: 100 rounds, 10,000,000 joins, 10,000 proposals and 16 MiB JSON; minimum JSON budget 1 KiB. Predicate/sign indexing limits candidate selection; every attempted candidate join is counted. This is a bounded deterministic rule interpreter, not a large distributed graph optimizer. Returned rounds/join_trials report actual work, not an ETA.

Inputs cap at 100,000 facts, 256 rules and 16 MiB total fact/rule string bytes. Labels/constants/variables are bounded and control-free; fact/rule/source IDs cap at 128 bytes, constants/predicates at 256. Source/allowed-ID sets cap at 100,000. Each fact has up to 64 source IDs; proof unions cap at 256 base facts and 64 sources. A limit gives an explicit StopReason and complete=false. Retained proposals remain individually supported. Output budgeting drops diagnostic IDs/conflicts when needed, then retains only a fitting proposal prefix, so retained proofs never reference removed proposals. JSON includes its own status/diagnostic envelope in the byte budget. Input counts/bytes are bounds, not exact process RAM limits.

With `execution`, options.with_control cooperatively checks cancellation/deadlines through validation, joins, rounds and final output. Interruption returns an error, not a partial successful result. Actual Reasoning progress reports completed rounds with unknown total; no fabricated percentage or hard wall-clock preemption is promised. Serialization and a single bounded matching operation cannot be stopped halfway.

## Validation and use

Tests compare thirty seeded directed graphs, including cycles, against independent breadth-first closure. Other fixtures cover safe binding, constants/repeated variables, proof/source integrity, missing support, explicit conflicts, hidden bridges, deterministic output, limits, JSON size and interruption. The reasoning example derives a prerequisite chain with explicit evidence and prints actual work/resources. It validates declared symbolic semantics, not natural-language reasoning quality.

Treat proposals as reviewable suggestions. Save accepted conclusions with rule/model/source/revision evidence in your application's own transaction, and revalidate when premises change. AIRust does not synchronize an external database or watch source updates. No DenkMaps/MCP integration is performed.
