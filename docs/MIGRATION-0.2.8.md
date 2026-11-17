# Migrating to AIRust 0.2.8

The independent `quality` feature is additive within Cargo's 0.2 compatibility range, adds no dependency and supports Rust 1.89. Default features remain empty. Existing entries/source JSON, retrieval, graph, context, provenance, cache and versioned-history formats are preserved. The local ONNX adapter retains its Rust 1.92 requirement.

```toml
airust = { version = "=0.2.8", default-features = false, features = ["quality"] }
```

Use `Judgments::new` with grades 0..31 and `evaluate_ranking` with ranked IDs, a positive cutoff and an explicit `UnjudgedPolicy`. Grade zero is nonrelevant; missing judgments are a separate state. `RequireJudged` rejects unknown IDs; `NonRelevant` reports and scores them as zero. Duplicate IDs within the evaluated window fail. Precision divides by K even for short lists. Recall and exponential-gain nDCG are undefined when no positive labels exist. `summarize` requires compatible nonempty query metrics and reports defined-denominator query counts. The [quality guide](QUALITY.md) specifies the formulas and limits.

The new thirty-document/thirty-six-query reference fixture is separate from the unchanged earlier semantic fixture. Its labels are authored smoke judgments, not independent application relevance or verified logical understanding. Performance tools retain raw samples and resource counters without significance or production-tail claims. The Python measurement wrapper is optional development tooling, not a Rust runtime dependency.

CI uses the verified official checkout v7.0.1 commit, Node 24, no persisted credentials and contents:read permissions. No new cloud/model account or downloads are required to compile/test the core. Read the [API cookbook](API-COOKBOOK.md) for all optional modules and [DenkMaps upgrade guide](DENKMAPS-UPGRADE.md) for proposed separate application work. No new core integration is performed.
