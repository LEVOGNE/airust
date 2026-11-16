# Migrating to AIRust 0.2.4

The independent `provenance` feature is additive within the 0.2 compatibility range. It supports Rust 1.89, adds no dependency and does not enable a graph, model, HTTP client or database. Default features remain empty. Existing knowledge entries, source JSON, retrieval algorithms and graph behavior are preserved.

```toml
airust = { version = "=0.2.4", default-features = false, features = ["provenance"] }
```

Keep source definitions and entry bindings in `ProvenanceStore`. Register a `SourceReference` once with `insert_source`, then `bind` multiple entry IDs to it. `add` is a convenience for both operations. Conflicting definitions under the same stable reference ID are errors; use a new ID for a different artifact revision or location. Replacements and store merges validate conflicts before mutation.

Page numbers start at one. Text spans use zero-based UTF-8 bytes and exclusive ends. Code positions use one-based lines and one-based UTF-8 **byte** columns with exclusive span ends. Constructors check structure; `validate_source_text` checks exact supplied text. The engine does not open paths, fetch URIs, infer page/offset mappings or verify source claims. Missing values stay missing.

Enable `context-assembly` separately to use `ContextAssembler::with_provenance`. Each selected `ContextReference` then adds `sources` and `source_status` to serialized results and exposes getters. Consumers using strict JSON schemas must accept these additive fields when enabling the feature. `Missing`, `Referenced`, and intentionally `Omitted` are distinguished. Entry metadata controls remain separate from source metadata.

Context excerpt offsets and external source offsets can use different coordinate systems. Source metadata and citations are outside the context text-byte budget. The application measures the complete prompt and updates or invalidates bindings when its entry text changes. No application storage migration, source-file access, automatic fact verification, or integration is performed. See [PROVENANCE.md](PROVENANCE.md).
