# AIRust 0.2.20: optional shared index storage

The new shared-index flag shares immutable analyzed records, posting maps and normalized duplicate-ID sets across lexical index clones. Mutations detach affected data, preserving old and sibling views. Public methods, defaults, dependency versions and portable/cache formats remain unchanged. No model/server is enabled and snapshots does not automatically enable this flag. No DenkMaps/MCP integration is performed.

## Same-workload comparison

The stress_index generator is byte-identical to 0.2.19. The 500k-record fixture retains three clones and edits one. Default peak child RSS is 3,457.09 MiB; flagged RSS is 1,198.97 MiB, a 65.3% reduction in this workload. Default clone times are 217.05, 215.77, 214.05 ms; shared clone times are 15.45, 13.12, 12.20 ms.

The tradeoff is explicit: the first edit of that shared clone takes 86.27 ms versus 0.038 ms after an already-deep default clone. Copying common posting maps moves work into the first affected write. Clones still copy outer maps and keys; they are not O(1). Select the feature for retained generations/copies and evaluate your write latency. No uniform speed improvement is claimed.

At two million records, default/shared peak RSS is 3,624.98/3,408.89 MiB and build time is 5.871/5.885 seconds. Selective p95 is 2.674/2.533 ms; common-term p95 is 531.70/528.18 ms. Score signatures agree. Small samples and host variation do not justify a general query-speed claim. Broad queries remain expensive.

The unchanged four-reader/two-writer 100k snapshot workload also passes with sharing, old-source consistency and stale-writer rejection. Peak RSS is 204.94 MiB. Reader counts vary because readers run until writers finish, so elapsed/read totals are not a fixed-work throughput comparison. The separate 10k positional snapshot profile still fully rebuilds text analysis: metadata-only preparation remains about 86 ms. That is the measured next incremental-computation target, not an improvement claimed here. Handwritten 2D vectors in that profile demonstrate consistency, not neural quality.

Raw outputs/resources and exact executable/source hashes with feature choices are in [measurements/0.2.20](measurements/0.2.20/), including builds.json. The default/flagged runs use the same generator and count/retention pattern. Whole-child RSS includes staging and retained generations, not just live records. Historical 0.2.19 results remain intact.

## Correctness and release gates

The complete suite passes **191 default and 547 all-feature Rust tests**, including compiled documentation, plus five Python helper tests. Three shared-storage integration regressions cover old metadata/removed values, exact scores, many retained generations against authoritative records and eight concurrent independent writers. Two private tests verify shared record/posting/duplicate buffers before edits and correct detachment afterward. Shared-only integration runs three cases. Existing seeded mutation, matching, cache, semantic, concurrent snapshot and retrieval tests run with the flag in all-feature validation.

Strict formatting, default/all-feature/shared-only Clippy, rustdoc, minimum Rust 1.89/1.92, static patch compatibility against 0.2.19, audit and an independent flagged consumer gate release. Extracted package, dry run, exact-commit CI and registry/tag/asset/VCS verification gate publication. No required third-party dependency is added.

Provenance definitions, positional rebuilding, model allocations and outer map copies retain their separate costs. Sharing is in-process immutable ownership, not persistence, cross-process shared memory or external synchronization. See [sharing contract](SHARED-INDEX.md), [API cookbook](API-COOKBOOK.md), [migration](MIGRATION-0.2.20.md) and [audit metadata](measurements/0.2.20/audit.json).
