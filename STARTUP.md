# Start AIRust

AIRust is a Rust library with optional tools. Version 0.5.1 keeps the 0.5.0 runtime and records a fixed-seed fuzz check of search, retrieval and bounded PDF import. Existing runtime calls and knowledge remain. Select Cargo 0.5 explicitly, or pin `=0.5.1`. Read [upgrade](docs/MIGRATION-0.5.1.md), [API cookbook](docs/API-COOKBOOK.md), [MCP](docs/MCP.md) and [fuzz scope](docs/CHANGES-0.5.1.md). Provider-neutral features use Rust 1.89+; optional local ONNX adapters need Rust 1.92. No DenkMaps integration is included.

## Keep build storage under control

For repeated release checks, choose one temporary build directory instead of accumulating targets for every experiment:

```bash
export CARGO_TARGET_DIR=/absolute/path/to/temporary/airust-build
export CARGO_PROFILE_DEV_DEBUG=0
export CARGO_PROFILE_TEST_DEBUG=0
export CARGO_INCREMENTAL=0
```

The directory must be dedicated to regenerable Cargo output. Keep PDFs, knowledge, source snapshots and measurement data elsewhere. `cargo clean` with the same configured target removes compiled output; the next build compiles it again. Archive release packages, exact locks and test receipts first. The 0.5.0 campaign follows this policy and removes its two named temporary build targets after verifying publication. Normal application consumers can enable only the features they need.

## Try the core

```bash
cargo run --locked --example core
```

The example demonstrates retrieval and comparison without changing existing knowledge files or databases.

## Use the existing agents

```bash
cargo run --locked --features cli --bin airust -- --version
cargo run --locked --features cli --bin airust -- cli query simple "What is AIRust?"
cargo run --locked --features cli --bin airust -- cli query fuzzy "What is AIRust?"
cargo run --locked --features cli --bin airust -- cli query tfidf "What is AIRust?"
cargo run --locked --features cli --bin airust -- cli interactive
```

`simple` compares exact text, `fuzzy` tolerates spelling changes, and the historical `tfidf` agent uses BM25. These adapters return stored answers.

`cli` enables sample knowledge from `knowledge/train.json`. Cargo rebuilds the embedded data when that file changes. The default library embeds no sample knowledge.

## Import a PDF

Version 0.4.0 retains the 0.3.15 import API and flags; full page-tree validation rejects incorrect root/subtree counts before extraction. Version 0.3.4 introduced the parser optimization. Its optional backend optimizes long-line object loading without rewriting source PDFs or dropping structure. See [measured results and limits](docs/PDF-PARSER.md).

```bash
cargo run --locked --features cli,pdf --bin pdf2kb -- data/pdf/sample.pdf /tmp/airust-sample.json --timeout-seconds 60
```

The output is a training file. `--timeout-seconds 60` supervises conversion in an owned child and enables bounded import defaults. The parent validates complete results before publication; `--max-result-mib` controls accepted JSON size. See [worker commands and limits](docs/PDF-WORKERS.md). Version 0.3.1 rejects empty conversions and protects destinations created while conversion is running. See the [PDF stability and retained-knowledge guide](docs/PDF-STABILITY.md). Existing output files require the explicit `--overwrite` flag. Unknown flags, invalid chunk settings, and using the same file as input and output are rejected. Use `--` before a filename that begins with a dash. Use `--max-file-mib 64 --max-pages 3000 --max-text-mib 64 --max-chunks 50000` for explicit limits. Selecting any of these flags enables the other bounded defaults; these are not hard CPU or RAM quotas. See the [limit contract](docs/PDF-LIMITS.md). Each passage includes its source filename, page number, and chunk position. Image-only PDFs require separate optical character recognition; AIRust has no OCR engine.

To see the available options:

```bash
cargo run --locked --features cli,pdf --bin pdf2kb -- --help
```

The maximum chunk length must be positive. The preferred minimum must not exceed the maximum. Overlap must be smaller than the maximum. A final short chunk is retained.

## Merge training files

```bash
cargo run --locked --features cli --bin merge_kb -- source-a.json source-b.json --output /tmp/airust-merged.json
```

The tool validates all source files before replacing the output. Completely identical examples are included once. Examples with different answers, weights, or metadata remain distinct.

With no source paths, it reads `knowledge/*.json` and writes `knowledge/train.json`. It excludes the output from the input list. No sources or an invalid source leaves an existing output untouched. `--include-embedded` explicitly includes embedded samples.

This tool reads the existing `input`/`output` training format. Generic `id`/`text` knowledge entries are loaded by `Engine::from_json()`.

## Start the dashboard

```bash
cargo run --locked --features web --bin airust
```

Open `http://127.0.0.1:7070`. The console endpoint is `ws://127.0.0.1:7071/ws/console`.

Use another port:

```bash
cargo run --locked --features web --bin airust -- --port 8080
```

