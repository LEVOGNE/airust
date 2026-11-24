# Upgrade to AIRust 0.5.1

0.5.1 is a patch on the 0.5 line. An application that requires `0.5` can take it with no code change. An application that pins `=0.5.0` selects 0.5.1 explicitly. Your application still owns its data and permissions. This patch adds no DenkMaps integration.

## Update the dependency

```toml
[dependencies]
airust = { version = "=0.5.1", default-features = false }
```

Keep the features the application already uses. MCP stays opt-in through `features = ["mcp"]`. Provider-neutral features require Rust 1.89 or newer. Optional local ONNX adapters require Rust 1.92.

## Calls, scores and saved data

Engine, SearchOptions, Retriever, WorkControl, McpServer and McpLimits stay. Search scores, ordering, matched terms, JSON knowledge, document snapshots and the cache format stay as in 0.5.0. Runtime source is unchanged apart from the version string. Read [what was fuzzed and what was not](CHANGES-0.5.1.md). The move from 0.4 remains [the 0.5.0 upgrade](MIGRATION-0.5.0.md). The 0.5.0 measurements were not repeated.

## MCP clients

Discovery schema IDs include the crate version. Refresh validators that pin the 0.5.0 URN. Tool argument and result shapes are unchanged.

```bash
cargo install airust --version 0.5.1 --locked --no-default-features --features mcp --bin airust-mcp
```
