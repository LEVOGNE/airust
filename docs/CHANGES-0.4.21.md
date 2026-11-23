# AIRust 0.4.21: less preparation work for small Core queries

A raw nonempty query made only of lowercase ASCII letters and digits is already one normalized token. AIRust can borrow that word and supply its exact frequency 1 directly to the existing checked search pipeline. It avoids a normalized String, an owned frequency-map key and the temporary frequency-map node. No query cache or persistent field is added. Queries with spaces, punctuation, uppercase, Unicode or repeated words keep the existing normalization and sorted frequency-map path; preanalyzed query maps also keep their exact sorted counts. The predicate is conservative rather than a new text-analysis mode.

Candidate collection, the 0.4.20 document cursor, posting-frequency reuse, broad explanations, score arithmetic/order, norm hints, scopes, matched terms, ties and original positions remain. The sorted-map adapter yields the same word/count sequence as before. The borrowed path has the same single prepared-term checkpoint and all original candidate/scoring/winner checkpoints; less allocation does not remove work checks. Cancellation returns an error before partial results escape. This is a preparation optimization, not proof of the historical cause of every 0.4.20 timing rise.

The primary evidence is two separate small-Core three-version campaigns: original six warm 1/4/16-entry cases plus all 96 extended cases. Median changes versus fresh exact 0.4.20 range **-30.63% to -6.02%** and **-31.99% to -6.61%** across the original six. Negative means shorter observed time. The two ordinary MCP campaigns additionally observe canonical repeated TF-IDF changes of **-3.58% / -3.11%** and one-entry repeated TF-IDF changes of **+2.02% / -0.50%**. All slower cases remain. Core and MCP are different boundaries; no universal speedup, statistical significance or semantic-quality claim is made.

## Executed validation and API contract

The full matrix passes **249 default tests, 899 all-feature checks including 39 documentation examples, 39 isolated feature entries and five Python tests**. Strict Clippy/docs, Rust 1.89 provider-neutral and Rust 1.92 all-feature checks pass. API comparison with tagged 0.4.20 reports **229 pass, 31 skip**. The independent Rust 1.89 consumer covers retrieval, coverage, quotes, document positions and MCP/schema construction. The unchanged PDF backend passes 14 regressions and its 381-source-file provenance check. The refreshed dependency audit reports zero known vulnerabilities in its checked database; maintenance notices remain in the resource record.

Three new regressions check the borrowed-word guard against independent normalization, ASCII/control/noncanonical/Unicode cases and iterator exhaustion; compare borrowed, sorted-map and public dispatch against independent hand-calculated scores, metadata filters, ties and limits across native sizes 1/4/33/127/128/129/1023/1024/1025; and require equal checkpoint counts on cold state plus precise injected failures, no partial successful result, unchanged knowledge and successful recovery. The existing native full-scan oracle independently checks 1,025 records, both rankers, wide queries, scopes, limits and cold/warm states. Earlier document-cursor gaps, source filters, clones/updates/reused IDs, original spans and hostile-MCP/timeout/disconnect/drain checks remain.

**Thirty finite Tree Borrows Miri cases pass.** All three new cases run with scaled data; native tests cover the full 1,023/1,024/1,025 boundaries, and the retained 2,048-record clustered/gap case is native only. Only extra floating-point rounding injection is disabled; memory/alias checks remain. Tokio/PDF/FFI/ONNX execution is outside this interpreter scope. These are not independent security or soundness audits. Extracted-package tests, exact-commit CI, a fresh Rust 1.89 registry consumer, tag/VCS/checksum and release-asset agreement gate completion.

Engine, SearchOptions, Retriever, WorkControl, McpServer and McpLimits usage stays the same. Default features stay empty. Use WorkControl or MCP timeout_ms for cooperative limits as before. Admission, absolute deadlines, worker leases, frame/result/rate budgets, progress delivery and connection cancellation/drain remain. Startup indexing is outside tool deadlines. Read [Core performance](CORE-PERFORMANCE.md), [API](API.md), [cookbook](API-COOKBOOK.md), [MCP](MCP.md) and [security scope](MCP-SECURITY.md).

## Frozen method and development record

Baseline is exact registry 0.4.20, VCS `e30a055fab3dae32c8e59047fc01a2e32226dbed`; the older reference is exact registry 0.4.19, VCS `0c4df4d1ff1d3624fe10869b02f1e41bef3f1adf`. Every cached registry source file is compared byte-for-byte with its checksum-verified published crate archive before final timing. Candidate 0.4.21 runs the same programs and inputs. Actual worker sources, manifests, locks, executables, runtime and inputs are hashed and preserved. Read the [reproduction method](validation-0.4.21/README.md).

Development retained three unsuccessful specialization trials (a ranking helper, its non-inlined variant, and whole-search specialization by index size). Each has seven rotating fresh 0.4.19/0.4.20/candidate triples, matching signatures and preserved source/worker/sample identities. Their small-query results were mixed or slower. The selected borrowed-word trial uses the same seven-triple protocol and has shorter medians on all six original cases in that pilot. A separate three-triple MCP pilot keeps large, standard-wide, extended-wide and tiny cases plus small Core controls. Complete compared responses agree. Its source includes the final fallback-wrapper correction and new tests. All development data remain separate from every final distribution; they do not establish independent production or relevance quality.

## Primary small-Core target and retained MCP guards

Two separate small-Core campaigns each run seven rotating triples of exact 0.4.19/0.4.20/0.4.21 with the original six cases and the 96-case extension: 42 processes, 2,142,000 timed calls and 1,344 complete original matrix-response comparisons per campaign. Together: **84 processes, 4,284,000 calls and 2,688 complete comparisons**, with matching signatures/results. Original six calls construct options inside timing; the matrix reuses fixed options. Compare within each program. All fixtures, positive and negative changes and raw samples remain in the detailed primary tables below.

## Retained large and broad MCP controls

The two large-corpus campaigns each run seven rotating 0.4.19/0.4.20/0.4.21 triples on the same 167,942-entry knowledge input and ten retained queries. Each has 21 fresh servers, 1,428 schema-valid replies and 952 complete cross-version structured comparisons. Together: **42 servers, 2,856 schema-valid replies and 1,904 complete comparisons**. All compared results agree except asserted status version and version-qualified schema URNs. Each query/ranker has seven initial and fourteen repeated samples. Input candidate counts range from zero to 29,600; do not infer gains for all corpus densities or gaps. Dense, filtered, sparse and cancellation correctness also has independent native tests.

Three separate retained broad controls each use 21 servers: **63 servers, 4,284 actual schema replies and 2,856 complete structured comparisons**. They guard the 0.4.19 broad-explanation improvement rather than replace large-corpus evidence. Standard/confirmation retain the 257-record sparse/dense vocabulary fixture and ten original queries. Extended probes 32/33/64/65/128/129/192/255/256 known terms and 255 known terms plus a duplicate and unknown word. Existing normalized-term and byte limits remain. Keep all five distributions separate.

Search timing includes Rust work, serialization, stdio, Python parsing and requested progress. Initial calls, repeated calls, startup, per-query means/medians/tails, per-process totals and RSS are separate. Original standard queries are not all equally wide. Aggregate medians alone can obscure differences; retain every per-query row and raw timing observation. No sample is removed or replaced.

### Standard large-corpus campaign

| Ranker / phase | 0.4.19 median / p95 ms | 0.4.20 median / p95 ms | 0.4.21 median / p95 ms | vs 0.4.20 | vs 0.4.19 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 3.250 / 5.323 | 2.822 / 4.524 | 2.766 / 4.399 | -1.98% | -14.89% |
| tfidf / initial | 20.245 / 36.754 | 19.399 / 35.126 | 19.671 / 35.403 | +1.40% | -2.83% |
| bm25 / repeated | 2.926 / 5.180 | 2.537 / 4.245 | 2.507 / 4.145 | -1.17% | -14.29% |
| tfidf / repeated | 3.767 / 6.820 | 3.235 / 5.568 | 3.073 / 5.174 | -5.02% | -18.43% |

| Query | Repeated TF-IDF median ms: 0.4.19 / 0.4.20 / 0.4.21 | vs 0.4.20 | Initial TF-IDF mean ms: 0.4.20 → 0.4.21 |
| --- | --- | --- | --- |
| 0 | 3.707 / 3.205 / 3.081 | -3.86% | 23.970 → 24.287 |
| 1 | 3.851 / 3.262 / 2.977 | -8.72% | 23.662 → 23.676 |
| 2 | 5.912 / 5.449 / 4.952 | -9.13% | 35.396 → 35.432 |
| 3 | 0.723 / 0.760 / 0.731 | -3.88% | 1.602 → 1.716 |
| 4 | 3.857 / 3.229 / 3.086 | -4.42% | 15.854 → 15.818 |
| 5 | 5.930 / 4.966 / 4.774 | -3.86% | 27.128 → 26.840 |
| 6 | 1.436 / 1.285 / 1.320 | +2.76% | 5.434 → 5.262 |
| 7 | 6.858 / 5.236 / 4.994 | -4.63% | 27.676 → 27.194 |
| 8 | 3.878 / 3.420 / 3.271 | -4.37% | 2.985 → 2.892 |
| 9 | 0.112 / 0.117 / 0.114 | -2.48% | 0.120 → 0.119 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.19 | 173.424 | 71.951 | 5110.796 | 1724.344 |
| 0.4.20 | 163.026 | 61.326 | 5155.456 | 1724.203 |
| 0.4.21 | 162.339 | 57.592 | 5104.600 | 1724.234 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.4.21/docs/measurements/0.4.21/large-standard-summary.json).

