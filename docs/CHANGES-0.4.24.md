# AIRust 0.4.24: bounded MCP reply preparation stays with its worker

MCP now constructs its complete tool result and checks its encoded size inside the admitted blocking worker. The worker keeps its permit and connection-tracker reservation through that work. Cooperative checks before and after final response preparation reject cancelled or expired work before a successful result escapes. The existing absolute request deadline and cancellation still take priority in the handler. Oversized final replies become worker errors before the final progress notification.

The defect was reproduced before the production change: a two-hit escaped payload passed the data-only budget, but its final duplicated/escaped tool result exceeded the bound. The old server announced `AIRust work completed` before returning `output_budget`. The same actual RPC test now receives `AIRust work stopped with an error`, a bounded error result, zero occupied workers afterward and successful later health. The original failing test log remains in the local record. This is a progress/lifetime correction, not a new error code or a relaxed byte limit.

The private immutable data/budget witness, same-budget reuse and owned envelope move remain. The complete result budget still includes both response representations and escaping. The command supplies its static operation name without a request-name copy. Successful tool-result wire bytes and existing error payloads remain compatible. No public API, tool argument, schema shape, permission, dependency or saved-format migration is added.

Core source is byte-for-byte unchanged from 0.4.23. Its repeated-word optimization, scores, scopes, source positions, cursors and checkpoints remain. Small-MCP repeated median changes across six cases and both rankers are **-5.07% to +10.13%** and **-1.49% to +2.88%** versus exact 0.4.23. Ordinary canonical TF-IDF changes are **-1.22% / -5.60%**. Every slower control remains; timing changes are observations, not proof of universal speedup or the cause of a historical regression.

## Executed validation and API contract

The full matrix passes **252 default tests, 908 all-feature checks including 39 documentation examples, 39 isolated feature entries and five Python tests**. Strict Clippy/docs, Rust 1.89 provider-neutral and Rust 1.92 all-feature checks pass. API comparison with 0.4.23 reports **229 pass, 31 skip**. An independent Rust 1.89 consumer exercises existing retrieval, source positions and MCP/schema construction. The unchanged PDF backend passes 14 regressions and its 381-source-file provenance check. The ignored crash-writer helper is deliberately launched by its parent recovery test. The refreshed dependency audit reports zero known vulnerabilities; maintenance notices remain in the resource record.

Three new regressions cover the real progress error and session recovery, pure before/after-finalization failure checkpoints with unchanged knowledge and legacy wire/budget results, and a deterministic admitted-worker boundary after final encoding. The latter holds work at that boundary, verifies busy admission, starts shutdown, verifies that shutdown still waits and new admission is closed, cancels/releases the worker and checks that its permit is freed with no successful partial result. Existing real deadline, hostile input, slow/stalled reader, cancellation, disconnect, per-connection drain and source-readback tests remain.

**Thirty-seven finite Tree Borrows Miri cases pass:** the previous 36 cases plus the new pure final-reply checkpoint/wire/budget case. Static operation mapping is also checked. This does not run Tokio I/O or the native shutdown/RPC tests inside Miri. Earlier large native boundaries and scaled Miri scopes remain. Only extra floating-point rounding injection is disabled; memory/alias checks remain. Tokio/PDF/FFI/ONNX execution is outside this interpreter scope. These checks are not independent security or soundness audits. Extracted-package tests, exact-commit CI, a fresh Rust 1.89 registry consumer, tag/VCS/checksum and release-asset agreement gate completion.

Engine, SearchOptions, Retriever, WorkControl, McpServer and McpLimits calls remain. Default features stay empty. Read [API](API.md), [cookbook](API-COOKBOOK.md), [MCP](MCP.md), [Core performance](CORE-PERFORMANCE.md) and [security scope](MCP-SECURITY.md).

## What the worker and deadline now cover

For admitted tools the blocking worker prepares both the textual JSON copy and structured result and performs the complete encoded-size check before it returns. Its permit/tracker remains held until that closure really exits, including cancelled work winding down. A final `output_budget` failure cannot be mistaken for successful work in its completion message.

This is cooperative cancellation, not forced termination inside an individual serialization call. The post-work checkpoint and the handler's original absolute deadline prevent an expired successful result, while cleanup can still take time. Startup loading/indexing, SDK JSON-RPC framing and final response writes remain separate from this tool-CPU boundary. Notification delivery keeps its existing absolute-deadline/stall rules. No hard OS CPU/RAM quota or end-to-end latency guarantee is added. `airust_status` stays a small, bounded asynchronous health path outside worker admission.

## Frozen method and development record

Baseline is exact registry 0.4.23, VCS `edec6b20e5f566b996201c2d0655899d106c7265`; reference is exact 0.4.22, VCS `0657dcdc637f7eb687129ea298556b90f7ec07b5`. Every cached registry file is compared byte-for-byte with its checksum-verified published archive before final timing. Actual worker sources/manifests/locks, binaries, runtime, functional schema inputs and data are hashed and preserved. Read the [method](validation-0.4.24/README.md).

The pre-fix failing RPC log, corrected native tests and private source identities remain. A separate three-triple development MCP pilot retains standard/extended-wide, canonical and tiny cases plus original-six Core controls, with complete compared responses. Previous Core preparation trials remain in the 0.4.23 archive; they are not new 0.4.24 trials. No development observation replaces a final sample.

## Actual MCP controls: primary small cases and retained large/broad work

Two separate six-case small-MCP campaigns cover 1/4/16 entries, each with/without progress. Each runs seven rotating exact 0.4.22/0.4.23/0.4.24 triples: 126 fresh servers, 8,568 schema-valid actual replies and 5,712 complete structured comparisons. Together: **252 / 17,136 / 11,424**. Discovery, full responses by RPC ID, requested/absent progress, hashes and EOF shutdown agree except checked status version and version-qualified schema URNs. Initial/repeated/query/ranker samples, process sums, startup and RSS stay separate. Complete small-Core controls retain 84 processes, 4,284,000 timed calls and 2,688 full original matrix-response comparisons; Core source is unchanged.

### Standard small-MCP campaign

| Case | Ranker | 0.4.22 repeated median / p95 ms | 0.4.23 repeated median / p95 ms | 0.4.24 repeated median / p95 ms | vs 0.4.23 |
| --- | --- | --- | --- | --- | --- |
| tiny | bm25 | 0.125 / 0.207 | 0.121 / 0.185 | 0.122 / 0.183 | +1.00% |
| tiny | tfidf | 0.128 / 0.237 | 0.119 / 0.215 | 0.123 / 0.226 | +3.01% |
| tiny4 | bm25 | 0.212 / 0.949 | 0.199 / 0.342 | 0.217 / 0.395 | +8.71% |
| tiny4 | tfidf | 0.206 / 1.469 | 0.196 / 0.339 | 0.216 / 0.370 | +10.13% |
| tiny16 | bm25 | 0.231 / 0.383 | 0.226 / 0.397 | 0.219 / 0.385 | -3.29% |
| tiny16 | tfidf | 0.229 / 0.406 | 0.231 / 0.411 | 0.220 / 0.422 | -5.07% |
| tiny-no-progress | bm25 | 0.083 / 0.137 | 0.077 / 0.149 | 0.076 / 0.118 | -0.30% |
| tiny-no-progress | tfidf | 0.079 / 0.166 | 0.075 / 0.170 | 0.074 / 0.174 | -1.16% |
| tiny4-no-progress | bm25 | 0.129 / 0.187 | 0.126 / 0.180 | 0.129 / 0.221 | +2.32% |
| tiny4-no-progress | tfidf | 0.124 / 0.161 | 0.124 / 0.177 | 0.128 / 0.203 | +3.25% |
| tiny16-no-progress | bm25 | 0.144 / 0.197 | 0.147 / 0.230 | 0.146 / 0.199 | -0.54% |
| tiny16-no-progress | tfidf | 0.146 / 0.182 | 0.145 / 0.210 | 0.145 / 0.191 | -0.04% |

All initial/repeated per-query means, medians, p95, process sums, startup, RSS and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.4.24/docs/measurements/0.4.24/small-mcp-standard-summary.json). No slower sample is removed.

### Confirmation small-MCP campaign

| Case | Ranker | 0.4.22 repeated median / p95 ms | 0.4.23 repeated median / p95 ms | 0.4.24 repeated median / p95 ms | vs 0.4.23 |
| --- | --- | --- | --- | --- | --- |
| tiny | bm25 | 0.115 / 0.134 | 0.116 / 0.135 | 0.115 / 0.139 | -0.52% |
| tiny | tfidf | 0.116 / 0.146 | 0.116 / 0.175 | 0.115 / 0.143 | -0.84% |
| tiny4 | bm25 | 0.169 / 0.220 | 0.168 / 0.201 | 0.167 / 0.211 | -1.06% |
| tiny4 | tfidf | 0.170 / 0.210 | 0.166 / 0.185 | 0.166 / 0.189 | -0.31% |
| tiny16 | bm25 | 0.186 / 0.207 | 0.188 / 0.215 | 0.185 / 0.216 | -1.49% |
| tiny16 | tfidf | 0.187 / 0.232 | 0.188 / 0.231 | 0.187 / 0.218 | -0.98% |
| tiny-no-progress | bm25 | 0.067 / 0.100 | 0.066 / 0.088 | 0.066 / 0.087 | +0.16% |
| tiny-no-progress | tfidf | 0.067 / 0.148 | 0.067 / 0.138 | 0.067 / 0.111 | -1.08% |
| tiny4-no-progress | bm25 | 0.118 / 0.153 | 0.119 / 0.162 | 0.119 / 0.163 | +0.53% |
| tiny4-no-progress | tfidf | 0.116 / 0.143 | 0.116 / 0.143 | 0.119 / 0.155 | +2.88% |
| tiny16-no-progress | bm25 | 0.140 / 0.191 | 0.136 / 0.185 | 0.139 / 0.176 | +1.86% |
| tiny16-no-progress | tfidf | 0.139 / 0.177 | 0.136 / 0.163 | 0.137 / 0.160 | +0.86% |

