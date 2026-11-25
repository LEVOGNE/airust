# AIRust 0.6.1: final-source confirmation

This is an **unpublished candidate**, not a registry release. The separate confirmation finished at **2026-11-25T07:16:36.178208+00:00**. Earlier [primary cohorts, failures and figures](../profile-practical-0.6.0/README.md) remain unchanged; their samples are not assigned to these binaries.

The accepted Rust/manifest/input files and method hashes match the frozen source before/after execution. Compatible baseline/candidate release builds use Rust 1.98.0, thin LTO and one codegen unit on the recorded M5 Max / 64-GiB Mac. Source archive, binaries, methods, lock and asset identities are in the Git-only [evidence manifest](evidence.json). The original database and private laboratory evidence are explicitly excluded from that source archive and retained separately.

Artifacts were precompiled during the remaining fuzz time. The gated controller rechecked/build-validated and copied them after fuzz completion, before timing; this is compiler-cache reuse, not a fresh test run. An initial archive-selection guard stopped before any archive/build/timing because the original database is historically tracked. The failed attempt remains retained; the corrected selection leaves the database untouched. No earlier timing sample was discarded.

| Program | SHA-256 | Bytes |
| --- | --- | --- |
| profile | 8aab6c303769875de7f0305be268771fa95e595d8c757f282caaa67631052c70 | 10343792 |
| small-host | da8cce2055a99258ffd7e4f879d08e74df6657cba8711050e47778837bd26e34 | 3892448 |
| training | 1fe9049cf7220f3be32caddd3b7f39d3040aa971fa53009147c27839091c609a | 550272 |

## Complete MCP requests

Two campaigns reverse build order. Each worker executes one initial request and one timed request per unchanged assistant-authored reference case: **600 timed + 600 initial requests** across four workers. Baseline is exact published source 8c098f15a39f6085f748c56ac52e5737abdff423, 2 threads / batch16 / memory model. Candidate is 8 threads / batch1 / verified file snapshot. Model, BGE-M3 digest, source/vector cache, twenty-candidate prefix and maximum pair length 512 remain fixed. Extra CPU threads are a resource tradeoff, not measured energy efficiency.

| Campaign | Baseline p95 ms | Candidate p95 ms | Ratio | Baseline peak MB | Candidate peak MB |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1 | 895.581 | 229.610 | 0.256 | 3394.879 | 1428.931 |
| 2 | 900.440 | 228.034 | 0.253 | 3395.092 | 1429.012 |

The predeclared repeated 20% p95-gain gate is **passed**. All top-six IDs/order and non-logit response fields match; no known-quote loss is observed. This confirmation has no slower per-query mean; the earlier primary/checkpoint regressions remain retained separately. All 1130 nonzero logit changes are retained under absolute 1e-4 + relative 1e-5 tolerance. This does not establish independent relevance/answer quality.

Request timing covers MCP write through decoded reply, including embedding roundtrip and native scoring. Startup, each initial query, worker peak RSS and external-service/answer costs are separate. All workers run sequentially after compilation/Miri/fuzz finish. Process startup is not a whole-machine cold-cache measurement.

## Training snapshots

Sixteen final-source workers cover two campaigns, four scenarios and both builds. All framed value signatures match; the candidate read-only 128-MiB observation budget passes. There are only two observations per cell here, so this confirmation does not establish a stable tail-latency percentile. The primary 80-worker campaign remains separate.

| Scenario | Baseline worst peak MB | Candidate worst peak MB |
| --- | ---: | ---: |
| readonly | 465.994 | 20.988 |
| one_edit | 465.961 | 39.469 |
| all_edits | 169.279 | 169.296 |
| small | 7.520 | 2.523 |

Shared readers avoid repeated full copies. First writes detach the whole shared collection; every independent writer eventually owns a copy. Higher isolated first-edit costs, the all-writer case and tiny timing/memory regressions remain in the raw CSV. This changes the training container, not the search-index algorithm.

## Model-free host

Eight workers execute **8000 timed + 256 warmup requests**, with additional oversized-query rejection/recovery controls. Each generated dataset has 2048 entries; short-unique adds 393216 distinct four-character terms inside the unchanged 2-MiB ID/text/metadata data budget. The JSON file has separate 4-MiB framing space. These are generated resource probes, not customer questions or quality labels.

