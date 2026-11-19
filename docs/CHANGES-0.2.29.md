# AIRust 0.2.29: Controlled training pipeline

trusted feedback curation, versioned datasets, grouped training and explicit quality-gated model promotion with rollback.

## Validation

The suite passes 194 default and 634 all-feature Rust tests, including documentation, plus five Python tests. Isolated feature checks, strict Clippy/rustdoc, Rust 1.89/1.92, patch compatibility against 0.2.28, audit, local/registry consumers, extracted-package tests, dry-run and exact-commit CI gate publication. Correction recorded during the 0.3.0 review: the ignored test is a helper launched by the parent crash-recovery test. External native-model inference was not rerun for this release.

See [API and limits](TRAINING-PIPELINE.md) and [measurements](measurements/0.2.29/). Dataset/model identities, explicit uncertainty and application-owned authorization remain part of the contract. Public measurements do not establish production quality or an independent security audit. No DenkMaps/MCP integration.