All initial/repeated per-query means, medians, p95, process sums, startup, RSS and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.4.24/docs/measurements/0.4.24/small-mcp-confirmation-summary.json). No slower sample is removed.

## Retained large and broad MCP controls

The two large-corpus campaigns each run seven rotating 0.4.22/0.4.23/0.4.24 triples on the same 167,942-entry knowledge input and ten retained queries. Each has 21 fresh servers, 1,428 schema-valid replies and 952 complete cross-version structured comparisons. Together: **42 servers, 2,856 schema-valid replies and 1,904 complete comparisons**. All compared results agree except asserted status version and version-qualified schema URNs. Each query/ranker has seven initial and fourteen repeated samples. Input candidate counts range from zero to 29,600; do not infer gains for all corpus densities or gaps. Dense, filtered, sparse and cancellation correctness also has independent native tests.

Three separate retained broad controls each use 21 servers: **63 servers, 4,284 actual schema replies and 2,856 complete structured comparisons**. They guard the 0.4.22 broad-explanation improvement rather than replace large-corpus evidence. Standard/confirmation retain the 257-record sparse/dense vocabulary fixture and ten original queries. Extended probes 32/33/64/65/128/129/192/255/256 known terms and 255 known terms plus a duplicate and unknown word. Existing normalized-term and byte limits remain. Keep all five distributions separate.

Search timing includes Rust work, serialization, stdio, Python parsing and requested progress. Initial calls, repeated calls, startup, per-query means/medians/tails, per-process totals and RSS are separate. Original standard queries are not all equally wide. Aggregate medians alone can obscure differences; retain every per-query row and raw timing observation. No sample is removed or replaced.

### Standard large-corpus campaign

| Ranker / phase | 0.4.22 median / p95 ms | 0.4.23 median / p95 ms | 0.4.24 median / p95 ms | vs 0.4.23 | vs 0.4.22 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 3.483 / 7.083 | 3.473 / 7.353 | 3.341 / 7.168 | -3.81% | -4.06% |
| tfidf / initial | 24.278 / 48.564 | 24.034 / 46.214 | 23.640 / 39.359 | -1.64% | -2.63% |
| bm25 / repeated | 3.151 / 8.980 | 3.202 / 7.169 | 2.861 / 6.875 | -10.66% | -9.22% |
| tfidf / repeated | 3.692 / 9.557 | 4.164 / 8.553 | 3.537 / 9.577 | -15.04% | -4.19% |

| Query | Repeated TF-IDF median ms: 0.4.22 / 0.4.23 / 0.4.24 | vs 0.4.23 | Initial TF-IDF mean ms: 0.4.23 → 0.4.24 |
| --- | --- | --- | --- |
| 0 | 3.696 / 4.164 / 3.359 | -19.33% | 29.076 → 27.477 |
| 1 | 3.797 / 3.955 / 3.533 | -10.67% | 29.939 → 28.685 |
| 2 | 5.895 / 6.347 / 5.632 | -11.26% | 44.195 → 39.967 |
| 3 | 0.966 / 1.071 / 0.971 | -9.33% | 2.403 → 2.476 |
| 4 | 4.053 / 4.402 / 3.822 | -13.17% | 20.454 → 21.365 |
| 5 | 5.948 / 6.207 / 5.456 | -12.10% | 34.032 → 31.533 |
| 6 | 1.756 / 1.829 / 1.702 | -6.94% | 7.906 → 6.471 |
| 7 | 6.250 / 6.520 / 5.738 | -12.00% | 37.606 → 31.994 |
| 8 | 3.584 / 3.444 / 3.145 | -8.68% | 4.017 → 3.693 |
| 9 | 0.194 / 0.197 / 0.167 | -15.42% | 0.389 → 0.386 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.22 | 206.095 | 73.751 | 5922.986 | 1726.016 |
| 0.4.23 | 206.590 | 77.502 | 6270.915 | 1725.188 |
| 0.4.24 | 182.813 | 65.490 | 6456.027 | 1724.922 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.4.24/docs/measurements/0.4.24/large-standard-summary.json).

### Confirmation large-corpus campaign

| Ranker / phase | 0.4.22 median / p95 ms | 0.4.23 median / p95 ms | 0.4.24 median / p95 ms | vs 0.4.23 | vs 0.4.22 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 2.930 / 8.863 | 2.678 / 4.243 | 2.822 / 4.504 | +5.41% | -3.68% |
| tfidf / initial | 21.309 / 39.690 | 18.055 / 32.286 | 19.741 / 33.553 | +9.34% | -7.36% |
| bm25 / repeated | 2.458 / 6.949 | 2.227 / 3.853 | 2.306 / 3.897 | +3.56% | -6.18% |
| tfidf / repeated | 2.832 / 8.942 | 2.381 / 4.392 | 2.544 / 4.694 | +6.86% | -10.14% |

| Query | Repeated TF-IDF median ms: 0.4.22 / 0.4.23 / 0.4.24 | vs 0.4.23 | Initial TF-IDF mean ms: 0.4.23 → 0.4.24 |
| --- | --- | --- | --- |
| 0 | 2.615 / 2.349 / 2.514 | +7.04% | 22.442 → 23.090 |
| 1 | 2.931 / 2.490 / 2.683 | +7.78% | 22.377 → 23.251 |
| 2 | 4.139 / 3.941 / 4.046 | +2.68% | 32.511 → 33.682 |
| 3 | 0.810 / 0.750 / 0.773 | +3.00% | 1.668 → 1.706 |
| 4 | 2.865 / 2.675 / 2.557 | -4.43% | 14.485 → 15.218 |
| 5 | 4.545 / 3.895 / 3.931 | +0.93% | 25.307 → 26.687 |
| 6 | 1.425 / 1.232 / 1.217 | -1.27% | 4.952 → 5.008 |
| 7 | 4.645 / 4.000 / 3.921 | -1.98% | 25.734 → 25.821 |
| 8 | 2.533 / 2.147 / 2.185 | +1.75% | 2.601 → 2.700 |
| 9 | 0.119 / 0.109 / 0.119 | +9.28% | 0.183 → 0.185 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.22 | 161.620 | 54.218 | 5178.202 | 1724.656 |
| 0.4.23 | 152.783 | 48.083 | 5369.443 | 1724.344 |
| 0.4.24 | 154.502 | 47.529 | 5146.390 | 1724.266 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.4.24/docs/measurements/0.4.24/large-confirmation-summary.json).

### Standard broad-query campaign

| Ranker / phase | 0.4.22 median / p95 ms | 0.4.23 median / p95 ms | 0.4.24 median / p95 ms | vs 0.4.23 | vs 0.4.22 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 0.836 / 4.504 | 0.959 / 6.420 | 0.748 / 5.875 | -21.99% | -10.55% |
| tfidf / initial | 0.987 / 3.631 | 1.040 / 3.439 | 0.837 / 3.311 | -19.53% | -15.27% |
| bm25 / repeated | 0.659 / 3.798 | 0.768 / 3.923 | 0.679 / 3.496 | -11.52% | +3.11% |
| tfidf / repeated | 0.708 / 3.914 | 0.925 / 4.012 | 0.742 / 5.457 | -19.78% | +4.80% |

| Query | Repeated TF-IDF median ms: 0.4.22 / 0.4.23 / 0.4.24 | vs 0.4.23 | Initial TF-IDF mean ms: 0.4.23 → 0.4.24 |
| --- | --- | --- | --- |
| 0 | 0.603 / 1.604 / 0.587 | -63.40% | 0.952 → 1.470 |
| 1 | 0.544 / 0.625 / 0.581 | -7.15% | 1.961 → 1.592 |
| 2 | 0.574 / 0.745 / 0.648 | -12.96% | 3.165 → 1.380 |
| 3 | 0.954 / 1.022 / 0.997 | -2.45% | 1.152 → 3.121 |
| 4 | 0.585 / 0.708 / 0.579 | -18.15% | 2.323 → 1.140 |
| 5 | 0.572 / 0.635 / 0.609 | -4.15% | 1.101 → 0.553 |
| 6 | 0.591 / 0.647 / 0.634 | -2.12% | 1.096 → 2.182 |
| 7 | 0.912 / 1.019 / 0.939 | -7.85% | 3.270 → 1.849 |
| 8 | 0.525 / 0.557 / 0.511 | -8.35% | 1.080 → 0.748 |
| 9 | 0.916 / 0.958 / 0.911 | -4.90% | 1.679 → 1.404 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.22 | 11.568 | 13.403 | 5.416 | 7.797 |
| 0.4.23 | 14.768 | 26.288 | 7.546 | 7.812 |
| 0.4.24 | 19.240 | 36.134 | 7.590 | 7.922 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.4.24/docs/measurements/0.4.24/wide-standard-summary.json).

### Confirmation broad-query campaign

