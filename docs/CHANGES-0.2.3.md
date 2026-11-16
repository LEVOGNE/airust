# AIRust 0.2.3: structured context assembly

The optional `context-assembly` feature composes the existing lexical, semantic, hybrid, and graph APIs. `ContextAssembler` borrows an existing index and optional graph. It returns structured excerpts with stable IDs, metadata, ranking evidence, and graph-path evidence. The default lexical build and original entry fields are preserved. No new dependency, mandatory model, prompt format, or application integration is added.

`ContextOptions` separates source-node count, total returned UTF-8 text bytes, each item's bytes, ranker settings, scope, deduplication, and traversal. Selected search seeds take priority over neighbors. Normalized duplicate content shares an excerpt while retaining distinct source references. Distinct source IDs still consume the node budget. A fixed total graph-visit budget spans seed traversals.

## Bug hunt and validation

Fifteen new integration tests cover scope intersection, metadata conflicts, hidden/missing graph bridges, stable priorities, traversal filters, global graph budgets, original input ranks, duplicate IDs and normalized content, metadata retention, provider errors, explicit hybrid fallback, invalid vectors, denied scopes, empty/zero budgets, serialization, and borrowed offsets. A nested budget sweep checks multilingual strings over 0–59 total bytes, six node limits, and eight per-item limits.

The budget review caught a subtle streaming issue: precollecting candidates up to the node budget could discard a later fitting entry after an earlier multibyte entry fitted no byte budget. Assembly now consumes the finite input iterator lazily until enough entries are actually selected. A regression also checks that a full node budget does not consume another iterator item. Query excerpts locate a deep original-text term instead of blindly returning an unrelated prefix, with UTF-8-safe clipping even at one byte.

The new `Context::intersection` returns `None` for disjoint explicit ID filters or conflicting metadata. It never represents a denied intersection as an empty, unrestricted ID set. `breadth_first_matching` applies its predicate before queueing graph nodes; excluded or missing source entries cannot become bridges. Existing graph traversal behavior is preserved by using an always-accepting predicate for the old API.

The complete release script passed **189 default tests** and **280 all-feature tests**, including the compiled context example. Formatting, warning-free Clippy, rustdoc, isolated feature builds, runnable examples, the source-comment check, and the Rust 1.89/1.92 boundaries are part of the release checks. The CLI process-identity regression needs process inspection; the sandbox-denied attempt was rerun with that capability rather than changing the test.

The dependency audit reports zero known vulnerabilities and the same two optional maintenance warnings: `paste` through tokenizers and `ttf-parser` through PDF support. These warnings remain documented and are not suppressed.

## Reproducible measurements

The release-mode benchmark builds unique identifier records with repeated multilingual text and a separate chain graph. One hundred queries each match a rare identifier, then assemble context with at most twenty source IDs, 2,048 returned text bytes, 256 bytes per item, ten graph hops, and one hundred total graph visits. The host is macOS ARM64 with 64 GiB memory and Rust 1.98.0.

| Entries | Index + graph build (ms) | Context p50 (ms) | Context p95 (ms) | Peak process RSS (MiB) |
| ---: | ---: | ---: | ---: | ---: |
| 1,000 | 16.330 | 0.040709 | 0.046000 | 8.22 |
| 10,000 | 166.440 | 0.045125 | 0.056750 | 57.19 |
| 100,000 | 1,661.529 | 0.054458 | 0.065458 | 541.23 |

```bash
cargo build --locked --release --features context-assembly --example context_benchmark
/usr/bin/time -l target/release/examples/context_benchmark 1000
/usr/bin/time -l target/release/examples/context_benchmark 10000
/usr/bin/time -l target/release/examples/context_benchmark 100000
```

Raw samples and resource counters are in [measurements/0.2.3](measurements/0.2.3/). These are warm, rare-term, synthetic queries on one sparse topology. There is no model inference or final prompt tokenization. Common terms, dense graphs, large metadata, and production contention can have different costs. Memory includes original text, lexical structures, and graph, not only the returned context. This is not a universal latency or memory promise.

The package inspection caught macOS Finder metadata inside broad explicit include patterns. Cargo ignores Git ignore rules when an explicit include list is present. The manifest now excludes `.DS_Store` and AppleDouble files directly, preserving local files while keeping them out of release artifacts. This follows the [Cargo packaging contract](https://doc.rust-lang.org/cargo/reference/manifest.html#the-exclude-and-include-fields).

## Limits

The explicit byte budget counts returned text only. IDs, metadata, relation evidence, serialization and prompt instructions are outside it. Applications must measure final tokenizer-specific prompts. Excerpts are a deterministic word-anchor heuristic and can cut words or sentences. Semantic matches without a literal anchor use a prefix. Duplicate excerpt offsets refer to the first source; normalized duplicates may differ in original whitespace and casing. See [CONTEXT.md](CONTEXT.md) for complete contracts.

No independent real-application relevance comparison or token-saving claim is made. An application remains responsible for permissions, model/provider policy, durable data and synchronization. Scope filtering is not authentication. Structured provenance, derived caches, ANN and revisions remain separate later stages.

## Package and compatibility gate

The inspected crate contains 147 files, approximately 271 KiB compressed, with required sources, external documentation, tests, examples and measurements. All 280 all-feature tests passed from the extracted archive. An independent Rust 1.89 consumer passed existing lexical/semantic/hybrid/graph operations and the new context API, including denied scope intersections. The clean package, dry run, candidate CI and registry/tag/asset checks gate publication and the next stage. Artifacts: [GitHub v0.2.3](https://github.com/LEVOGNE/airust/releases/tag/v0.2.3), [crates.io 0.2.3](https://crates.io/crates/airust/0.2.3).
