# Optional MCP in AIRust 0.4.20

AIRust can expose a fixed, read-only knowledge collection to an MCP client. Search, compare entries, suggest connections and retrieve an exact passage from an identified document revision. The engine, public Rust API and server executable are all in the **same `airust` crate**. No DenkMaps adapter is included. Version 0.4.2 introduced response schemas, and 0.4.4 reduced temporary allocations. Version 0.4.5 adds opt-in Rust evidence-selection/citation APIs; the bundled MCP request/response defaults remain unchanged. Applications holding a RetrievalResponse can use the [citation checker](CITATIONS.md); this transport does not silently inspect an external model's later answer.

Enable `mcp` explicitly. It enables `documents`, `provenance`, `retrieval`, `execution`, Tokio and the official Rust MCP SDK, pinned to `rmcp = 3.11.0`. It does not enable PDF parsing, HTTP, SQLite, embeddings, native models, training or the existing web application. The default library remains feature-free. Rust 1.89 supports this feature.

## Install and start

```bash
cargo install airust --version 0.4.20 --locked --no-default-features --features mcp --bin airust-mcp
airust-mcp --knowledge /absolute/path/knowledge.json
```

`--knowledge` accepts the JSON array produced by `Engine::to_json` or `pdf2kb`. Its entries have `id`, `text` and optional `metadata`. AIRust validates the entire collection before serving it. The server does not parse PDFs, import files through a tool or change the input. Old knowledge works, but old entries do not magically acquire document revisions or global text positions.

Alternatively, load exact snapshots created by `DocumentSnapshot::to_json`:

```bash
airust-mcp --document /absolute/path/manual-snapshot.json --document /absolute/path/second-snapshot.json
```

This indexes each nonblank supplied page as one entry, or one complete entry for a snapshot without a page map. Blank pages remain in the snapshot. A page or nonpaged text exceeding the entry limit is rejected, not silently cut. For custom chunking, construct passage entries and call `McpServer::new(entries, snapshots, limits)` in Rust. `new` registers the snapshots but indexes only the supplied entries. This is an indexing choice, not an access-control boundary: `airust_read_passage` can read any valid range in every registered snapshot. Register only snapshots the client may read. See [the document API](DOCUMENTS.md).

Choose `--knowledge` once **or** one or more `--document` options. There is no default filesystem scan. Relative paths resolve from the process directory; absolute paths make client configuration clearer. Files are opened only at startup. Replacing a file later does not mutate the running collection: restart with the intended data.

A client configuration using the common `mcpServers` convention looks like this; the client determines its exact configuration format and filename:

```json
{
  "mcpServers": {
    "airust": {
      "command": "/absolute/path/to/airust-mcp",
      "args": ["--knowledge", "/absolute/path/knowledge.json"]
    }
  }
}
```

Use `--document` instead when exact snapshot citations are required. The executable communicates over standard input/output. It opens no listening port. Startup counts and failures go to standard error; failures escape control characters and are capped at 2048 characters; server-mode standard output contains JSON-RPC only. `--help` and `--version` print ordinary CLI output and exit. Stop the server by closing stdin, stopping the client or sending Ctrl-C.

## Public Rust API

```rust,no_run
use airust::{mcp::{McpLimits, McpServer}, KnowledgeEntry};

async fn serve() -> Result<(), Box<dyn std::error::Error>> {
let limits = McpLimits {
    max_concurrent: 2,
    default_timeout_ms: 5_000,
    ..Default::default()
};
let server = McpServer::from_entries(
    [KnowledgeEntry::new("rust", "Rust ownership manages memory.")],
    limits,
)?;
assert_eq!(server.stats().entries, 1);
server.serve_stdio().await?;
Ok(())
}
```

