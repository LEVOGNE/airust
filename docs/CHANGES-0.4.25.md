# AIRust 0.4.25: release temporary startup JSON before indexing

Knowledge-file startup now drops its owned raw JSON string after parsing owned KnowledgeEntry values, before constructing the index and MCP schemas. Original entry IDs, text, metadata and source files remain. This is a single explicit lifetime change in the knowledge loader, not a streaming parser or persistent-index format. The bounded file read, UTF-8 and JSON checks, entry limits, error handling and all public APIs remain. Core source is byte-for-byte unchanged from 0.4.24. Default features stay empty; no dependency, model or DenkMaps integration is added.

The first startup-only controls ran before final validation and remain separate from final results. Three rotating triples cover the preserved 167,942-entry corpus and a 16-entry fixture. The ordinary development MCP pilot also retains full compared responses. No development observation replaces a final sample.

## Startup memory and time: separate final measurements

Each of two preplanned native campaigns runs seven rotating exact 0.4.23/0.4.24/0.4.25 triples for each of two cases: **42 fresh MCP server children per campaign, 84 total**. Every child initializes, reports the expected version/count/read-only state with no occupied workers, receives EOF and exits successfully. No search or norm warming occurs. Filesystem/process caches are warm. Peak RSS comes from wait4 for that exact child, not cumulative RUSAGE_CHILDREN polluted by other subprocesses. Post-ready RSS is a separate ps observation. All samples and input hashes remain.

### Standard startup-only campaign

| Case / version | Startup median / p95 ms | Exact child peak median MiB | Post-ready RSS median MiB | Peak vs 0.4.24 |
| --- | --- | --- | --- | --- |
| canonical / 0.4.23 | 4846.542 / 4857.313 | 1720.031 | 1720.031 | +0.01% |
| canonical / 0.4.24 | 4832.095 / 4865.144 | 1719.859 | 1719.844 | +0.00% |
| canonical / 0.4.25 | 4814.231 / 4841.716 | 1719.797 | 1719.797 | -0.00% |
| tiny16 / 0.4.23 | 3.268 / 3.540 | 5.172 | 5.094 | +0.00% |
| tiny16 / 0.4.24 | 3.355 / 3.625 | 5.172 | 5.109 | +0.00% |
| tiny16 / 0.4.25 | 3.377 / 3.761 | 5.125 | 5.062 | -0.91% |

CPU, individual startup/peak/live samples and all distributions remain in [the full startup record](https://github.com/LEVOGNE/airust/blob/v0.4.25/docs/measurements/0.4.25/startup-standard-reports.json). No smaller or slower case is removed.

### Confirmation startup-only campaign

| Case / version | Startup median / p95 ms | Exact child peak median MiB | Post-ready RSS median MiB | Peak vs 0.4.24 |
| --- | --- | --- | --- | --- |
| canonical / 0.4.23 | 4847.877 / 4862.883 | 1720.062 | 1720.062 | +0.01% |
| canonical / 0.4.24 | 4852.980 / 4947.160 | 1719.953 | 1719.953 | +0.00% |
| canonical / 0.4.25 | 4818.021 / 4861.535 | 1719.719 | 1719.719 | -0.01% |
| tiny16 / 0.4.23 | 3.408 / 3.759 | 5.156 | 5.094 | +0.00% |
| tiny16 / 0.4.24 | 3.569 / 3.968 | 5.156 | 5.094 | +0.00% |
| tiny16 / 0.4.25 | 3.246 / 3.324 | 5.125 | 5.062 | -0.61% |

CPU, individual startup/peak/live samples and all distributions remain in [the full startup record](https://github.com/LEVOGNE/airust/blob/v0.4.25/docs/measurements/0.4.25/startup-confirmation-reports.json). No smaller or slower case is removed.

## Separate startup allocation profiles

Six DHAT-instrumented processes load both cases across all three versions. Profiles include file reading, owned parsing, index and schema construction. Their maximum/live heap counts exclude profiler metadata and are not process RSS. Instrumented durations are never used as native latency evidence. Original profiles stay local; sanitized counters are published.

| Case / version | Maximum live heap before server drop MiB | Live after load MiB | Allocation traffic MiB | Live after server drop bytes |
| --- | --- | --- | --- | --- |
| tiny16 / 0.4.23 | 0.549 | 0.291 | 0.965 | 237410 |
| tiny16 / 0.4.24 | 0.549 | 0.291 | 0.965 | 237410 |
| tiny16 / 0.4.25 | 0.545 | 0.291 | 0.965 | 237410 |
| canonical / 0.4.23 | 1644.433 | 1368.677 | 3025.844 | 237410 |
| canonical / 0.4.24 | 1644.433 | 1368.677 | 3025.844 | 237410 |
| canonical / 0.4.25 | 1388.433 | 1368.677 | 3025.844 | 237410 |

Different allocators, page reuse and OS accounting can produce different peak/live RSS changes from DHAT heap changes. Dropping the raw input does not shrink the retained index, impose an OS quota or prove a faster search. This release publishes both allocation and process observations, including negative timings.

## Executed validation and unchanged API

The full matrix passes **252 default tests, 908 all-feature checks including 39 doctests, 39 isolated feature entries and five Python tests**. Strict Clippy/docs, Rust 1.89 provider-neutral and Rust 1.92 all-feature checks pass. API comparison against 0.4.24 reports **229 pass, 31 skip**. The independent Rust 1.89 consumer, unchanged PDF backend's 14 regressions and 381-file provenance check pass. The dependency audit reports zero known vulnerabilities; maintenance notices remain. Existing malformed/oversized/UTF-8/JSON input, hostile protocol, slow-reader, progress, cancellation, deadline, disconnect, final-reply and shutdown tests remain native. The deliberately ignored crash-writer helper is launched by its parent recovery test.

The same **37 finite Tree Borrows Miri cases** pass; no extra Miri coverage is inferred from this filesystem startup change. Native Tokio/PDF/FFI/ONNX execution stays outside that interpreter scope. Only injected extra floating-point rounding is disabled, not alias/memory checking. This is not an independent penetration, security or soundness audit. Extracted-package tests, four exact-commit CI jobs, a fresh Rust 1.89 registry consumer and tag/VCS/checksum/release-asset agreement gate publication completion.

Engine, SearchOptions, Retriever, WorkControl, McpServer and McpLimits remain. Existing knowledge and saved formats need no migration. Read [API](API.md), [cookbook](API-COOKBOOK.md), [MCP](MCP.md), [Core performance](CORE-PERFORMANCE.md) and [security limits](MCP-SECURITY.md). Source UTF-8 positions remain bound to their exact snapshot revision, not PDF coordinates.

## Frozen method and regression controls

Baseline is exact registry 0.4.24; reference is exact registry 0.4.23. Published archives/checksums and VCS identities are verified before timing. Eighteen actual worker source directories, manifests, locks, method programs, measured binaries, runtime and inputs are hashed and retained. No managed compiler runs during final timings. The [method](validation-0.4.25/README.md) describes the retained inputs and setup requirements; full local orchestration depends on private corpus/proof records. It is not a public standalone reproduction of private source documents. Read [power metadata corrections](BENCHMARK-ERRATA.md); power is observed at campaign start, not continuously controlled.

Both ordinary MCP campaigns, both six-case small-MCP campaigns, both large-corpus guards, three broad controls, exact-limit errors/recovery, document retrieval, cold/warm/controlled Core, corpus export and allocation controls remain. There are **555 fresh RPC-test servers**, separate from the 84 startup-only servers and six allocation-profile loader processes. The fixed 342 authored questions are exposed, not new production or semantic holdouts. There is no fresh LLM/embedding answer-quality or token-saving claim.

### Standard small-MCP campaign

| Case | Ranker | 0.4.23 repeated median / p95 ms | 0.4.24 repeated median / p95 ms | 0.4.25 repeated median / p95 ms | vs 0.4.24 |
| --- | --- | --- | --- | --- | --- |
| tiny | bm25 | 0.141 / 0.180 | 0.142 / 1.436 | 0.143 / 0.222 | +0.25% |
| tiny | tfidf | 0.142 / 0.321 | 0.143 / 2.971 | 0.145 / 0.319 | +1.31% |
| tiny4 | bm25 | 0.219 / 1.065 | 0.234 / 1.815 | 0.234 / 1.104 | -0.03% |
| tiny4 | tfidf | 0.216 / 1.259 | 0.230 / 2.759 | 0.230 / 0.873 | -0.05% |
| tiny16 | bm25 | 0.254 / 0.323 | 0.225 / 0.293 | 0.233 / 0.293 | +3.91% |
| tiny16 | tfidf | 0.259 / 0.361 | 0.226 / 0.302 | 0.232 / 0.292 | +2.73% |
| tiny-no-progress | bm25 | 0.074 / 0.118 | 0.075 / 0.115 | 0.078 / 0.127 | +4.05% |
| tiny-no-progress | tfidf | 0.077 / 0.164 | 0.078 / 0.138 | 0.076 / 0.167 | -1.93% |
| tiny4-no-progress | bm25 | 0.125 / 0.164 | 0.130 / 0.181 | 0.125 / 0.163 | -3.88% |
| tiny4-no-progress | tfidf | 0.123 / 0.151 | 0.126 / 0.176 | 0.122 / 0.149 | -3.33% |
| tiny16-no-progress | bm25 | 0.169 / 0.253 | 0.185 / 0.273 | 0.184 / 0.279 | -0.33% |
| tiny16-no-progress | tfidf | 0.170 / 0.253 | 0.180 / 0.260 | 0.183 / 0.291 | +1.62% |

All initial/repeated per-query means, medians, p95, process sums, startup, RSS and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.4.25/docs/measurements/0.4.25/small-mcp-standard-summary.json). No slower sample is removed.

### Confirmation small-MCP campaign

| Case | Ranker | 0.4.23 repeated median / p95 ms | 0.4.24 repeated median / p95 ms | 0.4.25 repeated median / p95 ms | vs 0.4.24 |
| --- | --- | --- | --- | --- | --- |
| tiny | bm25 | 0.116 / 0.149 | 0.114 / 0.135 | 0.117 / 0.135 | +2.29% |
| tiny | tfidf | 0.117 / 0.171 | 0.114 / 0.147 | 0.115 / 0.198 | +0.82% |
| tiny4 | bm25 | 0.165 / 0.198 | 0.163 / 0.198 | 0.165 / 0.202 | +1.12% |
| tiny4 | tfidf | 0.165 / 0.189 | 0.161 / 0.181 | 0.164 / 0.197 | +1.47% |
| tiny16 | bm25 | 0.186 / 0.222 | 0.186 / 0.246 | 0.184 / 0.207 | -0.83% |
| tiny16 | tfidf | 0.189 / 0.227 | 0.189 / 0.248 | 0.185 / 0.221 | -2.17% |
| tiny-no-progress | bm25 | 0.066 / 0.084 | 0.067 / 0.085 | 0.066 / 0.085 | -1.36% |
| tiny-no-progress | tfidf | 0.068 / 0.102 | 0.067 / 0.097 | 0.066 / 0.114 | -0.53% |
| tiny4-no-progress | bm25 | 0.115 / 0.152 | 0.120 / 0.177 | 0.114 / 0.160 | -4.62% |
| tiny4-no-progress | tfidf | 0.113 / 0.137 | 0.117 / 0.172 | 0.114 / 0.143 | -2.65% |
| tiny16-no-progress | bm25 | 0.134 / 0.187 | 0.134 / 0.180 | 0.134 / 0.181 | +0.06% |
| tiny16-no-progress | tfidf | 0.134 / 0.158 | 0.134 / 0.159 | 0.134 / 0.156 | +0.25% |

All initial/repeated per-query means, medians, p95, process sums, startup, RSS and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.4.25/docs/measurements/0.4.25/small-mcp-confirmation-summary.json). No slower sample is removed.

