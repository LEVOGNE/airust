# AIRust 0.4.20: ordered document lookup for large searches

Large searches already collect sorted, unique candidate IDs. Previously each candidate performed a fresh tree lookup of its document. After the first 128 candidates, a request with at least 1,024 candidates now uses a private DocumentCursor over the same immutable document map. It starts at the current ID, compares exact IDs (with a safe live-Arc identity shortcut), and advances through nearby documents. After eight lower keys it checks cancellation/deadlines and seeks directly to the requested ID. Small candidate sets retain direct lookup. This removes repeated root-to-leaf searches where candidates are nearby, without a new cache, persistent field, cursor heap allocation, dependency, API or format migration.

Original per-candidate checkpoints, posting visitation/progress, candidate order, context/source filters, score arithmetic/order, norm hints, winner explanations, ties and source positions remain. Gap scans add a checkpoint before each fallback seek; no existing check is removed. A failed cooperative check rejects the operation instead of exposing partial results. The single/two-term collection paths, frequency cursors, broad dictionary intersection and MCP transport controls remain. Query-local cursor stack state is excluded from heap profiles; this is not a total-memory bound.

Primary evidence is two separate preplanned large-corpus three-version campaigns below. The two ordinary campaigns additionally observe canonical repeated MCP TF-IDF median changes of **-14.84% / -19.52%** versus fresh exact registry 0.4.19. One-entry repeated MCP TF-IDF changes are **+1.21% / -1.85%**. Negative means shorter observed time. All slower cases remain. One host, exposed questions and finite trials do not establish a universal speedup, statistical significance or semantic-quality improvement.

## Executed validation and API contract

The full matrix passes **246 default tests, 896 all-feature checks including 39 documentation examples, 39 isolated feature entries and five Python tests**. Strict Clippy/docs, Rust 1.89 provider-neutral and Rust 1.92 all-feature checks pass. API comparison with tagged 0.4.19 reports **229 pass, 31 skip**. The independent Rust 1.89 consumer covers retrieval, coverage, quotes, document positions and MCP/schema construction. The unchanged PDF backend passes 14 regressions and its 381-source-file provenance check. The refreshed dependency audit reports zero known vulnerabilities in its checked database; maintenance notices remain in the resource record.

Three new native regressions check ordered cursor lookup against independent document lookup, shared/separately allocated equal IDs, control/Unicode IDs, missing IDs, long gaps and iterator exhaustion; interruption before a gap seek and recovery afterward; and a 2,048-record two-cluster fixture with a large middle gap, hand-calculated BM25/TF-IDF scores, ties, metadata/source filters, empty scope, limits and partial-result rejection. The existing independent full-scan oracle covers 1,025 native records, both rankers, query widths, filters and cold/warm states. Mutation, reused IDs, retained clones, source spans, hostile MCP input, cancellation, absolute deadlines, stalled readers, admission, duplicate tokens, disconnect and drain remain tested.

**Twenty-seven finite Tree Borrows Miri cases pass.** Both new direct cursor cases run under Miri with scaled inputs; the 2,048-record score/filter regression is native only. Earlier 25 finite cases remain, including scaled dictionary/frequency-cursor checks and the independent full-scan oracle. Only extra floating-point rounding injection is disabled; memory/alias checks remain. Tokio/PDF/FFI/ONNX execution is outside this interpreter scope. These checks are not an independent security or soundness audit. Extracted-package tests, exact-commit CI and a fresh registry consumer gate publication; package checksum, VCS, annotated tag and release asset must agree.

Engine, SearchOptions, Retriever, WorkControl, McpServer and McpLimits usage stays the same. Default features stay empty. Use WorkControl or MCP timeout_ms for cooperative limits as before. Admission, absolute deadlines, worker leases, frame/result/rate budgets, progress delivery and connection cancellation/drain remain. Startup indexing is outside tool deadlines. Read [Core performance](CORE-PERFORMANCE.md), [API](API.md), [cookbook](API-COOKBOOK.md), [MCP](MCP.md) and [security scope](MCP-SECURITY.md).

## Frozen method and development record

Baseline is exact registry 0.4.19, VCS `0c4df4d1ff1d3624fe10869b02f1e41bef3f1adf`; the older reference is exact registry 0.4.18, VCS `fd5a0e182ece603e135feb609351cc3559bca4bf`. Every cached registry source file is compared byte-for-byte with its checksum-verified published crate archive before final timing. Candidate 0.4.20 runs the same programs and inputs. Actual worker sources, manifests, locks, executables, runtime and inputs are hashed and preserved. Read the [reproduction method](validation-0.4.20/README.md).

The development pilot compares three rotating fresh 0.4.18/0.4.19/0.4.20 triples on large, standard-wide, extended-wide and tiny MCP plus six small Core cases. Complete compared responses agree. Large repeated medians are shorter in that pilot, while small/wide cases are mixed. Pilot timing never enters final distributions. Only an additional native regression and formatting changed after the pilot; the measured runtime algorithm is unchanged. There is no new out-of-bound failed pilot in 0.4.20; earlier release records remain in their original archives.

## Primary target and retained broad-query guards

The two large-corpus campaigns each run seven rotating 0.4.18/0.4.19/0.4.20 triples on the same 167,942-entry knowledge input and ten retained queries. Each has 21 fresh servers, 1,428 schema-valid replies and 952 complete cross-version structured comparisons. Together: **42 servers, 2,856 schema-valid replies and 1,904 complete comparisons**. All compared results agree except asserted status version and version-qualified schema URNs. Each query/ranker has seven initial and fourteen repeated samples. Input candidate counts range from zero to 29,600; do not infer gains for all corpus densities or gaps. Dense, filtered, sparse and cancellation correctness also has independent native tests.

Three separate retained broad controls each use 21 servers: **63 servers, 4,284 actual schema replies and 2,856 complete structured comparisons**. They guard the 0.4.19 broad-explanation improvement rather than replace large-corpus evidence. Standard/confirmation retain the 257-record sparse/dense vocabulary fixture and ten original queries. Extended probes 32/33/64/65/128/129/192/255/256 known terms and 255 known terms plus a duplicate and unknown word. Existing normalized-term and byte limits remain. Keep all five distributions separate.

Search timing includes Rust work, serialization, stdio, Python parsing and requested progress. Initial calls, repeated calls, startup, per-query means/medians/tails, per-process totals and RSS are separate. Original standard queries are not all equally wide. Aggregate medians alone can obscure differences; retain every per-query row and raw timing observation. No sample is removed or replaced.

### Standard large-corpus campaign

| Ranker / phase | 0.4.18 median / p95 ms | 0.4.19 median / p95 ms | 0.4.20 median / p95 ms | vs 0.4.19 | vs 0.4.18 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 3.124 / 5.521 | 3.095 / 5.303 | 2.652 / 4.141 | -14.31% | -15.09% |
| tfidf / initial | 18.449 / 32.711 | 19.293 / 31.742 | 16.931 / 30.088 | -12.25% | -8.23% |
| bm25 / repeated | 2.862 / 5.115 | 2.875 / 5.129 | 2.292 / 4.029 | -20.26% | -19.92% |
| tfidf / repeated | 3.335 / 6.293 | 3.344 / 6.597 | 2.650 / 5.073 | -20.76% | -20.53% |

| Query | Repeated TF-IDF median ms: 0.4.18 / 0.4.19 / 0.4.20 | vs 0.4.19 | Initial TF-IDF mean ms: 0.4.19 → 0.4.20 |
| --- | --- | --- | --- |
| 0 | 2.830 / 2.918 / 2.449 | -16.09% | 21.077 → 20.199 |
| 1 | 3.386 / 3.324 / 2.663 | -19.89% | 21.842 → 20.520 |
| 2 | 5.298 / 5.368 / 4.661 | -13.18% | 32.466 → 31.074 |
| 3 | 0.712 / 0.725 / 0.701 | -3.31% | 1.586 → 1.511 |
| 4 | 3.585 / 3.435 / 2.581 | -24.86% | 15.750 → 14.169 |
| 5 | 5.492 / 5.092 / 4.130 | -18.89% | 26.493 → 24.551 |
| 6 | 1.321 / 1.239 / 1.179 | -4.87% | 5.208 → 4.663 |
| 7 | 5.793 / 5.939 / 4.208 | -29.15% | 27.495 → 24.846 |
| 8 | 3.483 / 3.766 / 3.301 | -12.33% | 3.002 → 2.639 |
| 9 | 0.113 / 0.118 / 0.105 | -11.12% | 0.125 → 0.117 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.18 | 157.099 | 65.502 | 4935.663 | 1724.125 |
| 0.4.19 | 151.625 | 63.553 | 4934.636 | 1724.266 |
| 0.4.20 | 142.094 | 51.218 | 4967.143 | 1724.219 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](measurements/0.4.20/large-standard-summary.json).

