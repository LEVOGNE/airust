# AIRust 0.2.14: coherent search, vectors and sources

The optional snapshots feature adds DatasetManager, immutable DatasetSnapshot readers and checked prepared updates. One published revision covers the lexical search owner, optional vectors and source references. An optional configured TextIndex joins the same boundary. Existing default features, specialized APIs, excerpt behavior and persisted formats remain compatible. No DenkMaps/MCP integration is performed.

Preparation validates changes, stages source bindings and computes necessary vectors outside publication locks. Publishing compares the complete base revision and swaps one Arc only if it is still current. Concurrent losers cannot overwrite a newer dataset. Older readers keep the complete old version. Exact unchanged text shares its vector buffer for metadata/source edits; changed text requires inference. Model enable/replace/refresh explicitly rebuilds every vector. Omitted source lists clear previous bindings instead of silently retaining citations after edits.

`prepare_replace` reconciles a full source image and deletes missing IDs from search, vectors and sources together. An optional separate external StoreRevision must advance within its lineage. It is a caller assertion, not a database watcher or atomic transaction between external storage and RAM. The [complete contract](SNAPSHOTS.md), [migration](MIGRATION-0.2.14.md), [API cookbook](API-COOKBOOK.md) and documentation-only [proposed application guide](DENKMAPS-UPGRADE.md) describe ownership and integration boundaries.

## Correctness and bug hunting

The full suite passes **189 default and 482 all-feature Rust tests**, including compiled documentation, plus five Python helper tests. Twenty-two snapshot integration tests and two internal boundary tests cover coherent old/new readers, vector bit/buffer reuse, metadata/source changes, removal/reinsertion, full reconciliation beyond delta caps, shared source-definition conflicts, optional positional projection, model changes and provider failures. The isolated snapshots-only integration suite passes twenty tests.

Concurrency cases use twelve writers prepared against one base and assert exactly one winner. Four readers each capture six hundred views during repeated publication and verify matching text/vector/source values. A deliberately blocked provider permits another reader/writer to proceed; its later stale result is rejected. Provider failure after an earlier successful batch, wrong vector counts, non-finite vectors, changed/wrong spaces and unwinding panics leave the published view intact. Lock poisoning fails closed. Generation exhaustion preserves the current pointer.

Two hundred sixty seeded update/reconciliation steps are compared with an independent record/source map while retained older views are checked. Existing memory storage supplies consistent source records and committed markers for full reconciliation, including a deleted record. Existing SQLite adapter tests also rerun; no real application SQLite synchronization was tested. Relevant excerpts, source lookup and optional sourced context are composed from a single retained view. External source-file coordinates remain external coordinates, not incorrectly validated against shorter knowledge excerpts.

Review corrected operation-order-dependent capacity checks: limits apply to the final batch state. Full replacement validates identifier lengths before comparisons and rejects excessive positive record payloads before traversing later records. Old-snapshot destruction and stale-candidate release happen outside the publication lock. Metadata/source encoding, depth/node counts, dimensions, published vector bytes, duplicate IDs and configuration limits have targeted failure checks. These are executed consistency/boundary tests, not a completed independent security audit.

Strict formatting, all-target Clippy, rustdoc, isolated feature combinations, Rust 1.89 provider-independent and Rust 1.92 complete builds pass. An independent Rust 1.89 consumer exercises earlier APIs/literals and the new owner with German positional search, vectors, sources, retained views and stale publication. Static all-feature patch compatibility against 0.2.13 passes 229 checks with 31 skipped. Native model relevance/inference benchmarks were not repeated; handwritten vectors test consistency only.

## Reproducible preparation and retention costs

A release example uses short public synthetic records, one source reference per record and two-dimensional handwritten vectors. It performs a full initial load, one hundred text changes, one hundred metadata changes, one hundred deletions and a rejected competing publication. Generation occurs outside preparation timing. Each preparation phase is a single observation; acquisition is an average of ten thousand iterations. Publication measurements retain old views and exclude their later destruction.

| Entries | Positional projection | Initial prepare ms | 100 text changes ms | 100 metadata changes ms | 100 deletions ms | Whole-process peak RSS MiB |
| ---: | --- | ---: | ---: | ---: | ---: | ---: |
| 10,000 | Off | 61.419 | 10.580 | 8.659 | 8.121 | 243.75 |
| 100,000 | Off | 577.758 | 114.406 | 98.292 | 100.198 | 2,408.27 |
| 10,000 | German TextIndex | 130.459 | 89.251 | 86.972 | 85.790 | 451.59 |

Metadata changes embed zero entries. Text changes embed exactly one hundred and reuse the other vector buffers. The optional positional projection currently rebuilds the complete source image for record changes, explaining its extra cost. Captured publication observations were approximately 0.0002–0.0004 ms and uncontended snapshot acquisition averages approximately 7.6–8.6 ns. These tiny timing samples are not stable tail-latency claims or an end-to-end model workload.

Peak memory includes multiple deliberately retained generations, two competing candidates, source/index bookkeeping and all phases. It is **not** minimum steady-state memory or a hard heap cap. Source/index staging copies data even when vectors are shared. The 100k case demonstrates why callers must bound retained readers and preparation queues. No 100k positional or million-entry run was performed, and no production capacity or relevance guarantee is inferred.

Host: Apple M5 Max, macOS ARM64, 18 logical CPUs, 64 GiB RAM, Rust 1.98.0. Raw timings, arguments and OS CPU/RSS: [measurements/0.2.14](measurements/0.2.14/).

```bash
cargo build --release --locked --features snapshots,advanced-search --example snapshot_benchmark
python3 scripts/measure.py /tmp/snapshots-100000.json target/release/examples/snapshot_benchmark 100000
python3 scripts/measure.py /tmp/snapshots-text-10000.json target/release/examples/snapshot_benchmark 10000 text
```

## Dependencies, limits and publication

Snapshots compose the existing storage-validation, provider-neutral embedding and provenance features; no additional dependency version is introduced. Defaults remain empty and no SQLite/web/native-model feature is enabled. Core checks remain Rust 1.89; optional native adapters retain Rust 1.92. The default snapshot configuration bounds 100k records, 128 MiB encoded records/source lists, 256 MiB f32 components and batch/source/metadata limits. These do not count all tree/provider/retained-generation memory.

Fresh RustSec reports zero known vulnerabilities and the same optional unmaintained notices for chrono, paste and ttf-parser, recorded with the database revision in [audit metadata](measurements/0.2.14/audit.json). Coherence does not prove source truth, provider meaning, authorization or external freshness. Graph/HNSW/cache/history and combined transport-envelope budgets remain explicitly separate. A DatasetSnapshot is an in-memory read view, not a new persistent backup format.

Clean package/extracted tests, publish dry run, exact-commit GitHub CI and matching registry/tag/asset digests gate publication. [GitHub release](https://github.com/LEVOGNE/airust/releases/tag/v0.2.14), [crates.io](https://crates.io/crates/airust/0.2.14).