| API | Meaning |
| --- | --- |
| `McpLimits::default()` / `validate()` | Select and check explicit resource limits. |
| `McpServer::from_entries(entries, limits)` | Validate and index generic knowledge. |
| `McpServer::from_documents(snapshots, limits)` | Index nonblank pages or a nonpaged snapshot. |
| `McpServer::new(entries, snapshots, limits)` | Bind application-selected passage entries to registered snapshots. |
| `load_knowledge(path, limits)` | Read one bounded engine JSON file at startup. |
| `load_documents(paths, limits)` | Restore bounded snapshot JSON files at startup. |
| `limits()` / `stats()` | Inspect limits, collection counts and occupied worker slots. |
| `tool_definitions()` | Inspect tool schemas and read-only annotations. |
| `serve_stdio().await` | Serve a single stdio connection until it closes. |
| `serve(reader, writer).await` | Serve bounded newline JSON-RPC over application-supplied asynchronous I/O. |

Call this async function from your application's Tokio runtime. A consuming Rust package must declare Tokio directly if it uses Tokio APIs/macros; for example add `tokio = { version = "1", features = ["macros", "rt-multi-thread"] }` beside the `airust` dependency. The packaged executable already owns its runtime.

The immutable server is cloneable. Clones share the index, snapshots and worker limit. Each `serve` gets its own request IDs, progress tokens, connection budgets and worker-lifetime tracker. Closing one connection waits only for that connection's workers; it neither reserves global slots nor waits for work on another clone. The global admission limit still applies across every clone. A Rust caller must establish its own trust boundary before supplying any other I/O; `serve` does not add network authentication, TLS or tenant isolation. The SDK handler is private so these public entry points apply AIRust's transport guards.

Worker admission and connection shutdown use one synchronized state transition. After admission closes, no new reservation can be created. An owned lease holds both the global permit and the connection's tracking token, including while the initial progress message is being sent. It moves into the blocking job and remains there after handler cancellation. An early return, abandoned reservation or panic releases both resources. Shutdown therefore cannot miss work between permit acquisition and spawning. The global permit is released before the tracker reports that lease as finished.

`stats().in_flight` counts occupied permits, including a worker still winding down after cancellation. There is no unbounded waiting queue. Loading/indexing is synchronous startup work, outside request timeouts and progress. Set data limits before constructing the server. `cargo run --features mcp --example mcp` prints its local schemas and counts without starting a connection.

The five tool definitions are built once from that server's fixed limits and stored with its immutable shared data. Calls select the relevant definition without rebuilding all five schemas. `tool_definitions()` still returns an owned vector; its schema handles may share immutable data. Editing a returned tool or using copy-on-write to edit its schema does not change the server or another instance's limits. This retains a small fixed catalogue per data instance, shared by clones. Construct a new server when changing its configuration.

## Tools and arguments

All tools have JSON object arguments with `additionalProperties: false`. Unknown fields are rejected. Limits below are defaults. The schema advertises string-length ceilings; the server additionally checks **UTF-8 bytes**, which can be stricter for non-ASCII text. Optional fields may be omitted.

| Tool | Required arguments | Optional arguments and defaults |
| --- | --- | --- |
| `airust_search` | `query`: nonblank text, at most 4096 bytes | `limit=5` (1..50), `algorithm="bm25"` (`bm25` or `tfidf`), `excerpt_bytes=512` (1..4096), `timeout_ms=10000` (1..60000) |
| `airust_read_passage` | `document_id`, `revision`, `start_byte`, `end_byte` | `timeout_ms=10000` |
| `airust_find_matches` | `text`: nonblank text, at most 4096 bytes | `limit=5` (1..50), `strategy="token_overlap"` (`exact`, `fuzzy`, `token_overlap`), `min_similarity=0.5` (finite 0..1), `timeout_ms=10000` |
| `airust_relate` | `id`: loaded entry ID, at most 1024 bytes | `limit=5` (1..50), `timeout_ms=10000` |
| `airust_status` | None | No arguments; returns counts, worker use, version and read-only status. |

`airust_search` returns a bounded `retrieval` response with ranked hits, relevant excerpts, entry-relative spans, matched terms and evidence assessment, plus parallel `sources` records. Existing [retrieval](RETRIEVAL.md) and [evidence assessment](ASSESSMENT.md) semantics apply. Sources explicitly report `snapshot_bound`, `snapshot_unavailable` or `not_attached`. The latter two make no claim to verified global document coordinates.

