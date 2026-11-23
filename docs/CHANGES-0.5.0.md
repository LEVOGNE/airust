# AIRust 0.5.0: verified Core and optional MCP stability

This release stabilizes the existing Core and optional MCP contracts. Both src/index.rs and src/mcp.rs are byte-for-byte unchanged from published 0.4.25. It retains the measured fixes from 0.4.22 through 0.4.25 and adds an independent stateful regression test, finite million-entry/retained-clone controls and a documented Cargo compatibility boundary. There is no new ranking algorithm, model, default feature, server dependency or DenkMaps integration.

The 0.4.25 knowledge loader frees raw owned JSON after parsing and before index construction. Original records, files, metadata, bounded reading/errors and saved formats remain. This inherited change is remeasured below against exact 0.4.25 and the earlier 0.4.21 reference; runtime equality with the baseline does not imply equal timing on a shared host. The separate development MCP pilot preserves full compared responses; earlier startup development trials remain in the 0.4.25 archive, not new 0.5.0 samples.

## Independent state and finite scale verification

A new native regression performs 250 fixed-seed mutations against an independently maintained BTreeMap of records. After each step it compares complete entries/stats and exact f64 score bits, ordering, matched terms, both rankers, ten multilingual, normalized-empty and unknown query cases, three scopes and three limits with a rebuilt index. It exercises replacements, removals, invalid updates without partial state, periodic JSON restoration, warmed caches and retained clones read in parallel. Old snapshots and their saved JSON stay unchanged. This is a differential state/cache test, not an independent proof that the ranking formula is ideal; existing hand-calculated score and normalization oracles remain.

The finite Miri form uses eight mutation steps, four queries, two scopes, one nonzero limit and sequential retained readers. The full native matrix remains larger. Miri does not execute the million-entry workload or native thread/transport/PDF/ONNX paths. These are scoped checks, not an independent audit or a guarantee of zero bugs.

The unchanged public stress example is built against exact 0.4.21, exact 0.4.25 and 0.5.0. Seven rotating triples load one million short synthetic records, search selective/common/absent terms, update 100 records, delete 20 and run four parallel readers. Three more triples retain three clones of a 500,000-entry index, mutate one and verify old views. All 30 children must exit cleanly; million-entry score signatures agree within every triple. Each has a 600-second watchdog; timeout is a failure.

Build time includes lazy entry generation. Query entries below summarize each process's category median; they are not pooled independent calls. Exact-child wait4 RSS is separate from heap allocation profiles. The synthetic vocabulary and short records do not establish real-document capacity, endurance, security or fresh PDF parsing.

| Case / version | Generation + build median ms | Exact child peak median MiB | Selective category median ms | Common category median ms |
| --- | --- | --- | --- | --- |
| million / 0.4.21 | 1872.373 | 1305.078 | 0.650 | 41.457 |
| million / 0.4.25 | 1826.228 | 1304.984 | 0.628 | 40.978 |
| million / 0.5.0 | 1841.648 | 1305.031 | 0.631 | 41.530 |
| retained-clones / 0.4.21 | 894.449 | 2335.609 | n/a | n/a |
| retained-clones / 0.4.25 | 869.745 | 2335.641 | n/a | n/a |
| retained-clones / 0.5.0 | 882.115 | 2335.688 | n/a | n/a |

