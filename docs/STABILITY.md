# Stability and support contract

AIRust 0.5.1 keeps the public Core and optional MCP line stabilized in 0.5.0. Runtime source stays unchanged from 0.4.25 through 0.5.0, with the 0.5.0 state/cache/clone checks and finite large-index measurements still describing that runtime. 0.5.1 adds a fixed-seed fuzz check and no behavior change. Default features stay empty. Read [fuzz scope](CHANGES-0.5.1.md), [0.5.0 validation](CHANGES-0.5.0.md) and [upgrade](MIGRATION-0.5.1.md).

## Compatibility

AIRust is still before 1.0. Cargo treats 0.4 and 0.5 as separate compatibility lines. Select `=0.5.1` for reproducibility, or `0.5` for compatible later fixes. A 0.4 requirement does not select 0.5 automatically. Within 0.5.x, preserve public contracts and readable versioned formats; incompatible changes need a new line and migration notes. This is a maintenance policy, not an unlimited 1.0 support promise.

For 0.5.0, a deliberately strict patch-level comparison against 0.4.25 reports 229 pass, 31 skip; it finds no required application-call changes. Native validation passes 253 default, 909 all-feature checks including 39 doctests and 39 isolated features. The 38 finite Miri cases include the new state test reduced to eight steps, four queries, two scopes and one limit; native execution retains 250 steps and parallel readers. Earlier native-model evaluations remain historical, not newly rerun. Knowledge formats remain; MCP schema URNs include the crate version. Full scope and negative observations stay in the current report.

Since 0.3.0, malformed model objects imported directly through Serde fail immediately. Previously some invariants were checked only by constructors and the model-use APIs. `LinearModel::from_json` also enforces a 1 MiB input cap. General Serde calls validate the object but do not impose a total transport-byte cap.

## Supported builds

| Build | Minimum Rust | Validation |
| --- | --- | --- |
| Default core | 1.89 | Tests, all-target checks and strict Clippy |
| Provider-neutral optional features, including MCP, disk index and training | 1.89 | Combined all-target check, isolated-feature checks and behavior tests |
| Optional local ONNX adapters | 1.92 | Current compilation/isolated-feature checks; earlier native-model evaluation is separately scoped below |
| Complete repository check script | 1.92 and Python 3.11 | `./test.sh`, including generated documentation, examples and every declared feature |

Linux GitHub CI and the local macOS ARM64 run complement each other. Native ONNX execution is measured on macOS ARM64 with explicitly supplied assets; compiling a feature on Linux is not a native-inference test there. Windows and other CPU/runtime combinations have no release-specific execution claim. The Docker image is not rebuilt locally because Docker is unavailable on the validation host; its recipe is not counted as an executed container test.

`python3 -B scripts/check_features.py` reads Cargo.toml and checks every declared feature independently with `--all-targets --no-default-features`. All-feature tests and targeted interaction tests supplement this matrix. The matrix does not enumerate every possible subset of features.

## Result and failure contracts

Search scores rank candidates; they are not probabilities. Semantic similarity does not establish factual truth. Rule conclusions are proposals based on supplied premises. Evidence freshness concerns identified dependencies, not external facts that AIRust has not been told about.

Entry excerpt byte offsets refer to the exact entry UTF-8 text. With `documents`, passages refer to the identified complete snapshot text; use checked subspan projection and never infer original PDF coordinates. Stable ID tie ordering, Unicode normalization, scopes, update/delete behavior and model recipes remain part of their documented APIs. Choose one immutable dataset snapshot per request when combining vectors, records and sources. A separate application database still requires caller-managed synchronization.

Cancellation is cooperative. A custom provider can block until it returns. A committed transaction is not reported as cancelled after success. Resource budgets bound documented inputs, outputs or work; they are not hard process-memory or real-time guarantees. The disk index uses its own dedicated database and checked revisions. Do not open an application database as an AIRust disk index.

## What validation establishes

The [current release report](CHANGES-0.4.24.md) records fresh MCP/core checks, Miri runs, extended differential stress, scale fixtures and separate quiet costs. Historical native-model smoke checks and independent retrieval evaluations remain in the [0.3.0 report](CHANGES-0.3.0.md) and later scoped stability reports; those unchanged-module results are explicitly inherited. Package tests and an external consumer exercise the distributable crate. Publication requires successful CI for the exact release commit. Registry checks then compare the crate checksum, Git commit, annotated tag and GitHub assets.

A passing suite is evidence about these cases and platforms. It is not an independent security audit, an application performance guarantee, a proof of semantic understanding or a test of a DenkMaps integration or every MCP client. Applications still own authorization, secrets, authenticated artifacts, durable source-of-truth data and user approval of proposed relationships.

The [document contract](DOCUMENTS.md) specifies format version one, exact text/source identity, bounded JSON, snapshot retention and remaining trust/coverage boundaries. No authoritative application store or automatic legacy-knowledge migration is introduced.

The [MCP API](MCP.md) is opt-in in the main crate. Its guarded stdio transport exposes only an explicitly loaded immutable collection and enforces configured frame/rate/request/worker/result bounds. The [security model](MCP-SECURITY.md) explains tested attacks, client-controlled agent narration, untrusted source content, payload-vs-RAM limits and the absence of remote/multi-tenant authorization. These are scoped tested protections, not a universal attack-resistance claim.


Since 0.4.3, global vocabulary lookup uses the standard randomly seeded hash map. It is not used to enumerate result order or serialized cache records. Candidate IDs, per-entry term arithmetic and tie ordering remain deterministic. A bounded token prefix and winner-only explanations preserve source data and avoid whole-entry token buffering or strings for discarded hits. These are internal details with measured costs, not a new public storage layout or universal performance guarantee.

Version 0.4.4 combines repeated token counts in a private temporary buffer and keeps no new scratch storage in the completed index. MCP keeps a fixed immutable tool catalogue per shared data instance. It rejects new worker reservations after connection shutdown begins and waits for existing reservations/jobs from that connection; all clones still share the same admission limit. These costs and lifetimes are documented separately from whole-process RAM and hard OS isolation.

Version 0.4.23 recognizes at most 4,096 bytes of one repeated canonical ASCII word, borrowing the word and its exact count into checked ranking. The guard is not a new Core input limit. Other and preanalyzed queries retain original analysis; public calls and saved knowledge remain. Full final Core/MCP comparisons keep all slower observations. Final MCP response construction still follows search-worker completion in this release; startup and transport are outside tool computation deadlines. Finite tests and cooperative budgets do not impose an OS CPU/RAM quota.

Version 0.4.24 keeps full admitted MCP tool-result preparation and its last byte bound with the worker reservation. Error progress reflects a final-budget failure, and before/after checks reject cancelled/expired success. This supersedes the 0.4.23 final-preparation boundary described above. Startup, SDK JSON-RPC framing and final response writes remain separate; cooperative cancellation is not forced termination or an OS quota. Actual latency/allocation observations and every slower case stay in the current report.
