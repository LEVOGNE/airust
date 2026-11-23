# Upgrade to AIRust 0.5.0

AIRust remains a Rust knowledge engine with optional MCP in the same crate. Your application owns its data, permissions and durable storage. This release sequence adds no DenkMaps integration or required model, database or server.

## Update the dependency

Cargo treats 0.5 as a new compatibility range. An application using 0.4 must choose the update explicitly. For the core library:

```toml
[dependencies]
airust = { version = "=0.5.0", default-features = false }
```

Keep the features your application already uses. MCP remains opt-in through `features = ["mcp"]`. Provider-neutral features require Rust 1.89 or newer; optional local ONNX adapters require Rust 1.92. Both minimums pass the release matrix.

## Existing calls and knowledge

The released sequence retains Engine, SearchOptions, Retriever, WorkControl, McpServer and McpLimits. The strict patch-level comparison against 0.4.25 reports 229 pass, 31 skip; it finds no required application-call changes. Core/MCP runtime source is unchanged from 0.4.25.

Generic knowledge still uses IDs, original text and optional metadata. Keep your JSON and document snapshots. Loading a knowledge file rebuilds its search index. The temporary input is released after parsing owned records; the original file stays unchanged.

Document passages remain tied to an exact revision and checked UTF-8 range in the saved text. These positions are not coordinates in an original PDF. Old generic entries do not automatically acquire revision bindings.

## Retained changes in this sequence

Each change has its own published report:

- 0.4.22 removes duplicate MCP payload checks and an intermediate JSON copy. The complete encoded reply remains bounded.
- 0.4.23 avoids temporary frequency storage for short repetitions of one already normalized word. Different, long or noncanonical queries keep existing analysis. This is an internal optimization, not a new query language.
- 0.4.24 keeps final reply preparation inside the admitted blocking worker, checks cancellation/deadlines after that work, and reports final output-budget failures as errors in progress notifications.
- 0.4.25 releases owned raw JSON before index construction. Instrumented peak live heap decreases; native process RSS and retained index memory remain separate observations.

No optimization changes ranking into semantic understanding. The bundled MCP tools expose status, search, matching, suggested relations and registered passage readback. Optional advanced Rust search features are not automatically exposed through these tools.

## MCP clients

Install the optional executable from the same crate and load an explicit knowledge file:

```bash
cargo install airust --version 0.5.0 --locked --no-default-features --features mcp --bin airust-mcp
airust-mcp --knowledge /absolute/path/knowledge.json
```

Clients should discover the tools and schemas from the running server. Schema identifiers include the AIRust version; refresh cached validators when updating. Full schema/discovery/response comparisons retain compatible shapes and arguments except asserted status version and version-qualified schema URNs.

Progress notifications require a client-supplied progress token. The client chooses how to display them. AIRust cannot guarantee that an arbitrary agent explains its actions, shows a loading bar or predicts a completion time. Server instructions ask agents to explain longer work and its outcome.

Time limits are cooperative. Startup loading remains outside individual tool deadlines. Admitted work includes complete tool-result preparation and encoded-size checks. Before/after checkpoints reject cancelled/expired successful output. SDK JSON-RPC framing and final writes remain separate. Notification waits retain their original absolute-deadline/stall rules; these are not OS CPU/RAM quotas. Cancellation does not forcibly terminate arbitrary code between checkpoints.

The server is read-only for its loaded collection. Register only documents the client may read. A registered snapshot can be read by valid passage ranges even when only some of its passages were indexed. Entry selection and search context are not an application permission system.

## Check the update with your own data

Run your normal searches, update/remove/reinsert cases, source readbacks and MCP cancellation/error paths. Keep an untouched data copy for comparison. Compare scores, ordering, source revisions and byte ranges, then measure latency and memory separately.

The published local benchmarks retain slower outcomes and distinguish direct Core calls from MCP round trips. Fixed exposed questions and one benchmark machine do not establish universal speed, new semantic quality or a completed security audit. Read [executed scope and limits](CHANGES-0.5.0.md), [API cookbook](API-COOKBOOK.md), [MCP](MCP.md) and [stability contract](STABILITY.md).
