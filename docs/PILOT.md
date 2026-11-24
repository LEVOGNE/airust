# Local pilot and live search profiles

The reference example imports a chosen Markdown directory into an immutable local workspace. It returns ranked evidence with exact UTF-8 source coordinates through the CLI or five read-only MCP tools. It generates no answer. Case capture is explicit; normal query and serve commands do not log questions.

## Run the pilot

Build with Rust 1.89 or newer. `mcp` is sufficient for lexical search and all capture commands. `pilot` additionally compiles the explicit HTTP embedding profiles; the default library still has no HTTP, database or model dependency.

```bash
export CARGO_TARGET_DIR=/tmp/airust-pilot-build
cargo build --release --locked --features pilot --example reference_workspace
export AIRUST_PILOT_BIN="$CARGO_TARGET_DIR/release/examples/reference_workspace"
"$AIRUST_PILOT_BIN" pilot init /absolute/path/to/markdown /tmp/airust-local-pilot manuals
"$AIRUST_PILOT_BIN" query /tmp/airust-local-pilot/workspace "Which rules protect memory?"
```

Use a **new** destination. Initialization creates `workspace/`, `cases/`, `reviewed/` and `mcp-client.json`. Add the generated MCP server entry to the calling client's configuration; it records the absolute executable and workspace paths. The equivalent direct command is:

```bash
"$AIRUST_PILOT_BIN" serve /tmp/airust-local-pilot/workspace
```

The server exposes its complete loaded workspace to its connected client. It provides no account isolation. Imported source files are not changed. Hidden files are skipped; source symlinks are rejected. The import accepts 1–256 UTF-8 Markdown files, at most 4 MiB per file and 32 MiB total, with depth bounded to 32. Paragraphs are split at original UTF-8 boundaries into at most 1800-byte passages. The workspace is capped at 100000 passages, including very short paragraphs. Workspace JSON is limited to 64 MiB and retains version-one document/revision framing.

## Capture one search case

Write a request JSON with these exact fields. This runnable example is deliberately labeled as a synthetic example:

```json
{
  "case_id": "example-001",
  "query": "Which rules protect memory?",
  "language": "en",
  "origin": "synthetic_test",
  "origin_evidence": "Copied documentation example; not genuine user traffic."
}
```

```bash
"$AIRUST_PILOT_BIN" pilot capture /tmp/airust-local-pilot /tmp/request.json /tmp/airust-local-pilot/cases/example-001.json
"$AIRUST_PILOT_BIN" pilot check /tmp/airust-local-pilot /tmp/airust-local-pilot/cases/example-001.json
```

`case_id` accepts 1–64 ASCII letters/digits, `_` and `-`; the query is nonblank and at most 4096 UTF-8 bytes. Language tags contain 2–32 ASCII letters/hyphens. `origin_evidence` is a nonblank label of at most 2048 bytes without controls. Unknown fields are rejected. The request file is limited to 16 KiB.

Use `user_supplied` for a question actually submitted in the pilot, retaining a local provenance reference. `user_question_replay` means an older real question rerun now; it is not deployed search traffic. `assistant_authored`, `synthetic_test` and `unknown` remain separate. A caller can make false declarations; the file format does not authenticate a person or traffic source. Avoid putting unnecessary identifying information into case files.

The checksummed envelope contains a schema-one payload with capture time, implementation version, immutable request, workspace identity, active retrieval recipe, visible fallback/reranking details, elapsed `query_ms`, truncation and exact document/revision/byte-range quotes. `query_ms` measures profile initialization plus the bounded query/wrapper after workspace loading; it excludes process startup and workspace loading and is **not** a warm-request benchmark. Each case begins with `review: null`.

Source verification establishes exact provenance, not relevance or answerability. Nonempty candidates leave answerability unassessed; absence of hits means no evidence returned. Capture contains at most six excerpts, 2000 bytes each and 12000 bytes total. The encoded case limit is 128 KiB.

## Attach a review without replacing the capture

Reviews contain `reviewer`, `reviewer_kind` (`human`, `assistant`, `test`), `independence` (`independent`, `same_author`, `unknown`), `answerability` (`supported`, `unsupported`, `uncertain`), `relevant_sources` and `notes`. Only an actual independent human reviewer should declare `human` and `independent`; tooling cannot supply that review on their behalf.

