# AIRust

A Rust library for finding records and returning inspectable source evidence. Start with lexical search. Add document revisions, bounded excerpts, embeddings or an agent transport when your application needs them.

The **0.6.0 source milestone** adds a [bounded local pilot and explicit live search profiles](docs/PILOT.md). Engineering validation and real-data readiness are separate; a genuine independently reviewed reference-v2 dataset is still required for a new production-quality recommendation.

The published baseline **AIRust 0.5.3** adds a complete reference workflow, a frozen evaluation and measured maintenance improvements. Read [release scope](docs/CHANGES-0.5.3.md), [upgrade instructions](docs/MIGRATION-0.5.3.md) and [release verification](docs/validation-0.5.3/README.md). The published [0.5.2 baseline](docs/validation-0.5.2/closeout.json) remains reproducible.

## Search application records

Use Rust 1.89 or newer. Add the published library:

```toml
[dependencies]
airust = { version = "=0.5.3", default-features = false }
```

```rust
use airust::{Engine, KnowledgeEntry};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut index = Engine::new();
    index.upsert(KnowledgeEntry::new(
        "ownership",
        "Rust ownership protects memory.",
    ))?;
    index.upsert(KnowledgeEntry::new(
        "scripting",
        "Python helps with scripting.",
    ))?;

    let hits = index.search("Rust memory", 5);
    assert_eq!(hits[0].entry.id, "ownership");
    println!("{}", hits[0].entry.text);
    Ok(())
}
```

From a source checkout, run `cargo run --locked --example core`. Applications own IDs, storage, permissions and presentation. Search returns borrowed original entries, scores and matching terms. Equal scores use ascending IDs; a query without a match returns an empty list. BM25 is the default, with TF-IDF available explicitly. Scores are relevance values, not probabilities.

## Return source passages

Use `documents` to bind passages to an exact text revision and UTF-8 range. Add `retrieval` for owned, bounded excerpts and JSON suitable for an application transport.

```bash
cargo run --locked --features documents --example documents
cargo run --locked --features retrieval --example checked_evidence
```

Keep the snapshot together with its passages. Resolve citations against that revision after retrieval. A verified quotation establishes a byte match with supplied text; the application still reviews whether it supports an answer. Read the [task cookbook](docs/API-COOKBOOK.md), [document contract](docs/DOCUMENTS.md) and [citation checks](docs/CITATIONS.md).

## Connect a local agent

Install the published read-only MCP executable:

```bash
cargo install airust --version 0.5.3 --locked --no-default-features --features mcp --bin airust-mcp
airust-mcp --knowledge /absolute/path/knowledge.json
```

Use `--document /absolute/path/snapshot.json` for checked document coordinates. Select one knowledge file or one or more document snapshots. The client can read every loaded document. The five tools search, read a passage, compare entries, suggest relations and report status over standard input/output. Returned source text is untrusted content. See [client configuration, schemas and limits](docs/MCP.md) and [the trust boundary](docs/MCP-SECURITY.md).

## Try the complete reference workflow

The source checkout includes an application example that imports Markdown, saves immutable snapshots, returns citations and serves the same collection over MCP:

```bash
export AIRUST_DEMO_DIR="$(mktemp -d /tmp/airust-reference.XXXXXX)"
cargo run --locked --features mcp --example reference_workspace -- build tests/fixtures/reference-v1/sources "$AIRUST_DEMO_DIR/workspace" manuals
cargo run --locked --features mcp --example reference_workspace -- query "$AIRUST_DEMO_DIR/workspace" "Welche Bytepositionen verwendet TextSpan?"
cargo run --locked --features mcp --example reference_workspace -- serve "$AIRUST_DEMO_DIR/workspace"
```

The serve command runs until its MCP client closes standard input. Use a new output directory for each import. The saved collection retains its original text if input files later change. Queries return at most six passages with checked source coordinates and a complete 64 KiB JSON bound. Nonempty results leave answerability unassessed. [Workflow, accepted inputs and failure behavior](docs/REFERENCE-APPLICATION.md).

## Choose optional capabilities

Default Cargo features are empty. Enable only the capabilities used by the application.

| Task | Features | Guide |
| --- | --- | --- |
| Exact document revisions and positions | `documents` | [Documents](docs/DOCUMENTS.md) |
| Compact excerpts, explicit question preparation and citation checks | `retrieval` | [Retrieval](docs/RETRIEVAL.md) |
| Phrases and German word forms | `advanced-search` | [Text search](docs/TEXT-SEARCH.md) |
| Application-supplied embeddings and rank fusion | `embeddings`, `hybrid` | [Semantic](docs/SEMANTIC.md), [hybrid](docs/HYBRID.md) |
| A bounded second ranking stage | `reranking` | [Reranking](docs/RERANKING.md) |
| Local ONNX model adapters | `local-embeddings`, `local-reranking` | [Local embeddings](docs/LOCAL-EMBEDDINGS.md), [local reranking](docs/LOCAL-RERANKING.md) |
| Original-record persistence or SQLite full-text search | `storage`, `sqlite-storage`, `disk-index` | [Storage](docs/STORAGE.md), [disk index](docs/DISK-INDEX.md) |
| Consistent concurrent search/vector/source readers | `snapshots` | [Snapshots](docs/SNAPSHOTS.md) |
| Relationships, provenance, feedback or explicit rules | Select the corresponding module | [Complete feature and support matrix](docs/SUPPORT.md) |

The provider-neutral library supports Rust 1.89. Local ONNX adapters require Rust 1.92 and explicitly supplied assets. Builds do not download model weights or ONNX Runtime binaries. Historical `Agent`, `KnowledgeBase` and dashboard interfaces remain compatibility paths; start new record-search integrations with `Engine` and `Retriever`.

## Evaluate the chosen workload

The [reference application](docs/REFERENCE-APPLICATION.md) declares its use case and acceptance thresholds. Its dataset contains 150 explicitly authored questions over frozen documentation, with development and evaluation source families separated. The [reference report](docs/measurements/reference-v1/README.md) retains configuration costs and individual regressions. These reference results do not establish customer adoption or general retrieval superiority.

The [historical 0.4.4 comparison](docs/HONEST-COMPARISON-0.4.4.md) found no general hybrid-quality or token-saving advantage over SQLite plus the same embeddings. It remains historical evidence. Larger synthetic capacity and prior performance campaigns retain their original scope in the [benchmark guide](docs/BENCHMARKS.md) and [change history](CHANGELOG.md).

## Build and validate

```bash
./test.sh
```

This checks the default library, optional features, complete examples, real CLI/MCP boundaries, Clippy, documentation and package contents. Native-model evaluation is a separate explicit operation. Read [contribution requirements](CONTRIBUTING.md), [platform support](docs/SUPPORT.md), [startup and tools](STARTUP.md), and the [roadmap](ROADMAP.md).

AIRust is MIT-licensed. The optional maintained PDF backend preserves its own upstream licenses and notices; its maintenance contract is described in [PDF maintenance](docs/PDF-MAINTENANCE.md).
