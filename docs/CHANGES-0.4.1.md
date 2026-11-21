# AIRust 0.4.1: Optional bounded MCP in the same crate

AIRust can now serve a selected knowledge collection to an MCP client. Enable `mcp` to use the public `McpServer` API or install the `airust-mcp` executable from this same crate. There is no separate AIRust MCP package and no DenkMaps adapter. The default library stays feature-free.

The five read-only tools search with BM25/TF-IDF, read exact loaded document passages, find text matches, suggest shared-term connections and report status. Sources explicitly distinguish verified snapshot bindings, unavailable snapshots and legacy entries without bindings. An ID never becomes a file path. Startup files are explicit; tools cannot write, run shell commands or fetch URLs.

The feature uses the official `rmcp = 3.11.0` Rust SDK. It enables documents, retrieval, execution and Tokio, but no PDF parser, HTTP server, SQLite or model. Clients request progress and can cancel a running operation. Time budgets are absolute; notifications do not extend them. Worker permits remain occupied until the actual computation stops. Existing core APIs/formats remain, with additive controlled matching/relations and their separate error type. See the [complete API and installation guide](MCP.md), [security model](MCP-SECURITY.md) and [API cookbook](API-COOKBOOK.md).

## Defects found and corrected

A normal disconnect during progress could be misclassified as a delivery/security failure. The transport now signals EOF immediately and distinguishes a closed connection from a stalled active peer. Tests cover worker cleanup and retained permit ownership.

An error containing many control characters produced a 6,752-byte response even with a 4 KiB result setting. The complete textual/structured error now passes an encoded-size check and shortens at UTF-8 boundaries. The same independent stdio reproducer returns 1,760 bytes after the fix. Large worker values are also checked before building the SDK's duplicated textual/structured JSON. These figures describe that fixed test case, not every possible message size.

Malformed/empty line frames now count toward a rate guard before SDK decoding, alongside decoded-message, raw-byte, pending-request, total-request and ID limits. This prevents a malformed frame flood from bypassing the decoded-message counter. Oversized lines are discarded with bounded recovery; byte limits still apply before a newline arrives.

Startup errors escape terminal controls and cap diagnostics at 2,048 characters. Normal server stdout is JSON-RPC only. CLI help/version and diagnostics use fallible writes instead of panicking on a closed output. The new controlled edit-distance path uses two rows sized to the shorter text and checks within long rows. Ordinary matching retains its existing path.

## Executed validation

The local run passes **202 default tests, 783 all-feature checks including 36 documentation examples, 39 isolated declared feature entries and five Python tests**. Strict Clippy and documentation checks pass. Rust 1.89 covers provider-neutral features including MCP; Rust 1.92 covers the optional native ONNX adapters. An independent Rust 1.89 consumer exercises the existing public APIs and new MCP construction/configuration/schema and controlled matching interfaces. Patch-level API comparison against 0.4.0 passes 229 checks with 31 tool skips.

Fifteen raw-protocol tests cover legacy initialization and modern discovery, schemas/annotations, all tools, denied fields/unknown tools, Unicode limits, exact revisions/ranges, missing snapshots, hostile source content, escaped output/error bounds, live cancellation, absolute deadlines, busy slots, duplicate progress tokens, pending-request/ID/session guards, malformed/fragmented/oversized frames, valid/malformed message floods, raw-byte floods, disconnects and a peer that stops reading. Four CLI tests launch the actual binary and cover files/options, bounded startup, protected stderr and unchanged input bytes.

Three controlled matching integration tests check score/explanation equality, errors, cancellation and deadlines. Two additional unit tests compare controlled distances with independent `strsim` distances over short multilingual strings and stop inside a long row. Existing document tests retain their 26 cases. This is an adversarial regression campaign, not an independent penetration audit or proof against every attack.

The unchanged backend passes 14 regressions and the 381-file provenance check. Its source remains exactly `pdf-backend-v0.3.5`; parsing, cleanup, chunking and supervised workers are unchanged. Fresh dependency audits find zero known vulnerabilities, with three main and four backend maintenance warnings retained explicitly. See [machine-readable validation](measurements/0.4.1/validation.json), [audits](measurements/0.4.1/audit.json) and [backend audit](measurements/0.4.1/backend-audit.json).

## Real knowledge through actual MCP processes

The final executable is tested against **all 35 retained document snapshots**, each in its own stdio connection. Two fixed literal probes per document use both algorithms: 140 sequential searches. Their 624 returned passages are checked against exact snapshot bytes and read back through MCP. Every connection then submits four concurrent searches: 140 more searches, with checked interleaved progress and complete direct-API equality.

A separate process loads the complete canonical collection: **167,942 entries from 35 PDFs / 49,086 physical pages**. Seventy literal probes use both algorithms for 140 sequential searches, followed by four concurrent searches. Complete retrieval JSON equals a separate direct Rust API oracle, including ranking evidence, spans, excerpts, assessment and budgets. Legacy records remain explicit about missing snapshot bindings; they do not receive guessed global positions.

Together this is **424 searches across the final 36 correctness connections**, plus 624 exact passage readbacks. All input hashes are unchanged. The snapshot positions refer to the 48,983 provided text pages retained in 0.4.0. This campaign restores those snapshots; it does not claim new PDF extraction, independent glyph coverage, OCR or human relevance evaluation. Historical parser/model/large stress evidence is inherited only through unchanged modules.