The console then uses port 8081. Valid main ports range from 1 through 65534.

Start and stop a background process:

```bash
cargo run --locked --features web --bin airust -- -d
cargo run --locked --features web --bin airust -- stop
```

Show the product overview with the dashboard:

```bash
cargo run --locked --features web --bin airust -- --landingpage
```

`--help` lists supported commands. The console provides `help`, `clear`, `status`, `restart`, and `reset`. `reset` removes the dashboard database. Use the documented controls only when you intend their effect on local data.

Runtime state uses `AIRUST_RUNTIME_DIR`, an absolute private directory that defaults to `$HOME/.local/state/airust`. Its Unix mode is `0700`; insecure or symlink directories are rejected. The directory contains private `server.json` process identity and `airust.log` files. `stop` checks the recorded process identity before signaling.

The dashboard stores its database as `airust.db` in the current working directory. Keep a backup before changing an existing installation. Repository tests use separate temporary databases.

The default bind address is `127.0.0.1`. `AIRUST_BIND_ADDR` selects another IP address. Non-loopback operation requires `AIRUST_API_TOKEN`; a configured token also protects a loopback instance. The value needs 32 through 256 ASCII letters, digits, hyphens, or underscores.

For a token-protected local instance, generate and export a token before starting:

```bash
export AIRUST_API_TOKEN="$(openssl rand -hex 32)"
cargo run --locked --features web --bin airust
```

The browser prompts for that token and retains it only in page memory. An HTTP client supplies it in a header:

```bash
curl -H "Authorization: Bearer $AIRUST_API_TOKEN" http://127.0.0.1:7070/api/status
```

`AIRUST_ALLOWED_HOSTS` accepts explicit comma-separated hostnames without ports or wildcards. Host and Origin must match the expected host and service ports. A reverse proxy must be configured for these checks deliberately.

Administration is disabled by default. `AIRUST_ENABLE_ADMIN=1` enables file administration, knowledge load/save, and console reset. Enabled filesystem access remains confined to the working directory and rejects hidden, parent, absolute, and symlink paths. Read the [security guide](docs/SECURITY.md) before enabling it.

Ordinary queries do not change settings through keyword detection. Use the explicit `/settings` command prefix or the `/api/settings` endpoint.

## Use Docker

```bash
docker build -t airust:0.4.5 .
export AIRUST_API_TOKEN="$(openssl rand -hex 32)"
docker run --rm \
  -e AIRUST_API_TOKEN \
  -p 127.0.0.1:7070:7070 \
  -p 127.0.0.1:7071:7071 \
  airust:0.4.5
```

The image explicitly builds the web feature. Its non-loopback container listener requires the supplied token. These port mappings keep access local to the host. The build excludes the existing database, backup archive, and local logs. It includes this guide because the web server embeds it.

A database created inside a container lasts only as long as that container unless you provide persistent storage. Building the image requires access to the package registry.

## Run validation

```bash
./test.sh
cargo run --locked --release --example benchmark -- 1000 100
```

For a configurable index workload:

```bash
cargo run --locked --release --example index_benchmark -- 10000 300
```

This reports JSON with build, search percentiles, update, and delete timings. The entry and query counts are explicit. Larger benchmark methodology and evidence requirements are in [ROADMAP.md](ROADMAP.md).

The older benchmark measures a small fixed workload. Results depend on the host. Historical results remain in their versioned reports. Optional semantic evaluation and exact-vector measurements are recorded in [CHANGES-0.2.0.md](docs/CHANGES-0.2.0.md). Run the model-independent example with `cargo run --locked --features embeddings --example semantic`; the [local-model guide](docs/LOCAL-EMBEDDINGS.md) describes explicit native inference.

For optional derived-index startup and recovery, see [CACHE.md](docs/CACHE.md).

Approximate retrieval is explicitly opt-in; see [ANN.md](docs/ANN.md).

Optional conflict-checked revisions and history: [VERSIONING.md](docs/VERSIONING.md).

For the complete modular API, read [API-COOKBOOK.md](docs/API-COOKBOOK.md). Reproducible measurements: [BENCHMARKS.md](docs/BENCHMARKS.md).

## Optional intelligent reranking

Reranking is a library operation, not a new dashboard or MCP endpoint. Run the model-independent API example with `cargo run --locked --features reranking --example reranking`. It uses a handwritten demonstration scorer. For actual model inference and explicit pinned-asset download commands, follow [LOCAL-RERANKING.md](docs/LOCAL-RERANKING.md). The complete [API cookbook](docs/API-COOKBOOK.md) explains bounded candidate selection and composition with context. No existing application is upgraded automatically.

## Optional phrase and word-form search

Run `cargo run --locked --features advanced-search --example advanced_search`. The [text-search guide](docs/TEXT-SEARCH.md) documents required quoted phrases, German stemming, Unicode tokenization and budgets. This is a library-only feature and changes no existing dashboard or application search automatically.

