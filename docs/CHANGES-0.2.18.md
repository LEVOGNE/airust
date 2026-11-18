# AIRust 0.2.18: observable work phases

Optional execution observers now expose actual work phases, completed counts and known/unknown totals. A small application still loads no model or server. Existing response JSON does not gain a progress payload. Applications explicitly attach an observer and choose their own display or transport. No DenkMaps/MCP integration is performed.

Events cover candidate collection, ranking, embedding, hybrid fusion, reranking, text selection, assessment, serialization, dataset staging and publication. The Retriever additionally reports query start/completion; specialized entry points report their own phases. Unknown totals remain unknown. A phase can repeat for another batch/suboperation. These counts are neither answer confidence nor a global progress percentage or ETA.

## Executed verification and fixes

The complete suite passes **189 default and 538 all-feature Rust tests**, including compiled documentation, plus five Python helper tests. Fourteen progress cases validate count bounds, sequencing, observer composition/deduplication, panic isolation, concurrent/reentrant notification dropping, actual candidate/batch/fusion counts, unchanged response JSON, callback cancellation, absence of query completion on interrupted work, and safe reads/cancellation around publication. The isolated execution-only progress suite runs six cases. A compile-time regression preserves the original Send/Sync/UnwindSafe/RefUnwindSafe auto traits of WorkControl by requiring unwind-safe observer references.

A publication callback reads the manager through a separately timed reader, proving that notifications do not hold its write lock. Cancellation before publication leaves the old revision intact; cancellation from completion still returns the actual successful committed revision. A cancelling/panicking query-complete callback likewise cannot invalidate its completed response. Callback panics are counted and the busy flag is released for later delivery. Busy/reentrant slots drop notifications rather than deadlocking or recursing.

Hybrid final fused-candidate scanning/sorting now also checks execution control, closing an interruption gap after its earlier merge checkpoints. Existing ranking evidence and values remain unchanged. Strict formatting, default/all-feature/execution-only Clippy, rustdoc, minimum Rust 1.89/1.92, patch compatibility against 0.2.17, audit and an independent consumer gate release. Extracted package tests, publish dry run, exact-commit CI and registry/tag/asset/VCS verification gate publication. No fresh native-model evaluation is claimed.

## Measurement

The fixed release-build fixture has 10,000 records and 200 common-term searches per pass. Alternating no-observer/observer elapsed times are 362.66/368.04/371.39/366.35 ms. Each observed pass delivers 8,600 counter-only notifications with zero drops/failures. Score-bit signatures match. These overlapping samples do not establish a universal overhead percentage or speed improvement. Whole-child peak 12.59 is 12.59 MiB. Blocking callbacks, UI work, channels and network transports are not benchmarked.

```bash
cargo build --release --locked --features execution --example progress_benchmark
python3 scripts/measure.py /tmp/progress.json target/release/examples/progress_benchmark
cargo run --locked --features execution,retrieval --example progress
```

Raw output and OS resources: [measurements/0.2.18](measurements/0.2.18/).

## Contract and limitations

Delivery is synchronous and best effort. Callbacks can consume time; use a bounded nonblocking application channel when rendering/streaming. A busy slot counts discarded concurrent/reentrant events. Normal Rust callback unwinding is isolated, but process-aborting panics cannot be recovered and the panic hook may still print. Inspect observer_failures() and dropped_events(). Always decide completion from the operation Result and returned revision, since even the final event can be dropped. No callback runs inside dataset publication locks.

Candidate collection counts visited posting bindings; ranking counts inspected candidates, including excluded ones. A large indivisible provider call, clone, sort, normalization or lock wait can remain between notifications. No agent instruction, MCP endpoint, frontend progress bar or external application is installed. See [execution API](EXECUTION.md), [cookbook](API-COOKBOOK.md), [migration](MIGRATION-0.2.18.md) and [audit metadata](measurements/0.2.18/audit.json).
