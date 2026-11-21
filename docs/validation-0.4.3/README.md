# Reproduce AIRust 0.4.3 validation

Run `./test.sh` with Rust 1.92+ and Python 3.11+. Provider-neutral features, including MCP and the independent response tests, also pass Rust 1.89 checks. Use `cargo fmt --package airust`; the separate PDF backend remains exactly its 0.3.5 source tag.

## Focused checks

```bash
cargo test --locked --test core_optimization --test build_ranking
cargo test --locked --features execution,cache,shared-index --test core_optimization --test cache --lib
cargo test --locked --no-default-features --features mcp --test mcp --test mcp_cli --test controlled_matching
python3 -B scripts/check_mcp_dependencies.py
```

Core regressions check exact score bits against rebuilds, repeated updates/deletes, metadata-only reuse, divergent retained readers, parallel first queries, shared allocation identities, empty/BM25 paths, cancellation within a cold norm calculation and nonblocking/poisoned hint fallback. Protocol tests independently validate actual structured success/error replies and their matching JSON text copies. They retain the previous discovery, source-binding, cancellation, deadline, flood, frame, ID, output-budget and disconnect checks. CLI tests launch the packaged executable. These are regression tests, not an independent security audit.

Two additional internal regressions cover releasing an empty hash dictionary after a 20,000-word removal and shrinking a sparse 4,096-entry dictionary while retained readers stay correct. The empty-release regression fails on the first hash prototype and passes after the compaction fix.

The new `build_ranking` suite uses an independent exhaustive full scan for 840 BM25 parameter/scope/limit/query combinations, including extreme valid `k1`/`b`, repeated/unknown words and zero/large limits. It also checks first-batch boundaries, long Unicode tails, replacements/removals, serialization and exact tie/explanation order. Sorting/grouping is bounded to the first 1,024 borrowed tokens; the remaining stream is retained. The standard randomized vocabulary dictionary is not used to enumerate ranking or persisted records.

## Build an external program

Each `NAME.rs` / `NAME.toml` pair forms an independent Cargo executable. Copy the Rust file to `src/main.rs` and the manifest to `Cargo.toml` in a new directory. Use the exact registry dependency after publication. Before publication add a checkout `path` to the current `airust` dependency. Use `cargo +1.89.0 run --release -- ...` except where a separate server compiler is recorded.

`consumer` exercises broad existing APIs and MCP constructors/configuration/schemas. `corpus` preserves the existing legacy workload: full-file parsing, explicit working prefix, incremental/rebuild equivalence, concurrent signatures, retained views and cancellation/deadlines. The release runs a checked 1,000-entry prefix for one epoch. It is not a new million-entry stress claim.

Programs comparing versions intentionally use **different sources**: current `airust` is a checkout or independently checksum-verified extracted 0.4.3 package; `previous` is registry 0.4.2. Cargo cannot resolve two exact compatible patch versions from the same registry source in this arrangement. Do not edit the extracted runtime source.

## Exact core/cache comparison

Build `core.rs` / `core.toml`, then run:

```bash
cargo run --release -- compare /absolute/path/knowledge.json 10000 /absolute/path/queries.json /tmp/new-core-directory
```

The output directory must not exist. Queries are a nonempty JSON array of strings or objects with `query`. The program parses the whole input, indexes the requested prefix in both versions, compares ordered IDs, score bits and matched terms for BM25/TF-IDF, retains readers, applies 49 edits/deletes/inserts, compares the changed and old readers, then saves and cross-loads caches in both directions. It compares cache analysis/metadata after the expected version-specific producer label. Whole-file byte equality would be wrong because that label and checksum change. Keep cache files private: they contain source data.

The release uses prefixes 1,000 / 10,000 / all 167,942 and the same 70 fixed literal probes. This gives 840 query comparisons per prefix. Smaller prefixes may omit the intended source documents. The complete comparison covers the lexical results each prefix actually returns; all recorded quiet 10,000/full-corpus samples have positive hit counts. Dedicated test cases cover empty results, and separate matching/profile probes require hits.

For separate ordinary-call timings use modes `previous` and `current`. The build timer starts after full input parsing and owned-record preparation. Three fixed query passes include one first pass and two repeated passes; the first pass is **partly warmed across overlapping candidate sets**, not an independent cold query for every sample. Each result contains count/hash, build time, per-query hits and timings. Use `core_timing.py` to run one child per fresh wrapper and add lifetime child RSS. RSS includes full input decoding before prefix selection and is not isolated index heap. Alternate old/new process order and retain all samples. Avoid competing builds during measurement.

