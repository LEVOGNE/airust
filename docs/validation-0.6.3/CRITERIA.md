# AIRust 0.6.3 operational robustness: release criteria

These criteria were fixed on 2026-11-27, **before** any 0.6.3 test was run. A failed criterion stays failed and is documented. Thresholds are not changed after results are known. Any change to this file after the first counted run must be recorded as a dated amendment. That amendment cannot apply to runs that already happened.

Scope: the optional process worker (`local-reranking`). The public API stays compatible. New capabilities are additive and optional, and nothing new enters the default feature set or the core crate graph.

## Release condition

After several hours of load with repeated injected worker failures, AIRust either keeps working within the resource and error limits below, or it stops in a controlled way with a typed error. A panic, a hang beyond a declared bound, a surviving worker, a leftover private copy or a wrong score fails the release.

## R1 — worker soak with fault injection

- **Workload.** At least **3 hours** of wall-clock time with the real pinned MiniLMv2 model and ONNX Runtime 1.32.0, using the frozen synthetic resource fixture `tests/fixtures/native-lifecycle-v1.json`, `batch_size(1)` and 2 threads.
- **Fault schedule.** The schedule is deterministic and repeats every 10 cycles:
  - cycle `n % 10 == 3`: the worker is killed (SIGKILL) between two requests;
  - `n % 10 == 6`: a request runs with a 1 ms request timeout;
  - `n % 10 == 8`: startup runs with an invalid runtime path;
  - all other cycles run normally.
- **Pass conditions, all required:**
  1. Every normal cycle returns IDs, order and score bits identical to the per-case oracle recorded in the first normal cycle of that case.
  2. Every injected fault returns a typed `RerankError` within its timeout plus 2 s. A panic or a hang is a failure.
  3. After every drop, including faulted ones, no worker PID of that cycle survives, there are no zombie children, and the private snapshot parent is empty.
  4. The post-warm parent RSS range is at most **64 MiB** over the whole run, which is the existing lifecycle budget.
  5. The parent's open-file-descriptor count ranges by at most **8** after warm-up.
  6. The normal-cycle first-request p95 of the last hour is at most **1.10×** that of the first hour, with no drift beyond 10 %.
  7. The host load monitor is recorded. Foreign load is reported, but it does not turn a failure into a pass.

## R2 — bounded parallelism (optional pool)

- **API.** A new optional `ProcessPool` behind `local-reranking` owns at most `max_workers` `ProcessReranker` instances and a bounded wait queue. Callers acquire a worker with a deadline. When the queue is full, the call returns the new typed error `RerankError::Busy` immediately. An expired deadline also returns `Busy`, and a cancellation returns the existing execution error. Existing types and constructors are unchanged.
- **Pass conditions:**
  1. Under 16 concurrent callers with `max_workers = 2` and queue limit 4, the sampled live worker count never exceeds 2. Callers beyond the queue limit get `Busy` within 50 ms.
  2. No result is lost or duplicated. Every successful result equals the single-worker oracle bits.
  3. Killing a pooled worker during load is reported to the affected caller as a typed error. The pool replaces the worker and never exceeds `max_workers`.
  4. Dropping the pool reaps all workers and leaves the snapshot parent empty within 5 s.
  5. Throughput is measured and reported, not gated. Two workers versus one, requests per minute and per-request p95, with raw samples kept.

## R3 — Linux native validation

- **Job.** A separate CI job, not part of the normal build, fetches the pinned model files and ONNX Runtime by exact URL. It verifies size and SHA-256 against the packaged manifest or the recorded runtime identity, caches them by checksum, and runs the native worker tests and a shortened R1 soak (at least 50 cycles with the same fault schedule) on Linux x86_64.
- **Rules.** Normal builds and tests download nothing. If an artifact does not verify, the job fails; it never substitutes another artifact.
- **Pass condition.** The job passes on the release commit. An unavailable job is reported as unavailable, not as passed.

## R4 — evidence rules

- Every run keeps its raw per-cycle data, the fault log, the monitor log, binary and source hashes, and the exact commands.
- Percentiles use nearest rank, `sorted[ceil(n*p)-1]`. No warm-up is removed beyond what is declared above.
- Results reached outside these definitions are reported separately and labelled. They never replace the declared result.
- Windows and Intel execution are not covered by 0.6.3 unless a real host or runner becomes available. Their absence is stated.

## Separate work package (not a 0.6.3 gate)

- **Answer quality with a local LLM.** The 150 authored reference questions, no retrieval versus BM25 versus embeddings versus hybrid with reranking, Qwen3.8:27b via local Ollama, with deterministic quote, citation and abstention checks. It runs after the robustness work and does not block it.
- **Independently reviewed questions.** A reliable statement about answer quality requires the independently reviewed questions, currently 0 of 50. Authored-question results are labelled as such.

## Amendment 1 (2026-11-27, before any counted run)

Criteria R1.4 and R1.5 did not define the warm-up. They use the existing lifecycle definition: every cycle except the first two, including fault cycles. R1.2 bounds are the timeout of the faulted operation plus 2 s: kill uses the default 30 s request timeout (32 s), the timeout fault 1 ms (2.001 s), and invalid runtime the default 60 s startup timeout (62 s).
