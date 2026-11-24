# Independent validation of AIRust

This protocol makes the next product gates observable. It is prepared procedure, not an assertion that an independent developer, reviewer or customer has participated. Use [ROADMAP.md](../ROADMAP.md) for priorities and [WORK-PROGRAM-NEXT.json](WORK-PROGRAM-NEXT.json) for current states. Preserve every failed attempt and the original inputs. Work under a new ignored `local-lab/independent-validation/ATTEMPT_ID/` directory, outside Cargo target output.

## Attempt identity and independence

Record date, repository/tag/registry checksum, OS/CPU/RAM, Rust/Cargo/Python versions, exact commands, locks, relevant feature/model/runtime identities and the participant's declared role. Record prior AIRust/reference-corpus exposure and author assistance. A different automated agent or an assistant-authored review is not an independent human participant. Caller-declared independence in a pilot JSON file is a claim requiring provenance review, not independent verification by the validator.

Keep raw command output, elapsed-time observations and observed errors, including incomplete/unavailable results. Share only appropriate consented material; private source questions and local service environment logs remain outside public evidence. Do not replace failed results with a corrected rerun. Link a correction to the original attempt and state the changed source/configuration.

## G1: timed default-library onboarding

Use a clean supported environment with the declared Rust prerequisite. Start the 15-minute clock when the participant starts reading the installation/example instructions. Record whether Rust and network/registry access were already available; record toolchain installation separately if necessary. Stop only after a real query returns the expected original record. Record help, undocumented changes and source edits. Success requires no manual source correction and elapsed time at most 15 minutes. An author-driven smoke test is useful engineering evidence but does not pass this independent gate.

Create a normal Cargo binary and select the release explicitly:

```toml
[dependencies]
airust = { version = "=0.6.0", default-features = false }
```

The minimal program uses no server, MCP or model:

```rust
use airust::{Engine, KnowledgeEntry};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut index = Engine::new();
    index.upsert(KnowledgeEntry::new("ownership", "Rust ownership protects memory."))?;
    let hits = index.search("Rust memory", 5);
    assert_eq!(hits[0].entry.id, "ownership");
    println!("{}", hits[0].entry.text);
    Ok(())
}
```

Run `cargo run`, preserve the resolved lock and output, then repeat with `cargo run --locked`. Ask the participant to add one original record and query it. Record confusion and the actual number of edits/steps rather than inferring usability from line count. Rust 1.89 supports this API. A new transcript is required for a new independent participant; existing release consumer logs cannot be relabelled.

## G2: complete reference application

Follow [STARTUP.md](../STARTUP.md), [REFERENCE-APPLICATION.md](REFERENCE-APPLICATION.md) and [PILOT.md](PILOT.md) using a new workspace and a small participant-selected Markdown collection. Record original file hashes. Import, query, resolve a returned UTF-8 passage, start the same collection through MCP and query it with the selected client. Record client name/version/configuration and the actual initialization/tool invocation. A real external LLM-client run is separate evidence from a synthetic stdio protocol client.

Check that original input files and the earlier workspace remain unchanged. Edit a source document, build a separate revision and confirm the previous snapshot still resolves its original evidence. Exercise one malformed input, one existing-destination rejection and one cancellation/recovery path without corrupting prior data. A complete attempt requires no undocumented manual patch. Record which controls were actually attempted; absence of a failure is not evidence for an unexecuted recovery case.

## G3: genuine independently reviewed search cases

Capture actual participant questions explicitly with the existing pilot commands. Record origin/language, workspace/document revisions, exact source ranges and active recipe. Preserve the original case when creating a review. A human source reviewer checks supported/unsupported status, relevant sources, judgment coverage and independence; the tool can check bytes and declared fields but cannot certify that the declared role is true.

Collect at least 50 unique eligible questions, including unsupported cases. Do not count the 150 assistant-authored reference-v1 questions, synthetic failure fixtures or the three unreviewed historical conversation replays. Start this collection before waiting for a future version number.