When a snapshot and reserved entry descriptor are both loaded, AIRust validates the descriptor, entry ID and full entry text before serving. Excerpt spans are projected into that exact snapshot. `snapshot_bound` identifies consistency with supplied bytes and metadata; it does not prove publisher authenticity, factual truth or that a PDF extraction found every page.

`airust_read_passage` uses a loaded document ID, a 64-character lowercase hexadecimal revision and exclusive-end UTF-8 byte positions. The requested range must be inside the snapshot, at character boundaries and at most 4096 bytes. It returns exact text, a checked passage descriptor, source reference and source fingerprint. IDs are map keys, **never file paths**. PDF file byte offsets, screen coordinates and character counts are different coordinate systems.

`airust_find_matches` returns ID, score, explicit character/token similarities, a prefix excerpt, truncation flag and binding status. It has no embedding or semantic-equivalence claim. Its character explanation may be expensive on long text, so cancellation and deadlines also apply inside edit-distance rows. `airust_relate` suggests shared-term connections with `requires_review: true`; it does not infer logical contradictions or prerequisites. Its shared-term list is capped at 32 terms per suggestion and exposes a truncation flag.

Tool responses contain `structuredContent` and an SDK-compatible textual JSON copy in `content`. Successful content has `schema_version: 1`, `operation` and `data`. Returned source content is marked `untrusted_source_content: true`. Result budgets account for **both copies and JSON escaping**. Reducing hit count or excerpt size can resolve an `output_budget` error. `read_passage` remains bounded even for a large snapshot.

## Response schemas

All five tools advertise an `outputSchema` in `tools/list`; Rust callers inspect it through `tool_definitions()[...].output_schema`. Each is a self-contained JSON Schema 2020-12 object with local `$defs` references. No URL is fetched to resolve it. Definitions are cached and shared rather than rebuilt for each discovery request. The [complete version-one bundle](schemas/mcp-output-v1.json) is included in the crate.

Validate **`result.structuredContent`**, not the entire JSON-RPC message or its textual content block. Each tool schema has two alternatives:

- Success: `schema_version: 1`, that tool's fixed `operation`, and typed `data`. Known nested ranking, evidence, source, passage, match, relation and status fields are described.
- Tool failure: `schema_version: 1` and a structured `error` object containing `code` and `message`. The outer result also has `isError: true`.

Check `isError` before reading success data. The textual JSON copy contains the same object. A JSON-RPC protocol error has a different outer shape; cancellation can produce no tool result at all. Keep those lifecycle paths separate. Existing payload fields and `schema_version` remain version one. Clients supporting earlier AIRust releases should tolerate absent `outputSchema`; this release always supplies it for its five tools.

The schema validates structure and fixed declarations. It cannot establish source authenticity, permissions, factual truth, cross-field span ordering, a hash match or byte-range membership. AIRust's runtime checks still validate bindings and ranges, and the client still owns its authorization and rendering. Metadata can contain arbitrary untrusted JSON. Reject unexpected schemas/references in a hostile-server setting rather than enabling unrestricted remote resolution.

AIRust does not validate each response with a general JSON Schema engine in production. The schemas come from one bundled source; independent validators check actual success/error replies in the protocol suite and retained-document campaign. The test-only validator uses no HTTP/file resolver features. [Public reproduction programs](validation-0.4.4/README.md) include a separate schema checker that has no AIRust dependency. The bundle and per-tool definitions remain bounded by a discovery-size regression.

## Progress, cancellation and agent communication

The server supplies an `instructions` string during discovery/initialization. It asks the agent to explain its work before longer operations, request progress, treat source text as data, cite exact revisions and explain limitations afterward. **This is guidance to the client/agent, not an enforceable agent policy.** AIRust cannot force an LLM to speak or draw a loading bar. Configure the host agent's own rules and progress UI as well.

