# AIRust 0.2.2 implementation plan

Baseline: published and verified 0.2.1 at `14492f5`. This release adds a generic graph only. Context assembly, provenance, persistent derived caches, HNSW, revisions, and the complete quality suite remain later stages.

Reuse `petgraph 0.8.3` with only `std` and `stable_graph`, plus its union-find and iterative topological sort. It is MIT/Apache-2.0-licensed and supports Rust 1.64, below AIRust's existing minimum. The optional `graph` feature must not enable embeddings, HTTP, databases, or models.

`KnowledgeGraph` stores stable application IDs and explicit relationships, not copies of knowledge text. Intern node IDs through shared `Arc<str>` references. Relations have stable IDs, type, endpoints, direction, and optional metadata. New public types have private fields or are non-exhaustive. Leave the existing lexical `RelationKind` and proposals unchanged.

Support typed parent-child, dependency, reference, related, and application-defined relations, directed or undirected. Validate IDs and endpoints before updates. Preserve old edges on validation errors. Removing a node removes all incident edge-ID mappings. Neighbor queries preserve multiple distinct relationships and stable ordering.

Provide iterative BFS and DFS with maximum depth, result-node budget, relation-type filter, direction, optional explicit allowed-ID set, and start-node inclusion. No recursive traversal. Borrow node IDs and relationship evidence in results. Bound stored traversal candidates by the remaining node budget and deduplicate candidate node IDs; sort by target ID and edge ID for reproducibility. An explicitly empty allowed-ID set permits no nodes.

Detect cycles in mixed directed/undirected multigraphs without treating reversal over a single undirected edge as a cycle. First detect cycles in the undirected subgraph using union-find; contract its forest components and check directed edges on the quotient graph. A directed edge within an undirected component closes a cycle. Self-loops and parallel undirected edges count as cycles. Also allow checking selected relation types so unrelated links do not invalidate a dependency DAG.

Tests cover empty and missing nodes, self-loops, directed/undirected/mixed cycles, insertion-order independence, parallel relations, retargeting, metadata, node/edge removal, failed updates, depth/node budgets, allowed-ID boundaries, high-degree stars, and long chains. Compare cycle detection with exhaustive small-graph reference calculations. Benchmark 1k/10k/100k nodes and report build, traversal, cycle, update/delete costs, and memory.

Run the full release checks, both toolchain boundaries, package tests, independent consumer, clean artifact/dry run, candidate CI, and publication verification before starting 0.2.3.

Primary references: [StableGraph](https://docs.rs/petgraph/0.8.3/petgraph/stable_graph/struct.StableGraph.html), [UnionFind](https://docs.rs/petgraph/0.8.3/petgraph/unionfind/struct.UnionFind.html), and the upstream [petgraph source](https://github.com/petgraph/petgraph).
