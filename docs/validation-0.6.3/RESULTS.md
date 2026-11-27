# AIRust 0.6.3 operational robustness: results so far

Status on 2026-11-27, during development. **0.6.3 is not released.** The [criteria](CRITERIA.md) were frozen before these runs, and amendment 1 was also recorded before any counted run. Evidence: [evidence-2026-11-27](evidence-2026-11-27/) with [checksums](evidence-2026-11-27/SHA256SUMS).

## R1 — three-hour worker soak with fault injection: **failed (R1.6)**

The run lasted 15:30:03 to 18:30:04 UTC with the real pinned MiniLMv2 model, ONNX Runtime 1.32.0, the synthetic fixture, batch size 1 and 2 threads. It covered 3897 cycles: 2728 normal, 390 kills, 390 1-ms timeouts and 389 invalid-runtime startups. Source: the working tree on top of `0eceabc`, binary hash in [binaries.sha256](evidence-2026-11-27/binaries.sha256).

| Criterion | Result |
| --- | --- |
| R1.1 every normal cycle bit-identical to its oracle | **Passed** (2728/2728) |
| R1.2 every fault typed and within its bound | **Passed** (1169/1169; observed fault latency about 10–15 ms) |
| R1.3 no surviving worker, zombie or private copy | **Passed** (0 leftovers in 3897 cycles) |
| R1.4 post-warm parent RSS range at most 64 MiB | **Passed** (0.81 MiB) |
| R1.5 descriptor range at most 8 | **Passed** (0) |
| R1.6 last-hour first-request p95 at most 1.10 × first hour | **Failed**: 747.7 ms against 566.5 ms, ratio 1.32 |
| Duration at least 3 h | Passed (3.0002 h) |

The host was not quiet. The user ran other work, and the monitor recorded foreign compilers, a language server and a browser throughout. Load by hour:

| Hour | Load median / p95 | First request median / p95 ms |
| --- | --- | --- |
| 1 | 8.1 / 15.5 | 389.2 / 566.5 |
| 2 | 8.1 / 18.2 | 389.4 / 542.8 |
| 3 | 9.9 / 19.3 | 399.4 / 747.7 |

The median stayed within 3 %, and memory and descriptors stayed flat. The tail grew in the hour with the highest foreign load. This is consistent with host contention rather than a leak, but it is not a measured cause. Under the frozen rule, R1 is **failed**. A repeat on a quiet host would be additional evidence. It would not replace this result.

A 60-second smoke run before the counted run is retained as [r1-smoke-not-counted.jsonl](evidence-2026-11-27/r1-smoke-not-counted.jsonl) and is not counted.

## R2 — bounded parallelism

- **R2.1 to R2.4 (mock workers, unit test).** 16 concurrent callers ran with `max_workers = 2` and queue limit 4. Live workers never exceeded 2, and overflow callers received `Busy` within 50 ms. A killed pooled worker produced a typed error and was replaced, and dropping the pool left the snapshot parent empty. The test `pool_bounds_live_workers_rejects_overflow_and_replaces_failed_workers` passes.
- **R2.5 (real model, reported, not gated).** 4 callers made 10 requests each, under the same foreign load ([raw](evidence-2026-11-27/r2-throughput.jsonl)). All 80 results were bit-identical to the single-worker oracle, and the peak live workers stayed within the limit.

| Workers | Requests/min | p50 ms | p95 ms (includes queue wait and worker startup) |
| --- | --- | --- | --- |
| 1 | 154.1 | 410.4 | 5851.5 |
| 2 | 242.0 | 403.8 | 3828.5 |

The throughput binary was built before a `cargo fmt` pass that changed only formatting in its source.

## R3 — Linux native job: **prepared, not run**

The `native-worker-linux` job is in `.github/workflows/ci.yml`. It pins and verifies the model manifest and the ONNX Runtime 1.32.0 manylinux wheel (SHA-256 `99e1074c…`), and runs a 15-minute shortened soak evaluated with `--short`. It has not run, because nothing was pushed during the unattended session. Until it runs, it counts as **unavailable**.

## Not yet done

- A repeat of R1 on a quiet host. The 1.32 drift remains failed regardless.
- Running R3 on GitHub.
- Windows and Intel remain uncovered.