| Ranker / phase | 0.4.22 median / p95 ms | 0.4.23 median / p95 ms | 0.4.24 median / p95 ms | vs 0.4.23 | vs 0.4.22 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 0.537 / 0.668 | 0.523 / 0.639 | 0.534 / 0.671 | +2.10% | -0.61% |
| tfidf / initial | 0.529 / 0.841 | 0.505 / 0.838 | 0.505 / 0.858 | -0.11% | -4.58% |
| bm25 / repeated | 0.471 / 0.750 | 0.458 / 0.716 | 0.476 / 0.742 | +3.93% | +0.99% |
| tfidf / repeated | 0.464 / 0.781 | 0.460 / 0.764 | 0.480 / 0.773 | +4.37% | +3.45% |

| Query | Repeated TF-IDF median ms: 0.4.22 / 0.4.23 / 0.4.24 | vs 0.4.23 | Initial TF-IDF mean ms: 0.4.23 → 0.4.24 |
| --- | --- | --- | --- |
| 0 | 0.445 / 0.437 / 0.469 | +7.34% | 0.524 → 0.556 |
| 1 | 0.419 / 0.414 / 0.457 | +10.33% | 0.453 → 0.480 |
| 2 | 0.457 / 0.461 / 0.461 | +0.03% | 0.512 → 0.501 |
| 3 | 0.769 / 0.728 / 0.741 | +1.78% | 0.851 → 0.816 |
| 4 | 0.466 / 0.448 / 0.464 | +3.61% | 0.469 → 0.464 |
| 5 | 0.443 / 0.444 / 0.435 | -1.88% | 0.454 → 0.433 |
| 6 | 0.437 / 0.455 / 0.461 | +1.42% | 0.539 → 0.500 |
| 7 | 0.723 / 0.750 / 0.735 | -1.98% | 0.796 → 0.770 |
| 8 | 0.404 / 0.410 / 0.414 | +0.88% | 0.403 → 0.446 |
| 9 | 0.715 / 0.731 / 0.730 | -0.18% | 0.726 → 0.791 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.22 | 5.749 | 10.692 | 4.428 | 7.672 |
| 0.4.23 | 5.778 | 10.601 | 4.282 | 7.656 |
| 0.4.24 | 5.730 | 10.766 | 4.202 | 7.641 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.4.24/docs/measurements/0.4.24/wide-confirmation-summary.json).

### Extended broad-query campaign

| Ranker / phase | 0.4.22 median / p95 ms | 0.4.23 median / p95 ms | 0.4.24 median / p95 ms | vs 0.4.23 | vs 0.4.22 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 0.787 / 1.245 | 0.780 / 1.365 | 0.758 / 1.213 | -2.89% | -3.71% |
| tfidf / initial | 0.906 / 1.310 | 0.931 / 1.368 | 0.912 / 1.252 | -2.05% | +0.65% |
| bm25 / repeated | 0.702 / 1.478 | 0.718 / 1.415 | 0.689 / 1.359 | -3.98% | -1.87% |
| tfidf / repeated | 0.886 / 1.319 | 0.900 / 1.307 | 0.864 / 1.252 | -4.06% | -2.46% |

| Query | Repeated TF-IDF median ms: 0.4.22 / 0.4.23 / 0.4.24 | vs 0.4.23 | Initial TF-IDF mean ms: 0.4.23 → 0.4.24 |
| --- | --- | --- | --- |
| 0 | 0.574 / 0.555 / 0.552 | -0.47% | 0.598 → 0.638 |
| 1 | 0.510 / 0.516 / 0.525 | +1.61% | 0.561 → 0.538 |
| 2 | 0.595 / 0.546 / 0.537 | -1.58% | 0.631 → 0.586 |
| 3 | 0.537 / 0.559 / 0.539 | -3.63% | 0.607 → 0.588 |
| 4 | 0.892 / 0.936 / 0.864 | -7.72% | 0.960 → 0.925 |
| 5 | 0.931 / 0.922 / 0.893 | -3.18% | 0.953 → 0.895 |
| 6 | 1.071 / 1.077 / 1.070 | -0.57% | 1.140 → 1.173 |
| 7 | 1.111 / 1.138 / 1.134 | -0.36% | 1.171 → 1.158 |
| 8 | 1.183 / 1.117 / 1.131 | +1.29% | 1.220 → 1.156 |
| 9 | 1.143 / 1.109 / 1.103 | -0.49% | 1.133 → 1.139 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.22 | 8.938 | 17.392 | 4.721 | 7.938 |
| 0.4.23 | 9.022 | 17.019 | 4.898 | 7.969 |
| 0.4.24 | 8.848 | 16.851 | 4.667 | 8.062 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.4.24/docs/measurements/0.4.24/wide-extended-summary.json).

### Existing limit and recovery checks

Three additional fresh servers reject 257/512 known-term requests with both rankers: twelve expected errors, no partial successful results, error progress and successful later health requests. The existing 256-unique-term assessment limit remains; these are not successful-search latency samples. Large inputs remain subject to cooperative bounds, not an OS CPU/RAM quota.

## Actual MCP: two ordinary campaigns

Each ordinary campaign uses nine cases and five alternating old/new pairs: **90 servers each, 180 total**. Canonical knowledge, NumPy and broad synthetic input request progress; 1/4/16-entry inputs run with and without progress. Each makes ten queries with both rankers, two repeated passes, four concurrent searches, matching/relations, rejected arguments and health; snapshot inputs check exact UTF-8 readbacks.

Together **13,880 actual result/error replies validate against advertised schemas**, and **6,940 complete old/new structured pairs agree by RPC ID**, excluding only checked status version and version-qualified schema URNs. Discovery, progress/no-progress, bounds, ordering, input hashes and EOF shutdown pass. Search and readback time are separate. Child peak RSS includes runtime, buffers and source/index memory, unlike allocation profiles. Retain all slower rows.

### First complete campaign

| Case | Ranker | Initial median / p95 ms: old → new | Repeated median / p95 ms: old → new | Repeated median change |
| --- | --- | --- | --- | --- |
| canonical | bm25 | 2.743 / 4.411 → 2.956 / 5.096 | 2.278 / 3.784 → 2.358 / 4.878 | +3.52% |
| canonical | tfidf | 18.415 / 34.424 → 20.370 / 37.151 | 2.496 / 4.128 → 2.466 / 6.818 | -1.22% |
| numpy | bm25 | 1.030 / 1.476 → 1.051 / 1.623 | 0.929 / 1.448 → 0.945 / 1.383 | +1.79% |
| numpy | tfidf | 1.346 / 3.186 → 1.211 / 3.060 | 0.870 / 1.631 → 0.869 / 1.454 | -0.08% |
| wide | bm25 | 0.494 / 0.662 → 0.516 / 0.715 | 0.443 / 0.767 → 0.447 / 0.752 | +0.84% |
| wide | tfidf | 0.469 / 0.780 → 0.493 / 0.895 | 0.414 / 0.762 → 0.424 / 0.761 | +2.34% |
| tiny | bm25 | 0.197 / 0.299 → 0.222 / 0.321 | 0.198 / 0.285 → 0.193 / 0.266 | -2.55% |
| tiny | tfidf | 0.202 / 0.302 → 0.195 / 0.292 | 0.201 / 0.276 → 0.190 / 0.277 | -5.39% |
| tiny4 | bm25 | 0.293 / 0.428 → 0.303 / 0.447 | 0.266 / 0.330 → 0.237 / 0.323 | -11.00% |
| tiny4 | tfidf | 0.290 / 0.404 → 0.274 / 0.373 | 0.264 / 0.387 → 0.239 / 0.316 | -9.40% |
| tiny16 | bm25 | 0.283 / 0.410 → 0.271 / 0.387 | 0.236 / 0.338 → 0.233 / 0.306 | -1.55% |
| tiny16 | tfidf | 0.261 / 0.382 → 0.249 / 0.322 | 0.234 / 0.335 → 0.228 / 0.307 | -2.61% |
| tiny-no-progress | bm25 | 0.114 / 0.222 → 0.108 / 0.202 | 0.092 / 0.143 → 0.078 / 0.130 | -15.30% |
| tiny-no-progress | tfidf | 0.109 / 0.196 → 0.112 / 0.146 | 0.093 / 0.171 → 0.080 / 0.168 | -13.36% |
| tiny4-no-progress | bm25 | 0.163 / 0.291 → 0.166 / 0.309 | 0.134 / 0.200 → 0.134 / 0.182 | -0.47% |
| tiny4-no-progress | tfidf | 0.156 / 0.227 → 0.159 / 0.242 | 0.129 / 0.189 → 0.130 / 0.189 | +0.23% |
| tiny16-no-progress | bm25 | 0.205 / 0.355 → 0.201 / 0.333 | 0.185 / 0.277 → 0.158 / 0.269 | -14.24% |
| tiny16-no-progress | tfidf | 0.201 / 0.278 → 0.195 / 0.292 | 0.182 / 0.267 → 0.151 / 0.244 | -16.80% |

| Case | Startup median ms: old → new | First TF-IDF median ms: old → new | Peak RSS median MiB: old → new |
| --- | --- | --- | --- |
| canonical | 5208.614 → 5373.023 | 23.530 → 23.170 | 1724.266 → 1724.391 |
| numpy | 163.721 → 169.496 | 1.675 → 2.180 | 57.391 → 57.344 |
| wide | 3.892 → 3.971 | 0.484 → 0.488 | 7.672 → 7.750 |
| tiny | 4.119 → 4.281 | 0.203 → 0.233 | 5.766 → 5.891 |
| tiny4 | 4.319 → 3.765 | 0.330 → 0.342 | 5.891 → 5.969 |
| tiny16 | 3.609 → 3.680 | 0.338 → 0.312 | 6.078 → 6.219 |
| tiny-no-progress | 3.643 → 3.532 | 0.165 → 0.136 | 5.750 → 5.875 |
| tiny4-no-progress | 3.799 → 3.859 | 0.224 → 0.217 | 5.812 → 6.000 |
| tiny16-no-progress | 3.704 → 3.796 | 0.276 → 0.248 | 6.031 → 6.172 |

