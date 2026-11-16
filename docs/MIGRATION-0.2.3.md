# Migrating to AIRust 0.2.3

The optional context assembler is additive within the 0.2 compatibility range. Default features remain empty, `KnowledgeEntry` keeps its original fields, and existing lexical, semantic, hybrid, graph, `Context`, and `ContextAgent` behavior remains available.

```toml
airust = { version = "=0.2.3", default-features = false, features = ["context-assembly"] }
```

This feature enables the existing `hybrid` and `graph` features, supports Rust 1.89, and adds no dependency. Use `ContextAssembler::new` for lexical retrieval or `from_semantic` for optional provider-backed modes. Attach a generic graph explicitly. The assembler borrows existing data and returns text excerpts, distinct source IDs, metadata, ranking/path evidence, and examined-candidate counters.

`ContextOptions` separates source-node limits, returned text-byte limits, per-item byte limits, ranker configuration, scope, and traversal budgets. Distinct normalized duplicates retain their source IDs and consume the source-node budget. The text budget excludes metadata and serialized overhead; it is not a tokenizer-specific token count. Excerpt offsets apply to the first source reference.

The additive `Context::intersection` returns `None` for a denied intersection. Do not replace `None` with `Context::default()`, which would remove that restriction. The graph adds `breadth_first_matching` and `TraversalOptions::with_node_limit`. The predicate excludes disallowed or missing nodes before queueing, preventing hidden bridges without collecting all corpus IDs.

No application database synchronization, mandatory LLM, prompt schema, or external integration is introduced. See [CONTEXT.md](CONTEXT.md) for complete ordering, scope, clipping, fallback, and lifetime contracts.