## Retained large and broad MCP controls

Each large campaign has seven rotating three-version triples: together 42 servers, 2,856 actual schema-valid replies and 1,904 complete comparisons. The three broad campaigns retain 63 servers, 4,284 actual schema replies and 2,856 comparisons. The two small campaigns retain 252 servers, 17,136 actual schema replies and 11,424 comparisons. Status version and schema URNs are the only asserted version differences. Initial/repeated/ranker/query/startup/RSS boundaries remain separate. These results check compatibility and observed performance, not universal improvement.

### Standard large-corpus campaign

| Ranker / phase | 0.4.23 median / p95 ms | 0.4.24 median / p95 ms | 0.4.25 median / p95 ms | vs 0.4.24 | vs 0.4.23 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 2.800 / 6.848 | 2.926 / 4.566 | 2.717 / 4.285 | -7.14% | -2.96% |
| tfidf / initial | 20.295 / 45.774 | 19.332 / 34.595 | 18.011 / 33.006 | -6.83% | -11.26% |
| bm25 / repeated | 2.326 / 6.802 | 2.488 / 4.512 | 2.320 / 3.949 | -6.76% | -0.28% |
| tfidf / repeated | 2.572 / 7.473 | 2.726 / 5.549 | 2.481 / 4.642 | -8.97% | -3.54% |

| Query | Repeated TF-IDF median ms: 0.4.23 / 0.4.24 / 0.4.25 | vs 0.4.24 | Initial TF-IDF mean ms: 0.4.24 → 0.4.25 |
| --- | --- | --- | --- |
| 0 | 2.379 / 2.534 / 2.450 | -3.32% | 23.412 → 22.580 |
| 1 | 2.614 / 2.770 / 2.722 | -1.74% | 22.722 → 21.728 |
| 2 | 4.018 / 4.577 / 4.351 | -4.94% | 33.190 → 33.289 |
| 3 | 0.738 / 0.810 / 0.738 | -8.94% | 1.638 → 1.630 |
| 4 | 2.649 / 2.811 / 2.657 | -5.48% | 15.310 → 14.540 |
| 5 | 3.976 / 4.399 / 4.001 | -9.05% | 27.486 → 26.123 |
| 6 | 1.218 / 1.365 / 1.268 | -7.10% | 5.521 → 5.260 |
| 7 | 4.387 / 4.550 / 4.110 | -9.68% | 28.945 → 27.102 |
| 8 | 2.245 / 2.398 / 2.292 | -4.39% | 2.995 → 2.791 |
| 9 | 0.110 / 0.117 / 0.112 | -4.49% | 0.221 → 0.198 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.23 | 150.354 | 46.664 | 5122.387 | 1724.219 |
| 0.4.24 | 157.221 | 52.170 | 4975.772 | 1724.438 |
| 0.4.25 | 157.204 | 49.723 | 4951.770 | 1724.281 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.4.25/docs/measurements/0.4.25/large-standard-summary.json).

### Confirmation large-corpus campaign

| Ranker / phase | 0.4.23 median / p95 ms | 0.4.24 median / p95 ms | 0.4.25 median / p95 ms | vs 0.4.24 | vs 0.4.23 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 2.574 / 4.161 | 2.559 / 4.268 | 2.611 / 4.087 | +2.01% | +1.43% |
| tfidf / initial | 17.177 / 30.835 | 17.988 / 31.244 | 17.397 / 31.206 | -3.29% | +1.28% |
| bm25 / repeated | 2.123 / 3.525 | 2.150 / 3.643 | 2.154 / 3.627 | +0.18% | +1.44% |
| tfidf / repeated | 2.206 / 3.792 | 2.169 / 3.780 | 2.236 / 3.827 | +3.12% | +1.37% |

| Query | Repeated TF-IDF median ms: 0.4.23 / 0.4.24 / 0.4.25 | vs 0.4.24 | Initial TF-IDF mean ms: 0.4.24 → 0.4.25 |
| --- | --- | --- | --- |
| 0 | 2.185 / 2.184 / 2.286 | +4.70% | 22.552 → 22.023 |
| 1 | 2.282 / 2.212 / 2.382 | +7.66% | 21.440 → 21.162 |
| 2 | 3.696 / 3.588 / 3.601 | +0.37% | 31.402 → 31.906 |
| 3 | 0.679 / 0.669 / 0.680 | +1.70% | 1.520 → 1.488 |
| 4 | 2.283 / 2.222 / 2.303 | +3.64% | 14.089 → 13.743 |
| 5 | 3.455 / 3.429 / 3.465 | +1.07% | 24.371 → 24.463 |
| 6 | 1.068 / 1.120 / 1.083 | -3.37% | 4.527 → 4.521 |
| 7 | 3.475 / 3.642 / 3.648 | +0.17% | 23.810 → 23.649 |
| 8 | 1.981 / 1.959 / 1.997 | +1.94% | 2.322 → 2.276 |
| 9 | 0.107 / 0.103 / 0.105 | +1.97% | 0.181 → 0.167 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.23 | 144.174 | 42.348 | 4836.225 | 1724.172 |
| 0.4.24 | 143.963 | 42.026 | 4826.934 | 1724.250 |
| 0.4.25 | 144.884 | 42.958 | 4824.164 | 1724.031 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.4.25/docs/measurements/0.4.25/large-confirmation-summary.json).

### Standard broad-query campaign

| Ranker / phase | 0.4.23 median / p95 ms | 0.4.24 median / p95 ms | 0.4.25 median / p95 ms | vs 0.4.24 | vs 0.4.23 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 0.488 / 0.586 | 0.478 / 0.566 | 0.481 / 0.590 | +0.73% | -1.24% |
| tfidf / initial | 0.463 / 0.757 | 0.454 / 0.741 | 0.462 / 0.771 | +1.91% | -0.12% |
| bm25 / repeated | 0.407 / 0.660 | 0.425 / 0.675 | 0.409 / 0.733 | -3.63% | +0.46% |
| tfidf / repeated | 0.415 / 0.705 | 0.415 / 0.719 | 0.415 / 0.704 | +0.07% | -0.10% |

| Query | Repeated TF-IDF median ms: 0.4.23 / 0.4.24 / 0.4.25 | vs 0.4.24 | Initial TF-IDF mean ms: 0.4.24 → 0.4.25 |
| --- | --- | --- | --- |
| 0 | 0.407 / 0.402 / 0.405 | +0.80% | 0.475 → 0.497 |
| 1 | 0.369 / 0.367 / 0.363 | -1.11% | 0.415 → 0.426 |
| 2 | 0.411 / 0.406 / 0.413 | +1.67% | 0.479 → 0.456 |
| 3 | 0.684 / 0.678 / 0.671 | -1.09% | 0.735 → 0.762 |
| 4 | 0.408 / 0.406 / 0.418 | +2.97% | 0.391 → 0.401 |
| 5 | 0.404 / 0.401 / 0.389 | -3.00% | 0.396 → 0.401 |
| 6 | 0.408 / 0.405 / 0.405 | -0.07% | 0.456 → 0.469 |
| 7 | 0.683 / 0.666 / 0.669 | +0.43% | 0.689 → 0.717 |
| 8 | 0.359 / 0.363 / 0.361 | -0.55% | 0.378 → 0.385 |
| 9 | 0.672 / 0.683 / 0.667 | -2.38% | 0.675 → 0.696 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.23 | 5.145 | 9.832 | 4.268 | 7.688 |
| 0.4.24 | 5.066 | 9.641 | 3.980 | 7.812 |
| 0.4.25 | 5.216 | 9.559 | 3.989 | 7.625 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.4.25/docs/measurements/0.4.25/wide-standard-summary.json).

