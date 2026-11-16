# Migrating to AIRust 0.2.2

The graph feature is additive within Cargo's 0.2 compatibility range. Existing knowledge entries, lexical relation proposals, model fingerprints, source formats, and lexical/semantic/hybrid APIs remain unchanged. Default features remain empty.

```toml
airust = { version = "=0.2.2", default-features = false, features = ["graph"] }
```

`graph` enables petgraph's stable structure and shared-string serialization support. It does not enable embeddings, local models, HTTP, or databases, and supports Rust 1.89. Applications may combine it with other optional features explicitly.

Create nodes with stable application IDs, then add validated `Relation` values. Relations require existing endpoints and have independent stable IDs. Updating a relation validates first. Removing a node removes incident relationships. The graph stores no second copy of the knowledge records; keeping graph IDs and an `Engine` synchronized remains the caller's responsibility.

Explicit `RelationType` values are separate from the existing exhaustive lexical `RelationKind` enum, which is preserved. A stored graph relation is supplied by the application, not inferred as fact by text similarity.

Use `breadth_first` for shortest-hop neighborhoods within a depth limit. `depth_first` records first-discovery paths and may discover a node too deeply to expand it even if a shorter alternative path exists. Both return unique nodes within configured depth/count limits. Explicitly empty allowed-ID sets exclude all nodes. Hidden nodes are not traversal bridges.

`has_cycle()` includes all relation types and follows directed arrows while allowing either direction on undirected edges. Use `has_cycle_for()` to check only hierarchy or dependency types. A single undirected edge is not a cycle; a self-loop or parallel undirected pair is. Cycles are reported rather than automatically prohibited.

No application storage, graph database, automatic relation inference, context assembly, or MCP integration is added. See the [graph guide](GRAPH.md) and [release report](CHANGES-0.2.2.md).
