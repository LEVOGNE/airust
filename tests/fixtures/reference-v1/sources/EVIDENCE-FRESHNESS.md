# Evidence freshness

Enable `evidence-freshness`. The feature builds on explicit rule proposals. It compares the dependencies of a captured proof with a caller-supplied current view. It watches no database and changes no accepted facts.

Use `SourceVersion::new(id, version)` for stable source identifiers and caller-verified version/hash labels. `EvidenceView::new(revision, facts, rules, sources)` supplies one consistent authorized view. `with_allowed_facts` restricts dependency access; an empty set denies all facts. Source versions are assertions: registering a version does not establish source authenticity, reliability or truth.

`capture_evidence(result, view, options)` first checks the capture revision, replays every retained rule against its actual premise statements, checks signed patterns, repeated variables, head bindings, rule fingerprints, dependency unions and backward proof references. It records fact fingerprints, all transitive rule fingerprints and source versions. A mismatched view fails instead of signing an unverified conclusion. Capturing an incomplete inference retains that closure status.

`verify_evidence(receipt, current_view, options)` returns one status per retained proposal: Current, Stale, Missing or NotAllowed. Unrelated revision changes alone do not invalidate unchanged dependencies. Changed source versions, fact content/source bindings or any transitive rule mark dependent proofs stale. Removed dependencies are missing. Scope denial takes precedence and skips the remaining dependency checks for that proposal; reports expose no current hidden contents or versions. The caller must authorize access to receipts as well as facts/sources. `all_current()` is false for an empty proposal set. Current means matching declared dependencies, not a true conclusion or complete knowledge.

EvidenceReceipt::to_json/from_json uses bounded, versioned JSON and a canonical payload checksum. Unknown fields, duplicate proof/dependency IDs, invalid hashes, unsupported schemas/counts and checksum changes are rejected. The checksum detects accidental alteration; it is not a signature. Authenticate receipts from another trust boundary. JSON contains opaque IDs, source version labels and conclusion/dependency hashes, not original fact statements. These identifiers/hashes are not anonymization. Store the original authorized proposal, revision and receipt fingerprint together to preserve their audit binding.

FreshnessOptions defaults to 16 MiB receipt JSON; supported limits are 1 KiB..64 MiB. Views cap at 100,000 facts/sources and 256 rules. Receipts cap at 10,000 proposals, 256 base facts/rules and 64 sources per proof. Bounds constrain this representation, not precise RAM usage. With execution enabled, with_control supports cooperative cancellation/deadlines; interruption returns an error without a partial successful report. Parsing, serialization and individual bounded operations are synchronous.

Run `cargo run --features evidence-freshness --example evidence_freshness`. It captures a transitive proof, verifies unchanged dependencies and reports a changed source as stale. Tests cover replay, source/fact/rule changes, missing inputs, permission revocation, unrelated revisions, receipt round trips, corruption, limits and interruption. No source synchronization or DenkMaps/MCP integration is performed.

## Complete example

```rust
use airust::{evidence_freshness::*, reasoning::*};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let facts = [
        Fact::new("ab", Statement::new("a", "edge", "b")?)?.with_sources(["s".into()])?,
        Fact::new("bc", Statement::new("b", "edge", "c")?)?.with_sources(["s".into()])?,
    ];
    let rules = [Rule::new(
        "transitive",
        vec![
            Pattern::new(Term::variable("a"), "edge", Term::variable("b"))?,
            Pattern::new(Term::variable("b"), "edge", Term::variable("c"))?,
        ],
        Pattern::new(Term::variable("a"), "edge", Term::variable("c"))?,
    )?];
    let result = infer(&facts, &rules, &ReasoningOptions::new("r1", ["s".into()])?)?;
    let sources = [SourceVersion::new("s", "version-1")?];
    let view = EvidenceView::new("r1", &facts, &rules, &sources)?;
    let options = FreshnessOptions::default();
    let receipt = capture_evidence(&result, &view, &options)?;
    assert!(verify_evidence(&receipt, &view, &options)?.all_current());
    let changed = [SourceVersion::new("s", "version-2")?];
    let report = verify_evidence(
        &receipt,
        &EvidenceView::new("r2", &facts, &rules, &changed)?,
        &options,
    )?;
    assert_eq!(report.proofs()[0].state, FreshnessState::Stale);
    println!("{}", report.to_json()?);
    Ok(())
}

```