## Allocator-instrumented index heap

Build `heap.rs` / `heap.toml` as a separate program. DHAT is **external-only**; it is not an AIRust normal or dev dependency.

```bash
cargo run --release -- previous /absolute/path/knowledge.json 10000 /tmp/new-old.dhat.json /tmp/new-old-summary.json
cargo run --release -- current /absolute/path/knowledge.json 10000 /tmp/new-new.dhat.json /tmp/new-new-summary.json
```

The program preloads/truncates the source outside profiling, then clones the owned records under the profiler before indexing. It records build, warmed, clone, drop-clone and drop-all phases. It derives ten non-stopword positive queries from real entries and asserts that every query returns hits. Every tracked index allocation must be released at the end in the release campaign. Preloaded data, profiler overhead, OS memory and other application processes are excluded. The default index is measured without `shared-index`; its default clones copy containers but share immutable strings. Instrumentation changes execution costs, so its timings are **not** latency benchmarks. The release records separate 1,000/10,000-entry traces for both versions. DHAT traces are retained privately because allocation stacks/source context may be sensitive.

## Real MCP results and schema conformance

Build/install `airust-mcp` from the same crate. Build `oracle.rs` / `oracle.toml` as a direct Rust API oracle. It accepts `knowledge.json queries.json new-expected.json` and writes complete expected retrieval objects using MCP's default search settings. Snapshot mode must use the exact page entries derived from that snapshot. Expected results contain source text; protect them.

```bash
cargo install airust --version 0.4.3 --locked --no-default-features --features mcp --bin airust-mcp
python3 mcp_probe.py --binary /absolute/path/airust-mcp --document /absolute/path/snapshot.json --queries /absolute/path/queries.json --expected /absolute/path/expected.json --report /tmp/new-mcp-report.json --transcript /tmp/new-private-transcript.json
```

Use `--knowledge` instead of `--document` for legacy knowledge. The probe checks both algorithms, complete direct-API JSON equality, source bindings, exact snapshot byte spans and passage readbacks, monotonic progress/token sequences, four concurrent requests, denied unknown arguments, read-only status and unchanged input hash. Report files are source-free aggregates. With `--transcript`, it also calls matching/relations and captures discovery plus actual structured results privately. A transcript **contains document text**; never commit it with public measurements.

Build `schema.rs` / `schema.toml` as a separate executable. It depends on `jsonschema` with default resolver features disabled and has **no AIRust dependency**. Run it with `private-transcript.json new-schema-report.json`. It validates all advertised schemas against their metaschema, then validates every captured structured reply with the advertised per-tool schema, checks text-copy equality and `isError` agreement. No remote reference resolution is enabled. Unit tests separately reject missing/wrong nested fields and cross-tool/schema mismatches.

The release repeats this on 35 retained snapshots and the complete canonical collection. Quiet timing runs omit transcripts/extra tools and use fresh old/current server processes, alternating order over three repeats. Search timers include local request/progress/result transport and exclude passage follow-up calls. Startup includes spawn, JSON restoration, index build and protocol initialization. The single server child's lifetime RSS excludes the Python validator's expected text. Warm files and uncontrolled desktop activity limit generalization; no p95/p99 or hard capacity claim is made.

## Matching and retention

`matching.rs` / `matching.toml` compare complete serialized ordinary and controlled matching results against 0.4.2 and compare shared-term relations. Use `knowledge.json 10000 new-output.json`. Exact/token probes use twelve whole entries; fuzzy probes edit available 24..128-character entries. Every matching case must return hits. The 1,000-entry prefix has one qualifying short source reused with different edits; 10,000 has twelve. Paired timing excludes controlled checks and serialization. Retained knowledge/source hashes are rechecked separately.

All 35 originals, eight canonical exports and 35 existing 0.4.0 snapshots remain unchanged. Fresh tests reuse that knowledge; no PDF is re-extracted here. Previous parser/native-model/large-stress evidence is inherited only for unchanged modules. There is no DenkMaps integration, live third-party client UI check, Docker execution or independent penetration audit. See [results](../CHANGES-0.4.3.md), [core boundaries](../CORE-PERFORMANCE.md), [MCP API](../MCP.md) and [security](../MCP-SECURITY.md).
