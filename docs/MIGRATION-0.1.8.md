# Migrate from AIRust 0.1.7 to 0.1.8

Version 0.1.8 makes AIRust a knowledge library first. Several defaults changed deliberately. Check these differences before upgrading an existing application.

## New applications

Use `Engine` and `KnowledgeEntry`. Entries have stable application-owned IDs, text, and optional metadata. A second `upsert()` with the same ID replaces that entry.

Use `search()` for query terms, `find_matches()` for text comparisons, and `relate()` for explained lexical connection proposals. The calling application decides what to store and display.

Rust reserves the word `match`. The equivalent raw-identifier spelling is `r#match()`. `find_matches()` is the more readable alias.

## Existing library users

The existing types, traits, and agent names remain available. Dashboard and PDF support must be enabled explicitly:

```toml
[dependencies]
airust = { version = "0.1.8", features = ["web"] }
```

For the PDF library alone, use `pdf`. The `airust` and `merge_kb` programs require `cli`; `pdf2kb` requires `cli` and `pdf`. The default package builds the library only.

`colors` and `plotting` remain accepted, empty compatibility feature names. The old source did not use their dependencies. `async` still enables Tokio; the new core API is synchronous. `bots` enables the existing full web integration.

## Optional sample knowledge

`KnowledgeBase::from_embedded()` returns an empty collection without `embedded-knowledge`. `cli` and `web` enable sample data automatically. The generic index always starts with the caller's entries.

The build script writes within Cargo's build directory rather than creating knowledge files in the source tree.

## Text and ranking changes

The shared tokenizer preserves numbers. Punctuation creates word boundaries rather than joining words. Unicode forms are normalized. Existing result order can therefore change. Rebuild stored lexical vectors used by the optional bot integration.

The new index uses NFKC, lowercase conversion, and collapsed whitespace for exact duplicates. Accents and punctuation remain significant. The historical `MatchAgent::new_exact()` keeps its stricter whitespace policy.

Fuzzy distances and limits count Unicode scalar values instead of UTF-8 bytes. Similarity uses the longer of the two character sequences as its denominator. Confidence values can differ from 0.1.7.

The historical `TfidfAgent` continues to apply BM25 and example weights. The new `SearchAlgorithm::Tfidf` mode provides actual TF-IDF cosine retrieval. Old agents retain their English fallback response strings; the new core returns an empty result list when nothing matches.

BM25 ranks words. Legacy confidence values are heuristics, not calibrated probabilities or directly equivalent measurements across agent types.

## Training formats

`KnowledgeBase` accepts old string outputs and newer `Text`, `Markdown`, or `Json` variants in the same file. Metadata is retained.

The `input`/`output` training format and the generic `id`/`text` format have different roles. Applications must explicitly choose stable IDs and searchable content. One possible mapping uses a stored record ID as `id`, document content as `text`, and provenance as `metadata`.

## PDF and merging

PDF settings are validated. Overlap must be smaller than maximum chunk length. `min_chunk_size` is a preferred minimum; a final shorter passage is retained. Metadata includes the original page number. `chunk_index` and `total_chunks` are scoped to that page.

`merge_kb` no longer prompts interactively about samples. Use `--include-embedded` to include them. The output is excluded from its inputs. Every source must be valid before the existing destination is replaced.

## Dashboard and Docker

The web server binds to `127.0.0.1` by default. `AIRUST_BIND_ADDR` explicitly selects another address. A second port remains necessary for the console. The blanket CORS allowance was removed; clients on other origins need an appropriate external configuration.

The 0.1.8 work left the existing SQLite file untouched. On the next server startup, an optional metadata column is added to the training table while preserving rows. PDF provenance and imported metadata survive restarts. Exported field names `output_text` and `output_format` are accepted when importing. HTTP agent results can include `metadata`.

Old shipped interface text is refreshed only when it still matches the previous default. Customized translations remain intact.

Docker explicitly builds `web` and uses a stable Rust Bookworm image. The resolved optional dependencies require Rust 1.89 or newer. The lockfile records the tested dependency versions.

The subsequent development version adds further operational changes. Read [MIGRATION-0.1.9.md](MIGRATION-0.1.9.md) when upgrading beyond the published 0.1.8 baseline.

## Responsibilities of the application

DenkMaps stores and renders its own nodes. MCP agents supply their own tools and transport. AIRust supplies content retrieval and comparison results. No external DenkMaps or MCP repository was modified as part of this release.