### Confirmation large-corpus campaign

| Ranker / phase | 0.4.18 median / p95 ms | 0.4.19 median / p95 ms | 0.4.20 median / p95 ms | vs 0.4.19 | vs 0.4.18 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 3.045 / 5.118 | 3.040 / 5.081 | 2.552 / 3.941 | -16.04% | -16.17% |
| tfidf / initial | 17.515 / 31.638 | 17.429 / 30.764 | 16.597 / 29.386 | -4.77% | -5.24% |
| bm25 / repeated | 2.691 / 4.843 | 2.664 / 4.771 | 2.113 / 3.590 | -20.66% | -21.48% |
| tfidf / repeated | 2.915 / 5.259 | 2.877 / 5.119 | 2.333 / 3.859 | -18.92% | -19.97% |

| Query | Repeated TF-IDF median ms: 0.4.18 / 0.4.19 / 0.4.20 | vs 0.4.19 | Initial TF-IDF mean ms: 0.4.19 → 0.4.20 |
| --- | --- | --- | --- |
| 0 | 2.638 / 2.617 / 2.282 | -12.83% | 20.273 → 19.988 |
| 1 | 3.051 / 3.007 / 2.351 | -21.82% | 21.221 → 20.565 |
| 2 | 4.826 / 4.571 / 3.750 | -17.96% | 31.325 → 29.436 |
| 3 | 0.671 / 0.644 / 0.653 | +1.46% | 1.482 → 1.465 |
| 4 | 2.931 / 2.957 / 2.347 | -20.62% | 14.251 → 13.346 |
| 5 | 4.613 / 4.583 / 3.536 | -22.85% | 24.872 → 23.051 |
| 6 | 1.170 / 1.134 / 1.079 | -4.88% | 4.691 → 4.422 |
| 7 | 5.233 / 5.105 / 3.616 | -29.18% | 25.753 → 23.572 |
| 8 | 3.118 / 3.225 / 2.684 | -16.76% | 2.873 → 2.465 |
| 9 | 0.111 / 0.110 / 0.108 | -1.80% | 0.114 → 0.137 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.18 | 150.115 | 58.724 | 4789.372 | 1724.172 |
| 0.4.19 | 145.670 | 55.790 | 4791.158 | 1724.172 |
| 0.4.20 | 138.631 | 45.571 | 4805.520 | 1724.219 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](measurements/0.4.20/large-confirmation-summary.json).

### Standard broad-query campaign

| Ranker / phase | 0.4.18 median / p95 ms | 0.4.19 median / p95 ms | 0.4.20 median / p95 ms | vs 0.4.19 | vs 0.4.18 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 0.527 / 0.655 | 0.500 / 0.667 | 0.511 / 0.629 | +2.23% | -3.10% |
| tfidf / initial | 0.463 / 0.888 | 0.465 / 0.804 | 0.471 / 0.791 | +1.20% | +1.82% |
| bm25 / repeated | 0.432 / 0.802 | 0.431 / 1.158 | 0.433 / 0.870 | +0.29% | +0.05% |
| tfidf / repeated | 0.424 / 0.851 | 0.430 / 0.788 | 0.423 / 0.807 | -1.82% | -0.46% |

| Query | Repeated TF-IDF median ms: 0.4.18 / 0.4.19 / 0.4.20 | vs 0.4.19 | Initial TF-IDF mean ms: 0.4.19 → 0.4.20 |
| --- | --- | --- | --- |
| 0 | 0.396 / 0.398 / 0.404 | +1.58% | 0.516 → 0.515 |
| 1 | 0.382 / 0.396 / 0.378 | -4.60% | 0.450 → 0.471 |
| 2 | 0.438 / 0.433 / 0.427 | -1.41% | 0.469 → 0.485 |
| 3 | 0.799 / 0.748 / 0.755 | +0.85% | 0.803 → 0.784 |
| 4 | 0.414 / 0.414 / 0.414 | -0.05% | 0.419 → 0.423 |
| 5 | 0.379 / 0.384 / 0.388 | +1.13% | 0.438 → 0.444 |
| 6 | 0.430 / 0.429 / 0.421 | -2.00% | 0.449 → 0.442 |
| 7 | 0.780 / 0.701 / 0.723 | +3.11% | 0.759 → 0.732 |
| 8 | 0.381 / 0.385 / 0.390 | +1.20% | 0.401 → 0.406 |
| 9 | 0.806 / 0.771 / 0.748 | -2.96% | 0.745 → 0.701 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.18 | 5.577 | 10.347 | 4.030 | 7.828 |
| 0.4.19 | 5.302 | 10.092 | 4.337 | 7.844 |
| 0.4.20 | 5.320 | 10.111 | 4.389 | 7.781 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](measurements/0.4.20/wide-standard-summary.json).

### Confirmation broad-query campaign

| Ranker / phase | 0.4.18 median / p95 ms | 0.4.19 median / p95 ms | 0.4.20 median / p95 ms | vs 0.4.19 | vs 0.4.18 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 0.538 / 0.670 | 0.501 / 0.584 | 0.510 / 0.615 | +1.83% | -5.28% |
| tfidf / initial | 0.480 / 0.874 | 0.456 / 0.773 | 0.470 / 0.785 | +3.09% | -2.02% |
| bm25 / repeated | 0.437 / 0.829 | 0.426 / 0.656 | 0.429 / 0.667 | +0.67% | -1.67% |
| tfidf / repeated | 0.432 / 0.825 | 0.414 / 0.733 | 0.424 / 0.764 | +2.51% | -1.77% |

| Query | Repeated TF-IDF median ms: 0.4.18 / 0.4.19 / 0.4.20 | vs 0.4.19 | Initial TF-IDF mean ms: 0.4.19 → 0.4.20 |
| --- | --- | --- | --- |
| 0 | 0.397 / 0.398 / 0.395 | -0.80% | 0.493 → 0.540 |
| 1 | 0.383 / 0.374 / 0.381 | +1.82% | 0.427 → 0.455 |
| 2 | 0.432 / 0.419 / 0.425 | +1.54% | 0.465 → 0.470 |
| 3 | 0.798 / 0.719 / 0.735 | +2.19% | 0.781 → 0.780 |
| 4 | 0.418 / 0.417 / 0.427 | +2.23% | 0.418 → 0.415 |
| 5 | 0.385 / 0.382 / 0.386 | +1.23% | 0.440 → 0.436 |
| 6 | 0.435 / 0.420 / 0.425 | +1.29% | 0.427 → 0.451 |
| 7 | 0.772 / 0.692 / 0.698 | +0.84% | 0.723 → 0.723 |
| 8 | 0.386 / 0.381 / 0.390 | +2.30% | 0.391 → 0.403 |
| 9 | 0.795 / 0.704 / 0.714 | +1.35% | 0.710 → 0.706 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.18 | 5.532 | 10.326 | 3.912 | 7.812 |
| 0.4.19 | 5.270 | 9.829 | 4.033 | 7.859 |
| 0.4.20 | 5.235 | 9.953 | 3.953 | 7.891 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](measurements/0.4.20/wide-confirmation-summary.json).

### Extended broad-query campaign

| Ranker / phase | 0.4.18 median / p95 ms | 0.4.19 median / p95 ms | 0.4.20 median / p95 ms | vs 0.4.19 | vs 0.4.18 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 0.633 / 1.233 | 0.570 / 0.979 | 0.561 / 0.963 | -1.75% | -11.50% |
| tfidf / initial | 0.856 / 1.199 | 0.759 / 0.952 | 0.770 / 0.937 | +1.44% | -10.00% |
| bm25 / repeated | 0.603 / 1.193 | 0.532 / 0.994 | 0.523 / 0.954 | -1.78% | -13.33% |
| tfidf / repeated | 0.775 / 1.307 | 0.705 / 1.144 | 0.694 / 1.071 | -1.58% | -10.54% |

| Query | Repeated TF-IDF median ms: 0.4.18 / 0.4.19 / 0.4.20 | vs 0.4.19 | Initial TF-IDF mean ms: 0.4.19 → 0.4.20 |
| --- | --- | --- | --- |
| 0 | 0.399 / 0.400 / 0.404 | +1.09% | 0.487 → 0.485 |
| 1 | 0.413 / 0.408 / 0.412 | +0.85% | 0.433 → 0.434 |
| 2 | 0.435 / 0.420 / 0.420 | +0.02% | 0.469 → 0.469 |
| 3 | 0.477 / 0.452 / 0.455 | +0.76% | 0.446 → 0.458 |
| 4 | 0.771 / 0.705 / 0.689 | -2.29% | 0.758 → 0.771 |
| 5 | 0.780 / 0.705 / 0.698 | -1.00% | 0.773 → 0.772 |
| 6 | 1.010 / 0.850 / 0.820 | -3.52% | 0.869 → 0.865 |
| 7 | 1.744 / 1.523 / 1.511 | -0.78% | 0.944 → 0.928 |
| 8 | 1.146 / 0.916 / 0.925 | +0.96% | 0.938 → 0.916 |
| 9 | 1.186 / 0.921 / 0.929 | +0.95% | 0.923 → 0.908 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.18 | 8.114 | 16.694 | 4.008 | 8.109 |
| 0.4.19 | 6.981 | 14.727 | 4.046 | 8.172 |
| 0.4.20 | 6.905 | 14.467 | 3.874 | 8.250 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](measurements/0.4.20/wide-extended-summary.json).

