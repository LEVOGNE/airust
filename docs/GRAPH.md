# Optional generic graph

Enable `graph` for `airust::graph`. It reuses petgraph's stable adjacency structure and algorithms, adds no model requirement, and does not enable embeddings or any transport. The graph stores application-owned IDs and explicit relationships rather than knowledge text.

`KnowledgeGraph` supports `add_node`, `contains_node`, sorted `nodes` and `relations`, `get_relation`, `upsert_relation`, `remove_relation`, `remove_node`, `clear`, counts, neighbor queries, bounded traversal, and cycle detection. Duplicate node insertion returns false. Missing endpoints fail before a relation update. Node removal removes all incident relations. Public IDs remain stable even when internal slots are reused.

```rust
use airust::graph::{KnowledgeGraph, Relation, RelationType, TraversalOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut graph = KnowledgeGraph::new();
    graph.add_node("topic")?;
    graph.add_node("detail")?;
    graph.upsert_relation(Relation::new(
        "topic-detail", "topic", "detail", RelationType::ParentChild,
    )?)?;
    let visits = graph.breadth_first("topic", &TraversalOptions::new(1, 10))?;
    assert_eq!(visits.len(), 2);
    assert_eq!(visits[1].parent, Some("topic"));
    assert!(!graph.has_cycle());
    Ok(())
}
```

Construct a `Relation` with a stable relation ID, source ID, target ID, and `RelationType`. Available categories are `ParentChild`, `DependsOn`, `References`, `Related`, and `Custom(String)`. Custom labels must be nonempty. Relations are directed by default; `with_direction(RelationDirection::Undirected)` makes a relation traversable both ways. `with_metadata` retains optional application metadata. Getters expose immutable values. Node IDs and stored endpoints share string allocations internally.

These relationships are supplied by the application. AIRust does not infer that a dependency, contradiction, hierarchy, or other assertion is true. Existing lexical relation proposals remain separate and unchanged.

## Neighbors and traversal

`neighbors(id, direction)` returns borrowed neighbor IDs and relationship evidence, sorted by target ID then relation ID. Multiple distinct relationships are retained. A self-loop appears once. `Outgoing`, `Incoming`, and `Both` control directed relations; undirected relations work in either direction.

`breadth_first` and `depth_first` are iterative. Configure `TraversalOptions::new(max_depth, max_nodes)`, direction, relation types, an optional allowed-ID set, and start-node inclusion. Defaults are depth two, at most one hundred returned nodes, outgoing traversal, and inclusion of the start.

Depth counts edges from the start, whose depth is zero. A zero node budget returns no visits; zero depth permits only the start if included. Excluding the start leaves the full result budget for neighbors. Unknown start IDs are errors, including at zero budget. An explicitly empty allowed-ID set excludes every node. Disallowed or filtered nodes are not used as hidden bridges.

Visits borrow IDs, parent IDs, and the selected relation. Each node appears once. Parallel eligible relations choose the lowest relation ID as traversal evidence. Candidate node IDs are sorted for deterministic traversal independent of insertion order. DFS records each node at its first-discovery depth; use BFS for complete shortest-hop neighborhoods within a depth limit. Stored candidates are capped by the remaining result budget; inspecting a high-degree node still scans its incident relationships. The budget limits returned nodes and each retained candidate set, not total edge-inspection time. BFS keeps a bounded queue; DFS retains candidate frames along its active path.

## Cycles

`has_cycle()` checks all stored relation types. `has_cycle_for(types)` checks only selected types; an empty list has no cycles. A dependency application can exclude unrelated links from its DAG check.

Mixed directed/undirected cycles follow directed arrows and may traverse undirected edges either way. Reversing over the same single undirected edge is not a cycle. Self-loops and parallel undirected edges are cycles. The implementation detects undirected cycles with union-find, contracts the resulting forest components, then applies petgraph's iterative topological sort to the quotient. A directed edge inside an undirected component closes a mixed cycle.

Cycle work is linear apart from union-find's inverse-Ackermann factor and optional type filtering. Temporary unit graphs contain no copied source records. Stable storage can retain internal capacity after removals; `clear()` releases this graph's allocations. Other owners of cloned relation values may retain their shared IDs and metadata.

The caller owns storage and permissions. The allowed-ID filter is a traversal control, not a user-account system. This release adds no graph database, automatic relationship inference, application adapter, context assembly, or UI.