The affected core comparison parses the full canonical export and indexes prefixes of 1,000 and 10,000 entries in both 0.4.0 and 0.4.1. For each prefix it compares 72 complete matching results and 12 relation results, including current controlled variants. Every matching probe must return a hit. Exact/token probes use full real entries; fuzzy probes edit available 24..128-character entries. The 1,000-entry prefix has one qualifying fuzzy source, reused with twelve distinct edits; the 10,000-entry prefix has twelve. Negative cases are covered separately by the test suite. All compared results agree.

The independent legacy corpus program also executes a checked 1,000-entry working prefix for one epoch after parsing all 167,942 entries. Incremental/rebuild equivalence, JSON/concurrent signatures, retained readers, cancellation and deadlines pass. This is not a new million-record concurrent stress campaign.

## Quiet MCP costs

Fifteen fresh release server processes cover five cases with three repeats in alternating order, after deliberate builds/checks finish. Input caches are warm; unrelated desktop activity is uncontrolled. MCP uses Rust 1.98 on macOS ARM64, thin LTO and one codegen unit. The independent direct oracle uses Rust 1.89; this is a correctness comparison, not a cross-compiler performance claim.

Startup-to-initialized includes process launch, file decoding, checked snapshot restoration where applicable, index construction and lifecycle exchange. Search times include the complete local request, computation, progress and response transport, excluding later passage readbacks. Canonical search medians have 210 samples per algorithm; each individual-document median has six samples per algorithm, spanning its two fixed queries and three repeats.

| Case | Startup median ms | BM25 round trip median ms | TF-IDF round trip median ms | Server peak RSS median MiB |
| --- | ---: | ---: | ---: | ---: |
| canonical | 6283.3 | 18.80 | 155.94 | 2269.4 |
| vkspec | 421.2 | 5.08 | 28.80 | 165.2 |
| scipy | 278.3 | 5.74 | 40.84 | 119.6 |
| racket-reference | 101.5 | 2.17 | 7.85 | 48.1 |
| pdf-12 | 45.3 | 1.64 | 3.63 | 24.6 |

The canonical file has 167,942 old chunk entries. The four snapshot cases index nonblank whole pages: vkspec has 5,491 provided pages, scipy 3,581, racket-reference 1,438 and pdf-12 494. Their entry granularity differs from the canonical chunks. The tested TF-IDF implementation is more expensive on these probes; the measurements do not promise equivalent speed between algorithms.

Peak RSS is the single server child's lifetime maximum, including startup JSON buffers, original/normalized entries, postings, snapshots, runtime and transport. It excludes the Python validator and its expected text. The 512 MiB configured dataset accounting is **not** a 512 MiB RAM guarantee; the full canonical run peaks around 2.2 GiB. Size collections and OS limits using measured host requirements. Startup is synchronous and outside request progress/deadlines, so clients must allow for collection loading.

Four concurrent requests per connection complete with distinct checked progress streams and oracle-equal responses. Their separate elapsed fields remain in the raw samples; they are not mixed into sequential latency medians or presented as a general concurrency-capacity result. Three warm process repeats do not establish cold-start, p95/p99 or hard latency/RAM limits.

## Ordinary matching costs

Three fresh Rust 1.89 release processes repeat the 10,000-entry paired comparison. Both engines enable `execution`; timers cover ordinary matching, alternating old/new order by query. Controlled oracle calls and serialization checks are outside the timed sections. Each strategy has 72 paired calls over the three repeats.

| Strategy | 0.4.0 median ms | 0.4.1 median ms | Paired calls |
| --- | ---: | ---: | ---: |
| Exact | 0.02346 | 0.02135 | 72 |
| Fuzzy | 5.51040 | 5.36202 | 72 |
| TokenOverlap | 31.30213 | 31.18379 | 72 |

No material ordinary-matching regression appears on this workload. Small differences are not claimed as a universal speedup. The early setup failures, preliminary probes, binaries, complete samples and final positive-hit checks remain retained privately. Public data contains aggregates and hashes rather than document text. See [all measurements](measurements/0.4.1/) and [reproduction programs](validation-0.4.1/README.md).

## Retention, boundaries and release gates

All eight canonical exports, all 35 original PDF hashes and all 35 revisioned snapshots remain unchanged. The canonical SHA-256 remains `13168d87872a137355fd8499c90a6084f52243a2ec7456f29b5fcb2a1c7423c3`. New MCP/oracle checks, measurements and binaries are additional retained artifacts. No application database or DenkMaps source is changed. No knowledge is deleted or silently migrated.

Read-only tools reduce the exposed capability surface; they do not sandbox a compromised host or make source text trustworthy. Every connected client can read the selected collection. There is no remote authorization or tenant isolation. The agent instruction asks for narration and safe source handling, but the host must enforce agent behavior and render progress. Resource limits are documented work/payload bounds, not hard OS isolation. Exact positions identify supplied text revisions, not PDF drawing coordinates or source authenticity.

Clean packaging, extracted-package tests, dry-run, exact-commit Linux CI and registry/VCS/tag/asset verification gate publication. Actual post-publication registry consumers and immutable identities are recorded in the release verification asset. The documentation-only final edit rebuilds to the **same MCP executable hash** as the measured binary. No Docker execution, live third-party MCP-client UI test or independent security-audit claim is made. See [security boundaries](MCP-SECURITY.md) before deployment.
