# AIRust 0.6.3 operational robustness: results

Status on 2026-11-28. R1 passed in the third run, R2 and R3 passed; the two earlier R1 failures remain recorded below. The [criteria](CRITERIA.md) were frozen before these runs, and amendment 1 was also recorded before any counted run. Evidence: [evidence-2026-11-27](evidence-2026-11-27/) with [checksums](evidence-2026-11-27/SHA256SUMS).

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

## R1 repeat — second three-hour run: **failed (R1.6)**

The run lasted 2026-11-27 20:22:51 to 23:22:53 UTC with the same binary (`884f9068…`, identical to the first run) and source `3a2660e`. The [addendum](evidence-2026-11-27/r1-repeat-policy-addendum.json) was frozen before the start: this run is reported on its own, and the first failure is neither replaced nor averaged. It covered 4202 cycles: 2942 normal and 420 of each fault.

- R1.1 to R1.5 **passed**: all results bit-identical, all 1260 faults typed and within bound, 0 leftovers, RSS range 0.69 MiB, descriptor range 0.
- R1.6 **failed**: last-hour p95 733.8 ms against 487.8 ms in the first hour, ratio 1.50.

| Interval | Load median | First-request median / p95 ms |
| --- | --- | --- |
| Hour 1 | 4.5 | 389.1 / 487.8 |
| Hour 2 | 3.1 | 385.8 / 461.9 |
| Hour 3 | 12.4 (foreign compilers and an application started) | 390.0 / 733.8 |

15-minute p95 values were 485, 493, 488, 460, 460, 459, 460 and 468 ms while load was low. They rose to 648, 821, 627 and 877 ms once foreign load rose, while the medians stayed between 385 and 391 ms.

Across both runs the tail grows only when foreign load rises, and memory, descriptors and medians stay flat. That strongly suggests host contention rather than degradation of the worker, but the criterion as frozen (last hour against first hour on the same host) is still **failed twice**. **0.6.3 is therefore not released.** A defensible pass needs either a host without foreign load for 3 hours, or a criterion revised by the user that accounts for host load, applied only to future runs.

## R1 third run — host quiet at start: **passed**

The run lasted 2026-11-28 04:50:00 to 07:50:02 UTC with the same binary (`884f9068…`) and source `3a2660e`. The [addendum](evidence-2026-11-27/r1-third-policy-addendum.json) was frozen before the run. The run started only after 10 consecutive minutes without `rustc`, `cargo` or `clang` and with load below 3 ([start script](evidence-2026-11-27/r1-third-wait-for-quiet.sh.txt), [wait log](evidence-2026-11-27/r1-third-wait.txt)).

The start gate's rule on `rust-analyzer` was relaxed before the start, from "absent" to "each below 5 % CPU", because five idle language servers (0 % CPU) blocked the original gate. This is recorded in the addendum. While a second, duplicate starter with the original gate ran by mistake, its log lines interleave in the wait log. It never started a run and was stopped at 04:34 UTC.

The run covered 4529 cycles: 3170 normal and 453 of each fault.

| Criterion | Result |
| --- | --- |
| R1.1 bit-identical | **Passed** (3170/3170) |
| R1.2 faults typed and within bound | **Passed** (1359/1359) |
| R1.3 no survivors, zombies or copies | **Passed** (0 in 4529 cycles) |
| R1.4 RSS range | **Passed** (0.75 MiB) |
| R1.5 descriptor range | **Passed** (0) |
| R1.6 drift | **Passed**: last-hour p95 461.8 ms against 462.0 ms, ratio 0.9995 |
| Duration | Passed (3.0003 h) |

The 15-minute p95 values were 463, 461, 464, 461, 460, 460, 462, 462, 457, 462, 464 and 462 ms. Load median was 1.96 and maximum 5.22, and the foreign samples were mainly the window server.

**Overall R1 verdict:** R1 passed in the third run on a host that was quiet at start. The earlier runs failed R1.6 at 1.32 and 1.50, when foreign load rose during their last hour, and they remain recorded. All three runs passed R1.1 to R1.5. The evidence points to host contention as the cause of the earlier tail growth. The release statement is "passed when the host is not loaded by other heavy work", not "unaffected by host load".

## R2 — bounded parallelism

- **R2.1 to R2.4 (mock workers, unit test).** 16 concurrent callers ran with `max_workers = 2` and queue limit 4. Live workers never exceeded 2, and overflow callers received `Busy` within 50 ms. A killed pooled worker produced a typed error and was replaced, and dropping the pool left the snapshot parent empty. The test `pool_bounds_live_workers_rejects_overflow_and_replaces_failed_workers` passes.
- **R2.5 (real model, reported, not gated).** 4 callers made 10 requests each, under the same foreign load ([raw](evidence-2026-11-27/r2-throughput.jsonl)). All 80 results were bit-identical to the single-worker oracle, and the peak live workers stayed within the limit.

| Workers | Requests/min | p50 ms | p95 ms (includes queue wait and worker startup) |
| --- | --- | --- | --- |
| 1 | 154.1 | 410.4 | 5851.5 |
| 2 | 242.0 | 403.8 | 3828.5 |

The throughput binary was built before a `cargo fmt` pass that changed only formatting in its source.

## R3 — Linux native job: **passed**

Hosted CI run `46057398768` on commit `3586682` passed all six jobs, including `native-worker-linux` (ubuntu-latest, x86_64). The pinned ONNX Runtime 1.32.0 manylinux wheel verified against its SHA-256 (`99e1074c…`), and the model files verified against the packaged manifest. The shortened 15-minute soak ran **278 cycles** with the same fault schedule:

- all normal cycles were bit-identical, and all faults were typed and within their bounds;
- there were no surviving workers, zombies or copies;
- the RSS range was 0.027 MiB and the descriptor range 0.

`--short` mode evaluates R1.1 to R1.5 and at least 50 cycles. Duration and hourly drift are not meaningful for a 15-minute run.

## Not covered

- Windows and Intel remain uncovered.