| Workload | Startup range ms | Warm p95 range ms | Worst peak MB | Worst sampled post-warm growth bytes | Initial request range ms |
| --- | ---: | ---: | ---: | ---: | ---: |
| unicode | 33.230–67.894 | 2.052–2.103 | 16.613 | 114688 | 1.242–1.357 |
| high-vocabulary | 29.027–30.466 | 1.086–1.224 | 111.280 | 65536 | 8.788–8.964 |
| long-token | 44.959–57.370 | 1.872–1.906 | 17.187 | 49152 | 1.200–1.265 |
| short-unique | 54.102–54.407 | 1.162–1.212 | 212.074 | 32768 | 15.188–16.825 |

All 512-MiB worker-peak and 32-MiB observed-growth budgets pass. [Resource limits](../../RESOURCE-LIMITS.md) explain input/admission/thread/output boundaries. An earlier primary Unicode worker took 513.559 ms to start, followed by a 1.171-ms first query; both remain retained. Different startup values here do not prove that earlier outlier was fixed or guarantee cold-device latency. Input byte limits are not total-heap quotas. Actual 1-GiB hardware, OS overhead and independent reproduction remain unestablished.

## Optional macOS allocator request cost

The same final binary and 2-thread/batch1/snapshot profile are used for four workers: default launch (flag cleared) versus child-only MallocSpaceEfficient=1, reversed order in campaign two. Each has 150 initial + 150 timed requests. No parent/global allocator/environment change occurs.

| Campaign | Default p95 ms | Optional policy p95 ms | Ratio |
| --- | ---: | ---: | ---: |
| 1 | 353.701 | 347.734 | 0.983 |
| 2 | 353.609 | 352.535 | 0.997 |

The optional predeclared maximum 10% repeated p95-overhead gate **passed**. All 159 slower per-query means remain exported; no returned logit changed in this same-profile comparison. Worker peak was about 1.256 GB with the optional policy versus 1.429 GB with the default launch. This request experiment does not replace the earlier repeated-drop lifecycle controls: both default-allocator growth failures remain failed, while the separate optional lifecycle control passed. It remains an OS-specific application-launch experiment and never an automatic library default or active-model 1-GiB guarantee. These two-thread results must not be combined with the separate eight-thread latency as an unmeasured configuration.

## Raw evidence and remaining boundaries

CSV exports contain allowlisted timings/numbers/public IDs/hashes; no private conversation questions, answers, excerpts, service inventory or logs. Verify every CSV with [SHA256SUMS](SHA256SUMS). Git attributes preserve exact exported line endings, and package controls verify those bytes after extraction. Detailed JSON is retained in Git and excluded from crates by the existing packaging policy. Power observations are reduced to AC/battery availability; raw device identifiers remain private. Public worker tables include startup/initial values and exact-child peak; raw sampled RSS is separate.

- [allocator-request-cost-logit-differences.csv](allocator-request-cost-logit-differences.csv)
- [allocator-request-cost-samples.csv](allocator-request-cost-samples.csv)
- [allocator-request-cost-slower-queries.csv](allocator-request-cost-slower-queries.csv)
- [allocator-request-cost-workers.csv](allocator-request-cost-workers.csv)
- [paired-mcp-logit-differences.csv](paired-mcp-logit-differences.csv)
- [paired-mcp-samples.csv](paired-mcp-samples.csv)
- [paired-mcp-slower-queries.csv](paired-mcp-slower-queries.csv)
- [paired-mcp-workers.csv](paired-mcp-workers.csv)
- [small-host-rss.csv](small-host-rss.csv)
- [small-host-samples.csv](small-host-samples.csv)
- [small-host-workers.csv](small-host-workers.csv)
- [training-snapshot-confirmation.csv](training-snapshot-confirmation.csv)

The actual local-LLM pilot still includes a quote mismatch and an incomplete historical-roadmap answer. The 150 authored cases and generated probes are not independently reviewed real traffic; zero eligible human-reviewed cases are established. Native FFI remains outside Miri; PDF fuzzing is coverage-only. File snapshots cost 470883696 bytes of disk per loaded model plus filesystem overhead and can remain after abrupt termination. First writes, native allocator retention, unmeasured cross-host/system/GPU costs and unsupported/unavailable compiler/platform scopes are not erased by these results.

See [candidate validation](../../validation-0.6.1/README.md), [reproduction](../../validation-0.6.1/REPRODUCE.md) and [the session handoff](../../SESSION-HANDOFF.md) for current source/package/CI closeout. No 0.6.1 registry release, master merge or tag is implied.