### Confirmation large-corpus campaign

| Ranker / phase | 0.4.19 median / p95 ms | 0.4.20 median / p95 ms | 0.4.21 median / p95 ms | vs 0.4.20 | vs 0.4.19 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 3.137 / 5.164 | 2.782 / 4.658 | 2.715 / 4.291 | -2.38% | -13.44% |
| tfidf / initial | 19.000 / 34.355 | 19.357 / 32.707 | 17.718 / 32.965 | -8.47% | -6.75% |
| bm25 / repeated | 2.731 / 4.950 | 2.344 / 4.046 | 2.264 / 3.743 | -3.40% | -17.11% |
| tfidf / repeated | 3.114 / 5.777 | 2.624 / 5.267 | 2.426 / 4.150 | -7.55% | -22.10% |

| Query | Repeated TF-IDF median ms: 0.4.19 / 0.4.20 / 0.4.21 | vs 0.4.20 | Initial TF-IDF mean ms: 0.4.20 → 0.4.21 |
| --- | --- | --- | --- |
| 0 | 2.742 / 2.401 / 2.350 | -2.11% | 22.898 → 22.589 |
| 1 | 3.201 / 2.630 / 2.457 | -6.57% | 22.553 → 22.022 |
| 2 | 4.850 / 4.194 / 3.892 | -7.19% | 34.378 → 33.055 |
| 3 | 0.677 / 0.704 / 0.681 | -3.33% | 1.649 → 1.495 |
| 4 | 3.178 / 2.668 / 2.415 | -9.48% | 15.315 → 14.293 |
| 5 | 5.017 / 4.143 / 3.672 | -11.37% | 26.037 → 25.366 |
| 6 | 1.281 / 1.214 / 1.103 | -9.12% | 5.028 → 4.751 |
| 7 | 5.644 / 4.461 / 3.802 | -14.77% | 26.554 → 24.995 |
| 8 | 3.462 / 3.483 / 3.099 | -11.02% | 2.883 → 2.551 |
| 9 | 0.107 / 0.111 / 0.108 | -2.82% | 0.121 → 0.116 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.19 | 158.905 | 58.683 | 4884.529 | 1724.172 |
| 0.4.20 | 154.860 | 50.596 | 4969.868 | 1724.203 |
| 0.4.21 | 150.509 | 46.124 | 4963.607 | 1724.219 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.4.21/docs/measurements/0.4.21/large-confirmation-summary.json).

### Standard broad-query campaign

| Ranker / phase | 0.4.19 median / p95 ms | 0.4.20 median / p95 ms | 0.4.21 median / p95 ms | vs 0.4.20 | vs 0.4.19 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 0.520 / 0.605 | 0.514 / 0.598 | 0.510 / 0.599 | -0.79% | -1.77% |
| tfidf / initial | 0.472 / 0.772 | 0.466 / 0.769 | 0.460 / 0.766 | -1.42% | -2.53% |
| bm25 / repeated | 0.432 / 0.692 | 0.424 / 0.629 | 0.428 / 0.942 | +0.82% | -0.98% |
| tfidf / repeated | 0.426 / 0.760 | 0.419 / 0.753 | 0.419 / 0.771 | -0.09% | -1.85% |

| Query | Repeated TF-IDF median ms: 0.4.19 / 0.4.20 / 0.4.21 | vs 0.4.20 | Initial TF-IDF mean ms: 0.4.20 → 0.4.21 |
| --- | --- | --- | --- |
| 0 | 0.403 / 0.396 / 0.408 | +3.06% | 0.512 → 0.497 |
| 1 | 0.397 / 0.377 / 0.395 | +4.79% | 0.443 → 0.437 |
| 2 | 0.426 / 0.421 / 0.420 | -0.17% | 0.474 → 0.462 |
| 3 | 0.732 / 0.729 / 0.731 | +0.29% | 0.777 → 0.765 |
| 4 | 0.421 / 0.414 / 0.416 | +0.64% | 0.412 → 0.426 |
| 5 | 0.388 / 0.380 / 0.381 | +0.24% | 0.441 → 0.441 |
| 6 | 0.426 / 0.420 / 0.424 | +0.99% | 0.446 → 0.439 |
| 7 | 0.698 / 0.704 / 0.701 | -0.35% | 0.706 → 0.725 |
| 8 | 0.386 / 0.381 / 0.383 | +0.60% | 0.384 → 0.409 |
| 9 | 0.723 / 0.720 / 0.734 | +1.86% | 0.704 → 0.717 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.19 | 5.315 | 10.064 | 4.294 | 7.844 |
| 0.4.20 | 5.316 | 10.005 | 4.424 | 7.875 |
| 0.4.21 | 5.300 | 10.029 | 4.218 | 7.844 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.4.21/docs/measurements/0.4.21/wide-standard-summary.json).

### Confirmation broad-query campaign

| Ranker / phase | 0.4.19 median / p95 ms | 0.4.20 median / p95 ms | 0.4.21 median / p95 ms | vs 0.4.20 | vs 0.4.19 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 0.519 / 0.654 | 0.517 / 0.626 | 0.521 / 0.633 | +0.81% | +0.26% |
| tfidf / initial | 0.480 / 0.794 | 0.473 / 0.778 | 0.480 / 0.816 | +1.45% | +0.02% |
| bm25 / repeated | 0.451 / 0.859 | 0.434 / 0.740 | 0.448 / 0.937 | +3.21% | -0.83% |
| tfidf / repeated | 0.436 / 0.783 | 0.420 / 0.759 | 0.446 / 0.798 | +6.08% | +2.17% |

| Query | Repeated TF-IDF median ms: 0.4.19 / 0.4.20 / 0.4.21 | vs 0.4.20 | Initial TF-IDF mean ms: 0.4.20 → 0.4.21 |
| --- | --- | --- | --- |
| 0 | 0.426 / 0.408 / 0.411 | +0.84% | 0.514 → 0.520 |
| 1 | 0.397 / 0.394 / 0.403 | +2.24% | 0.452 → 0.458 |
| 2 | 0.427 / 0.419 / 0.446 | +6.31% | 0.467 → 0.484 |
| 3 | 0.753 / 0.748 / 0.762 | +1.98% | 0.779 → 0.820 |
| 4 | 0.435 / 0.413 / 0.424 | +2.47% | 0.438 → 0.447 |
| 5 | 0.404 / 0.387 / 0.405 | +4.51% | 0.439 → 0.446 |
| 6 | 0.441 / 0.423 / 0.437 | +3.28% | 0.454 → 0.472 |
| 7 | 0.731 / 0.703 / 0.730 | +3.89% | 0.744 → 0.745 |
| 8 | 0.392 / 0.386 / 0.394 | +2.15% | 0.410 → 0.419 |
| 9 | 0.746 / 0.722 / 0.737 | +1.99% | 0.712 → 0.720 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.19 | 5.453 | 10.252 | 4.594 | 7.891 |
| 0.4.20 | 5.397 | 10.067 | 4.278 | 7.812 |
| 0.4.21 | 5.522 | 10.455 | 4.308 | 7.844 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.4.21/docs/measurements/0.4.21/wide-confirmation-summary.json).

### Extended broad-query campaign

| Ranker / phase | 0.4.19 median / p95 ms | 0.4.20 median / p95 ms | 0.4.21 median / p95 ms | vs 0.4.20 | vs 0.4.19 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 0.585 / 0.997 | 0.599 / 1.013 | 0.569 / 0.981 | -4.90% | -2.64% |
| tfidf / initial | 0.779 / 0.965 | 0.788 / 0.980 | 0.764 / 0.997 | -2.95% | -1.85% |
| bm25 / repeated | 0.534 / 0.955 | 0.535 / 0.946 | 0.525 / 0.936 | -1.84% | -1.55% |
| tfidf / repeated | 0.713 / 1.056 | 0.709 / 1.236 | 0.705 / 1.188 | -0.55% | -1.04% |

