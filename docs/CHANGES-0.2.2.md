# AIRust 0.2.2: optional generic graph

The independent `graph` feature adds an ID-based graph with explicit typed relationships. It reuses petgraph 0.8.3's stable storage, union-find, and iterative topological sorting. Only `std` and `stable_graph` are enabled. This dependency supports Rust 1.64; AIRust retains its Rust 1.89 core boundary.

Source knowledge remains in the caller's existing index or database. The graph stores shared node IDs, relation IDs, typed endpoints, direction, and optional relation metadata. It supports replacement, retargeting, incident-edge cleanup, deterministic neighbors, bounded iterative BFS/DFS, filters, and mixed-cycle queries. No model, transport, graph database, or application integration is required.

## Bug hunt and tests

The first long-chain test exposed stack overflow in petgraph's recursive `is_cyclic_directed` path. AIRust now uses iterative `toposort` on the contracted graph. The regression checks 20,000-node chains and the measurement checks 100,000-node graphs. This fix was made before publication.

Mixed cycle detection was compared with an independent simple-cycle reference on all 512 three-node combinations of directed and undirected edges. Additional cases cover self-loops, parallel undirected edges, selected relation types, failed retargeting, metadata, removal, 600 mutation steps, insertion-order independence, hidden-bridge exclusion, node/depth budgets, and 10,000-neighbor stars.

The complete check script passed 187 default tests and 264 all-feature tests, including eleven graph tests and the compiled graph documentation example. The new feature remains independent of embedding, HTTP, and database dependency paths. Formatting, Clippy with warnings denied, rustdoc, isolated builds, runnable examples, and minimum-toolchain checks use the existing release process.

DFS uses first-discovery paths; it does not promise complete shortest-hop coverage at a depth limit. BFS supplies that behavior. A regression records this distinction. Cycle detection reports mathematical cycles, not whether a relationship is valid or true. The application selects which kinds must form a DAG.

## Reproducible measurements

Each release-mode process creates a directed chain with deterministic forward shortcuts. Fifty query pairs request at most one hundred visits within depth 256. The host is macOS ARM64 with 64 GiB memory and Rust 1.98.0.

| Nodes | Relations | Build (ms) | Acyclic check (ms) | BFS p50 (µs) | DFS p50 (µs) | Peak RSS (MiB) |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1,000 | 1,075 | 1.500 | 0.053 | 17.375 | 17.541 | 2.91 |
| 10,000 | 10,768 | 6.604 | 0.186 | 6.417 | 6.625 | 6.28 |
| 100,000 | 107,691 | 73.545 | 1.782 | 7.209 | 6.792 | 38.59 |

The 100,000-node run measured 0.077 ms for one hundred relation replacements and 0.041 ms for removing one hundred nodes with incident edges. Raw samples, p95 values, operation counts, and process resource output are in [measurements/0.2.2](measurements/0.2.2/).

```bash
cargo build --locked --release --features graph --example graph_benchmark
/usr/bin/time -l target/release/examples/graph_benchmark 1000
/usr/bin/time -l target/release/examples/graph_benchmark 10000
/usr/bin/time -l target/release/examples/graph_benchmark 100000
```

These single-process results concern this sparse topology and bounded traversal. The small-run variation shows why they are not scaling guarantees, production tail estimates, or claims about dense or mixed application graphs. No knowledge text or model inference is included in this benchmark.

## Compatibility and boundaries

All new public error/type enums are extensible, and original entry fields and lexical `RelationKind` are preserved. Stored endpoint strings share node allocations. Result visits borrow node IDs and relation evidence. Edge inspection can still scale with degree; result budgets do not cap all work. Internal capacity may remain allocated after removal until `clear()`.

The audit reports zero known vulnerabilities and the same two documented optional maintenance warnings from earlier releases. No new maintenance warning was introduced by the graph dependency.

The package contains 132 files and is approximately 255 KiB compressed. Required graph code, tests, examples, documentation, and measurements are included; private data and external model/runtime files are excluded. All 264 all-feature tests passed from the extracted crate. An independent Rust 1.89 consumer passed the existing lexical/semantic/hybrid APIs and the new graph API, including cycle filtering and incident-edge removal. The clean package, publish dry run, candidate CI, and registry/tag/asset checks precede the next release. Artifacts: [GitHub v0.2.2](https://github.com/LEVOGNE/airust/releases/tag/v0.2.2), [crates.io 0.2.2](https://crates.io/crates/airust/0.2.2).
