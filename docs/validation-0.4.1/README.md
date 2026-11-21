# Reproduce AIRust 0.4.1 validation

Use `./test.sh` with Rust 1.92+ and Python 3.11+. Provider-neutral builds, including MCP, support Rust 1.89. Formatting is scoped to `cargo fmt --package airust`; the PDF backend remains byte-identical to its 0.3.5 source tag.

The focused suite is:

```bash
cargo test --locked --no-default-features --features mcp --test mcp --test mcp_cli --test controlled_matching
cargo test --locked --no-default-features --features execution --lib matching::bounded_tests
python3 -B scripts/check_mcp_dependencies.py
```

Raw wire tests cover legacy and modern discovery, all tools, exact UTF-8 ranges/revisions, argument limits and denied fields, source injection markers, explicit missing bindings, progress sequences, live cancellation, absolute deadlines, busy slots, duplicate progress tokens, disconnected/stalled clients and transport flood/ID/frame limits. CLI tests launch the actual executable, verify JSON-only stdout, reject bad options/oversized startup files and preserve input bytes. The controlled-distance oracle enumerates short multilingual words against independent `strsim` distances; another case cancels inside a long edit-distance row.

## Public API consumer

Copy `consumer.toml` to `Cargo.toml` and `consumer.rs` to `src/main.rs` of a new package. Run using Rust 1.89. Before release use a checkout path override; after release use the exact registry dependency. This retains the previous broad API consumer and adds the public MCP constructors, limits, schemas, shared clone and controlled matching APIs. `corpus.toml` / `corpus.rs` exercise the retained legacy workload on a supplied knowledge JSON file; the release parses all 167,942 records and works on an explicit 1,000-entry prefix for one checked epoch.

## Actual stdio validation with real knowledge

Install/build the binary from the same crate, then create a query JSON file such as `["memory ownership", "error handling"]`. These are literal probes, not human relevance labels.

```bash
cargo install airust --version 0.4.1 --locked --no-default-features --features mcp --bin airust-mcp
python3 mcp_probe.py --binary /absolute/path/airust-mcp --knowledge /absolute/path/knowledge.json --queries /absolute/path/queries.json --report /tmp/new-mcp-result.json
```

Use `--document /absolute/path/snapshot.json` instead to verify every returned excerpt against global snapshot UTF-8 bytes and read it back through `airust_read_passage`. The validator checks tool annotations, both lexical algorithms, progress monotonicity/tokens, read-only status, a denied path argument and unchanged source-file hash. It writes source-free timing/count aggregates to a **new** report file. It does not print queries or source text on success. Preserve private failure logs securely.

For a separate direct-API oracle, create another package with `oracle.toml` / `oracle.rs`. Run its binary with `knowledge.json queries.json new-expected.json`, then pass `--expected new-expected.json` to the Python validator. For snapshot mode, supply the exact page entries created from that snapshot. The probe requires complete retrieval JSON equality, including ranking evidence, spans, excerpts, assessment and budgets; transport progress is checked separately. The oracle's settings intentionally match default MCP search settings. Keep expected content private if it includes private text.

Search timing includes a warm stdio request, computation, progress and result transport. It excludes later exact passage readback calls. Startup-to-initialized includes spawn, JSON restoration, index build and lifecycle exchange. OS buffers, file caches and desktop activity affect these measurements. A few warm samples do not establish tail latency or hard CPU/RAM guarantees. This feature performs no native PDF parsing during these runs.

## Affected core comparison

Use `matching.toml` / `matching.rs` with current `airust` as a checkout or verified extracted-package path and `previous` as registry 0.4.0. Cargo cannot resolve two different exact compatible patch versions from the same registry, so the current path source is intentional. After publication use the independently checksum-verified extracted 0.4.1 crate, without editing its runtime source.

```bash
cargo run --release -- /absolute/path/knowledge.json 10000 /tmp/new-matching-result.json
```

It parses the whole supplied collection, indexes the specified prefix in both engines, compares complete serialized matching results for 12 real entry-derived probes per strategy across three strategies/two thresholds, compares controlled/current results and checks 12 relation cases. Both engines enable `execution`; paired timers cover ordinary calls only; oracle/serialization work is outside the timed sections. Repeat in fresh release processes with no deliberate competing builds, alternate order and retain every sample. Exact/token probes use full source entries; fuzzy probes edit available 24..128-character entries so comparison remains bounded and still has positive matches. The program asserts every matching case returns a hit and reports the number of short fuzzy source entries (the 1,000-entry prefix has one, reused with twelve distinct edits). Negative/Unicode boundary cases are covered by the separate test suite. The default APIs do not gain cancellation overhead intentionally; compare measurements before making a speed claim.

## Evidence and limits

The existing 35 PDF sources, eight canonical exports and 0.4.0 revisioned snapshots are retained unchanged. Fresh MCP checks reuse them; unchanged PDF parser, glyph/reference and native-model campaigns remain inherited evidence. No DenkMaps adapter or live application database is tested. A local adversarial regression campaign is not an independent security audit. See [executed release results](../CHANGES-0.4.1.md), [API and limits](../MCP.md) and [security responsibilities](../MCP-SECURITY.md).

The Python validator also records the single MCP child process's lifetime peak RSS using `getrusage(RUSAGE_CHILDREN)`, normalized from macOS bytes/Linux KiB. Each validator invocation launches one server child, so this includes the server's startup/index/snapshot/runtime and transport working memory, excludes the Python validator's separately loaded expected text and does not represent all processes in a client application. It is an observed peak, not a configured memory quota.

Each probe finally submits four searches concurrently on the same connection. It checks interleaved progress tokens/sequences, successful responses for the correct IDs and complete retrieval equality against the separate oracle when provided. These four calls have their own elapsed/count fields and are not mixed into sequential latency samples. This is a bounded four-request behavior check, not a broad concurrency capacity benchmark.
