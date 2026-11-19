# Security and operating boundaries

AIRust's default knowledge core is an in-memory library. Optional local inference, dashboard, file administration, crawler, and process controls add separate operating boundaries. The existing web boundaries were introduced in 0.1.9; the optional embedding boundary is described below. Validation results belong in the versioned release reports.

## Bind address and authentication

The web listener defaults to `127.0.0.1`. The console uses the following port. `AIRUST_BIND_ADDR` explicitly selects another IP address.

`AIRUST_API_TOKEN` is optional on loopback and required for a non-loopback bind address. A token must contain 32 through 256 ASCII letters, digits, hyphens, or underscores. Once configured, it protects `/api` requests and the console WebSocket.

HTTP clients send `Authorization: Bearer <token>`. The browser prompts for the token and retains it only in JavaScript memory. It must be supplied again after the page's state is discarded. WebSocket clients offer `airust` and `token.<token>` as subprotocols. The credential is not a URL query parameter.

The token grants one shared level of service access. It is not a multi-user account model. Use transport encryption when a deployment crosses an untrusted network, and configure infrastructure independently for that environment.

## Host and origin checks

Requests require an allowed Host header and the receiving service's port. The main server checks its configured port; the console checks the next port.

`AIRUST_ALLOWED_HOSTS` accepts comma-separated explicit hostnames, without ports or wildcards. Defaults include localhost, loopback addresses, and the selected bind IP.

A browser Origin must use the same hostname and either the main or console port, allowing the main interface to connect to its console. Cross-site fetches are rejected. Ordinary command-line clients without an Origin header are accepted if their Host and any configured token are valid.

A reverse proxy must deliberately preserve or rewrite headers to the expected service Host. Public-origin port rewriting is not implicitly supported. A forwarded connection alone does not establish an approved origin.

## Administrative operations

`AIRUST_ENABLE_ADMIN=1` opts into file-administration endpoints, knowledge load/save operations, and console reset. They are disabled by default. Enabling the flag does not remove authentication or origin checks.

Enabled filesystem administration is confined to the server's working directory. It rejects absolute paths, parent traversal, hidden path components, and symlink components. Administrative reads are bounded to 1 MiB.

These path checks address HTTP-supplied paths. A separate local process that can change the same directories can race a path check. The interface is not an operating-system sandbox or a substitute for filesystem ownership controls.

The SQL administration interface is a trusted administrative capability. Enable it only for an operator who is intended to control the application's database.

## Request and message limits

| Boundary | Limit |
| --- | ---: |
| API token | 32–256 permitted ASCII characters |
| HTTP request body | 8 MiB |
| Query input | 1–4,096 UTF-8 bytes, with non-whitespace content |
| Training input or serialized output | 1 MiB each; input must be nonempty |
| Training import batch | 1,000 items |
| Administrative file read | 1 MiB |
| Console WebSocket frame or message | 4 KiB |
| Trimmed console command | 1 KiB |
| SQL query pagination | 1–1,000 rows; nonnegative offset |
| Crawler response body | 2 MiB |
| Crawler traversal depth | 5 |
| Crawler pages per run | 100 |

Training imports validate the complete batch before writing it in one SQL transaction. Invalid input does not partially add a batch.

Multipart uploads share the total request limit with their framing overhead. These boundaries limit individual requests and runs; they are not an aggregate service-throughput guarantee.

## Crawler destinations

The crawler accepts public HTTP and HTTPS destinations. It rejects embedded URL credentials, localhost, and nonpublic resolved addresses. All returned addresses are checked, the selected address is pinned for the request, proxy discovery is disabled, and redirects are disabled.

DNS resolution is bounded to five seconds and a request to 30 seconds. Inter-request delay is clamped to 100 through 30,000 milliseconds.

A URL that redirects to another site is not automatically followed. The operator must supply an eligible final destination explicitly. Network restrictions apply to the optional crawler rather than to library ingestion of application-owned text.

## Local runtime files

`AIRUST_RUNTIME_DIR` must be an absolute path to a real private directory. The default is `$HOME/.local/state/airust`. Existing symlink or insecure runtime directories are rejected. The Unix directory mode is `0700`.