### Confirmation broad-query campaign

| Ranker / phase | 0.4.23 median / p95 ms | 0.4.24 median / p95 ms | 0.4.25 median / p95 ms | vs 0.4.24 | vs 0.4.23 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 0.486 / 0.554 | 0.464 / 0.550 | 0.490 / 0.578 | +5.56% | +0.73% |
| tfidf / initial | 0.445 / 0.739 | 0.436 / 0.707 | 0.457 / 0.750 | +4.74% | +2.58% |
| bm25 / repeated | 0.403 / 0.708 | 0.390 / 0.589 | 0.408 / 0.704 | +4.66% | +1.23% |
| tfidf / repeated | 0.401 / 0.681 | 0.396 / 0.664 | 0.399 / 0.679 | +0.63% | -0.64% |

| Query | Repeated TF-IDF median ms: 0.4.23 / 0.4.24 / 0.4.25 | vs 0.4.24 | Initial TF-IDF mean ms: 0.4.24 → 0.4.25 |
| --- | --- | --- | --- |
| 0 | 0.386 / 0.376 / 0.390 | +3.75% | 0.462 → 0.474 |
| 1 | 0.358 / 0.352 / 0.358 | +1.68% | 0.397 → 0.411 |
| 2 | 0.390 / 0.397 / 0.400 | +0.57% | 0.440 → 0.452 |
| 3 | 0.672 / 0.652 / 0.650 | -0.27% | 0.706 → 0.744 |
| 4 | 0.400 / 0.386 / 0.393 | +1.87% | 0.386 → 0.396 |
| 5 | 0.372 / 0.381 / 0.392 | +2.99% | 0.358 → 0.386 |
| 6 | 0.396 / 0.396 / 0.387 | -2.31% | 0.434 → 0.475 |
| 7 | 0.662 / 0.656 / 0.655 | -0.13% | 0.670 → 0.692 |
| 8 | 0.348 / 0.346 / 0.355 | +2.57% | 0.352 → 0.388 |
| 9 | 0.651 / 0.642 / 0.637 | -0.74% | 0.663 → 0.690 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.23 | 5.039 | 9.360 | 3.886 | 7.641 |
| 0.4.24 | 4.870 | 9.168 | 3.900 | 7.781 |
| 0.4.25 | 5.018 | 9.240 | 3.772 | 7.625 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.4.25/docs/measurements/0.4.25/wide-confirmation-summary.json).

### Extended broad-query campaign

| Ranker / phase | 0.4.23 median / p95 ms | 0.4.24 median / p95 ms | 0.4.25 median / p95 ms | vs 0.4.24 | vs 0.4.23 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 0.552 / 0.945 | 0.541 / 0.904 | 0.549 / 0.948 | +1.45% | -0.63% |
| tfidf / initial | 0.699 / 0.908 | 0.697 / 0.914 | 0.705 / 0.956 | +1.14% | +0.92% |
| bm25 / repeated | 0.499 / 1.083 | 0.505 / 1.057 | 0.507 / 0.993 | +0.37% | +1.55% |
| tfidf / repeated | 0.663 / 0.917 | 0.675 / 0.908 | 0.669 / 0.894 | -0.95% | +0.85% |

| Query | Repeated TF-IDF median ms: 0.4.23 / 0.4.24 / 0.4.25 | vs 0.4.24 | Initial TF-IDF mean ms: 0.4.24 → 0.4.25 |
| --- | --- | --- | --- |
| 0 | 0.399 / 0.369 / 0.372 | +0.79% | 0.461 → 0.485 |
| 1 | 0.362 / 0.358 / 0.351 | -1.84% | 0.394 → 0.407 |
| 2 | 0.392 / 0.402 / 0.399 | -0.89% | 0.442 → 0.427 |
| 3 | 0.414 / 0.403 / 0.414 | +2.76% | 0.426 → 0.423 |
| 4 | 0.682 / 0.681 / 0.673 | -1.16% | 0.707 → 0.721 |
| 5 | 0.661 / 0.659 / 0.667 | +1.21% | 0.684 → 0.695 |
| 6 | 0.803 / 0.814 / 0.818 | +0.50% | 0.870 → 0.889 |
| 7 | 0.857 / 0.855 / 0.866 | +1.30% | 0.889 → 0.934 |
| 8 | 0.884 / 0.851 / 0.875 | +2.80% | 0.874 → 0.914 |
| 9 | 0.851 / 0.846 / 0.850 | +0.47% | 0.861 → 0.879 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.23 | 6.750 | 12.659 | 3.961 | 7.922 |
| 0.4.24 | 6.610 | 12.628 | 3.721 | 7.938 |
| 0.4.25 | 6.754 | 12.682 | 3.847 | 7.953 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.4.25/docs/measurements/0.4.25/wide-extended-summary.json).

### Existing limit and recovery checks

Three additional fresh servers reject 257/512 known-term requests with both rankers: twelve expected errors, no partial successful results, error progress and successful later health requests. The existing 256-unique-term assessment limit remains; these are not successful-search latency samples. Large inputs remain subject to cooperative bounds, not an OS CPU/RAM quota.

## Actual MCP: two ordinary campaigns

Each ordinary campaign uses nine cases and five alternating old/new pairs: **90 servers each, 180 total**. Canonical knowledge, NumPy and broad synthetic input request progress; 1/4/16-entry inputs run with and without progress. Each makes ten queries with both rankers, two repeated passes, four concurrent searches, matching/relations, rejected arguments and health; snapshot inputs check exact UTF-8 readbacks.

Together **13,880 actual result/error replies validate against advertised schemas**, and **6,940 complete old/new structured pairs agree by RPC ID**, excluding only checked status version and version-qualified schema URNs. Discovery, progress/no-progress, bounds, ordering, input hashes and EOF shutdown pass. Search and readback time are separate. Child peak RSS includes runtime, buffers and source/index memory, unlike allocation profiles. Retain all slower rows.

### First complete campaign

| Case | Ranker | Initial median / p95 ms: old → new | Repeated median / p95 ms: old → new | Repeated median change |
| --- | --- | --- | --- | --- |
| canonical | bm25 | 2.821 / 4.766 → 2.712 / 4.269 | 2.542 / 5.258 → 2.265 / 3.846 | -10.90% |
| canonical | tfidf | 19.596 / 35.189 → 18.996 / 33.078 | 2.815 / 7.387 → 2.408 / 4.517 | -14.45% |
| numpy | bm25 | 0.888 / 1.254 → 0.893 / 1.301 | 0.770 / 1.109 → 0.768 / 1.131 | -0.22% |
| numpy | tfidf | 1.093 / 2.681 → 1.024 / 2.670 | 0.708 / 1.176 → 0.720 / 1.300 | +1.70% |
| wide | bm25 | 0.490 / 0.591 → 0.486 / 0.577 | 0.410 / 0.620 → 0.399 / 0.623 | -2.74% |
| wide | tfidf | 0.455 / 0.780 → 0.450 / 0.744 | 0.408 / 0.694 → 0.405 / 0.684 | -0.61% |
| tiny | bm25 | 0.142 / 0.238 → 0.128 / 0.233 | 0.120 / 0.148 → 0.116 / 0.142 | -3.38% |
| tiny | tfidf | 0.140 / 0.211 → 0.127 / 0.170 | 0.120 / 0.207 → 0.115 / 0.177 | -4.07% |
| tiny4 | bm25 | 0.199 / 0.319 → 0.186 / 0.313 | 0.168 / 0.212 → 0.169 / 0.212 | +0.61% |
| tiny4 | tfidf | 0.185 / 0.249 → 0.181 / 0.243 | 0.168 / 0.219 → 0.167 / 0.195 | -0.32% |
| tiny16 | bm25 | 0.211 / 0.364 → 0.223 / 0.338 | 0.189 / 0.237 → 0.189 / 0.234 | -0.12% |
| tiny16 | tfidf | 0.201 / 0.295 → 0.213 / 0.278 | 0.189 / 0.233 → 0.191 / 0.240 | +1.28% |
| tiny-no-progress | bm25 | 0.104 / 0.173 → 0.101 / 0.189 | 0.069 / 0.136 → 0.070 / 0.127 | +2.67% |
| tiny-no-progress | tfidf | 0.100 / 0.136 → 0.092 / 0.166 | 0.073 / 0.158 → 0.070 / 0.134 | -4.54% |
| tiny4-no-progress | bm25 | 0.131 / 0.250 → 0.141 / 0.262 | 0.118 / 0.162 → 0.121 / 0.185 | +2.95% |
| tiny4-no-progress | tfidf | 0.120 / 0.184 → 0.132 / 0.224 | 0.115 / 0.144 → 0.119 / 0.192 | +3.40% |
| tiny16-no-progress | bm25 | 0.179 / 0.322 → 0.195 / 4.685 | 0.156 / 0.237 → 0.152 / 0.303 | -2.87% |
| tiny16-no-progress | tfidf | 0.168 / 0.244 → 0.181 / 5.070 | 0.157 / 0.204 → 0.150 / 0.282 | -4.77% |