All update/delete/concurrent/clone distributions, negative changes and individual observations remain in [the finite stress record](https://github.com/LEVOGNE/airust/blob/v0.5.0/docs/measurements/0.5.0/stability-stress-reports.json). Read [migration](MIGRATION-0.5.0.md) for explicit Cargo selection, preserved API calls, snapshot positions and MCP schema refresh.

## Startup memory and time: separate final measurements

Each of two preplanned native campaigns runs seven rotating exact 0.4.21/0.4.25/0.5.0 triples for each of two cases: **42 fresh MCP server children per campaign, 84 total**. Every child initializes, reports the expected version/count/read-only state with no occupied workers, receives EOF and exits successfully. No search or norm warming occurs. Filesystem/process caches are warm. Peak RSS comes from wait4 for that exact child, not cumulative RUSAGE_CHILDREN polluted by other subprocesses. Post-ready RSS is a separate ps observation. All samples and input hashes remain.

### Standard startup-only campaign

| Case / version | Startup median / p95 ms | Exact child peak median MiB | Post-ready RSS median MiB | Peak vs 0.4.25 |
| --- | --- | --- | --- | --- |
| canonical / 0.4.21 | 4813.732 / 4844.284 | 1719.953 | 1719.953 | +0.01% |
| canonical / 0.4.25 | 4807.830 / 4837.746 | 1719.812 | 1719.797 | +0.00% |
| canonical / 0.5.0 | 4823.588 / 4912.287 | 1719.953 | 1719.953 | +0.01% |
| tiny16 / 0.4.21 | 3.548 / 3.782 | 5.078 | 5.016 | -1.52% |
| tiny16 / 0.4.25 | 3.542 / 3.668 | 5.156 | 5.078 | +0.00% |
| tiny16 / 0.5.0 | 3.481 / 3.636 | 5.172 | 5.109 | +0.30% |

CPU, individual startup/peak/live samples and all distributions remain in [the full startup record](https://github.com/LEVOGNE/airust/blob/v0.5.0/docs/measurements/0.5.0/startup-standard-reports.json). No smaller or slower case is removed.

### Confirmation startup-only campaign

| Case / version | Startup median / p95 ms | Exact child peak median MiB | Post-ready RSS median MiB | Peak vs 0.4.25 |
| --- | --- | --- | --- | --- |
| canonical / 0.4.21 | 4804.967 / 4822.769 | 1719.984 | 1719.984 | +0.01% |
| canonical / 0.4.25 | 4797.534 / 4810.051 | 1719.797 | 1719.797 | +0.00% |
| canonical / 0.5.0 | 4819.104 / 4860.146 | 1719.938 | 1719.938 | +0.01% |
| tiny16 / 0.4.21 | 3.503 / 3.666 | 5.078 | 5.016 | -1.52% |
| tiny16 / 0.4.25 | 3.458 / 3.487 | 5.156 | 5.078 | +0.00% |
| tiny16 / 0.5.0 | 3.447 / 3.866 | 5.172 | 5.125 | +0.30% |

CPU, individual startup/peak/live samples and all distributions remain in [the full startup record](https://github.com/LEVOGNE/airust/blob/v0.5.0/docs/measurements/0.5.0/startup-confirmation-reports.json). No smaller or slower case is removed.

## Separate startup allocation profiles

Six DHAT-instrumented processes load both cases across all three versions. Profiles include file reading, owned parsing, index and schema construction. Their maximum/live heap counts exclude profiler metadata and are not process RSS. Instrumented durations are never used as native latency evidence. Original profiles stay local; sanitized counters are published.

| Case / version | Maximum live heap before server drop MiB | Live after load MiB | Allocation traffic MiB | Live after server drop bytes |
| --- | --- | --- | --- | --- |
| tiny16 / 0.4.21 | 0.549 | 0.291 | 0.965 | 237410 |
| tiny16 / 0.4.25 | 0.545 | 0.291 | 0.965 | 237410 |
| tiny16 / 0.5.0 | 0.545 | 0.291 | 0.965 | 237405 |
| canonical / 0.4.21 | 1644.433 | 1368.677 | 3025.844 | 237410 |
| canonical / 0.4.25 | 1388.433 | 1368.677 | 3025.844 | 237410 |
| canonical / 0.5.0 | 1388.433 | 1368.677 | 3025.844 | 237405 |

Different allocators, page reuse and OS accounting can produce different peak/live RSS changes from DHAT heap changes. Dropping the raw input does not shrink the retained index, impose an OS quota or prove a faster search. This release publishes both allocation and process observations, including negative timings.

## Executed validation and unchanged API

The full matrix passes **253 default tests, 909 all-feature checks including 39 doctests, 39 isolated feature entries and five Python tests**. Strict Clippy/docs, Rust 1.89 provider-neutral and Rust 1.92 all-feature checks pass. API comparison against 0.4.25 reports **229 pass, 31 skip**. The independent Rust 1.89 consumer, unchanged PDF backend's 14 regressions and 381-file provenance check pass. The dependency audit reports zero known vulnerabilities; maintenance notices remain. Existing malformed/oversized/UTF-8/JSON input, hostile protocol, slow-reader, progress, cancellation, deadline, disconnect, final-reply and shutdown tests remain native. The deliberately ignored crash-writer helper is launched by its parent recovery test.

**38 finite Tree Borrows Miri cases** pass: the previous 37 plus the new stateful regression with eight mutation steps and a reduced query/scope/limit matrix. The native test keeps all 250 steps, all multilingual queries and parallel retained readers. No extra Miri coverage is inferred for filesystem startup or native I/O. Native Tokio/PDF/FFI/ONNX execution stays outside that interpreter scope. Only injected extra floating-point rounding is disabled, not alias/memory checking. This is not an independent penetration, security or soundness audit. Extracted-package tests, four exact-commit CI jobs, a fresh Rust 1.89 registry consumer and tag/VCS/checksum/release-asset agreement gate publication completion.

Engine, SearchOptions, Retriever, WorkControl, McpServer and McpLimits remain. Existing knowledge and saved formats need no migration. Read [API](API.md), [cookbook](API-COOKBOOK.md), [MCP](MCP.md), [Core performance](CORE-PERFORMANCE.md) and [security limits](MCP-SECURITY.md). Source UTF-8 positions remain bound to their exact snapshot revision, not PDF coordinates.

## Frozen method and regression controls

Baseline is exact registry 0.4.25; reference is exact registry 0.4.21. Published archives/checksums and VCS identities are verified before timing. Twenty-one actual worker source directories, manifests, locks, method programs, measured binaries, runtime and inputs are hashed and retained. No managed compiler runs during final timings. The [method](validation-0.5.0/README.md) describes the retained inputs and setup requirements; full local orchestration depends on private corpus/proof records. It is not a public standalone reproduction of private source documents. Read [power metadata corrections](BENCHMARK-ERRATA.md); power is observed at campaign start, not continuously controlled.

Both ordinary MCP campaigns, both six-case small-MCP campaigns, both large-corpus guards, three broad controls, exact-limit errors/recovery, document retrieval, cold/warm/controlled Core, corpus export and allocation controls remain. There are **555 fresh RPC-test servers**, separate from the 84 startup-only servers and six allocation-profile loader processes. The fixed 342 authored questions are exposed, not new production or semantic holdouts. There is no fresh LLM/embedding answer-quality or token-saving claim.

### Standard small-MCP campaign

| Case | Ranker | 0.4.21 repeated median / p95 ms | 0.4.25 repeated median / p95 ms | 0.5.0 repeated median / p95 ms | vs 0.4.25 |
| --- | --- | --- | --- | --- | --- |
| tiny | bm25 | 0.117 / 0.128 | 0.114 / 0.138 | 0.113 / 0.133 | -0.57% |
| tiny | tfidf | 0.117 / 0.153 | 0.115 / 0.171 | 0.116 / 0.150 | +0.69% |
| tiny4 | bm25 | 0.173 / 0.201 | 0.165 / 0.196 | 0.164 / 0.195 | -0.58% |
| tiny4 | tfidf | 0.170 / 0.190 | 0.163 / 0.188 | 0.163 / 0.181 | -0.28% |
| tiny16 | bm25 | 0.192 / 0.218 | 0.182 / 0.202 | 0.186 / 0.212 | +2.31% |
| tiny16 | tfidf | 0.194 / 0.227 | 0.183 / 0.209 | 0.184 / 0.227 | +0.32% |
| tiny-no-progress | bm25 | 0.068 / 0.078 | 0.066 / 0.084 | 0.066 / 0.080 | +0.67% |
| tiny-no-progress | tfidf | 0.068 / 0.095 | 0.065 / 0.114 | 0.065 / 0.129 | +0.16% |
| tiny4-no-progress | bm25 | 0.127 / 0.165 | 0.116 / 0.148 | 0.115 / 0.147 | -0.63% |
| tiny4-no-progress | tfidf | 0.125 / 0.145 | 0.115 / 0.131 | 0.115 / 0.137 | +0.02% |
| tiny16-no-progress | bm25 | 0.146 / 0.202 | 0.138 / 0.183 | 0.135 / 0.177 | -1.63% |
| tiny16-no-progress | tfidf | 0.146 / 0.173 | 0.137 / 0.166 | 0.136 / 0.160 | -0.94% |

All initial/repeated per-query means, medians, p95, process sums, startup, RSS and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.5.0/docs/measurements/0.5.0/small-mcp-standard-summary.json). No slower sample is removed.

### Confirmation small-MCP campaign

| Case | Ranker | 0.4.21 repeated median / p95 ms | 0.4.25 repeated median / p95 ms | 0.5.0 repeated median / p95 ms | vs 0.4.25 |
| --- | --- | --- | --- | --- | --- |
| tiny | bm25 | 0.117 / 0.134 | 0.112 / 0.126 | 0.112 / 0.129 | +0.15% |
| tiny | tfidf | 0.117 / 0.147 | 0.113 / 0.152 | 0.114 / 0.140 | +0.31% |
| tiny4 | bm25 | 0.174 / 0.218 | 0.167 / 0.211 | 0.166 / 0.207 | -0.41% |
| tiny4 | tfidf | 0.173 / 0.195 | 0.165 / 0.186 | 0.166 / 0.194 | +0.34% |
| tiny16 | bm25 | 0.194 / 0.217 | 0.185 / 0.208 | 0.185 / 0.209 | +0.14% |
| tiny16 | tfidf | 0.196 / 0.236 | 0.187 / 0.221 | 0.186 / 0.223 | -0.79% |
| tiny-no-progress | bm25 | 0.069 / 0.094 | 0.064 / 0.074 | 0.065 / 0.080 | +1.56% |
| tiny-no-progress | tfidf | 0.068 / 0.110 | 0.064 / 0.099 | 0.065 / 0.100 | +0.49% |
| tiny4-no-progress | bm25 | 0.128 / 0.168 | 0.116 / 0.147 | 0.117 / 0.153 | +0.84% |
| tiny4-no-progress | tfidf | 0.126 / 0.149 | 0.114 / 0.131 | 0.116 / 0.134 | +1.42% |
| tiny16-no-progress | bm25 | 0.148 / 0.180 | 0.135 / 0.181 | 0.137 / 0.185 | +1.03% |
| tiny16-no-progress | tfidf | 0.148 / 0.166 | 0.137 / 0.149 | 0.137 / 0.161 | +0.46% |

All initial/repeated per-query means, medians, p95, process sums, startup, RSS and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.5.0/docs/measurements/0.5.0/small-mcp-confirmation-summary.json). No slower sample is removed.

## Retained large and broad MCP controls

Each large campaign has seven rotating three-version triples: together 42 servers, 2,856 actual schema-valid replies and 1,904 complete comparisons. The three broad campaigns retain 63 servers, 4,284 actual schema replies and 2,856 comparisons. The two small campaigns retain 252 servers, 17,136 actual schema replies and 11,424 comparisons. Status version and schema URNs are the only asserted version differences. Initial/repeated/ranker/query/startup/RSS boundaries remain separate. These results check compatibility and observed performance, not universal improvement.

### Standard large-corpus campaign

| Ranker / phase | 0.4.21 median / p95 ms | 0.4.25 median / p95 ms | 0.5.0 median / p95 ms | vs 0.4.25 | vs 0.4.21 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 2.497 / 3.986 | 2.425 / 3.854 | 2.351 / 3.897 | -3.07% | -5.88% |
| tfidf / initial | 16.020 / 28.995 | 15.985 / 29.167 | 16.300 / 28.694 | +1.97% | +1.75% |
| bm25 / repeated | 2.128 / 3.581 | 2.039 / 3.449 | 2.081 / 3.462 | +2.04% | -2.21% |
| tfidf / repeated | 2.196 / 3.736 | 2.134 / 3.650 | 2.135 / 3.662 | +0.06% | -2.79% |

| Query | Repeated TF-IDF median ms: 0.4.21 / 0.4.25 / 0.5.0 | vs 0.4.25 | Initial TF-IDF mean ms: 0.4.25 → 0.5.0 |
| --- | --- | --- | --- |
| 0 | 2.196 / 2.140 / 2.146 | +0.28% | 18.683 → 18.907 |
| 1 | 2.300 / 2.245 / 2.253 | +0.39% | 19.120 → 19.539 |
| 2 | 3.532 / 3.549 / 3.464 | -2.39% | 29.241 → 29.384 |
| 3 | 0.667 / 0.652 / 0.649 | -0.48% | 1.418 → 1.447 |
| 4 | 2.255 / 2.179 / 2.159 | -0.91% | 12.981 → 13.106 |
| 5 | 3.523 / 3.401 / 3.385 | -0.48% | 22.202 → 23.170 |
| 6 | 1.094 / 1.046 / 1.044 | -0.16% | 4.348 → 4.431 |
| 7 | 3.626 / 3.477 / 3.542 | +1.89% | 22.823 → 23.203 |
| 8 | 1.984 / 1.947 / 1.983 | +1.85% | 2.252 → 2.244 |
| 9 | 0.103 / 0.101 / 0.103 | +1.23% | 0.159 → 0.158 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.21 | 134.081 | 42.625 | 4686.829 | 1724.172 |
| 0.4.25 | 134.454 | 41.348 | 4695.228 | 1724.016 |
| 0.5.0 | 133.031 | 41.229 | 4716.444 | 1724.156 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.5.0/docs/measurements/0.5.0/large-standard-summary.json).

### Confirmation large-corpus campaign

| Ranker / phase | 0.4.21 median / p95 ms | 0.4.25 median / p95 ms | 0.5.0 median / p95 ms | vs 0.4.25 | vs 0.4.21 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 2.587 / 4.080 | 2.545 / 4.009 | 2.562 / 3.978 | +0.69% | -0.96% |
| tfidf / initial | 16.813 / 30.250 | 17.226 / 30.312 | 17.377 / 31.498 | +0.88% | +3.35% |
| bm25 / repeated | 2.150 / 3.545 | 2.093 / 3.484 | 2.092 / 3.548 | -0.06% | -2.70% |
| tfidf / repeated | 2.189 / 3.729 | 2.170 / 3.724 | 2.157 / 3.745 | -0.59% | -1.47% |

| Query | Repeated TF-IDF median ms: 0.4.21 / 0.4.25 / 0.5.0 | vs 0.4.25 | Initial TF-IDF mean ms: 0.4.25 → 0.5.0 |
| --- | --- | --- | --- |
| 0 | 2.144 / 2.133 / 2.157 | +1.15% | 21.509 → 22.153 |
| 1 | 2.284 / 2.230 / 2.274 | +1.97% | 20.495 → 21.144 |
| 2 | 3.604 / 3.609 / 3.532 | -2.14% | 30.688 → 31.611 |
| 3 | 0.685 / 0.663 / 0.658 | -0.69% | 1.464 → 1.458 |
| 4 | 2.271 / 2.276 / 2.241 | -1.54% | 13.772 → 13.875 |
| 5 | 3.476 / 3.508 / 3.434 | -2.10% | 23.604 → 23.879 |
| 6 | 1.081 / 1.088 / 1.075 | -1.15% | 4.494 → 4.515 |
| 7 | 3.538 / 3.564 / 3.643 | +2.22% | 23.116 → 23.627 |
| 8 | 2.008 / 2.009 / 2.023 | +0.73% | 2.259 → 2.286 |
| 9 | 0.104 / 0.102 / 0.100 | -1.53% | 0.158 → 0.164 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.21 | 138.715 | 42.467 | 4743.356 | 1724.125 |
| 0.4.25 | 140.817 | 42.378 | 4752.126 | 1724.047 |
| 0.5.0 | 143.733 | 42.921 | 4831.000 | 1724.141 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.5.0/docs/measurements/0.5.0/large-confirmation-summary.json).

