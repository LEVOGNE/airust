# Start AIRust

AIRust is a Rust library with optional tools. Version 0.2.4 adds optional source references alongside lexical, semantic, hybrid, graph and context queries. Use Rust 1.89 or newer for the core, provider interface, and hybrid search; the optional local ONNX adapter needs Rust 1.92. Run these commands from the source checkout.

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

```bash
cargo run --locked --features cli,pdf --bin pdf2kb -- data/pdf/sample.pdf /tmp/airust-sample.json
```

The output is a training file. Existing output files require the explicit `--overwrite` flag. Unknown flags, invalid chunk settings, and using the same file as input and output are rejected. Use `--` before a filename that begins with a dash. Each passage includes its source filename, page number, and chunk position. Image-only PDFs require separate optical character recognition; AIRust has no OCR engine.

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
docker build -t airust:0.2.4 .
export AIRUST_API_TOKEN="$(openssl rand -hex 32)"
docker run --rm \
  -e AIRUST_API_TOKEN \
  -p 127.0.0.1:7070:7070 \
  -p 127.0.0.1:7071:7071 \
  airust:0.2.4
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