| Query | Repeated TF-IDF median ms: 0.4.19 / 0.4.20 / 0.4.21 | vs 0.4.20 | Initial TF-IDF mean ms: 0.4.20 → 0.4.21 |
| --- | --- | --- | --- |
| 0 | 0.394 / 0.403 / 0.403 | +0.14% | 0.502 → 0.500 |
| 1 | 0.420 / 0.421 / 0.403 | -4.44% | 0.447 → 0.429 |
| 2 | 0.426 / 0.427 / 0.422 | -1.07% | 0.506 → 0.459 |
| 3 | 0.472 / 0.465 / 0.450 | -3.25% | 0.464 → 0.461 |
| 4 | 0.712 / 0.708 / 0.702 | -0.82% | 0.777 → 0.785 |
| 5 | 0.713 / 0.710 / 0.705 | -0.60% | 0.813 → 0.787 |
| 6 | 0.837 / 0.831 / 0.832 | +0.05% | 0.891 → 0.865 |
| 7 | 1.573 / 1.493 / 1.622 | +8.65% | 0.999 → 0.950 |
| 8 | 0.918 / 0.909 / 0.905 | -0.50% | 0.942 → 0.920 |
| 9 | 0.919 / 0.927 / 0.933 | +0.71% | 0.924 → 0.929 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.19 | 7.190 | 14.923 | 4.218 | 8.094 |
| 0.4.20 | 7.148 | 14.633 | 4.143 | 8.281 |
| 0.4.21 | 6.988 | 14.556 | 4.072 | 8.188 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.4.21/docs/measurements/0.4.21/wide-extended-summary.json).

### Existing limit and recovery checks

Three additional fresh servers reject 257/512 known-term requests with both rankers: twelve expected errors, no partial successful results, error progress and successful later health requests. The existing 256-unique-term assessment limit remains; these are not successful-search latency samples. Large inputs remain subject to cooperative bounds, not an OS CPU/RAM quota.

## Actual MCP: two ordinary campaigns

Each ordinary campaign uses nine cases and five alternating old/new pairs: **90 servers each, 180 total**. Canonical knowledge, NumPy and broad synthetic input request progress; 1/4/16-entry inputs run with and without progress. Each makes ten queries with both rankers, two repeated passes, four concurrent searches, matching/relations, rejected arguments and health; snapshot inputs check exact UTF-8 readbacks.

Together **13,880 actual result/error replies validate against advertised schemas**, and **6,940 complete old/new structured pairs agree by RPC ID**, excluding only checked status version and version-qualified schema URNs. Discovery, progress/no-progress, bounds, ordering, input hashes and EOF shutdown pass. Search and readback time are separate. Child peak RSS includes runtime, buffers and source/index memory, unlike allocation profiles. Retain all slower rows.

### First complete campaign

| Case | Ranker | Initial median / p95 ms: old → new | Repeated median / p95 ms: old → new | Repeated median change |
| --- | --- | --- | --- | --- |
| canonical | bm25 | 2.870 / 4.316 → 2.791 / 4.274 | 2.321 / 3.922 → 2.297 / 3.858 | -1.02% |
| canonical | tfidf | 19.402 / 33.493 → 18.408 / 34.393 | 2.674 / 4.905 → 2.578 / 4.525 | -3.58% |
| numpy | bm25 | 0.908 / 1.300 → 0.903 / 1.223 | 0.786 / 1.130 → 0.782 / 1.164 | -0.51% |
| numpy | tfidf | 1.149 / 2.710 → 1.068 / 2.669 | 0.741 / 1.238 → 0.737 / 1.234 | -0.50% |
| wide | bm25 | 0.521 / 0.622 → 0.496 / 0.599 | 0.432 / 0.751 → 0.422 / 0.611 | -2.20% |
| wide | tfidf | 0.467 / 0.786 → 0.460 / 0.780 | 0.422 / 0.747 → 0.416 / 0.731 | -1.53% |
| tiny | bm25 | 0.138 / 0.235 → 0.139 / 0.244 | 0.118 / 0.145 → 0.119 / 0.163 | +0.56% |
| tiny | tfidf | 0.136 / 0.181 → 0.135 / 0.179 | 0.119 / 0.139 → 0.121 / 0.136 | +2.02% |
| tiny4 | bm25 | 0.193 / 0.335 → 0.187 / 0.342 | 0.174 / 0.192 → 0.176 / 0.191 | +1.14% |
| tiny4 | tfidf | 0.185 / 0.260 → 0.188 / 0.262 | 0.175 / 0.206 → 0.176 / 0.203 | +0.72% |
| tiny16 | bm25 | 0.214 / 0.360 → 0.205 / 0.346 | 0.197 / 0.224 → 0.198 / 0.220 | +0.34% |
| tiny16 | tfidf | 0.211 / 0.339 → 0.214 / 0.330 | 0.198 / 0.243 → 0.198 / 0.232 | -0.13% |
| tiny-no-progress | bm25 | 0.080 / 0.183 → 0.077 / 0.173 | 0.070 / 0.111 → 0.070 / 0.105 | -0.33% |
| tiny-no-progress | tfidf | 0.078 / 0.120 → 0.078 / 0.117 | 0.071 / 0.082 → 0.070 / 0.080 | -2.37% |
| tiny4-no-progress | bm25 | 0.135 / 0.264 → 0.136 / 0.282 | 0.125 / 0.148 → 0.125 / 0.148 | +0.65% |
| tiny4-no-progress | tfidf | 0.134 / 0.194 → 0.131 / 0.198 | 0.125 / 0.175 → 0.127 / 0.166 | +1.30% |
| tiny16-no-progress | bm25 | 0.156 / 0.291 → 0.157 / 0.277 | 0.147 / 0.170 → 0.148 / 0.178 | +0.47% |
| tiny16-no-progress | tfidf | 0.156 / 0.273 → 0.162 / 0.277 | 0.150 / 0.188 → 0.150 / 0.187 | +0.31% |

| Case | Startup median ms: old → new | First TF-IDF median ms: old → new | Peak RSS median MiB: old → new |
| --- | --- | --- | --- |
| canonical | 4954.907 → 4981.313 | 23.418 → 23.612 | 1724.266 → 1724.156 |
| numpy | 137.800 → 137.752 | 1.630 → 1.662 | 57.094 → 57.141 |
| wide | 4.141 → 4.084 | 0.524 → 0.498 | 7.797 → 7.844 |
| tiny | 3.560 → 3.556 | 0.176 → 0.180 | 5.812 → 5.703 |
| tiny4 | 3.797 → 3.725 | 0.263 → 0.258 | 5.969 → 5.828 |
| tiny16 | 3.620 → 3.706 | 0.282 → 0.281 | 6.094 → 6.000 |
| tiny-no-progress | 3.550 → 3.440 | 0.120 → 0.115 | 5.797 → 5.641 |
| tiny4-no-progress | 3.572 → 3.544 | 0.195 → 0.198 | 5.891 → 5.797 |
| tiny16-no-progress | 3.783 → 3.600 | 0.227 → 0.218 | 6.031 → 6.031 |

### Second preplanned complete campaign

| Case | Ranker | Initial median / p95 ms: old → new | Repeated median / p95 ms: old → new | Repeated median change |
| --- | --- | --- | --- | --- |
| canonical | bm25 | 2.762 / 4.506 → 2.721 / 4.285 | 2.336 / 4.102 → 2.270 / 3.789 | -2.82% |
| canonical | tfidf | 18.278 / 32.962 → 18.444 / 32.524 | 2.540 / 4.630 → 2.461 / 4.111 | -3.11% |
| numpy | bm25 | 0.915 / 1.230 → 0.913 / 1.306 | 0.779 / 1.152 → 0.781 / 1.142 | +0.27% |
| numpy | tfidf | 1.114 / 2.915 → 1.069 / 2.770 | 0.734 / 1.252 → 0.721 / 1.216 | -1.77% |
| wide | bm25 | 0.500 / 0.587 → 0.506 / 0.574 | 0.418 / 0.594 → 0.420 / 0.641 | +0.49% |
| wide | tfidf | 0.448 / 0.766 → 0.455 / 0.745 | 0.409 / 0.734 → 0.412 / 0.728 | +0.76% |
| tiny | bm25 | 0.137 / 0.241 → 0.139 / 0.236 | 0.117 / 0.164 → 0.116 / 0.140 | -0.73% |
| tiny | tfidf | 0.132 / 0.187 → 0.130 / 0.173 | 0.118 / 0.136 → 0.117 / 0.132 | -0.50% |
| tiny4 | bm25 | 0.191 / 0.322 → 0.204 / 0.342 | 0.175 / 0.205 → 0.173 / 0.190 | -1.25% |
| tiny4 | tfidf | 0.185 / 0.255 → 0.186 / 0.254 | 0.174 / 0.222 → 0.173 / 0.203 | -0.40% |
| tiny16 | bm25 | 0.217 / 0.356 → 0.219 / 0.341 | 0.198 / 0.227 → 0.201 / 0.221 | +1.94% |
| tiny16 | tfidf | 0.216 / 0.336 → 0.217 / 0.330 | 0.198 / 0.234 → 0.200 / 0.233 | +1.34% |
| tiny-no-progress | bm25 | 0.080 / 0.173 → 0.082 / 0.181 | 0.069 / 0.098 → 0.073 / 0.124 | +5.23% |
| tiny-no-progress | tfidf | 0.078 / 0.120 → 0.082 / 0.113 | 0.069 / 0.077 → 0.072 / 0.096 | +4.70% |
| tiny4-no-progress | bm25 | 0.148 / 0.271 → 0.137 / 0.275 | 0.127 / 0.150 → 0.127 / 0.148 | +0.64% |
| tiny4-no-progress | tfidf | 0.139 / 0.186 → 0.132 / 0.184 | 0.127 / 0.185 → 0.128 / 0.163 | +0.87% |
| tiny16-no-progress | bm25 | 0.160 / 0.303 → 0.155 / 0.288 | 0.149 / 0.171 → 0.149 / 0.172 | -0.08% |
| tiny16-no-progress | tfidf | 0.160 / 0.292 → 0.158 / 0.275 | 0.149 / 0.192 → 0.152 / 0.193 | +2.22% |

