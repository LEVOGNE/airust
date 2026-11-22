# MCP security model

AIRust 0.4.4 serves a deliberately bounded, read-only collection over local stdio. Its five tools can search, compare, suggest links, inspect counts and read an exact loaded text range. They have no command runner, file browser, URL fetcher, database mutator or model downloader. A path-like document ID is just a lookup key. Tool annotations describe this contract; enforcement is in the tool implementations and transport guards.

## Trust boundaries

The process owner chooses the executable, startup files and limits. A connected client can read **all loaded knowledge**, not only its current search hits. There are no per-user permissions or separate tenants within one server. Only start it with data that the chosen client and its configured model providers may receive. The process has the current user's OS privileges; read-only tools do not sandbox a compromised executable, OS or dependency. Use a restricted account/process environment for sensitive deployments.

The binary opens no TCP listener. There is no OAuth/token passthrough, HTTP endpoint, remote session authorization or TLS implementation in this feature. Do not put stdio behind an unauthenticated network bridge. A custom `serve(reader, writer)` caller must authenticate, authorize and limit its transport before handing it to AIRust. Sharing clones also shares their collection and global worker slots; it is not tenant separation.

The corpus, excerpts, IDs and metadata are untrusted content. Returned text can contain hostile instructions or misleading claims. AIRust preserves source data and marks source-bearing responses as untrusted; it neither executes those instructions nor proves that a client model will ignore them. The initialization guidance asks the agent to narrate work and handle source content as data. The client must enforce its own system policy, tool permissions, output rendering and user approvals. Never insert excerpts into a privileged instruction channel or render them as active HTML.

A document revision/hash proves consistency with supplied data, not authenticity or semantic correctness. Bindings are verified against loaded snapshot text and descriptors. Missing snapshots report `snapshot_unavailable`; legacy entries report `not_attached`. An attacker who controls both a snapshot and its hashes can supply a new internally consistent snapshot. Authenticate source acquisition outside this library.

Response schemas describe data shape; they do not grant permission or make document text trustworthy. A valid success object can still contain hostile or false source text. The server uses bundled local schemas without runtime remote reference resolution. Independent test/client validation has no authority over another tool or a host agent. Registered snapshots are fully readable even when `McpServer::new` indexes only selected passages.

## Defensive boundaries

| Threat | Implemented boundary | Remaining responsibility |
| --- | --- | --- |
| Tool arguments contain shell commands, paths or URLs | Fixed tool allowlist, typed arguments, denied unknown fields; no shell/network/free-path operation | Client must not execute returned source text through another tool. |
| Source asks the agent to ignore rules or steal secrets | Untrusted content marker and agent instructions; text remains inert data in AIRust | Host agent policy and approval; no end-to-end prompt-injection guarantee. |
| Huge or malformed input messages | Bounded SDK line decoder, argument bounds, raw-byte limits even for invalid/incomplete JSON and line-frame rates before decoding | OS/process hard quotas; fixed windows are not perfectly smooth rate limiters. |
| Flood of requests, duplicate IDs or repeated cancellation | Pending-request, total-request, ID-size/reuse and frame/message-rate guards | Reconnect with fresh IDs after a session closes; client controls its retry policy. |
| Expensive fuzzy comparison or repeated work | Shared worker permits, immediate busy response, absolute deadlines and checkpoints inside edit distance | Cooperative work may take time between checkpoints; OS termination for hard limits. |
| Client cancels or disconnects | SDK request cancellation, core drop guard, worker retains permit until exit; session close signals stop | Client must display cancellation and stop treating already-buffered messages as new work. |
| Client stops consuming progress | Coalesced phase state, throttled notifications, the shorter of remaining request time and the 100 ms send wait; actual transport failure closes the session | Host transport and OS buffers still add memory; no arbitrary native provider is run by MCP. |
| Response is huge or contains heavily escaped strings | Encoded result check includes text and structured copies; bounded excerpts/hits and error messages | Result cap is on the tool object with reserved overhead, not every conceivable outer transport wrapper. |
| Oversized or conflicting startup collection | Bounded file read, explicit entry/document/payload limits, duplicate-ID rejection, snapshot integrity checks | JSON/index allocations, recipes/page maps and overhead are not fully covered by payload accounting; startup is synchronous. |
| Wrong document revision or fake span | Exact revision lookup, UTF-8/range checks and entry binding validation | Source authenticity, PDF extraction coverage and factual truth remain external concerns. |
| Secret leakage in logs | No source paths, tokens, text or client cancellation reasons in normal status/startup logs | CLI diagnostics escape control characters and are capped at 2048 characters, but supplied IDs may contain user data; protect stderr and host logs. |

