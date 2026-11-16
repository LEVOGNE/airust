# AIRust 0.2.3 implementation plan

Baseline: published, checksum-verified 0.2.2 at `1675219`. Add an optional context assembler over the existing lexical/semantic/hybrid rankers and generic graph. No model, application integration, source database, provenance schema, cache, ANN, or revision system is introduced in this stage.

The `context-assembly` feature enables the existing `hybrid` and `graph` features without further dependencies. `ContextAssembler` borrows an existing `Engine` or `SemanticIndex` and optional `KnowledgeGraph`. It returns borrowed excerpts, source IDs, original metadata, rank evidence, and graph path evidence. Existing `Context` and `ContextAgent` keep their meanings.

`ContextOptions` selects the ranker, scope, total unique source-node budget, returned UTF-8 text-byte budget, per-item byte budget, content deduplication, and graph traversal settings. Search seeds take priority; neighbors follow in stable breadth-first order. Duplicate content shares an excerpt but retains every included source ID and its metadata. Metadata, IDs, and serialization overhead are outside the explicitly named text-byte budget. No tokenizer-specific token count is claimed.

Intersect the global scope with the selected ranker's scope. A disjoint ID intersection or conflicting metadata returns no context. Extend `Context::intersection` without interpreting an empty intersection as unrestricted access. Extend graph BFS with a predicate that filters nodes before traversal, so missing or excluded source entries cannot become bridges. Limit total graph visits across seed traversals.

Reuse the index's cached normalized content for deduplication. Reuse shared Unicode normalization to locate an original-text query-word anchor when possible, then clip on UTF-8 boundaries and preserve excerpt byte offsets. Semantic-only matches fall back to a prefix when no query word occurs. Never clone the corpus or serialize full entries behind a small excerpt budget.

Tests: multilingual excerpts, tiny/zero budgets, deep query terms, stable ordering, duplicate IDs/content, retained metadata, missing graph IDs, hidden bridges, scope intersections, provider failure behavior, and 100k-record assembly costs. Run the full local suite, feature/toolchain checks, audit, extracted package, independent consumer, dry run, and candidate CI. Publish and verify before 0.2.4 begins.
