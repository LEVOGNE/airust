# Reproduce AIRust 0.3.14 validation

Run `./test.sh` with Rust 1.92+ and Python 3.11+. Provider-neutral features use Rust 1.89. Scope formatting to `cargo fmt --package airust`; preserve unchanged backend formatting and notices. Separately run its 14 regressions and provenance checker:

```bash
cargo test --manifest-path crates/airust-pdf-backend/Cargo.toml --test regressions --locked
python3 scripts/check_pdf_backend.py
```

Ten internal metadata tests count actual leaves through a bounded loader, including incorrect intermediate totals, empty branches, indirect counters, duplicate/cyclic/generation aliases and depth/reference/page budgets. Public tests use five owned PDFs and require complete ordered witnesses, failure before prefix preview, actual page limits and old-output preservation in direct/supervised modes. The backend source remains identical to its published 0.3.5 tag.

Copy `consumer.rs` into `src/main.rs` and `consumer.toml` into `Cargo.toml` of an independent package. Use a local override before publication and the exact registry pin afterward. Run it with Rust 1.89. It constructs separate two-page documents with declared counts 1/2/3 and checks the public loader/preview contract.

`corpus.rs` / `corpus.toml` retain the read-only knowledge workload. Create its lockfile and run:

```bash
cargo run --release --locked -- /absolute/path/to/trusted-knowledge.json 10000 3
```

It parses the entire trusted export before selecting its prefix, then checks updates against rebuilds, JSON restoration, concurrent readers, retained clones and cancellation/deadline behavior. Full comparison views use additional RAM. Its fixed lexical queries are not semantic relevance labels.

Fresh native checks compare complete outputs for all 33 identified PDFs with frozen 0.3.13. Authored tree fixtures require every known witness on success and explicit failure for inconsistent metadata. Arbitrary byte mutations test containment and failure atomicity, not complete-text fidelity of every accepted mutant. A million repeated edges exercise the fixed reference-work budget, not valid million-page capacity.

Measure complete converter and bounded-prefix preview costs separately using alternating fresh release processes after deliberate competing work ends. Record compiler/profile/features, binary/source hashes, warm/cold conditions and complete result signatures. Converter parent RSS excludes its native worker's peak; a library preview process contains its own parser. Keep those categories separate.

Originals, prior knowledge, references, all candidate imports, failed harness assumptions and raw measurements remain in ignored `local-lab/pdf-stability`. [Public aggregates](../measurements/0.3.14/) omit private text and source paths. Clean package/extracted tests, dry-run, exact-commit Linux CI, registry consumers and matching artifact identities finish publication. See [results](../CHANGES-0.3.14.md) and [API/limits](../PDF-PAGE-COUNTS.md).


For separate preview comparisons, copy `preview.rs` to `src/main.rs` and `preview.toml` to `Cargo.toml` of another independent package. Generate its lock, then run `cargo run --release --locked -- 0.3.13 /absolute/path/manual.pdf` and repeat with `0.3.14`. Each fresh process selects one aliased library version, requests one usable page and prints method time plus a complete text/physical-page signature. The published recipe uses the exact registry versions and their common backend 0.3.5. The prerelease experiment overrides both backend dependencies with that byte-identical local source. This library harness uses Cargo's default release profile; the converter uses the repository's thin-LTO release profile. Compare paired versions within each workload.
