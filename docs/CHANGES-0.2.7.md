# AIRust 0.2.7: revisions, conflicts and optional history

The independent `versioning` feature adds revision ownership over an existing lexical engine. It adds no dependency, uses shared immutable change metadata, and enables no model, graph, HTTP client or database. Default features, original entry fields/JSON, retrieval methods, derived cache and provenance formats stay unchanged. No external application or MCP integration is added.

Writes require a never-seen ID or an exact current revision. Deletions retain tombstones, preventing stale delete/recreate writes from silently succeeding. Revision sequence arithmetic is checked before mutation. Caller-supplied actor/reason/metadata describe changes; they do not prove identity, time or audit authenticity. Imported original text is moved into the wrapper without a second copy; bulk metadata is shared.

History defaults to zero. Opt into a global committed-snapshot count when old states must remain available. Text is copied only for enabled retained snapshots. Pruning is explicit and bounded by record count, not bytes; increasing capacity does not recreate past snapshots. The journal records future commits after activation, including deletion states. `at` borrows an available state, `compare` reports field/existence changes, and `restore` creates a new revision from a current or retained live/deleted state. It never rewinds the sequence.

Versioned JSON format 1 validates original entries, unique revision heads, live/deleted agreement, actions, current snapshots, journal identities/order and counter consistency. Standalone revision/metadata/version/record deserialization enforces its own applicable invariants. Default input is capped at 16 MiB, with an explicit override; generic Serde uses caller resource policy. Loading reconstructs the lexical index from current originals. This format remains separate from portable entry JSON and the derived cache.

## Validation and review

Thirteen new integration tests cover disabled/bounded history, shared metadata and moved text, repeated edits, stale writes, delete/recreate, restoration of live and deletion states, comparison, missing/pruned revisions, metadata/Unicode limits, round trips, generic Serde and malicious/inconsistent format mutations. A 300-step sequence compares current source data and exact search scores against an independent state map and fresh index. Real concurrent Rust threads using the caller's mutex and the same expected revision produce exactly one winner and one conflict. Maximum-u64 imports reject further writes without changing any state.

Review tightened standalone deserialization so a valid outer snapshot is not the only path enforcing action/deletion and record consistency. Import validation avoids repeating normalization for already validated current entries. Restore captures an available state before mutation and assigns new change metadata. Tombstones and heads are retained independently of history pruning; their memory grows with tracked IDs, which is documented rather than hidden behind the journal quota.

The complete release script passes **189 default tests** and **339 all-feature tests**, with one existing intentionally ignored crash-helper entry point exercised by its parent. Formatting, strict Clippy, rustdoc, isolated features and examples pass. Rust 1.89 checks provider-independent features including revisions; Rust 1.92 checks the complete feature set. An independent Rust 1.89 consumer compiles existing APIs plus revision writes, comparisons, restore and versioned round trips. Audit: zero known vulnerabilities and the unchanged optional unmaintained `paste`/`ttf-parser` warnings.

## Reproducible costs

The release benchmark generates multilingual source entries, builds the lexical engine, moves it into revision tracking, then performs up to 1,000 replacements/deletions. History is disabled or capped globally at 128 snapshots. It exports versioned JSON, drops the original index, restores it and compares exact BM25/TF-IDF query score bits, tombstones, sequence and retained count. Original-record generation is included in lexical-build timing. JSON import explicitly permits the generated file size; the 100k snapshots exceed the default input cap.

Host: Apple M5 Max, 18 logical CPUs, 64 GiB memory, macOS ARM64, Rust 1.98.0. One process/run per size and history policy, release build, no concurrent compilation, no inference. Raw output, CPU/wall counters and environment metadata are in [measurements/0.2.7](measurements/0.2.7/).

| Initial entries | History capacity | Lexical build (ms) | Revision setup (ms) | Changes (ms) | JSON bytes | Export (ms) | Restore (ms) | Peak RSS (MiB) |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1,000 | 0 | 12.452 | 0.208 | 4.037 | 196,804 | 0.229 | 2.675 | 7.66 |
| 10,000 | 0 | 71.415 | 1.162 | 3.346 | 4,530,512 | 4.143 | 61.140 | 60.12 |
| 100,000 | 0 | 664.976 | 14.415 | 6.015 | 48,038,572 | 41.759 | 759.993 | 576.73 |
| 1,000 | 128 | 7.151 | 0.116 | 3.083 | 223,187 | 0.174 | 1.905 | 7.75 |
| 10,000 | 128 | 68.275 | 1.254 | 3.694 | 4,557,023 | 4.595 | 68.028 | 60.31 |
| 100,000 | 128 | 732.611 | 15.370 | 6.091 | 48,065,211 | 38.833 | 808.391 | 576.62 |

```bash
cargo build --locked --release --features versioning --example versioning_benchmark
/usr/bin/time -l target/release/examples/versioning_benchmark 1000 0
/usr/bin/time -l target/release/examples/versioning_benchmark 10000 128
/usr/bin/time -l target/release/examples/versioning_benchmark 100000 128
```

Peak memory includes current data, heads, JSON, allocator retention and restoration staging. Single-run differences are not statistically significant evidence that one history policy is faster. Large records/metadata and larger quotas change the costs; the snapshot-count limit is not a memory ceiling. These checks establish no production database throughput or application speedup.

## Caller boundaries

An application owns transactions, permissions, durable history and coordination. Restore does not update SQLite, refresh embeddings, repair graph links or validate old provenance automatically. Whole-index snapshot rollback or a rebuilt/cloned independent lineage requires a new application coordination epoch and discarded old expected revisions. Revisions are not global identities or a tamper-proof log. See [VERSIONING.md](VERSIONING.md), [migration notes](MIGRATION-0.2.7.md) and the [plan](PLAN-0.2.7.md).

Clean package/extracted tests, candidate CI, dry run and registry/tag/asset verification gate publication and the next stage. Artifacts: [GitHub v0.2.7](https://github.com/LEVOGNE/airust/releases/tag/v0.2.7), [crates.io 0.2.7](https://crates.io/crates/airust/0.2.7).