### Second preplanned complete campaign

| Case | Ranker | Initial median / p95 ms: old → new | Repeated median / p95 ms: old → new | Repeated median change |
| --- | --- | --- | --- | --- |
| canonical | bm25 | 2.989 / 6.529 → 2.971 / 5.270 | 2.677 / 5.876 → 2.538 / 4.876 | -5.20% |
| canonical | tfidf | 21.963 / 44.186 → 22.110 / 37.467 | 3.326 / 7.737 → 3.139 / 6.569 | -5.60% |
| numpy | bm25 | 1.459 / 2.584 → 1.300 / 3.257 | 1.167 / 2.459 → 1.200 / 3.464 | +2.89% |
| numpy | tfidf | 1.887 / 4.918 → 1.662 / 4.535 | 1.143 / 3.150 → 1.199 / 3.550 | +4.89% |
| wide | bm25 | 1.627 / 4.540 → 1.632 / 12.011 | 1.177 / 7.230 → 0.782 / 10.031 | -33.55% |
| wide | tfidf | 1.903 / 5.946 → 1.505 / 11.207 | 1.135 / 10.289 → 0.881 / 6.018 | -22.37% |
| tiny | bm25 | 0.242 / 1.527 → 0.348 / 4.673 | 0.161 / 0.311 → 0.219 / 3.118 | +36.20% |
| tiny | tfidf | 0.228 / 2.346 → 0.332 / 3.679 | 0.165 / 0.301 → 0.229 / 1.835 | +39.16% |
| tiny4 | bm25 | 0.889 / 10.440 → 1.242 / 3.654 | 0.319 / 2.038 → 0.377 / 3.074 | +18.23% |
| tiny4 | tfidf | 0.557 / 6.489 → 1.324 / 10.840 | 0.327 / 1.251 → 0.362 / 2.799 | +10.58% |
| tiny16 | bm25 | 0.704 / 2.648 → 0.744 / 2.309 | 0.479 / 1.586 → 0.725 / 2.689 | +51.52% |
| tiny16 | tfidf | 0.738 / 3.225 → 0.729 / 1.848 | 0.481 / 2.814 → 0.741 / 2.481 | +54.10% |
| tiny-no-progress | bm25 | 0.171 / 3.280 → 0.251 / 1.590 | 0.138 / 10.240 → 0.131 / 1.684 | -4.98% |
| tiny-no-progress | tfidf | 0.155 / 0.883 → 0.208 / 3.258 | 0.137 / 4.995 → 0.141 / 1.358 | +2.80% |
| tiny4-no-progress | bm25 | 0.360 / 2.364 → 0.323 / 2.277 | 0.220 / 0.988 → 0.197 / 0.761 | -10.46% |
| tiny4-no-progress | tfidf | 0.367 / 4.039 → 0.299 / 4.128 | 0.218 / 0.973 → 0.192 / 0.656 | -12.32% |
| tiny16-no-progress | bm25 | 0.315 / 0.457 → 0.290 / 0.481 | 0.247 / 0.328 → 0.247 / 0.336 | -0.20% |
| tiny16-no-progress | tfidf | 0.279 / 0.385 → 0.265 / 0.414 | 0.240 / 0.333 → 0.245 / 0.315 | +2.24% |

| Case | Startup median ms: old → new | First TF-IDF median ms: old → new | Peak RSS median MiB: old → new |
| --- | --- | --- | --- |
| canonical | 5811.359 → 5669.336 | 24.802 → 25.612 | 1724.609 → 1724.453 |
| numpy | 206.459 → 195.382 | 2.694 → 2.475 | 58.391 → 58.250 |
| wide | 9.549 → 10.496 | 2.501 → 0.956 | 7.906 → 7.906 |
| tiny | 8.927 → 9.324 | 0.366 → 1.708 | 5.797 → 5.953 |
| tiny4 | 8.979 → 8.789 | 1.759 → 2.544 | 5.891 → 6.016 |
| tiny16 | 6.576 → 6.985 | 0.874 → 0.613 | 6.062 → 6.156 |
| tiny-no-progress | 5.956 → 5.932 | 0.508 → 0.367 | 5.719 → 5.875 |
| tiny4-no-progress | 5.667 → 7.925 | 0.354 → 0.378 | 5.812 → 5.922 |
| tiny16-no-progress | 5.015 → 5.282 | 0.363 → 0.396 | 6.078 → 6.203 |

## Retained full small-Core regression controls

These separate campaigns compare freshly built exact registry 0.4.22 and 0.4.23 with the candidate, using the same programs. Seven rotating triples per campaign run both the original six cases and a 96-case extension. Warm-up, full original-JSON comparison and index setup occur before timers. Each campaign has **42 processes and 2,142,000 timed Core calls**. Both together have 84 processes, 4,284,000 calls and **2,688 complete original matrix-response comparisons**, with identical results. Signature checks also cover every original-six and timed matrix sample. This is repeated warm Core time, without MCP transport or DHAT instrumentation.

Keep the two campaigns separate. Original six-case calls build SearchOptions inside the measured loop; the extension reuses fixed options. Do not compare their absolute times or merge distributions. The 96 cases combine four entry counts, both rankers and twelve fixed query/filter/limit cases. Different-word, wide, punctuated and Unicode queries retain fallback analysis; short canonical repetitions retain the existing path. Unknown-only ASCII queries, scope rejection and zero limits are also retained; no case is removed for missing the optimized path. Historical absolute values with different programs/loads are not a reference substitute.

### First small Core campaign

#### Original six warm cases

| Fixture | 0.4.22 median ns | 0.4.23 median ns | 0.4.24 median ns | vs 0.4.23 | vs 0.4.22 |
| --- | --- | --- | --- | --- | --- |
| tiny16_bm25 | 954.959 | 937.291 | 951.583 | +1.52% | -0.35% |
| tiny16_tfidf | 1019.542 | 1021.750 | 991.208 | -2.99% | -2.78% |
| tiny4_bm25 | 299.667 | 294.875 | 291.709 | -1.07% | -2.66% |
| tiny4_tfidf | 319.959 | 318.417 | 312.708 | -1.79% | -2.27% |
| tiny_bm25 | 135.042 | 121.417 | 120.541 | -0.72% | -10.74% |
| tiny_tfidf | 129.208 | 129.917 | 126.708 | -2.47% | -1.93% |