For an established legacy stdio session, request progress with a unique active token:

```json
{"jsonrpc":"2.0","id":17,"method":"tools/call","params":{"name":"airust_search","arguments":{"query":"Rust ownership","limit":5},"_meta":{"progressToken":"search-17"}}}
```

AIRust sends `notifications/progress` only when the client supplies a token. The token is echoed. `progress` starts at 0 and increases per notification; it is a **notification sequence**, not a percentage. `message` describes the actual current phase and the latest available count. Counts are local to a phase, can restart and may be coalesced. Overall duration is unknown, so `total` is omitted. Show an indeterminate loading indicator and the message; do not invent an ETA or correctness percentage. Finish it on the request result, error, cancellation or connection closure, even when no completion notification arrives. Correlate concurrent responses by JSON-RPC request ID, not arrival order. A fast call may have only start and completion notifications. When delivery succeeds within the request budget, the final notification precedes the result; none are generated after completion. Transport-buffered notifications can already be in flight when the client cancels.

Missed heartbeat ticks are skipped instead of replayed in a burst after a delay. This keeps progress focused on current work; it does not turn notifications into an ETA or a reliable percentage.

To cancel that request over stdio:

```json
{"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":17,"reason":"User stopped the search"}}
```

The SDK cancels the request handler and suppresses its response. AIRust's drop guard signals the core cancellation token. Checkpoints cover candidate collection, ranking, selected character explanations and edit-distance rows/cells. The worker retains its slot until it really exits. Closing the connection or dropping the serving future also signals work to stop. Unknown/already-completed cancellation IDs are harmless. Request IDs and progress tokens are separate concepts.

`timeout_ms` is an absolute operation budget measured by a monotonic clock, beginning before start-progress delivery. Progress never extends it. Since 0.4.8, the start, heartbeat and final notification waits each observe that same absolute deadline and connection cancellation. An expired request returns `deadline_exceeded` without classifying that expiry as a transport failure; work is not started after expiry during initial notification delivery. Deadline errors are visible tool errors. Startup, OS scheduling, allocation, a JSON serialization section or a client that stops reading can still delay cleanup; cooperative cancellation is not a process-kill sandbox. The executable gives the runtime one second to shut down; library `serve` allows workers one second to return. A progress send that reaches its 100 ms transport-stall limit closes the session and stops its work. If the request deadline or cancellation wins first, that shorter request is stopped without treating it as a 100 ms transport failure. Response writes, OS scheduling and non-yielding work are still outside a hard wall-clock guarantee.

Closing stdin starts connection shutdown and cancels outstanding requests. Keep the connection open until the responses you need have arrived; EOF is not a request to drain and deliver every pending result.

A useful host rule is: "Before longer AIRust work, state the action and scope. Show supplied progress messages. On cancellation, say that work was stopped. Afterward, report the result, evidence and any missing source binding." The host must implement that rule. Keep source text out of system instructions and never execute commands suggested inside retrieved content.

## Protocol and errors

The official SDK handles JSON-RPC and lifecycle negotiation. Wire tests exercise legacy `initialize` / `notifications/initialized` with protocol `2025-11-25`, and `2026-07-28` discovery with required per-request protocol/client-capability metadata. For modern discovery, use `server/discover` and the SDK's request metadata helpers, or the [public wire validator](validation-0.4.4/mcp_probe.py). This release does not advertise HTTP, subscriptions, persistent tasks, resource subscriptions, prompts or sampling. Tool calls complete in one response; input continuations are rejected explicitly.

Unknown tools and unsupported list cursors use JSON-RPC invalid-params errors. Ordinary invalid arguments, worker saturation, missing revisions, deadlines and output limits return a tool response with `isError: true` and a structured `error.code` / bounded `error.message`. Codes include `invalid_arguments`, `argument_budget`, `busy`, `deadline_exceeded`, `invalid_data`, `operation_failed`, `output_budget`, `progress_token_budget`, `progress_token_in_use` and `unsupported_continuation`. A cancelled request normally receives no response. Protocol or connection-budget violations can close the connection instead of amplifying an error flood.

