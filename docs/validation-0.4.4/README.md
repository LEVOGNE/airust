# Reproduce AIRust 0.4.4 validation

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

Programs comparing versions intentionally use **different sources**: current `airust` is a checkout or independently checksum-verified extracted 0.4.4 package; `previous` is registry 0.4.3. Cargo cannot resolve two exact compatible patch versions from the same registry source in this arrangement. Do not edit the extracted runtime source.

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

For authored edge distributions, run `python3 heap_fixtures.py /tmp/new-heap-fixtures`. This creates eight source-free collections of 64 entries: short words, a long single word, two long words, Unicode, repeated words and unique prefixes/tails. Pass each JSON file to the same heap consumer with count `64`, once per version and with fresh output names. These profiles deliberately check over-reservation as well as allocation savings. Few long words can increase cumulative allocated bytes while reducing allocation calls. Completed live index counts must remain equal and every tracked allocation must be released. The fixture generator refuses an existing output directory.

## Real MCP results and schema conformance

Build/install `airust-mcp` from the same crate. Build `oracle.rs` / `oracle.toml` as a direct Rust API oracle. It accepts `knowledge.json queries.json new-expected.json` and writes complete expected retrieval objects using MCP's default search settings. Snapshot mode must use the exact page entries derived from that snapshot. Expected results contain source text; protect them.

```bash
cargo install airust --version 0.4.4 --locked --no-default-features --features mcp --bin airust-mcp
python3 mcp_probe.py --binary /absolute/path/airust-mcp --document /absolute/path/snapshot.json --queries /absolute/path/queries.json --expected /absolute/path/expected.json --report /tmp/new-mcp-report.json --transcript /tmp/new-private-transcript.json
```

Use `--knowledge` instead of `--document` for legacy knowledge. The probe checks both algorithms, complete direct-API JSON equality, source bindings, exact snapshot byte spans and passage readbacks, monotonic progress/token sequences, four concurrent requests, denied unknown arguments, read-only status and unchanged input hash. Report files are source-free aggregates. With `--transcript`, it also calls matching/relations and captures discovery plus actual structured results privately. A transcript **contains document text**; never commit it with public measurements.

Build `schema.rs` / `schema.toml` as a separate executable. It depends on `jsonschema` with default resolver features disabled and has **no AIRust dependency**. Run it with `private-transcript.json new-schema-report.json`. It validates all advertised schemas against their metaschema, then validates every captured structured reply with the advertised per-tool schema, checks text-copy equality and `isError` agreement. No remote reference resolution is enabled. Unit tests separately reject missing/wrong nested fields and cross-tool/schema mismatches.

The release repeats this on 35 retained snapshots and the complete canonical collection. Quiet timing runs omit transcripts/extra tools and use fresh old/current server processes, alternating order over three repeats. Search timers include local request/progress/result transport and exclude passage follow-up calls. Startup includes spawn, JSON restoration, index build and protocol initialization. The single server child's lifetime RSS excludes the Python validator's expected text. Warm files and uncontrolled desktop activity limit generalization; no p95/p99 or hard capacity claim is made.

An initial canonical process is a latency outlier. The report retains it and adds one fixed follow-up of three pairs, using the same probe, binaries, input and queries with fresh output names. The order is current/previous, previous/current, current/previous. Process medians are summarized separately from pooled heterogeneous query times. This is a bounded investigation, with all original and follow-up samples retained.

## Matching and retention

`matching.rs` / `matching.toml` compare complete serialized ordinary and controlled matching results against 0.4.3 and compare shared-term relations. Use `knowledge.json 10000 new-output.json`. Exact/token probes use twelve whole entries; fuzzy probes edit available 24..128-character entries. Every matching case must return hits. The 1,000-entry prefix has one qualifying short source reused with different edits; 10,000 has twelve. Paired timing excludes controlled checks and serialization. Retained knowledge/source hashes are rechecked separately.

