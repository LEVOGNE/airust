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

## Cooperative time budgets (0.2.17)

```rust
use airust::execution::{WorkControl, ExecutionError};
use std::time::{Duration, Instant};
let control = WorkControl::default().with_timeout(Duration::from_secs(2)).unwrap();
assert!(control.deadline().is_some());
let expired = control.clone().with_deadline(Instant::now() - Duration::from_secs(1));
assert_eq!(expired.checkpoint(), Err(ExecutionError::DeadlineExceeded));
assert_eq!(expired.remaining(), Some(Duration::ZERO));
```

`with_timeout(Duration)` uses the monotonic clock and returns InvalidBudget if checked clock addition overflows. The budget starts when this builder is called, not when a later operation starts. Construct one per task or deliberately share an overall budget. `with_deadline(Instant)` selects an absolute monotonic deadline. Both methods can only shorten a deadline already attached to that control. Clone/combined controls preserve the earliest deadline; a later child budget cannot extend it. `deadline()` returns the optional Instant; `remaining()` returns None for unlimited work or a saturating Duration, including zero when expired. These monotonic values are not portable persisted timestamps.

Every supported checkpoint described above checks cancellation first, then deadline expiry. If both apply, cancellation wins. No deadline is enabled by default. A deadline error is distinct from cancellation, provider failure and insufficient evidence. The existing typed execution error paths carry it through search, reranking, retrieval and dataset operations. Fallback rules do not consume it.

A budget is a cooperative limit, not a hard timeout or CPU quota. Blocking inference, a large normalization/clone/sort or a publication lock wait may exceed it before returning to a checkpoint. A late provider result is discarded even if it succeeded or returned a fallback-eligible error. Provider side effects are not rolled back. Controlled publication checks again after lock acquisition and before its commit boundary; a later racing deadline cannot turn an already committed success into an apparent failure. Application/network/provider timeouts must be configured separately when hard availability limits are needed.

Run `cargo run --features execution,retrieval --example time_budget`. It measures deadline checking separately from untimed control and deliberately demonstrates a 25 ms blocking call with a 5 ms budget. The returned error does not imply that the thread stopped after 5 ms. See [0.2.17 evidence](CHANGES-0.2.17.md).

## Progress notifications (0.2.18)

```rust
use airust::execution::{WorkControl, ProgressEvent, WorkPhase};
use std::sync::{Arc, Mutex};
let events = Arc::new(Mutex::new(Vec::new()));
let received = events.clone();
let control = WorkControl::default().with_observer(move |event: &ProgressEvent| {
    received.lock().unwrap().push(*event);
});
control.report(ProgressEvent::new(WorkPhase::Ranking, 3, Some(10)).unwrap());
assert_eq!(events.lock().unwrap()[0].fraction(), Some(0.3));
assert_eq!(control.observer_failures(), 0);
```

`with_observer` opts into a synchronous ProgressObserver. A Send + Sync + RefUnwindSafe closure taking `&ProgressEvent` is supported. The builder replaces earlier observers on that control; combined controls retain both sets and deduplicate shared observer slots. No observer, background worker, event queue, network call or progress JSON field is enabled by default. WorkControl remains cloneable across threads.

ProgressEvent::new validates completed <= total when a total is known. It exposes phase, completed, total, sequence and fraction. None means the phase total is unknown; show an indeterminate indicator. A known zero total is an empty completed phase with fraction 1. Sequence starts at one on delivery and is scoped to an observer slot, shared through clones; counters saturate at usize::MAX. Constructed but unreported events have sequence zero. Counts describe the current phase, not the whole pipeline. A phase can repeat for another provider batch or suboperation; do not combine those fractions into a guessed global percentage or ETA.

Built-in notifications cover query start/completion, source retrieval, lexical candidate collection, ranking, embedding batches, hybrid fusion, reranking, excerpt selection, evidence assessment, serialization attempts, dataset validation/staging, positional rebuilding and publication. Candidate collection counts visited posting bindings, including repeated IDs across query terms. Ranking counts inspected candidates, including excluded ones, not just accepted hits. Provider batches restart their own counts. Some phases report only start/end; a large indivisible clone, provider call or normalization remains between those reports. Counts are actual work units, not answer confidence.

## Callback isolation and completion

Callbacks run outside dataset publication locks. Publication start is sent before lock acquisition, completion after the lock is released. An observer may inspect the manager. If it cancels before the final publication checkpoint, publication fails without a commit. Cancellation from a publication-complete callback cannot change the successful committed result. Query completion is emitted only after a complete response passes its final interruption checkpoint and serialization.

A callback panic is caught under normal Rust unwinding, counted by observer_failures(), and does not turn successful work into a failure. Rust's panic hook may still print the panic. Process-aborting panics cannot be recovered. A busy observer slot drops concurrent or reentrant notifications rather than waiting or recursing; dropped_events() reports them. Delivered callbacks for that slot are serialized. Other independent observers can still receive their events. Notifications are best effort: use the operation's returned Result/revision as the authoritative completion signal, even when its last notification was dropped.

A callback itself can block its invoking thread and consume a time budget. Keep it short. For a user interface or transport, use a bounded channel and a nonblocking try_send inside the callback; let the application render or stream those updates. Queue overflow is application telemetry, separate from AIRust's busy-slot drop counter. Use one observer per request or attach the request ID in the observer closure when correlating concurrent operations. Avoid a strong reference cycle by capturing a control inside its own observer.

Run `cargo run --features execution,retrieval --example progress`. Progress goes to stderr; the final compact response goes to stdout. This provides a transport-neutral hook for a future progress bar or status text. It neither modifies another application nor installs an instruction forcing external agents to narrate. A client can display 'retrieving', 'ranking' or 'waiting for provider', but cannot infer an answer arrival time from these events. See [0.2.18 verification](CHANGES-0.2.18.md).
