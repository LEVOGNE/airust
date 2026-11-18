# AIRust 0.2.19: measured stress validation

This release adds guarded public stress workloads and regression tests. Existing runtime APIs, dependency choices, defaults, response JSON and persisted formats remain unchanged. The goal is evidence before optimization, without a new mandatory subsystem. No DenkMaps/MCP integration is performed.

## Executed scale evidence

On the recorded 64 GiB ARM64 host, the two-million-record synthetic lexical fixture builds in 5.930 seconds. Whole-child peak RSS is 3,625.02 MiB. Selective-query p95 is 2.671 ms; broad common-term p95 is 546.41 ms. Four concurrent readers complete eighty selective queries with p95 2.440 ms. One hundred updates, twenty deletions, remaining count and eighty update-marker hits are checked. Only eight broad-query samples were taken; these percentiles are descriptive, not stable tail/SLA estimates.

Broad common-word queries remain expensive because they inspect large posting sets. The near-zero missing-term lookup time must not be used to describe those searches. No native embedding/model, ANN recall, disk throughput or production corpus is measured.

The retained-copy profile builds 500,000 records and keeps three clones. Clone times are 218.29, 216.33, 217.22 ms. Whole-child peak RSS is 3,457.06 MiB. Mutating one clone preserves the original and other clones. This is the fixed baseline for the next memory optimization; it intentionally exposes full-copy cost.

At 100,000 lexical/source records, four readers and two competing writers complete ten successful commits and 74,689 consistency-checked read requests. 3 stale publication attempts are rejected. Readers observe complete generations and matching state/source markers; the final generation is eleven and the retained initial view is unchanged. Whole-child peak RSS is 545.30 MiB. A rejected externally timed publish includes dropping its discarded draft outside the lock, so a large elapsed rejection does not measure write-lock hold time.

Raw commands, OS metadata and outputs: [measurements/0.2.19](measurements/0.2.19/). The [methodology](STRESS.md) defines each scope and exact reproduction commands.

## Regression and release gates

The complete suite passes **191 default and 541 all-feature Rust tests**, including compiled documentation, plus five Python helper tests. New regressions check 4,000 seeded insert/update/delete operations against authoritative records, independent token counts and fresh rebuilt rankings, plus stable ranking across eight simultaneous readers. CI runs scale/clone/concurrent-snapshot smoke programs at 1,000 records; the large actual runs are separate published evidence.

Strict formatting, default/all-feature/execution Clippy, rustdoc, minimum Rust 1.89/1.92, patch compatibility against 0.2.18, fresh audit and an independent consumer gate release. Extracted-package tests, publish dry run, exact-commit GitHub CI and registry/asset/tag/VCS verification gate publication. No additional dependency or mandatory model is introduced.

Count guards cap workloads, not exact heap usage. The snapshot run uses a cooperative 120-second overall budget and bounded attempts/reads; no hard thread termination is claimed. Synthetic repeated vocabulary does not prove capacity for long or adversarial real documents. No real application schema, German semantic quality, external synchronization or production security audit is claimed. See [API cookbook](API-COOKBOOK.md), [migration](MIGRATION-0.2.19.md) and [audit metadata](measurements/0.2.19/audit.json).