### Existing limit and recovery checks

Three additional fresh servers reject 257/512 known-term requests with both rankers: twelve expected errors, no partial successful results, error progress and successful later health requests. The existing 256-unique-term assessment limit remains; these are not successful-search latency samples. Large inputs remain subject to cooperative bounds, not an OS CPU/RAM quota.

## Actual MCP: two ordinary campaigns

Each ordinary campaign uses nine cases and five alternating old/new pairs: **90 servers each, 180 total**. Canonical knowledge, NumPy and broad synthetic input request progress; 1/4/16-entry inputs run with and without progress. Each makes ten queries with both rankers, two repeated passes, four concurrent searches, matching/relations, rejected arguments and health; snapshot inputs check exact UTF-8 readbacks.

Together **13,880 actual result/error replies validate against advertised schemas**, and **6,940 complete old/new structured pairs agree by RPC ID**, excluding only checked status version and version-qualified schema URNs. Discovery, progress/no-progress, bounds, ordering, input hashes and EOF shutdown pass. Search and readback time are separate. Child peak RSS includes runtime, buffers and source/index memory, unlike allocation profiles. Retain all slower rows.

### First complete campaign

| Case | Ranker | Initial median / p95 ms: old → new | Repeated median / p95 ms: old → new | Repeated median change |
| --- | --- | --- | --- | --- |
| canonical | bm25 | 3.031 / 5.008 → 2.700 / 4.268 | 2.584 / 4.725 → 2.218 / 3.837 | -14.17% |
| canonical | tfidf | 17.862 / 31.571 → 18.161 / 30.863 | 2.827 / 5.010 → 2.407 / 4.433 | -14.84% |
| numpy | bm25 | 0.934 / 1.240 → 0.912 / 1.250 | 0.776 / 1.111 → 0.775 / 1.113 | -0.14% |
| numpy | tfidf | 1.047 / 2.812 → 1.040 / 2.780 | 0.731 / 1.171 → 0.736 / 1.224 | +0.68% |
| wide | bm25 | 0.508 / 0.581 → 0.501 / 0.578 | 0.423 / 0.628 → 0.415 / 0.671 | -2.04% |
| wide | tfidf | 0.451 / 0.756 → 0.446 / 0.750 | 0.414 / 0.738 → 0.412 / 0.723 | -0.46% |
| tiny | bm25 | 0.132 / 0.229 → 0.140 / 0.237 | 0.118 / 0.178 → 0.120 / 0.169 | +2.14% |
| tiny | tfidf | 0.133 / 0.177 → 0.135 / 0.184 | 0.118 / 0.133 → 0.120 / 0.145 | +1.21% |
| tiny4 | bm25 | 0.187 / 0.318 → 0.195 / 0.322 | 0.173 / 0.199 → 0.172 / 0.190 | -0.97% |
| tiny4 | tfidf | 0.186 / 0.258 → 0.186 / 0.266 | 0.174 / 0.220 → 0.173 / 0.207 | -0.48% |
| tiny16 | bm25 | 0.210 / 0.348 → 0.212 / 0.355 | 0.194 / 0.217 → 0.196 / 0.222 | +0.58% |
| tiny16 | tfidf | 0.209 / 0.321 → 0.217 / 0.337 | 0.198 / 0.236 → 0.195 / 0.237 | -1.61% |
| tiny-no-progress | bm25 | 0.084 / 0.176 → 0.082 / 0.181 | 0.075 / 0.111 → 0.073 / 0.116 | -2.42% |
| tiny-no-progress | tfidf | 0.079 / 0.124 → 0.082 / 0.128 | 0.075 / 0.089 → 0.073 / 0.088 | -3.01% |
| tiny4-no-progress | bm25 | 0.140 / 0.262 → 0.140 / 0.272 | 0.127 / 0.155 → 0.126 / 0.150 | -0.48% |
| tiny4-no-progress | tfidf | 0.132 / 0.180 → 0.132 / 0.204 | 0.128 / 0.162 → 0.127 / 0.166 | -0.80% |
| tiny16-no-progress | bm25 | 0.161 / 0.290 → 0.153 / 0.286 | 0.147 / 0.184 → 0.148 / 0.180 | +0.67% |
| tiny16-no-progress | tfidf | 0.158 / 0.282 → 0.156 / 0.270 | 0.149 / 0.186 → 0.147 / 0.188 | -0.81% |

| Case | Startup median ms: old → new | First TF-IDF median ms: old → new | Peak RSS median MiB: old → new |
| --- | --- | --- | --- |
| canonical | 4749.689 → 4815.533 | 21.565 → 21.594 | 1724.203 → 1724.172 |
| numpy | 137.058 → 137.302 | 1.546 → 1.569 | 56.984 → 57.094 |
| wide | 4.133 → 3.787 | 0.487 → 0.491 | 7.922 → 7.828 |
| tiny | 3.304 → 3.683 | 0.179 → 0.184 | 5.703 → 5.812 |
| tiny4 | 3.480 → 3.479 | 0.258 → 0.265 | 5.828 → 5.938 |
| tiny16 | 3.437 → 3.508 | 0.280 → 0.292 | 6.000 → 6.016 |
| tiny-no-progress | 3.463 → 3.455 | 0.126 → 0.133 | 5.703 → 5.750 |
| tiny4-no-progress | 3.723 → 3.573 | 0.179 → 0.205 | 5.875 → 5.922 |
| tiny16-no-progress | 3.547 → 3.436 | 0.236 → 0.218 | 6.062 → 6.031 |

### Second preplanned complete campaign

| Case | Ranker | Initial median / p95 ms: old → new | Repeated median / p95 ms: old → new | Repeated median change |
| --- | --- | --- | --- | --- |
| canonical | bm25 | 3.110 / 5.417 → 2.649 / 4.202 | 2.671 / 4.805 → 2.219 / 3.870 | -16.90% |
| canonical | tfidf | 19.075 / 32.285 → 17.071 / 31.084 | 2.848 / 5.367 → 2.292 / 4.310 | -19.52% |
| numpy | bm25 | 0.928 / 1.236 → 0.888 / 1.233 | 0.793 / 1.145 → 0.771 / 1.106 | -2.85% |
| numpy | tfidf | 1.042 / 2.792 → 1.106 / 2.653 | 0.750 / 1.181 → 0.740 / 1.129 | -1.28% |
| wide | bm25 | 0.499 / 0.574 → 0.509 / 0.580 | 0.417 / 0.724 → 0.418 / 0.642 | +0.28% |
| wide | tfidf | 0.445 / 0.761 → 0.466 / 0.784 | 0.410 / 0.728 → 0.413 / 0.718 | +0.70% |
| tiny | bm25 | 0.138 / 0.237 → 0.137 / 0.222 | 0.119 / 0.229 → 0.118 / 0.152 | -0.89% |
| tiny | tfidf | 0.137 / 0.165 → 0.132 / 0.173 | 0.120 / 0.153 → 0.118 / 0.133 | -1.85% |
| tiny4 | bm25 | 0.200 / 0.337 → 0.196 / 0.338 | 0.176 / 0.215 → 0.175 / 0.199 | -0.65% |
| tiny4 | tfidf | 0.194 / 0.270 → 0.187 / 0.249 | 0.176 / 0.238 → 0.175 / 0.229 | -0.27% |
| tiny16 | bm25 | 0.214 / 0.338 → 0.213 / 0.349 | 0.198 / 0.222 → 0.197 / 0.227 | -0.58% |
| tiny16 | tfidf | 0.215 / 0.338 → 0.212 / 0.327 | 0.201 / 0.228 → 0.196 / 0.234 | -2.17% |
| tiny-no-progress | bm25 | 0.080 / 0.173 → 0.081 / 0.168 | 0.070 / 0.104 → 0.071 / 0.143 | +2.07% |
| tiny-no-progress | tfidf | 0.076 / 0.121 → 0.081 / 0.133 | 0.070 / 0.087 → 0.070 / 0.096 | +0.68% |
| tiny4-no-progress | bm25 | 0.166 / 0.316 → 0.160 / 0.298 | 0.134 / 0.171 → 0.134 / 0.191 | -0.11% |
| tiny4-no-progress | tfidf | 0.159 / 0.236 → 0.157 / 0.193 | 0.132 / 0.201 → 0.133 / 0.203 | +0.49% |
| tiny16-no-progress | bm25 | 0.202 / 0.334 → 0.193 / 0.327 | 0.181 / 0.224 → 0.173 / 0.264 | -4.30% |
| tiny16-no-progress | tfidf | 0.204 / 0.353 → 0.201 / 0.346 | 0.184 / 0.243 → 0.174 / 0.244 | -5.16% |

