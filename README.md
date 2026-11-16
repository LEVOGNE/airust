# AIRust 0.1.9

**A Modular Knowledge Intelligence Engine**

AIRust is a Rust library for finding information, comparing text, and suggesting possible connections. Give it entries with IDs, text, and optional metadata. Your application decides how to display and store the results.

Version 0.1.9 adds English documentation and interfaces, safer persistence and process controls, and hardened optional web tools. See the [release notes](https://github.com/LEVOGNE/airust/releases/tag/v0.1.9) and [crates.io package](https://crates.io/crates/airust/0.1.9).

## What it does

| Operation | Result | Useful for |
| --- | --- | --- |
| `search()` | Relevant entries ranked with BM25. TF-IDF is also available. | Finding notes and document passages. |
| `find_matches()` | Exact matches, spelling similarity, or shared words. Rust also accepts `r#match()`. | Reviewing duplicates and similar content. |
| `relate()` | Suggested connections with the shared terms that explain them. | Helping users review possible links. |

These methods compare words and characters. They do not automatically recognize synonyms or infer meaning. A calling application can add a language model or semantic embeddings when it needs those capabilities. AIRust has no required LLM dependency.

A relationship suggestion is evidence of shared words. The application must decide whether that evidence supports a useful connection, hierarchy, prerequisite, or contradiction.

## Try the source checkout

Use Rust 1.89 or newer. From the project directory:

```bash
cargo run --locked --example core
```

The example demonstrates search, text comparison, duplicates, relation suggestions, filters, and JSON round trips.

## Add AIRust to an application

Add AIRust to your `Cargo.toml`:

```toml
[dependencies]
airust = "0.1.9"
```

For local source development, use the checkout path instead:

```toml
[dependencies]
airust = { path = "../airust-master" }
```

A complete example:

```rust
use airust::{Engine, KnowledgeEntry};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut knowledge = Engine::new();
    knowledge.upsert(KnowledgeEntry::new(
        "note-1",
        "Rust provides memory safety without a garbage collector.",
    ))?;
    knowledge.upsert(KnowledgeEntry::new(
        "note-2",
        "Python is useful for scripting.",
    ))?;

    let results = knowledge.search("Rust memory", 5);
    assert_eq!(results[0].entry.id, "note-1");
    println!("{}", results[0].entry.text);

    let matches = knowledge.find_matches(
        "Rust provides memory safety without a garbage collector.",
        5,
    );
    assert_eq!(matches[0].score, 1.0);

    knowledge.upsert(KnowledgeEntry::new("note-2", "Python helps with automation."))?;
    knowledge.remove("note-2");

    let restored = Engine::from_json(&knowledge.to_json()?)?;
    assert_eq!(restored.len(), 1);
    Ok(())
}
```

Your application owns the IDs, storage model, and presentation. AIRust accepts the same entry format from document tools, graph applications, and agent services.

Search results contain the original entry, a relevance score, and matching terms. A query without a match returns an empty list. BM25 scores are relevance values, not probabilities. Equal scores use ascending entry IDs for a stable order.

## Optional integrations

The default build is the library. PDF processing, HTTP, SQLite, bots, command-line programs, and embedded sample knowledge are enabled explicitly.

| Cargo feature | Enables |
| --- | --- |
| None | Knowledge index, BM25, TF-IDF, matching, duplicates, filters, and relation suggestions. |
| `cli` | The existing command-line programs and embedded sample knowledge. |
| `pdf` | PDF text extraction and chunking. |
| `web` | The existing dashboard, HTTP, SQLite, bots, PDF, and CLI. |
| `bots` | A compatibility name for the existing web and bot integration. |
| `embedded-knowledge` | Sample data from `knowledge/train.json` at build time. |

The existing `MatchAgent`, `TfidfAgent`, `ContextAgent`, and `KnowledgeBase` adapters remain available. The historical `TfidfAgent` name uses the shared BM25 index. The actual TF-IDF search mode is `SearchAlgorithm::Tfidf`.

## Run the tools

Run a stored-answer query:

```bash
cargo run --locked --features cli --bin airust -- cli query tfidf "What is AIRust?"
```

Convert a sample PDF into training data:

```bash
cargo run --locked --features cli,pdf --bin pdf2kb -- data/pdf/sample.pdf /tmp/airust-sample.json
```

Start the optional dashboard:

```bash
cargo run --locked --features web --bin airust
```

The default dashboard address is `http://127.0.0.1:7070`. The console uses port 7071. Administration is disabled by default; network-facing operation requires a configured API token. Follow [STARTUP.md](STARTUP.md) and the [security guide](docs/SECURITY.md) for configuration.

## Project layout

| Location | Purpose |
| --- | --- |
| `src/text.rs` | Shared tokenization and Unicode normalization. |
| `src/matching.rs` | Character and token comparisons. |
| `src/search.rs` | Search modes, BM25 settings, and result types. |
| `src/context.rs` | Search filters for IDs and metadata. |
| `src/knowledge.rs` | Generic knowledge entries and the existing training collection. |
| `src/index.rs` | Shared storage and retrieval operations. |
| `src/agent.rs`, `src/*_agent.rs` | Compatibility with the existing agent API. |
| `src/pdf_loader.rs`, `src/bin/`, `src/web/` | Import and user-interface integrations. |
| `examples/` | Runnable examples, a small runtime comparison, and a scalable index benchmark. |
| `tests/` | Cross-module tests and regression cases. |
| `docs/` | API contracts, architecture, migration, and change reports. |
| `knowledge/`, `data/pdf/` | Sample knowledge and a PDF fixture. |
| `airust.db` | Existing local dashboard database, excluded from the crate package. |
| `backups/` | The preserved original archive, excluded from the crate package. |
| `build.rs` | Copies optional sample knowledge into the build directory. |
| `Dockerfile`, `.dockerignore` | Container configuration for the dashboard. |
| `test.sh`, `.github/workflows/ci.yml` | Repeatable validation. |

The [API guide](docs/API.md) describes the public contracts. The [architecture guide](docs/ARCHITECTURE.md) explains how the modules work together. Source code is kept free of comments; explanations live in these documents and executable examples.

## Changes and compatibility

See the [0.1.9 change report](docs/CHANGES-0.1.9.md) and [0.1.9 migration guide](docs/MIGRATION-0.1.9.md) for the English-language and hardening work.

The [roadmap](ROADMAP.md) describes optional semantic providers, evidence-based relations, persistence, and generic structural context as future work. External application integration follows a tested engine release.

The published 0.1.8 baseline is described in the [0.1.8 change report](docs/CHANGES-0.1.8.md). The [original analysis](docs/ANALYSIS-0.1.7.md) records what existed in 0.1.7. [CHANGELOG.md](CHANGELOG.md) lists the versions.

## Validate a change

```bash
./test.sh
cargo doc --locked --no-deps --open
```

The script checks formatting, the default library, optional integrations, documentation examples, and Clippy. Validation results for a particular version belong in its change report.

## License

MIT. See [LICENSE](./LICENSE).