## Resource defaults

`McpLimits` exposes every setting below to Rust callers. `--timeout-ms`, `--max-concurrent`, `--max-file-mib` and `--max-result-kib` override corresponding CLI defaults. CLI option values are checked, including multiplication overflow. Run `--help` for syntax.

| Boundary | Default |
| --- | --- |
| Entries / snapshots | 1,000,000 / 1,000 |
| Accounted dataset bytes | 512 MiB (entry IDs/text/encoded metadata plus registered snapshot text) |
| Entry text / ID / encoded metadata | 1 MiB / 1024 bytes / 16 KiB |
| Startup file | 256 MiB each, bounded read before JSON parsing |
| Query or match input / hits / excerpt | 4096 bytes / 50 / 4096 bytes |
| Full tool result | 64 KiB; checked encoded `CallToolResult`, with 1024 bytes reserved for envelope overhead |
| Active workers | 4; excess work returns `busy` |
| Operation time / maximum requested time | 10 s / 60 s |
| Progress interval / send wait | 100 ms / 100 ms |
| Input line / tool argument encoding | 64 KiB / half the configured frame limit |
| Pending protocol requests | 64 |
| Input line frames / decoded incoming messages | 256 each per one-second fixed window, including malformed line frames |
| Raw input | 1 MiB per one-second fixed window, 256 MiB per connection |
| Request IDs | 128 encoded bytes, no reuse within a connection |
| Total requests | 100,000 per connection; reconnect with new state afterward |
| Active progress token | 1024 encoded bytes, unique among active calls |

Configuration also has supported ceilings: at most 64 workers, 1024 pending requests, 1 MiB frames/results, 1024-byte request IDs, one million total requests and five-minute maximum requested timeout. The result minimum is 4 KiB; the progress interval minimum is 10 ms. A worker count cannot exceed pending-request capacity.

An overlong line or invalid JSON syntax is discarded by the bounded SDK line decoder; a later valid line can be processed. Valid JSON with an invalid protocol shape can return an invalid-request error. Raw-byte budgets apply even to discarded bytes and incomplete lines. Line-frame rate checks include malformed and empty lines before SDK decoding. Duplicate/overlong request IDs and connection count/rate budgets close the session. This deliberately strict ID policy is compatible with clients using fresh IDs; reconnect if the policy or total-request budget is reached.

These are work/payload bounds, not a precise process-RAM quota. Index vocabulary, normalized strings, page maps, JSON parsing, SDK queues, allocator overhead and OS buffers need extra memory. Fixed windows can admit a burst spanning a window boundary. Startup is synchronous and has no request deadline. Apply OS resource controls if untrusted data or a host requires a hard process limit. Read the [security model and tested attack cases](MCP-SECURITY.md) before exposing a collection.

## References and validation

