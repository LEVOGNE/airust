# AIRust 0.2.11: bounded original-record storage

Optional `storage` introduces an object-safe StorageBackend, MemoryStore, complete StoreRevision tokens, atomic Put/Delete batches, bounded sorted pages and consistent snapshot loading. `sqlite-storage` adds a standalone versioned SQLite adapter without enabling web. It reuses rusqlite 0.43/bundled SQLite already present as an optional dependency; getrandom 0.4.3 supplies 128-bit lineage identities. No upstream implementation was copied. Default features and existing index/cache formats remain unchanged. No DenkMaps/MCP integration is performed.

This is record persistence, not a persistent inverted/vector index. Snapshot loading still constructs a RAM Engine. Storage, vector freshness, graph/source sidecars and derived view swapping require explicit application coordination. Independent restores/copies require identity renewal before accepting old tokens. The [storage contract](STORAGE.md) and [migration](MIGRATION-0.2.11.md) explain transaction, cursor and backup boundaries.

## Tests and fault checks

The full release suite passes **189 default and 414 all-feature Rust tests**, including compiled documentation. Five Python helper tests also pass. Fifteen new storage tests exercise both adapters through one conformance suite. Four hundred seeded commits per adapter are checked against an independent BTreeMap and periodic full snapshots. Cases include UTF-8 ID ordering, byte-deferred pages, malformed third-party pages, writes during snapshot reads, cross-store/stale tokens, duplicate changes, metadata/encoded-entry budgets and no partial invalid batches.

SQLite-specific checks cover file reopening, independent namespaces/connections, read-only failures, private Unix files and symlink refusal. A deliberately injected SQL trigger aborts a later operation: earlier rows and the revision roll back. Two concurrent connections with the same expected token yield exactly one winner. Generation exhaustion preserves state; identity renewal recovers the lineage. Unknown/unrelated schemas, damaged JSON and mismatched stored IDs fail explicitly. A page test initially assumed two records would exceed 90 bytes when they fit; its fixture was corrected to 60 bytes and now demonstrates deferral without skipped records. No untested crash/power-loss guarantee is claimed.

Strict formatting, all-target Clippy, rustdoc, isolated features, the independent Rust 1.89 consumer and Rust 1.89/1.92 builds pass. All-feature static cargo-semver-checks against 0.2.10 passes 229 checks with 31 skipped for a patch release. Existing behavior/defaults remain unchanged. Fresh RustSec reports zero known vulnerabilities and the same two optional unmaintained warnings, recorded in [audit metadata](measurements/0.2.11/audit.json). These checks are not an independent security audit.

## Reproducible synthetic scale check

One release process runs memory then SQLite adapters with generated short records. It writes batches of at most 1,000, pages every row, performs 1,000 point reads, builds a snapshot-derived Engine, updates 100 records and verifies SQLite reopening. SQLite uses WAL/FULL. Record generation and validation are included in write time; opening is outside initial write timing. These are single observations, not latency distributions or production throughput claims.

| Records | Adapter | Write all ms | Page all ms | 1,000 gets ms | Snapshot + RAM index ms | Update 100 ms |
| ---: | --- | ---: | ---: | ---: | ---: | ---: |
| 10,000 | Memory | 23.002 | 16.509 | 1.599 | 60.882 | 0.092 |
| 10,000 | SQLite | 31.524 | 13.165 | 5.214 | 57.057 | 0.656 |
| 100,000 | Memory | 138.442 | 117.692 | 1.279 | 596.072 | 0.069 |
| 100,000 | SQLite | 328.825 | 140.048 | 5.628 | 618.619 | 0.660 |

Peak RSS was 37.75 / 318.19 MiB for the complete 10k / 100k process, including source records, transient snapshots and the derived RAM index. SQLite reopen plus revision/point-read verification took 0.410 / 0.534 ms. Host: Apple M5 Max, 18 logical CPUs, 64 GiB RAM, macOS ARM64, Rust 1.98.0. No real application corpus or million-record run was used. Raw data and OS resource metadata are in [measurements/0.2.11](measurements/0.2.11/).

```bash
cargo build --release --locked --features sqlite-storage --example storage_benchmark
python3 scripts/measure.py /tmp/storage-100000.json target/release/examples/storage_benchmark 100000
```

Clean-package extraction/tests, publish dry run, exact-commit GitHub CI and matching registry/tag/asset digests gate publication. [GitHub release](https://github.com/LEVOGNE/airust/releases/tag/v0.2.11), [crates.io](https://crates.io/crates/airust/0.2.11), [complete API cookbook](API-COOKBOOK.md).
