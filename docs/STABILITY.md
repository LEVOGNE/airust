# Stability and support contract

AIRust 0.3.0 checks the existing engine rather than adding another search strategy. The default build has no enabled features. HTTP, SQLite, PDF handling, native models and training remain explicit choices.

## Compatibility

AIRust is still before 1.0. Cargo treats `0.2` and `0.3` as separate compatibility lines. Select `=0.3.0` for a reproducible dependency, or `0.3` to accept later compatible fixes. Changes within 0.3.x should retain public API contracts and readable versioned formats; future incompatible changes require migration notes and a new minor line. This is a maintenance policy, not an unlimited 1.0 support promise.

For this transition, the complete public API is compared against 0.2.30 using patch-level compatibility checks, which are stricter than Cargo requires for a new 0.x minor line. Format versions are separate from crate versions. The engine JSON, version-1 learned model, feedback ledger and disk-index schema are retained. Checked-in fixtures were produced by the unchanged v0.2.30 source; tests read them without regeneration, check identities and update/reopen the old disk file when the analysis recipe matches. If Unicode/SQLite versions differ, the existing explicit rebuild-required error is retained and the original file stays unchanged. Other optional artifact round trips remain covered by their existing suites. This is not a claim to have sampled every historical file.

Malformed model objects imported directly through Serde now fail immediately. Previously some invariants were checked only by constructors and the model-use APIs. `LinearModel::from_json` also enforces a 1 MiB input cap. General Serde calls validate the object but do not impose a total transport-byte cap.

## Supported builds

| Build | Minimum Rust | Validation |
| --- | --- | --- |
| Default core | 1.89 | Tests, all-target checks and strict Clippy |
| Provider-neutral optional features, including disk index and training | 1.89 | Combined all-target check, isolated-feature checks and behavior tests |
| Optional local ONNX adapters | 1.92 | All-feature check, isolated-feature checks and explicit native-model evaluation |
| Complete repository check script | 1.92 and Python 3.11 | `./test.sh`, including generated documentation, examples and every declared feature |

Linux GitHub CI and the local macOS ARM64 run complement each other. Native ONNX execution is measured on macOS ARM64 with explicitly supplied assets; compiling a feature on Linux is not a native-inference test there. Windows and other CPU/runtime combinations have no release-specific execution claim. The Docker image is not rebuilt locally because Docker is unavailable on the validation host; its recipe is not counted as an executed container test.

`python3 -B scripts/check_features.py` reads Cargo.toml and checks every declared feature independently with `--all-targets --no-default-features`. All-feature tests and targeted interaction tests supplement this matrix. The matrix does not enumerate every possible subset of features.

## Result and failure contracts

Search scores rank candidates; they are not probabilities. Semantic similarity does not establish factual truth. Rule conclusions are proposals based on supplied premises. Evidence freshness concerns identified dependencies, not external facts that AIRust has not been told about.

Original records and excerpt byte offsets refer to original UTF-8 text. Stable ID tie ordering, Unicode normalization, scopes, update/delete behavior and model recipes remain part of their documented APIs. Choose one immutable dataset snapshot per request when combining vectors, records and sources. A separate application database still requires caller-managed synchronization.

Cancellation is cooperative. A custom provider can block until it returns. A committed transaction is not reported as cancelled after success. Resource budgets bound documented inputs, outputs or work; they are not hard process-memory or real-time guarantees. The disk index uses its own dedicated database and checked revisions. Do not open an application database as an AIRust disk index.

## What validation establishes

The [release report](CHANGES-0.3.0.md) records executed tests, real native-model smoke checks, independent retrieval evaluations, synthetic stress tests and audit output. Package tests and an external consumer exercise the distributable crate. Publication requires successful CI for the exact release commit. Registry checks then compare the crate checksum, Git commit, annotated tag and GitHub asset.

A passing suite is evidence about these cases and platforms. It is not an independent security audit, an application performance guarantee, a proof of semantic understanding or a test of a DenkMaps/MCP integration. Applications still own authorization, secrets, authenticated artifacts, durable source-of-truth data and user approval of proposed relationships.
