# AIRust 0.2.5: validated derived-index snapshots

The optional `cache` feature saves derived lexical analysis and, with `embeddings`, model-specific vectors. Original application records remain authoritative. Default features, existing source entries/JSON, retrieval, graph, context and provenance APIs are preserved. No application or MCP integration is added.

`IndexCache` requires a dataset identity and a trusted private path. Loading also requires the complete consistent original-record snapshot. SHA-256 fingerprints include IDs, original text and metadata in sorted ID order. Format, source revision, Unicode-analysis recipe, kind and model checks reject stale state. On every returned load error the original vector remains available for rebuilding without cloning the corpus. Successful loads move those records into reconstructed posting and duplicate trees, reusing analysis and vector bits without inference.

The versioned Postcard format streams into an exclusive mode-0600 temporary file, syncs it, renames it atomically and syncs the parent directory on Unix. Full snapshots replace previous snapshots; in-memory updates retain existing APIs. There is no append journal, automatic SQLite watcher or multi-process transaction protocol. Explicit file, entry, dimension and term-binding limits precede relevant allocations and loops. Checksums detect accidental damage, not a hostile writer able to rehash cache data.

## Validation and review

Integration checks compare exact BM25 and TF-IDF score bits, matching, duplicates, relationships, metadata filters and semantic scores before/after restart. Repeated mutation/restart sequences compare against a fresh authoritative rebuild. Invalid source/model/identity changes, duplicate and malformed records, corrupt/truncated/oversize/trailing files, symlinks, permissions, failed-save preservation and failed-load ownership are exercised. Private-format tests forge correct checksums around invalid counts, IDs, token frequencies, analysis metadata and vectors so structural checks are tested independently of the checksum.

A streaming-write failure leaves the previous file intact and cleans its temporary output. A real child process is killed after writing/syncing a partial temporary snapshot but before rename; the previous published file remains intact and a later write recovers. The ignored child entry point is launched explicitly by that passing parent test. No production crash hook or unsafe code is introduced. Review added bounded reads with an extra-byte probe and early document/vector count checks.

## Reproducible startup measurements

Release-mode synthetic records contain roughly 450 bytes of multilingual text plus small metadata. Original-record generation is outside the timers. Build consumes records; save streams a complete snapshot; the original engine is dropped before a separately generated original snapshot is loaded. BM25/TF-IDF queries are compared bit-for-bit across restart. macOS ARM64, 64 GiB RAM, Rust 1.98.0; one run per size, with normal background activity and durable file sync costs.

| Entries | Build (ms) | Save (ms) | Load (ms) | Cache bytes | Peak RSS (MiB) |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1,000 | 31.804 | 126.232 | 10.250 | 563,955 | 9.69 |
| 10,000 | 215.516 | 582.717 | 117.194 | 5,657,956 | 72.77 |
| 100,000 | 2462.790 | 5915.828 | 1366.981 | 56,777,958 | 700.34 |

```bash
cargo build --locked --release --features cache --example cache_benchmark
/usr/bin/time -l target/release/examples/cache_benchmark 1000
/usr/bin/time -l target/release/examples/cache_benchmark 10000
/usr/bin/time -l target/release/examples/cache_benchmark 100000
```

Raw output and OS counters are in [measurements/0.2.5](measurements/0.2.5/). Load includes full source fingerprinting, validation and posting-tree reconstruction. Peak memory includes allocator retention, original records, encoded input and restoration staging; it is not a minimal steady-state footprint. These runs show a benefit for this workload, not a universal startup improvement, p95 estimate or production capacity. No embedding inference is timed; vector restoration is verified separately.

## Recovery and limits

Rebuild from originals on a missing, stale or damaged cache, then save a new snapshot. A cache directory can contain normalized private text and must be application-controlled. Model recipe IDs must identify the complete inference recipe. Graph/provenance data remain separately caller-owned. File/count limits constrain encoded data, not exact peak heap. Concurrent processes must coordinate committed source revisions; last-writer-wins atomic snapshots do not provide transaction isolation. See [CACHE.md](CACHE.md) and [migration notes](MIGRATION-0.2.5.md).

## Release checks

The complete script passes 189 default tests; the final all-feature run passes 314 tests with one intentionally ignored child-process entry point exercised by its parent test. Formatting, strict Clippy, rustdoc, isolated features, examples, Rust 1.89 provider-independent checks and Rust 1.92 complete-feature checks pass. An independent Rust 1.89 consumer uses the existing APIs and saves/restores semantic vectors with the new cache. The dependency audit reports zero known vulnerabilities and the unchanged optional unmaintained `paste` and `ttf-parser` warnings. Postcard/cobs add no reported advisories in the checked RustSec database.

Clean packaging, extracted-package tests, candidate CI, publish dry run and published artifact verification gate this release and the next stage. Artifacts: [GitHub v0.2.5](https://github.com/LEVOGNE/airust/releases/tag/v0.2.5), [crates.io 0.2.5](https://crates.io/crates/airust/0.2.5).