A supported review must contain at least one relevant source. Copy the exact `document_id`, `revision`, `start_byte`, `end_byte` and `quote` from the captured evidence or a separately verified passage. Unsupported reviews contain no positive sources. Uncertain reviews stay ineligible for a frozen quality dataset. Maximums are 32 source ranges, 2000 bytes per quote and 4096 bytes of notes. Review JSON is limited to 128 KiB. For example, a synthetic negative check is:

```json
{
  "reviewer": "example-test",
  "reviewer_kind": "test",
  "independence": "same_author",
  "answerability": "unsupported",
  "relevant_sources": [],
  "notes": "Synthetic format example, not an independent human judgment."
}
```

```bash
"$AIRUST_PILOT_BIN" pilot review /tmp/airust-local-pilot /tmp/airust-local-pilot/cases/example-001.json /tmp/review.json /tmp/airust-local-pilot/reviewed/example-001.json
```

The new file preserves the request and source identity and includes the previous payload SHA-256. The original capture is retained. The hash establishes a payload link when both files are retained; it does not authenticate the reviewer or prevent someone constructing a different history. Re-review produces another new file. Select exactly one immutable revision per case for an export.

## Export and independently verify

```bash
"$AIRUST_PILOT_BIN" pilot export /tmp/airust-local-pilot /tmp/airust-local-pilot/reviewed /tmp/airust-pilot-export
python3 -B scripts/pilot_dataset.py /tmp/airust-pilot-export
```

An export contains the restorable workspace, sorted `cases.jsonl` and a manifest binding artifact hashes, byte sizes, workspace identity and declared origin/review counts. It accepts 1–256 regular non-symlink JSON case files, at most 32 MiB of JSONL. Duplicate IDs, modified text/revisions, split UTF-8, unknown schema fields or an incompatible workspace fail validation. The Python validator independently computes document framing, workspace identity, exact UTF-8 source projections and declared counts; it makes no model calls.

All semantic validation precedes output creation. Files use exclusive publication and existing destinations are never overwritten. An I/O failure can leave a partial **new** directory; `manifest.json`, written last, is the completion marker. Keep the original data and choose a fresh destination for recovery. These checks do not claim protection against concurrent hostile filesystem mutation or multi-file crash-atomic transactions.

An export always reports `evaluation_ready: false`. Counts are caller declarations. The validator reports the number of settled, declared user-supplied and independently human-reviewed cases, unique questions and unsupported cases. It does not certify that those declarations are true.

## Prepare a separately versioned evaluation

`scripts/pilot_dataset.py --freeze-plan PLAN --output NEW_DATASET EXPORT` validates the export before reserving a new output. It requires at least 50 unique eligible questions, unsupported cases and a complete plan assigning source families to development/evaluation. Each split needs at least ten questions and both supported/unsupported labels. A source document or family cannot appear across both splits.

The plan has `schema_version: 1`, `dataset: "reference-v2"` (or a later new version), `provenance_reviewed_by`, `assignments` keyed by eligible case ID with `family`/`split`, and `policy`. Policy fields are `ndcg_at_10_minimum` (at least 0.75), `hit_at_6_minimum` (at least 0.80), `optional_ndcg_gain_minimum`, `optional_hit_loss_maximum`, `lexical_warm_p95_ms`, `hybrid_warm_p95_ms`, `rerank_warm_p95_ms`, `worker_peak_rss_mib` and `embedding_service_peak_rss_mib`. Freeze budgets and judgments before running candidates. Recommendations need actual first-stage rankings, independent known-reference measurements, acceptable costs on both splits and retained individual losses.

The completion manifest binds the plan/cases/workspace and explicitly leaves `quality_gate_passed: false`: freezing inputs is not executing an evaluation. Human provenance review remains a prerequisite outside the tool. The frozen assistant-authored reference-v1 corpus/results are unchanged. This version does not manufacture the missing genuine independently reviewed pilot dataset.

## Explicit live profiles

