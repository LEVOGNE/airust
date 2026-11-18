# AIRust 0.2.16: cooperative cancellation

The new optional dependency-free execution feature supplies CancellationToken, WorkControl and typed interruption errors. Existing APIs/defaults retain their behavior unless control is explicitly supplied. Supported operations check during candidate collection/ranking, vector scanning, hybrid fusion, reranking batches, positional construction, compact retrieval and off-lock dataset staging. Another thread can cancel shared work without receiving a misleading empty result or provider fallback. No DenkMaps/MCP integration is performed.

Dataset preparation remains unpublished; cancellation discards the draft. Controlled publication checks before and after lock acquisition, immediately before the atomic commit boundary. It never checks after a successful commit and reports an error suggesting nothing changed. Cancellation racing after that boundary can therefore return a successful committed revision. A blocked provider, clone, normalization, sort or lock wait must return before the next checkpoint. This is cooperative cancellation, not thread termination or a hard latency bound.

## Verification and bug hunting

The complete suite passes **189 default and 514 all-feature Rust tests**, including compiled documentation, plus five Python helper tests. Eleven execution integration tests cover shared one-way cancellation across 32 concurrent signallers, exact ranking preservation for BM25/TF-IDF, pre-cancelled zero limits, custom source errors/fallbacks, cancellation during reranking and embedding batches, German phrase evidence, immutable old readers and cancelled publication. An internal regression stops within candidate construction after exactly thirty checkpoints and returns no partial ranking. The isolated execution-only integration suite passes three cases.

A composition regression ensures an existing scorer cancellation token is not erased by a request-level control. Semantic/hybrid adapters likewise preserve source-level controls. Cancellation is checked after provider return before provider errors can activate a fallback. This also prevents a cancelled fallback from being misreported as insufficient evidence.

Formatting, strict default/all-feature/execution-only Clippy, rustdoc, all shipped examples, minimum Rust 1.89/1.92, static patch compatibility against 0.2.15, fresh audit and an independent consumer are release gates. Extracted crate tests, publish dry run, exact-commit CI and registry/asset/tag/VCS verification gate publication. Native models are not downloaded or re-evaluated by these provider-independent tests.

## Measurements

The fixed release-build lexical fixture has 10,000 short records and 200 identical common-term searches per pass. Alternating uncontrolled/controlled times are 437.21/416.66 ms and 385.83/398.13 ms. Score-bit signatures are identical. These overlapping samples do not establish a speed improvement or a stable overhead percentage. Ten thousand already-cancelled calls take 0.028 ms in this tiny loop; that does not measure cancellation latency during blocking work. Whole-child peak RSS is 22.27 MiB. Raw commands, OS metadata and outputs are in [measurements/0.2.16](measurements/0.2.16/).

```bash
cargo build --release --locked --features execution --example execution_benchmark
python3 scripts/measure.py /tmp/execution.json target/release/examples/execution_benchmark
cargo run --locked --features execution,retrieval --example cancellation
```

## Limits and use

Not every specialized API has inner-loop control: graph traversal, matching, cache and storage calls retain their existing contracts. Custom sources must check a control themselves during their own long work. A single very large record/batch can delay a checkpoint. Provider side effects are outside AIRust's rollback boundary. Token composition does not broadcast child cancellation back into all parent tokens. Use a fresh independent token per task, since a cancelled token cannot be reset.

See [execution contract](EXECUTION.md), [cookbook](API-COOKBOOK.md), [migration](MIGRATION-0.2.16.md) and [audit metadata](measurements/0.2.16/audit.json). No model, network client or database is enabled by the execution feature.
