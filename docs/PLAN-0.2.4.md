# AIRust 0.2.4 implementation plan

Baseline: published, artifact-verified 0.2.3 at `c40ed10`. Add optional structured source references without changing `KnowledgeEntry`, existing source JSON or any retrieval algorithm. No database integration, cache, ANN or revision journal is part of this release.

The independent `provenance` feature adds no dependency. `SourceReference` records a stable reference ID and optional document ID, page, UTF-8 source byte span, file path/code span, revision, URI and metadata. Source strings are opaque caller-supplied evidence; the engine performs no downloads or filesystem reads and does not infer truth or verify a source's claims. Missing positions remain missing.

Validate nonempty identities, positive page numbers, ordered ranges, one-based code positions and optional checks against exact supplied source text. Deserialization must go through the same validation. Keep public fields private and errors extensible. Text/code offsets have explicit units and boundary semantics; do not infer a mapping from a knowledge excerpt to an external source span.

`ProvenanceStore` interns references globally by stable reference ID and maps entry IDs to sorted reference-ID sets. This avoids repeating the same source metadata for each entry. It supports additive references, replacement, removal, pruning, merging entry bindings, merging stores, stable iteration and versioned JSON. Conflicting definitions of a reference ID are errors before mutation. Missing references remain explicit; removal and import preserve referential integrity.

When `context-assembly` and `provenance` are both enabled, `ContextAssembler::with_provenance` attaches borrowed references to each selected source ID, including all normalized duplicates. Keep entry excerpt offsets separate from external citation spans. Allow callers to omit source references. A plain provenance build must not enable graph, model, HTTP or database dependencies.

Tests cover multiple/missing sources, malformed IDs/pages/ranges/UTF-8 positions, serialization round trips and invalid deserialization, dangling IDs, duplicate import IDs, conflicting merges without partial mutation, source retention on entry merge/removal, context duplicate references, optional scopes and omitted sources. Measure shared-reference storage and export/import on 1k/10k/100k bindings. Finish full local/packaged/toolchain/consumer/CI checks, publish and verify before starting 0.2.5.