Structured responses follow the [official tools/output-schema contract](https://modelcontextprotocol.io/specification/2026-07-28/server/tools). The implementation uses the [official Rust SDK](https://github.com/modelcontextprotocol/rust-sdk). Progress and cancellation follow the [MCP progress pattern](https://modelcontextprotocol.io/specification/2026-07-28/basic/patterns/progress) and [stdio cancellation pattern](https://modelcontextprotocol.io/specification/2026-07-28/basic/patterns/cancellation). AIRust's local threat model is narrower than a remote authorized MCP service; see the [official security guidance](https://modelcontextprotocol.io/docs/2026-07-28/tutorials/security/security_best_practices).

See [executed results](CHANGES-0.4.4.md), [security tests](../tests/mcp.rs), [executable tests](../tests/mcp_cli.rs), [controlled matching tests](../tests/controlled_matching.rs) and the public validator. Test evidence establishes behavior for those cases and platforms; it is not an independent penetration audit or a guarantee against all attacks.

## Bounded JSON checks in 0.4.9

McpServer checks sizes with the same bounded serializer used by retrieval, without retaining JSON needed only for counting. Limits count encoded bytes, including escapes and multi-byte UTF-8; they are not character counts or the unescaped query length. Configuration limits, `argument_budget`, `output_budget`, response schemas and session-recovery behavior remain. Actual response envelopes are still encoded and owned. A counting sink reduces temporary allocation; it does not impose a total process-RAM quota. Search also benefits from the existing excerpt API's simpler ASCII/mixed-text path. Clients need no protocol change. See [MCP measurements and limits](CHANGES-0.4.9.md).

## Core execution in 0.4.10

McpServer uses the same central Engine. Startup benefits from direct ASCII normalization where applicable; BM25/TF-IDF candidate collection uses bounded ordered merge for up to 32 known terms and the prior tree route for broader queries. Clients use the same tool names, arguments, schemas and error codes. Source binding, exact reads, byte budgets, progress, cancellation, worker admission and absolute deadlines remain. No new MCP capability or permission is added. Lower candidate-allocation traffic is not a hard RAM quota or universal latency guarantee. Read [both preplanned MCP campaigns and limits](CHANGES-0.4.10.md).

## TF-IDF and broad search in 0.4.11

Use the existing `airust_search` arguments and select `algorithm: "tfidf"` when needed. The same central Engine now uses protected hash norm hints and sparse broad scoring. Tool names, schemas, source binding, byte limits, worker admission, absolute deadlines, progress and cancellation remain. Broad scoring still inspects bounded batches between checkpoints; it does not guarantee a hard process-time or RAM quota. Cache lookup improvements can cost different retained cache memory. Read [both initial/warm MCP campaigns and limits](CHANGES-0.4.11.md).

## Cold TF-IDF cache in 0.4.12

Select `algorithm: "tfidf"` with the existing `airust_search` arguments. The same central Engine stores smaller private norm-cache keys; no new tool, option or response field is required. Public IDs, scores, source bindings, spans and output schemas remain. Hints are transient and discarded for changed collections. Worker admission, byte limits, deadlines, progress and cancellation retain their contracts. This is not a hard process-memory quota. See [separate initial/warm MCP campaigns, cache heap and RSS](CHANGES-0.4.12.md).

## Cold search execution in 0.4.13

Use the existing `airust_search` arguments with `algorithm: "tfidf"`. The central lexical engine reuses exact word weights in a bounded memo while computing missing norms. It does not prewarm at startup or store extra corpus state. The temporary memo is allocated only if needed, adds about 4 KiB on the measured build and is released after the request. Existing tools, output schemas, IDs, scores, source bindings, spans, progress, cancellation, worker admission and deadlines remain. Warm-hit and tiny-input costs can differ from large cold queries; see [all initial/repeated measurements and memory costs](CHANGES-0.4.13.md).

## Warm and small execution in 0.4.14

Keep the existing `airust_search` arguments and `algorithm: "tfidf"`. The central Engine batches warm hint reads, uses an eight-slot inline word memo for small cold work and promotes to the existing bounded heap memo after enough completed work. The server adds no tool, argument, response field or startup prewarming. Scores, IDs, source binding, spans, progress, cancellation, worker admission and deadlines remain. Contended hints fall back to exact computation without waiting; their publication can be deferred. Core changes do not imply equal gains in tiny stdio round trips. Read [both four-input MCP campaigns, small controls and mixed work](CHANGES-0.4.14.md).

## Small requests in 0.4.15

Use the same tools and arguments. After an admitted worker starts, the handler immediately waits for completion, cancellation or the original absolute deadline. It no longer awaits an initial timer tick. The first periodic progress update is scheduled one `progress_interval_ms` later, with missed ticks skipped. A requested start message still precedes worker launch and a completion/error message precedes the response; small requests can therefore have just two notifications. Without a progress token there are no notifications and no initial timer wait.

To request progress, pass `_meta: {"progressToken": "unique-request-token"}` in the existing `tools/call` parameters, outside tool arguments. Each simultaneous request needs a unique token. The agent/host decides how notifications appear and whether it narrates its actions; these notifications do not predict remaining time. Defaults, rate/frame/result budgets, worker admission and bounded notification delivery remain. Lower latency does not increase the configured requests/messages allowance. Read [all nine cases in both campaigns](CHANGES-0.4.15.md).

## Large search work in 0.4.16

Use the same airust_search arguments and algorithm values. The central Engine can reuse frequencies from sorted postings on large small-term requests, reducing repeated document word-map lookup. No tool, argument, result field or startup work is added. Scores, source positions, scopes and exact responses remain. The cursor state is temporary and bounded; large calls can allocate extra query scratch while persistent index memory remains.

The 0.4.15 timer behavior remains: immediate completion/cancellation/deadline selection, requested start/completion and delayed periodic updates. Admission, cooperative checks, notification delivery limits, disconnect drain and byte/rate/result budgets remain. Faster calls do not increase configured allowances. Read [all small/large timings, direct control modes, earlier-version reference and limits](CHANGES-0.4.16.md).

## Shared Core candidate collection in 0.4.17

Use the existing tools, arguments and algorithms. A query with one known indexed term can skip merging its single candidate list in the central Engine. Multiple terms and the previous large-query frequency reuse retain their strategies. Scores, scopes, evidence, source positions, schemas and saved collections retain their contracts. No new endpoint, argument or startup work is added.

Every posting still checks the cooperative control. Worker admission, progress/start/completion delivery bounds, absolute deadlines, connection cancellation/drain and frame/rate/result limits remain. Small Core improvements do not imply equal changes in MCP round trips. Read [initial/repeated transport results, small Core controls and negative outcomes](CHANGES-0.4.17.md).

## Two-list Core merging in 0.4.18

Existing tools, arguments and algorithms remain. The shared Engine can directly merge exactly two known terms' sorted candidate lists. It keeps both overlap visits and every tail checkpoint, then uses the same scoring, scopes, source positions and result schemas. One-term, wider-query and large-scoring strategies remain. No endpoint, startup work or feature dependency is added.

Progress delivery, admission, worker leases, original absolute deadlines, connection cancellation/drain and frame/rate/result budgets keep their contracts. Direct Core gains and full transport latency have different costs. Read [initial/repeated MCP results, complete response checks, small guards and all negative observations](CHANGES-0.4.18.md).

## Broad search execution in 0.4.19

Existing tools, arguments, algorithms and response shapes stay the same. Broad search winner explanations use the shared Core exact dictionary intersection, avoiding repeated absent-word probes in sparse documents. Scores, matched terms, scopes, evidence and original positions retain their contracts. No new endpoint, cache or permission is added.

The 4,096-byte query limit and existing 256-unique-normalized-term assessment limit are separate. A query can fit the byte budget and still exceed the term budget; this returns operation_failed, reports error progress and exposes no partial successful result. Twelve explicit rejection/recovery checks cover both rankers and all three compared releases. This release does not raise limits.

Cancellation/deadline checkpoints cover each winner and every 128 actually inspected explanation words. Sparse scans may inspect fewer words and make fewer checks. Admission, worker leases, absolute deadlines, progress delivery, connection drain and protocol budgets remain. See [actual reply validation, all timing boundaries and limits](CHANGES-0.4.19.md).

## Large-query document access in 0.4.20

Existing tools, arguments and response schemas remain. The shared Engine uses ordered document access for large sorted candidate sets, retaining exact scores, source positions and filters. Small sets keep their previous lookup path. The cursor checks before a fallback seek after eight lower map keys; original candidate/scoring checks remain. It adds no endpoint, permission or persistent cache.

Admission, absolute deadlines, progress delivery, worker leases, disconnect cancellation/drain, byte/term/frame/result/rate limits and no-partial-result errors retain their contracts. Startup indexing remains outside tool deadlines. Sparse gaps, query-local stack state, startup, Rust search and transport are separate costs. Read [actual MCP comparisons, initial/repeated timings, RAM and all limits](CHANGES-0.4.20.md).