### Standard broad-query campaign

| Ranker / phase | 0.4.21 median / p95 ms | 0.4.25 median / p95 ms | 0.5.0 median / p95 ms | vs 0.4.25 | vs 0.4.21 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 0.480 / 0.583 | 0.456 / 0.549 | 0.459 / 0.544 | +0.56% | -4.54% |
| tfidf / initial | 0.464 / 0.761 | 0.434 / 0.731 | 0.433 / 0.710 | -0.11% | -6.64% |
| bm25 / repeated | 0.419 / 0.618 | 0.388 / 0.615 | 0.384 / 0.567 | -0.85% | -8.34% |
| tfidf / repeated | 0.417 / 0.712 | 0.385 / 0.662 | 0.382 / 0.658 | -0.77% | -8.36% |

| Query | Repeated TF-IDF median ms: 0.4.21 / 0.4.25 / 0.5.0 | vs 0.4.25 | Initial TF-IDF mean ms: 0.4.25 → 0.5.0 |
| --- | --- | --- | --- |
| 0 | 0.405 / 0.367 / 0.372 | +1.30% | 0.449 → 0.456 |
| 1 | 0.378 / 0.348 / 0.343 | -1.44% | 0.416 → 0.387 |
| 2 | 0.417 / 0.391 / 0.381 | -2.44% | 0.446 → 0.427 |
| 3 | 0.692 / 0.648 / 0.634 | -2.19% | 0.728 → 0.708 |
| 4 | 0.426 / 0.372 / 0.368 | -1.02% | 0.375 → 0.378 |
| 5 | 0.408 / 0.370 / 0.375 | +1.21% | 0.366 → 0.363 |
| 6 | 0.413 / 0.379 / 0.383 | +1.07% | 0.437 → 0.449 |
| 7 | 0.681 / 0.627 / 0.635 | +1.35% | 0.668 → 0.651 |
| 8 | 0.375 / 0.338 / 0.350 | +3.49% | 0.353 → 0.356 |
| 9 | 0.690 / 0.627 / 0.628 | +0.20% | 0.665 → 0.658 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.21 | 5.176 | 9.729 | 3.824 | 7.906 |
| 0.4.25 | 4.899 | 9.047 | 3.833 | 7.719 |
| 0.5.0 | 4.845 | 9.026 | 3.612 | 7.688 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.5.0/docs/measurements/0.5.0/wide-standard-summary.json).

### Confirmation broad-query campaign

| Ranker / phase | 0.4.21 median / p95 ms | 0.4.25 median / p95 ms | 0.5.0 median / p95 ms | vs 0.4.25 | vs 0.4.21 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 0.485 / 0.577 | 0.449 / 0.540 | 0.460 / 0.538 | +2.58% | -5.07% |
| tfidf / initial | 0.462 / 0.770 | 0.436 / 0.697 | 0.432 / 0.717 | -0.86% | -6.43% |
| bm25 / repeated | 0.420 / 0.609 | 0.388 / 0.578 | 0.396 / 0.631 | +2.11% | -5.83% |
| tfidf / repeated | 0.418 / 0.710 | 0.384 / 0.660 | 0.389 / 0.686 | +1.41% | -6.75% |

| Query | Repeated TF-IDF median ms: 0.4.21 / 0.4.25 / 0.5.0 | vs 0.4.25 | Initial TF-IDF mean ms: 0.4.25 → 0.5.0 |
| --- | --- | --- | --- |
| 0 | 0.405 / 0.370 / 0.373 | +0.92% | 0.452 → 0.460 |
| 1 | 0.378 / 0.348 / 0.353 | +1.35% | 0.398 → 0.411 |
| 2 | 0.417 / 0.386 / 0.384 | -0.42% | 0.441 → 0.422 |
| 3 | 0.695 / 0.638 / 0.653 | +2.27% | 0.693 → 0.718 |
| 4 | 0.409 / 0.377 / 0.384 | +1.83% | 0.379 → 0.366 |
| 5 | 0.403 / 0.371 / 0.372 | +0.35% | 0.364 → 0.353 |
| 6 | 0.409 / 0.379 / 0.394 | +4.05% | 0.429 → 0.428 |
| 7 | 0.678 / 0.640 / 0.633 | -1.15% | 0.649 → 0.647 |
| 8 | 0.372 / 0.345 / 0.351 | +1.60% | 0.354 → 0.357 |
| 9 | 0.674 / 0.637 / 0.642 | +0.83% | 0.664 → 0.651 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.21 | 5.194 | 9.724 | 3.625 | 7.984 |
| 0.4.25 | 4.798 | 9.054 | 3.545 | 7.656 |
| 0.5.0 | 4.798 | 9.058 | 3.620 | 7.703 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.5.0/docs/measurements/0.5.0/wide-confirmation-summary.json).

### Extended broad-query campaign

| Ranker / phase | 0.4.21 median / p95 ms | 0.4.25 median / p95 ms | 0.5.0 median / p95 ms | vs 0.4.25 | vs 0.4.21 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 0.558 / 0.959 | 0.542 / 0.910 | 0.527 / 0.898 | -2.83% | -5.66% |
| tfidf / initial | 0.742 / 0.944 | 0.694 / 0.913 | 0.693 / 0.902 | -0.16% | -6.62% |
| bm25 / repeated | 0.531 / 1.043 | 0.500 / 0.987 | 0.498 / 0.963 | -0.40% | -6.28% |
| tfidf / repeated | 0.706 / 0.931 | 0.657 / 0.881 | 0.657 / 0.885 | -0.05% | -7.01% |

| Query | Repeated TF-IDF median ms: 0.4.21 / 0.4.25 / 0.5.0 | vs 0.4.25 | Initial TF-IDF mean ms: 0.4.25 → 0.5.0 |
| --- | --- | --- | --- |
| 0 | 0.396 / 0.372 / 0.374 | +0.36% | 0.444 → 0.463 |
| 1 | 0.378 / 0.357 / 0.351 | -1.66% | 0.394 → 0.382 |
| 2 | 0.419 / 0.394 / 0.390 | -1.05% | 0.417 → 0.424 |
| 3 | 0.435 / 0.406 / 0.406 | -0.02% | 0.410 → 0.423 |
| 4 | 0.718 / 0.670 / 0.677 | +1.01% | 0.710 → 0.706 |
| 5 | 0.699 / 0.651 / 0.650 | -0.24% | 0.678 → 0.680 |
| 6 | 0.857 / 0.816 / 0.811 | -0.60% | 0.865 → 0.853 |
| 7 | 0.900 / 0.852 / 0.850 | -0.20% | 0.912 → 0.891 |
| 8 | 0.917 / 0.862 / 0.865 | +0.31% | 0.868 → 0.892 |
| 9 | 0.897 / 0.846 / 0.840 | -0.79% | 0.848 → 0.855 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.21 | 6.959 | 13.276 | 3.550 | 8.375 |
| 0.4.25 | 6.575 | 12.465 | 3.605 | 8.094 |
| 0.5.0 | 6.583 | 12.421 | 3.620 | 8.000 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.5.0/docs/measurements/0.5.0/wide-extended-summary.json).

### Existing limit and recovery checks

Three additional fresh servers reject 257/512 known-term requests with both rankers: twelve expected errors, no partial successful results, error progress and successful later health requests. The existing 256-unique-term assessment limit remains; these are not successful-search latency samples. Large inputs remain subject to cooperative bounds, not an OS CPU/RAM quota.

## Actual MCP: two ordinary campaigns

Each ordinary campaign uses nine cases and five alternating old/new pairs: **90 servers each, 180 total**. Canonical knowledge, NumPy and broad synthetic input request progress; 1/4/16-entry inputs run with and without progress. Each makes ten queries with both rankers, two repeated passes, four concurrent searches, matching/relations, rejected arguments and health; snapshot inputs check exact UTF-8 readbacks.

Together **13,880 actual result/error replies validate against advertised schemas**, and **6,940 complete old/new structured pairs agree by RPC ID**, excluding only checked status version and version-qualified schema URNs. Discovery, progress/no-progress, bounds, ordering, input hashes and EOF shutdown pass. Search and readback time are separate. Child peak RSS includes runtime, buffers and source/index memory, unlike allocation profiles. Retain all slower rows.

### First complete campaign

