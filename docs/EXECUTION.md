# Cooperative execution control

Enable `execution`. It has no dependencies and does not enable retrieval, models, storage or servers. Select those separately when needed. Cancellation is opt-in; uncontrolled APIs retain their behavior.

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use airust::{Engine, KnowledgeEntry, SearchOptions, execution::*};
let index = Engine::from_entries([KnowledgeEntry::new("a", "Rust ownership")])?;
let token = CancellationToken::new();
let control = WorkControl::new(token.clone());
assert_eq!(index.search_controlled("Rust", &SearchOptions::default(), &control)?.len(), 1);
assert!(token.cancel());
assert!(!token.cancel());
assert!(matches!(index.search_controlled("Rust", &SearchOptions::default(), &control), Err(ExecutionError::Cancelled)));
# Ok(()) }
```

## Tokens and composition

A cloned CancellationToken shares one atomic one-way signal across threads. `cancel()` returns true only for the first signal. `is_cancelled()` reads it. Tokens have no reset; use a new token for an independent task. `WorkControl::new(token)` carries it; `checkpoint()` returns typed ExecutionError::Cancelled when signaled. Default creates a fresh token. `token()` exposes the primary token for cancellation.

`control.combined(&other)` retains both primary and parent signals, deduplicated by shared identity. Either cancels the combined work; combining does not cancel either original token. Options builders replace their previous attached control. The Retriever combines request control with an existing reranker or semantic/hybrid source control so that neither signal is erased. Cancelling a child alone need not cancel its parent.

## Supported entry points

| API | Where control is supplied |
| --- | --- |
| Lexical Engine | `search_controlled(query, &SearchOptions, &control)` |
| TextIndex | `search_controlled(query, &TextSearchOptions, &control)`; `from_entries_controlled(config, entries, &control)` |
| SemanticIndex search / search_embedding | `SemanticSearchOptions::with_control(control)` |
| Hybrid search | `HybridSearchOptions::with_control(control)` |
| Standalone reranking | `RerankOptions::with_control(control)` |
| Retriever | `RetrievalOptions::with_control(control)`; built-in sources forward it |
| DatasetManager | `prepare_controlled`, `prepare_replace_controlled`, `prepare_model_controlled`, `publish_controlled`, with the original arguments followed by `&control` |

Checks occur while collecting/scoring lexical candidates, between semantic vectors, while fusing hybrid results, between positional rebuild records, while staging dataset changes, around provider batches, during retrieval validation/excerpts and between output-serialization attempts. Source methods also honor their own configured controls. A custom RetrievalSource must cooperate inside its own work; the Retriever checks before and after its call. Existing specialized graph, matching, cache and storage operations do not automatically gain inner-loop cancellation.

## Failure and publication semantics

Cancellation is an execution error, not an empty result, an insufficient-evidence decision or provider failure fallback. Rerank and semantic errors include an Execution variant; HybridError wraps EmbeddingError. RetrievalError and DatasetError expose Execution directly or through their typed reranking/embedding/text variants. Match non-exhaustive enums with a wildcard. A cancelled operation does not return a partial success response.

Dataset preparation remains off-lock and unpublished. A cancelled draft is discarded. `publish_controlled` checks before waiting for the publication lock and again after acquiring it, immediately before the commit boundary. After that checkpoint, an atomic publication can succeed even if cancellation races with the commit. No post-commit cancellation check returns an error pretending the commit did not happen. Inspect the returned revision. Use the controlled publication call as well as controlled preparation; the ordinary publish method deliberately retains its original behavior.

These controls are cooperative. They cannot forcibly interrupt a blocked model, normalization call, allocation, clone, sort or lock wait. They check after that work returns. A single very large record or batch can increase cancellation latency. Use bounded records/batches and provider-specific cancellation where needed. Provider side effects are outside AIRust's rollback guarantee. No hard response-time guarantee or thread termination is claimed.

Run `cargo run --features execution,retrieval --example cancellation`. The `execution_benchmark` example compares controlled/uncontrolled lexical searches and already-cancelled calls. See [release evidence](CHANGES-0.2.16.md).
