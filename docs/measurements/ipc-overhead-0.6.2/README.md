# Steady-state process-boundary cost of `ProcessReranker` (0.6.2 development)

Measured on 2026-11-26 during an unattended maintenance session. This record adds evidence about the failed serial warm-recovery confirmation (ratio 1.213) in [native-lifecycle-0.6.2](../native-lifecycle-0.6.2/README.md). It does not replace, average away or reinterpret that failure.

## Question and design

How much does the process boundary itself cost per request once both scorers are warm, and does buffering the worker's stdin reduce it?

- One parent process loads the direct in-process scorer and a `ProcessReranker` once each, both from verified snapshots of the same MiniLMv2 weights with ONNX Runtime 1.32.0, two intra-op threads each.
- Workload: the frozen synthetic resource fixture `tests/fixtures/native-lifecycle-v1.json` (4 cases x 20 candidates), `batch_size(1)`, so each rerank makes 20 provider calls. These are synthetic resource cases, not user questions.
- Each round scores the same case with both scorers in ABBA order (even rounds direct first). Score bits had to match in every round; all 1,920 counted rounds matched.
- Blocks of 240 rounds alternate the baseline binary (source `9e792bc`) and a candidate (same source plus a 64 KiB `BufWriter` around the worker stdin). Every round is kept; a 4-round smoke run before the protocol was frozen is retained but not counted.
- Estimator: nearest rank, `sorted[ceil(n*p)-1]`. Declared per-block gate: per-case p95 worker/direct ratio at most 1.10.
- Claim rule frozen before execution: a candidate reduction is stated only if its paired median overhead is lower in every base/cand pair.

Method files: [policy](policy.json), [set-2 amendment](policy-amendment-set-2.json), [decision](decision.json), [probe source](ipc_overhead_probe.rs.txt), [analysis](analyze.py.txt), run scripts per set, [checksums](SHA256SUMS). Host: Apple M5 Max, 64 GiB, macOS, not isolated from interactive use.

## Results (all blocks)

Paired difference = worker total minus direct total for the same case in the same round, per 20-call rerank.

| Set / block | Median paired diff ms | p95 paired diff ms | Rounds worker slower / 240 | Per-case p95 ratio worker/direct (cases 0-3) | Gate <= 1.10 |
| --- | --- | --- | --- | --- | --- |
| 1 / base | 2.373 | 9.303 | 174 | 1.014, 1.006, 1.021, 1.004 | Passed |
| 1 / cand | 1.807 | 219.858 | 149 | 0.583, 1.286, 0.794, 1.255 | **Failed** |
| 1 / base | 2.018 | 10.812 | 182 | 1.031, 1.004, 1.005, 1.009 | Passed |
| 1 / cand | 1.300 | 143.074 | 143 | 0.681, 1.144, 0.568, 1.305 | **Failed** |
| 2 / cand | 1.660 | 19.730 | 175 | 0.873, 1.240, 1.002, 0.921 | **Failed** |
| 2 / base | 1.636 | 77.875 | 165 | 0.645, 1.371, 0.651, 1.477 | **Failed** |
| 2 / cand | 3.217 | 17.441 | 195 | 1.002, 1.013, 1.020, 0.993 | Passed |
| 2 / base | 1.871 | 8.428 | 173 | 1.022, 0.997, 1.008, 0.999 | Passed |

Raw per-round CSVs: [set 1](set-1/), [set 2](set-2/).

## Findings

1. **The steady-state boundary cost is small.** Across all eight blocks the median paired overhead is 1.3-3.2 ms per 20-call rerank (about 0.07-0.16 ms per call, roughly 0.3-0.8 % of a 90-480 ms rerank). In quiet blocks the per-case p95 ratio stays at or below 1.031. This steady-state cost does not explain a 21 % warm-recovery p95 difference.
2. **The 1.10 p95 gate is sensitive to host disturbances.** Four of eight blocks failed it, including one block of the *unchanged baseline binary* (1.477). Failing blocks show bursts that slow both scorers in the same rounds (for example set 1 candidate block, first rounds of case 1 up to 1.7-2.3 s for both). An interactive session was active on the host during set 1 block 2 (17:48-17:51 UTC). This supports, but does not prove, an environmental explanation for the earlier 1.213 failure; that failure remains recorded.
3. **The buffered stdin candidate is rejected.** Its median was lower in both set-1 pairs (1.807 vs 2.373, 1.300 vs 2.018 ms) but not in set 2 (1.660 vs 1.636, 3.217 vs 1.871 ms). Under the frozen rule no improvement is claimed, and the source change was reverted; `src/reranking/process/transport.rs` equals commit `9e792bc`.

## Set 3: unchanged baseline on an unattended host (2026-11-26, 21:56-22:08 UTC)

The [amendment](policy-amendment-set-3.json) was frozen before the run. Four consecutive blocks ran with `probe-base` only. The rejected buffered-stdin candidate was deliberately not re-tested. The rule: "the gate holds on an unattended host" may only be written if all four blocks pass 1.10.

| Block | Median paired diff ms | p95 paired diff ms | Rounds worker slower / 240 | Per-case p95 ratio worker/direct (cases 0-3) | Gate <= 1.10 |
| --- | --- | --- | --- | --- | --- |
| 3 / base 1 | 1.389 | 59.921 | 153 | 0.642, 1.162, 0.661, 1.317 | **Failed** |
| 3 / base 2 | 1.433 | 135.638 | 146 | 0.701, 1.097, 0.603, 1.182 | **Failed** |
| 3 / base 3 | 1.509 | 182.895 | 154 | 0.676, 1.175, 0.680, 1.390 | **Failed** |
| 3 / base 4 | 1.217 | 13.309 | 152 | 1.221, 1.449, 0.899, 1.827 | **Failed** |

**Result under the frozen rule:** the per-case p95 gate failed in all four blocks, so it does not hold for the unchanged baseline binary on this host.

- The median paired overhead stays at 1.2 to 1.5 ms per 20-call rerank. That is consistent with sets 1 and 2.
- The per-case ratios fall on both sides of 1 (0.60 to 1.83). Direct runs are often the slower ones. This fits disturbance bursts that hit whichever scorer is running at the time, rather than a one-sided worker cost. It is an observation, not a measured cause.

**Documented foreign load.** After set 3, a process check found a Miri test process of an unrelated project on the same host. It belonged to another Claude Code session, not to this work, and ran at about 96 % of one core. It had been started at 2026-11-26 00:17:49 UTC (see [foreign-load-observed.txt](foreign-load-observed.txt)), so it ran during **every** 2026-11-26 measurement in this record and in the lifecycle repeats. Transient compiler processes from that session were also seen. The process was not stopped, because it is not owned by this work.

This load is not instrumented per round, and its effect is not quantified. All 2026-11-26 measurements should therefore be read as taken on a loaded host. They do not affect the original 2026-11-25 failure at 1.213, whose run predates this process.

## Limits

Single host, macOS ARM64, synthetic fixture, two threads, warm models only. Not a cold-start, load-time, memory or lifecycle measurement. Disturbance sources were not instrumented, so the burst attribution is an observation, not a measured cause. No speed claim for the process mode follows from this record.