| Case | Ranker | Initial median / p95 ms: old → new | Repeated median / p95 ms: old → new | Repeated median change |
| --- | --- | --- | --- | --- |
| canonical | bm25 | 2.555 / 4.046 → 2.462 / 3.837 | 2.035 / 3.427 → 2.021 / 3.495 | -0.69% |
| canonical | tfidf | 17.639 / 29.983 → 16.578 / 29.429 | 2.114 / 3.557 → 2.151 / 3.607 | +1.75% |
| numpy | bm25 | 0.860 / 1.193 → 0.852 / 1.203 | 0.738 / 1.054 → 0.737 / 1.059 | -0.06% |
| numpy | tfidf | 1.047 / 2.684 → 0.987 / 2.666 | 0.700 / 1.111 → 0.691 / 1.091 | -1.32% |
| wide | bm25 | 0.455 / 0.541 → 0.443 / 0.536 | 0.388 / 0.579 → 0.382 / 0.592 | -1.40% |
| wide | tfidf | 0.431 / 0.707 → 0.421 / 0.721 | 0.384 / 0.664 → 0.383 / 0.652 | -0.40% |
| tiny | bm25 | 0.124 / 0.224 → 0.128 / 0.229 | 0.113 / 0.127 → 0.114 / 0.131 | +1.03% |
| tiny | tfidf | 0.124 / 0.167 → 0.125 / 0.167 | 0.110 / 0.142 → 0.116 / 0.154 | +4.93% |
| tiny4 | bm25 | 0.186 / 0.301 → 0.179 / 0.306 | 0.164 / 0.191 → 0.163 / 0.202 | -0.20% |
| tiny4 | tfidf | 0.174 / 0.224 → 0.175 / 0.235 | 0.162 / 0.182 → 0.162 / 0.185 | +0.03% |
| tiny16 | bm25 | 0.199 / 0.330 → 0.200 / 0.324 | 0.182 / 0.203 → 0.180 / 0.199 | -1.09% |
| tiny16 | tfidf | 0.192 / 0.263 → 0.191 / 0.263 | 0.184 / 0.222 → 0.182 / 0.214 | -1.08% |
| tiny-no-progress | bm25 | 0.072 / 0.166 → 0.075 / 0.169 | 0.064 / 0.074 → 0.066 / 0.076 | +2.07% |
| tiny-no-progress | tfidf | 0.071 / 0.109 → 0.075 / 0.113 | 0.065 / 0.107 → 0.065 / 0.103 | +0.93% |
| tiny4-no-progress | bm25 | 0.125 / 0.238 → 0.129 / 0.242 | 0.115 / 0.148 → 0.117 / 0.149 | +1.39% |
| tiny4-no-progress | tfidf | 0.120 / 0.164 → 0.122 / 0.175 | 0.114 / 0.125 → 0.117 / 0.129 | +2.04% |
| tiny16-no-progress | bm25 | 0.142 / 0.274 → 0.150 / 0.270 | 0.135 / 0.172 → 0.135 / 0.169 | +0.31% |
| tiny16-no-progress | tfidf | 0.143 / 0.196 → 0.147 / 0.203 | 0.136 / 0.154 → 0.136 / 0.154 | +0.26% |

| Case | Startup median ms: old → new | First TF-IDF median ms: old → new | Peak RSS median MiB: old → new |
| --- | --- | --- | --- |
| canonical | 4684.143 → 4749.230 | 20.911 → 21.625 | 1724.047 → 1724.141 |
| numpy | 134.711 → 134.919 | 1.492 → 1.491 | 57.203 → 57.188 |
| wide | 3.771 → 3.593 | 0.469 → 0.465 | 7.656 → 7.594 |
| tiny | 3.109 → 3.204 | 0.172 → 0.169 | 5.797 → 5.859 |
| tiny4 | 3.001 → 3.324 | 0.233 → 0.239 | 5.906 → 5.938 |
| tiny16 | 3.237 → 3.028 | 0.265 → 0.263 | 6.094 → 6.109 |
| tiny-no-progress | 3.202 → 3.070 | 0.110 → 0.113 | 5.797 → 5.859 |
| tiny4-no-progress | 2.981 → 3.158 | 0.165 → 0.172 | 5.906 → 5.953 |
| tiny16-no-progress | 3.130 → 3.137 | 0.199 → 0.211 | 6.094 → 6.156 |

### Second preplanned complete campaign

| Case | Ranker | Initial median / p95 ms: old → new | Repeated median / p95 ms: old → new | Repeated median change |
| --- | --- | --- | --- | --- |
| canonical | bm25 | 2.438 / 3.978 → 2.497 / 3.898 | 2.035 / 3.455 → 2.071 / 3.432 | +1.76% |
| canonical | tfidf | 16.648 / 30.838 → 17.134 / 30.111 | 2.108 / 3.516 → 2.122 / 3.615 | +0.67% |
| numpy | bm25 | 0.869 / 1.235 → 0.878 / 1.216 | 0.738 / 1.066 → 0.752 / 1.058 | +1.95% |
| numpy | tfidf | 1.022 / 2.702 → 1.022 / 2.717 | 0.701 / 1.115 → 0.708 / 1.104 | +1.04% |
| wide | bm25 | 0.452 / 0.542 → 0.450 / 0.552 | 0.390 / 0.602 → 0.387 / 0.669 | -0.57% |
| wide | tfidf | 0.435 / 0.714 → 0.429 / 0.714 | 0.389 / 0.665 → 0.391 / 0.668 | +0.55% |
| tiny | bm25 | 0.126 / 0.221 → 0.127 / 0.230 | 0.113 / 0.128 → 0.112 / 0.121 | -0.20% |
| tiny | tfidf | 0.126 / 0.166 → 0.125 / 0.172 | 0.113 / 0.131 → 0.112 / 0.145 | -1.33% |
| tiny4 | bm25 | 0.185 / 0.301 → 0.189 / 0.300 | 0.163 / 0.199 → 0.162 / 0.197 | -0.22% |
| tiny4 | tfidf | 0.174 / 0.234 → 0.172 / 0.240 | 0.161 / 0.176 → 0.160 / 0.177 | -0.84% |
| tiny16 | bm25 | 0.198 / 0.326 → 0.199 / 0.327 | 0.180 / 0.201 → 0.182 / 0.206 | +0.97% |
| tiny16 | tfidf | 0.192 / 0.254 → 0.192 / 0.271 | 0.182 / 0.221 → 0.183 / 0.218 | +0.71% |
| tiny-no-progress | bm25 | 0.076 / 0.165 → 0.076 / 0.171 | 0.064 / 0.076 → 0.065 / 0.077 | +0.55% |
| tiny-no-progress | tfidf | 0.074 / 0.117 → 0.077 / 0.112 | 0.064 / 0.094 → 0.065 / 0.101 | +1.83% |
| tiny4-no-progress | bm25 | 0.130 / 0.241 → 0.133 / 0.242 | 0.117 / 0.150 → 0.116 / 0.141 | -1.00% |
| tiny4-no-progress | tfidf | 0.123 / 0.195 → 0.124 / 0.181 | 0.117 / 0.143 → 0.115 / 0.126 | -2.22% |
| tiny16-no-progress | bm25 | 0.148 / 0.276 → 0.152 / 0.269 | 0.134 / 0.183 → 0.135 / 0.170 | +0.84% |
| tiny16-no-progress | tfidf | 0.146 / 0.192 → 0.145 / 0.202 | 0.134 / 0.154 → 0.135 / 0.159 | +0.31% |

| Case | Startup median ms: old → new | First TF-IDF median ms: old → new | Peak RSS median MiB: old → new |
| --- | --- | --- | --- |
| canonical | 4676.124 → 4699.582 | 20.645 → 21.218 | 1724.094 → 1724.031 |
| numpy | 135.513 → 135.342 | 1.565 → 1.574 | 57.141 → 57.125 |
| wide | 3.741 → 3.581 | 0.464 → 0.451 | 7.625 → 7.672 |
| tiny | 3.032 → 3.188 | 0.168 → 0.176 | 5.828 → 5.875 |
| tiny4 | 3.090 → 3.116 | 0.237 → 0.240 | 5.922 → 5.953 |
| tiny16 | 3.131 → 3.174 | 0.257 → 0.274 | 6.172 → 6.078 |
| tiny-no-progress | 3.062 → 3.153 | 0.121 → 0.112 | 5.812 → 5.859 |
| tiny4-no-progress | 3.057 → 3.004 | 0.195 → 0.181 | 5.906 → 5.922 |
| tiny16-no-progress | 3.144 → 3.025 | 0.192 → 0.207 | 6.109 → 6.156 |

## Retained full small-Core regression controls

These separate campaigns compare freshly built exact registry 0.4.21 and 0.4.25 with the candidate, using the same programs. Seven rotating triples per campaign run both the original six cases and a 96-case extension. Warm-up, full original-JSON comparison and index setup occur before timers. Each campaign has **42 processes and 2,142,000 timed Core calls**. Both together have 84 processes, 4,284,000 calls and **2,688 complete original matrix-response comparisons**, with identical results. Signature checks also cover every original-six and timed matrix sample. This is repeated warm Core time, without MCP transport or DHAT instrumentation.

Keep the two campaigns separate. Original six-case calls build SearchOptions inside the measured loop; the extension reuses fixed options. Do not compare their absolute times or merge distributions. The 96 cases combine four entry counts, both rankers and twelve fixed query/filter/limit cases. Different-word, wide, punctuated and Unicode queries retain fallback analysis; short canonical repetitions retain the existing path. Unknown-only ASCII queries, scope rejection and zero limits are also retained; no case is removed for missing the optimized path. Historical absolute values with different programs/loads are not a reference substitute.

### First small Core campaign

#### Original six warm cases

| Fixture | 0.4.21 median ns | 0.4.25 median ns | 0.5.0 median ns | vs 0.4.25 | vs 0.4.21 |
| --- | --- | --- | --- | --- | --- |
| tiny16_bm25 | 894.250 | 905.708 | 908.958 | +0.36% | +1.64% |
| tiny16_tfidf | 968.625 | 979.667 | 977.333 | -0.24% | +0.90% |
| tiny4_bm25 | 288.875 | 288.959 | 283.625 | -1.85% | -1.82% |
| tiny4_tfidf | 304.167 | 306.500 | 304.917 | -0.52% | +0.25% |
| tiny_bm25 | 126.292 | 124.083 | 122.458 | -1.31% | -3.04% |
| tiny_tfidf | 126.667 | 127.791 | 125.375 | -1.89% | -1.02% |

