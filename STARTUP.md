# Start AIRust

Choose the task you need below. This checkout prepares the AIRust 0.6.0 source milestone with a bounded local pilot and explicit live profiles. Registry installation examples below use the last verified published baseline, 0.5.3; the source version is not a publication receipt. Use Rust 1.89 for provider-neutral code, Rust 1.92 for local ONNX adapters, and Python 3.11 or newer for the repository checks.

## Search records

```bash
cargo run --locked --example core
```

For an application dependency and a minimal Rust program, start with [the README](README.md#search-application-records). Generic entries have `id`, `text` and optional `metadata`. `Engine::from_json` loads this record format. Your application owns storage and permissions.

## Return verifiable source text

```bash
cargo run --locked --features documents --example documents
cargo run --locked --features retrieval --example checked_evidence
```

The first example retains document revision and original UTF-8 positions. The second checks citations against the exact delivered response. Follow the [task cookbook](docs/API-COOKBOOK.md) when combining those APIs.

## Import a Markdown collection and connect MCP

This complete source-checkout example builds a new workspace, queries it and exposes the saved collection through the existing read-only MCP transport:

```bash
export AIRUST_DEMO_DIR="$(mktemp -d /tmp/airust-reference.XXXXXX)"
cargo run --locked --features mcp --example reference_workspace -- build tests/fixtures/reference-v1/sources "$AIRUST_DEMO_DIR/workspace" manuals
cargo run --locked --features mcp --example reference_workspace -- query "$AIRUST_DEMO_DIR/workspace" "Welche Bytepositionen verwendet TextSpan?"
cargo run --locked --features mcp --example reference_workspace -- serve "$AIRUST_DEMO_DIR/workspace"
```

For your own collection, replace the input directory and collection label. The output directory must be new. Original source files remain unchanged. The saved workspace retains its exact text if the input files later change; create another workspace to import a new revision. [Input limits, integrity checks and failure behavior](docs/REFERENCE-APPLICATION.md).

An MCP client launches the compiled example executable with `serve` and the absolute workspace path. Standard output carries protocol messages. Closing client input shuts down the session. Every loaded document is readable by the connected client.

For the published executable:

```bash
cargo install airust --version 0.5.3 --locked --no-default-features --features mcp --bin airust-mcp
airust-mcp --knowledge /absolute/path/knowledge.json
```

Select one `--knowledge` file or one or more `--document` snapshot files. The standalone executable reads those files at startup; it does not scan arbitrary client-supplied paths. Read [configuration, schemas, progress and limits](docs/MCP.md) and [the trust boundary](docs/MCP-SECURITY.md).

## Evaluate search configurations

Validate the frozen dataset:

```bash
python3 -B scripts/reference_dataset.py
```

The [reference evaluation](docs/measurements/reference-v1/README.md) explains exact reproduction, optional model preparation, thresholds and all measured regressions. Its questions are explicitly authored reference cases. Development selection precedes the reserved evaluation run.

Run the five model-independent comparison profiles into a new directory:

```bash
export CARGO_TARGET_DIR=/tmp/airust-build
cargo build --release --locked --features mcp,sqlite-storage,advanced-search,quality-gates,hybrid,reranking --example reference_evaluation
python3 -B scripts/run_reference_evaluation.py --binary "$CARGO_TARGET_DIR/release/examples/reference_evaluation" --output /tmp/airust-reference-lexical-run
```

The runner exports the verified dataset, runs all development profiles, freezes the development choice, and then evaluates the reserved split. Its output directory must not exist. To prepare real local BGE-M3 vectors, first export the dataset into another new directory and start your locally configured Ollama service with the intended model available:

```bash
python3 -B scripts/reference_dataset.py --output /tmp/airust-reference-dataset
python3 -B scripts/reference_embeddings.py /tmp/airust-reference-dataset /tmp/airust-reference-vectors.json --url http://127.0.0.1:11434 --model bge-m3:latest
```

Model identity is resolved and checked during preparation; the report records the digest, not just the mutable model label. Preparation sends source text only to the explicitly selected local service. Add `--vectors /tmp/airust-reference-vectors.json` to a new runner invocation for the two hybrid profiles.

For the complete nine-profile campaign, explicitly obtain the pinned reranker assets and supply your trusted platform ONNX Runtime library. This downloads the named model only when you invoke the helper:

```bash
python3 -B scripts/download_reranker.py /tmp/airust-mmarco-minilm-v2
cargo build --release --locked --features mcp,sqlite-storage,advanced-search,quality-gates,hybrid,local-reranking --example reference_evaluation
python3 -B scripts/run_reference_evaluation.py --binary "$CARGO_TARGET_DIR/release/examples/reference_evaluation" --output /tmp/airust-reference-model-run --vectors /tmp/airust-reference-vectors.json --reranker /tmp/airust-mmarco-minilm-v2 --runtime /absolute/path/to/libonnxruntime.dylib
```

The runtime filename differs by platform. The runner records its hash and exact-child peak RSS on macOS/Linux. Model-backed campaigns take substantially longer than lexical checks. Keep reports and assets outside the Cargo target directory. Selection never uses the reserved evaluation labels; do not retune version one from those results.

For model-independent examples of existing optional APIs:

```bash
cargo run --locked --features embeddings --example semantic
cargo run --locked --features advanced-search --example advanced_search
cargo run --locked --features reranking --example reranking
```

The semantic and reranking demonstrations use handwritten providers. For actual inference, supply the files described in [local embeddings](docs/LOCAL-EMBEDDINGS.md) or [local reranking](docs/LOCAL-RERANKING.md). Builds and ordinary tests do not download model weights or native runtimes.

## Run checks with bounded build storage

Use a dedicated directory for regenerable Cargo output:

```bash
export CARGO_TARGET_DIR=/tmp/airust-build
export CARGO_PROFILE_DEV_DEBUG=0
export CARGO_PROFILE_TEST_DEBUG=0
export CARGO_INCREMENTAL=0
./test.sh
```

The script checks default and all-feature behavior, isolated features, examples, real CLI/MCP boundaries, Clippy, documentation and package contents. Model-backed evaluation is separate. After preserving packages, locks and test receipts, `cargo clean` with the same target directory removes only that build output. Keep source documents, workspaces, original knowledge and measurement archives elsewhere.

## Import PDF training data

The compatibility PDF tool converts text-bearing PDFs into the historical `input`/`output` training format:

```bash
cargo run --locked --features cli,pdf --bin pdf2kb -- data/pdf/sample.pdf /tmp/airust-sample.json --timeout-seconds 60
cargo run --locked --features cli,pdf --bin pdf2kb -- --help
```

The supervised command enables bounded import defaults and validates the result before publication. An existing destination requires `--overwrite`. Explicit limits include `--max-file-mib`, `--max-pages`, `--max-text-mib`, `--max-chunks` and `--max-result-mib`. These are not hard OS CPU/RAM quotas. Image-only PDFs require an external OCR step. [PDF limits](docs/PDF-LIMITS.md), [workers](docs/PDF-WORKERS.md), [retained knowledge](docs/PDF-STABILITY.md).

## Use historical training files and agents

```bash
cargo run --locked --features cli --bin merge_kb -- source-a.json source-b.json --output /tmp/airust-merged.json
cargo run --locked --features cli --bin airust -- cli query simple "What is AIRust?"
cargo run --locked --features cli --bin airust -- cli query fuzzy "What is AIRust?"
cargo run --locked --features cli --bin airust -- cli query tfidf "What is AIRust?"
```

`merge_kb` validates every source before publishing output. With no source arguments it reads `knowledge/*.json` and targets `knowledge/train.json`. The historical `tfidf` agent uses BM25; the generic Engine selects TF-IDF explicitly through `SearchOptions`. These adapters return stored answers. The `cli` feature embeds sample knowledge, while the default library embeds none.

## Run the compatibility dashboard

```bash
cargo run --locked --features web --bin airust
```

The dashboard listens on `127.0.0.1:7070`, with its console on `127.0.0.1:7071`. `--port 8080` changes them to 8080 and 8081. `--help` lists commands. The dashboard database is `airust.db` in the working directory; preserve a backup when changing an existing installation.

Non-loopback operation requires `AIRUST_API_TOKEN`, containing 32..256 ASCII letters, digits, hyphens or underscores. A configured token also protects loopback operation. For example:

```bash
export AIRUST_API_TOKEN="$(openssl rand -hex 32)"
cargo run --locked --features web --bin airust
```

The browser asks for the token and holds it in page memory. HTTP clients send it as a Bearer authorization header. `AIRUST_ALLOWED_HOSTS` configures explicit expected hosts. Administration is disabled by default; `AIRUST_ENABLE_ADMIN=1` enables filesystem administration, knowledge load/save and destructive console reset within the documented working-directory restrictions. [Security and operating limits](docs/SECURITY.md).

Background startup uses `-d`; `airust stop` verifies recorded process identity before signaling. `AIRUST_RUNTIME_DIR` selects an absolute private state directory, defaulting to `$HOME/.local/state/airust`. Its Unix mode is `0700`, and insecure or symlink directories are rejected. Use the console's `reset` only when intending to delete its dashboard database.

## Build the optional web container

```bash
docker build -t airust:dev .
export AIRUST_API_TOKEN="$(openssl rand -hex 32)"
docker run --rm -e AIRUST_API_TOKEN -p 127.0.0.1:7070:7070 -p 127.0.0.1:7071:7071 airust:dev
```

The image builds the web feature and requires the token for its container listener. These mappings expose it only on the host's loopback interface. Container data needs an explicit persistent volume to survive container removal. Existing host databases, backups and logs are excluded from the build context.

For the full interface, read [the cookbook](docs/API-COOKBOOK.md). Platform and feature commitments are in [SUPPORT.md](docs/SUPPORT.md); historical changes remain in [CHANGELOG.md](CHANGELOG.md).


## Collect pilot questions and run live profiles

Follow [PILOT.md](docs/PILOT.md) for the complete new-directory setup, MCP client config, explicit capture/review/export and independent Python verification. Use the source build for 0.6.0 profile commands; no model/runtime/service is downloaded or started automatically. The [migration guide](docs/MIGRATION-0.6.0.md) lists the new passage bound, optional features, preserved formats and explicit quality-data gate.