| Case | Startup median ms: old → new | First TF-IDF median ms: old → new | Peak RSS median MiB: old → new |
| --- | --- | --- | --- |
| canonical | 4887.334 → 4875.237 | 22.744 → 23.346 | 1724.125 → 1724.203 |
| numpy | 138.709 → 138.220 | 1.598 → 1.632 | 57.078 → 57.078 |
| wide | 4.070 → 4.230 | 0.493 → 0.502 | 7.828 → 7.828 |
| tiny | 3.547 → 3.518 | 0.185 → 0.178 | 5.797 → 5.656 |
| tiny4 | 3.558 → 3.677 | 0.258 → 0.259 | 5.953 → 5.859 |
| tiny16 | 3.732 → 3.649 | 0.286 → 0.292 | 6.062 → 6.016 |
| tiny-no-progress | 3.563 → 3.566 | 0.120 → 0.113 | 5.797 → 5.656 |
| tiny4-no-progress | 3.569 → 3.777 | 0.188 → 0.184 | 5.891 → 5.781 |
| tiny16-no-progress | 3.725 → 3.599 | 0.224 → 0.234 | 6.016 → 5.953 |

## Primary target: small Core regression controls

These separate campaigns compare freshly built exact registry 0.4.19 and 0.4.20 with the candidate, using the same programs. Seven rotating triples per campaign run both the original six cases and a 96-case extension. Warm-up, full original-JSON comparison and index setup occur before timers. Each campaign has **42 processes and 2,142,000 timed Core calls**. Both together have 84 processes, 4,284,000 calls and **2,688 complete original matrix-response comparisons**, with identical results. Signature checks also cover every original-six and timed matrix sample. This is repeated warm Core time, without MCP transport or DHAT instrumentation.

Keep the two campaigns separate. Original six-case calls build SearchOptions inside the measured loop; the extension reuses fixed options. Do not compare their absolute times or merge distributions. The 96 cases combine four entry counts, both rankers and twelve fixed query/filter/limit cases. Wide, repeated, punctuated and Unicode queries retain fallback analysis. Unknown-only ASCII queries, scope rejection and zero limits are also retained; no case is removed for missing the optimized path. Historical absolute values with different programs/loads are not a reference substitute.

### First small Core campaign

#### Original six warm cases

| Fixture | 0.4.19 median ns | 0.4.20 median ns | 0.4.21 median ns | vs 0.4.20 | vs 0.4.19 |
| --- | --- | --- | --- | --- | --- |
| tiny16_bm25 | 961.583 | 1001.875 | 928.584 | -7.32% | -3.43% |
| tiny16_tfidf | 1026.500 | 1067.792 | 1003.542 | -6.02% | -2.24% |
| tiny4_bm25 | 338.875 | 344.625 | 295.792 | -14.17% | -12.71% |
| tiny4_tfidf | 360.250 | 366.833 | 314.208 | -14.35% | -12.78% |
| tiny_bm25 | 178.208 | 181.459 | 125.875 | -30.63% | -29.37% |
| tiny_tfidf | 180.459 | 183.708 | 129.292 | -29.62% | -28.35% |

