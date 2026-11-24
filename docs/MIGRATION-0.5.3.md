# Upgrade to AIRust 0.5.3

Version 0.5.3 preserves the 0.5.2 public library calls, empty default feature set and readable saved formats. Existing generic Engine records, retrieval responses, document snapshots and source coordinates do not require a data migration. Existing applications keep their explicit feature choices.

```toml
[dependencies]
airust = { version = "=0.5.3", default-features = false }
```

For a dependency already allowing the 0.5 line, update its lockfile explicitly:

```bash
cargo update -p airust --precise 0.5.3
```

Provider-neutral modules still require Rust 1.89. Optional local ONNX embedding/reranking adapters still require Rust 1.92 and caller-supplied trusted model/runtime files. Builds and ordinary tests do not download them. The PDF backend remains exactly 0.3.5.

## MCP clients

Update the executable or dependency and refresh cached output-schema identifiers. They are qualified by the crate version and now contain `0.5.3`. The five read-only tool names, wire shapes, configured limits and collection ownership remain compatible.

```bash
cargo install airust --version 0.5.3 --locked --no-default-features --features mcp --bin airust-mcp
```

## New source-checkout workflow

The `reference_workspace` example adds directory import, its own checksummed workspace, exact citations and stdio serving. Use a new destination directory for each import; do not pass this example's workspace JSON as the standalone executable's generic knowledge format. Existing workspaces intentionally retain their original text revision when source files later change. [Run the complete workflow](REFERENCE-APPLICATION.md).

The new evaluation is an explicit development command. Its model recommendation does not change library or MCP defaults, rebuild user data, start a model service or enable telemetry. The frozen 150 questions are assistant-authored references, not independently collected customer traffic. [Release changes](CHANGES-0.5.3.md).
