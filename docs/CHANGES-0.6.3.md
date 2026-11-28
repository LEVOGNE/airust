# AIRust 0.6.3: operational robustness of the process worker

AIRust 0.6.3 hardens the opt-in native-model process worker from 0.6.2 under sustained load and failures. It also applies the security decisions from the 0.6.2 source review. It adds no search algorithm, no default feature and no core dependency. Existing public APIs keep their signatures.

## New (optional, behind `local-reranking`)

- **`ProcessPool`.** It admits at most `max_workers` worker processes and at most `queue_limit` waiting callers.
  - Overflow and expired waits return the new `RerankError::Busy` immediately. `RerankError` is `non_exhaustive`, so this is additive.
  - With the `execution` feature, `acquire_controlled` honors a `WorkControl` deadline and cancellation.
  - A failed lease retires its worker, and the next caller gets a new one without exceeding the limit.
- **`ProcessOptions::with_runtime_identity(bytes, sha256)`.** It pins the ONNX Runtime library, which is verified while it is copied into the private snapshot directory. The worker loads that copy.
- **Examples.** `native_worker_soak` (fault-injection soak) and `native_pool_throughput`, plus `scripts/analyze_worker_soak.py`.

## Changed behavior

- **`AIRUST_ENABLE_ADMIN=1` now requires `AIRUST_API_TOKEN`.** Without a token, the web server refuses to start. Before, any local process could use file administration and SQL on a loopback server. Migration: set a 32–256-character token whenever administration is enabled.
- **The model worker no longer inherits the parent environment.** Only `PATH`, temporary-directory, locale and Windows system variables are passed.
- **The web API rejects oversized values with HTTP 413 before storage.** This covers settings, chat titles, category and collection names and descriptions, vector content and metadata, and Tatoeba rows (see [SECURITY.md](SECURITY.md)).
- **Newly created web databases get three lookup indexes.** Existing database files are not changed.
- **The web server sends `Content-Security-Policy-Report-Only`.** Enforcement needs an interface rewrite and is not part of this release.

## Validation

The criteria were fixed before testing ([CRITERIA.md](validation-0.6.3/CRITERIA.md)); results are in [RESULTS.md](validation-0.6.3/RESULTS.md) and release checks in [RELEASE.md](validation-0.6.3/RELEASE.md).

- **Three-hour fault-injection soak.** It passed every criterion in a third run on a host without foreign load: 4529 cycles, 1359 injected kills, timeouts and invalid runtimes, 0 surviving workers or copies, and p95 drift 0.9995. Two earlier runs under heavy foreign load failed the drift criterion (1.32 and 1.50) and are kept.
- **Linux.** The native job passed with pinned, checksum-verified artifacts.
- **Not covered.** Windows and Intel. The independent answer-quality gate remains 0/50.