Defaults and configurable ceilings are listed in [MCP.md](MCP.md#resource-defaults). Limits are checked before serving. Every public serving entry point wraps the official SDK with AIRust's guards. A separate private handler prevents accidentally using an unguarded public SDK handler.

Each connection tracks the lifetime of its own blocking workers. Shutdown waits for those workers, including cancelled work still winding down, without reserving the shared admission permits or waiting for other connections. Missed progress ticks are skipped rather than replayed in a burst. The SDK rejects JSON-RPC batch arrays as invalid requests; no batch request bypasses the individual-request guards.

Admission closes under the same lock that creates tracked worker reservations. Each reservation owns its permit and tracking token before any progress-send await, then transfers them into the blocking job. Late admissions are rejected; dropped startup work and panicking jobs release their leases. A task-tracker close alone would not provide that admission barrier. Dropping a serving future still signals cooperative cancellation; it cannot synchronously wait inside Rust's destructor.

## Executed attack-oriented checks

The release suite exercises raw JSON-RPC through the public transport rather than only calling Rust handler methods. It covers legacy and modern discovery, tool input/output schemas, independently validated success/error envelopes, unknown tools and fields, Unicode byte limits, invalid enum/range/time values, path traversal strings, stale revisions, invalid UTF-8 positions, edited passage bindings, huge escaped results, malformed and fragmented frames, line overflow and recovery, duplicate/long request IDs, total request count, pending-request limits, decoded-message floods, malformed-frame floods and raw-byte floods.

It also exercises live cancellation during expensive fuzzy ranking, deadline expiry despite progress, worker saturation, duplicate active progress tokens, a client that stops reading, disconnect cleanup, missing snapshots, hostile source instructions and subprocess stdin/stdout separation. A two-connection regression closes one connection while another owns the only worker slot, then cancels the surviving work. A separate deterministic regression holds an unrelated clone's permit throughout shutdown. A batch-array probe checks rejection and subsequent valid-request recovery. Independent stdio checks load retained real PDF-derived knowledge and document snapshots. See the [release report](CHANGES-0.4.4.md) for actual counts, measurements and scope.

This is an automated adversarial regression campaign. It is not a third-party security audit, a full OS penetration test or a proof that every parser/SDK/dependency is defect-free. Dependency advisories are checked at release time; absence of known advisories does not prove absence of vulnerabilities. Unchanged PDF-parser results are retained as historical evidence rather than presented as a new MCP parser audit.

## Operating guidance

Use an exact crate version and the published checksum/tag for reproducible deployments. Keep the executable and its directory owned by a trusted user. Launch it directly with an argument array, not by interpolating untrusted paths into a shell command. Supply only the intended collection, keep its original snapshot revisions, use conservative limits and avoid unnecessary inherited secrets. The server needs no crates.io token or cloud credentials at runtime.

For an incident, stop the client process, preserve the knowledge/snapshot bytes and record the crate version, configuration and relevant protocol messages with sensitive text removed. Reproduce with the public validator and a minimal synthetic fixture. Do not publish private documents or tokens in a GitHub issue. Upgrading/restarting rebuilds the in-memory index; this release does not write knowledge or repair original files in place.

Since 0.4.8, progress delivery checks request expiry and connection cancellation separately from transport errors. Start/heartbeat/final sends share the original monotonic request deadline. An expired request does not mark a healthy session as failed. A stalled or failed transport still closes it when that outcome wins. A final completion notification can be omitted on expiry. Cooperative scheduling and bounded queues are not a hard process-time or RAM guarantee.

Since 0.4.9, size-only guards use a non-retaining counting sink through the shared bounded JSON serializer. Exact encoded byte limits still include escaping and UTF-8. Tests cover accepted values exactly at the argument limit, rejected ASCII/Unicode/escaped values above it and later successful requests on the same connection. Existing parser, depth/node, frame, response, worker and time limits remain. This does not eliminate already owned request/response data or establish an OS memory quota. See [executed checks and scope](CHANGES-0.4.9.md).
