# AIRust 0.1.7 technical inventory

Recorded on November 16, 2026. This is a historical report about the supplied source directory before the knowledge-core extraction.

## Preservation and baseline checks

The directory contained one Rust package, `airust` 0.1.7, without a `.git` history. A complete archive was created before changes. The original tree was then preserved locally as commit `fb7badb` on `baseline/airust-v0.1.7`.

The archive was `/private/tmp/airust-v0.1.7-original-20261116-015240.tar.gz`. Its SHA-256 checksum is `56355ca8cd6d92363da7808829f6aa4b3466598711378b3eb41cd46666b45d4d`. A second copy was placed in the project's excluded `backups/` directory.

The original GitHub history was fetched later during release preparation. The supplied tree matched GitHub commit `052510449fe99a047e69e2dd6781a369498eedd1`. Version 0.1.8 was attached to that history before publication. This later step did not erase the local preservation branch.

The original checks ran with Rust and Cargo 1.98.0 on macOS:

| Check | Result before changes |
| --- | --- |
| `cargo metadata --no-deps` | One package, three programs, one integration-test target. |
| `cargo test --workspace --locked --offline` | 158 unit tests and ten integration tests passed. |
| `cargo test --workspace --no-default-features --locked --offline` | The same 168 tests passed. |
| `cargo clippy --workspace --all-targets --locked --offline` | Completed with six warnings. |

A passing test demonstrates only the behavior it checks. These tests did not establish general answer quality, semantic understanding, or comprehensive web-server coverage.

## Algorithms present in 0.1.7

| Area | Implemented behavior | Interpretation |
| --- | --- | --- |
| `agent::text_utils` | Lowercasing, tokenization, stopwords, n-grams, Levenshtein, Jaccard. | Classical text processing. |
| `match_agent` | Exact text and edit-distance comparison. | Matching spelling and tolerating typos. |
| `tfidf_agent` | BM25 with example weights. | Lexical retrieval, despite the historical TF-IDF name. |
| `context_agent` | Bounded conversation history appended to a query. | Text memory rather than understanding follow-up questions. |
| `knowledge` | JSON persistence, merging, embedded training examples. | A question-answer collection without stable entry IDs. |
| `pdf_loader` | PDF extraction, cleanup, and chunking. | Import tooling rather than a learning model. |
| `web::vectordb` | TF-IDF vectors and cosine similarity. | Lexical vectors rather than neural embeddings. |
| `web::bots::vectordb` | A separate TF-IDF path and database retrieval. | Another tokenizer and retrieval implementation. |
| `web::bots::processor` | Question templates, paragraph and sentence processing, content hashes. | Rule-based preparation and duplicate checking. |
| `web::bots::crawler` | HTTP requests and HTML extraction. | Content collection. |

There was no LLM engine, neural training, or demonstrated synonym recognition. Product statements claiming that BM25 understood meaning and synonyms exceeded the actual implementation.

## Dependency structure

```text
agent: shared types, traits, and text utilities
  ├── match_agent
  ├── tfidf_agent
  ├── context_agent: wraps another agent
  └── knowledge: stores TrainingExample
        └── pdf_loader: creates training examples

CLI and web use these agents and the training collection.
Web adds SQLite, HTTP, a console, bots, and two further retrieval paths.
```

Default features enabled `web` and `colors`. Web also enabled bots, Tokio, SQLite, and HTTP dependencies. `pdf_oxide`, `chrono`, `regex`, `once_cell`, and `indexmap` were direct dependencies even without default features. The source did not use the declared `colored`, `plotly`, or `plotters` libraries.

## Concrete findings

1. **Repeated BM25 work.** Every document score recalculated the average length of the entire corpus. A query also scored every document. The repeated length calculation added quadratic work.
2. **Full rebuilds on append.** Adding examples rebuilt the complete search index. Targeted replacement and deletion by stable ID were absent.
3. **Inconsistent tokenization.** The central tokenizer removed numbers and punctuation, turning `Rust-Python` into `rustpython`. Bot retrieval tokenized the same content differently.
4. **Mixed distance units.** Fuzzy thresholds used byte lengths while Levenshtein counted characters. Non-ASCII input received inconsistent limits.
5. **Fallback strings mixed with results.** A missing result was returned as an English answer sentence. Default confidence inspected that sentence, while detailed confidence could disagree with normal prediction.
6. **Lost provenance.** PDF examples contained metadata that `pdf_to_knowledge_base()` discarded. Old agent results also omitted metadata.
7. **Build-time source mutation.** The original build script created `knowledge/train.json` when absent. Library consumers should not receive new source-tree knowledge files during compilation.
8. **Unchecked PDF settings.** Excessive overlap could underflow a subtraction. Long segments could exceed the configured maximum chunk length.
9. **Repeated merge output.** `merge_kb` read its own destination as another source and could include embedded data again.
10. **Stale tooling and claims.** The shell test script used outdated CLI commands. Web version numbers were repeated literals. A license file was absent despite the MIT manifest declaration.

The web interface exposed file and database administration. Its console included database-reset commands. Contrary to the old README, it did not run arbitrary shell commands. Keeping that interface as an optional adapter did not establish its suitability for public deployment.

## Extractable foundations

BM25, Levenshtein distance, and Jaccard similarity were useful starting points. Text preparation and ranking could be shared without requiring HTTP, SQLite, PDF, or a language model.

Existing agent names and training formats could remain compatibility adapters. Cargo features could isolate tools from the default library. Existing data files could be preserved.

## Missing application-facing capabilities

- Generic entries with stable IDs, text, and optional metadata.
- Results tied to source entries with scores and matching terms.
- Targeted insertion, replacement, and removal.
- A shared Unicode tokenizer preserving numbers and word boundaries.
- Duplicate groups and consistent comparison evidence.
- Scope filters that leave the query unchanged.
- Explained connection proposals that do not treat word overlap as a logical fact.
- A small integration example and regression tests for index mutations.

DenkMaps was an example of a possible calling application in the original request. Its repository and data model were not part of this source directory. No external application or MCP transport was integrated during this work.

## Decision following the analysis

Version 0.1.8 would provide a modular library through `text`, `matching`, `search`, `context`, `knowledge`, and `index`. A shared index would support search, text matching, and basic explained relation proposals.

Search and matching had priority. Relation proposals would reuse the same term data and infer lexical relatedness only. Logical relation types would remain the calling application's responsibility.

Implementation results are recorded in [CHANGES-0.1.8.md](CHANGES-0.1.8.md). Upgrade details are in [MIGRATION-0.1.8.md](MIGRATION-0.1.8.md).

Technical references: [Cargo features](https://doc.rust-lang.org/cargo/reference/features.html) and [Unicode normalization](https://docs.rs/unicode-normalization/latest/unicode_normalization/trait.UnicodeNormalization.html). Findings about AIRust came from the local source and the baseline checks listed above.