| Case | Startup median ms: old → new | First TF-IDF median ms: old → new | Peak RSS median MiB: old → new |
| --- | --- | --- | --- |
| canonical | 4946.032 → 4831.054 | 22.209 → 21.742 | 1724.219 → 1724.312 |
| numpy | 136.537 → 137.272 | 1.528 → 1.614 | 56.938 → 57.062 |
| wide | 3.805 → 3.737 | 0.479 → 0.487 | 7.750 → 7.844 |
| tiny | 3.430 → 3.216 | 0.166 → 0.173 | 5.719 → 5.766 |
| tiny4 | 3.633 → 3.448 | 0.274 → 0.247 | 5.812 → 5.938 |
| tiny16 | 3.530 → 3.759 | 0.277 → 0.285 | 6.031 → 6.016 |
| tiny-no-progress | 3.567 → 3.237 | 0.126 → 0.133 | 5.703 → 5.781 |
| tiny4-no-progress | 3.705 → 3.803 | 0.225 → 0.192 | 5.875 → 5.906 |
| tiny16-no-progress | 4.114 → 3.879 | 0.259 → 0.269 | 6.031 → 6.016 |

## Small Core regression controls

These separate campaigns compare freshly built exact registry 0.4.18 and 0.4.19 with the candidate, using the same programs. Seven rotating triples per campaign run both the original six cases and a 96-case extension. Warm-up, full original-JSON comparison and index setup occur before timers. Each campaign has **42 processes and 2,142,000 timed Core calls**. Both together have 84 processes, 4,284,000 calls and **2,688 complete original matrix-response comparisons**, with identical results. Signature checks also cover every original-six and timed matrix sample. This is repeated warm Core time, without MCP transport or DHAT instrumentation.

Keep the two campaigns separate. Original six-case calls build SearchOptions inside the measured loop; the extension reuses fixed options. Do not compare their absolute times or merge distributions. The 96 cases combine four entry counts, both rankers and twelve fixed query/filter/limit cases. Wide query preparation, scope rejection, unknown-only queries and zero limits are retained even though they are outside the optimized sparse explanation work. Historical absolute values with different programs/loads are not a reference substitute.

### First small Core campaign

#### Original six warm cases

| Fixture | 0.4.18 median ns | 0.4.19 median ns | 0.4.20 median ns | vs 0.4.19 | vs 0.4.18 |
| --- | --- | --- | --- | --- | --- |
| tiny16_bm25 | 978.667 | 955.041 | 987.250 | +3.37% | +0.88% |
| tiny16_tfidf | 1037.375 | 1018.417 | 1047.375 | +2.84% | +0.96% |
| tiny4_bm25 | 339.208 | 333.167 | 337.000 | +1.15% | -0.65% |
| tiny4_tfidf | 356.458 | 352.666 | 357.667 | +1.42% | +0.34% |
| tiny_bm25 | 193.458 | 175.875 | 182.041 | +3.51% | -5.90% |
| tiny_tfidf | 183.709 | 177.208 | 179.292 | +1.18% | -2.40% |

Higher candidate medians versus 0.4.19: **6 of 6**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](measurements/0.4.20/small-reference-summary.json). No threshold-based rejection or removal of slow timing samples is used.

#### All 96 extended cases

