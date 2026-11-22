# Stability and support contract

AIRust 0.4.4 reduces temporary construction allocations while preserving the shared core memory, ranking optimizations, TF-IDF reuse and optional MCP response schemas. Optional read-only MCP, progress and cancellation remain available, with isolated connection shutdown. The default build has no enabled features. HTTP, SQLite, PDF handling, native models and training remain explicit choices.

## Compatibility

AIRust is still before 1.0. Cargo treats `0.3` and `0.4` as separate compatibility lines. Select `=0.4.4` for a reproducible dependency, or `0.4` to accept later compatible fixes. Changes within 0.4.x should retain public API contracts and readable versioned formats; future incompatible changes require migration notes and a new minor line. This is a maintenance policy, not an unlimited 1.0 support promise.

For 0.4.4, existing public APIs are compared against 0.4.3 using patch-level checks: 229 pass and 31 tool skips. The earlier 0.3.0 transition was checked against 0.2.30. These checks are stricter than Cargo requires for a new 0.x minor line. Format versions are separate from crate versions. The engine JSON, version-1 learned model, feedback ledger and disk-index schema are retained. Index cache payloads cross-load between 0.4.3 and 0.4.4; their producer label and checksum differ by version. Internal TF-IDF hints are not serialized. MCP response data stays schema version one; output schemas are additive discovery metadata. Checked-in fixtures were produced by the unchanged v0.2.30 source; tests read them without regeneration, check identities and update/reopen the old disk file when the analysis recipe matches. If Unicode/SQLite versions differ, the existing explicit rebuild-required error is retained and the original file stays unchanged. Other optional artifact round trips remain covered by their existing suites. This is not a claim to have sampled every historical file.

Since 0.3.0, malformed model objects imported directly through Serde fail immediately. Previously some invariants were checked only by constructors and the model-use APIs. `LinearModel::from_json` also enforces a 1 MiB input cap. General Serde calls validate the object but do not impose a total transport-byte cap.

## Supported builds

| Build | Minimum Rust | Validation |
| --- | --- | --- |
| Default core | 1.89 | Tests, all-target checks and strict Clippy |
| Provider-neutral optional features, including MCP, disk index and training | 1.89 | Combined all-target check, isolated-feature checks and behavior tests |
| Optional local ONNX adapters | 1.92 | All-feature check, isolated-feature checks and explicit native-model evaluation |
| Complete repository check script | 1.92 and Python 3.11 | `./test.sh`, including generated documentation, examples and every declared feature |

Linux GitHub CI and the local macOS ARM64 run complement each other. Native ONNX execution is measured on macOS ARM64 with explicitly supplied assets; compiling a feature on Linux is not a native-inference test there. Windows and other CPU/runtime combinations have no release-specific execution claim. The Docker image is not rebuilt locally because Docker is unavailable on the validation host; its recipe is not counted as an executed container test.

`python3 -B scripts/check_features.py` reads Cargo.toml and checks every declared feature independently with `--all-targets --no-default-features`. All-feature tests and targeted interaction tests supplement this matrix. The matrix does not enumerate every possible subset of features.

## Result and failure contracts

Search scores rank candidates; they are not probabilities. Semantic similarity does not establish factual truth. Rule conclusions are proposals based on supplied premises. Evidence freshness concerns identified dependencies, not external facts that AIRust has not been told about.

Entry excerpt byte offsets refer to the exact entry UTF-8 text. With `documents`, passages refer to the identified complete snapshot text; use checked subspan projection and never infer original PDF coordinates. Stable ID tie ordering, Unicode normalization, scopes, update/delete behavior and model recipes remain part of their documented APIs. Choose one immutable dataset snapshot per request when combining vectors, records and sources. A separate application database still requires caller-managed synchronization.

Cancellation is cooperative. A custom provider can block until it returns. A committed transaction is not reported as cancelled after success. Resource budgets bound documented inputs, outputs or work; they are not hard process-memory or real-time guarantees. The disk index uses its own dedicated database and checked revisions. Do not open an application database as an AIRust disk index.

## What validation establishes

The [current release report](CHANGES-0.4.4.md) records fresh MCP/core checks, Miri runs, extended differential stress, scale fixtures and separate quiet costs. Historical native-model smoke checks and independent retrieval evaluations remain in the [0.3.0 report](CHANGES-0.3.0.md) and later scoped stability reports; those unchanged-module results are explicitly inherited. Package tests and an external consumer exercise the distributable crate. Publication requires successful CI for the exact release commit. Registry checks then compare the crate checksum, Git commit, annotated tag and GitHub assets.

A passing suite is evidence about these cases and platforms. It is not an independent security audit, an application performance guarantee, a proof of semantic understanding or a test of a DenkMaps integration or every MCP client. Applications still own authorization, secrets, authenticated artifacts, durable source-of-truth data and user approval of proposed relationships.

The [document contract](DOCUMENTS.md) specifies format version one, exact text/source identity, bounded JSON, snapshot retention and remaining trust/coverage boundaries. No authoritative application store or automatic legacy-knowledge migration is introduced.

The [MCP API](MCP.md) is opt-in in the main crate. Its guarded stdio transport exposes only an explicitly loaded immutable collection and enforces configured frame/rate/request/worker/result bounds. The [security model](MCP-SECURITY.md) explains tested attacks, client-controlled agent narration, untrusted source content, payload-vs-RAM limits and the absence of remote/multi-tenant authorization. These are scoped tested protections, not a universal attack-resistance claim.


Since 0.4.3, global vocabulary lookup uses the standard randomly seeded hash map. It is not used to enumerate result order or serialized cache records. Candidate IDs, per-entry term arithmetic and tie ordering remain deterministic. A bounded token prefix and winner-only explanations preserve source data and avoid whole-entry token buffering or strings for discarded hits. These are internal details with measured costs, not a new public storage layout or universal performance guarantee.

Version 0.4.4 combines repeated token counts in a private temporary buffer and keeps no new scratch storage in the completed index. MCP keeps a fixed immutable tool catalogue per shared data instance. It rejects new worker reservations after connection shutdown begins and waits for existing reservations/jobs from that connection; all clones still share the same admission limit. These costs and lifetimes are documented separately from whole-process RAM and hard OS isolation.