`server.json` records PID, executable, and process-start identity. The stop command verifies that identity before sending a signal. Logs are stored in private `airust.log` files. These controls prevent a stale PID file from authorizing a signal to an unrelated process.

## Persistence

Training weights must be finite and nonnegative when a collection is loaded or saved. Saving rejects symlink and non-file destinations. It validates and serializes before replacing an existing destination.

A save uses an exclusively created private temporary file, flushes and synchronizes its content, then replaces the destination. Existing file permissions are retained. A rejected operation must not truncate the existing training file.

Atomic replacement preserves complete file content. It does not merge concurrent changes or provide database transaction isolation. Callers that need coordinated updates must serialize them or supply a separate storage layer.

The CLI process-control commands currently depend on Unix-style `ps` and `kill`. They were tested on macOS in this development pass. Windows process control has not been validated.

## Scope of assurance

Optional embedding providers are application-owned code. They may access a local model or external service; the caller controls credentials, deadlines, cancellation, and which text may leave its process. The checked batch boundary validates input, output count, finite nonzero vectors, dimensions, and model-space identity before committing index changes. It cannot prove a provider's claimed model identity or roll back external side effects.

The local E5 adapter reads explicitly supplied model and tokenizer files, fingerprints its recipe, and dynamically loads an explicitly supplied ONNX Runtime library. It disables automatic model-hub and runtime downloads. Model and native library files must be trusted. Semantic similarities are not truth or authorization decisions. Search scope must be supplied by the application; the core exposes no new web endpoint for embeddings.

The 0.2.0 audit found zero known vulnerabilities and retained optional-path maintenance warnings for `paste` through tokenizers and `ttf-parser` through PDF support. See [CHANGES-0.2.0.md](CHANGES-0.2.0.md) for the exact versions and advisory summary.

The changes have targeted regression cases for their boundaries. They are not a comprehensive independent production security audit. No arbitrary user content should be treated as executable instructions, and lexical results should not bypass the calling application's access rules.

Release validation must check the exact final source, all relevant feature combinations, minimum supported Rust, package contents, and deployment-specific configuration. Report untested environments explicitly rather than implying a complete certification.

## Optional reranking boundary

Reranking receives only the query and the selected authorized document texts. It applies Context before its candidate cap, validates all selected records/byte budgets before any inference and retains original evidence. Counts/bytes bound pipeline work, not provider allocations or model accuracy. An empty Context ID set remains unrestricted. Use authorized candidate retrieval and scope later graph/context expansion independently.

Provider-only original-order fallback is explicit. Malformed counts, non-finite scores, duplicate selected IDs and recipe changes fail; a configured score threshold cannot be bypassed by unscored fallback. Provider error text is application-owned and may require redaction at a service boundary. Query/document text is data, not permission to run instructions. A reranker score is not truth, authorization, human approval or a calibrated probability.

The local adapter verifies exact pinned assets before native loading and disables model/runtime downloads. The shared ONNX environment permits local E5 and reranker coexistence with the same canonical runtime-library path; it is process-wide, not an isolated native-code sandbox. Runtime files remain explicitly trusted. The [0.2.9 report](CHANGES-0.2.9.md) records tests, compatibility checks and dependency-audit limits.

## Advanced text inputs

The positional analyzer enforces original/normalized byte and token limits; query syntax, token/phrase counts and returned span counts are bounded separately. Normalization expansion keeps original UTF-8 evidence and can map multiple tokens to one grapheme. Invalid replacements are rejected before prior state changes. Scope checks and phrase constraints precede top-K; missing authorized IDs still require caller-side denial because empty Context ID sets are unrestricted. Stems and analyzed phrase matches do not establish semantic equivalence, authorship or factual truth. Common queries can still visit many documents; individual budgets are not an aggregate service-throughput guarantee.

Storage adapters are optional and own original records, not automatic search-index synchronization. See [the storage contract](STORAGE.md).

The optional compact Retrieval API reuses existing ranking, scopes results and bounds complete JSON. See [the retrieval contract](RETRIEVAL.md).

