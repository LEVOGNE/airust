# Structured context assembly

Enable `context-assembly` to combine the existing rankers and graph. It enables `hybrid` and `graph`, adds no model dependency, and supports Rust 1.89. Existing `Context` remains an ID/metadata filter, and `ContextAgent` keeps its historical behavior.

```rust
# #[cfg(feature = "context-assembly")]
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use airust::{Engine, KnowledgeEntry};
use airust::context_engine::{ContextAssembler, ContextOptions};
use airust::graph::{KnowledgeGraph, Relation, RelationType};

let mut index = Engine::new();
index.upsert(KnowledgeEntry::new("rust", "Rust provides memory safety."))?;
index.upsert(KnowledgeEntry::new("owner", "Ownership controls resource lifetimes."))?;
let mut graph = KnowledgeGraph::new();
graph.add_node("rust")?;
graph.add_node("owner")?;
graph.upsert_relation(Relation::new("e1", "rust", "owner", RelationType::References)?)?;
let context = ContextAssembler::new(&index).with_graph(&graph)
    .query("Rust", None, &ContextOptions::new(4, 512))?;
assert_eq!(context.node_count(), 2);
assert!(context.used_text_bytes() <= 512);
assert_eq!(context.items()[0].references()[0].id(), "rust");
println!("{}", serde_json::to_string(&context)?);
# Ok(())
# }
# #[cfg(not(feature = "context-assembly"))]
# fn main() {}
```

## Inputs and rankers

`ContextAssembler::new(&Engine)` borrows a lexical index. `from_semantic(&SemanticIndex)` borrows its existing lexical and vector data together. `with_graph(&KnowledgeGraph)` adds explicit relationships without copying source records. Results borrow their data from those sources, so the sources cannot be mutated while results are used.

`query(text, provider, &options)` defaults to BM25. Choose `with_lexical(SearchOptions)`, `with_semantic(SemanticSearchOptions)`, or `with_hybrid(HybridSearchOptions)`. Semantic/hybrid modes require the semantic assembler and an explicit `Some(&mut provider)`; missing components are errors. Model vectors, weights, minimum scores, candidates, and failure policy use the existing ranker contracts. Hybrid provider-only fallback remains opt-in and is visible through `semantic_used()` and `fallback_reason()`. Ranker scores are retained as evidence, not calibrated truth probabilities.

`assemble(ranked_ids, &options)` accepts already ranked IDs, preserves their order, skips repeated IDs, and counts processed missing or excluded IDs. It does not need a provider. The input iterator must be finite. When the node budget is full, later IDs need not be examined. Rank positions refer to original input positions, including skipped ones.

`ContextOptions::default()` uses twenty unique source IDs, 8,192 returned text bytes, 2,048 bytes per item, ten search seeds, normalized-content deduplication, original metadata, and graph neighbors within one hop in either direction. Configure these with `new(max_nodes, max_text_bytes)`, `with_item_bytes`, the ranker builders, `with_deduplication`, `with_metadata`, and `with_traversal(TraversalOptions)`. Zero node, text, or per-item budgets and blank queries return empty results without inference.

## Priority, scope, and graph evidence

Search/input seeds take priority over graph neighbors. Selected seeds are expanded in rank order using stable, bounded BFS. Graph settings keep their direction, depth, allowed IDs, and relation kinds. Start nodes are excluded from expansion results. The traversal's node limit is a total visit budget across seed traversals, including repeated visits across different seeds. `graph_visits()` reports those visits. Missing graph seeds are skipped. Missing source entries and excluded entries cannot be traversal bridges. Budgets bound selected output and stored traversal candidates; they do not bound every inspected edge or ranker operation.

`with_scope(Context)` intersects with the active ranker's context filter. Every returned ID must satisfy both. `Context::intersection(&other)` returns `None` for disjoint explicit ID sets or conflicting metadata. `Some(Context)` is the combined filter; an empty ID set in the existing filter format still means unrestricted IDs. Never turn `None` into an unrestricted default filter. This utility does not replace application authentication or authorization.

Each item preserves source references. A reference contains its stable ID, optional unmodified metadata, and origin. Origin getters expose `kind`, rank, score, ranker mode, or graph seed/parent/relation/depth. A graph origin explains one selected BFS path, not every possible path. Metadata and relation metadata can contain private information; applications may omit entry metadata with `with_metadata(false)` and must choose permitted graph data.

## Byte budgets, excerpts, and duplicate content

`used_text_bytes()` is the sum of returned `item.text().len()` values. It excludes IDs, metadata, relation evidence, serialization, prompt instructions, and model-specific tokenization. Measure the final serialized prompt with the chosen model's tokenizer when a token limit matters.

Text slices always end on UTF-8 boundaries. For an overlong text with a query-word match, AIRust tries to place an excerpt around the first original-text word that matches a normalized, non-stopword query term. Otherwise it takes a prefix. This is a deterministic excerpt heuristic, not semantic sentence selection. A tiny byte budget may fit no character. Items report `truncated`, `start_byte`, and exclusive `end_byte`; offsets refer to the original text of the **first reference**. A snippet may cut words or sentences. Compatibility characters that change word boundaries during normalization can miss an anchor and fall back to a prefix.

Deduplication uses cached Unicode-normalized full content, not the clipped excerpt. Distinct content that shares a prefix is not merged. Case/whitespace/Unicode-equivalent content can share one excerpt while retaining separate source references and their metadata. Every distinct included source ID consumes one node of the node budget, even when text is shared. Offsets and quotation spelling belong to the first reference; other normalized duplicates may have different original offsets or casing. No content, source ID, or metadata is invented.

The bundle exposes `items`, `node_count`, text bytes, missing/scope/budget counters, graph visits, and semantic fallback evidence, and implements `Serialize`. Counters describe examined candidates, not every record that could have been selected. Output is deterministic for fixed inputs and deterministic provider results. AIRust does not create a prompt, run an LLM, infer factual relationships, or synchronize the application database.