| Fixture | 0.4.18 median ns | 0.4.19 median ns | 0.4.20 median ns | vs 0.4.19 | vs 0.4.18 |
| --- | --- | --- | --- | --- | --- |
| 16_Bm25_disjoint | 484.208 | 466.584 | 497.875 | +6.71% | +2.82% |
| 16_Bm25_limit_one | 635.500 | 612.834 | 645.208 | +5.28% | +1.53% |
| 16_Bm25_limit_zero | 2.667 | 2.667 | 2.958 | +10.91% | +10.91% |
| 16_Bm25_multi | 1618.292 | 1567.417 | 1602.708 | +2.25% | -0.96% |
| 16_Bm25_punctuation | 145.500 | 144.500 | 142.875 | -1.12% | -1.80% |
| 16_Bm25_repeated | 1185.750 | 1170.042 | 1178.750 | +0.74% | -0.59% |
| 16_Bm25_scope | 999.333 | 965.625 | 978.958 | +1.38% | -2.04% |
| 16_Bm25_single | 1128.500 | 1121.833 | 1137.667 | +1.41% | +0.81% |
| 16_Bm25_unicode | 1311.042 | 1321.333 | 1334.708 | +1.01% | +1.81% |
| 16_Bm25_unknown | 68.500 | 68.083 | 69.417 | +1.96% | +1.34% |
| 16_Bm25_unknown_added | 1189.333 | 1154.084 | 1160.750 | +0.58% | -2.40% |
| 16_Bm25_wide | 18966.041 | 16930.542 | 16811.542 | -0.70% | -11.36% |
| 16_Tfidf_disjoint | 480.459 | 459.708 | 521.833 | +13.51% | +8.61% |
| 16_Tfidf_limit_one | 703.458 | 700.292 | 738.291 | +5.43% | +4.95% |
| 16_Tfidf_limit_zero | 2.667 | 2.708 | 2.958 | +9.23% | +10.91% |
| 16_Tfidf_multi | 1676.709 | 1643.208 | 1669.250 | +1.58% | -0.44% |
| 16_Tfidf_punctuation | 142.584 | 145.042 | 145.833 | +0.55% | +2.28% |
| 16_Tfidf_repeated | 1259.333 | 1247.792 | 1276.916 | +2.33% | +1.40% |
| 16_Tfidf_scope | 1046.542 | 1031.584 | 1058.458 | +2.61% | +1.14% |
| 16_Tfidf_single | 1207.000 | 1193.417 | 1221.167 | +2.33% | +1.17% |
| 16_Tfidf_unicode | 1384.417 | 1397.708 | 1442.292 | +3.19% | +4.18% |
| 16_Tfidf_unknown | 67.500 | 68.708 | 68.834 | +0.18% | +1.98% |
| 16_Tfidf_unknown_added | 1241.916 | 1221.834 | 1254.792 | +2.70% | +1.04% |
| 16_Tfidf_wide | 18854.833 | 16909.500 | 16717.208 | -1.14% | -11.34% |
| 1_Bm25_disjoint | 108.208 | 111.917 | 111.375 | -0.48% | +2.93% |
| 1_Bm25_limit_one | 181.625 | 182.625 | 181.333 | -0.71% | -0.16% |
| 1_Bm25_limit_zero | 2.667 | 2.667 | 2.917 | +9.37% | +9.37% |
| 1_Bm25_multi | 260.166 | 252.625 | 261.417 | +3.48% | +0.48% |
| 1_Bm25_punctuation | 143.125 | 143.709 | 146.208 | +1.74% | +2.15% |
| 1_Bm25_repeated | 242.959 | 249.583 | 249.334 | -0.10% | +2.62% |
| 1_Bm25_scope | 184.958 | 186.042 | 187.750 | +0.92% | +1.51% |
| 1_Bm25_single | 223.458 | 224.459 | 224.042 | -0.19% | +0.26% |
| 1_Bm25_unicode | 424.333 | 424.875 | 428.959 | +0.96% | +1.09% |
| 1_Bm25_unknown | 68.334 | 68.833 | 69.709 | +1.27% | +2.01% |
| 1_Bm25_unknown_added | 221.209 | 221.666 | 225.708 | +1.82% | +2.03% |
| 1_Bm25_wide | 4435.708 | 4342.792 | 4283.417 | -1.37% | -3.43% |
| 1_Tfidf_disjoint | 112.750 | 110.958 | 112.584 | +1.47% | -0.15% |
| 1_Tfidf_limit_one | 188.125 | 188.000 | 190.250 | +1.20% | +1.13% |
| 1_Tfidf_limit_zero | 2.667 | 2.667 | 3.125 | +17.17% | +17.17% |
| 1_Tfidf_multi | 257.167 | 250.875 | 262.417 | +4.60% | +2.04% |
| 1_Tfidf_punctuation | 142.459 | 144.417 | 143.125 | -0.89% | +0.47% |
| 1_Tfidf_repeated | 243.666 | 246.666 | 244.375 | -0.93% | +0.29% |
| 1_Tfidf_scope | 193.209 | 192.959 | 197.666 | +2.44% | +2.31% |
| 1_Tfidf_single | 185.708 | 186.500 | 190.125 | +1.94% | +2.38% |
| 1_Tfidf_unicode | 426.458 | 426.791 | 438.542 | +2.75% | +2.83% |
| 1_Tfidf_unknown | 69.167 | 68.958 | 69.292 | +0.48% | +0.18% |
| 1_Tfidf_unknown_added | 220.459 | 224.291 | 225.458 | +0.52% | +2.27% |
| 1_Tfidf_wide | 4447.500 | 4371.792 | 4370.875 | -0.02% | -1.72% |
| 33_Bm25_disjoint | 979.042 | 962.833 | 1012.541 | +5.16% | +3.42% |
| 33_Bm25_limit_one | 1236.000 | 1237.083 | 1282.084 | +3.64% | +3.73% |
| 33_Bm25_limit_zero | 2.708 | 2.667 | 2.917 | +9.37% | +7.72% |
| 33_Bm25_multi | 2390.333 | 2410.083 | 2451.167 | +1.70% | +2.55% |
| 33_Bm25_punctuation | 143.667 | 142.625 | 147.084 | +3.13% | +2.38% |
| 33_Bm25_repeated | 1784.208 | 1738.500 | 1806.708 | +3.92% | +1.26% |
| 33_Bm25_scope | 1753.708 | 1756.000 | 1826.875 | +4.04% | +4.17% |
| 33_Bm25_single | 1684.958 | 1690.459 | 1750.167 | +3.53% | +3.87% |
| 33_Bm25_unicode | 1865.583 | 1896.334 | 1942.667 | +2.44% | +4.13% |
| 33_Bm25_unknown | 68.083 | 69.125 | 69.750 | +0.90% | +2.45% |
| 33_Bm25_unknown_added | 1764.375 | 1761.833 | 1773.000 | +0.63% | +0.49% |
| 33_Bm25_wide | 28744.917 | 26268.708 | 26676.292 | +1.55% | -7.20% |
| 33_Tfidf_disjoint | 963.583 | 946.167 | 1008.833 | +6.62% | +4.70% |
| 33_Tfidf_limit_one | 1362.833 | 1345.625 | 1382.042 | +2.71% | +1.41% |
| 33_Tfidf_limit_zero | 2.667 | 2.792 | 2.917 | +4.48% | +9.37% |
| 33_Tfidf_multi | 2550.750 | 2557.292 | 2611.667 | +2.13% | +2.39% |
| 33_Tfidf_punctuation | 143.291 | 148.917 | 142.750 | -4.14% | -0.38% |
| 33_Tfidf_repeated | 1908.292 | 1917.875 | 1898.084 | -1.03% | -0.53% |
| 33_Tfidf_scope | 1822.959 | 1821.875 | 1883.125 | +3.36% | +3.30% |
| 33_Tfidf_single | 1829.500 | 1868.875 | 1830.208 | -2.07% | +0.04% |
| 33_Tfidf_unicode | 2003.250 | 2019.250 | 2024.958 | +0.28% | +1.08% |
| 33_Tfidf_unknown | 67.542 | 70.750 | 68.709 | -2.88% | +1.73% |
| 33_Tfidf_unknown_added | 1833.041 | 1907.209 | 1879.292 | -1.46% | +2.52% |
| 33_Tfidf_wide | 28456.917 | 26344.417 | 26629.042 | +1.08% | -6.42% |
| 4_Bm25_disjoint | 160.458 | 162.000 | 162.500 | +0.31% | +1.27% |
| 4_Bm25_limit_one | 240.375 | 247.792 | 244.250 | -1.43% | +1.61% |
| 4_Bm25_limit_zero | 2.667 | 2.667 | 2.958 | +10.91% | +10.91% |
| 4_Bm25_multi | 552.208 | 567.584 | 548.083 | -3.44% | -0.75% |
| 4_Bm25_punctuation | 142.834 | 144.417 | 145.417 | +0.69% | +1.81% |
| 4_Bm25_repeated | 419.916 | 420.500 | 424.708 | +1.00% | +1.14% |
| 4_Bm25_scope | 287.042 | 290.000 | 289.708 | -0.10% | +0.93% |
| 4_Bm25_single | 370.625 | 383.625 | 371.708 | -3.11% | +0.29% |
| 4_Bm25_unicode | 588.750 | 608.167 | 603.708 | -0.73% | +2.54% |
| 4_Bm25_unknown | 69.375 | 70.083 | 69.958 | -0.18% | +0.84% |
| 4_Bm25_unknown_added | 400.958 | 401.792 | 407.875 | +1.51% | +1.73% |
| 4_Bm25_wide | 8112.167 | 7409.291 | 7328.959 | -1.08% | -9.65% |
| 4_Tfidf_disjoint | 163.000 | 158.250 | 162.958 | +2.98% | -0.03% |
| 4_Tfidf_limit_one | 262.042 | 269.542 | 266.625 | -1.08% | +1.75% |
| 4_Tfidf_limit_zero | 3.833 | 2.667 | 2.958 | +10.91% | -22.83% |
| 4_Tfidf_multi | 570.916 | 567.416 | 574.292 | +1.21% | +0.59% |
| 4_Tfidf_punctuation | 144.250 | 145.542 | 145.667 | +0.09% | +0.98% |
| 4_Tfidf_repeated | 450.125 | 451.625 | 458.125 | +1.44% | +1.78% |
| 4_Tfidf_scope | 292.459 | 299.917 | 300.500 | +0.19% | +2.75% |
| 4_Tfidf_single | 393.375 | 385.625 | 401.708 | +4.17% | +2.12% |
| 4_Tfidf_unicode | 621.250 | 635.584 | 639.584 | +0.63% | +2.95% |
| 4_Tfidf_unknown | 70.875 | 68.917 | 70.667 | +2.54% | -0.29% |
| 4_Tfidf_unknown_added | 433.500 | 434.417 | 438.125 | +0.85% | +1.07% |
| 4_Tfidf_wide | 8018.209 | 7281.583 | 7306.292 | +0.34% | -8.88% |

Higher candidate medians versus 0.4.19: **72 of 96**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](measurements/0.4.20/small-reference-summary.json). No threshold-based rejection or removal of slow timing samples is used.

### Second preplanned small Core campaign

#### Original six warm cases

| Fixture | 0.4.18 median ns | 0.4.19 median ns | 0.4.20 median ns | vs 0.4.19 | vs 0.4.18 |
| --- | --- | --- | --- | --- | --- |
| tiny16_bm25 | 1003.750 | 976.459 | 978.208 | +0.18% | -2.54% |
| tiny16_tfidf | 1047.917 | 1032.708 | 1042.375 | +0.94% | -0.53% |
| tiny4_bm25 | 345.209 | 341.250 | 337.667 | -1.05% | -2.18% |
| tiny4_tfidf | 377.541 | 355.417 | 357.792 | +0.67% | -5.23% |
| tiny_bm25 | 198.583 | 175.541 | 183.583 | +4.58% | -7.55% |
| tiny_tfidf | 192.917 | 179.500 | 179.583 | +0.05% | -6.91% |

Higher candidate medians versus 0.4.19: **5 of 6**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](measurements/0.4.20/small-confirmation-summary.json). No threshold-based rejection or removal of slow timing samples is used.

#### All 96 extended cases