Use the independent validator from [PILOT.md](PILOT.md). Before running candidates, freeze a separately named reference-v2 input set, provenance review, development/evaluation source-family split, judgments and numerical resource/quality policy. Both splits need supported and unsupported cases, at least ten cases each and no source/document leakage under the implemented freeze contract. Select configurations on development data before reserved evaluation. Keep version one frozen and do not tune from its exposed evaluation labels.

The initial answerable-case floors are nDCG@10 >= 0.75 and Hit@6 >= 0.80 with zero source-range mismatches/scope leaks. Predeclare the optional profile's gain, allowed loss and complete-request/resource budgets before execution. Preserve every question result and regression. Known-quote hit counts from authored reference cases are not independently judged nDCG. Candidate returns on unsupported questions are not generated-answer error rates; the application returns evidence, not answers. Freezing valid inputs alone does not pass measured quality gates.

## G4: independent operating-cost reproduction

Reproduce [the release measurements](measurements/pilot-0.6.0/README.md) with the exact retained compilers, locks, baseline/candidate sources/features and model/runtime/cache identities. If an input changes, record a new configuration; do not silently merge its results with the old campaign. Retain complete raw samples and failures. Do not compile or run Miri concurrently with native timing campaigns.

Measure startup/model loading, warm retrieval, complete request including query inference/final framing, worker RSS and external-service RSS separately. State whether memory is exact-child peak or process-tree sampling; sampled service RSS is not a continuous peak or unique physical GPU/shared memory. Record unavailable observations explicitly. Do not extrapolate sequential latency to throughput or million-entry controls to production capacity. Same-host identical package archives do not establish cross-host bit-for-bit binaries.

For a product comparison, first define the concrete local-document/evidence workload. Use appropriate SQLite/Tantivy/LanceDB configurations, comparable scopes, source revisions, models, tuning allowance and cost boundaries. Predeclare criteria and retain negative outcomes. A tool with different responsibilities may be the better choice; no superiority claim is presumed.

## G5: compatibility and release controls

Review the public contracts used by actual consumers: Engine/search, original-entry ownership, typed errors, retained readers, revision-bound spans, saved formats, MCP schemas and optional backend trust. Restore retained data and run independent released-crate consumers on the documented compilers/platforms. Cargo 0.5 and 0.6 are distinct pre-1.0 ranges; future 0.6 patches must preserve the supported contract or explicitly move the break to a new minor line.

Run meaningful changed-behavior tests, full release checks, feature/MSRV gates, scoped Miri and both-lock advisory audits as applicable under [CONTRIBUTING.md](../CONTRIBUTING.md). Record exclusions, maintenance notices, unavailable CI and all source/package/tag identities. Never infer that native dependencies are interpreted because owned Rust tests pass Miri. A 1.0 compatibility promise needs review and real operational evidence, not a version bump alone.

## Evidence record template

The example below is an empty template. Null fields and `not_run` are deliberately not passing results. Keep full participant details privately where needed and publish only consented evidence.

```json
{
  "format_version": 1,
  "attempt_id": null,
  "source_commit": null,
  "registry_checksum": null,
  "environment_record": null,
  "participant_role": null,
  "independence_provenance": null,
  "prior_exposure": null,
  "author_assistance": null,
  "started_at": null,
  "finished_at": null,
  "elapsed_seconds": null,
  "commands_and_raw_logs": [],
  "original_input_hashes": {},
  "gate": null,
  "status": "not_run",
  "observed_result": null,
  "failures_and_exclusions": [],
  "correction_of_attempt": null
}
```

Update the next work program only after recording the actual result and evidence path. The agent may prepare artifacts and analyze results, but external human input remains awaiting inputs until it exists.

The release macOS CI job explicitly selects the supported `macos-15` ARM64 image, uses two Cargo build jobs and a 45-minute job bound, with architecture-specific cache keys. Earlier macOS 26 ARM64 capacity cancellations and an unresponsive/log-unavailable Intel attempt remain separate non-passing evidence. The Intel attempt does not establish Intel support or a proven library failure. Read actual final job conclusions in the release manifest; keep complete local ARM64 checks and scoped Miri separately recorded.
