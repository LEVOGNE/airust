# AIRust 0.2.24 acceptance plan

an optional small linear model with replaceable bounded features, explicit training, grouped holdout evaluation and versioned artifacts.

Eight integration cases plus a numerical-gradient unit test cover deterministic learning, held-out groups, unseen-query ordering, loss reduction, malformed artifacts, feature/schema changes, Unicode, budgets and controlled cancellation/deadlines/progress.

Require default/all-feature and isolated-feature tests, strict formatting/Clippy/rustdoc, Rust 1.89/1.92, patch compatibility, audit, independent consumer, package/dry-run, exact-commit CI, registry source/checksum/tag/asset verification. No application integration.
