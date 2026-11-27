# Local model process ownership

This API was added in **AIRust 0.6.2**. The published 0.6.1 package does not contain it.

`ProcessReranker` implements the existing `Reranker` interface using one application-owned worker process. The worker reuses the validated `FastEmbedReranker`, pinned MiniLM weights, CPU execution, tokenization, 512-token recipe and score checks. This moves native allocation lifetimes outside the application process. It does not add a search algorithm, model framework, database or mandatory default dependency.

## Why a process boundary

Repeated 0.6.1 model construction/drop failed its post-warm RSS-growth control. The new thirty-cycle reproduction and three allocation-policy ablations are retained with the [measurement record](measurements/native-lifecycle-0.6.2/README.md). Native memory can be freed logically while the system allocator keeps fragmented or cached resident regions. Destroying Rust values does not itself require that allocator to return all pages to the operating system. Zero unreachable allocations reported by a restricted diagnostic tool is not an absence-of-leaks proof.

The ordinary in-process constructor remains compatible and still has that resource boundary. Reusing one instance can avoid reload work, but does not turn a failed repeated-load test into a pass. For applications that must unload/reload models predictably, the explicit process mode ends the native heap lifetime by ending and reaping its process. The library does not set a global allocator environment variable or call unsafe allocator hooks.

## Application integration

Build the matching trusted worker from this checkout:

```bash
cargo build --release --locked --no-default-features \
  --features local-reranking --bin airust-model-worker
```

```rust,no_run
use airust::reranking::{Reranker, process::{ProcessOptions, ProcessReranker}};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let options = ProcessOptions::new("/absolute/path/to/airust-model-worker");
let mut scorer = ProcessReranker::multilingual_minilm_v2_from_snapshot(
    options, "/owned/model-directory", "/owned/runtime-library", 2, "/owned/private-snapshot-parent",
)?;
let scores = scorer.score("Rust ownership", &["Rust owns its values."])?;
assert_eq!(scores.len(), 1);
drop(scorer);
# Ok(()) }
```

Verified file-snapshot loading is required and takes an application-owned snapshot parent directory. The parent creates a private subdirectory and retains its guard until the worker has exited. The worker streams/checks the existing model into a private file using the existing snapshot API. Normal native drop removes the copy; parent cleanup also removes its private subtree after a worker crash/timeout. Original models, workspaces and databases are not deleted. Cleanup applies only to that private subtree.

Snapshot deletion uses temporary-file guards; filesystem cleanup errors or abrupt application termination can leave private copies. Applications should account for disk space and inspect their own snapshot parent.

The bundled worker and library must have the same package version. The startup handshake verifies version, PID and a bounded nonempty recipe. The internal protocol is not a stable third-party interoperability API. Supply a trusted executable implementing this role; it must not spawn descendants that inherit the pipes. The worker starts with a reduced environment: only `PATH`, `TMPDIR`, `TMP`, `TEMP`, `LANG`, `LC_ALL`, `LC_CTYPE`, `SYSTEMROOT` and `WINDIR` are passed when present, so secrets such as `AIRUST_API_TOKEN` are not inherited (unreleased, after 0.6.2). `ProcessOptions::with_runtime_identity(bytes, sha256)` optionally pins the ONNX Runtime library: its size and SHA-256 are verified while it is copied into the private snapshot directory, and the worker loads that verified copy. Without it, runtime provenance remains an application responsibility. Model provenance is always verified against the packaged manifest; file directories remain application responsibilities. This is resource isolation, **not a security sandbox**.

The reference pilot can use its own executable as the worker. With `--features pilot,local-reranking`, add `"isolated": true` and an explicit `model_snapshot_directory` to the existing `reranker` profile object. Missing snapshot configuration is rejected before worker/model work. Missing/false preserves the existing in-process behavior. `profile-check` remains offline: it reports configuration and never starts a worker or verifies assets. Model/runtime paths are startup configuration, not MCP tool arguments.

The rejected memory-backed worker prototype passed its RSS-growth control but exceeded the declared warm p95 cost budget by about 23% in the primary cohort. It is retained as a negative result and was removed from the new public API. The supported worker constructor always loads the verified private file snapshot. Disk space of at least 470883696 bytes plus filesystem overhead is required per active worker.

## Bounded parallel workers (unreleased, after 0.6.2)

`ProcessPool` admits several `ProcessReranker` workers under one explicit limit. At most `max_workers` processes exist at once; at most `queue_limit` callers wait. A caller beyond the queue limit, or one whose wait expires, receives `RerankError::Busy` immediately instead of starting another process. With the `execution` feature, `acquire_controlled` also honors a `WorkControl` deadline and cancellation token. A lease whose scoring fails retires its worker; the next caller gets a new one, and the limit is never exceeded. Healthy leases return their worker for reuse. Dropping the pool and all leases reaps every worker.

