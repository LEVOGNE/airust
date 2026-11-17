# Contribute to AIRust

Use Rust 1.89 or newer for the core and provider-neutral embeddings/reranking. The optional local ONNX adapters and the complete all-feature check script need Rust 1.92 or newer. Python 3.11 or newer is needed for the explicit model-download helper tests, not for the library. Run the repository checks before submitting a change:

```bash
./test.sh
```

Keep the default build independent of PDF, HTTP, databases, and language models. Lexical retrieval features should use the shared index and tokenizer. Embedding providers receive original text and explicitly define their model's preprocessing recipe.

A relevance score is not a probability. Shared words do not prove a logical relationship. Name results accurately and describe their limitations.

For index changes, verify insertion, replacement, and removal. For text processing, cover Unicode and word boundaries. Preserve supported training formats and metadata.

Keep source identifiers, errors, interface text, and project documentation in English. Language samples used to test Unicode behavior may remain in their original script. Source files contain no comments. Put public API contracts and design explanations in `docs/`, and demonstrate behavior with executable examples and meaningful tests.

Update the README, startup guide, API guide, migration notes, and changelog when public behavior changes. DenkMaps storage and rendering belong in DenkMaps. MCP tools and transport belong in the calling agent.

A published crate version is immutable. Prepare subsequent changes under a new version. Before publication, run the checks, create a clean release commit, and verify packaging:

```bash
cargo package --locked
cargo publish --locked --dry-run
```

Inspect the package contents. Local databases, credentials, logs, and backups must stay out of the crate. Publishing with `cargo publish` requires explicit release authorization. Check that the repository tag, packaged commit, registry checksum, and release asset identify the same source.

For reranking changes, retain frozen judgments, compare actual first-stage prefixes without injecting relevant documents, record individual regressions and added cost, and test malformed outputs/recipe changes/scopes/fallbacks. Native inference remains an explicit asset-backed check; normal builds/tests must not download models or runtimes.