All 35 originals, eight canonical exports and 35 existing 0.4.0 snapshots remain unchanged. Fresh tests reuse that knowledge; no PDF is re-extracted here. Previous parser/native-model/large-stress evidence is inherited only for unchanged modules. There is no DenkMaps integration, live third-party client UI check, Docker execution or independent penetration audit. See [results](../CHANGES-0.4.4.md), [core boundaries](../CORE-PERFORMANCE.md), [MCP API](../MCP.md) and [security](../MCP-SECURITY.md).

## Extended differential stress and Miri

Build `soak.rs` with `soak.toml`. The optional `shared` feature enables shared blocks in both compared versions. Run with `SOURCE_JSON QUERIES_JSON PREFIX SEED SECONDS NEW_OUTPUT_DIR`. Prefix is limited to 1..100,000 records and duration to 1..28,800 seconds. Each epoch performs 128 seeded mutations, exact comparisons with registry 0.4.3 and a rebuilt index, retained-reader checks, four concurrent readers, and cancelled/expired calls. Periodic bidirectional cache loading and truncated-cache recovery verify ownership preservation on error. The release uses four seeds, two prefix sizes and both sharing modes.

The output folder contains caches and a final mutated **test copy** of the selected source entries. These files contain private text. Use a protected directory and retain the originals separately. `progress.json` and `summary.json` contain counts and timing only. The harness does not modify its input collection. Long stress runs compete for CPU and do not supply the quiet latency baseline.

The original long-run binaries copied their last progress timestamp into the final summary. A progress-file write can make that sample slightly earlier than the stop-condition check; one retained row is 1.1 milliseconds below its 90-minute target. Raw measurements remain unchanged. The reproduction source now records loop-completion time before saving the final index. External case spans include setup and final saving and are reported separately; correctness counters and library code are unchanged.

Miri runs target the bounded token batch, shared readers, core APIs, norm ownership/concurrency and the independent BM25 reference tests. Run `cargo +nightly miri setup`, then for example `MIRIFLAGS=-Zmiri-no-extra-rounding-error cargo +nightly miri test --no-default-features --features shared-index --test core -- --test-threads=1`. The flag prevents extra interpreter-injected rounding variation from invalidating exact floating-point comparisons; it does not disable memory, aliasing or race checks. Raw default-float attempts are retained separately. Native PDF/ONNX code and the asynchronous MCP transport are outside this interpreter campaign. A passing Miri run covers the executed paths and schedules, not every possible program behavior. See the [official Miri explanation](https://github.com/rust-lang/miri#readme).

The MCP suite also reproduces cross-connection shutdown: a connection must finish while another clone owns the worker permit. Its companion wire test closes one connection during another's real fuzzy calculation, then cancels that surviving work. A batch-array probe expects an invalid-request error and successful subsequent individual request. These checks supplement the general schema validator; schema shape alone cannot establish connection isolation or worker cleanup.

## MCP connection lifetimes and allocation traffic

Build `mcp_churn.rs` with `mcp_churn.toml`, then run `NEW_OUTPUT_DIR [PARALLEL=4]`. Choose 1 or 4 concurrent connections. The program reuses one immutable 1,000-record synthetic server across 1,000 connections. Each connection initializes, searches with progress, requests status and closes. Every batch checks that worker slots have been released. Numeric telemetry storage is allocated before profiling, and report JSON is built afterward. This avoids attributing the probe's own result buffers to retained sessions.

The published comparison uses identical **serial** runs against registry 0.4.3 and 0.4.4; change only the AIRust pin when reproducing the baseline. A separate four-connection run checks robustness. The old version can return a false `busy` while other connections close; failed partial runs are retained and excluded from allocation comparisons. Source-free summaries are public. Raw DHAT stacks remain private. The profile excludes the preloaded index, runtime and fixed tool catalogue; the new catalogue retains five definitions per shared data instance. Cumulative counters, live snapshots and process RSS are different metrics. The growth guard and finite session count do not prove absence of every leak.