```rust,no_run
use airust::reranking::{Reranker, process::{ProcessOptions, ProcessPool}};
use std::time::Duration;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let pool = ProcessPool::multilingual_minilm_v2_from_snapshot(
    ProcessOptions::new("/absolute/path/to/airust-model-worker"),
    "/owned/model-directory", "/owned/runtime-library", 2, "/owned/private-snapshot-parent",
    2, 8,
)?;
let mut lease = pool.acquire(Duration::from_secs(5))?;
let scores = lease.score("Rust ownership", &["Rust owns its values."])?;
assert_eq!(scores.len(), 1);
# Ok(()) }
```

Each worker holds its own model copy in memory and on disk, so the limit is also the memory and disk bound. The pool is optional, lives behind `local-reranking` and adds no dependency.

## Bounds and failure behavior

| Resource | Bound |
| --- | ---: |
| Native workers / I/O threads per scorer | 1 / 1 |
| Pending command / response slots | 1 / 1 |
| Input query / individual document | 16 KiB / 64 KiB |
| Documents / total input text per scoring call | 128 / 2 MiB |
| Request frame / response frame, including newline | 16 MiB / 64 KiB |
| Returned native-provider diagnostic | 2048 UTF-8 bytes |
| Worker CPU-thread setting | 1 through 64 |
| Default initialization / scoring timeout | 60 / 30 seconds |
| Configurable timeout | positive, at most 120 seconds |
| Normal-exit grace before kill/reap | 1 second |

The input/frame limits bound individual transport allocations. The application allocator can retain its own buffer high-water mark; they are not a guarantee of a fixed total parent RSS under every allowed payload.

Byte/count checks happen before cloning and serialization. JSON escaping is covered by the separate frame bound. Scores cross the pipe as integer IEEE-754 bit patterns and are reconstructed/checked before publication; their precision does not depend on decimal JSON float parsing. Responses must match the outstanding request ID and score count. Invalid identity, shape or non-finite values fail closed. Explicit input rejection does not perform native work. Initialization errors return no scorer.

`ProcessOptions::with_timeouts` configures initialization and one native scoring call. A timed-out, crashed or malformed worker is stopped; subsequent calls on that scorer fail. Recreate it explicitly. There is no retry storm, hidden process cache or automatic model reload on a search request. A reference server still constructs one configured scorer and reuses it for requests.

Dropping a healthy scorer closes its request channel and stdin, permits normal teardown, and waits for the child. If the child does not exit during the grace period it is killed and reaped before the parent-owned snapshot guard is released. Timeout/error paths kill before joining blocked pipe work. Caller-owned binaries must obey the no-descendants/pipes contract; OS-level uninterruptible I/O and process termination are outside a hard wall-clock guarantee.

Existing `WorkControl` cancellation/deadline checks still occur between native batches. This interface does not forward an arbitrary cancellation token into an executing ONNX call. The worker call timeout bounds a separate failure path; it is not a promise that every shorter MCP deadline preempts native work immediately. Input bounds do not impose a whole-process RAM/CPU quota. Multiple scorer instances mean multiple workers and need application admission limits.

## Validation and remaining scope

Three matched thirty-cycle selected file-worker cohorts observed 0.328, 0.406 and 0.375 MiB post-warm after-drop parent RSS ranges, with no surviving model children. The small lifecycle test parent remained about 4.1–4.2 MiB after unload; that is not a full application footprint. A serial warm-p95 cost confirmation failed at ratio 1.213; an alternating diagnostic passed at 0.986 and does not erase it. Loading was slower than direct memory loading at p95 in all three selected comparisons. Read the complete raw evidence before choosing an operating policy.

Run the real-asset lifecycle example with the public controller:

```bash
cargo build --release --locked --features local-reranking,execution \
  --example native_model_lifecycle
python3 -B scripts/benchmark_model_lifecycle.py \
  "${CARGO_TARGET_DIR:-target}/release/examples/native_model_lifecycle" /tmp/new-lifecycle-results \
  --model-directory /owned/model-directory --runtime /owned/runtime-library \
  --backend process --mode snapshot --cycles 30
```

The controller refuses an existing result directory, uses a frozen synthetic resource fixture by default, records all events and watches its own process tree. It separates parent RSS after drop from sampled parent-plus-child RSS while active; summing RSS can double-count shared pages. It checks native cancellation after one actual batch, an expired deadline with no added work, bit-exact recovery, zero surviving children after drop, private-copy cleanup and failed initialization. Original replay controls remain private and are not independent human validation.

The [measurement record](measurements/native-lifecycle-0.6.2/README.md) governs actual counts, pass/failure status, startup/inference costs and every regression. Active native-model memory remains separate from unloaded parent memory; this does not certify an actual 1-GiB laptop. Actual E5 inference assets were unavailable, and this process adapter currently implements the pinned MiniLM reranker only. Windows/Intel native execution is not established by macOS ARM64 measurements. Native ONNX and OS subprocess execution remain outside the finite Miri protocol/asset scope. Protocol fuzzing does not execute arbitrary native models. No independent question-quality or customer-adoption gate is closed here.
