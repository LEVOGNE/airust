# Reproduce AIRust 0.4.0 validation

Run `./test.sh` using Rust 1.92+ and Python 3.11+. Provider-neutral features support Rust 1.89. Scope formatting to `cargo fmt --package airust`; preserve the unchanged backend's notices and source formatting.

`cargo test --features documents,pdf,retrieval,snapshots --test documents --locked` covers 26 cases. They include 4,096 deterministic manifest mutations, an independently calculated Python format fixture, exact Unicode boundaries, source/text/recipe/page changes, checked JSON, shared clones/readers, core retrieval projection and staged dataset replacement. The `documents` feature alone runs 23 cases and does not enable PDF, retrieval, storage or models.

For the backend, run its 14 regressions and provenance checker:

```bash
cargo test --manifest-path crates/airust-pdf-backend/Cargo.toml --test regressions --locked
python3 scripts/check_pdf_backend.py
```

## Independent consumers

Copy `consumer.toml` into `Cargo.toml` and `consumer.rs` into `src/main.rs` of a new package. Run with Rust 1.89. This checks the existing APIs and new document identity, restoration, Unicode, provenance, retrieval and dataset composition. Before publication use a checkout override; afterward use the exact registry pin.

`corpus.toml` / `corpus.rs` run the legacy read-only knowledge workload:

```bash
cargo run --release --locked -- /absolute/path/trusted-knowledge.json 1000 1
```

The release gate parses all 167,942 retained entries before selecting a 1,000-entry working prefix for one checked epoch. It verifies rebuild/update equivalence, JSON, concurrent readers, retained clones and cooperative stops. Earlier larger core measurements are inherited through unchanged source; this is not a fresh full-corpus index stress run.

## Real document positions and costs

Create another independent package using `documents.toml` / `documents.rs`. Build with `cargo build --release`. The current 0.4 and previous 0.3 registry pins are different Cargo compatibility ranges; both can coexist. There is no unpublished backend override in the public manifest.

Create a query JSON file, for example `[{"query":"memory ownership"},{"query":"type inference"}]`. These are literal probes, not human relevance labels. With a trusted PDF, run:

```bash
python3 measure.py /absolute/path/target/release/airust-documents-validation pdf /absolute/path/manual.pdf /absolute/path/queries.json /tmp/new-document-check manual:42
python3 measure.py /absolute/path/target/release/airust-documents-validation restore /tmp/new-document-check/snapshot.json /absolute/path/queries.json /tmp/new-document-restore
```

Use new output paths. `pdf` mode fingerprints the source, extracts bounded native pages and builds a new revisioned snapshot. Before/after fingerprints check that the tested source remained unchanged; applications must still supply a stable original rather than treat this as an adversarial file-lock guarantee. `restore` mode reuses the retained snapshot and performs no native extraction.

Both modes check every provided full-page position, three selected relative UTF-8 ranges per page, source references, passage JSON restoration and entry metadata. They compare complete ranked IDs, score bits and matched terms between both core versions on identical new page entries. They test a changed-source rejection, 10,000 shared clones and an entry JSON round trip. These page entries differ in granularity from legacy 1,000-character chunks; this comparison does not establish identical rankings across those different corpora or better semantic quality.

The runner records complete process wall/peak RSS and library operation timings. It uses a 240-second outer watchdog, 200-second CPU budget and 600 MiB output-file cap. RSS is normalized to bytes for macOS/Linux. The process retains multiple snapshots/JSON buffers, page entries and both SDK indexes; its peak is not the memory requirement of one production snapshot or index. Limits do not provide an OS RAM sandbox.

Repeat restore cases in fresh processes after deliberate competing builds/checks end; retain every sample and alternate case order. Record compiler/profile/input/source/binary hashes and warm/cold conditions. Three warm repeats do not establish tail-latency, cold-start or general capacity guarantees. The actual release uses the 35 identified retained PDFs; private original text and reference-query lists remain local. Public sources/hashes and source-free aggregates are under [measurements](../measurements/0.4.0/).

The release's prerelease harness uses the local checkout and the byte-identical local backend 0.3.5. Registry consumers use published sources and verify checksums/VCS identities. Clean package/extracted tests, dry-run, exact-commit Linux CI and matching tag/crate/assets gate publication. See [executed results](../CHANGES-0.4.0.md), [API and limits](../DOCUMENTS.md) and [API cookbook](../API-COOKBOOK.md). No DenkMaps/MCP integration.