| Fixture | 0.4.18 median ns | 0.4.19 median ns | 0.4.20 median ns | vs 0.4.19 | vs 0.4.18 |
| --- | --- | --- | --- | --- | --- |
| 16_Bm25_disjoint | 487.750 | 463.834 | 504.042 | +8.67% | +3.34% |
| 16_Bm25_limit_one | 655.167 | 620.250 | 657.750 | +6.05% | +0.39% |
| 16_Bm25_limit_zero | 2.667 | 2.667 | 2.875 | +7.80% | +7.80% |
| 16_Bm25_multi | 1594.416 | 1564.708 | 1593.125 | +1.82% | -0.08% |
| 16_Bm25_punctuation | 143.500 | 145.041 | 144.333 | -0.49% | +0.58% |
| 16_Bm25_repeated | 1165.208 | 1168.958 | 1161.042 | -0.68% | -0.36% |
| 16_Bm25_scope | 978.167 | 978.333 | 990.542 | +1.25% | +1.27% |
| 16_Bm25_single | 1137.667 | 1111.833 | 1136.709 | +2.24% | -0.08% |
| 16_Bm25_unicode | 1329.750 | 1343.417 | 1342.500 | -0.07% | +0.96% |
| 16_Bm25_unknown | 68.417 | 68.500 | 69.166 | +0.97% | +1.09% |
| 16_Bm25_unknown_added | 1155.958 | 1145.875 | 1141.792 | -0.36% | -1.23% |
| 16_Bm25_wide | 19205.666 | 17054.167 | 16825.125 | -1.34% | -12.39% |
| 16_Tfidf_disjoint | 477.750 | 469.291 | 506.250 | +7.88% | +5.97% |
| 16_Tfidf_limit_one | 699.375 | 717.500 | 718.334 | +0.12% | +2.71% |
| 16_Tfidf_limit_zero | 2.667 | 2.708 | 2.875 | +6.17% | +7.80% |
| 16_Tfidf_multi | 1668.542 | 1699.583 | 1668.542 | -1.83% | +0.00% |
| 16_Tfidf_punctuation | 145.542 | 145.875 | 142.875 | -2.06% | -1.83% |
| 16_Tfidf_repeated | 1240.500 | 1289.959 | 1257.000 | -2.56% | +1.33% |
| 16_Tfidf_scope | 1037.333 | 1042.750 | 1061.875 | +1.83% | +2.37% |
| 16_Tfidf_single | 1201.042 | 1233.167 | 1213.208 | -1.62% | +1.01% |
| 16_Tfidf_unicode | 1384.583 | 1404.000 | 1424.917 | +1.49% | +2.91% |
| 16_Tfidf_unknown | 69.041 | 69.833 | 68.500 | -1.91% | -0.78% |
| 16_Tfidf_unknown_added | 1250.042 | 1283.084 | 1243.208 | -3.11% | -0.55% |
| 16_Tfidf_wide | 18886.417 | 17072.042 | 16587.625 | -2.84% | -12.17% |
| 1_Bm25_disjoint | 106.833 | 109.709 | 113.250 | +3.23% | +6.01% |
| 1_Bm25_limit_one | 179.000 | 180.334 | 182.583 | +1.25% | +2.00% |
| 1_Bm25_limit_zero | 2.708 | 2.667 | 2.916 | +9.34% | +7.68% |
| 1_Bm25_multi | 260.166 | 257.458 | 262.166 | +1.83% | +0.77% |
| 1_Bm25_punctuation | 143.500 | 143.166 | 147.125 | +2.77% | +2.53% |
| 1_Bm25_repeated | 241.666 | 252.083 | 252.208 | +0.05% | +4.36% |
| 1_Bm25_scope | 186.416 | 189.833 | 188.083 | -0.92% | +0.89% |
| 1_Bm25_single | 222.708 | 229.167 | 232.375 | +1.40% | +4.34% |
| 1_Bm25_unicode | 417.041 | 427.750 | 438.083 | +2.42% | +5.05% |
| 1_Bm25_unknown | 70.708 | 69.667 | 71.417 | +2.51% | +1.00% |
| 1_Bm25_unknown_added | 217.250 | 220.500 | 228.417 | +3.59% | +5.14% |
| 1_Bm25_wide | 4313.917 | 4342.375 | 4302.667 | -0.91% | -0.26% |
| 1_Tfidf_disjoint | 111.791 | 112.750 | 114.458 | +1.51% | +2.39% |
| 1_Tfidf_limit_one | 187.708 | 189.583 | 196.708 | +3.76% | +4.79% |
| 1_Tfidf_limit_zero | 2.667 | 2.708 | 3.083 | +13.85% | +15.60% |
| 1_Tfidf_multi | 258.917 | 258.916 | 263.916 | +1.93% | +1.93% |
| 1_Tfidf_punctuation | 147.125 | 141.875 | 145.500 | +2.56% | -1.10% |
| 1_Tfidf_repeated | 241.167 | 249.084 | 251.250 | +0.87% | +4.18% |
| 1_Tfidf_scope | 193.708 | 197.625 | 200.208 | +1.31% | +3.36% |
| 1_Tfidf_single | 189.083 | 188.625 | 192.708 | +2.16% | +1.92% |
| 1_Tfidf_unicode | 427.833 | 433.792 | 437.708 | +0.90% | +2.31% |
| 1_Tfidf_unknown | 69.458 | 68.250 | 71.834 | +5.25% | +3.42% |
| 1_Tfidf_unknown_added | 222.791 | 221.167 | 225.667 | +2.03% | +1.29% |
| 1_Tfidf_wide | 4466.500 | 4383.875 | 4393.209 | +0.21% | -1.64% |
| 33_Bm25_disjoint | 970.916 | 940.375 | 982.833 | +4.52% | +1.23% |
| 33_Bm25_limit_one | 1240.958 | 1192.042 | 1252.208 | +5.05% | +0.91% |
| 33_Bm25_limit_zero | 2.667 | 2.667 | 2.917 | +9.37% | +9.37% |
| 33_Bm25_multi | 2387.209 | 2444.083 | 2449.959 | +0.24% | +2.63% |
| 33_Bm25_punctuation | 144.041 | 143.459 | 145.375 | +1.34% | +0.93% |
| 33_Bm25_repeated | 1711.792 | 1728.833 | 1767.125 | +2.21% | +3.23% |
| 33_Bm25_scope | 1728.458 | 1742.458 | 1753.000 | +0.61% | +1.42% |
| 33_Bm25_single | 1662.250 | 1671.000 | 1705.667 | +2.07% | +2.61% |
| 33_Bm25_unicode | 1863.000 | 1878.375 | 1897.750 | +1.03% | +1.87% |
| 33_Bm25_unknown | 70.084 | 67.375 | 68.791 | +2.10% | -1.84% |
| 33_Bm25_unknown_added | 1706.083 | 1708.250 | 1754.708 | +2.72% | +2.85% |
| 33_Bm25_wide | 28795.166 | 26384.583 | 26513.000 | +0.49% | -7.93% |
| 33_Tfidf_disjoint | 948.250 | 921.000 | 970.333 | +5.36% | +2.33% |
| 33_Tfidf_limit_one | 1320.667 | 1343.959 | 1342.375 | -0.12% | +1.64% |
| 33_Tfidf_limit_zero | 2.667 | 2.667 | 2.875 | +7.80% | +7.80% |
| 33_Tfidf_multi | 2624.667 | 2561.250 | 2606.833 | +1.78% | -0.68% |
| 33_Tfidf_punctuation | 144.500 | 143.625 | 148.208 | +3.19% | +2.57% |
| 33_Tfidf_repeated | 1864.792 | 1907.000 | 1891.834 | -0.80% | +1.45% |
| 33_Tfidf_scope | 1839.750 | 1807.500 | 1842.792 | +1.95% | +0.17% |
| 33_Tfidf_single | 1829.333 | 1879.583 | 1859.792 | -1.05% | +1.67% |
| 33_Tfidf_unicode | 2005.792 | 1990.917 | 2025.667 | +1.75% | +0.99% |
| 33_Tfidf_unknown | 68.333 | 68.958 | 69.250 | +0.42% | +1.34% |
| 33_Tfidf_unknown_added | 1892.917 | 1898.458 | 1879.750 | -0.99% | -0.70% |
| 33_Tfidf_wide | 28446.792 | 26443.625 | 26424.792 | -0.07% | -7.11% |
| 4_Bm25_disjoint | 160.291 | 158.958 | 162.375 | +2.15% | +1.30% |
| 4_Bm25_limit_one | 236.583 | 245.959 | 243.458 | -1.02% | +2.91% |
| 4_Bm25_limit_zero | 2.708 | 2.667 | 2.958 | +10.91% | +9.23% |
| 4_Bm25_multi | 532.084 | 549.750 | 536.875 | -2.34% | +0.90% |
| 4_Bm25_punctuation | 143.250 | 145.792 | 145.917 | +0.09% | +1.86% |
| 4_Bm25_repeated | 416.792 | 430.791 | 429.916 | -0.20% | +3.15% |
| 4_Bm25_scope | 281.333 | 288.125 | 295.917 | +2.70% | +5.18% |
| 4_Bm25_single | 362.958 | 383.542 | 382.000 | -0.40% | +5.25% |
| 4_Bm25_unicode | 586.125 | 607.416 | 608.250 | +0.14% | +3.77% |
| 4_Bm25_unknown | 69.000 | 75.292 | 71.792 | -4.65% | +4.05% |
| 4_Bm25_unknown_added | 394.333 | 414.792 | 412.708 | -0.50% | +4.66% |
| 4_Bm25_wide | 8003.292 | 7296.958 | 7221.375 | -1.04% | -9.77% |
| 4_Tfidf_disjoint | 166.625 | 158.459 | 164.917 | +4.08% | -1.03% |
| 4_Tfidf_limit_one | 267.042 | 270.459 | 274.750 | +1.59% | +2.89% |
| 4_Tfidf_limit_zero | 2.709 | 2.709 | 2.917 | +7.68% | +7.68% |
| 4_Tfidf_multi | 573.916 | 560.000 | 577.792 | +3.18% | +0.68% |
| 4_Tfidf_punctuation | 145.417 | 144.542 | 148.167 | +2.51% | +1.89% |
| 4_Tfidf_repeated | 438.833 | 445.042 | 461.208 | +3.63% | +5.10% |
| 4_Tfidf_scope | 289.292 | 295.541 | 305.750 | +3.45% | +5.69% |
| 4_Tfidf_single | 386.917 | 390.167 | 407.333 | +4.40% | +5.28% |
| 4_Tfidf_unicode | 617.584 | 639.417 | 647.083 | +1.20% | +4.78% |
| 4_Tfidf_unknown | 69.166 | 68.792 | 70.834 | +2.97% | +2.41% |
| 4_Tfidf_unknown_added | 431.167 | 423.458 | 444.208 | +4.90% | +3.02% |
| 4_Tfidf_wide | 8138.500 | 7193.667 | 7223.083 | +0.41% | -11.25% |