Relevant excerpts are explicitly enabled through RetrievalOptions, preserve original UTF-8 positions and impose separate analysis/highlight limits. See [the excerpt contract](EXCERPTS.md).

The optional snapshot coordinator publishes search, vectors and source bindings together. Acquire one immutable view per request; its revision is a consistency token, not authorization or an external database transaction. See [the snapshot contract](SNAPSHOTS.md).

Optional retrieval evidence assessment distinguishes policy success, insufficient evidence and uncertainty. It evaluates delivered text and explicit score gates, not source truth or calibrated answer probability. See [assessment contract](ASSESSMENT.md).

The optional execution feature adds shared cooperative cancellation. Supported controlled APIs return typed interruption errors and preserve the dataset publication boundary. Blocking providers/lock waits are not forcibly interrupted. See [execution contract](EXECUTION.md).

Execution controls also support monotonic cooperative deadlines, saturating remaining budgets and parent limits that cannot be extended by a derived control. A late provider response is discarded; blocked work is not forcibly stopped. See [time-budget contract](EXECUTION.md).

Optional execution observers expose actual work phases and counts. Notifications contain no query text, use best-effort delivery, isolate callback panics and run outside publication locks. Results/revisions remain authoritative. See [progress contract](EXECUTION.md).

Reproducible opt-in synthetic scale/concurrency workloads have explicit count guards and correctness assertions. They measure lexical RAM/source snapshots, not universal capacity or production security. See [stress methodology](STRESS.md).

The optional shared-index feature shares immutable record/posting/duplicate payloads across clones, detaches changed buffers and preserves old readers. Default storage and wire formats remain unchanged. See [sharing contract](SHARED-INDEX.md).

Incremental updates retain unchanged term/positional analysis, update metadata in each projection and refresh coordinates whenever original text bytes change. Diagnostic preparation receipts distinguish rebuilt/reused analysis. See [incremental contract](INCREMENTAL.md).

Optional adaptive routing selects only explicitly registered sources, freezes analyzer selection during metadata-only preflight and preserves actual recipes/scopes. Source failures never trigger hidden retries. Route traces are separate diagnostics. See [routing contract](ADAPTIVE.md).

## Optional explicit feedback learning (0.2.23)

a bounded local feedback ledger and inspectable smoothed ranking adjustments. Enable `feedback` explicitly. See [API and limits](FEEDBACK.md).

## Optional inspectable trainable ranking (0.2.24)

an optional small linear model with replaceable bounded features, explicit training, grouped holdout evaluation and versioned artifacts. Enable `learned-ranking` explicitly. See [API and limits](LEARNED_RANKING.md).

## Optional evidence-backed rule proposals (0.2.25)

an optional bounded rule interpreter that proposes conclusions with explicit premises, sources, revision, conflicts and incomplete-result status. Enable `reasoning` explicitly. See [API and limits](REASONING.md).

## Comparable quality gates (0.2.26)

comparisons of identified retrieval runs, visible per-query regressions and explicit quality/resource criteria. Enable `quality-gates` explicitly. See [contract](QUALITY-GATES.md) and [release report](CHANGES-0.2.26.md).

## Evidence freshness (0.2.27)

verification of captured proof dependencies against current facts, rules, source versions and allowed scope. Enable `evidence-freshness` explicitly. See [contract](EVIDENCE-FRESHNESS.md) and [release report](CHANGES-0.2.27.md).

## Exact ranking explanations (0.2.28)

per-document score explanations using the actual learned features and feedback adjustments. Enable `rank-explanations` explicitly. See [contract](RANK-EXPLANATIONS.md) and [release report](CHANGES-0.2.28.md).

## Controlled training pipeline (0.2.29)

trusted feedback curation, versioned datasets, grouped training and explicit quality-gated model promotion with rollback. Enable `training-pipeline` explicitly. See [contract](TRAINING-PIPELINE.md) and [release report](CHANGES-0.2.29.md).

## Optional disk search index (0.2.30)

a dedicated disk-backed full-text index with atomic revisions, scoped retrieval, bounded original-record export and integrity checks. Enable `disk-index` explicitly. See [contract](DISK-INDEX.md) and [release report](CHANGES-0.2.30.md).