Higher candidate medians versus 0.4.20: **0 of 6**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](https://github.com/LEVOGNE/airust/blob/v0.4.21/docs/measurements/0.4.21/small-reference-summary.json). No threshold-based rejection or removal of slow timing samples is used.

#### All 96 extended cases

| Fixture | 0.4.19 median ns | 0.4.20 median ns | 0.4.21 median ns | vs 0.4.20 | vs 0.4.19 |
| --- | --- | --- | --- | --- | --- |
| 16_Bm25_disjoint | 468.917 | 488.584 | 476.833 | -2.41% | +1.69% |
| 16_Bm25_limit_one | 621.500 | 633.542 | 604.375 | -4.60% | -2.76% |
| 16_Bm25_limit_zero | 2.709 | 3.000 | 3.583 | +19.43% | +32.26% |
| 16_Bm25_multi | 1555.209 | 1589.084 | 1614.250 | +1.58% | +3.80% |
| 16_Bm25_punctuation | 145.834 | 145.291 | 145.625 | +0.23% | -0.14% |
| 16_Bm25_repeated | 1157.292 | 1173.750 | 1196.458 | +1.93% | +3.38% |
| 16_Bm25_scope | 966.750 | 988.125 | 980.959 | -0.73% | +1.47% |
| 16_Bm25_single | 1109.167 | 1132.542 | 1084.917 | -4.21% | -2.19% |
| 16_Bm25_unicode | 1313.833 | 1318.334 | 1349.709 | +2.38% | +2.73% |
| 16_Bm25_unknown | 65.834 | 67.292 | 19.625 | -70.84% | -70.19% |
| 16_Bm25_unknown_added | 1139.125 | 1159.458 | 1182.083 | +1.95% | +3.77% |
| 16_Bm25_wide | 16870.708 | 17178.750 | 17374.375 | +1.14% | +2.99% |
| 16_Tfidf_disjoint | 475.083 | 503.709 | 465.041 | -7.68% | -2.11% |
| 16_Tfidf_limit_one | 708.875 | 723.458 | 691.750 | -4.38% | -2.42% |
| 16_Tfidf_limit_zero | 2.750 | 2.917 | 2.708 | -7.16% | -1.53% |
| 16_Tfidf_multi | 1636.500 | 1684.875 | 1706.791 | +1.30% | +4.30% |
| 16_Tfidf_punctuation | 143.916 | 147.834 | 143.000 | -3.27% | -0.64% |
| 16_Tfidf_repeated | 1248.875 | 1274.542 | 1295.584 | +1.65% | +3.74% |
| 16_Tfidf_scope | 1033.709 | 1072.125 | 1031.083 | -3.83% | -0.25% |
| 16_Tfidf_single | 1200.875 | 1226.584 | 1203.542 | -1.88% | +0.22% |
| 16_Tfidf_unicode | 1393.917 | 1415.333 | 1450.500 | +2.48% | +4.06% |
| 16_Tfidf_unknown | 68.375 | 68.250 | 19.750 | -71.06% | -71.12% |
| 16_Tfidf_unknown_added | 1227.042 | 1257.625 | 1278.416 | +1.65% | +4.19% |
| 16_Tfidf_wide | 16947.084 | 17052.708 | 17303.541 | +1.47% | +2.10% |
| 1_Bm25_disjoint | 110.208 | 107.666 | 55.333 | -48.61% | -49.79% |
| 1_Bm25_limit_one | 178.500 | 176.166 | 123.292 | -30.01% | -30.93% |
| 1_Bm25_limit_zero | 2.708 | 3.000 | 2.667 | -11.10% | -1.51% |
| 1_Bm25_multi | 254.667 | 261.542 | 260.584 | -0.37% | +2.32% |
| 1_Bm25_punctuation | 144.750 | 142.958 | 145.625 | +1.87% | +0.60% |
| 1_Bm25_repeated | 241.042 | 242.625 | 283.541 | +16.86% | +17.63% |
| 1_Bm25_scope | 182.625 | 182.542 | 130.833 | -28.33% | -28.36% |
| 1_Bm25_single | 219.166 | 225.583 | 158.792 | -29.61% | -27.55% |
| 1_Bm25_unicode | 426.875 | 419.458 | 433.000 | +3.23% | +1.43% |
| 1_Bm25_unknown | 68.292 | 69.500 | 19.709 | -71.64% | -71.14% |
| 1_Bm25_unknown_added | 219.291 | 218.500 | 234.708 | +7.42% | +7.03% |
| 1_Bm25_wide | 4308.750 | 4425.334 | 4352.416 | -1.65% | +1.01% |
| 1_Tfidf_disjoint | 111.250 | 108.375 | 57.333 | -47.10% | -48.46% |
| 1_Tfidf_limit_one | 185.125 | 186.708 | 131.750 | -29.44% | -28.83% |
| 1_Tfidf_limit_zero | 2.667 | 2.916 | 2.667 | -8.54% | +0.00% |
| 1_Tfidf_multi | 255.833 | 257.750 | 262.666 | +1.91% | +2.67% |
| 1_Tfidf_punctuation | 144.750 | 143.500 | 146.292 | +1.95% | +1.07% |
| 1_Tfidf_repeated | 244.250 | 240.292 | 256.125 | +6.59% | +4.86% |
| 1_Tfidf_scope | 191.167 | 192.333 | 141.208 | -26.58% | -26.13% |
| 1_Tfidf_single | 186.291 | 184.958 | 136.833 | -26.02% | -26.55% |
| 1_Tfidf_unicode | 426.292 | 430.625 | 441.583 | +2.54% | +3.59% |
| 1_Tfidf_unknown | 67.791 | 69.000 | 19.916 | -71.14% | -70.62% |
| 1_Tfidf_unknown_added | 219.792 | 221.833 | 232.541 | +4.83% | +5.80% |
| 1_Tfidf_wide | 4285.166 | 4395.667 | 4478.833 | +1.89% | +4.52% |
| 33_Bm25_disjoint | 919.709 | 974.250 | 965.750 | -0.87% | +5.01% |
| 33_Bm25_limit_one | 1176.334 | 1225.959 | 1194.958 | -2.53% | +1.58% |
| 33_Bm25_limit_zero | 2.709 | 2.958 | 2.667 | -9.84% | -1.55% |
| 33_Bm25_multi | 2426.500 | 2488.417 | 2428.041 | -2.43% | +0.06% |
| 33_Bm25_punctuation | 142.958 | 142.834 | 145.375 | +1.78% | +1.69% |
| 33_Bm25_repeated | 1732.333 | 1781.625 | 1804.541 | +1.29% | +4.17% |
| 33_Bm25_scope | 1730.125 | 1820.166 | 1783.833 | -2.00% | +3.10% |
| 33_Bm25_single | 1682.958 | 1737.209 | 1680.834 | -3.25% | -0.13% |
| 33_Bm25_unicode | 1851.167 | 1914.292 | 1906.625 | -0.40% | +3.00% |
| 33_Bm25_unknown | 67.917 | 68.958 | 19.958 | -71.06% | -70.61% |
| 33_Bm25_unknown_added | 1719.250 | 1763.583 | 1783.917 | +1.15% | +3.76% |
| 33_Bm25_wide | 26153.125 | 26773.667 | 26877.875 | +0.39% | +2.77% |
| 33_Tfidf_disjoint | 959.500 | 1003.250 | 991.750 | -1.15% | +3.36% |
| 33_Tfidf_limit_one | 1351.542 | 1356.875 | 1376.000 | +1.41% | +1.81% |
| 33_Tfidf_limit_zero | 2.666 | 2.875 | 2.667 | -7.23% | +0.04% |
| 33_Tfidf_multi | 2676.042 | 2694.750 | 2682.709 | -0.45% | +0.25% |
| 33_Tfidf_punctuation | 145.958 | 142.708 | 143.458 | +0.53% | -1.71% |
| 33_Tfidf_repeated | 1946.458 | 1948.583 | 1988.958 | +2.07% | +2.18% |
| 33_Tfidf_scope | 1868.083 | 1907.667 | 1903.584 | -0.21% | +1.90% |
| 33_Tfidf_single | 1878.458 | 1855.458 | 1878.000 | +1.21% | -0.02% |
| 33_Tfidf_unicode | 2020.250 | 2051.625 | 2085.417 | +1.65% | +3.23% |
| 33_Tfidf_unknown | 69.000 | 68.917 | 19.750 | -71.34% | -71.38% |
| 33_Tfidf_unknown_added | 1960.000 | 1932.375 | 1995.750 | +3.28% | +1.82% |
| 33_Tfidf_wide | 26800.083 | 26340.833 | 27042.375 | +2.66% | +0.90% |
| 4_Bm25_disjoint | 158.834 | 159.667 | 110.625 | -30.72% | -30.35% |
| 4_Bm25_limit_one | 237.834 | 242.875 | 191.792 | -21.03% | -19.36% |
| 4_Bm25_limit_zero | 2.667 | 2.834 | 2.666 | -5.93% | -0.04% |
| 4_Bm25_multi | 543.959 | 544.083 | 555.250 | +2.05% | +2.08% |
| 4_Bm25_punctuation | 144.708 | 142.541 | 144.708 | +1.52% | +0.00% |
| 4_Bm25_repeated | 419.541 | 431.292 | 436.875 | +1.29% | +4.13% |
| 4_Bm25_scope | 281.583 | 287.291 | 240.833 | -16.17% | -14.47% |
| 4_Bm25_single | 368.209 | 377.708 | 318.542 | -15.66% | -13.49% |
| 4_Bm25_unicode | 592.708 | 595.500 | 612.417 | +2.84% | +3.33% |
| 4_Bm25_unknown | 69.208 | 67.875 | 19.875 | -70.72% | -71.28% |
| 4_Bm25_unknown_added | 399.334 | 402.667 | 420.750 | +4.49% | +5.36% |
| 4_Bm25_wide | 7293.417 | 7326.125 | 7381.417 | +0.75% | +1.21% |
| 4_Tfidf_disjoint | 157.667 | 162.375 | 111.667 | -31.23% | -29.18% |
| 4_Tfidf_limit_one | 261.375 | 262.125 | 218.167 | -16.77% | -16.53% |
| 4_Tfidf_limit_zero | 2.708 | 2.917 | 2.708 | -7.16% | +0.00% |
| 4_Tfidf_multi | 568.500 | 569.875 | 586.583 | +2.93% | +3.18% |
| 4_Tfidf_punctuation | 145.833 | 142.584 | 146.333 | +2.63% | +0.34% |
| 4_Tfidf_repeated | 443.750 | 444.750 | 460.166 | +3.47% | +3.70% |
| 4_Tfidf_scope | 287.125 | 296.958 | 247.292 | -16.72% | -13.87% |
| 4_Tfidf_single | 390.334 | 393.084 | 346.708 | -11.80% | -11.18% |
| 4_Tfidf_unicode | 626.208 | 635.208 | 648.792 | +2.14% | +3.61% |
| 4_Tfidf_unknown | 68.208 | 68.542 | 20.041 | -70.76% | -70.62% |
| 4_Tfidf_unknown_added | 429.541 | 430.625 | 446.875 | +3.77% | +4.04% |
| 4_Tfidf_wide | 7222.125 | 7342.125 | 7375.000 | +0.45% | +2.12% |

Higher candidate medians versus 0.4.20: **45 of 96**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](https://github.com/LEVOGNE/airust/blob/v0.4.21/docs/measurements/0.4.21/small-reference-summary.json). No threshold-based rejection or removal of slow timing samples is used.

### Second preplanned small Core campaign

#### Original six warm cases

| Fixture | 0.4.19 median ns | 0.4.20 median ns | 0.4.21 median ns | vs 0.4.20 | vs 0.4.19 |
| --- | --- | --- | --- | --- | --- |
| tiny16_bm25 | 968.000 | 1001.833 | 917.583 | -8.41% | -5.21% |
| tiny16_tfidf | 1027.334 | 1059.167 | 989.209 | -6.61% | -3.71% |
| tiny4_bm25 | 334.000 | 333.917 | 289.208 | -13.39% | -13.41% |
| tiny4_tfidf | 351.500 | 355.792 | 309.834 | -12.92% | -11.85% |
| tiny_bm25 | 180.250 | 180.792 | 122.958 | -31.99% | -31.78% |
| tiny_tfidf | 178.000 | 182.375 | 126.708 | -30.52% | -28.82% |

Higher candidate medians versus 0.4.20: **0 of 6**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](https://github.com/LEVOGNE/airust/blob/v0.4.21/docs/measurements/0.4.21/small-confirmation-summary.json). No threshold-based rejection or removal of slow timing samples is used.

#### All 96 extended cases

| Fixture | 0.4.19 median ns | 0.4.20 median ns | 0.4.21 median ns | vs 0.4.20 | vs 0.4.19 |
| --- | --- | --- | --- | --- | --- |
| 16_Bm25_disjoint | 453.333 | 501.750 | 454.625 | -9.39% | +0.29% |
| 16_Bm25_limit_one | 602.375 | 649.292 | 574.333 | -11.54% | -4.66% |
| 16_Bm25_limit_zero | 2.708 | 2.875 | 2.709 | -5.77% | +0.04% |
| 16_Bm25_multi | 1550.334 | 1593.750 | 1631.917 | +2.39% | +5.26% |
| 16_Bm25_punctuation | 145.416 | 142.500 | 145.167 | +1.87% | -0.17% |
| 16_Bm25_repeated | 1142.667 | 1177.333 | 1183.958 | +0.56% | +3.61% |
| 16_Bm25_scope | 958.459 | 992.458 | 937.458 | -5.54% | -2.19% |
| 16_Bm25_single | 1096.084 | 1129.708 | 1061.125 | -6.07% | -3.19% |
| 16_Bm25_unicode | 1306.541 | 1324.500 | 1348.625 | +1.82% | +3.22% |
| 16_Bm25_unknown | 67.750 | 68.916 | 19.666 | -71.46% | -70.97% |
| 16_Bm25_unknown_added | 1130.459 | 1155.833 | 1162.625 | +0.59% | +2.85% |
| 16_Bm25_wide | 16851.292 | 17046.333 | 16998.584 | -0.28% | +0.87% |
| 16_Tfidf_disjoint | 463.917 | 497.709 | 464.500 | -6.67% | +0.13% |
| 16_Tfidf_limit_one | 709.250 | 734.417 | 683.625 | -6.92% | -3.61% |
| 16_Tfidf_limit_zero | 3.792 | 2.917 | 2.667 | -8.57% | -29.67% |
| 16_Tfidf_multi | 1619.291 | 1641.958 | 1680.416 | +2.34% | +3.77% |
| 16_Tfidf_punctuation | 144.916 | 144.584 | 143.625 | -0.66% | -0.89% |
| 16_Tfidf_repeated | 1236.250 | 1243.917 | 1299.084 | +4.43% | +5.08% |
| 16_Tfidf_scope | 1021.166 | 1064.708 | 1037.250 | -2.58% | +1.58% |
| 16_Tfidf_single | 1190.958 | 1196.750 | 1168.084 | -2.40% | -1.92% |
| 16_Tfidf_unicode | 1379.833 | 1416.417 | 1433.000 | +1.17% | +3.85% |
| 16_Tfidf_unknown | 67.125 | 68.708 | 19.750 | -71.26% | -70.58% |
| 16_Tfidf_unknown_added | 1212.083 | 1222.625 | 1278.584 | +4.58% | +5.49% |
| 16_Tfidf_wide | 16876.084 | 17030.625 | 17328.833 | +1.75% | +2.68% |
| 1_Bm25_disjoint | 111.584 | 108.500 | 56.208 | -48.20% | -49.63% |
| 1_Bm25_limit_one | 183.792 | 175.583 | 124.250 | -29.24% | -32.40% |
| 1_Bm25_limit_zero | 2.667 | 2.917 | 2.667 | -8.57% | +0.00% |
| 1_Bm25_multi | 259.834 | 258.125 | 269.500 | +4.41% | +3.72% |
| 1_Bm25_punctuation | 143.792 | 142.208 | 144.917 | +1.90% | +0.78% |
| 1_Bm25_repeated | 251.958 | 240.334 | 274.500 | +14.22% | +8.95% |
| 1_Bm25_scope | 188.750 | 181.875 | 130.583 | -28.20% | -30.82% |
| 1_Bm25_single | 221.500 | 231.416 | 144.000 | -37.77% | -34.99% |
| 1_Bm25_unicode | 429.834 | 416.834 | 425.333 | +2.04% | -1.05% |
| 1_Bm25_unknown | 68.708 | 68.833 | 19.792 | -71.25% | -71.19% |
| 1_Bm25_unknown_added | 225.584 | 220.708 | 230.125 | +4.27% | +2.01% |
| 1_Bm25_wide | 4354.167 | 4281.334 | 4367.666 | +2.02% | +0.31% |
| 1_Tfidf_disjoint | 111.334 | 107.792 | 57.958 | -46.23% | -47.94% |
| 1_Tfidf_limit_one | 191.667 | 184.041 | 131.500 | -28.55% | -31.39% |
| 1_Tfidf_limit_zero | 2.667 | 2.916 | 2.667 | -8.54% | +0.00% |
| 1_Tfidf_multi | 260.292 | 257.583 | 269.000 | +4.43% | +3.35% |
| 1_Tfidf_punctuation | 143.416 | 143.084 | 144.542 | +1.02% | +0.79% |
| 1_Tfidf_repeated | 244.500 | 238.666 | 252.208 | +5.67% | +3.15% |
| 1_Tfidf_scope | 196.125 | 191.791 | 138.917 | -27.57% | -29.17% |
| 1_Tfidf_single | 188.041 | 183.375 | 133.958 | -26.95% | -28.76% |
| 1_Tfidf_unicode | 429.917 | 424.834 | 434.209 | +2.21% | +1.00% |
| 1_Tfidf_unknown | 67.417 | 67.958 | 19.750 | -70.94% | -70.70% |
| 1_Tfidf_unknown_added | 223.958 | 221.625 | 230.209 | +3.87% | +2.79% |
| 1_Tfidf_wide | 4320.292 | 4474.250 | 4520.833 | +1.04% | +4.64% |
| 33_Bm25_disjoint | 913.750 | 1001.208 | 940.958 | -6.02% | +2.98% |
| 33_Bm25_limit_one | 1184.500 | 1270.792 | 1161.875 | -8.57% | -1.91% |
| 33_Bm25_limit_zero | 2.667 | 2.916 | 2.667 | -8.54% | +0.00% |
| 33_Bm25_multi | 2322.083 | 2454.709 | 2428.250 | -1.08% | +4.57% |
| 33_Bm25_punctuation | 142.250 | 143.250 | 143.500 | +0.17% | +0.88% |
| 33_Bm25_repeated | 1742.333 | 1775.000 | 1797.125 | +1.25% | +3.14% |
| 33_Bm25_scope | 1699.667 | 1782.334 | 1739.208 | -2.42% | +2.33% |
| 33_Bm25_single | 1685.083 | 1700.709 | 1691.875 | -0.52% | +0.40% |
| 33_Bm25_unicode | 1830.041 | 1879.375 | 1863.291 | -0.86% | +1.82% |
| 33_Bm25_unknown | 66.417 | 69.333 | 19.708 | -71.57% | -70.33% |
| 33_Bm25_unknown_added | 1685.333 | 1753.333 | 1746.333 | -0.40% | +3.62% |
| 33_Bm25_wide | 26073.792 | 26345.000 | 26465.875 | +0.46% | +1.50% |
| 33_Tfidf_disjoint | 928.542 | 980.375 | 948.000 | -3.30% | +2.10% |
| 33_Tfidf_limit_one | 1361.459 | 1370.209 | 1319.584 | -3.69% | -3.08% |
| 33_Tfidf_limit_zero | 3.416 | 2.958 | 2.667 | -9.84% | -21.93% |
| 33_Tfidf_multi | 2587.750 | 2610.583 | 2648.791 | +1.46% | +2.36% |
| 33_Tfidf_punctuation | 143.167 | 142.417 | 143.334 | +0.64% | +0.12% |
| 33_Tfidf_repeated | 1905.833 | 1901.375 | 1968.542 | +3.53% | +3.29% |
| 33_Tfidf_scope | 1812.125 | 1845.084 | 1837.958 | -0.39% | +1.43% |
| 33_Tfidf_single | 1847.208 | 1856.167 | 1823.042 | -1.78% | -1.31% |
| 33_Tfidf_unicode | 2012.250 | 2017.750 | 2072.917 | +2.73% | +3.01% |
| 33_Tfidf_unknown | 67.166 | 68.334 | 19.792 | -71.04% | -70.53% |
| 33_Tfidf_unknown_added | 1891.792 | 1882.625 | 1957.792 | +3.99% | +3.49% |
| 33_Tfidf_wide | 26580.542 | 26122.750 | 26347.750 | +0.86% | -0.88% |
| 4_Bm25_disjoint | 156.208 | 161.167 | 112.250 | -30.35% | -28.14% |
| 4_Bm25_limit_one | 240.208 | 242.750 | 189.333 | -22.00% | -21.18% |
| 4_Bm25_limit_zero | 2.666 | 2.916 | 2.667 | -8.54% | +0.04% |
| 4_Bm25_multi | 530.709 | 552.458 | 547.458 | -0.91% | +3.16% |
| 4_Bm25_punctuation | 144.666 | 142.167 | 143.584 | +1.00% | -0.75% |
| 4_Bm25_repeated | 413.792 | 437.917 | 436.375 | -0.35% | +5.46% |
| 4_Bm25_scope | 279.708 | 286.959 | 239.792 | -16.44% | -14.27% |
| 4_Bm25_single | 367.792 | 376.541 | 318.000 | -15.55% | -13.54% |
| 4_Bm25_unicode | 596.250 | 603.083 | 611.292 | +1.36% | +2.52% |
| 4_Bm25_unknown | 68.000 | 70.458 | 19.916 | -71.73% | -70.71% |
| 4_Bm25_unknown_added | 395.083 | 410.166 | 418.292 | +1.98% | +5.87% |
| 4_Bm25_wide | 7389.375 | 7257.375 | 7413.000 | +2.14% | +0.32% |
| 4_Tfidf_disjoint | 158.917 | 158.958 | 111.875 | -29.62% | -29.60% |
| 4_Tfidf_limit_one | 263.625 | 262.833 | 215.167 | -18.14% | -18.38% |
| 4_Tfidf_limit_zero | 2.667 | 2.875 | 2.667 | -7.23% | +0.00% |
| 4_Tfidf_multi | 577.917 | 560.167 | 592.083 | +5.70% | +2.45% |
| 4_Tfidf_punctuation | 145.417 | 143.833 | 147.208 | +2.35% | +1.23% |
| 4_Tfidf_repeated | 446.541 | 444.250 | 469.125 | +5.60% | +5.06% |
| 4_Tfidf_scope | 294.083 | 293.833 | 249.708 | -15.02% | -15.09% |
| 4_Tfidf_single | 394.542 | 388.750 | 345.041 | -11.24% | -12.55% |
| 4_Tfidf_unicode | 629.791 | 623.083 | 645.875 | +3.66% | +2.55% |
| 4_Tfidf_unknown | 69.000 | 70.000 | 19.709 | -71.84% | -71.44% |
| 4_Tfidf_unknown_added | 431.042 | 421.792 | 456.875 | +8.32% | +5.99% |
| 4_Tfidf_wide | 7313.791 | 7257.917 | 7394.042 | +1.88% | +1.10% |

Higher candidate medians versus 0.4.20: **41 of 96**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](https://github.com/LEVOGNE/airust/blob/v0.4.21/docs/measurements/0.4.21/small-confirmation-summary.json). No threshold-based rejection or removal of slow timing samples is used.

## Direct canonical search with explicit control modes

Fourteen processes (seven alternating pairs) retain one warmed Engine each. Ten rounds rotate plain, deadline and progress within each query/ranker pair: **8,400 timed Retriever calls**, excluding loading/readiness, stdio and profiling. Each progress request includes an observer and the same deadline/cancellation setup. Every signature agrees; **140 complete original response JSON pairs match**, and control modes also match plain output. Full readiness source text stays private. These are direct retrieval boundaries, not MCP round trips or capacity evidence.

| Control | Ranker | 0.4.20 median / p95 ms | 0.4.21 median / p95 ms | Median change |
| --- | --- | --- | --- | --- |
| plain | Bm25 | 1.344 / 2.686 | 1.341 / 2.679 | -0.23% |
| plain | Tfidf | 1.422 / 3.157 | 1.383 / 3.141 | -2.75% |
| deadline | Bm25 | 1.664 / 3.338 | 1.655 / 3.275 | -0.54% |
| deadline | Tfidf | 1.875 / 3.969 | 1.784 / 3.823 | -4.80% |
| progress | Bm25 | 1.658 / 3.384 | 1.641 / 3.236 | -1.00% |
| progress | Tfidf | 1.864 / 4.111 | 1.764 / 3.748 | -5.38% |

| Query | Candidates | Progress BM25 ms: old → new | Progress TF-IDF ms: old → new |
| --- | --- | --- | --- |
| 0 | 12365 | 1.629 → 1.622 | 1.901 → 1.778 |
| 1 | 13864 | 1.626 → 1.673 | 1.871 → 1.767 |
| 2 | 24299 | 2.921 → 2.947 | 3.478 → 3.274 |
| 3 | 596 | 0.316 → 0.315 | 0.293 → 0.290 |
| 4 | 15403 | 1.829 → 1.705 | 1.969 → 1.863 |
| 5 | 24140 | 2.847 → 2.733 | 3.245 → 3.073 |
| 6 | 3358 | 0.643 → 0.651 | 0.687 → 0.665 |
| 7 | 29600 | 2.774 → 2.747 | 3.326 → 3.163 |
| 8 | 11504 | 1.542 → 1.520 | 1.672 → 1.546 |
| 9 | 0 | 0.002 → 0.002 | 0.001 → 0.001 |

## Separate canonical three-version reference

Five rotating triples compare fresh exact 0.4.19/0.4.20/candidate servers on canonical knowledge with requested progress. Fifteen servers validate **1,020 actual schema replies**; **680 complete structured comparisons** against the candidate match by RPC ID except checked status version and version-qualified schema URNs. Schemas, source ranges and inputs agree. This control is separate from the two ordinary campaigns. It checks whether the previous large-query benefit remains visible against the older release; it does not establish the cause of any historical regression.

| Version | Ranker | Initial median / p95 ms | Repeated median / p95 ms | Peak child RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.19 | bm25 | 3.190 / 5.216 | 2.778 / 5.169 | 1724.094 |
| 0.4.19 | tfidf | 18.929 / 33.664 | 3.272 / 6.589 | 1724.094 |
| 0.4.20 | bm25 | 2.858 / 4.361 | 2.270 / 4.019 | 1724.141 |
| 0.4.20 | tfidf | 18.133 / 32.442 | 2.475 / 4.570 | 1724.141 |
| 0.4.21 | bm25 | 2.657 / 4.298 | 2.249 / 3.723 | 1724.094 |
| 0.4.21 | tfidf | 17.783 / 32.599 | 2.417 / 4.055 | 1724.094 |

| Query | 0.4.19 repeated TF-IDF ms | 0.4.20 repeated TF-IDF ms | 0.4.21 repeated TF-IDF ms |
| --- | --- | --- | --- |
| 0 | 2.734 | 2.378 | 2.354 |
| 1 | 3.151 | 2.653 | 2.469 |
| 2 | 4.936 | 4.282 | 3.939 |
| 3 | 0.714 | 0.691 | 0.675 |
| 4 | 3.394 | 2.519 | 2.447 |
| 5 | 5.054 | 3.836 | 3.622 |
| 6 | 1.302 | 1.177 | 1.124 |
| 7 | 5.605 | 4.089 | 3.757 |
| 8 | 3.792 | 2.914 | 2.750 |
| 9 | 0.105 | 0.104 | 0.110 |

## Canonical first-pass heap traffic

Two instrumented processes index all 167,942 records before profiling the same ten initial TF-IDF queries, then drop results before net deltas. The existing direct candidate collection remains; the previous large-query posting cursors and norm memo/cache still exist. Heap traffic, net persistent hints, stack state and whole-process RSS are different costs. Instrumented durations are excluded from latency comparisons.

| Metric | 0.4.20 | 0.4.21 |
| --- | --- | --- |
| Retained hint bytes | 2228312 | 2228312 |
| Retained hint blocks | 2 | 2 |
| Total query allocated bytes | 7213987 | 7213183 |
| Total query allocation blocks | 415 | 409 |

## Development trials and host-load limits

The three rejected specialization trials and selected borrowed-query trial are retained separately. All seven rotating triples per trial keep source/binary hashes, exact workers and every original sample. The selected full MCP pilot also remains separate. Selection during development never replaces samples in the frozen final campaigns. These are tuned development observations, not fresh production or semantic holdouts.

| Development variant | Original six median range vs0.4.20 | Final selection |
| --- | --- | --- |
| ranking helper, default inlining | -3.77% to +0.89% | rejected |
| ranking helper, inline-never | -4.40% to +1.74% | rejected |
| whole search, index-size specialization | -2.67% to +6.56% | rejected |
| selected borrowed query | -31.57% to -6.94% | selected |

Two-second monitoring retains **493 snapshots**, including **0 with other rustc processes**, maximum 0. All managed compilers finish before timing. Other workloads are not stopped, and snapshots do not prove continuous idleness or identify a timing cause. CPU counts include the active benchmark workers.

| Stage start | Other rustc count | Other rustc CPU total | Processes over 50% CPU |
| --- | --- | --- | --- |
| small-reference | 0 | 0.000 | 1 |
| large-standard | 0 | 0.000 | 2 |
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
| limits | 0 | 0.000 | 2 |

All observed slower cases remain in their own distributions; no sample is replaced because of load. One machine, randomized hashing, exposed questions, tuned fixtures and finite trials limit generalization.

## Retained Core controls

These controls retain positive and negative timing changes. Single known-term requests retain the direct candidate path and exactly two retain ordered merging. Wider requests keep the heap/set strategies; large frequency cursors remain available afterward. Large sorted candidate sets can use the bounded document cursor after 128 candidates; earlier shared explanation intersection remains. Read every row independently, including slower cases. Seven alternating pairs give fourteen plain processes per family. Norms are initialized for warm timing; two separate DHAT processes measure allocations. Instrumented durations are excluded from latency comparisons. Every result signature agrees and temporary-call retained deltas are zero.

### Small warm Core: 84,000 calls on 1/4/16 entries

| Fixture | Median change | Blocks/call: old → new | Allocated bytes/call: old → new |
| --- | --- | --- | --- |
| tiny16_bm25 | -9.46% | 29.0 → 26.0 | 2536.0 → 2156.0 |
| tiny16_tfidf | -8.33% | 29.0 → 26.0 | 2536.0 → 2156.0 |
| tiny4_bm25 | -13.14% | 15.0 → 12.0 | 1204.0 → 824.0 |
| tiny4_tfidf | -14.16% | 15.0 → 12.0 | 1204.0 → 824.0 |
| tiny_bm25 | -32.21% | 9.0 → 6.0 | 778.0 → 398.0 |
| tiny_tfidf | -29.28% | 9.0 → 6.0 | 778.0 → 398.0 |

### All 25 warm controls: 282,800 calls

| Fixture | Median change | Blocks/call: old → new | Allocated bytes/call: old → new |
| --- | --- | --- | --- |
| coverage_repeated | +2.07% | 1047.0 → 1047.0 | 92942.0 → 92942.0 |
| coverage_small | +1.48% | 784.0 → 784.0 | 78046.0 → 78046.0 |
| excerpt_ascii | -0.15% | 762.0 → 762.0 | 252438.0 → 252438.0 |
| excerpt_mixed | +0.25% | 569.0 → 569.0 | 417113.0 → 417113.0 |
| excerpt_unicode | +0.85% | 197.0 → 197.0 | 72254.0 → 72254.0 |
| long_repeated | +0.48% | 4.0 → 4.0 | 18798.0 → 18798.0 |
| mixed_unicode | +3.23% | 7.0 → 7.0 | 3155.0 → 3155.0 |
| punctuation | +4.90% | 1.0 → 1.0 | 768.0 → 768.0 |
| relevant_retrieval | +1.20% | 812.0 → 812.0 | 93562.0 → 93562.0 |
| repeated_search | -4.15% | 35.0 → 35.0 | 19432.0 → 19432.0 |
| repeated_short | -1.17% | 4.0 → 4.0 | 1656.0 → 1656.0 |
| sparse_bm25_128 | -1.05% | 359.0 → 359.0 | 29074.0 → 29074.0 |
| sparse_bm25_33 | +0.13% | 241.0 → 241.0 | 16309.0 → 16309.0 |
| sparse_bm25_512 | +0.85% | 1337.0 → 1337.0 | 119124.0 → 119124.0 |
| sparse_tfidf_128 | -0.87% | 474.0 → 474.0 | 35140.0 → 35140.0 |
| sparse_tfidf_33 | -0.45% | 241.0 → 241.0 | 16309.0 → 16309.0 |
| sparse_tfidf_512 | +1.04% | 1337.0 → 1337.0 | 119124.0 → 119124.0 |
| tfidf_union_32 | -0.35% | 244.0 → 244.0 | 19834.0 → 19834.0 |
| tfidf_union_33 | -0.39% | 296.0 → 296.0 | 27854.0 → 27854.0 |
| tfidf_warm | -0.53% | 71.0 → 71.0 | 19335.0 → 19335.0 |
| union_32 | +1.93% | 244.0 → 244.0 | 19834.0 → 19834.0 |
| union_33 | +0.14% | 296.0 → 296.0 | 27854.0 → 27854.0 |
| unique_long | +1.70% | 299.0 → 299.0 | 32672.0 → 32672.0 |
| unique_search | +1.74% | 72.0 → 72.0 | 19350.0 → 19350.0 |
| unique_short | +1.87% | 296.0 → 296.0 | 18548.0 → 18548.0 |

### Cold norm and heap controls

Six fixtures make twenty fresh-index-clone calls each across fourteen processes: 1,680 calls. Clone/setup is outside timing. Process/instruction/filesystem caches remain; cold means norm hints, not disks. Separate heap runs drop results before net retained measurements. The existing inline/optional heap word memo and persistent norm storage remain. Large small-term requests may add up to four temporary posting cursors; canonical allocation traffic below records that cost. Heap profiles exclude stack state/control fields and temporary frames.

| Fixture | Old median / p95 ns | New median / p95 ns |
| --- | --- | --- |
| index | 114709.000 / 148100.650 | 112833.500 / 131906.250 |
| wide | 42458.500 / 66025.000 | 33520.500 / 59426.450 |
| sparse | 447166.500 / 557288.950 | 438708.500 / 501927.400 |
| tiny | 750.000 / 2849.700 | 625.000 / 1798.200 |
| tiny4 | 875.000 / 2133.300 | 791.500 / 1333.000 |
| tiny16 | 2750.000 / 4383.300 | 2520.500 / 3676.450 |

| Cold fixture | Allocated bytes: old → new | Retained hint bytes: old → new |
| --- | --- | --- |
| index | 92356 → 91976 | 34904 → 34904 |
| wide | 19165 → 19165 | 4440 → 4440 |
| sparse | 140704 → 140704 | 8792 → 8792 |
| tiny | 934 → 554 | 156 → 156 |
| tiny4 | 1504 → 1124 | 224 → 224 |
| tiny16 | 3668 → 3288 | 632 → 632 |

Canonical first-pass heap records, all samples, blocks and cold tails remain in public JSON. Retained hints, heap traffic and whole-process RAM are different measures.

## Finite mixed control, documents and construction

The 6,000-entry mixed control retains four continuous warm readers while 100 initially uncached-group driver queries finish, then makes two quiescent probes. Five alternating pairs give 1,000 driver calls and twenty quiescent probes. Warm counts vary with the stop condition and new-group calls can become warm. This finite control is not fixed-load throughput, capacity or fairness evidence.

| Mixed work, ns | Old median / p95 | New median / p95 |
| --- | --- | --- |
| new_group_query_ns | 181458.000 / 269745.200 | 170521.000 / 302131.600 |
| post_readers_first_ns | 162375.000 / 188116.800 | 187041.000 / 198300.400 |
| post_readers_second_ns | 87000.000 / 87925.000 | 82375.000 / 82783.400 |

The private document corpus retains 72 Markdown files, 2,755,468 bytes and 1,910 chunks. All 342 authored questions (276 positive, 66 unsupported) are exposed regressions, not fresh holdouts or logged production queries. Literal BM25, prepared BM25 and coverage retain six passages, 100 candidates, 2,000 bytes/pass and 12,000 total bytes. All **1,026 complete original response JSON strings match byte-for-byte**, including scores, ordering, evidence, source text and positions. One warm-up and five repeats/question/configuration/version give **10,260 timed round trips**. These three configurations are BM25 controls; independent score oracles and actual protocol pairs cover TF-IDF. No new relevance judgment or model reply is produced.

All document timing aggregates and per-question results remain in the records. Construction uses forty plain processes on canonical data and three synthetic distributions, plus six separate profiles. Parsing/generation is outside construction timing; stats/signatures/live derived heap agree. The execution/shared-index worker checks all 167,942 entries through 128 changes/restoration, rebuilds, JSON round trips, retained clones, parallel readers and cancellation/deadline rejection. Portable exports and source hashes agree.

### Document round trips

| Case | Old median / p95 ms | New median / p95 ms |
| --- | --- | --- |
| baseline | 0.789 / 1.241 | 0.756 / 1.195 |
| prepared | 0.897 / 1.342 | 0.880 / 1.295 |
| evidence | 0.877 / 1.318 | 0.870 / 1.312 |

### Construction only

| Case | Old median / p95 ms | New median / p95 ms |
| --- | --- | --- |
| canonical | 5251.826 / 5284.263 | 5276.910 / 5400.554 |
| ascii | 15.195 / 16.058 | 15.244 / 15.541 |
| mixed | 34.736 / 36.081 | 35.149 / 35.459 |
| long_tokens | 1.643 / 1.666 | 1.638 / 1.667 |

## Limits and preservation

Measurements use one AC-powered Apple M5 Max with 64 GB, Rust 1.98.0 and Python 3.15.2. Stage-start CPU/compiler snapshots remain. Own compilers finish before timing; this does not establish continuously idle hardware. Randomized hashing, scheduling, one machine/corpus, exposed questions and finite process counts limit generalization. The separate development pilot remains; assess final per-case medians and tails independently. Host-load sampling records observations rather than continuous idleness. No universal latency, total-memory, semantic-quality, token-saving or security-audit claim is made.

Canonical knowledge remains 167,942 entries from 35 PDFs and 49,086 physical pages, SHA-256 `13168d87872a137355fd8499c90a6084f52243a2ec7456f29b5fcb2a1c7423c3`. The PDF parser is unchanged; this is not a fresh large-PDF extraction campaign. Lexical overlap and exact quotes do not prove factual support or entailment. No model, daemon or DenkMaps integration is added.

Preserve PDFs, knowledge, snapshots, all rejected/selected pilot and final measurements, slower results, raw transcripts, profiles, exact small executables and locks. Raw measurement JSON stays in the tagged GitHub repository and local archives, outside the registry source package; API guides, Markdown reports, method source, examples and required fixtures remain in that package. After verified publication archive package/proof, then delete only the named temporary Cargo/Miri targets. Default features remain empty.