| Case | Startup median ms: old → new | First TF-IDF median ms: old → new | Peak RSS median MiB: old → new |
| --- | --- | --- | --- |
| canonical | 5036.445 → 5067.560 | 24.166 → 22.945 | 1724.312 → 1724.422 |
| numpy | 138.577 → 137.497 | 1.563 → 1.622 | 57.188 → 57.141 |
| wide | 4.072 → 3.961 | 0.461 → 0.458 | 7.766 → 7.594 |
| tiny | 3.244 → 3.478 | 0.191 → 0.173 | 5.859 → 5.781 |
| tiny4 | 3.557 → 3.279 | 0.255 → 0.233 | 5.938 → 5.844 |
| tiny16 | 3.575 → 3.461 | 0.303 → 0.279 | 6.141 → 6.047 |
| tiny-no-progress | 3.499 → 3.711 | 0.123 → 0.130 | 5.891 → 5.812 |
| tiny4-no-progress | 3.448 → 3.415 | 0.187 → 0.203 | 5.984 → 5.812 |
| tiny16-no-progress | 3.814 → 3.862 | 0.234 → 0.222 | 6.219 → 6.078 |

### Second preplanned complete campaign

| Case | Ranker | Initial median / p95 ms: old → new | Repeated median / p95 ms: old → new | Repeated median change |
| --- | --- | --- | --- | --- |
| canonical | bm25 | 2.909 / 14.567 → 2.900 / 13.310 | 2.368 / 6.535 → 2.379 / 6.698 | +0.47% |
| canonical | tfidf | 20.317 / 38.134 → 21.486 / 42.513 | 2.612 / 8.473 → 2.668 / 8.380 | +2.15% |
| numpy | bm25 | 0.975 / 1.310 → 0.901 / 1.290 | 0.822 / 1.193 → 0.811 / 1.224 | -1.33% |
| numpy | tfidf | 1.063 / 2.737 → 1.047 / 2.787 | 0.758 / 1.469 → 0.748 / 1.892 | -1.29% |
| wide | bm25 | 0.461 / 0.553 → 0.455 / 0.546 | 0.392 / 0.605 → 0.387 / 0.569 | -1.17% |
| wide | tfidf | 0.436 / 0.719 → 0.428 / 0.725 | 0.397 / 0.669 → 0.385 / 0.647 | -2.88% |
| tiny | bm25 | 0.146 / 0.244 → 0.137 / 0.227 | 0.119 / 0.239 → 0.116 / 0.157 | -2.70% |
| tiny | tfidf | 0.137 / 0.237 → 0.126 / 0.173 | 0.118 / 0.237 → 0.117 / 0.202 | -0.64% |
| tiny4 | bm25 | 0.182 / 0.321 → 0.190 / 0.321 | 0.166 / 0.208 → 0.168 / 0.199 | +1.37% |
| tiny4 | tfidf | 0.173 / 0.253 → 0.180 / 0.259 | 0.165 / 0.184 → 0.165 / 0.192 | +0.14% |
| tiny16 | bm25 | 0.202 / 0.335 → 0.199 / 0.326 | 0.186 / 0.211 → 0.182 / 0.204 | -2.26% |
| tiny16 | tfidf | 0.197 / 0.272 → 0.198 / 0.264 | 0.189 / 0.229 → 0.182 / 0.215 | -3.85% |
| tiny-no-progress | bm25 | 0.077 / 0.183 → 0.078 / 0.169 | 0.065 / 0.080 → 0.066 / 0.084 | +2.24% |
| tiny-no-progress | tfidf | 0.075 / 0.114 → 0.075 / 0.113 | 0.065 / 0.109 → 0.065 / 0.106 | -0.06% |
| tiny4-no-progress | bm25 | 0.133 / 0.250 → 0.134 / 0.260 | 0.116 / 0.142 → 0.117 / 0.153 | +1.31% |
| tiny4-no-progress | tfidf | 0.122 / 0.174 → 0.122 / 0.177 | 0.115 / 0.126 → 0.115 / 0.136 | +0.45% |
| tiny16-no-progress | bm25 | 0.148 / 0.275 → 0.149 / 0.277 | 0.136 / 0.182 → 0.136 / 0.178 | +0.32% |
| tiny16-no-progress | tfidf | 0.146 / 0.215 → 0.144 / 0.213 | 0.137 / 0.159 → 0.136 / 0.160 | -0.46% |

| Case | Startup median ms: old → new | First TF-IDF median ms: old → new | Peak RSS median MiB: old → new |
| --- | --- | --- | --- |
| canonical | 4933.312 → 5103.826 | 21.688 → 23.676 | 1724.297 → 1724.422 |
| numpy | 139.093 → 137.995 | 1.609 → 1.587 | 57.219 → 57.125 |
| wide | 4.279 → 4.293 | 0.451 → 0.444 | 7.875 → 7.766 |
| tiny | 4.096 → 3.746 | 0.185 → 0.170 | 5.891 → 5.781 |
| tiny4 | 3.728 → 3.770 | 0.255 → 0.249 | 5.984 → 5.906 |
| tiny16 | 3.771 → 3.569 | 0.275 → 0.265 | 6.188 → 6.125 |
| tiny-no-progress | 3.968 → 3.525 | 0.115 → 0.115 | 5.891 → 5.781 |
| tiny4-no-progress | 3.652 → 3.668 | 0.171 → 0.184 | 6.016 → 5.891 |
| tiny16-no-progress | 3.665 → 3.800 | 0.217 → 0.214 | 6.203 → 6.094 |

## Retained full small-Core regression controls

These separate campaigns compare freshly built exact registry 0.4.23 and 0.4.24 with the candidate, using the same programs. Seven rotating triples per campaign run both the original six cases and a 96-case extension. Warm-up, full original-JSON comparison and index setup occur before timers. Each campaign has **42 processes and 2,142,000 timed Core calls**. Both together have 84 processes, 4,284,000 calls and **2,688 complete original matrix-response comparisons**, with identical results. Signature checks also cover every original-six and timed matrix sample. This is repeated warm Core time, without MCP transport or DHAT instrumentation.

Keep the two campaigns separate. Original six-case calls build SearchOptions inside the measured loop; the extension reuses fixed options. Do not compare their absolute times or merge distributions. The 96 cases combine four entry counts, both rankers and twelve fixed query/filter/limit cases. Different-word, wide, punctuated and Unicode queries retain fallback analysis; short canonical repetitions retain the existing path. Unknown-only ASCII queries, scope rejection and zero limits are also retained; no case is removed for missing the optimized path. Historical absolute values with different programs/loads are not a reference substitute.

### First small Core campaign

#### Original six warm cases

| Fixture | 0.4.23 median ns | 0.4.24 median ns | 0.4.25 median ns | vs 0.4.24 | vs 0.4.23 |
| --- | --- | --- | --- | --- | --- |
| tiny16_bm25 | 990.833 | 940.833 | 967.875 | +2.87% | -2.32% |
| tiny16_tfidf | 1079.500 | 1029.625 | 1025.792 | -0.37% | -4.98% |
| tiny4_bm25 | 319.250 | 298.708 | 306.500 | +2.61% | -3.99% |
| tiny4_tfidf | 348.125 | 325.083 | 331.709 | +2.04% | -4.72% |
| tiny_bm25 | 134.583 | 132.708 | 124.833 | -5.93% | -7.24% |
| tiny_tfidf | 139.333 | 131.958 | 133.917 | +1.48% | -3.89% |