Higher candidate medians versus 0.4.25: **1 of 6**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](https://github.com/LEVOGNE/airust/blob/v0.5.0/docs/measurements/0.5.0/small-reference-summary.json). No threshold-based rejection or removal of slow timing samples is used.

#### All 96 extended cases

| Fixture | 0.4.21 median ns | 0.4.25 median ns | 0.5.0 median ns | vs 0.4.25 | vs 0.4.21 |
| --- | --- | --- | --- | --- | --- |
| 16_Bm25_disjoint | 450.083 | 421.375 | 420.875 | -0.12% | -6.49% |
| 16_Bm25_limit_one | 569.875 | 576.625 | 569.875 | -1.17% | +0.00% |
| 16_Bm25_limit_zero | 2.625 | 3.625 | 3.666 | +1.13% | +39.66% |
| 16_Bm25_multi | 1569.875 | 1612.375 | 1651.125 | +2.40% | +5.18% |
| 16_Bm25_punctuation | 142.708 | 148.375 | 145.833 | -1.71% | +2.19% |
| 16_Bm25_repeated | 1178.833 | 1069.709 | 1079.375 | +0.90% | -8.44% |
| 16_Bm25_scope | 944.125 | 937.542 | 930.458 | -0.76% | -1.45% |
| 16_Bm25_single | 1067.125 | 1057.416 | 1056.500 | -0.09% | -1.00% |
| 16_Bm25_unicode | 1324.208 | 1350.041 | 1354.792 | +0.35% | +2.31% |
| 16_Bm25_unknown | 19.708 | 19.166 | 19.292 | +0.66% | -2.11% |
| 16_Bm25_unknown_added | 1156.875 | 1173.792 | 1167.167 | -0.56% | +0.89% |
| 16_Bm25_wide | 17233.958 | 16762.625 | 16582.459 | -1.07% | -3.78% |
| 16_Tfidf_disjoint | 457.500 | 430.417 | 425.875 | -1.06% | -6.91% |
| 16_Tfidf_limit_one | 694.250 | 671.083 | 663.292 | -1.16% | -4.46% |
| 16_Tfidf_limit_zero | 2.666 | 3.666 | 2.791 | -23.87% | +4.69% |
| 16_Tfidf_multi | 1741.791 | 1675.250 | 1635.417 | -2.38% | -6.11% |
| 16_Tfidf_punctuation | 143.417 | 149.167 | 145.083 | -2.74% | +1.16% |
| 16_Tfidf_repeated | 1302.791 | 1189.291 | 1191.667 | +0.20% | -8.53% |
| 16_Tfidf_scope | 1027.958 | 1017.375 | 996.209 | -2.08% | -3.09% |
| 16_Tfidf_single | 1187.291 | 1148.542 | 1151.334 | +0.24% | -3.03% |
| 16_Tfidf_unicode | 1418.250 | 1421.958 | 1409.875 | -0.85% | -0.59% |
| 16_Tfidf_unknown | 19.708 | 19.292 | 19.209 | -0.43% | -2.53% |
| 16_Tfidf_unknown_added | 1272.375 | 1283.125 | 1232.625 | -3.94% | -3.12% |
| 16_Tfidf_wide | 17302.292 | 17060.333 | 17417.042 | +2.09% | +0.66% |
| 1_Bm25_disjoint | 58.334 | 56.208 | 55.375 | -1.48% | -5.07% |
| 1_Bm25_limit_one | 124.125 | 124.166 | 122.583 | -1.27% | -1.24% |
| 1_Bm25_limit_zero | 2.666 | 2.791 | 2.791 | +0.00% | +4.69% |
| 1_Bm25_multi | 267.917 | 271.208 | 268.292 | -1.08% | +0.14% |
| 1_Bm25_punctuation | 144.625 | 144.292 | 144.916 | +0.43% | +0.20% |
| 1_Bm25_repeated | 255.000 | 158.875 | 157.625 | -0.79% | -38.19% |
| 1_Bm25_scope | 131.500 | 130.083 | 129.917 | -0.13% | -1.20% |
| 1_Bm25_single | 149.459 | 144.542 | 148.292 | +2.59% | -0.78% |
| 1_Bm25_unicode | 428.834 | 435.250 | 429.167 | -1.40% | +0.08% |
| 1_Bm25_unknown | 19.959 | 19.250 | 19.125 | -0.65% | -4.18% |
| 1_Bm25_unknown_added | 230.209 | 262.834 | 239.875 | -8.74% | +4.20% |
| 1_Bm25_wide | 4371.709 | 4361.167 | 4416.125 | +1.26% | +1.02% |
| 1_Tfidf_disjoint | 58.875 | 57.333 | 56.250 | -1.89% | -4.46% |
| 1_Tfidf_limit_one | 130.917 | 135.833 | 132.166 | -2.70% | +0.95% |
| 1_Tfidf_limit_zero | 2.709 | 2.750 | 2.792 | +1.53% | +3.06% |
| 1_Tfidf_multi | 254.292 | 266.333 | 263.458 | -1.08% | +3.60% |
| 1_Tfidf_punctuation | 143.250 | 145.125 | 146.250 | +0.78% | +2.09% |
| 1_Tfidf_repeated | 249.292 | 152.916 | 151.875 | -0.68% | -39.08% |
| 1_Tfidf_scope | 136.458 | 141.875 | 138.042 | -2.70% | +1.16% |
| 1_Tfidf_single | 132.709 | 137.375 | 133.875 | -2.55% | +0.88% |
| 1_Tfidf_unicode | 434.209 | 438.208 | 427.541 | -2.43% | -1.54% |
| 1_Tfidf_unknown | 19.667 | 19.292 | 19.333 | +0.21% | -1.70% |
| 1_Tfidf_unknown_added | 227.417 | 237.542 | 233.209 | -1.82% | +2.55% |
| 1_Tfidf_wide | 4485.541 | 4454.667 | 4542.542 | +1.97% | +1.27% |
| 33_Bm25_disjoint | 952.500 | 899.625 | 904.416 | +0.53% | -5.05% |
| 33_Bm25_limit_one | 1155.208 | 1145.958 | 1154.916 | +0.78% | -0.03% |
| 33_Bm25_limit_zero | 2.667 | 2.792 | 2.750 | -1.50% | +3.11% |
| 33_Bm25_multi | 2405.416 | 2467.709 | 2551.625 | +3.40% | +6.08% |
| 33_Bm25_punctuation | 144.958 | 146.583 | 148.666 | +1.42% | +2.56% |
| 33_Bm25_repeated | 1785.750 | 1698.459 | 1686.375 | -0.71% | -5.56% |
| 33_Bm25_scope | 1745.583 | 1707.916 | 1717.458 | +0.56% | -1.61% |
| 33_Bm25_single | 1628.625 | 1630.125 | 1599.000 | -1.91% | -1.82% |
| 33_Bm25_unicode | 1885.083 | 1872.416 | 1906.166 | +1.80% | +1.12% |
| 33_Bm25_unknown | 19.625 | 19.291 | 19.417 | +0.65% | -1.06% |
| 33_Bm25_unknown_added | 1779.375 | 1761.667 | 1781.625 | +1.13% | +0.13% |
| 33_Bm25_wide | 26681.333 | 26044.125 | 25812.958 | -0.89% | -3.25% |
| 33_Tfidf_disjoint | 1002.375 | 897.792 | 882.583 | -1.69% | -11.95% |
| 33_Tfidf_limit_one | 1365.041 | 1314.417 | 1324.083 | +0.74% | -3.00% |
| 33_Tfidf_limit_zero | 2.667 | 2.792 | 3.834 | +37.32% | +43.76% |
| 33_Tfidf_multi | 2680.375 | 2695.083 | 2630.750 | -2.39% | -1.85% |
| 33_Tfidf_punctuation | 143.417 | 145.417 | 147.375 | +1.35% | +2.76% |
| 33_Tfidf_repeated | 1987.916 | 1875.209 | 1861.625 | -0.72% | -6.35% |
| 33_Tfidf_scope | 1886.584 | 1798.667 | 1790.083 | -0.48% | -5.12% |
| 33_Tfidf_single | 1831.958 | 1855.292 | 1806.959 | -2.61% | -1.36% |
| 33_Tfidf_unicode | 2069.750 | 2056.042 | 2034.250 | -1.06% | -1.72% |
| 33_Tfidf_unknown | 19.792 | 19.334 | 19.375 | +0.21% | -2.11% |
| 33_Tfidf_unknown_added | 1970.542 | 1996.875 | 1949.541 | -2.37% | -1.07% |
| 33_Tfidf_wide | 26642.292 | 26326.208 | 26402.625 | +0.29% | -0.90% |
| 4_Bm25_disjoint | 110.833 | 110.584 | 107.666 | -2.64% | -2.86% |
| 4_Bm25_limit_one | 185.250 | 192.625 | 192.375 | -0.13% | +3.85% |
| 4_Bm25_limit_zero | 2.667 | 2.792 | 2.792 | +0.00% | +4.69% |
| 4_Bm25_multi | 533.958 | 565.792 | 576.167 | +1.83% | +7.90% |
| 4_Bm25_punctuation | 141.750 | 146.708 | 147.666 | +0.65% | +4.17% |
| 4_Bm25_repeated | 419.792 | 340.458 | 350.042 | +2.82% | -16.62% |
| 4_Bm25_scope | 231.666 | 239.292 | 241.750 | +1.03% | +4.35% |
| 4_Bm25_single | 313.333 | 340.875 | 335.291 | -1.64% | +7.01% |
| 4_Bm25_unicode | 589.334 | 611.041 | 627.875 | +2.75% | +6.54% |
| 4_Bm25_unknown | 19.584 | 19.583 | 19.250 | -1.70% | -1.71% |
| 4_Bm25_unknown_added | 402.667 | 423.875 | 437.709 | +3.26% | +8.70% |
| 4_Bm25_wide | 7330.250 | 7461.000 | 7339.375 | -1.63% | +0.12% |
| 4_Tfidf_disjoint | 112.292 | 109.792 | 107.833 | -1.78% | -3.97% |
| 4_Tfidf_limit_one | 217.125 | 217.792 | 209.917 | -3.62% | -3.32% |
| 4_Tfidf_limit_zero | 2.667 | 2.792 | 2.834 | +1.50% | +6.26% |
| 4_Tfidf_multi | 592.875 | 588.542 | 563.500 | -4.25% | -4.95% |
| 4_Tfidf_punctuation | 146.875 | 148.375 | 146.000 | -1.60% | -0.60% |
| 4_Tfidf_repeated | 474.500 | 368.167 | 360.125 | -2.18% | -24.10% |
| 4_Tfidf_scope | 250.750 | 248.792 | 238.834 | -4.00% | -4.75% |
| 4_Tfidf_single | 355.667 | 356.500 | 339.208 | -4.85% | -4.63% |
| 4_Tfidf_unicode | 652.125 | 660.541 | 628.750 | -4.81% | -3.58% |
| 4_Tfidf_unknown | 19.625 | 19.333 | 19.250 | -0.43% | -1.91% |
| 4_Tfidf_unknown_added | 456.334 | 450.959 | 433.917 | -3.78% | -4.91% |
| 4_Tfidf_wide | 7447.041 | 7265.166 | 7230.541 | -0.48% | -2.91% |

Higher candidate medians versus 0.4.25: **35 of 96**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](https://github.com/LEVOGNE/airust/blob/v0.5.0/docs/measurements/0.5.0/small-reference-summary.json). No threshold-based rejection or removal of slow timing samples is used.

### Second preplanned small Core campaign

#### Original six warm cases

| Fixture | 0.4.21 median ns | 0.4.25 median ns | 0.5.0 median ns | vs 0.4.25 | vs 0.4.21 |
| --- | --- | --- | --- | --- | --- |
| tiny16_bm25 | 900.167 | 906.958 | 913.792 | +0.75% | +1.51% |
| tiny16_tfidf | 979.417 | 975.417 | 984.834 | +0.97% | +0.55% |
| tiny4_bm25 | 288.333 | 286.417 | 284.625 | -0.63% | -1.29% |
| tiny4_tfidf | 306.333 | 307.709 | 304.542 | -1.03% | -0.58% |
| tiny_bm25 | 123.750 | 127.000 | 123.208 | -2.99% | -0.44% |
| tiny_tfidf | 123.375 | 128.250 | 125.250 | -2.34% | +1.52% |

Higher candidate medians versus 0.4.25: **2 of 6**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](https://github.com/LEVOGNE/airust/blob/v0.5.0/docs/measurements/0.5.0/small-confirmation-summary.json). No threshold-based rejection or removal of slow timing samples is used.

#### All 96 extended cases

| Fixture | 0.4.21 median ns | 0.4.25 median ns | 0.5.0 median ns | vs 0.4.25 | vs 0.4.21 |
| --- | --- | --- | --- | --- | --- |
| 16_Bm25_disjoint | 437.209 | 421.625 | 415.167 | -1.53% | -5.04% |
| 16_Bm25_limit_one | 575.292 | 564.000 | 562.584 | -0.25% | -2.21% |
| 16_Bm25_limit_zero | 3.792 | 3.625 | 3.625 | +0.00% | -4.40% |
| 16_Bm25_multi | 1578.833 | 1614.250 | 1596.042 | -1.13% | +1.09% |
| 16_Bm25_punctuation | 143.333 | 146.417 | 146.875 | +0.31% | +2.47% |
| 16_Bm25_repeated | 1175.959 | 1088.125 | 1078.416 | -0.89% | -8.29% |
| 16_Bm25_scope | 942.333 | 939.292 | 960.167 | +2.22% | +1.89% |
| 16_Bm25_single | 1066.208 | 1091.000 | 1067.583 | -2.15% | +0.13% |
| 16_Bm25_unicode | 1330.000 | 1331.000 | 1327.917 | -0.23% | -0.16% |
| 16_Bm25_unknown | 19.708 | 19.333 | 19.250 | -0.43% | -2.32% |
| 16_Bm25_unknown_added | 1155.875 | 1173.791 | 1164.416 | -0.80% | +0.74% |
| 16_Bm25_wide | 17063.459 | 17061.458 | 16772.834 | -1.69% | -1.70% |
| 16_Tfidf_disjoint | 461.625 | 443.625 | 434.625 | -2.03% | -5.85% |
| 16_Tfidf_limit_one | 687.916 | 680.208 | 654.458 | -3.79% | -4.86% |
| 16_Tfidf_limit_zero | 2.666 | 2.792 | 2.791 | -0.04% | +4.69% |
| 16_Tfidf_multi | 1675.167 | 1682.875 | 1652.000 | -1.83% | -1.38% |
| 16_Tfidf_punctuation | 145.417 | 145.625 | 145.125 | -0.34% | -0.20% |
| 16_Tfidf_repeated | 1275.042 | 1187.625 | 1170.542 | -1.44% | -8.20% |
| 16_Tfidf_scope | 1017.167 | 1026.834 | 1011.583 | -1.49% | -0.55% |
| 16_Tfidf_single | 1166.541 | 1160.625 | 1139.750 | -1.80% | -2.30% |
| 16_Tfidf_unicode | 1408.084 | 1404.541 | 1402.542 | -0.14% | -0.39% |
| 16_Tfidf_unknown | 19.667 | 19.500 | 19.625 | +0.64% | -0.21% |
| 16_Tfidf_unknown_added | 1253.542 | 1254.708 | 1250.917 | -0.30% | -0.21% |
| 16_Tfidf_wide | 17431.375 | 17343.084 | 16901.125 | -2.55% | -3.04% |
| 1_Bm25_disjoint | 55.542 | 56.708 | 55.958 | -1.32% | +0.75% |
| 1_Bm25_limit_one | 122.209 | 124.166 | 123.667 | -0.40% | +1.19% |
| 1_Bm25_limit_zero | 2.667 | 2.834 | 2.792 | -1.48% | +4.69% |
| 1_Bm25_multi | 267.875 | 274.208 | 264.375 | -3.59% | -1.31% |
| 1_Bm25_punctuation | 141.625 | 144.333 | 143.333 | -0.69% | +1.21% |
| 1_Bm25_repeated | 273.334 | 161.000 | 156.916 | -2.54% | -42.59% |
| 1_Bm25_scope | 129.792 | 130.416 | 130.875 | +0.35% | +0.83% |
| 1_Bm25_single | 142.167 | 142.167 | 148.042 | +4.13% | +4.13% |
| 1_Bm25_unicode | 432.083 | 434.458 | 426.459 | -1.84% | -1.30% |
| 1_Bm25_unknown | 19.666 | 19.250 | 19.333 | +0.43% | -1.69% |
| 1_Bm25_unknown_added | 225.917 | 263.083 | 246.750 | -6.21% | +9.22% |
| 1_Bm25_wide | 4325.708 | 4469.125 | 4403.917 | -1.46% | +1.81% |
| 1_Tfidf_disjoint | 56.750 | 58.042 | 56.042 | -3.45% | -1.25% |
| 1_Tfidf_limit_one | 130.291 | 135.125 | 132.958 | -1.60% | +2.05% |
| 1_Tfidf_limit_zero | 2.667 | 2.792 | 2.750 | -1.50% | +3.11% |
| 1_Tfidf_multi | 260.291 | 269.625 | 269.917 | +0.11% | +3.70% |
| 1_Tfidf_punctuation | 143.833 | 143.459 | 145.417 | +1.36% | +1.10% |
| 1_Tfidf_repeated | 248.958 | 153.375 | 151.167 | -1.44% | -39.28% |
| 1_Tfidf_scope | 136.375 | 143.125 | 138.333 | -3.35% | +1.44% |
| 1_Tfidf_single | 130.208 | 135.833 | 133.459 | -1.75% | +2.50% |
| 1_Tfidf_unicode | 437.125 | 434.958 | 426.875 | -1.86% | -2.34% |
| 1_Tfidf_unknown | 19.541 | 19.167 | 19.209 | +0.22% | -1.70% |
| 1_Tfidf_unknown_added | 229.291 | 238.083 | 229.792 | -3.48% | +0.22% |
| 1_Tfidf_wide | 4431.209 | 4519.417 | 4339.834 | -3.97% | -2.06% |
| 33_Bm25_disjoint | 968.458 | 913.500 | 884.542 | -3.17% | -8.66% |
| 33_Bm25_limit_one | 1195.333 | 1159.541 | 1134.375 | -2.17% | -5.10% |
| 33_Bm25_limit_zero | 2.666 | 2.792 | 2.791 | -0.04% | +4.69% |
| 33_Bm25_multi | 2447.834 | 2562.584 | 2510.250 | -2.04% | +2.55% |
| 33_Bm25_punctuation | 143.000 | 147.292 | 145.250 | -1.39% | +1.57% |
| 33_Bm25_repeated | 1796.000 | 1774.291 | 1643.625 | -7.36% | -8.48% |
| 33_Bm25_scope | 1771.042 | 1747.542 | 1680.375 | -3.84% | -5.12% |
| 33_Bm25_single | 1679.625 | 1664.083 | 1607.667 | -3.39% | -4.28% |
| 33_Bm25_unicode | 1903.084 | 1969.958 | 1869.084 | -5.12% | -1.79% |
| 33_Bm25_unknown | 19.584 | 19.250 | 19.375 | +0.65% | -1.07% |
| 33_Bm25_unknown_added | 1784.333 | 1827.292 | 1753.875 | -4.02% | -1.71% |
| 33_Bm25_wide | 26260.167 | 26385.083 | 25712.250 | -2.55% | -2.09% |
| 33_Tfidf_disjoint | 971.458 | 896.375 | 887.917 | -0.94% | -8.60% |
| 33_Tfidf_limit_one | 1358.458 | 1284.542 | 1279.583 | -0.39% | -5.81% |
| 33_Tfidf_limit_zero | 2.667 | 2.792 | 3.833 | +37.29% | +43.72% |
| 33_Tfidf_multi | 2581.292 | 2638.584 | 2607.458 | -1.18% | +1.01% |
| 33_Tfidf_punctuation | 144.084 | 145.542 | 146.333 | +0.54% | +1.56% |
| 33_Tfidf_repeated | 1948.667 | 1888.375 | 1828.541 | -3.17% | -6.16% |
| 33_Tfidf_scope | 1866.875 | 1794.250 | 1815.791 | +1.20% | -2.74% |
| 33_Tfidf_single | 1840.208 | 1887.292 | 1767.333 | -6.36% | -3.96% |
| 33_Tfidf_unicode | 2024.000 | 2035.959 | 2024.833 | -0.55% | +0.04% |
| 33_Tfidf_unknown | 19.542 | 19.375 | 19.291 | -0.43% | -1.28% |
| 33_Tfidf_unknown_added | 1919.292 | 1962.541 | 1919.125 | -2.21% | -0.01% |
| 33_Tfidf_wide | 26750.500 | 26406.875 | 26257.541 | -0.57% | -1.84% |
| 4_Bm25_disjoint | 109.625 | 109.459 | 106.208 | -2.97% | -3.12% |
| 4_Bm25_limit_one | 190.667 | 186.042 | 182.709 | -1.79% | -4.17% |
| 4_Bm25_limit_zero | 2.625 | 2.708 | 2.792 | +3.10% | +6.36% |
| 4_Bm25_multi | 548.583 | 568.500 | 555.750 | -2.24% | +1.31% |
| 4_Bm25_punctuation | 142.458 | 144.708 | 144.791 | +0.06% | +1.64% |
| 4_Bm25_repeated | 429.875 | 338.083 | 331.625 | -1.91% | -22.86% |
| 4_Bm25_scope | 242.375 | 235.042 | 231.209 | -1.63% | -4.61% |
| 4_Bm25_single | 311.542 | 325.459 | 313.542 | -3.66% | +0.64% |
| 4_Bm25_unicode | 603.500 | 594.375 | 605.916 | +1.94% | +0.40% |
| 4_Bm25_unknown | 19.709 | 19.167 | 19.333 | +0.87% | -1.91% |
| 4_Bm25_unknown_added | 410.958 | 414.125 | 419.542 | +1.31% | +2.09% |
| 4_Bm25_wide | 7424.125 | 7317.583 | 7299.084 | -0.25% | -1.68% |
| 4_Tfidf_disjoint | 108.250 | 105.834 | 105.083 | -0.71% | -2.93% |
| 4_Tfidf_limit_one | 210.250 | 213.833 | 209.250 | -2.14% | -0.48% |
| 4_Tfidf_limit_zero | 2.666 | 2.792 | 2.792 | +0.00% | +4.73% |
| 4_Tfidf_multi | 580.041 | 581.583 | 570.458 | -1.91% | -1.65% |
| 4_Tfidf_punctuation | 144.583 | 147.666 | 147.042 | -0.42% | +1.70% |
| 4_Tfidf_repeated | 451.833 | 355.417 | 358.666 | +0.91% | -20.62% |
| 4_Tfidf_scope | 241.416 | 238.958 | 237.417 | -0.64% | -1.66% |
| 4_Tfidf_single | 342.417 | 343.292 | 344.334 | +0.30% | +0.56% |
| 4_Tfidf_unicode | 630.125 | 638.417 | 625.667 | -2.00% | -0.71% |
| 4_Tfidf_unknown | 19.708 | 19.292 | 19.333 | +0.21% | -1.90% |
| 4_Tfidf_unknown_added | 436.750 | 446.041 | 436.166 | -2.21% | -0.13% |
| 4_Tfidf_wide | 7282.375 | 7352.000 | 7270.959 | -1.10% | -0.16% |

Higher candidate medians versus 0.4.25: **21 of 96**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](https://github.com/LEVOGNE/airust/blob/v0.5.0/docs/measurements/0.5.0/small-confirmation-summary.json). No threshold-based rejection or removal of slow timing samples is used.

## Direct canonical search with explicit control modes

Fourteen processes (seven alternating pairs) retain one warmed Engine each. Ten rounds rotate plain, deadline and progress within each query/ranker pair: **8,400 timed Retriever calls**, excluding loading/readiness, stdio and profiling. Each progress request includes an observer and the same deadline/cancellation setup. Every signature agrees; **140 complete original response JSON pairs match**, and control modes also match plain output. Full readiness source text stays private. These are direct retrieval boundaries, not MCP round trips or capacity evidence.

| Control | Ranker | 0.4.25 median / p95 ms | 0.5.0 median / p95 ms | Median change |
| --- | --- | --- | --- | --- |
| plain | Bm25 | 1.239 / 2.457 | 1.232 / 2.451 | -0.54% |
| plain | Tfidf | 1.298 / 2.346 | 1.290 / 2.363 | -0.66% |
| deadline | Bm25 | 1.519 / 3.031 | 1.521 / 3.036 | +0.15% |
| deadline | Tfidf | 1.640 / 3.012 | 1.665 / 3.050 | +1.56% |
| progress | Bm25 | 1.513 / 3.004 | 1.514 / 3.010 | +0.06% |
| progress | Tfidf | 1.645 / 2.993 | 1.656 / 3.068 | +0.68% |

| Query | Candidates | Progress BM25 ms: old → new | Progress TF-IDF ms: old → new |
| --- | --- | --- | --- |
| 0 | 12365 | 1.505 → 1.542 | 1.651 → 1.669 |
| 1 | 13864 | 1.483 → 1.472 | 1.662 → 1.671 |
| 2 | 24299 | 2.669 → 2.613 | 2.896 → 2.904 |
| 3 | 596 | 0.301 → 0.297 | 0.293 → 0.292 |
| 4 | 15403 | 1.600 → 1.568 | 1.749 → 1.735 |
| 5 | 24140 | 2.550 → 2.521 | 2.768 → 2.779 |
| 6 | 3358 | 0.588 → 0.596 | 0.625 → 0.645 |
| 7 | 29600 | 2.492 → 2.507 | 2.814 → 2.893 |
| 8 | 11504 | 1.327 → 1.340 | 1.465 → 1.478 |
| 9 | 0 | 0.001 → 0.001 | 0.001 → 0.001 |

## Separate canonical three-version reference

Five rotating triples compare fresh exact 0.4.21/0.4.25/candidate servers on canonical knowledge with requested progress. Fifteen servers validate **1,020 actual schema replies**; **680 complete structured comparisons** against the candidate match by RPC ID except checked status version and version-qualified schema URNs. Schemas, source ranges and inputs agree. This control is separate from the two ordinary campaigns. It checks whether the previous large-query benefit remains visible against the older release; it does not establish the cause of any historical regression.

| Version | Ranker | Initial median / p95 ms | Repeated median / p95 ms | Peak child RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.21 | bm25 | 2.683 / 4.079 | 2.082 / 3.567 | 1724.141 |
| 0.4.21 | tfidf | 16.755 / 29.348 | 2.145 / 3.582 | 1724.141 |
| 0.4.25 | bm25 | 2.488 / 3.952 | 2.047 / 3.414 | 1724.016 |
| 0.4.25 | tfidf | 17.037 / 29.326 | 2.126 / 3.534 | 1724.016 |
| 0.5.0 | bm25 | 2.493 / 3.982 | 2.031 / 3.551 | 1724.062 |
| 0.5.0 | tfidf | 16.929 / 30.265 | 2.095 / 3.528 | 1724.062 |

| Query | 0.4.21 repeated TF-IDF ms | 0.4.25 repeated TF-IDF ms | 0.5.0 repeated TF-IDF ms |
| --- | --- | --- | --- |
| 0 | 2.145 | 2.125 | 2.090 |
| 1 | 2.259 | 2.225 | 2.247 |
| 2 | 3.528 | 3.473 | 3.500 |
| 3 | 0.667 | 0.643 | 0.643 |
| 4 | 2.174 | 2.212 | 2.158 |
| 5 | 3.372 | 3.418 | 3.351 |
| 6 | 1.072 | 1.049 | 1.019 |
| 7 | 3.516 | 3.436 | 3.459 |
| 8 | 1.945 | 1.870 | 1.915 |
| 9 | 0.106 | 0.103 | 0.104 |

## Canonical first-pass heap traffic

Two instrumented processes index all 167,942 records before profiling the same ten initial TF-IDF queries, then drop results before net deltas. The existing direct candidate collection remains; the previous large-query posting cursors and norm memo/cache still exist. Heap traffic, net persistent hints, stack state and whole-process RSS are different costs. Instrumented durations are excluded from latency comparisons.

| Metric | 0.4.25 | 0.5.0 |
| --- | --- | --- |
| Retained hint bytes | 2228312 | 2228312 |
| Retained hint blocks | 2 | 2 |
| Total query allocated bytes | 7212357 | 7212357 |
| Total query allocation blocks | 401 | 401 |

## Development trials and host-load limits

The full three-triple development MCP pilot is retained separately from all final campaigns. Its source snapshots include the MCP implementation and Core; all complete compared structured results agree. Initial/repeated data stay separate and no final sample is replaced by a pilot or a faster rerun. Earlier releases retain their own rejected trials. These are development observations, not fresh production or semantic holdouts.

Two-second monitoring retains **722 snapshots**, including **0 with other rustc processes**, maximum 0. All managed compilers finish before timing. Other workloads are not stopped, and snapshots do not prove continuous idleness or identify a timing cause. CPU counts include the active benchmark workers.

| Stage start | Other rustc count | Other rustc CPU total | Processes over 50% CPU |
| --- | --- | --- | --- |
| small-mcp-standard | 0 | 0.000 | 1 |
| small-reference | 0 | 0.000 | 2 |
| large-standard | 0 | 0.000 | 1 |
| wide-standard | 0 | 0.000 | 1 |
| retrieval | 0 | 0.000 | 1 |
| query | 0 | 0.000 | 1 |
| build | 0 | 0.000 | 1 |
| controlled | 0 | 0.000 | 1 |
| mcp | 0 | 0.000 | 1 |
| corpus | 0 | 0.000 | 1 |
| mcp-confirmation | 0 | 0.000 | 1 |
| reference | 0 | 0.000 | 1 |
| wide-confirmation | 0 | 0.000 | 1 |
| wide-extended | 0 | 0.000 | 1 |
| large-confirmation | 0 | 0.000 | 1 |
| small-confirmation | 0 | 0.000 | 1 |
| small-mcp-confirmation | 0 | 0.000 | 1 |
| limits | 0 | 0.000 | 1 |
| startup-standard | 0 | 0.000 | 1 |
| startup-confirmation | 0 | 0.000 | 1 |
| startup-heap | 0 | 0.000 | 1 |
| stability-stress | 0 | 0.000 | 1 |

All observed slower cases remain in their own distributions; no sample is replaced because of load. One machine, randomized hashing, exposed questions, tuned fixtures and finite trials limit generalization.

## Retained Core controls

These controls retain positive and negative timing changes. Single known-term requests retain the direct candidate path and exactly two retain ordered merging. Wider requests keep the heap/set strategies; large frequency cursors remain available afterward. Large sorted candidate sets can use the bounded document cursor after 128 candidates; earlier shared explanation intersection remains. Read every row independently, including slower cases. Seven alternating pairs give fourteen plain processes per family. Norms are initialized for warm timing; two separate DHAT processes measure allocations. Instrumented durations are excluded from latency comparisons. Every result signature agrees and temporary-call retained deltas are zero.

### Small warm Core: 84,000 calls on 1/4/16 entries

| Fixture | Median change | Blocks/call: old → new | Allocated bytes/call: old → new |
| --- | --- | --- | --- |
| tiny16_bm25 | -1.12% | 26.0 → 26.0 | 2156.0 → 2156.0 |
| tiny16_tfidf | -2.34% | 26.0 → 26.0 | 2156.0 → 2156.0 |
| tiny4_bm25 | +0.49% | 12.0 → 12.0 | 824.0 → 824.0 |
| tiny4_tfidf | -0.71% | 12.0 → 12.0 | 824.0 → 824.0 |
| tiny_bm25 | +1.18% | 6.0 → 6.0 | 398.0 → 398.0 |
| tiny_tfidf | -0.87% | 6.0 → 6.0 | 398.0 → 398.0 |

### All 25 warm controls: 282,800 calls

| Fixture | Median change | Blocks/call: old → new | Allocated bytes/call: old → new |
| --- | --- | --- | --- |
| coverage_repeated | +0.25% | 1047.0 → 1047.0 | 92942.0 → 92942.0 |
| coverage_small | +0.03% | 784.0 → 784.0 | 78046.0 → 78046.0 |
| excerpt_ascii | +0.70% | 762.0 → 762.0 | 252438.0 → 252438.0 |
| excerpt_mixed | +0.69% | 569.0 → 569.0 | 417113.0 → 417113.0 |
| excerpt_unicode | -0.19% | 197.0 → 197.0 | 72254.0 → 72254.0 |
| long_repeated | +4.01% | 4.0 → 4.0 | 18798.0 → 18798.0 |
| mixed_unicode | +1.81% | 7.0 → 7.0 | 3155.0 → 3155.0 |
| punctuation | +2.25% | 1.0 → 1.0 | 768.0 → 768.0 |
| relevant_retrieval | +0.41% | 812.0 → 812.0 | 93562.0 → 93562.0 |
| repeated_search | +0.92% | 31.0 → 31.0 | 18156.0 → 18156.0 |
| repeated_short | +2.74% | 4.0 → 4.0 | 1656.0 → 1656.0 |
| sparse_bm25_128 | -1.92% | 359.0 → 359.0 | 29074.0 → 29074.0 |
| sparse_bm25_33 | -1.76% | 241.0 → 241.0 | 16309.0 → 16309.0 |
| sparse_bm25_512 | -3.92% | 1337.0 → 1337.0 | 119124.0 → 119124.0 |
| sparse_tfidf_128 | +2.04% | 474.0 → 474.0 | 35140.0 → 35140.0 |
| sparse_tfidf_33 | +0.86% | 241.0 → 241.0 | 16309.0 → 16309.0 |
| sparse_tfidf_512 | -0.38% | 1337.0 → 1337.0 | 119124.0 → 119124.0 |
| tfidf_union_32 | -3.15% | 244.0 → 244.0 | 19834.0 → 19834.0 |
| tfidf_union_33 | -0.51% | 296.0 → 296.0 | 27854.0 → 27854.0 |
| tfidf_warm | -3.30% | 71.0 → 71.0 | 19335.0 → 19335.0 |
| union_32 | -0.72% | 244.0 → 244.0 | 19834.0 → 19834.0 |
| union_33 | -1.37% | 296.0 → 296.0 | 27854.0 → 27854.0 |
| unique_long | +1.46% | 299.0 → 299.0 | 32672.0 → 32672.0 |
| unique_search | +0.57% | 72.0 → 72.0 | 19350.0 → 19350.0 |
| unique_short | +2.55% | 296.0 → 296.0 | 18548.0 → 18548.0 |

### Cold norm and heap controls

Six fixtures make twenty fresh-index-clone calls each across fourteen processes: 1,680 calls. Clone/setup is outside timing. Process/instruction/filesystem caches remain; cold means norm hints, not disks. Separate heap runs drop results before net retained measurements. The existing inline/optional heap word memo and persistent norm storage remain. Large small-term requests may add up to four temporary posting cursors; canonical allocation traffic below records that cost. Heap profiles exclude stack state/control fields and temporary frames.

| Fixture | Old median / p95 ns | New median / p95 ns |
| --- | --- | --- |
| index | 109333.500 / 125472.600 | 108937.500 / 122933.950 |
| wide | 33770.500 / 49177.400 | 31542.000 / 39395.850 |
| sparse | 422249.500 / 483629.500 | 419895.500 / 469298.550 |
| tiny | 458.500 / 1086.050 | 417.000 / 1048.250 |
| tiny4 | 750.000 / 1083.050 | 750.000 / 1044.100 |
| tiny16 | 2416.000 / 3754.150 | 2333.000 / 3460.100 |

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
| new_group_query_ns | 121541.500 / 186082.700 | 173229.000 / 246020.850 |
| post_readers_first_ns | 148333.000 / 184366.800 | 147709.000 / 166383.600 |
| post_readers_second_ns | 81666.000 / 87058.200 | 82791.000 / 83766.600 |

The private document corpus retains 72 Markdown files, 2,755,468 bytes and 1,910 chunks. All 342 authored questions (276 positive, 66 unsupported) are exposed regressions, not fresh holdouts or logged production queries. Literal BM25, prepared BM25 and coverage retain six passages, 100 candidates, 2,000 bytes/pass and 12,000 total bytes. All **1,026 complete original response JSON strings match byte-for-byte**, including scores, ordering, evidence, source text and positions. One warm-up and five repeats/question/configuration/version give **10,260 timed round trips**. These three configurations are BM25 controls; independent score oracles and actual protocol pairs cover TF-IDF. No new relevance judgment or model reply is produced.

All document timing aggregates and per-question results remain in the records. Construction uses forty plain processes on canonical data and three synthetic distributions, plus six separate profiles. Parsing/generation is outside construction timing; stats/signatures/live derived heap agree. The execution/shared-index worker checks all 167,942 entries through 128 changes/restoration, rebuilds, JSON round trips, retained clones, parallel readers and cancellation/deadline rejection. Portable exports and source hashes agree.

### Document round trips

| Case | Old median / p95 ms | New median / p95 ms |
| --- | --- | --- |
| baseline | 0.701 / 1.111 | 0.695 / 1.089 |
| prepared | 0.814 / 1.232 | 0.809 / 1.206 |
| evidence | 0.788 / 1.219 | 0.786 / 1.200 |

### Construction only

| Case | Old median / p95 ms | New median / p95 ms |
| --- | --- | --- |
| canonical | 4847.979 / 4920.107 | 4853.515 / 4878.192 |
| ascii | 14.373 / 14.564 | 14.277 / 14.685 |
| mixed | 34.149 / 34.732 | 33.958 / 34.982 |
| long_tokens | 1.517 / 1.590 | 1.542 / 1.578 |

## Limits and preservation

Measurements use one Apple M5 Max (power source observed at campaign start only; see [benchmark metadata errata](BENCHMARK-ERRATA.md)) with 64 GB, Rust 1.98.0 and Python 3.15.2. Stage-start CPU/compiler snapshots remain. Own compilers finish before timing; this does not establish continuously idle hardware. Randomized hashing, scheduling, one machine/corpus, exposed questions and finite process counts limit generalization. The separate development pilot remains; assess final per-case medians and tails independently. Host-load sampling records observations rather than continuous idleness. No universal latency, total-memory, semantic-quality, token-saving or security-audit claim is made.

Canonical knowledge remains 167,942 entries from 35 PDFs and 49,086 physical pages, SHA-256 `13168d87872a137355fd8499c90a6084f52243a2ec7456f29b5fcb2a1c7423c3`. The PDF parser is unchanged; this is not a fresh large-PDF extraction campaign. Lexical overlap and exact quotes do not prove factual support or entailment. No model, daemon or DenkMaps integration is added.

Preserve PDFs, knowledge, snapshots, all rejected/selected pilot and final measurements, slower results, raw transcripts, profiles, exact small executables and locks. Raw measurement JSON stays in the tagged GitHub repository and local archives, outside the registry source package; API guides, Markdown reports, method source, examples and required fixtures remain in that package. After verified publication archive package/proof, then delete only the named temporary Cargo/Miri targets. Default features remain empty.