Higher candidate medians versus 0.4.23: **1 of 6**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](https://github.com/LEVOGNE/airust/blob/v0.4.24/docs/measurements/0.4.24/small-reference-summary.json). No threshold-based rejection or removal of slow timing samples is used.

#### All 96 extended cases

| Fixture | 0.4.22 median ns | 0.4.23 median ns | 0.4.24 median ns | vs 0.4.23 | vs 0.4.22 |
| --- | --- | --- | --- | --- | --- |
| 16_Bm25_disjoint | 469.875 | 428.625 | 429.625 | +0.23% | -8.57% |
| 16_Bm25_limit_one | 596.834 | 579.791 | 577.292 | -0.43% | -3.27% |
| 16_Bm25_limit_zero | 2.708 | 2.917 | 3.625 | +24.27% | +33.86% |
| 16_Bm25_multi | 1619.333 | 1674.375 | 1653.833 | -1.23% | +2.13% |
| 16_Bm25_punctuation | 146.542 | 147.584 | 145.417 | -1.47% | -0.77% |
| 16_Bm25_repeated | 1183.167 | 1157.666 | 1106.625 | -4.41% | -6.47% |
| 16_Bm25_scope | 971.083 | 994.834 | 953.625 | -4.14% | -1.80% |
| 16_Bm25_single | 1081.375 | 1131.875 | 1077.708 | -4.79% | -0.34% |
| 16_Bm25_unicode | 1354.916 | 1411.542 | 1385.917 | -1.82% | +2.29% |
| 16_Bm25_unknown | 19.916 | 19.584 | 19.541 | -0.22% | -1.88% |
| 16_Bm25_unknown_added | 1200.875 | 1204.292 | 1181.583 | -1.89% | -1.61% |
| 16_Bm25_wide | 17650.375 | 17195.250 | 17370.666 | +1.02% | -1.58% |
| 16_Tfidf_disjoint | 466.958 | 438.834 | 438.833 | -0.00% | -6.02% |
| 16_Tfidf_limit_one | 701.250 | 669.708 | 692.292 | +3.37% | -1.28% |
| 16_Tfidf_limit_zero | 2.708 | 2.792 | 2.750 | -1.50% | +1.55% |
| 16_Tfidf_multi | 1734.667 | 1724.333 | 1743.666 | +1.12% | +0.52% |
| 16_Tfidf_punctuation | 147.000 | 148.041 | 144.416 | -2.45% | -1.76% |
| 16_Tfidf_repeated | 1332.041 | 1230.250 | 1210.875 | -1.57% | -9.10% |
| 16_Tfidf_scope | 1057.375 | 1053.875 | 1049.583 | -0.41% | -0.74% |
| 16_Tfidf_single | 1199.750 | 1185.541 | 1189.125 | +0.30% | -0.89% |
| 16_Tfidf_unicode | 1449.542 | 1468.875 | 1440.917 | -1.90% | -0.60% |
| 16_Tfidf_unknown | 20.209 | 19.625 | 19.542 | -0.42% | -3.30% |
| 16_Tfidf_unknown_added | 1322.416 | 1265.417 | 1291.208 | +2.04% | -2.36% |
| 16_Tfidf_wide | 17744.834 | 17195.041 | 17514.750 | +1.86% | -1.30% |
| 1_Bm25_disjoint | 59.750 | 55.334 | 54.875 | -0.83% | -8.16% |
| 1_Bm25_limit_one | 127.167 | 122.334 | 125.375 | +2.49% | -1.41% |
| 1_Bm25_limit_zero | 2.667 | 2.750 | 2.791 | +1.49% | +4.65% |
| 1_Bm25_multi | 264.750 | 267.583 | 275.250 | +2.87% | +3.97% |
| 1_Bm25_punctuation | 144.875 | 150.084 | 147.584 | -1.67% | +1.87% |
| 1_Bm25_repeated | 274.791 | 158.292 | 153.250 | -3.19% | -44.23% |
| 1_Bm25_scope | 133.417 | 136.875 | 132.792 | -2.98% | -0.47% |
| 1_Bm25_single | 148.292 | 146.250 | 144.834 | -0.97% | -2.33% |
| 1_Bm25_unicode | 423.917 | 433.792 | 429.458 | -1.00% | +1.31% |
| 1_Bm25_unknown | 19.875 | 19.416 | 19.292 | -0.64% | -2.93% |
| 1_Bm25_unknown_added | 229.375 | 247.708 | 251.667 | +1.60% | +9.72% |
| 1_Bm25_wide | 4527.209 | 4487.042 | 4554.708 | +1.51% | +0.61% |
| 1_Tfidf_disjoint | 59.959 | 58.209 | 57.958 | -0.43% | -3.34% |
| 1_Tfidf_limit_one | 135.166 | 135.500 | 133.708 | -1.32% | -1.08% |
| 1_Tfidf_limit_zero | 2.708 | 2.792 | 2.750 | -1.50% | +1.55% |
| 1_Tfidf_multi | 266.959 | 271.750 | 277.792 | +2.22% | +4.06% |
| 1_Tfidf_punctuation | 146.667 | 147.542 | 148.167 | +0.42% | +1.02% |
| 1_Tfidf_repeated | 252.125 | 153.209 | 154.916 | +1.11% | -38.56% |
| 1_Tfidf_scope | 139.542 | 141.958 | 142.500 | +0.38% | +2.12% |
| 1_Tfidf_single | 134.917 | 134.334 | 136.250 | +1.43% | +0.99% |
| 1_Tfidf_unicode | 438.416 | 440.667 | 443.958 | +0.75% | +1.26% |
| 1_Tfidf_unknown | 20.042 | 19.375 | 19.333 | -0.22% | -3.54% |
| 1_Tfidf_unknown_added | 232.500 | 233.625 | 239.458 | +2.50% | +2.99% |
| 1_Tfidf_wide | 4450.333 | 4528.458 | 4507.000 | -0.47% | +1.27% |
| 33_Bm25_disjoint | 1020.042 | 938.959 | 891.583 | -5.05% | -12.59% |
| 33_Bm25_limit_one | 1194.250 | 1194.583 | 1185.167 | -0.79% | -0.76% |
| 33_Bm25_limit_zero | 2.708 | 2.791 | 2.833 | +1.50% | +4.62% |
| 33_Bm25_multi | 2547.500 | 2559.084 | 2519.833 | -1.53% | -1.09% |
| 33_Bm25_punctuation | 143.541 | 148.542 | 144.250 | -2.89% | +0.49% |
| 33_Bm25_repeated | 1816.334 | 1690.583 | 1683.042 | -0.45% | -7.34% |
| 33_Bm25_scope | 1818.459 | 1768.000 | 1760.250 | -0.44% | -3.20% |
| 33_Bm25_single | 1708.500 | 1645.166 | 1661.209 | +0.98% | -2.77% |
| 33_Bm25_unicode | 1936.500 | 1953.792 | 1945.083 | -0.45% | +0.44% |
| 33_Bm25_unknown | 19.875 | 19.667 | 19.333 | -1.70% | -2.73% |
| 33_Bm25_unknown_added | 1784.750 | 1823.750 | 1761.291 | -3.42% | -1.31% |
| 33_Bm25_wide | 26872.833 | 26706.875 | 26825.583 | +0.44% | -0.18% |
| 33_Tfidf_disjoint | 997.250 | 906.416 | 935.333 | +3.19% | -6.21% |
| 33_Tfidf_limit_one | 1356.958 | 1296.208 | 1332.500 | +2.80% | -1.80% |
| 33_Tfidf_limit_zero | 2.667 | 2.833 | 2.792 | -1.45% | +4.69% |
| 33_Tfidf_multi | 2659.750 | 2735.042 | 2735.458 | +0.02% | +2.85% |
| 33_Tfidf_punctuation | 144.500 | 146.167 | 143.166 | -2.05% | -0.92% |
| 33_Tfidf_repeated | 1964.250 | 1902.042 | 1885.667 | -0.86% | -4.00% |
| 33_Tfidf_scope | 1887.583 | 1854.625 | 1868.708 | +0.76% | -1.00% |
| 33_Tfidf_single | 1833.458 | 1861.250 | 1861.667 | +0.02% | +1.54% |
| 33_Tfidf_unicode | 2071.083 | 2046.417 | 2132.208 | +4.19% | +2.95% |
| 33_Tfidf_unknown | 19.959 | 19.417 | 19.500 | +0.43% | -2.30% |
| 33_Tfidf_unknown_added | 2000.500 | 1990.500 | 2004.458 | +0.70% | +0.20% |
| 33_Tfidf_wide | 26906.208 | 26903.958 | 27037.000 | +0.49% | +0.49% |
| 4_Bm25_disjoint | 110.708 | 107.417 | 107.417 | +0.00% | -2.97% |
| 4_Bm25_limit_one | 186.041 | 186.084 | 187.042 | +0.51% | +0.54% |
| 4_Bm25_limit_zero | 2.625 | 2.792 | 2.833 | +1.47% | +7.92% |
| 4_Bm25_multi | 543.166 | 560.292 | 569.250 | +1.60% | +4.80% |
| 4_Bm25_punctuation | 145.333 | 145.667 | 144.833 | -0.57% | -0.34% |
| 4_Bm25_repeated | 436.667 | 348.708 | 331.417 | -4.96% | -24.10% |
| 4_Bm25_scope | 234.416 | 236.042 | 234.042 | -0.85% | -0.16% |
| 4_Bm25_single | 320.125 | 325.084 | 323.500 | -0.49% | +1.05% |
| 4_Bm25_unicode | 608.250 | 602.458 | 605.583 | +0.52% | -0.44% |
| 4_Bm25_unknown | 19.875 | 19.416 | 19.458 | +0.22% | -2.10% |
| 4_Bm25_unknown_added | 414.958 | 418.209 | 425.791 | +1.81% | +2.61% |
| 4_Bm25_wide | 7481.334 | 7464.042 | 7549.875 | +1.15% | +0.92% |
| 4_Tfidf_disjoint | 113.459 | 107.459 | 105.875 | -1.47% | -6.68% |
| 4_Tfidf_limit_one | 217.792 | 214.792 | 215.375 | +0.27% | -1.11% |
| 4_Tfidf_limit_zero | 2.708 | 3.666 | 2.750 | -24.99% | +1.55% |
| 4_Tfidf_multi | 583.625 | 613.625 | 579.500 | -5.56% | -0.71% |
| 4_Tfidf_punctuation | 145.167 | 148.208 | 148.250 | +0.03% | +2.12% |
| 4_Tfidf_repeated | 469.292 | 365.875 | 367.958 | +0.57% | -21.59% |
| 4_Tfidf_scope | 245.541 | 243.625 | 249.000 | +2.21% | +1.41% |
| 4_Tfidf_single | 345.500 | 350.708 | 348.791 | -0.55% | +0.95% |
| 4_Tfidf_unicode | 646.917 | 648.625 | 649.500 | +0.13% | +0.40% |
| 4_Tfidf_unknown | 19.959 | 19.459 | 19.417 | -0.22% | -2.72% |
| 4_Tfidf_unknown_added | 452.750 | 456.708 | 449.708 | -1.53% | -0.67% |
| 4_Tfidf_wide | 7457.958 | 7500.708 | 7447.667 | -0.71% | -0.14% |

Higher candidate medians versus 0.4.23: **44 of 96**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](https://github.com/LEVOGNE/airust/blob/v0.4.24/docs/measurements/0.4.24/small-reference-summary.json). No threshold-based rejection or removal of slow timing samples is used.

### Second preplanned small Core campaign

#### Original six warm cases

| Fixture | 0.4.22 median ns | 0.4.23 median ns | 0.4.24 median ns | vs 0.4.23 | vs 0.4.22 |
| --- | --- | --- | --- | --- | --- |
| tiny16_bm25 | 917.500 | 916.458 | 905.334 | -1.21% | -1.33% |
| tiny16_tfidf | 1003.250 | 992.750 | 979.541 | -1.33% | -2.36% |
| tiny4_bm25 | 292.292 | 294.584 | 287.417 | -2.43% | -1.67% |
| tiny4_tfidf | 310.542 | 320.708 | 309.458 | -3.51% | -0.35% |
| tiny_bm25 | 122.834 | 120.375 | 121.500 | +0.93% | -1.09% |
| tiny_tfidf | 128.875 | 126.375 | 125.666 | -0.56% | -2.49% |

Higher candidate medians versus 0.4.23: **1 of 6**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](https://github.com/LEVOGNE/airust/blob/v0.4.24/docs/measurements/0.4.24/small-confirmation-summary.json). No threshold-based rejection or removal of slow timing samples is used.

#### All 96 extended cases

| Fixture | 0.4.22 median ns | 0.4.23 median ns | 0.4.24 median ns | vs 0.4.23 | vs 0.4.22 |
| --- | --- | --- | --- | --- | --- |
| 16_Bm25_disjoint | 451.208 | 419.041 | 423.667 | +1.10% | -6.10% |
| 16_Bm25_limit_one | 569.709 | 578.250 | 568.709 | -1.65% | -0.18% |
| 16_Bm25_limit_zero | 2.708 | 3.833 | 2.875 | -24.99% | +6.17% |
| 16_Bm25_multi | 1564.750 | 1598.459 | 1609.958 | +0.72% | +2.89% |
| 16_Bm25_punctuation | 145.459 | 145.167 | 143.583 | -1.09% | -1.29% |
| 16_Bm25_repeated | 1162.792 | 1070.500 | 1079.875 | +0.88% | -7.13% |
| 16_Bm25_scope | 928.834 | 927.792 | 935.916 | +0.88% | +0.76% |
| 16_Bm25_single | 1037.875 | 1048.833 | 1051.250 | +0.23% | +1.29% |
| 16_Bm25_unicode | 1324.667 | 1328.458 | 1349.083 | +1.55% | +1.84% |
| 16_Bm25_unknown | 20.458 | 19.250 | 19.333 | +0.43% | -5.50% |
| 16_Bm25_unknown_added | 1148.750 | 1163.792 | 1154.292 | -0.82% | +0.48% |
| 16_Bm25_wide | 17261.500 | 16801.584 | 17046.333 | +1.46% | -1.25% |
| 16_Tfidf_disjoint | 450.666 | 424.291 | 426.083 | +0.42% | -5.45% |
| 16_Tfidf_limit_one | 676.375 | 656.291 | 670.291 | +2.13% | -0.90% |
| 16_Tfidf_limit_zero | 2.667 | 3.833 | 2.792 | -27.16% | +4.69% |
| 16_Tfidf_multi | 1706.667 | 1668.459 | 1695.125 | +1.60% | -0.68% |
| 16_Tfidf_punctuation | 141.833 | 145.584 | 143.833 | -1.20% | +1.41% |
| 16_Tfidf_repeated | 1284.500 | 1205.750 | 1197.959 | -0.65% | -6.74% |
| 16_Tfidf_scope | 1009.167 | 1007.334 | 999.417 | -0.79% | -0.97% |
| 16_Tfidf_single | 1166.792 | 1194.750 | 1185.583 | -0.77% | +1.61% |
| 16_Tfidf_unicode | 1443.917 | 1405.375 | 1413.917 | +0.61% | -2.08% |
| 16_Tfidf_unknown | 19.667 | 19.292 | 19.416 | +0.64% | -1.28% |
| 16_Tfidf_unknown_added | 1261.709 | 1256.625 | 1260.583 | +0.31% | -0.09% |
| 16_Tfidf_wide | 17253.875 | 17192.917 | 17109.375 | -0.49% | -0.84% |
| 1_Bm25_disjoint | 59.208 | 54.875 | 55.583 | +1.29% | -6.12% |
| 1_Bm25_limit_one | 124.667 | 122.167 | 121.750 | -0.34% | -2.34% |
| 1_Bm25_limit_zero | 2.667 | 2.791 | 2.750 | -1.47% | +3.11% |
| 1_Bm25_multi | 263.709 | 262.958 | 267.625 | +1.77% | +1.48% |
| 1_Bm25_punctuation | 144.666 | 144.625 | 143.083 | -1.07% | -1.09% |
| 1_Bm25_repeated | 265.875 | 155.375 | 161.791 | +4.13% | -39.15% |
| 1_Bm25_scope | 132.333 | 129.542 | 129.042 | -0.39% | -2.49% |
| 1_Bm25_single | 148.958 | 146.083 | 145.375 | -0.48% | -2.41% |
| 1_Bm25_unicode | 424.917 | 423.250 | 424.750 | +0.35% | -0.04% |
| 1_Bm25_unknown | 20.583 | 19.125 | 19.125 | +0.00% | -7.08% |
| 1_Bm25_unknown_added | 229.208 | 241.459 | 255.542 | +5.83% | +11.49% |
| 1_Bm25_wide | 4364.208 | 4367.417 | 4344.041 | -0.54% | -0.46% |
| 1_Tfidf_disjoint | 60.250 | 57.041 | 56.167 | -1.53% | -6.78% |
| 1_Tfidf_limit_one | 135.208 | 132.333 | 130.959 | -1.04% | -3.14% |
| 1_Tfidf_limit_zero | 2.667 | 2.792 | 2.792 | +0.00% | +4.69% |
| 1_Tfidf_multi | 259.708 | 266.708 | 264.375 | -0.87% | +1.80% |
| 1_Tfidf_punctuation | 143.917 | 142.416 | 143.916 | +1.05% | -0.00% |
| 1_Tfidf_repeated | 250.333 | 149.708 | 150.333 | +0.42% | -39.95% |
| 1_Tfidf_scope | 140.459 | 139.250 | 138.417 | -0.60% | -1.45% |
| 1_Tfidf_single | 133.792 | 131.708 | 132.667 | +0.73% | -0.84% |
| 1_Tfidf_unicode | 437.958 | 432.958 | 433.584 | +0.14% | -1.00% |
| 1_Tfidf_unknown | 19.833 | 19.250 | 19.166 | -0.44% | -3.36% |
| 1_Tfidf_unknown_added | 229.542 | 233.917 | 231.708 | -0.94% | +0.94% |
| 1_Tfidf_wide | 4482.667 | 4378.333 | 4475.667 | +2.22% | -0.16% |
| 33_Bm25_disjoint | 969.959 | 907.708 | 881.917 | -2.84% | -9.08% |
| 33_Bm25_limit_one | 1148.833 | 1130.208 | 1143.000 | +1.13% | -0.51% |
| 33_Bm25_limit_zero | 2.667 | 2.750 | 3.833 | +39.38% | +43.72% |
| 33_Bm25_multi | 2400.459 | 2513.459 | 2459.792 | -2.14% | +2.47% |
| 33_Bm25_punctuation | 144.709 | 143.542 | 144.000 | +0.32% | -0.49% |
| 33_Bm25_repeated | 1767.875 | 1667.500 | 1668.125 | +0.04% | -5.64% |
| 33_Bm25_scope | 1722.917 | 1709.958 | 1687.125 | -1.34% | -2.08% |
| 33_Bm25_single | 1596.917 | 1630.166 | 1601.042 | -1.79% | +0.26% |
| 33_Bm25_unicode | 1886.083 | 1865.750 | 1851.792 | -0.75% | -1.82% |
| 33_Bm25_unknown | 19.709 | 19.458 | 19.375 | -0.43% | -1.69% |
| 33_Bm25_unknown_added | 1745.625 | 1772.125 | 1749.458 | -1.28% | +0.22% |
| 33_Bm25_wide | 26653.583 | 26051.709 | 26293.000 | +0.93% | -1.35% |
| 33_Tfidf_disjoint | 955.625 | 896.084 | 878.500 | -1.96% | -8.07% |
| 33_Tfidf_limit_one | 1304.458 | 1289.750 | 1274.333 | -1.20% | -2.31% |
| 33_Tfidf_limit_zero | 2.667 | 2.834 | 2.791 | -1.52% | +4.65% |
| 33_Tfidf_multi | 2614.125 | 2644.250 | 2664.333 | +0.76% | +1.92% |
| 33_Tfidf_punctuation | 144.250 | 146.584 | 141.875 | -3.21% | -1.65% |
| 33_Tfidf_repeated | 1933.542 | 1838.500 | 1865.375 | +1.46% | -3.53% |
| 33_Tfidf_scope | 1867.875 | 1767.875 | 1771.083 | +0.18% | -5.18% |
| 33_Tfidf_single | 1835.584 | 1774.125 | 1802.833 | +1.62% | -1.78% |
| 33_Tfidf_unicode | 2033.083 | 1989.000 | 2023.667 | +1.74% | -0.46% |
| 33_Tfidf_unknown | 19.792 | 19.458 | 19.458 | +0.00% | -1.69% |
| 33_Tfidf_unknown_added | 1917.125 | 1951.667 | 1973.708 | +1.13% | +2.95% |
| 33_Tfidf_wide | 26141.333 | 26429.958 | 26585.292 | +0.59% | +1.70% |
| 4_Bm25_disjoint | 108.542 | 105.500 | 106.250 | +0.71% | -2.11% |
| 4_Bm25_limit_one | 185.875 | 184.792 | 183.666 | -0.61% | -1.19% |
| 4_Bm25_limit_zero | 2.667 | 2.792 | 2.750 | -1.50% | +3.11% |
| 4_Bm25_multi | 537.167 | 550.042 | 559.500 | +1.72% | +4.16% |
| 4_Bm25_punctuation | 144.250 | 143.958 | 146.458 | +1.74% | +1.53% |
| 4_Bm25_repeated | 431.542 | 338.667 | 334.500 | -1.23% | -22.49% |
| 4_Bm25_scope | 232.833 | 232.208 | 236.250 | +1.74% | +1.47% |
| 4_Bm25_single | 317.042 | 317.291 | 323.375 | +1.92% | +2.00% |
| 4_Bm25_unicode | 608.333 | 598.375 | 602.667 | +0.72% | -0.93% |
| 4_Bm25_unknown | 19.917 | 19.250 | 19.334 | +0.44% | -2.93% |
| 4_Bm25_unknown_added | 417.542 | 424.083 | 420.750 | -0.79% | +0.77% |
| 4_Bm25_wide | 7352.583 | 7414.000 | 7276.584 | -1.85% | -1.03% |
| 4_Tfidf_disjoint | 109.459 | 107.541 | 106.250 | -1.20% | -2.93% |
| 4_Tfidf_limit_one | 208.792 | 208.542 | 211.417 | +1.38% | +1.26% |
| 4_Tfidf_limit_zero | 2.666 | 3.625 | 2.750 | -24.14% | +3.15% |
| 4_Tfidf_multi | 573.125 | 573.750 | 574.917 | +0.20% | +0.31% |
| 4_Tfidf_punctuation | 146.084 | 143.459 | 147.416 | +2.76% | +0.91% |
| 4_Tfidf_repeated | 461.333 | 362.250 | 358.959 | -0.91% | -22.19% |
| 4_Tfidf_scope | 237.375 | 239.875 | 241.375 | +0.63% | +1.69% |
| 4_Tfidf_single | 343.792 | 347.000 | 346.125 | -0.25% | +0.68% |
| 4_Tfidf_unicode | 635.041 | 635.458 | 630.709 | -0.75% | -0.68% |
| 4_Tfidf_unknown | 19.875 | 19.167 | 19.333 | +0.87% | -2.73% |
| 4_Tfidf_unknown_added | 443.250 | 445.750 | 439.125 | -1.49% | -0.93% |
| 4_Tfidf_wide | 7249.083 | 7223.416 | 7233.541 | +0.14% | -0.21% |

Higher candidate medians versus 0.4.23: **49 of 96**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](https://github.com/LEVOGNE/airust/blob/v0.4.24/docs/measurements/0.4.24/small-confirmation-summary.json). No threshold-based rejection or removal of slow timing samples is used.

## Direct canonical search with explicit control modes

Fourteen processes (seven alternating pairs) retain one warmed Engine each. Ten rounds rotate plain, deadline and progress within each query/ranker pair: **8,400 timed Retriever calls**, excluding loading/readiness, stdio and profiling. Each progress request includes an observer and the same deadline/cancellation setup. Every signature agrees; **140 complete original response JSON pairs match**, and control modes also match plain output. Full readiness source text stays private. These are direct retrieval boundaries, not MCP round trips or capacity evidence.

| Control | Ranker | 0.4.23 median / p95 ms | 0.4.24 median / p95 ms | Median change |
| --- | --- | --- | --- | --- |
| plain | Bm25 | 1.362 / 2.763 | 1.350 / 2.777 | -0.84% |
| plain | Tfidf | 1.403 / 3.118 | 1.392 / 3.248 | -0.79% |
| deadline | Bm25 | 1.623 / 3.387 | 1.630 / 3.439 | +0.48% |
| deadline | Tfidf | 1.785 / 3.770 | 1.760 / 3.725 | -1.37% |
| progress | Bm25 | 1.633 / 3.332 | 1.614 / 3.401 | -1.17% |
| progress | Tfidf | 1.786 / 3.964 | 1.761 / 3.885 | -1.39% |

| Query | Candidates | Progress BM25 ms: old → new | Progress TF-IDF ms: old → new |
| --- | --- | --- | --- |
| 0 | 12365 | 1.626 → 1.581 | 1.808 → 1.765 |
| 1 | 13864 | 1.650 → 1.622 | 1.842 → 1.808 |
| 2 | 24299 | 2.948 → 2.870 | 3.151 → 3.136 |
| 3 | 596 | 0.312 → 0.305 | 0.302 → 0.297 |
| 4 | 15403 | 1.725 → 1.702 | 1.830 → 1.851 |
| 5 | 24140 | 2.787 → 2.707 | 2.984 → 2.898 |
| 6 | 3358 | 0.630 → 0.615 | 0.670 → 0.660 |
| 7 | 29600 | 2.740 → 2.631 | 3.096 → 3.148 |
| 8 | 11504 | 1.497 → 1.460 | 1.596 → 1.569 |
| 9 | 0 | 0.002 → 0.002 | 0.001 → 0.001 |

## Separate canonical three-version reference

Five rotating triples compare fresh exact 0.4.22/0.4.23/candidate servers on canonical knowledge with requested progress. Fifteen servers validate **1,020 actual schema replies**; **680 complete structured comparisons** against the candidate match by RPC ID except checked status version and version-qualified schema URNs. Schemas, source ranges and inputs agree. This control is separate from the two ordinary campaigns. It checks whether the previous large-query benefit remains visible against the older release; it does not establish the cause of any historical regression.

| Version | Ranker | Initial median / p95 ms | Repeated median / p95 ms | Peak child RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.22 | bm25 | 3.182 / 7.739 | 2.813 / 6.194 | 1725.109 |
| 0.4.22 | tfidf | 24.928 / 41.455 | 3.294 / 7.000 | 1725.109 |
| 0.4.23 | bm25 | 3.166 / 4.963 | 2.778 / 4.517 | 1725.312 |
| 0.4.23 | tfidf | 21.324 / 40.444 | 3.381 / 6.300 | 1725.312 |
| 0.4.24 | bm25 | 3.181 / 6.937 | 3.394 / 6.814 | 1724.766 |
| 0.4.24 | tfidf | 25.694 / 44.442 | 4.107 / 8.395 | 1724.766 |

| Query | 0.4.22 repeated TF-IDF ms | 0.4.23 repeated TF-IDF ms | 0.4.24 repeated TF-IDF ms |
| --- | --- | --- | --- |
| 0 | 3.251 | 3.405 | 4.102 |
| 1 | 3.323 | 3.265 | 4.057 |
| 2 | 4.961 | 5.607 | 6.590 |
| 3 | 0.954 | 0.933 | 1.024 |
| 4 | 3.163 | 3.502 | 4.299 |
| 5 | 5.394 | 5.650 | 6.809 |
| 6 | 1.519 | 1.633 | 1.757 |
| 7 | 5.415 | 6.074 | 6.211 |
| 8 | 3.254 | 3.092 | 3.857 |
| 9 | 0.181 | 0.192 | 0.180 |

## Canonical first-pass heap traffic

Two instrumented processes index all 167,942 records before profiling the same ten initial TF-IDF queries, then drop results before net deltas. The existing direct candidate collection remains; the previous large-query posting cursors and norm memo/cache still exist. Heap traffic, net persistent hints, stack state and whole-process RSS are different costs. Instrumented durations are excluded from latency comparisons.

| Metric | 0.4.23 | 0.4.24 |
| --- | --- | --- |
| Retained hint bytes | 2228312 | 2228312 |
| Retained hint blocks | 2 | 2 |
| Total query allocated bytes | 7212357 | 7212357 |
| Total query allocation blocks | 401 | 401 |

## Development trials and host-load limits

The full three-triple development MCP pilot is retained separately from all final campaigns. Its source snapshots include the MCP implementation and Core; all complete compared structured results agree. Initial/repeated data stay separate and no final sample is replaced by a pilot or a faster rerun. Earlier releases retain their own rejected trials. These are development observations, not fresh production or semantic holdouts.

Two-second monitoring retains **573 snapshots**, including **270 with other rustc processes**, maximum 4. All managed compilers finish before timing. Other workloads are not stopped, and snapshots do not prove continuous idleness or identify a timing cause. CPU counts include the active benchmark workers.

| Stage start | Other rustc count | Other rustc CPU total | Processes over 50% CPU |
| --- | --- | --- | --- |
| small-mcp-standard | 0 | 0.000 | 1 |
| small-reference | 0 | 0.000 | 1 |
| large-standard | 0 | 0.000 | 2 |
| wide-standard | 3 | 457.800 | 4 |
| retrieval | 1 | 200.000 | 3 |
| query | 0 | 0.000 | 2 |
| build | 2 | 0.000 | 3 |
| controlled | 0 | 0.000 | 3 |
| mcp | 0 | 0.000 | 2 |
| corpus | 1 | 100.000 | 4 |
| mcp-confirmation | 0 | 0.000 | 2 |
| reference | 2 | 282.700 | 5 |
| wide-confirmation | 0 | 0.000 | 4 |
| wide-extended | 0 | 0.000 | 4 |
| large-confirmation | 1 | 94.400 | 3 |
| small-confirmation | 0 | 0.000 | 2 |
| small-mcp-confirmation | 0 | 0.000 | 1 |
| limits | 0 | 0.000 | 1 |

All observed slower cases remain in their own distributions; no sample is replaced because of load. One machine, randomized hashing, exposed questions, tuned fixtures and finite trials limit generalization.

## Retained Core controls

These controls retain positive and negative timing changes. Single known-term requests retain the direct candidate path and exactly two retain ordered merging. Wider requests keep the heap/set strategies; large frequency cursors remain available afterward. Large sorted candidate sets can use the bounded document cursor after 128 candidates; earlier shared explanation intersection remains. Read every row independently, including slower cases. Seven alternating pairs give fourteen plain processes per family. Norms are initialized for warm timing; two separate DHAT processes measure allocations. Instrumented durations are excluded from latency comparisons. Every result signature agrees and temporary-call retained deltas are zero.

### Small warm Core: 84,000 calls on 1/4/16 entries

| Fixture | Median change | Blocks/call: old → new | Allocated bytes/call: old → new |
| --- | --- | --- | --- |
| tiny16_bm25 | -16.96% | 26.0 → 26.0 | 2156.0 → 2156.0 |
| tiny16_tfidf | -0.01% | 26.0 → 26.0 | 2156.0 → 2156.0 |
| tiny4_bm25 | -19.66% | 12.0 → 12.0 | 824.0 → 824.0 |
| tiny4_tfidf | -13.90% | 12.0 → 12.0 | 824.0 → 824.0 |
| tiny_bm25 | +1.92% | 6.0 → 6.0 | 398.0 → 398.0 |
| tiny_tfidf | -18.96% | 6.0 → 6.0 | 398.0 → 398.0 |

### All 25 warm controls: 282,800 calls

| Fixture | Median change | Blocks/call: old → new | Allocated bytes/call: old → new |
| --- | --- | --- | --- |
| coverage_repeated | -0.81% | 1047.0 → 1047.0 | 92942.0 → 92942.0 |
| coverage_small | +1.37% | 784.0 → 784.0 | 78046.0 → 78046.0 |
| excerpt_ascii | +0.04% | 762.0 → 762.0 | 252438.0 → 252438.0 |
| excerpt_mixed | +0.33% | 569.0 → 569.0 | 417113.0 → 417113.0 |
| excerpt_unicode | +2.62% | 197.0 → 197.0 | 72254.0 → 72254.0 |
| long_repeated | -0.40% | 4.0 → 4.0 | 18798.0 → 18798.0 |
| mixed_unicode | -2.42% | 7.0 → 7.0 | 3155.0 → 3155.0 |
| punctuation | -1.18% | 1.0 → 1.0 | 768.0 → 768.0 |
| relevant_retrieval | +3.02% | 812.0 → 812.0 | 93562.0 → 93562.0 |
| repeated_search | +0.39% | 31.0 → 31.0 | 18156.0 → 18156.0 |
| repeated_short | -4.62% | 4.0 → 4.0 | 1656.0 → 1656.0 |
| sparse_bm25_128 | +1.42% | 359.0 → 359.0 | 29074.0 → 29074.0 |
| sparse_bm25_33 | +0.79% | 241.0 → 241.0 | 16309.0 → 16309.0 |
| sparse_bm25_512 | +0.94% | 1337.0 → 1337.0 | 119124.0 → 119124.0 |
| sparse_tfidf_128 | +3.73% | 474.0 → 474.0 | 35140.0 → 35140.0 |
| sparse_tfidf_33 | +1.86% | 241.0 → 241.0 | 16309.0 → 16309.0 |
| sparse_tfidf_512 | +0.26% | 1337.0 → 1337.0 | 119124.0 → 119124.0 |
| tfidf_union_32 | +0.31% | 244.0 → 244.0 | 19834.0 → 19834.0 |
| tfidf_union_33 | +2.39% | 296.0 → 296.0 | 27854.0 → 27854.0 |
| tfidf_warm | -1.38% | 71.0 → 71.0 | 19335.0 → 19335.0 |
| union_32 | +0.24% | 244.0 → 244.0 | 19834.0 → 19834.0 |
| union_33 | -1.42% | 296.0 → 296.0 | 27854.0 → 27854.0 |
| unique_long | -2.39% | 299.0 → 299.0 | 32672.0 → 32672.0 |
| unique_search | +1.10% | 72.0 → 72.0 | 19350.0 → 19350.0 |
| unique_short | -1.78% | 296.0 → 296.0 | 18548.0 → 18548.0 |

### Cold norm and heap controls

Six fixtures make twenty fresh-index-clone calls each across fourteen processes: 1,680 calls. Clone/setup is outside timing. Process/instruction/filesystem caches remain; cold means norm hints, not disks. Separate heap runs drop results before net retained measurements. The existing inline/optional heap word memo and persistent norm storage remain. Large small-term requests may add up to four temporary posting cursors; canonical allocation traffic below records that cost. Heap profiles exclude stack state/control fields and temporary frames.

| Fixture | Old median / p95 ns | New median / p95 ns |
| --- | --- | --- |
| index | 111104.500 / 124747.600 | 110437.500 / 124931.250 |
| wide | 31521.000 / 41629.200 | 41958.000 / 63891.700 |
| sparse | 419458.500 / 478335.100 | 423896.000 / 487727.400 |
| tiny | 459.000 / 1338.150 | 459.000 / 1336.050 |
| tiny4 | 709.000 / 1086.050 | 750.000 / 1086.050 |
| tiny16 | 2250.000 / 3864.250 | 2375.000 / 3627.100 |

| Cold fixture | Allocated bytes: old → new | Retained hint bytes: old → new |
| --- | --- | --- |
| index | 91976 → 91976 | 34904 → 34904 |
| wide | 19165 → 19165 | 4440 → 4440 |
| sparse | 140704 → 140704 | 8792 → 8792 |
| tiny | 554 → 554 | 156 → 156 |
| tiny4 | 1124 → 1124 | 224 → 224 |
| tiny16 | 3288 → 3288 | 632 → 632 |

Canonical first-pass heap records, all samples, blocks and cold tails remain in public JSON. Retained hints, heap traffic and whole-process RAM are different measures.

## Finite mixed control, documents and construction

The 6,000-entry mixed control retains four continuous warm readers while 100 initially uncached-group driver queries finish, then makes two quiescent probes. Five alternating pairs give 1,000 driver calls and twenty quiescent probes. Warm counts vary with the stop condition and new-group calls can become warm. This finite control is not fixed-load throughput, capacity or fairness evidence.

| Mixed work, ns | Old median / p95 | New median / p95 |
| --- | --- | --- |
| new_group_query_ns | 195250.500 / 334377.100 | 251312.500 / 347643.750 |
| post_readers_first_ns | 200166.000 / 215441.600 | 183042.000 / 236067.200 |
| post_readers_second_ns | 85208.000 / 89000.600 | 90458.000 / 106708.200 |

The private document corpus retains 72 Markdown files, 2,755,468 bytes and 1,910 chunks. All 342 authored questions (276 positive, 66 unsupported) are exposed regressions, not fresh holdouts or logged production queries. Literal BM25, prepared BM25 and coverage retain six passages, 100 candidates, 2,000 bytes/pass and 12,000 total bytes. All **1,026 complete original response JSON strings match byte-for-byte**, including scores, ordering, evidence, source text and positions. One warm-up and five repeats/question/configuration/version give **10,260 timed round trips**. These three configurations are BM25 controls; independent score oracles and actual protocol pairs cover TF-IDF. No new relevance judgment or model reply is produced.

All document timing aggregates and per-question results remain in the records. Construction uses forty plain processes on canonical data and three synthetic distributions, plus six separate profiles. Parsing/generation is outside construction timing; stats/signatures/live derived heap agree. The execution/shared-index worker checks all 167,942 entries through 128 changes/restoration, rebuilds, JSON round trips, retained clones, parallel readers and cancellation/deadline rejection. Portable exports and source hashes agree.

### Document round trips

| Case | Old median / p95 ms | New median / p95 ms |
| --- | --- | --- |
| baseline | 0.875 / 1.403 | 0.879 / 1.420 |
| prepared | 1.006 / 1.566 | 0.999 / 1.576 |
| evidence | 0.983 / 1.569 | 0.997 / 1.585 |

### Construction only

| Case | Old median / p95 ms | New median / p95 ms |
| --- | --- | --- |
| canonical | 5173.806 / 5855.097 | 5118.921 / 5413.131 |
| ascii | 14.461 / 14.762 | 14.799 / 16.961 |
| mixed | 34.820 / 35.868 | 34.954 / 38.162 |
| long_tokens | 1.561 / 1.583 | 1.574 / 1.974 |

## Limits and preservation

Measurements use one Apple M5 Max on battery power at campaign start with 64 GB, Rust 1.98.0 and Python 3.15.2. Stage-start CPU/compiler snapshots remain. Own compilers finish before timing; this does not establish continuously idle hardware. Randomized hashing, scheduling, one machine/corpus, exposed questions and finite process counts limit generalization. The separate development pilot remains; assess final per-case medians and tails independently. Host-load sampling records observations rather than continuous idleness. No universal latency, total-memory, semantic-quality, token-saving or security-audit claim is made.

Canonical knowledge remains 167,942 entries from 35 PDFs and 49,086 physical pages, SHA-256 `13168d87872a137355fd8499c90a6084f52243a2ec7456f29b5fcb2a1c7423c3`. The PDF parser is unchanged; this is not a fresh large-PDF extraction campaign. Lexical overlap and exact quotes do not prove factual support or entailment. No model, daemon or DenkMaps integration is added.

Preserve PDFs, knowledge, snapshots, all rejected/selected pilot and final measurements, slower results, raw transcripts, profiles, exact small executables and locks. Raw measurement JSON stays in the tagged GitHub repository and local archives, outside the registry source package; API guides, Markdown reports, method source, examples and required fixtures remain in that package. After verified publication archive package/proof, then delete only the named temporary Cargo/Miri targets. Default features remain empty.

Power metadata correction: the original AC label was hard-coded and incorrect. The preserved environment record reports Battery Power at campaign start. Power was not continuously sampled. Timing samples remain unchanged; see [benchmark errata](BENCHMARK-ERRATA.md).
