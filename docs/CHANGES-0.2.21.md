# AIRust 0.2.21: incremental text computation

Existing lexical/text upserts and prepared datasets now retain derived text data when the original text is unchanged. Metadata is updated in every projection without removing/rebuilding terms or positions. Positional snapshots clone/update affected records and can reuse the complete old projection for source-only changes. TextIndex gains independent Clone; shared-index optionally shares token vectors. No new mandatory module, dependency or feature is added. No DenkMaps/MCP integration is performed.

Original text byte changes always refresh analysis, even when normalization gives identical terms. This preserves phrase/excerpt coordinates across fullwidth/composed characters or spacing changes. Model refresh still deliberately rebuilds requested vectors. Literal-text vector reuse from earlier releases is preserved. Diagnostic non-exhaustive PreparationStats adds reused/metadata-only/unchanged counts; rebuilt counts now represent actual reanalysis. Existing persisted records/cache/response JSON remain unchanged; explicitly serialized work receipts gain additive fields.

## Identical-workload measurements

The unchanged 0.2.20 10k positional snapshot benchmark reports the following one-observation values, using the same feature choices and handwritten 2D consistency vectors:

| Phase | 0.2.20 full rebuilding (ms) | 0.2.21 incremental (ms) |
| --- | ---: | ---: |
| Initial construction | 132.01 | 133.20 |
| 100 changed records | 87.44 | 12.26 |
| Metadata-only update | 86.08 | 5.16 |
| Deletion | 84.51 | 8.11 |

Whole-child peak RSS falls from 358.98 to 229.06 MiB with old readers and competing prepared snapshots retained. Metadata still invokes zero embeddings. Initial construction still analyzes the corpus; its small timing variation is not an optimization claim. Preparation includes source/index map staging, so it is not proportional only to changed-record count. No neural quality or general latency guarantee is measured. Raw resources and exact source/binary hashes: [measurements/0.2.21](measurements/0.2.21/).

```bash
cargo build --release --locked --features shared-index,snapshots,advanced-search --example snapshot_benchmark
python3 scripts/measure.py /tmp/incremental.json target/release/examples/snapshot_benchmark 10000 text
cargo run --locked --features snapshots,advanced-search --example incremental
```

## Verification and bug hunting

The complete suite passes **194 default and 558 all-feature Rust tests**, including compiled documentation, plus five Python helper tests. Ten focused incremental cases cover 600 seeded updates against complete German/phrase rebuilds, retained views, current metadata/source/model receipts, full reconciliation, changed original positions, failed analysis and cancellation. Source-only updates assert reuse of the complete positional object. The projection-only suite without sharing/execution runs six cases.

An exact-payload regression prevents metadata shortcuts from treating -0.0/0.0, integer/float encodings or serialized object order as interchangeable. An iterative metadata comparison preserves the previous returned value and the new serialized payload. Normalized-equal text still rebuilds coordinates, and invalid records cannot enter the fast path. Existing scalar/vector/source concurrency, cache, retrieval and matching tests remain enabled.

Strict formatting, all-target default/all-feature/shared Clippy, rustdoc, minimum Rust 1.89/1.92, patch compatibility against 0.2.20, audit and an independent consumer gate release. Extracted package/dry run, exact-commit CI and registry/tag/asset/VCS checks gate publication.

Source lists still replace complete bindings on every put; omitting them clears old references, including metadata-only record updates. The engine does not infer external transactions, keep every specialized graph/cache/HNSW component synchronized or guarantee source truth. Map/source cloning and one large analyzer/provider call retain their costs. See [incremental contract](INCREMENTAL.md), [API cookbook](API-COOKBOOK.md), [migration](MIGRATION-0.2.21.md) and [audit metadata](measurements/0.2.21/audit.json).
