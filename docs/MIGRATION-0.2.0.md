# Migrating to AIRust 0.2.0

The existing lexical API, entry format, JSON source export, and empty default feature set remain compatible. The internal entry validation helper was shared with the new semantic index; BM25, TF-IDF, matching, duplicates, and lexical relation behavior are preserved and regression-tested.

Cargo considers 0.1.x and 0.2.x separate compatibility ranges. Select the new version explicitly:

```toml
airust = { version = "=0.2.0", default-features = false }
```

Add `features = ["embeddings"]` only when you need the provider-neutral semantic API. This adds no dependencies and still supports Rust 1.89. Your existing `Engine` does not automatically call a model or change its search algorithm.

To reuse an existing engine, consume it with `SemanticIndex::from_index(engine, provider, batch_size)`. This generates document vectors and retains one source-entry store. The constructor consumes its input even if inference fails; keep durable application records as the recovery source. `into_lexical()` recovers the owned lexical index without cloning when changing model spaces.

Use the semantic index's update and remove methods to maintain both lexical and vector state. Its `lexical()` accessor is read-only. Do not mix vector spaces from different model revisions or preprocessing recipes, even when dimensions agree.

The optional `local-embeddings` feature requires Rust 1.92 because the selected ONNX Rust bindings require it. It needs application-supplied local E5 model files and a compatible platform shared library. Normal Cargo compilation and all-feature tests do not download these assets. Existing web/PDF features remain optional and unchanged.

Scores have different meanings: BM25 is a lexical relevance value, character matching measures spelling, and semantic cosine similarity ranges from -1 to 1. None is a probability of truth or a logical relationship. The default semantic threshold is -1; choose a suitable threshold from measurements on your own corpus.

Source JSON exports still omit vectors. This release adds no persistent vector format, hybrid ranking, graph model, or application integration. See the [embedding API](SEMANTIC.md), [local provider guide](LOCAL-EMBEDDINGS.md), and [release report](CHANGES-0.2.0.md).