Lexical defaults require no service. To select a profile, append `--profile /absolute/path/config.json` to `query`, `serve` or `pilot capture`. Config paths for vectors/model/runtime are resolved relative to the config file. Loading/config validation never downloads assets or starts services. The config is schema one and rejects unknown fields. A lexical config is:

```json
{"schema_version":1,"profile":"bm25","failure_policy":"error","embedding":null,"vectors":null,"reranker":null}
```

MCP may explicitly request `algorithm: "tfidf"` with this lexical profile; its recipe then says `airust-pilot-tfidf-v1`. Hybrid profiles require BM25 fusion and reject TF-IDF rather than ignoring it.

A hybrid config uses a model digest obtained from the **explicitly selected existing local service**, for example:

```json
{
  "schema_version": 1,
  "profile": "hybrid",
  "failure_policy": "error",
  "embedding": {
    "url": "http://127.0.0.1:11434",
    "model": "bge-m3:latest",
    "digest": "7907646426070047a77226ac3e684fbbe8410524f7b4a74d02837e43f2146bab",
    "dimensions": 1024,
    "timeout_ms": 2000
  },
  "vectors": "vectors.json",
  "reranker": null
}
```

The digest above identifies the retained reference model, not a promise about the model currently installed on another machine. Match it explicitly. Only plain HTTP loopback roots are accepted, without credentials/query/fragment; redirects and environment proxies are disabled. Source/query text is sent to that configured service. Model names alone are mutable; `/api/tags` digest checks bracket each `/api/embed` request. Raw original text is sent with `truncate: false`, and the recipe records normalization/dimensions. Dimension, zero/nonfinite values, count and byte bounds are checked.

```bash
"$AIRUST_PILOT_BIN" profile-build /tmp/airust-local-pilot/workspace /tmp/hybrid.json /tmp/vectors.json
"$AIRUST_PILOT_BIN" query /tmp/airust-local-pilot/workspace "Which rules protect memory?" --profile /tmp/hybrid.json
"$AIRUST_PILOT_BIN" serve /tmp/airust-local-pilot/workspace --profile /tmp/hybrid.json
```

The cache is tied to actual workspace passage IDs, source revisions and the model recipe. It is rebuildable derived data, at most 64 MiB encoded. Preparation also rejects more than 64 MiB of raw f32 vector values before embedding. Workspace, case and vector-envelope serialization stops when its byte budget is reached, including JSON escaping. A stale/corrupt/missing cache is a setup error. Preparation batches at most 16 texts/64 KiB with responses limited to 2 MiB. Service timeouts are 1–30000 ms; the active request's remaining budget can shorten them. Digest bracketing detects normal model replacement, not a malicious inference server impersonating an identity.

`hybrid` uses exact cosine plus BM25 with fixed RRF k=60 and equal weights, candidates capped at 100. `hybrid_rerank` additionally reranks the actual first 20 candidates (or the requested limit if larger). Build with `--features pilot,local-reranking` on Rust 1.92 or newer and set `reranker` to an object with `directory`, `runtime` and `threads` (1–64). Use the explicit asset setup in [LOCAL-RERANKING.md](LOCAL-RERANKING.md); the constructor verifies its pinned model files before loading the supplied native runtime. Native code is outside owned Rust's `forbid(unsafe_code)` and Miri coverage.

`failure_policy: "error"` returns provider/reranker failures. Opt-in `"lexical_fallback"` retries lexical retrieval only for inference-stage source/reranker errors and reports its distinct recipe and `fallback_reason`. Invalid setup and elapsed/cancelled requests never become successful fallbacks. Profile access is serialized per loaded model state; lock waiting observes cancellation/deadlines. HTTP/native work is cooperative and cannot be forcibly interrupted at an arbitrary instruction. CLI query work has a two-second budget after profile loading; MCP applies its request/server budgets and retains worker admission until the worker actually returns.

## Validate changes

`./test.sh` includes the Rust format/source-binding tests, real CLI/stdio pilot workflow, independent Python export checks and a synthetic loopback model protocol/failure suite. Ordinary tests download no model/runtime and claim no model quality. Miri and actual model-backed benchmarks have separate recorded scopes in the 0.6.0 validation evidence.
