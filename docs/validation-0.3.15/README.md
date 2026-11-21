# Reproduce AIRust 0.3.15 validation

Run `./test.sh` with Rust 1.92+ and Python 3.11+. Provider-neutral features use Rust 1.89. Scope formatting to `cargo fmt --package airust`. Separately run:

```bash
cargo test --manifest-path crates/airust-pdf-backend/Cargo.toml --test regressions --locked
python3 scripts/check_pdf_backend.py
```

Eleven internal page-tree tests retain the earlier malformed/count/resource cases and add 128 deterministic varied nested trees. Generated subtree totals are calculated independently; a second check corrupts the root total and requires rejection. The backend source remains identical to its published 0.3.5 tag.

Copy `consumer.rs` into `src/main.rs` and `consumer.toml` into `Cargo.toml` of an independent package. Use a path override before publication and the exact registry pin afterward. Run with Rust 1.89. It constructs its own malformed-count PDFs and checks public extraction/preview behavior along with the existing feature APIs.

`corpus.rs` / `corpus.toml` run the read-only knowledge workload:

```bash
cargo run --release --locked -- /absolute/path/to/trusted-knowledge.json 10000 3
```

The program parses the entire trusted export before selecting a prefix. It checks incremental updates against rebuilds, JSON restoration, concurrent readers, retained clones and cancellation/deadline behavior. Full working views use additional RAM. The release gate actually runs a 1,000-record prefix and one epoch; earlier larger-core workloads remain inherited through unchanged relevant code. Literal lexical queries are not semantic labels.

## Isolated metadata memory

Obtain and verify the previous/current registry crates against their release verification assets, then extract them into sibling source directories. The helper source is included in each crate. This reproduction copies only its runtime section and appends the same fake-loader probe to both versions. It does not alter installed AIRust.

```bash
python3 memory.py prepare /absolute/path/airust-0.3.14 /absolute/path/airust-0.3.15 /tmp/airust-tree-memory
cargo +1.89.0 build --release --manifest-path /tmp/airust-tree-memory/Cargo.toml
python3 memory.py run /tmp/airust-tree-memory /tmp/airust-tree-results /tmp/airust-tree-memory/target/release/airust-page-tree-memory-check
```

Use new work/output directories. On macOS `wait4` peak RSS is bytes, as recorded here; Linux uses KiB and requires conversion before comparing the field. Six alternating repeats per case/version produce 24 processes. `prefix` constructs 999,999 references and requires the same actual-page-limit error at leaf two. `full` requires exactly 100,000 leaves. The fake loader constructs real parser Object arrays/dictionaries but does no native file parsing. RSS includes construction, traversal and destruction. This is not million-page capacity or whole-job native memory.

## Complete imports and previews

Fresh complete imports compare frozen 0.3.14/0.3.15 with identical bounded supervisor flags. Thirty-six import runs represent 35 unique PDFs: the newly downloaded PGF snapshot matches the existing source SHA exactly and is a repeated control. It contributes no new canonical entries. MuPDF all-page coverage and selected pypdf queries are fresh for PGF, Racket and Guile. Inspect rendered pages and keep differences/failed literal queries. Reader agreement on nonempty pages does not prove identical glyph decoding or reading order.

For preview comparisons, create a parent directory, extract the verified current crate into `airust-0.3.15/`, and create `preview-consumer/` beside it. Copy `preview.rs` into `preview-consumer/src/main.rs` and `preview.toml` into its manifest. Generate a lockfile, then run both versions in separate processes:

```bash
cargo run --release --locked -- 0.3.14 /absolute/path/manual.pdf
cargo run --release --locked -- 0.3.15 /absolute/path/manual.pdf
```

The previous alias uses the exact registry pin; current uses the verified extracted source path. Two exact compatible patch pins from the same registry conflict in one Cargo graph. The prerelease experiment overrides both aliases with the byte-identical local backend 0.3.5; the published recipe uses their shared registry backend. Record compiler/profile/binary hashes and warm/cold conditions. Preview API timing excludes final signature/stdout; its process RSS includes native parsing. Complete converter parent RSS excludes the worker. Compare paired versions within each workload, after deliberate competing work ends.

Owned malformed fixtures and deterministic byte mutations check bounded failure, complete known witnesses and old-output preservation. Arbitrary accepted mutations have no complete-text oracle. No exhaustive security, OCR, semantic-quality, cold-start or arbitrary-recovery claim follows. Originals, earlier knowledge, setup failures, raw references, renders and measurements stay in ignored `local-lab/pdf-stability`. [Public aggregates](../measurements/0.3.15/) omit source text/private paths. See [results](../CHANGES-0.3.15.md), [API/limits](../PDF-PAGE-TREE-MEMORY.md) and [next-version gates](../PREPARING-0.4.md).
