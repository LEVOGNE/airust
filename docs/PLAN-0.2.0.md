# AIRust 0.2.0 implementation plan

Baseline: 0.1.9, with the documentation follow-up at `c4bd8d5`. This release adds optional embeddings. Hybrid fusion, graphs, context assembly, provenance, persistent derived indexes, HNSW, and revisions belong to later releases.

## Inventory and reuse

`KnowledgeIndex` already owns validated generic entries, metadata, lexical postings, and duplicate groups. `ranking::TopK` already provides bounded deterministic result ordering. `Context` already implements ID and metadata filtering. No embedding provider, vector validation, cosine ranking, or local inference adapter exists.

Reuse those components. Store each source entry once. Keep semantic vectors in a side table owned by a semantic index; expose an immutable view of its lexical index so callers cannot silently invalidate vectors. Do not alter the existing BM25, TF-IDF, matching, or relation APIs.

## Modules and dependencies

- `src/semantic.rs`: provider contract, vector space identity, validated embeddings, robust cosine similarity, exact semantic retrieval, and atomic batch updates.
- `src/semantic/local.rs`: optional FastEmbed adapter loading application-supplied local model files. Use dynamic ONNX Runtime loading. Disable upstream model-hub and binary-download features.
- `tests/semantic.rs`: public API, invalid inputs, provider failures, model changes, batching, deterministic ordering, filters, updates, and lexical regressions.
- `examples/semantic.rs`: a small deterministic contract example that downloads nothing.
- A separate real-model evaluation example and labeled public multilingual fixtures.
- English API, feature, migration, research, and release documentation.

The `embeddings` Cargo feature adds no dependencies. `local-embeddings` enables FastEmbed and the already optional SHA-256 dependency. No default features are enabled. No model or native runtime is downloaded while compiling any AIRust feature combination.

## Public API contracts

An embedding space identifies the complete model recipe and vector dimension. Vectors must be nonempty, finite, nonzero, and dimensionally correct. Embedding construction normalizes values using a double-precision norm. Model-space mismatches are errors even when dimensions agree.

Providers receive borrowed text batches and an explicit query/document purpose. The caller owns model access, credentials, deadlines, and cancellation. Provider errors and malformed batch output are propagated. Text is not normalized by AIRust before embedding; models may need case, punctuation, and language details.

A semantic index owns the existing lexical index and one vector per entry. It supports provider-backed or precomputed insertion, removal, read-only lexical access, and exact cosine retrieval. Batch changes are validated and staged before any index mutation. Search results borrow original entries; equal scores use ascending IDs. Metadata scope is applied before retaining top results.

The local E5 adapter reads explicit local files, fingerprints the model and tokenizer recipe, and applies `query: ` / `passage: ` prefixes. It performs no automatic download. The external ONNX runtime requirement and model license must be documented and tested separately.

## Acceptance

Preserve all existing tests. Check zero, NaN, infinity, dimension mismatches, extreme finite values, empty text, batch sizes, provider output counts, provider errors and identity changes, stale postings, duplicate IDs, replacements, deletions, filters, ties, and batch atomicity. Compare exact retrieval against a full cosine reference.

Evaluate a pinned real multilingual model on an identified set of German and multilingual reference cases. Report measured quality and latency without treating hand-written fixture vectors as semantic evidence. Record model revision, files, license, hashes, runtime, build profile, raw outputs, and limitations.

The fixed 12-document, 16-query semantic smoke corpus must reach at least 0.75 top-one accuracy and 0.875 recall at three. The German vehicle-registration paraphrase must appear in the first three results. These release gates concern this small corpus only; they do not establish general semantic accuracy.

Run formatting, all-target checks, Clippy with warnings denied, default and all-feature tests, rustdoc, minimum-Rust checks, dependency audit, packaging, extracted-package tests, examples, relevant benchmarks, and a publish dry run. Review the public API and boundary cases before publishing. Verify the registry artifact and tag before starting 0.2.1.