Higher candidate medians versus 0.4.19: **70 of 96**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](measurements/0.4.20/small-confirmation-summary.json). No threshold-based rejection or removal of slow timing samples is used.

## Direct canonical search with explicit control modes

Fourteen processes (seven alternating pairs) retain one warmed Engine each. Ten rounds rotate plain, deadline and progress within each query/ranker pair: **8,400 timed Retriever calls**, excluding loading/readiness, stdio and profiling. Each progress request includes an observer and the same deadline/cancellation setup. Every signature agrees; **140 complete original response JSON pairs match**, and control modes also match plain output. Full readiness source text stays private. These are direct retrieval boundaries, not MCP round trips or capacity evidence.

| Control | Ranker | 0.4.19 median / p95 ms | 0.4.20 median / p95 ms | Median change |
| --- | --- | --- | --- | --- |
| plain | Bm25 | 1.791 / 3.600 | 1.213 / 2.486 | -32.28% |
| plain | Tfidf | 1.923 / 3.750 | 1.282 / 2.426 | -33.34% |
| deadline | Bm25 | 2.065 / 4.134 | 1.551 / 3.134 | -24.87% |
| deadline | Tfidf | 2.248 / 4.359 | 1.770 / 3.314 | -21.24% |
| progress | Bm25 | 2.071 / 4.132 | 1.531 / 3.136 | -26.07% |
| progress | Tfidf | 2.255 / 4.419 | 1.768 / 3.336 | -21.60% |

| Query | Candidates | Progress BM25 ms: old → new | Progress TF-IDF ms: old → new |
| --- | --- | --- | --- |
| 0 | 12365 | 1.889 → 1.519 | 2.099 → 1.788 |
| 1 | 13864 | 2.093 → 1.514 | 2.342 → 1.800 |
| 2 | 24299 | 3.573 → 2.734 | 3.950 → 3.078 |
| 3 | 596 | 0.294 → 0.295 | 0.288 → 0.283 |
| 4 | 15403 | 2.219 → 1.589 | 2.408 → 1.839 |
| 5 | 24140 | 3.468 → 2.616 | 3.863 → 3.009 |
| 6 | 3358 | 0.654 → 0.585 | 0.696 → 0.637 |
| 7 | 29600 | 3.879 → 2.583 | 4.311 → 3.061 |
| 8 | 11504 | 1.760 → 1.374 | 1.876 → 1.564 |
| 9 | 0 | 0.002 → 0.002 | 0.001 → 0.001 |

## Separate canonical three-version reference

Five rotating triples compare fresh exact 0.4.18/0.4.19/candidate servers on canonical knowledge with requested progress. Fifteen servers validate **1,020 actual schema replies**; **680 complete structured comparisons** against the candidate match by RPC ID except checked status version and version-qualified schema URNs. Schemas, source ranges and inputs agree. This control is separate from the two ordinary campaigns. It checks whether the previous large-query benefit remains visible against the older release; it does not establish the cause of any historical regression.

| Version | Ranker | Initial median / p95 ms | Repeated median / p95 ms | Peak child RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.18 | bm25 | 3.297 / 7.333 | 3.013 / 7.494 | 1724.266 |
| 0.4.18 | tfidf | 22.285 / 47.144 | 3.715 / 9.251 | 1724.266 |
| 0.4.19 | bm25 | 3.447 / 6.233 | 3.000 / 5.753 | 1725.094 |
| 0.4.19 | tfidf | 20.611 / 37.172 | 3.851 / 7.538 | 1725.094 |
| 0.4.20 | bm25 | 3.747 / 9.312 | 3.122 / 10.657 | 1725.344 |
| 0.4.20 | tfidf | 22.642 / 52.394 | 4.055 / 10.872 | 1725.344 |

| Query | 0.4.18 repeated TF-IDF ms | 0.4.19 repeated TF-IDF ms | 0.4.20 repeated TF-IDF ms |
| --- | --- | --- | --- |
| 0 | 3.109 | 3.107 | 3.583 |
| 1 | 3.664 | 3.778 | 3.341 |
| 2 | 5.868 | 5.802 | 5.298 |
| 3 | 0.779 | 0.779 | 1.003 |
| 4 | 3.714 | 3.934 | 3.362 |
| 5 | 5.957 | 5.327 | 5.073 |
| 6 | 1.466 | 1.361 | 1.501 |
| 7 | 6.511 | 6.155 | 5.138 |
| 8 | 4.079 | 3.940 | 4.172 |
| 9 | 0.140 | 0.121 | 0.209 |

## Canonical first-pass heap traffic

Two instrumented processes index all 167,942 records before profiling the same ten initial TF-IDF queries, then drop results before net deltas. The existing direct candidate collection remains; the previous large-query posting cursors and norm memo/cache still exist. Heap traffic, net persistent hints, stack state and whole-process RSS are different costs. Instrumented durations are excluded from latency comparisons.

| Metric | 0.4.19 | 0.4.20 |
| --- | --- | --- |
| Retained hint bytes | 2228312 | 2228312 |
| Retained hint blocks | 2 | 2 |
| Total query allocated bytes | 7213987 | 7213987 |
| Total query allocation blocks | 415 | 415 |

## Development pilot and host-load limits

The [development pilot](measurements/0.4.20/development-pilot-summary.json) remains separate from all final samples. Its separate small/wide/canonical/tiny rotating triples supported the candidate choice. The initial stopped out-of-bound pilot is retained as well. Pilot samples do not enter final distributions. All source/lock/binary identities and raw observations remain locally retained.

Two-second monitoring retains **493 snapshots**, including **54 with other rustc processes**, maximum 3. All managed compilers finish before timing. Other workloads are not stopped, and snapshots do not prove continuous idleness or identify a timing cause. CPU counts include the active benchmark workers.

| Stage start | Other rustc count | Other rustc CPU total | Processes over 50% CPU |
| --- | --- | --- | --- |
| large-standard | 0 | 0.000 | 1 |
| wide-standard | 0 | 0.000 | 1 |
| small-reference | 0 | 0.000 | 2 |
| retrieval | 0 | 0.000 | 1 |
| query | 0 | 0.000 | 1 |
| build | 0 | 0.000 | 1 |
| controlled | 0 | 0.000 | 1 |
| mcp | 0 | 0.000 | 1 |
| corpus | 0 | 0.000 | 1 |
| mcp-confirmation | 0 | 0.000 | 1 |
| reference | 3 | 299.500 | 5 |
| small-confirmation | 0 | 0.000 | 1 |
| wide-confirmation | 0 | 0.000 | 1 |
| wide-extended | 0 | 0.000 | 2 |
| large-confirmation | 0 | 0.000 | 1 |
| limits | 0 | 0.000 | 2 |

All observed slower cases remain in their own distributions; no sample is replaced because of load. One machine, randomized hashing, exposed questions, tuned fixtures and finite trials limit generalization.

## Retained Core controls

These controls retain positive and negative timing changes. Single known-term requests retain the direct candidate path and exactly two retain ordered merging. Wider requests keep the heap/set strategies; large frequency cursors remain available afterward. Large sorted candidate sets can use the bounded document cursor after 128 candidates; earlier shared explanation intersection remains. Read every row independently, including slower cases. Seven alternating pairs give fourteen plain processes per family. Norms are initialized for warm timing; two separate DHAT processes measure allocations. Instrumented durations are excluded from latency comparisons. Every result signature agrees and temporary-call retained deltas are zero.

### Small warm Core: 84,000 calls on 1/4/16 entries

