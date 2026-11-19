# AIRust 0.3.0 acceptance plan

Release a verified stability baseline without new application integrations or mandatory model dependencies.

1. Reproduce and fix actual import-validation defects; preserve valid artifact identities.
2. Test records, model, feedback and a real disk file produced by unchanged 0.2.30 source.
3. Check every declared feature independently with all targets; run default/all-feature tests, strict Clippy/rustdoc, examples, Python helper tests and both minimum Rust toolchains.
4. Run local E5 and cross-encoder inference with explicitly supplied, checksum-verified assets. Rerun identified independent lexical evaluations and representative synthetic scale/concurrency tests.
5. Run the stricter patch-level API compatibility check against 0.2.30 and a fresh dependency audit. Explain skips, maintenance notices and platform limits.
6. Update English README, API, migration, support and results guides. Verify a standalone consumer and the extracted clean package; perform publication dry-run.
7. Publish only after exact-commit GitHub CI succeeds. Verify registry consumer, checksum, VCS commit, tag and GitHub asset before reporting completion.
