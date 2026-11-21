# Reproduce AIRust 0.3.11 validation

Use `./test.sh` with Rust 1.92+ and Python 3.11+. Provider-neutral features use Rust 1.89. Scope formatting to `cargo fmt --package airust`; preserve the unchanged backend's upstream formatting/notices. Separately verify its provenance and ten regressions.

Three new token-matching tests compare all returned fields with complete `matching::compare` results over Unicode, thresholds, zero/small/large limits, ties, updates and JSON round trips. Frozen old/current default-feature binaries run the same fixed cases on the same read-only canonical export; compare IDs/order and the exact floating-point bits for scores and both similarities. Real-data content and raw outputs remain local. Final paired timing runs alternate versions after intentional build/test work ends; index build and method time are separate.

## Independent consumers

`consumer.rs` and `consumer.toml` exercise the published APIs from a separate Rust 1.89 package. Copy them into `src/main.rs` and `Cargo.toml` of a new temporary package. Use a local dependency override before publication; use the exact registry dependency afterward.

`corpus.rs` and `corpus.toml` provide the reusable real-knowledge benchmark. Copy them into another temporary package. From that package, run:

```bash
cargo run --release --locked -- /absolute/path/to/trusted-knowledge.json 10000 3
```

Create its initial lockfile before using `--locked`. Input is an AIRust generic-entry JSON export, not PDF bytes or legacy training-example JSON. Choose an entry limit from 1,000 to 200,000 and 1–32 rounds. The process first parses the complete trusted input before selecting the prefix. Input/host resources remain caller-controlled. Begin with a smaller corpus on limited machines; full comparison views and rebuilt indexes consume additional memory.

The tool checks updates against a complete rebuild, restored original entries, JSON reconstruction, two lexical ranking algorithms, four concurrent readers, pre-cancelled/expired controlled searches and isolated clones. Timings distinguish BM25/TF-IDF; they include correctness overhead where stated. Fixed queries are not semantic relevance labels. Temporary mutations stay in memory; the source export is never written. Observe process peak RSS separately and record build profile, binary/source hash and host conditions. Repeated warm-cache measurements do not prove cold-start performance or absence of every memory leak.

## Release gates and scope

Local gates cover tests/examples, strict lint/docs, 37 isolated features, minimum Rust versions, patch compatibility and dated dependency audits. Clean packaging, extracted-crate tests, publish dry-run, exact-commit Linux CI, registry-only consumers and byte/VCS/tag/asset identity finish publication. Actual final outcomes remain in the release verification asset.

Original PDFs, reference extraction/renders, canonical knowledge, mutation fixtures and raw measurements remain retained in ignored `local-lab/pdf-stability`. [Public aggregates](../measurements/0.3.11/) omit private text/source paths. See [API/limits](../TOKEN-MATCHING.md), [release results](../CHANGES-0.3.11.md) and [0.3.x gates](../STABILITY-0.3.md). Native PDF/cleaning proof from 0.3.10 is inherited only where those code paths and exact artifacts remain unchanged; any new runs are identified separately. No Docker, exhaustive security proof or DenkMaps/MCP integration is claimed.
