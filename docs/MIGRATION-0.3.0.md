# Upgrade from 0.2.30 to 0.3.0

Version 0.3.0 keeps the existing APIs and algorithms. Its changes strengthen validation and release checks. No DenkMaps or MCP adapter is installed or changed.

## Update the dependency

```toml
[dependencies]
airust = { version = "=0.3.0", default-features = false }
```

Keep your existing explicit feature list. For example, `features = ["retrieval", "advanced-search"]` enables compact excerpts and advanced lexical search. Add `disk-index` only for a dedicated AIRust database. Provider-neutral builds still use Rust 1.89; local ONNX adapters require 1.92.

Run `cargo update -p airust` after changing your manifest, then run your application's tests. A dependency requirement of `0.2` does not select 0.3.0 automatically because Cargo treats these as different pre-1.0 compatibility lines.

## Existing data

No new on-disk schema or model recipe is introduced. The checked-in [0.2.30 fixtures](../tests/fixtures/compat-0.2.30/manifest.json) cover original records, a trained model, feedback and an actual SQLite disk-index file. Tests preserve search ordering, model fingerprints, feedback profiles and stored revisions; the old database can be updated and reopened when its analysis recipe matches.

The disk analysis recipe includes Unicode and SQLite versions, as it did in 0.2.30. A different Rust toolchain can change Unicode rules even when the crate version is unchanged. A matching recipe permits direct reopen; a mismatch returns an explicit rebuild-required error and preserves the original file. This release tests both same-recipe reopening and rejection with the Rust 1.89 toolchain.

For a recipe change, export original entries in bounded revision-fenced pages using the old compatible executable. Create a new dedicated index with the new executable, import bounded batches and run its integrity check. Compare counts and representative searches before switching application configuration. Keep the old file until that check succeeds; do not rewrite metadata to force a match.

Keep a backup before any application migration. Load data through the relevant validated APIs. Use `integrity_check` for a dedicated disk index. Existing in-memory caches, graph/source references and other optional artifacts retain their documented formats and tests. Application schema migrations and synchronization with an external database remain application work.

## Stricter JSON validation

Direct `serde_json::from_str`, `from_slice` or `from_value` calls for `FeatureSchema`, `TrainingConfig`, `TrainingReport` and `LinearModel` now reject invalid invariants immediately. Examples include duplicate feature names, zero epochs, impossible positive-label counts and an unsupported model schema version. These objects were already rejected by constructors or when used as models; the direct import path now applies those same rules.

A validation failure during JSON import is now reported by Serde and may surface through `LearningError::Json` rather than a later `LearningError::Invalid`. Constructor validation continues to use `Invalid`. Handle errors at the operation boundary; message strings are not a stable error-code contract.

Valid version-1 JSON retains its shape, floating-point values and model fingerprint. No retraining is required for a valid existing model. Prefer `LinearModel::from_json` when importing text because it also applies the 1 MiB model-input limit. General Serde deserialization does not supply a total input-byte budget; bound incoming transport bytes separately.

Disk-state length validation and data retrieval now use one SQL statement. The revision and count APIs no longer perform a separate size check before fetching the value. The 16 KiB metadata limit is unchanged.

## Before enabling this in an application

First keep the application's persistent data and permissions authoritative. Build an AIRust index from authorized records, compare existing search results on a fixed labeled set, and verify create/update/delete synchronization. Test relevant excerpts against their original UTF-8 source and inspect no-answer cases. Measure memory and latency on representative data. The [application guide](APPLICATION-INTEGRATION.md) explains the APIs and a possible future adapter; this release performs no such integration.