Storage adapters are optional and own original records, not automatic search-index synchronization. See [the storage contract](docs/STORAGE.md).

The optional compact Retrieval API reuses existing ranking, scopes results and bounds complete JSON. See [the retrieval contract](docs/RETRIEVAL.md).

Relevant excerpts are explicitly enabled through RetrievalOptions, preserve original UTF-8 positions and impose separate analysis/highlight limits. See [the excerpt contract](docs/EXCERPTS.md).

The optional snapshot coordinator publishes search, vectors and source bindings together. Acquire one immutable view per request; its revision is a consistency token, not authorization or an external database transaction. See [the snapshot contract](docs/SNAPSHOTS.md).

Optional retrieval evidence assessment distinguishes policy success, insufficient evidence and uncertainty. It evaluates delivered text and explicit score gates, not source truth or calibrated answer probability. See [assessment contract](docs/ASSESSMENT.md).

The optional execution feature adds shared cooperative cancellation. Supported controlled APIs return typed interruption errors and preserve the dataset publication boundary. Blocking providers/lock waits are not forcibly interrupted. See [execution contract](docs/EXECUTION.md).

Execution controls also support monotonic cooperative deadlines, saturating remaining budgets and parent limits that cannot be extended by a derived control. A late provider response is discarded; blocked work is not forcibly stopped. See [time-budget contract](docs/EXECUTION.md).

Optional execution observers expose actual work phases and counts. Notifications contain no query text, use best-effort delivery, isolate callback panics and run outside publication locks. Results/revisions remain authoritative. See [progress contract](docs/EXECUTION.md).

Reproducible opt-in synthetic scale/concurrency workloads have explicit count guards and correctness assertions. They measure lexical RAM/source snapshots, not universal capacity or production security. See [stress methodology](docs/STRESS.md).

The optional shared-index feature shares immutable record/posting/duplicate payloads across clones, detaches changed buffers and preserves old readers. Default storage and wire formats remain unchanged. See [sharing contract](docs/SHARED-INDEX.md).

Incremental updates retain unchanged term/positional analysis, update metadata in each projection and refresh coordinates whenever original text bytes change. Diagnostic preparation receipts distinguish rebuilt/reused analysis. See [incremental contract](docs/INCREMENTAL.md).

Optional adaptive routing selects only explicitly registered sources, freezes analyzer selection during metadata-only preflight and preserves actual recipes/scopes. Source failures never trigger hidden retries. Route traces are separate diagnostics. See [routing contract](docs/ADAPTIVE.md).

## Optional explicit feedback learning (0.2.23)

a bounded local feedback ledger and inspectable smoothed ranking adjustments. Enable `feedback` explicitly. See [API and limits](docs/FEEDBACK.md).

## Optional inspectable trainable ranking (0.2.24)

an optional small linear model with replaceable bounded features, explicit training, grouped holdout evaluation and versioned artifacts. Enable `learned-ranking` explicitly. See [API and limits](docs/LEARNED_RANKING.md).

## Optional evidence-backed rule proposals (0.2.25)

an optional bounded rule interpreter that proposes conclusions with explicit premises, sources, revision, conflicts and incomplete-result status. Enable `reasoning` explicitly. See [API and limits](docs/REASONING.md).


For current pagewise PDF limits, difficult-file checks and retained knowledge, see [the 0.3.5 import guide](docs/PDF-IMPORT-PIPELINE.md). The same command-line flags and library methods apply.


See [the 0.3.6 cleaning controls and migration notes](docs/PDF-TEXT-FIDELITY.md) before rebuilding PDF-derived indexes. Older knowledge exports remain valid.

The [current memory/API guide](docs/PDF-PAGE-TREE-MEMORY.md) explains the implementation change, reproducible checks and remaining limits. Existing knowledge files require no migration.

## Keep exact document positions

Run `cargo run --locked --features documents --example documents`. Supply a stable document ID, source fingerprint, processing recipe and exact text. Keep the immutable snapshot with its passages. Read [the document guide](docs/DOCUMENTS.md) before mapping search snippets: offsets belong to the snapshot, and stale revisions must fail. Existing knowledge is never automatically rewritten or deleted.

## Local read-only MCP

```bash
cargo run --locked --no-default-features --features mcp --bin airust-mcp -- --knowledge /absolute/path/knowledge.json
```

Use one or more `--document /absolute/path/snapshot.json` options instead for revision-bound page entries. Standard output is reserved for MCP JSON-RPC; diagnostic counts use standard error. Configure the client with a direct executable and argument list. The [MCP guide](docs/MCP.md) explains installation, all five tools, progress display, cancellation and resource defaults. Read the [security model](docs/MCP-SECURITY.md): the connected client can read the entire supplied collection, and source text must remain untrusted data.