| Fixture | Median change | Blocks/call: old → new | Allocated bytes/call: old → new |
| --- | --- | --- | --- |
| tiny16_bm25 | +1.88% | 29.0 → 29.0 | 2536.0 → 2536.0 |
| tiny16_tfidf | +1.29% | 29.0 → 29.0 | 2536.0 → 2536.0 |
| tiny4_bm25 | +0.38% | 15.0 → 15.0 | 1204.0 → 1204.0 |
| tiny4_tfidf | +0.83% | 15.0 → 15.0 | 1204.0 → 1204.0 |
| tiny_bm25 | +4.55% | 9.0 → 9.0 | 778.0 → 778.0 |
| tiny_tfidf | +1.89% | 9.0 → 9.0 | 778.0 → 778.0 |

### All 25 warm controls: 282,800 calls

| Fixture | Median change | Blocks/call: old → new | Allocated bytes/call: old → new |
| --- | --- | --- | --- |
| coverage_repeated | +1.75% | 1047.0 → 1047.0 | 92942.0 → 92942.0 |
| coverage_small | +1.60% | 784.0 → 784.0 | 78046.0 → 78046.0 |
| excerpt_ascii | +1.48% | 762.0 → 762.0 | 252438.0 → 252438.0 |
| excerpt_mixed | +0.71% | 569.0 → 569.0 | 417113.0 → 417113.0 |
| excerpt_unicode | +0.33% | 197.0 → 197.0 | 72254.0 → 72254.0 |
| long_repeated | -0.67% | 4.0 → 4.0 | 18798.0 → 18798.0 |
| mixed_unicode | +5.42% | 7.0 → 7.0 | 3155.0 → 3155.0 |
| punctuation | +2.32% | 1.0 → 1.0 | 768.0 → 768.0 |
| relevant_retrieval | +1.03% | 812.0 → 812.0 | 93562.0 → 93562.0 |
| repeated_search | +6.28% | 35.0 → 35.0 | 19432.0 → 19432.0 |
| repeated_short | -1.75% | 4.0 → 4.0 | 1656.0 → 1656.0 |
| sparse_bm25_128 | +0.32% | 359.0 → 359.0 | 29074.0 → 29074.0 |
| sparse_bm25_33 | +1.05% | 241.0 → 241.0 | 16309.0 → 16309.0 |
| sparse_bm25_512 | +1.02% | 1337.0 → 1337.0 | 119124.0 → 119124.0 |
| sparse_tfidf_128 | +0.38% | 474.0 → 474.0 | 35140.0 → 35140.0 |
| sparse_tfidf_33 | +2.68% | 241.0 → 241.0 | 16309.0 → 16309.0 |
| sparse_tfidf_512 | +0.70% | 1337.0 → 1337.0 | 119124.0 → 119124.0 |
| tfidf_union_32 | +0.87% | 244.0 → 244.0 | 19834.0 → 19834.0 |
| tfidf_union_33 | +2.48% | 296.0 → 296.0 | 27854.0 → 27854.0 |
| tfidf_warm | +1.55% | 71.0 → 71.0 | 19335.0 → 19335.0 |
| union_32 | -0.27% | 244.0 → 244.0 | 19834.0 → 19834.0 |
| union_33 | +1.75% | 296.0 → 296.0 | 27854.0 → 27854.0 |
| unique_long | -0.17% | 299.0 → 299.0 | 32672.0 → 32672.0 |
| unique_search | +2.40% | 72.0 → 72.0 | 19350.0 → 19350.0 |
| unique_short | -1.77% | 296.0 → 296.0 | 18548.0 → 18548.0 |

### Cold norm and heap controls

Six fixtures make twenty fresh-index-clone calls each across fourteen processes: 1,680 calls. Clone/setup is outside timing. Process/instruction/filesystem caches remain; cold means norm hints, not disks. Separate heap runs drop results before net retained measurements. The existing inline/optional heap word memo and persistent norm storage remain. Large small-term requests may add up to four temporary posting cursors; canonical allocation traffic below records that cost. Heap profiles exclude stack state/control fields and temporary frames.

| Fixture | Old median / p95 ns | New median / p95 ns |
| --- | --- | --- |
| index | 106520.500 / 119291.700 | 112812.500 / 127729.800 |
| wide | 32854.500 / 50391.700 | 35000.000 / 42858.000 |
| sparse | 430416.500 / 490262.200 | 424604.500 / 487726.800 |
| tiny | 583.000 / 1669.100 | 583.000 / 1500.000 |
| tiny4 | 834.000 / 1417.000 | 833.000 / 1425.300 |
| tiny16 | 2375.000 / 3835.100 | 2458.000 / 3902.050 |

| Cold fixture | Allocated bytes: old → new | Retained hint bytes: old → new |
| --- | --- | --- |
| index | 92356 → 92356 | 34904 → 34904 |
| wide | 19165 → 19165 | 4440 → 4440 |
| sparse | 140704 → 140704 | 8792 → 8792 |
| tiny | 934 → 934 | 156 → 156 |
| tiny4 | 1504 → 1504 | 224 → 224 |
| tiny16 | 3668 → 3668 | 632 → 632 |

Canonical first-pass heap records, all samples, blocks and cold tails remain in public JSON. Retained hints, heap traffic and whole-process RAM are different measures.

## Finite mixed control, documents and construction

The 6,000-entry mixed control retains four continuous warm readers while 100 initially uncached-group driver queries finish, then makes two quiescent probes. Five alternating pairs give 1,000 driver calls and twenty quiescent probes. Warm counts vary with the stop condition and new-group calls can become warm. This finite control is not fixed-load throughput, capacity or fairness evidence.

| Mixed work, ns | Old median / p95 | New median / p95 |
| --- | --- | --- |
| new_group_query_ns | 140229.500 / 148975.650 | 173271.000 / 190670.200 |
| post_readers_first_ns | 137458.000 / 165166.400 | 145208.000 / 182908.000 |
| post_readers_second_ns | 84250.000 / 86150.000 | 85375.000 / 86975.200 |

The private document corpus retains 72 Markdown files, 2,755,468 bytes and 1,910 chunks. All 342 authored questions (276 positive, 66 unsupported) are exposed regressions, not fresh holdouts or logged production queries. Literal BM25, prepared BM25 and coverage retain six passages, 100 candidates, 2,000 bytes/pass and 12,000 total bytes. All **1,026 complete original response JSON strings match byte-for-byte**, including scores, ordering, evidence, source text and positions. One warm-up and five repeats/question/configuration/version give **10,260 timed round trips**. These three configurations are BM25 controls; independent score oracles and actual protocol pairs cover TF-IDF. No new relevance judgment or model reply is produced.

All document timing aggregates and per-question results remain in the records. Construction uses forty plain processes on canonical data and three synthetic distributions, plus six separate profiles. Parsing/generation is outside construction timing; stats/signatures/live derived heap agree. The execution/shared-index worker checks all 167,942 entries through 128 changes/restoration, rebuilds, JSON round trips, retained clones, parallel readers and cancellation/deadline rejection. Portable exports and source hashes agree.

### Document round trips

| Case | Old median / p95 ms | New median / p95 ms |
| --- | --- | --- |
| baseline | 0.825 / 1.241 | 0.771 / 1.170 |
| prepared | 0.876 / 1.348 | 0.873 / 1.307 |
| evidence | 0.858 / 1.348 | 0.853 / 1.276 |

### Construction only

| Case | Old median / p95 ms | New median / p95 ms |
| --- | --- | --- |
| canonical | 4910.114 / 4935.561 | 4908.644 / 5076.727 |
| ascii | 14.401 / 14.450 | 14.543 / 14.834 |
| mixed | 34.490 / 34.801 | 33.995 / 34.562 |
| long_tokens | 1.497 / 1.548 | 1.589 / 1.600 |

## Limits and preservation

Measurements use one AC-powered Apple M5 Max with 64 GB, Rust 1.98.0 and Python 3.15.2. Stage-start CPU/compiler snapshots remain. Own compilers finish before timing; this does not establish continuously idle hardware. Randomized hashing, scheduling, one machine/corpus, exposed questions and finite process counts limit generalization. The separate development pilot remains; assess final per-case medians and tails independently. Host-load sampling records observations rather than continuous idleness. No universal latency, total-memory, semantic-quality, token-saving or security-audit claim is made.

Canonical knowledge remains 167,942 entries from 35 PDFs and 49,086 physical pages, SHA-256 `13168d87872a137355fd8499c90a6084f52243a2ec7456f29b5fcb2a1c7423c3`. The PDF parser is unchanged; this is not a fresh large-PDF extraction campaign. Lexical overlap and exact quotes do not prove factual support or entailment. No model, daemon or DenkMaps integration is added.

Preserve PDFs, knowledge, snapshots, all pilot/final measurements, slower results, raw transcripts, profiles, exact small executables and locks. After verified publication archive package/proof, then delete only the named temporary Cargo/Miri targets. Default features remain empty.