Higher candidate medians versus 0.4.24: **4 of 6**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](https://github.com/LEVOGNE/airust/blob/v0.4.25/docs/measurements/0.4.25/small-reference-summary.json). No threshold-based rejection or removal of slow timing samples is used.

#### All 96 extended cases

| Fixture | 0.4.23 median ns | 0.4.24 median ns | 0.4.25 median ns | vs 0.4.24 | vs 0.4.23 |
| --- | --- | --- | --- | --- | --- |
| 16_Bm25_disjoint | 459.291 | 467.750 | 445.083 | -4.85% | -3.09% |
| 16_Bm25_limit_one | 632.917 | 618.084 | 592.291 | -4.17% | -6.42% |
| 16_Bm25_limit_zero | 3.875 | 3.916 | 3.083 | -21.27% | -20.44% |
| 16_Bm25_multi | 1755.333 | 1805.417 | 1707.792 | -5.41% | -2.71% |
| 16_Bm25_punctuation | 154.625 | 165.334 | 150.959 | -8.69% | -2.37% |
| 16_Bm25_repeated | 1186.000 | 1198.625 | 1161.750 | -3.08% | -2.04% |
| 16_Bm25_scope | 1000.208 | 1039.375 | 983.458 | -5.38% | -1.67% |
| 16_Bm25_single | 1123.500 | 1172.333 | 1079.291 | -7.94% | -3.93% |
| 16_Bm25_unicode | 1408.917 | 1466.291 | 1411.416 | -3.74% | +0.18% |
| 16_Bm25_unknown | 20.584 | 21.458 | 20.250 | -5.63% | -1.62% |
| 16_Bm25_unknown_added | 1285.125 | 1310.083 | 1242.917 | -5.13% | -3.28% |
| 16_Bm25_wide | 18348.000 | 18252.625 | 17491.958 | -4.17% | -4.67% |
| 16_Tfidf_disjoint | 458.666 | 450.708 | 479.208 | +6.32% | +4.48% |
| 16_Tfidf_limit_one | 717.041 | 704.833 | 730.959 | +3.71% | +1.94% |
| 16_Tfidf_limit_zero | 3.125 | 3.125 | 4.041 | +29.31% | +29.31% |
| 16_Tfidf_multi | 1799.500 | 1875.667 | 1801.916 | -3.93% | +0.13% |
| 16_Tfidf_punctuation | 150.167 | 151.166 | 155.959 | +3.17% | +3.86% |
| 16_Tfidf_repeated | 1238.875 | 1233.958 | 1275.084 | +3.33% | +2.92% |
| 16_Tfidf_scope | 1068.333 | 1083.709 | 1110.875 | +2.51% | +3.98% |
| 16_Tfidf_single | 1243.125 | 1236.375 | 1226.875 | -0.77% | -1.31% |
| 16_Tfidf_unicode | 1489.583 | 1479.625 | 1534.458 | +3.71% | +3.01% |
| 16_Tfidf_unknown | 20.375 | 20.167 | 21.084 | +4.55% | +3.48% |
| 16_Tfidf_unknown_added | 1341.542 | 1310.750 | 1359.875 | +3.75% | +1.37% |
| 16_Tfidf_wide | 18395.000 | 18248.250 | 17938.834 | -1.70% | -2.48% |
| 1_Bm25_disjoint | 60.750 | 60.708 | 58.958 | -2.88% | -2.95% |
| 1_Bm25_limit_one | 133.875 | 136.041 | 139.042 | +2.21% | +3.86% |
| 1_Bm25_limit_zero | 3.792 | 3.042 | 3.083 | +1.35% | -18.70% |
| 1_Bm25_multi | 271.542 | 286.417 | 298.666 | +4.28% | +9.99% |
| 1_Bm25_punctuation | 155.541 | 154.750 | 151.667 | -1.99% | -2.49% |
| 1_Bm25_repeated | 162.500 | 166.917 | 166.334 | -0.35% | +2.36% |
| 1_Bm25_scope | 141.125 | 141.875 | 137.500 | -3.08% | -2.57% |
| 1_Bm25_single | 155.125 | 156.917 | 145.166 | -7.49% | -6.42% |
| 1_Bm25_unicode | 448.250 | 457.667 | 472.125 | +3.16% | +5.33% |
| 1_Bm25_unknown | 21.041 | 20.750 | 21.542 | +3.82% | +2.38% |
| 1_Bm25_unknown_added | 261.334 | 276.708 | 263.042 | -4.94% | +0.65% |
| 1_Bm25_wide | 4739.209 | 4772.208 | 4724.458 | -1.00% | -0.31% |
| 1_Tfidf_disjoint | 60.500 | 63.792 | 67.250 | +5.42% | +11.16% |
| 1_Tfidf_limit_one | 143.875 | 146.250 | 146.041 | -0.14% | +1.51% |
| 1_Tfidf_limit_zero | 2.958 | 3.000 | 3.250 | +8.33% | +9.87% |
| 1_Tfidf_multi | 279.250 | 282.584 | 350.667 | +24.09% | +25.57% |
| 1_Tfidf_punctuation | 156.458 | 155.000 | 161.375 | +4.11% | +3.14% |
| 1_Tfidf_repeated | 166.667 | 165.458 | 206.833 | +25.01% | +24.10% |
| 1_Tfidf_scope | 146.334 | 154.125 | 159.542 | +3.51% | +9.03% |
| 1_Tfidf_single | 145.917 | 144.417 | 181.416 | +25.62% | +24.33% |
| 1_Tfidf_unicode | 455.209 | 460.750 | 561.541 | +21.88% | +23.36% |
| 1_Tfidf_unknown | 20.500 | 20.834 | 23.584 | +13.20% | +15.04% |
| 1_Tfidf_unknown_added | 251.000 | 252.416 | 308.917 | +22.38% | +23.07% |
| 1_Tfidf_wide | 4535.917 | 4858.917 | 4650.916 | -4.28% | +2.54% |
| 33_Bm25_disjoint | 932.417 | 946.833 | 956.291 | +1.00% | +2.56% |
| 33_Bm25_limit_one | 1176.792 | 1193.250 | 1236.958 | +3.66% | +5.11% |
| 33_Bm25_limit_zero | 3.083 | 2.958 | 3.708 | +25.35% | +20.27% |
| 33_Bm25_multi | 2625.584 | 2702.375 | 2597.167 | -3.89% | -1.08% |
| 33_Bm25_punctuation | 152.834 | 150.333 | 151.125 | +0.53% | -1.12% |
| 33_Bm25_repeated | 1776.167 | 1812.167 | 1754.375 | -3.19% | -1.23% |
| 33_Bm25_scope | 1758.375 | 1813.417 | 1788.208 | -1.39% | +1.70% |
| 33_Bm25_single | 1719.250 | 1730.917 | 1705.583 | -1.46% | -0.79% |
| 33_Bm25_unicode | 1983.500 | 1994.166 | 2006.542 | +0.62% | +1.16% |
| 33_Bm25_unknown | 20.958 | 19.916 | 20.417 | +2.52% | -2.58% |
| 33_Bm25_unknown_added | 1855.875 | 1932.417 | 1883.083 | -2.55% | +1.47% |
| 33_Bm25_wide | 26986.000 | 26730.000 | 29624.958 | +10.83% | +9.78% |
| 33_Tfidf_disjoint | 942.166 | 972.750 | 954.417 | -1.88% | +1.30% |
| 33_Tfidf_limit_one | 1426.458 | 1377.083 | 1394.292 | +1.25% | -2.25% |
| 33_Tfidf_limit_zero | 3.125 | 2.959 | 3.000 | +1.39% | -4.00% |
| 33_Tfidf_multi | 2797.084 | 2857.208 | 2871.000 | +0.48% | +2.64% |
| 33_Tfidf_punctuation | 149.208 | 152.250 | 152.209 | -0.03% | +2.01% |
| 33_Tfidf_repeated | 1946.958 | 1991.167 | 2067.500 | +3.83% | +6.19% |
| 33_Tfidf_scope | 1872.167 | 1921.458 | 1945.666 | +1.26% | +3.93% |
| 33_Tfidf_single | 1896.042 | 1922.834 | 1997.083 | +3.86% | +5.33% |
| 33_Tfidf_unicode | 2115.791 | 2164.500 | 2196.417 | +1.47% | +3.81% |
| 33_Tfidf_unknown | 21.708 | 20.500 | 20.667 | +0.81% | -4.80% |
| 33_Tfidf_unknown_added | 2060.583 | 2054.333 | 2162.667 | +5.27% | +4.95% |
| 33_Tfidf_wide | 29452.166 | 27715.750 | 29346.458 | +5.88% | -0.36% |
| 4_Bm25_disjoint | 110.417 | 108.458 | 113.459 | +4.61% | +2.76% |
| 4_Bm25_limit_one | 193.583 | 192.042 | 194.708 | +1.39% | +0.58% |
| 4_Bm25_limit_zero | 3.000 | 2.917 | 3.125 | +7.13% | +4.17% |
| 4_Bm25_multi | 582.041 | 571.292 | 595.542 | +4.24% | +2.32% |
| 4_Bm25_punctuation | 151.542 | 147.667 | 154.167 | +4.40% | +1.73% |
| 4_Bm25_repeated | 354.792 | 351.083 | 355.375 | +1.22% | +0.16% |
| 4_Bm25_scope | 242.375 | 242.541 | 242.167 | -0.15% | -0.09% |
| 4_Bm25_single | 345.958 | 328.542 | 335.584 | +2.14% | -3.00% |
| 4_Bm25_unicode | 635.083 | 613.458 | 632.125 | +3.04% | -0.47% |
| 4_Bm25_unknown | 20.792 | 19.833 | 20.791 | +4.83% | -0.00% |
| 4_Bm25_unknown_added | 434.458 | 421.500 | 439.959 | +4.38% | +1.27% |
| 4_Bm25_wide | 7719.042 | 7321.458 | 7620.542 | +4.09% | -1.28% |
| 4_Tfidf_disjoint | 114.291 | 109.791 | 112.459 | +2.43% | -1.60% |
| 4_Tfidf_limit_one | 223.792 | 222.833 | 228.250 | +2.43% | +1.99% |
| 4_Tfidf_limit_zero | 3.708 | 3.750 | 2.958 | -21.12% | -20.23% |
| 4_Tfidf_multi | 614.500 | 586.167 | 637.625 | +8.78% | +3.76% |
| 4_Tfidf_punctuation | 154.416 | 150.250 | 149.750 | -0.33% | -3.02% |
| 4_Tfidf_repeated | 379.625 | 378.250 | 379.625 | +0.36% | +0.00% |
| 4_Tfidf_scope | 255.375 | 251.875 | 250.583 | -0.51% | -1.88% |
| 4_Tfidf_single | 364.500 | 360.917 | 363.666 | +0.76% | -0.23% |
| 4_Tfidf_unicode | 674.208 | 657.458 | 685.375 | +4.25% | +1.66% |
| 4_Tfidf_unknown | 20.709 | 20.792 | 20.292 | -2.40% | -2.01% |
| 4_Tfidf_unknown_added | 462.833 | 453.666 | 473.250 | +4.32% | +2.25% |
| 4_Tfidf_wide | 7809.625 | 7727.708 | 7443.417 | -3.68% | -4.69% |

Higher candidate medians versus 0.4.24: **59 of 96**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](https://github.com/LEVOGNE/airust/blob/v0.4.25/docs/measurements/0.4.25/small-reference-summary.json). No threshold-based rejection or removal of slow timing samples is used.

### Second preplanned small Core campaign

#### Original six warm cases

| Fixture | 0.4.23 median ns | 0.4.24 median ns | 0.4.25 median ns | vs 0.4.24 | vs 0.4.23 |
| --- | --- | --- | --- | --- | --- |
| tiny16_bm25 | 916.917 | 917.666 | 915.875 | -0.20% | -0.11% |
| tiny16_tfidf | 985.792 | 985.250 | 994.084 | +0.90% | +0.84% |
| tiny4_bm25 | 291.000 | 284.625 | 287.833 | +1.13% | -1.09% |
| tiny4_tfidf | 307.875 | 310.709 | 308.458 | -0.72% | +0.19% |
| tiny_bm25 | 119.625 | 122.292 | 117.833 | -3.65% | -1.50% |
| tiny_tfidf | 127.458 | 128.000 | 125.250 | -2.15% | -1.73% |

Higher candidate medians versus 0.4.24: **2 of 6**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](https://github.com/LEVOGNE/airust/blob/v0.4.25/docs/measurements/0.4.25/small-confirmation-summary.json). No threshold-based rejection or removal of slow timing samples is used.

#### All 96 extended cases

| Fixture | 0.4.23 median ns | 0.4.24 median ns | 0.4.25 median ns | vs 0.4.24 | vs 0.4.23 |
| --- | --- | --- | --- | --- | --- |
| 16_Bm25_disjoint | 423.541 | 427.042 | 417.541 | -2.22% | -1.42% |
| 16_Bm25_limit_one | 572.375 | 567.750 | 558.792 | -1.58% | -2.37% |
| 16_Bm25_limit_zero | 2.792 | 2.792 | 3.625 | +29.84% | +29.84% |
| 16_Bm25_multi | 1682.750 | 1633.709 | 1606.750 | -1.65% | -4.52% |
| 16_Bm25_punctuation | 142.958 | 145.584 | 150.459 | +3.35% | +5.25% |
| 16_Bm25_repeated | 1112.458 | 1110.792 | 1069.459 | -3.72% | -3.87% |
| 16_Bm25_scope | 955.792 | 950.834 | 934.208 | -1.75% | -2.26% |
| 16_Bm25_single | 1114.166 | 1099.291 | 1060.250 | -3.55% | -4.84% |
| 16_Bm25_unicode | 1356.125 | 1354.709 | 1346.292 | -0.62% | -0.73% |
| 16_Bm25_unknown | 19.500 | 19.541 | 19.416 | -0.64% | -0.43% |
| 16_Bm25_unknown_added | 1209.500 | 1196.167 | 1175.750 | -1.71% | -2.79% |
| 16_Bm25_wide | 17043.709 | 17263.125 | 16679.875 | -3.38% | -2.13% |
| 16_Tfidf_disjoint | 428.375 | 426.083 | 423.833 | -0.53% | -1.06% |
| 16_Tfidf_limit_one | 669.750 | 679.833 | 666.167 | -2.01% | -0.53% |
| 16_Tfidf_limit_zero | 3.833 | 2.833 | 2.833 | +0.00% | -26.09% |
| 16_Tfidf_multi | 1731.250 | 1747.250 | 1679.125 | -3.90% | -3.01% |
| 16_Tfidf_punctuation | 148.042 | 142.958 | 144.959 | +1.40% | -2.08% |
| 16_Tfidf_repeated | 1223.959 | 1239.541 | 1178.833 | -4.90% | -3.69% |
| 16_Tfidf_scope | 1030.250 | 1038.291 | 1009.833 | -2.74% | -1.98% |
| 16_Tfidf_single | 1205.334 | 1216.125 | 1159.959 | -4.62% | -3.76% |
| 16_Tfidf_unicode | 1432.958 | 1449.459 | 1409.334 | -2.77% | -1.65% |
| 16_Tfidf_unknown | 19.375 | 19.416 | 19.333 | -0.43% | -0.22% |
| 16_Tfidf_unknown_added | 1312.292 | 1310.208 | 1252.125 | -4.43% | -4.58% |
| 16_Tfidf_wide | 17123.083 | 17280.333 | 16805.917 | -2.75% | -1.85% |
| 1_Bm25_disjoint | 55.416 | 57.125 | 57.125 | +0.00% | +3.08% |
| 1_Bm25_limit_one | 123.041 | 125.084 | 124.458 | -0.50% | +1.15% |
| 1_Bm25_limit_zero | 2.750 | 2.833 | 2.791 | -1.48% | +1.49% |
| 1_Bm25_multi | 270.791 | 271.125 | 275.625 | +1.66% | +1.79% |
| 1_Bm25_punctuation | 144.125 | 145.584 | 145.000 | -0.40% | +0.61% |
| 1_Bm25_repeated | 157.084 | 153.250 | 157.791 | +2.96% | +0.45% |
| 1_Bm25_scope | 130.459 | 131.667 | 131.583 | -0.06% | +0.86% |
| 1_Bm25_single | 140.667 | 143.959 | 143.250 | -0.49% | +1.84% |
| 1_Bm25_unicode | 433.583 | 428.917 | 433.750 | +1.13% | +0.04% |
| 1_Bm25_unknown | 19.334 | 19.750 | 19.458 | -1.48% | +0.64% |
| 1_Bm25_unknown_added | 238.333 | 257.375 | 248.208 | -3.56% | +4.14% |
| 1_Bm25_wide | 4410.791 | 4417.584 | 4417.417 | -0.00% | +0.15% |
| 1_Tfidf_disjoint | 56.500 | 58.458 | 60.875 | +4.13% | +7.74% |
| 1_Tfidf_limit_one | 130.916 | 134.458 | 138.291 | +2.85% | +5.63% |
| 1_Tfidf_limit_zero | 2.750 | 2.833 | 2.791 | -1.48% | +1.49% |
| 1_Tfidf_multi | 269.292 | 268.958 | 273.166 | +1.56% | +1.44% |
| 1_Tfidf_punctuation | 144.375 | 145.042 | 144.917 | -0.09% | +0.38% |
| 1_Tfidf_repeated | 151.917 | 156.041 | 153.833 | -1.42% | +1.26% |
| 1_Tfidf_scope | 138.875 | 141.917 | 142.250 | +0.23% | +2.43% |
| 1_Tfidf_single | 132.417 | 136.875 | 138.958 | +1.52% | +4.94% |
| 1_Tfidf_unicode | 435.458 | 434.792 | 439.667 | +1.12% | +0.97% |
| 1_Tfidf_unknown | 19.291 | 19.708 | 19.458 | -1.27% | +0.87% |
| 1_Tfidf_unknown_added | 231.625 | 239.292 | 238.625 | -0.28% | +3.02% |
| 1_Tfidf_wide | 4506.750 | 4425.333 | 4372.917 | -1.18% | -2.97% |
| 33_Bm25_disjoint | 899.583 | 890.542 | 894.167 | +0.41% | -0.60% |
| 33_Bm25_limit_one | 1137.416 | 1129.583 | 1142.583 | +1.15% | +0.45% |
| 33_Bm25_limit_zero | 2.750 | 3.833 | 2.791 | -27.18% | +1.49% |
| 33_Bm25_multi | 2489.459 | 2478.542 | 2457.709 | -0.84% | -1.28% |
| 33_Bm25_punctuation | 144.875 | 145.250 | 143.875 | -0.95% | -0.69% |
| 33_Bm25_repeated | 1694.875 | 1703.458 | 1665.291 | -2.24% | -1.75% |
| 33_Bm25_scope | 1720.000 | 1714.666 | 1687.958 | -1.56% | -1.86% |
| 33_Bm25_single | 1630.542 | 1623.792 | 1613.666 | -0.62% | -1.03% |
| 33_Bm25_unicode | 1896.792 | 1892.334 | 1870.125 | -1.17% | -1.41% |
| 33_Bm25_unknown | 19.416 | 19.417 | 19.375 | -0.22% | -0.21% |
| 33_Bm25_unknown_added | 1802.375 | 1773.250 | 1755.791 | -0.98% | -2.58% |
| 33_Bm25_wide | 26124.709 | 26083.583 | 25920.417 | -0.63% | -0.78% |
| 33_Tfidf_disjoint | 916.166 | 890.916 | 903.542 | +1.42% | -1.38% |
| 33_Tfidf_limit_one | 1334.834 | 1313.583 | 1285.833 | -2.11% | -3.67% |
| 33_Tfidf_limit_zero | 2.792 | 3.542 | 2.792 | -21.17% | +0.00% |
| 33_Tfidf_multi | 2701.333 | 2643.375 | 2646.458 | +0.12% | -2.03% |
| 33_Tfidf_punctuation | 144.459 | 144.916 | 145.541 | +0.43% | +0.75% |
| 33_Tfidf_repeated | 1879.834 | 1860.625 | 1854.083 | -0.35% | -1.37% |
| 33_Tfidf_scope | 1815.667 | 1815.500 | 1794.459 | -1.16% | -1.17% |
| 33_Tfidf_single | 1824.584 | 1843.209 | 1773.958 | -3.76% | -2.77% |
| 33_Tfidf_unicode | 2073.375 | 2045.166 | 2044.584 | -0.03% | -1.39% |
| 33_Tfidf_unknown | 19.541 | 19.500 | 19.542 | +0.22% | +0.01% |
| 33_Tfidf_unknown_added | 1985.083 | 1951.208 | 1935.333 | -0.81% | -2.51% |
| 33_Tfidf_wide | 26371.292 | 26543.833 | 26552.958 | +0.03% | +0.69% |
| 4_Bm25_disjoint | 105.250 | 105.250 | 106.542 | +1.23% | +1.23% |
| 4_Bm25_limit_one | 189.917 | 188.083 | 186.334 | -0.93% | -1.89% |
| 4_Bm25_limit_zero | 2.750 | 3.625 | 2.792 | -22.98% | +1.53% |
| 4_Bm25_multi | 587.292 | 551.042 | 551.958 | +0.17% | -6.02% |
| 4_Bm25_punctuation | 144.959 | 143.667 | 145.792 | +1.48% | +0.57% |
| 4_Bm25_repeated | 340.958 | 333.750 | 334.083 | +0.10% | -2.02% |
| 4_Bm25_scope | 232.458 | 231.750 | 234.583 | +1.22% | +0.91% |
| 4_Bm25_single | 323.375 | 312.917 | 317.375 | +1.42% | -1.86% |
| 4_Bm25_unicode | 612.708 | 606.208 | 612.667 | +1.07% | -0.01% |
| 4_Bm25_unknown | 19.541 | 19.416 | 19.500 | +0.43% | -0.21% |
| 4_Bm25_unknown_added | 425.792 | 422.583 | 424.708 | +0.50% | -0.25% |
| 4_Bm25_wide | 7468.875 | 7325.833 | 7288.917 | -0.50% | -2.41% |
| 4_Tfidf_disjoint | 107.208 | 105.125 | 108.542 | +3.25% | +1.24% |
| 4_Tfidf_limit_one | 213.917 | 212.708 | 216.291 | +1.68% | +1.11% |
| 4_Tfidf_limit_zero | 2.792 | 2.792 | 3.667 | +31.34% | +31.34% |
| 4_Tfidf_multi | 576.500 | 573.208 | 577.209 | +0.70% | +0.12% |
| 4_Tfidf_punctuation | 145.375 | 144.667 | 145.958 | +0.89% | +0.40% |
| 4_Tfidf_repeated | 366.583 | 360.375 | 366.375 | +1.66% | -0.06% |
| 4_Tfidf_scope | 245.375 | 244.042 | 245.250 | +0.49% | -0.05% |
| 4_Tfidf_single | 351.667 | 343.959 | 352.500 | +2.48% | +0.24% |
| 4_Tfidf_unicode | 644.125 | 643.792 | 647.542 | +0.58% | +0.53% |
| 4_Tfidf_unknown | 19.542 | 19.375 | 19.375 | +0.00% | -0.85% |
| 4_Tfidf_unknown_added | 444.542 | 448.417 | 450.417 | +0.45% | +1.32% |
| 4_Tfidf_wide | 7431.875 | 7354.208 | 7254.625 | -1.35% | -2.38% |

Higher candidate medians versus 0.4.24: **38 of 96**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](https://github.com/LEVOGNE/airust/blob/v0.4.25/docs/measurements/0.4.25/small-confirmation-summary.json). No threshold-based rejection or removal of slow timing samples is used.

## Direct canonical search with explicit control modes

Fourteen processes (seven alternating pairs) retain one warmed Engine each. Ten rounds rotate plain, deadline and progress within each query/ranker pair: **8,400 timed Retriever calls**, excluding loading/readiness, stdio and profiling. Each progress request includes an observer and the same deadline/cancellation setup. Every signature agrees; **140 complete original response JSON pairs match**, and control modes also match plain output. Full readiness source text stays private. These are direct retrieval boundaries, not MCP round trips or capacity evidence.

| Control | Ranker | 0.4.24 median / p95 ms | 0.4.25 median / p95 ms | Median change |
| --- | --- | --- | --- | --- |
| plain | Bm25 | 1.504 / 3.727 | 1.477 / 3.335 | -1.77% |
| plain | Tfidf | 1.540 / 4.792 | 1.622 / 4.066 | +5.32% |
| deadline | Bm25 | 1.746 / 4.382 | 1.768 / 3.966 | +1.27% |
| deadline | Tfidf | 1.912 / 6.269 | 2.004 / 5.411 | +4.82% |
| progress | Bm25 | 1.753 / 5.137 | 1.795 / 3.963 | +2.42% |
| progress | Tfidf | 1.933 / 6.285 | 2.053 / 5.362 | +6.17% |

| Query | Candidates | Progress BM25 ms: old → new | Progress TF-IDF ms: old → new |
| --- | --- | --- | --- |
| 0 | 12365 | 1.701 → 1.756 | 1.916 → 2.122 |
| 1 | 13864 | 1.821 → 1.837 | 2.003 → 2.151 |
| 2 | 24299 | 3.258 → 3.243 | 3.508 → 3.853 |
| 3 | 596 | 0.383 → 0.333 | 0.331 → 0.325 |
| 4 | 15403 | 1.904 → 1.929 | 2.025 → 2.143 |
| 5 | 24140 | 2.969 → 3.142 | 3.300 → 3.496 |
| 6 | 3358 | 0.704 → 0.725 | 0.736 → 0.754 |
| 7 | 29600 | 2.960 → 3.019 | 3.475 → 3.846 |
| 8 | 11504 | 1.750 → 1.682 | 1.704 → 1.793 |
| 9 | 0 | 0.002 → 0.002 | 0.001 → 0.001 |

## Separate canonical three-version reference

Five rotating triples compare fresh exact 0.4.23/0.4.24/candidate servers on canonical knowledge with requested progress. Fifteen servers validate **1,020 actual schema replies**; **680 complete structured comparisons** against the candidate match by RPC ID except checked status version and version-qualified schema URNs. Schemas, source ranges and inputs agree. This control is separate from the two ordinary campaigns. It checks whether the previous large-query benefit remains visible against the older release; it does not establish the cause of any historical regression.

| Version | Ranker | Initial median / p95 ms | Repeated median / p95 ms | Peak child RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.23 | bm25 | 2.590 / 4.079 | 2.124 / 3.530 | 1724.188 |
| 0.4.23 | tfidf | 17.318 / 31.519 | 2.201 / 3.690 | 1724.188 |
| 0.4.24 | bm25 | 2.637 / 4.088 | 2.085 / 3.505 | 1724.234 |
| 0.4.24 | tfidf | 17.208 / 30.807 | 2.143 / 3.591 | 1724.234 |
| 0.4.25 | bm25 | 2.684 / 4.129 | 2.137 / 3.658 | 1724.047 |
| 0.4.25 | tfidf | 17.200 / 31.858 | 2.171 / 3.687 | 1724.047 |

| Query | 0.4.23 repeated TF-IDF ms | 0.4.24 repeated TF-IDF ms | 0.4.25 repeated TF-IDF ms |
| --- | --- | --- | --- |
| 0 | 2.110 | 2.148 | 2.224 |
| 1 | 2.240 | 2.242 | 2.354 |
| 2 | 3.573 | 3.522 | 3.638 |
| 3 | 0.680 | 0.664 | 0.672 |
| 4 | 2.246 | 2.185 | 2.168 |
| 5 | 3.364 | 3.378 | 3.389 |
| 6 | 1.059 | 1.029 | 1.041 |
| 7 | 3.536 | 3.483 | 3.533 |
| 8 | 1.902 | 1.963 | 1.956 |
| 9 | 0.104 | 0.103 | 0.103 |

## Canonical first-pass heap traffic

Two instrumented processes index all 167,942 records before profiling the same ten initial TF-IDF queries, then drop results before net deltas. The existing direct candidate collection remains; the previous large-query posting cursors and norm memo/cache still exist. Heap traffic, net persistent hints, stack state and whole-process RSS are different costs. Instrumented durations are excluded from latency comparisons.

| Metric | 0.4.24 | 0.4.25 |
| --- | --- | --- |
| Retained hint bytes | 2228312 | 2228312 |
| Retained hint blocks | 2 | 2 |
| Total query allocated bytes | 7212357 | 7212357 |
| Total query allocation blocks | 401 | 401 |

## Development trials and host-load limits

The full three-triple development MCP pilot is retained separately from all final campaigns. Its source snapshots include the MCP implementation and Core; all complete compared structured results agree. Initial/repeated data stay separate and no final sample is replaced by a pilot or a faster rerun. Earlier releases retain their own rejected trials. These are development observations, not fresh production or semantic holdouts.

Two-second monitoring retains **724 snapshots**, including **213 with other rustc processes**, maximum 4. All managed compilers finish before timing. Other workloads are not stopped, and snapshots do not prove continuous idleness or identify a timing cause. CPU counts include the active benchmark workers.

| Stage start | Other rustc count | Other rustc CPU total | Processes over 50% CPU |
| --- | --- | --- | --- |
| small-mcp-standard | 2 | 1180.100 | 3 |
| small-reference | 3 | 1182.000 | 5 |
| large-standard | 1 | 100.000 | 4 |
| wide-standard | 0 | 0.000 | 2 |
| retrieval | 1 | 99.600 | 3 |
| query | 0 | 0.000 | 2 |
| build | 1 | 376.500 | 4 |
| controlled | 1 | 100.000 | 3 |
| mcp | 1 | 1344.800 | 4 |
| corpus | 1 | 1186.700 | 2 |
| mcp-confirmation | 0 | 0.000 | 1 |
| reference | 0 | 0.000 | 2 |
| wide-confirmation | 0 | 0.000 | 2 |
| wide-extended | 0 | 0.000 | 2 |
| large-confirmation | 0 | 0.000 | 2 |
| small-confirmation | 0 | 0.000 | 2 |
| small-mcp-confirmation | 0 | 0.000 | 2 |
| limits | 0 | 0.000 | 2 |
| startup-standard | 0 | 0.000 | 2 |
| startup-confirmation | 0 | 0.000 | 2 |
| startup-heap | 0 | 0.000 | 2 |

All observed slower cases remain in their own distributions; no sample is replaced because of load. One machine, randomized hashing, exposed questions, tuned fixtures and finite trials limit generalization.

## Retained Core controls

These controls retain positive and negative timing changes. Single known-term requests retain the direct candidate path and exactly two retain ordered merging. Wider requests keep the heap/set strategies; large frequency cursors remain available afterward. Large sorted candidate sets can use the bounded document cursor after 128 candidates; earlier shared explanation intersection remains. Read every row independently, including slower cases. Seven alternating pairs give fourteen plain processes per family. Norms are initialized for warm timing; two separate DHAT processes measure allocations. Instrumented durations are excluded from latency comparisons. Every result signature agrees and temporary-call retained deltas are zero.

### Small warm Core: 84,000 calls on 1/4/16 entries

| Fixture | Median change | Blocks/call: old → new | Allocated bytes/call: old → new |
| --- | --- | --- | --- |
| tiny16_bm25 | +0.64% | 26.0 → 26.0 | 2156.0 → 2156.0 |
| tiny16_tfidf | +4.32% | 26.0 → 26.0 | 2156.0 → 2156.0 |
| tiny4_bm25 | +1.84% | 12.0 → 12.0 | 824.0 → 824.0 |
| tiny4_tfidf | +2.00% | 12.0 → 12.0 | 824.0 → 824.0 |
| tiny_bm25 | -1.80% | 6.0 → 6.0 | 398.0 → 398.0 |
| tiny_tfidf | +0.81% | 6.0 → 6.0 | 398.0 → 398.0 |

### All 25 warm controls: 282,800 calls

| Fixture | Median change | Blocks/call: old → new | Allocated bytes/call: old → new |
| --- | --- | --- | --- |
| coverage_repeated | -0.93% | 1047.0 → 1047.0 | 92942.0 → 92942.0 |
| coverage_small | -0.61% | 784.0 → 784.0 | 78046.0 → 78046.0 |
| excerpt_ascii | -0.74% | 762.0 → 762.0 | 252438.0 → 252438.0 |
| excerpt_mixed | +0.51% | 569.0 → 569.0 | 417113.0 → 417113.0 |
| excerpt_unicode | -0.84% | 197.0 → 197.0 | 72254.0 → 72254.0 |
| long_repeated | -1.46% | 4.0 → 4.0 | 18798.0 → 18798.0 |
| mixed_unicode | -1.78% | 7.0 → 7.0 | 3155.0 → 3155.0 |
| punctuation | -1.08% | 1.0 → 1.0 | 768.0 → 768.0 |
| relevant_retrieval | +0.60% | 812.0 → 812.0 | 93562.0 → 93562.0 |
| repeated_search | -0.55% | 31.0 → 31.0 | 18156.0 → 18156.0 |
| repeated_short | -3.77% | 4.0 → 4.0 | 1656.0 → 1656.0 |
| sparse_bm25_128 | -0.99% | 359.0 → 359.0 | 29074.0 → 29074.0 |
| sparse_bm25_33 | -2.15% | 241.0 → 241.0 | 16309.0 → 16309.0 |
| sparse_bm25_512 | +1.08% | 1337.0 → 1337.0 | 119124.0 → 119124.0 |
| sparse_tfidf_128 | +0.90% | 474.0 → 474.0 | 35140.0 → 35140.0 |
| sparse_tfidf_33 | -1.63% | 241.0 → 241.0 | 16309.0 → 16309.0 |
| sparse_tfidf_512 | +1.00% | 1337.0 → 1337.0 | 119124.0 → 119124.0 |
| tfidf_union_32 | -1.11% | 244.0 → 244.0 | 19834.0 → 19834.0 |
| tfidf_union_33 | +0.44% | 296.0 → 296.0 | 27854.0 → 27854.0 |
| tfidf_warm | +2.23% | 71.0 → 71.0 | 19335.0 → 19335.0 |
| union_32 | +1.37% | 244.0 → 244.0 | 19834.0 → 19834.0 |
| union_33 | -1.78% | 296.0 → 296.0 | 27854.0 → 27854.0 |
| unique_long | -1.11% | 299.0 → 299.0 | 32672.0 → 32672.0 |
| unique_search | -1.37% | 72.0 → 72.0 | 19350.0 → 19350.0 |
| unique_short | -0.13% | 296.0 → 296.0 | 18548.0 → 18548.0 |

### Cold norm and heap controls

Six fixtures make twenty fresh-index-clone calls each across fourteen processes: 1,680 calls. Clone/setup is outside timing. Process/instruction/filesystem caches remain; cold means norm hints, not disks. Separate heap runs drop results before net retained measurements. The existing inline/optional heap word memo and persistent norm storage remain. Large small-term requests may add up to four temporary posting cursors; canonical allocation traffic below records that cost. Heap profiles exclude stack state/control fields and temporary frames.

| Fixture | Old median / p95 ns | New median / p95 ns |
| --- | --- | --- |
| index | 109750.500 / 125287.500 | 110937.500 / 125666.650 |
| wide | 34479.500 / 49673.250 | 34083.500 / 43648.550 |
| sparse | 423979.000 / 501633.050 | 427083.500 / 498332.700 |
| tiny | 541.000 / 1375.000 | 500.000 / 1879.150 |
| tiny4 | 709.000 / 1210.100 | 750.000 / 1210.100 |
| tiny16 | 2333.000 / 4127.100 | 2312.500 / 4083.050 |

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
| new_group_query_ns | 250729.000 / 356046.150 | 220645.500 / 327720.550 |
| post_readers_first_ns | 195125.000 / 213125.600 | 187250.000 / 200767.000 |
| post_readers_second_ns | 91875.000 / 103358.200 | 84959.000 / 102166.800 |

The private document corpus retains 72 Markdown files, 2,755,468 bytes and 1,910 chunks. All 342 authored questions (276 positive, 66 unsupported) are exposed regressions, not fresh holdouts or logged production queries. Literal BM25, prepared BM25 and coverage retain six passages, 100 candidates, 2,000 bytes/pass and 12,000 total bytes. All **1,026 complete original response JSON strings match byte-for-byte**, including scores, ordering, evidence, source text and positions. One warm-up and five repeats/question/configuration/version give **10,260 timed round trips**. These three configurations are BM25 controls; independent score oracles and actual protocol pairs cover TF-IDF. No new relevance judgment or model reply is produced.

All document timing aggregates and per-question results remain in the records. Construction uses forty plain processes on canonical data and three synthetic distributions, plus six separate profiles. Parsing/generation is outside construction timing; stats/signatures/live derived heap agree. The execution/shared-index worker checks all 167,942 entries through 128 changes/restoration, rebuilds, JSON round trips, retained clones, parallel readers and cancellation/deadline rejection. Portable exports and source hashes agree.

### Document round trips

| Case | Old median / p95 ms | New median / p95 ms |
| --- | --- | --- |
| baseline | 0.741 / 1.172 | 0.751 / 1.203 |
| prepared | 0.854 / 1.330 | 0.863 / 1.353 |
| evidence | 0.835 / 1.320 | 0.849 / 1.336 |

### Construction only

| Case | Old median / p95 ms | New median / p95 ms |
| --- | --- | --- |
| canonical | 5562.105 / 7075.003 | 5159.049 / 6214.507 |
| ascii | 15.548 / 18.178 | 14.467 / 16.329 |
| mixed | 38.577 / 44.208 | 34.296 / 38.114 |
| long_tokens | 1.687 / 2.176 | 1.555 / 1.715 |

## Limits and preservation

Measurements use one Apple M5 Max (power source observed at campaign start only; see [benchmark metadata errata](BENCHMARK-ERRATA.md)) with 64 GB, Rust 1.98.0 and Python 3.15.2. Stage-start CPU/compiler snapshots remain. Own compilers finish before timing; this does not establish continuously idle hardware. Randomized hashing, scheduling, one machine/corpus, exposed questions and finite process counts limit generalization. The separate development pilot remains; assess final per-case medians and tails independently. Host-load sampling records observations rather than continuous idleness. No universal latency, total-memory, semantic-quality, token-saving or security-audit claim is made.

Canonical knowledge remains 167,942 entries from 35 PDFs and 49,086 physical pages, SHA-256 `13168d87872a137355fd8499c90a6084f52243a2ec7456f29b5fcb2a1c7423c3`. The PDF parser is unchanged; this is not a fresh large-PDF extraction campaign. Lexical overlap and exact quotes do not prove factual support or entailment. No model, daemon or DenkMaps integration is added.

Preserve PDFs, knowledge, snapshots, all rejected/selected pilot and final measurements, slower results, raw transcripts, profiles, exact small executables and locks. Raw measurement JSON stays in the tagged GitHub repository and local archives, outside the registry source package; API guides, Markdown reports, method source, examples and required fixtures remain in that package. After verified publication archive package/proof, then delete only the named temporary Cargo/Miri targets. Default features remain empty.
