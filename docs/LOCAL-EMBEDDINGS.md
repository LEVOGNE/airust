# Optional local E5 embeddings

The `local-embeddings` feature adds `FastEmbedProvider`, a CPU adapter for explicitly supplied `intfloat/multilingual-e5-small` ONNX files. It also enables `embeddings`. The adapter needs Rust 1.92 or newer; the default core and provider interface still support Rust 1.89.

```toml
[dependencies]
airust = { version = "=0.2.0", default-features = false, features = ["local-embeddings"] }
```

The selected FastEmbed dependency disables its defaults, model-hub access, and binary downloads. Neither a normal build nor `--all-features` downloads model weights or ONNX Runtime binaries. The application obtains those files separately and supplies their paths. Native dependencies are still compiled as ordinary Cargo dependencies.

## Required local files

The model directory must contain:

```text
config.json
special_tokens_map.json
tokenizer.json
tokenizer_config.json
onnx/model.onnx
```

Supply the platform's ONNX Runtime shared library as a separate explicit path. AIRust uses dynamic loading and CPU inference; it does not modify process environment variables or download native runtime files. Missing or unloadable runtime libraries are returned as provider errors. Only load trusted model and native library files.

```no_run
use airust::{
    semantic::{local::FastEmbedProvider, SemanticIndex, SemanticSearchOptions},
    Engine, KnowledgeEntry,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut provider = FastEmbedProvider::multilingual_e5_small(
        "/models/multilingual-e5-small",
        "/runtime/libonnxruntime.dylib",
        2,
    )?;
    let lexical = Engine::from_entries([
        KnowledgeEntry::new("registration", "Kfz-Zulassung durchführen."),
        KnowledgeEntry::new("cooking", "Brot mit Mehl, Wasser und Hefe backen."),
    ])?;
    let index = SemanticIndex::from_index(lexical, &mut provider, 16)?;
    let hits = index.search(
        "Ein Fahrzeug bei der Behörde anmelden",
        &mut provider,
        &SemanticSearchOptions::new(3),
    )?;
    for hit in hits {
        println!("{}: {:.3}", hit.entry.id, hit.score);
    }
    Ok(())
}
```

The final constructor argument is a positive CPU thread count. The adapter uses mean pooling, 512-token truncation, `query: ` for queries, and `passage: ` for documents. It passes batches of at most 32 texts to FastEmbed. Model load and native inference can be relatively expensive; reuse the provider rather than constructing it for every query.

The embedding-space fingerprint hashes the five actual files, their names and lengths, and the preprocessing recipe. Changing weights, tokenizer files, or the declared recipe produces another space identity. Model names or dimensions alone are not compatibility guarantees. E5 returns 384-dimensional vectors.

The evaluated ONNX model file is 470,268,510 bytes. The tokenizer is another 17,082,730 bytes, plus small JSON files. These assets are external to the crate and are not small enough to include in the default build. The E5 model is MIT-licensed; FastEmbed is Apache-2.0-licensed. Review the original model and runtime licenses for your distribution.

## Quality checks

Run the public fixture evaluation with your local paths:

```bash
cargo run --locked --release --features local-embeddings --example semantic_quality -- MODEL_DIRECTORY ONNX_RUNTIME_LIBRARY
```

The example embeds twelve public records and checks sixteen German, English, French, and Spanish queries. It prints model identity, timing, ranking results, and explicit smoke-quality gates. A small hand-labeled corpus does not establish general semantic accuracy. Negation, prerequisites, contradictions, and factual truth are not inferred by cosine similarity.

The default test suite and all-feature tests use deterministic provider fixtures and missing-file checks. They need no model or native runtime installation. Real-model evaluation is an explicit additional command with externally supplied files.

Primary references: [FastEmbed source and license](https://github.com/Anush008/fastembed-rs), [E5 model card and license](https://huggingface.co/intfloat/multilingual-e5-small), and [ONNX Runtime releases](https://github.com/microsoft/onnxruntime/releases).
