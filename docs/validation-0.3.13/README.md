# Reproduce AIRust 0.3.13 validation

Run `./test.sh` with Rust 1.92+ and Python 3.11+. Provider-neutral features use Rust 1.89. Format only `cargo fmt --package airust`; preserve upstream formatting and notices. Separately run:

```bash
cargo test --manifest-path crates/airust-pdf-backend/Cargo.toml --test regressions --locked
cargo check --manifest-path crates/airust-pdf-backend/Cargo.toml --features c-api --lib --locked
python3 scripts/check_pdf_backend.py
```

The backend has 14 owned regressions. Its new tests generate supported/excessive nesting, 499/500/501 Form processing cases and fallback counts. Repeated calls across text/spans/words/characters must fail explicitly or return every known text witness when valid caches permit completion. The main library tests include three owned PDFs; complete supported text and old-output preservation are required.

Copy `consumer.rs` into `src/main.rs` and `consumer.toml` into `Cargo.toml` of an independent temporary package. Use a local override before publication and the exact registry dependency afterward. It constructs its own Form fixtures through the public loader API under Rust 1.89. `corpus.rs` / `corpus.toml` provide the existing read-only real-knowledge workload:

```bash
cargo run --release --locked -- /absolute/path/to/trusted-knowledge.json 10000 3
```

Create the initial lock before `--locked`. The program parses the complete trusted export before choosing its prefix, then compares incremental updates with rebuilds, JSON restoration, concurrent readers, retained clones and cancellation/deadline behavior. Begin with smaller inputs on limited machines; the comparison retains multiple full views. Fixed lexical queries do not provide semantic relevance labels. See the [0.3.12 resource experiment](../CHANGES-0.3.12.md).

Fresh native checks reuse identified source hashes, compare complete outputs against frozen 0.3.12, require all owned witnesses on successful structural probes and retain old output on failure. Deterministic damaged-input cases test normal parent exit, worker deadlines and failure atomicity; they do not prove arbitrary accepted mutations contain all original text. Observe RSS and wall time in separate, alternating fresh release processes after deliberate competing work ends. Identify sources, compiler, features, binary hashes and warm/cold conditions.

All original PDFs, earlier exports, rejected prototypes, references, fixtures and raw results stay in ignored `local-lab/pdf-stability`. [Public aggregates](../measurements/0.3.13/) omit private document text/paths. Clean package/extracted tests, exact-commit Linux CI, Rust 1.89 registry consumers and matching crate/VCS/tag/assets finish publication. See [results](../CHANGES-0.3.13.md), [API/limits](../PDF-STRUCTURE-LIMITS.md) and [stability gates](../STABILITY-0.3.md).
