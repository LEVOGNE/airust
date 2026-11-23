# AIRust 0.4.23: less temporary storage for short repeated-word searches

A raw query of at most 4,096 bytes containing repetitions of one unchanged lowercase ASCII alphanumeric word can now borrow that word and its exact count. A private iterator supplies the same word/count pair to the existing checked ranking body. This avoids temporary normalized text, owned keys and a frequency-map node. Different words, longer input, uppercase, punctuation, Unicode and preanalyzed frequency maps keep existing analysis. The additional recognition scan has a fixed byte bound and constant auxiliary memory. The 4 KiB bound is an optimization guard, not a new rejection limit on the public Core search API.

The original single-word path remains. Candidate collection, posting/document cursors, exact broad explanations, score arithmetic/order, scopes, norm hints, matched terms, ties and original source positions retain their contracts. Results own their matched strings; borrowed query text does not escape. Cancellation/deadlines retain the initial and all prepared-term/candidate/scoring/winner checkpoints and reject partial successful results. Existing long-query analysis is not newly made preemptible.

Across the eight repeated-word cases in two separate three-version Core campaigns, median changes versus fresh exact 0.4.22 range **-43.87% to -4.80%** and **-57.15% to +8.55%**. Negative means shorter observed time. The original six single-word controls range **-3.01% to +1.26%** and **-1.51% to +1.98%**. Every fallback and slower case remains. Core and MCP are different boundaries; no universal speedup, statistical significance or semantic-quality claim is made.

MCP implementation source is byte-for-byte unchanged from 0.4.22. Its bundled retrieval uses preanalyzed frequency maps and does not acquire a new repeated-word query mode. Its earlier bounded-payload/move optimization remains. The two ordinary canonical repeated MCP TF-IDF changes are **+15.92% / +3.54%**; separate large/broad/small controls remain independent below. Final response construction still follows worker search completion, and startup loading/transport are outside tool computation deadlines. This release does not claim those lifecycle limits are solved.

## Executed validation and API contract

The full matrix passes **252 default tests, 905 all-feature checks including 39 documentation examples, 39 isolated feature entries and five Python tests**. Strict Clippy/docs, Rust 1.89 provider-neutral and Rust 1.92 all-feature checks pass. API comparison with tagged 0.4.22 reports **229 pass, 31 skip**. The independent Rust 1.89 consumer covers retrieval, coverage, quotes, document positions and MCP/schema construction. The unchanged PDF backend passes 14 regressions and its 381-source-file provenance check. The listed ignored crash-writer helper is intentionally launched by its parent recovery test. The refreshed dependency audit reports zero known vulnerabilities in its checked database; maintenance notices remain in the resource record.

Three new regressions compare exact repetition counts with independent normalization, conservative rejection and iterator exhaustion, including all ASCII controls and exact 4,096/4,097-byte guards; compare borrowed/sorted/public search with independently calculated scores, ties, metadata scopes and zero/one/many limits at native sizes 1/4/33/127/128/129/1,023/1,024/1,025; and check exact cold-state checkpoint counts, injected failures, unchanged knowledge and successful recovery. Native execution-feature checks cover public controlled dispatch and pre-cancellation. The existing full-scan oracle independently covers both rankers, wide queries, scopes, limits and cache states. Earlier source-position, hostile-MCP, timeout, disconnect and drain checks remain.

**Thirty-six finite Tree Borrows Miri cases pass:** the previous 33 cases plus scaled versions of the three new repeated-word tests. Native tests retain larger boundaries. The three earlier pure MCP reply cases do not execute Tokio I/O. Only extra floating-point rounding injection is disabled; memory/alias checks remain. Tokio/PDF/FFI/ONNX execution is outside this interpreter scope. These are not independent security or soundness audits. Extracted-package tests, exact-commit CI, a fresh Rust 1.89 registry consumer, tag/VCS/checksum and release-asset agreement gate completion.

Engine, SearchOptions, Retriever, WorkControl, McpServer and McpLimits usage stays the same. Default features stay empty. No dependency, tool argument, advertised response shape or saved-format migration is added. Read [Core performance](CORE-PERFORMANCE.md), [API](API.md), [cookbook](API-COOKBOOK.md), [MCP](MCP.md) and [security scope](MCP-SECURITY.md).

## Frozen method and development record

Baseline is exact registry 0.4.22, VCS `0657dcdc637f7eb687129ea298556b90f7ec07b5`; the earlier reference is exact registry 0.4.21, VCS `8ce456be12218f38cffc32ed1737accbe24f8668`. Every cached registry source file is compared byte-for-byte with its checksum-verified published archive before final timing. Actual worker sources/manifests/locks, executables, runtime, compile-time schemas and inputs are hashed and preserved. Read the [method](validation-0.4.23/README.md).

Two separate seven-triple Core development trials retain all original-six and 96-case samples. The second checks differing words before validating the first word as canonical, avoiding unnecessary validation on fallback. The first trial completed every response/signature comparison but its final display hit an old versioned metric-key label. The original summary, failed display log and explicit correction remain; percentages were rederived from unchanged arm-labeled medians, without rerunning timing. A separate three-triple MCP pilot keeps large, standard-wide, extended-wide and tiny cases plus original six Core controls. All development observations remain outside final distributions. No pilot or faster rerun replaces a final sample.

## Primary repeated-word Core target and confirmation

Each campaign runs seven rotating triples of exact 0.4.21/0.4.22/0.4.23 with the original six cases and all 96 extended cases: **42 processes, 2,142,000 timed Core calls and 1,344 complete original matrix-response comparisons**. Together: **84 processes, 4,284,000 calls and 2,688 complete comparisons**, with equal results. The eight repeated-word cases cover 1/4/16/33 entries and both rankers. The full 96 also retain distinct/unknown/Unicode/punctuation/wide queries, metadata scopes and limit controls. All result and timing signatures agree. Keep each distribution separate. Original-six options are constructed inside their loop; the matrix reuses fixed options. Do not compare those absolute boundaries interchangeably.

## Retained small-MCP controls

Two separate six-case three-version campaigns use 1/4/16 entries, each with/without progress. Each has 126 fresh servers, 8,568 schema-valid actual replies and 5,712 complete structured comparisons; together **252 / 17,136 / 11,424**. Complete results, schemas, requested/absent progress, input hashes and EOF shutdown agree except checked status version and version-qualified schema URNs. Initial/repeated, query/ranker, process-sum, startup and RSS records stay separate. These guard earlier behavior; this Core change does not imply an MCP speedup.

### Standard small-MCP campaign

| Case | Ranker | 0.4.21 repeated median / p95 ms | 0.4.22 repeated median / p95 ms | 0.4.23 repeated median / p95 ms | vs 0.4.22 |
| --- | --- | --- | --- | --- | --- |
| tiny | bm25 | 0.152 / 0.237 | 0.155 / 0.245 | 0.165 / 1.092 | +6.37% |
| tiny | tfidf | 0.151 / 0.278 | 0.155 / 0.288 | 0.163 / 0.956 | +5.59% |
| tiny4 | bm25 | 0.266 / 4.708 | 0.230 / 2.328 | 0.255 / 5.289 | +10.85% |
| tiny4 | tfidf | 0.268 / 3.438 | 0.226 / 1.142 | 0.248 / 4.550 | +9.48% |
| tiny16 | bm25 | 0.328 / 4.395 | 0.254 / 3.633 | 0.286 / 5.637 | +12.52% |
| tiny16 | tfidf | 0.318 / 6.412 | 0.255 / 3.789 | 0.282 / 8.335 | +10.71% |
| tiny-no-progress | bm25 | 0.112 / 0.140 | 0.115 / 0.156 | 0.111 / 0.153 | -3.45% |
| tiny-no-progress | tfidf | 0.119 / 0.203 | 0.116 / 0.188 | 0.115 / 0.199 | -1.24% |
| tiny4-no-progress | bm25 | 0.139 / 0.195 | 0.132 / 0.216 | 0.130 / 0.183 | -1.69% |
| tiny4-no-progress | tfidf | 0.134 / 0.178 | 0.127 / 0.213 | 0.128 / 0.173 | +1.10% |
| tiny16-no-progress | bm25 | 0.152 / 0.199 | 0.143 / 0.210 | 0.153 / 0.242 | +7.28% |
| tiny16-no-progress | tfidf | 0.151 / 0.178 | 0.143 / 0.192 | 0.151 / 0.212 | +5.25% |

All initial/repeated per-query means, medians, p95, process sums, startup, RSS and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.4.23/docs/measurements/0.4.23/small-mcp-standard-summary.json). No slower sample is removed.

### Confirmation small-MCP campaign

| Case | Ranker | 0.4.21 repeated median / p95 ms | 0.4.22 repeated median / p95 ms | 0.4.23 repeated median / p95 ms | vs 0.4.22 |
| --- | --- | --- | --- | --- | --- |
| tiny | bm25 | 0.159 / 2.506 | 0.158 / 1.675 | 0.144 / 1.430 | -8.91% |
| tiny | tfidf | 0.162 / 2.406 | 0.162 / 2.621 | 0.141 / 0.769 | -13.22% |
| tiny4 | bm25 | 0.267 / 6.296 | 0.279 / 3.414 | 0.249 / 5.977 | -10.68% |
| tiny4 | tfidf | 0.255 / 2.605 | 0.264 / 3.135 | 0.249 / 3.399 | -5.84% |
| tiny16 | bm25 | 0.290 / 1.928 | 0.246 / 2.228 | 0.290 / 3.944 | +17.91% |
| tiny16 | tfidf | 0.294 / 1.104 | 0.251 / 0.904 | 0.293 / 2.905 | +17.10% |
| tiny-no-progress | bm25 | 0.089 / 0.134 | 0.088 / 0.136 | 0.082 / 0.137 | -6.27% |
| tiny-no-progress | tfidf | 0.092 / 0.153 | 0.089 / 0.180 | 0.082 / 0.177 | -7.16% |
| tiny4-no-progress | bm25 | 0.150 / 0.227 | 0.128 / 0.169 | 0.129 / 0.187 | +0.88% |
| tiny4-no-progress | tfidf | 0.143 / 0.228 | 0.124 / 0.152 | 0.125 / 0.176 | +0.70% |
| tiny16-no-progress | bm25 | 0.162 / 0.248 | 0.150 / 0.228 | 0.156 / 0.219 | +4.32% |
| tiny16-no-progress | tfidf | 0.162 / 0.215 | 0.151 / 0.203 | 0.154 / 0.218 | +2.38% |

All initial/repeated per-query means, medians, p95, process sums, startup, RSS and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.4.23/docs/measurements/0.4.23/small-mcp-confirmation-summary.json). No slower sample is removed.

## Retained large and broad MCP controls

The two large-corpus campaigns each run seven rotating 0.4.21/0.4.22/0.4.23 triples on the same 167,942-entry knowledge input and ten retained queries. Each has 21 fresh servers, 1,428 schema-valid replies and 952 complete cross-version structured comparisons. Together: **42 servers, 2,856 schema-valid replies and 1,904 complete comparisons**. All compared results agree except asserted status version and version-qualified schema URNs. Each query/ranker has seven initial and fourteen repeated samples. Input candidate counts range from zero to 29,600; do not infer gains for all corpus densities or gaps. Dense, filtered, sparse and cancellation correctness also has independent native tests.

Three separate retained broad controls each use 21 servers: **63 servers, 4,284 actual schema replies and 2,856 complete structured comparisons**. They guard the 0.4.21 broad-explanation improvement rather than replace large-corpus evidence. Standard/confirmation retain the 257-record sparse/dense vocabulary fixture and ten original queries. Extended probes 32/33/64/65/128/129/192/255/256 known terms and 255 known terms plus a duplicate and unknown word. Existing normalized-term and byte limits remain. Keep all five distributions separate.

Search timing includes Rust work, serialization, stdio, Python parsing and requested progress. Initial calls, repeated calls, startup, per-query means/medians/tails, per-process totals and RSS are separate. Original standard queries are not all equally wide. Aggregate medians alone can obscure differences; retain every per-query row and raw timing observation. No sample is removed or replaced.

### Standard large-corpus campaign

| Ranker / phase | 0.4.21 median / p95 ms | 0.4.22 median / p95 ms | 0.4.23 median / p95 ms | vs 0.4.22 | vs 0.4.21 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 3.493 / 7.533 | 3.354 / 9.781 | 3.676 / 7.456 | +9.60% | +5.22% |
| tfidf / initial | 24.534 / 47.323 | 24.929 / 55.835 | 23.318 / 43.996 | -6.46% | -4.96% |
| bm25 / repeated | 2.952 / 7.750 | 3.502 / 9.204 | 3.014 / 7.274 | -13.92% | +2.13% |
| tfidf / repeated | 3.331 / 9.584 | 4.035 / 11.177 | 3.609 / 10.248 | -10.54% | +8.36% |

| Query | Repeated TF-IDF median ms: 0.4.21 / 0.4.22 / 0.4.23 | vs 0.4.22 | Initial TF-IDF mean ms: 0.4.22 → 0.4.23 |
| --- | --- | --- | --- |
| 0 | 3.274 / 3.910 / 3.439 | -12.06% | 31.532 → 29.322 |
| 1 | 3.445 / 3.675 / 3.741 | +1.79% | 31.533 → 28.529 |
| 2 | 5.346 / 7.311 / 5.543 | -24.19% | 44.191 → 42.208 |
| 3 | 0.979 / 1.129 / 1.053 | -6.69% | 4.355 → 2.936 |
| 4 | 3.354 / 4.612 / 3.490 | -24.34% | 29.340 → 18.375 |
| 5 | 5.540 / 5.193 / 4.869 | -6.23% | 38.478 → 39.802 |
| 6 | 1.598 / 1.540 / 1.532 | -0.49% | 8.068 → 8.007 |
| 7 | 5.997 / 5.357 / 5.545 | +3.51% | 35.107 → 35.347 |
| 8 | 3.129 / 3.012 / 2.824 | -6.21% | 4.347 → 4.583 |
| 9 | 0.171 / 0.204 / 0.198 | -2.57% | 2.535 → 1.360 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.21 | 191.427 | 66.947 | 6178.486 | 1724.516 |
| 0.4.22 | 247.514 | 68.025 | 5711.774 | 1724.828 |
| 0.4.23 | 187.341 | 61.110 | 6770.736 | 1725.000 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.4.23/docs/measurements/0.4.23/large-standard-summary.json).

### Confirmation large-corpus campaign

| Ranker / phase | 0.4.21 median / p95 ms | 0.4.22 median / p95 ms | 0.4.23 median / p95 ms | vs 0.4.22 | vs 0.4.21 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 3.036 / 11.252 | 3.100 / 8.114 | 3.012 / 5.172 | -2.85% | -0.80% |
| tfidf / initial | 20.785 / 36.654 | 22.063 / 47.006 | 22.472 / 37.385 | +1.85% | +8.12% |
| bm25 / repeated | 2.639 / 4.922 | 2.482 / 7.112 | 2.664 / 7.289 | +7.33% | +0.95% |
| tfidf / repeated | 3.113 / 6.129 | 2.798 / 9.269 | 3.194 / 6.913 | +14.16% | +2.58% |

| Query | Repeated TF-IDF median ms: 0.4.21 / 0.4.22 / 0.4.23 | vs 0.4.22 | Initial TF-IDF mean ms: 0.4.22 → 0.4.23 |
| --- | --- | --- | --- |
| 0 | 2.919 / 2.506 / 3.209 | +28.05% | 27.048 → 25.924 |
| 1 | 3.256 / 3.035 / 3.190 | +5.11% | 29.423 → 25.825 |
| 2 | 5.035 / 5.042 / 5.295 | +5.01% | 42.148 → 38.100 |
| 3 | 0.949 / 0.802 / 0.893 | +11.31% | 5.291 → 2.023 |
| 4 | 3.230 / 3.132 / 3.269 | +4.37% | 20.370 → 17.385 |
| 5 | 5.307 / 4.670 / 4.771 | +2.15% | 31.050 → 28.812 |
| 6 | 1.524 / 1.399 / 1.526 | +9.04% | 6.263 → 5.611 |
| 7 | 4.763 / 4.889 / 5.113 | +4.59% | 32.491 → 30.263 |
| 8 | 2.494 / 2.447 / 3.009 | +22.93% | 3.100 → 3.633 |
| 9 | 0.136 / 0.134 / 0.152 | +13.24% | 0.384 → 0.267 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.21 | 170.501 | 60.900 | 5431.476 | 1724.469 |
| 0.4.22 | 166.252 | 54.124 | 5313.535 | 1724.359 |
| 0.4.23 | 174.742 | 57.944 | 5716.215 | 1724.469 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.4.23/docs/measurements/0.4.23/large-confirmation-summary.json).

### Standard broad-query campaign

| Ranker / phase | 0.4.21 median / p95 ms | 0.4.22 median / p95 ms | 0.4.23 median / p95 ms | vs 0.4.22 | vs 0.4.21 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 0.699 / 2.676 | 0.677 / 2.441 | 0.625 / 5.439 | -7.69% | -10.53% |
| tfidf / initial | 0.730 / 1.780 | 0.672 / 2.026 | 0.614 / 2.921 | -8.63% | -15.87% |
| bm25 / repeated | 0.622 / 2.070 | 0.621 / 2.018 | 0.572 / 2.168 | -7.86% | -8.05% |
| tfidf / repeated | 0.627 / 1.312 | 0.640 / 1.089 | 0.563 / 0.965 | -11.96% | -10.11% |

| Query | Repeated TF-IDF median ms: 0.4.21 / 0.4.22 / 0.4.23 | vs 0.4.22 | Initial TF-IDF mean ms: 0.4.22 → 0.4.23 |
| --- | --- | --- | --- |
| 0 | 0.572 / 0.606 / 0.551 | -8.98% | 0.794 → 0.974 |
| 1 | 0.578 / 0.532 / 0.488 | -8.34% | 0.851 → 0.778 |
| 2 | 0.550 / 0.601 / 0.554 | -7.82% | 1.108 → 1.988 |
| 3 | 0.932 / 0.885 / 0.857 | -3.22% | 1.655 → 1.047 |
| 4 | 0.591 / 0.618 / 0.541 | -12.43% | 0.809 → 1.256 |
| 5 | 0.580 / 0.605 / 0.527 | -12.88% | 0.569 → 0.926 |
| 6 | 0.597 / 0.579 / 0.553 | -4.50% | 0.752 → 0.714 |
| 7 | 0.978 / 0.890 / 0.817 | -8.28% | 1.031 → 1.003 |
| 8 | 0.538 / 0.543 / 0.509 | -6.27% | 0.710 → 0.685 |
| 9 | 0.961 / 0.867 / 0.878 | +1.27% | 0.975 → 1.089 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.21 | 7.469 | 14.425 | 5.556 | 7.938 |
| 0.4.22 | 7.113 | 13.135 | 5.069 | 7.781 |
| 0.4.23 | 6.439 | 12.582 | 4.738 | 7.656 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.4.23/docs/measurements/0.4.23/wide-standard-summary.json).

### Confirmation broad-query campaign

| Ranker / phase | 0.4.21 median / p95 ms | 0.4.22 median / p95 ms | 0.4.23 median / p95 ms | vs 0.4.22 | vs 0.4.21 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 0.530 / 0.705 | 0.492 / 0.647 | 0.496 / 0.632 | +0.96% | -6.31% |
| tfidf / initial | 0.517 / 0.916 | 0.482 / 0.789 | 0.459 / 0.763 | -4.64% | -11.09% |
| bm25 / repeated | 0.501 / 0.876 | 0.416 / 0.713 | 0.419 / 0.819 | +0.90% | -16.37% |
| tfidf / repeated | 0.472 / 0.905 | 0.416 / 0.718 | 0.425 / 0.747 | +2.21% | -10.02% |

| Query | Repeated TF-IDF median ms: 0.4.21 / 0.4.22 / 0.4.23 | vs 0.4.22 | Initial TF-IDF mean ms: 0.4.22 → 0.4.23 |
| --- | --- | --- | --- |
| 0 | 0.447 / 0.408 / 0.406 | -0.58% | 0.492 → 0.511 |
| 1 | 0.414 / 0.364 / 0.358 | -1.51% | 0.442 → 0.419 |
| 2 | 0.464 / 0.403 / 0.408 | +1.32% | 0.480 → 0.466 |
| 3 | 0.768 / 0.676 / 0.699 | +3.45% | 0.767 → 0.764 |
| 4 | 0.436 / 0.407 / 0.413 | +1.42% | 0.422 → 0.404 |
| 5 | 0.434 / 0.407 / 0.393 | -3.63% | 0.403 → 0.392 |
| 6 | 0.450 / 0.407 / 0.419 | +2.93% | 0.499 → 0.494 |
| 7 | 0.722 / 0.695 / 0.675 | -2.82% | 0.717 → 0.733 |
| 8 | 0.413 / 0.365 / 0.357 | -2.15% | 0.378 → 0.369 |
| 9 | 0.729 / 0.660 / 0.669 | +1.32% | 0.701 → 0.679 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.21 | 5.542 | 10.355 | 4.441 | 7.781 |
| 0.4.22 | 5.298 | 9.556 | 4.140 | 7.703 |
| 0.4.23 | 5.054 | 9.620 | 4.031 | 7.641 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.4.23/docs/measurements/0.4.23/wide-confirmation-summary.json).

### Extended broad-query campaign

| Ranker / phase | 0.4.21 median / p95 ms | 0.4.22 median / p95 ms | 0.4.23 median / p95 ms | vs 0.4.22 | vs 0.4.21 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 0.593 / 0.999 | 0.595 / 0.933 | 0.571 / 0.935 | -4.06% | -3.80% |
| tfidf / initial | 0.766 / 0.999 | 0.730 / 0.942 | 0.751 / 0.944 | +2.78% | -2.02% |
| bm25 / repeated | 0.554 / 1.116 | 0.513 / 1.116 | 0.522 / 1.093 | +1.67% | -5.90% |
| tfidf / repeated | 0.737 / 1.006 | 0.699 / 0.962 | 0.698 / 0.946 | -0.11% | -5.30% |

| Query | Repeated TF-IDF median ms: 0.4.21 / 0.4.22 / 0.4.23 | vs 0.4.22 | Initial TF-IDF mean ms: 0.4.22 → 0.4.23 |
| --- | --- | --- | --- |
| 0 | 0.408 / 0.391 / 0.404 | +3.45% | 0.492 → 0.485 |
| 1 | 0.391 / 0.368 / 0.360 | -2.08% | 0.422 → 0.410 |
| 2 | 0.430 / 0.400 / 0.413 | +3.07% | 0.459 → 0.446 |
| 3 | 0.461 / 0.418 / 0.420 | +0.50% | 0.440 → 0.441 |
| 4 | 0.738 / 0.705 / 0.715 | +1.42% | 0.765 → 0.769 |
| 5 | 0.735 / 0.688 / 0.689 | +0.06% | 0.720 → 0.764 |
| 6 | 0.904 / 0.858 / 0.836 | -2.60% | 0.916 → 0.905 |
| 7 | 0.929 / 0.899 / 0.889 | -1.05% | 0.913 → 0.912 |
| 8 | 0.944 / 0.920 / 0.913 | -0.74% | 0.900 → 0.896 |
| 9 | 0.905 / 0.906 / 0.873 | -3.62% | 0.891 → 0.937 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.21 | 7.210 | 13.863 | 4.177 | 8.156 |
| 0.4.22 | 6.992 | 13.261 | 4.362 | 8.000 |
| 0.4.23 | 7.032 | 13.014 | 4.099 | 7.891 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.4.23/docs/measurements/0.4.23/wide-extended-summary.json).

### Existing limit and recovery checks

Three additional fresh servers reject 257/512 known-term requests with both rankers: twelve expected errors, no partial successful results, error progress and successful later health requests. The existing 256-unique-term assessment limit remains; these are not successful-search latency samples. Large inputs remain subject to cooperative bounds, not an OS CPU/RAM quota.

## Actual MCP: two ordinary campaigns

Each ordinary campaign uses nine cases and five alternating old/new pairs: **90 servers each, 180 total**. Canonical knowledge, NumPy and broad synthetic input request progress; 1/4/16-entry inputs run with and without progress. Each makes ten queries with both rankers, two repeated passes, four concurrent searches, matching/relations, rejected arguments and health; snapshot inputs check exact UTF-8 readbacks.

Together **13,880 actual result/error replies validate against advertised schemas**, and **6,940 complete old/new structured pairs agree by RPC ID**, excluding only checked status version and version-qualified schema URNs. Discovery, progress/no-progress, bounds, ordering, input hashes and EOF shutdown pass. Search and readback time are separate. Child peak RSS includes runtime, buffers and source/index memory, unlike allocation profiles. Retain all slower rows.

### First complete campaign

| Case | Ranker | Initial median / p95 ms: old → new | Repeated median / p95 ms: old → new | Repeated median change |
| --- | --- | --- | --- | --- |
| canonical | bm25 | 2.819 / 4.573 → 3.508 / 10.476 | 2.471 / 4.240 → 3.067 / 9.456 | +24.14% |
| canonical | tfidf | 20.117 / 36.413 → 21.879 / 49.231 | 2.627 / 5.271 → 3.045 / 10.541 | +15.92% |
| numpy | bm25 | 0.962 / 1.423 → 0.978 / 1.428 | 0.805 / 1.166 → 0.806 / 1.207 | +0.15% |
| numpy | tfidf | 1.123 / 2.900 → 1.157 / 2.918 | 0.762 / 1.482 → 0.752 / 1.346 | -1.28% |
| wide | bm25 | 0.508 / 0.623 → 0.487 / 0.610 | 0.433 / 0.673 → 0.428 / 0.708 | -0.99% |
| wide | tfidf | 0.472 / 0.783 → 0.473 / 0.772 | 0.430 / 0.726 → 0.418 / 0.734 | -2.74% |
| tiny | bm25 | 0.165 / 0.268 → 0.142 / 0.241 | 0.126 / 0.196 → 0.121 / 0.149 | -4.02% |
| tiny | tfidf | 0.153 / 0.224 → 0.134 / 0.182 | 0.129 / 0.212 → 0.122 / 0.179 | -5.74% |
| tiny4 | bm25 | 0.220 / 0.353 → 0.199 / 0.356 | 0.177 / 0.222 → 0.183 / 0.297 | +2.94% |
| tiny4 | tfidf | 0.209 / 0.284 → 0.197 / 0.263 | 0.173 / 0.217 → 0.177 / 0.264 | +2.40% |
| tiny16 | bm25 | 0.215 / 0.348 → 0.223 / 0.355 | 0.192 / 0.249 → 0.193 / 0.220 | +0.58% |
| tiny16 | tfidf | 0.206 / 0.295 → 0.215 / 0.283 | 0.196 / 0.266 → 0.194 / 0.240 | -0.88% |
| tiny-no-progress | bm25 | 0.075 / 0.177 → 0.078 / 0.173 | 0.065 / 0.089 → 0.066 / 0.089 | +1.15% |
| tiny-no-progress | tfidf | 0.073 / 0.119 → 0.075 / 0.123 | 0.066 / 0.098 → 0.066 / 0.103 | -0.31% |
| tiny4-no-progress | bm25 | 0.137 / 0.257 → 0.134 / 0.254 | 0.122 / 0.158 → 0.117 / 0.151 | -4.46% |
| tiny4-no-progress | tfidf | 0.128 / 0.193 → 0.131 / 0.186 | 0.119 / 0.163 → 0.114 / 0.135 | -3.92% |
| tiny16-no-progress | bm25 | 0.152 / 0.281 → 0.162 / 0.293 | 0.139 / 0.186 → 0.138 / 0.190 | -0.81% |
| tiny16-no-progress | tfidf | 0.146 / 0.202 → 0.152 / 0.212 | 0.141 / 0.176 → 0.137 / 0.170 | -2.42% |

| Case | Startup median ms: old → new | First TF-IDF median ms: old → new | Peak RSS median MiB: old → new |
| --- | --- | --- | --- |
| canonical | 5146.542 → 5479.247 | 23.679 → 25.557 | 1724.531 → 1724.266 |
| numpy | 142.720 → 141.895 | 1.727 → 1.656 | 57.172 → 57.109 |
| wide | 4.085 → 4.162 | 0.484 → 0.494 | 7.672 → 7.609 |
| tiny | 3.424 → 3.696 | 0.208 → 0.185 | 5.766 → 5.719 |
| tiny4 | 3.734 → 3.790 | 0.283 → 0.262 | 5.859 → 5.812 |
| tiny16 | 3.641 → 3.615 | 0.299 → 0.284 | 6.094 → 6.078 |
| tiny-no-progress | 3.383 → 3.370 | 0.117 → 0.122 | 5.781 → 5.703 |
| tiny4-no-progress | 3.556 → 3.399 | 0.176 → 0.187 | 5.828 → 5.875 |
| tiny16-no-progress | 3.570 → 3.707 | 0.204 → 0.215 | 6.047 → 6.031 |

### Second preplanned complete campaign

| Case | Ranker | Initial median / p95 ms: old → new | Repeated median / p95 ms: old → new | Repeated median change |
| --- | --- | --- | --- | --- |
| canonical | bm25 | 2.903 / 4.540 → 2.843 / 4.662 | 2.473 / 4.222 → 2.462 / 4.191 | -0.45% |
| canonical | tfidf | 20.226 / 35.488 → 20.518 / 36.107 | 2.716 / 5.296 → 2.812 / 5.353 | +3.54% |
| numpy | bm25 | 1.037 / 1.449 → 1.087 / 1.603 | 0.902 / 1.361 → 0.905 / 1.366 | +0.32% |
| numpy | tfidf | 1.248 / 3.014 → 1.378 / 3.155 | 0.854 / 1.455 → 0.846 / 1.529 | -0.91% |
| wide | bm25 | 0.489 / 0.608 → 0.518 / 0.714 | 0.429 / 0.699 → 0.479 / 1.014 | +11.78% |
| wide | tfidf | 0.465 / 0.741 → 0.487 / 0.815 | 0.419 / 0.725 → 0.476 / 0.808 | +13.74% |
| tiny | bm25 | 0.166 / 0.277 → 0.207 / 0.325 | 0.126 / 0.241 → 0.143 / 0.263 | +13.87% |
| tiny | tfidf | 0.156 / 0.264 → 0.202 / 0.277 | 0.126 / 0.256 → 0.148 / 0.249 | +16.95% |
| tiny4 | bm25 | 0.212 / 0.319 → 0.207 / 0.316 | 0.172 / 0.238 → 0.173 / 0.231 | +0.65% |
| tiny4 | tfidf | 0.195 / 0.343 → 0.188 / 0.277 | 0.171 / 0.239 → 0.170 / 0.219 | -0.99% |
| tiny16 | bm25 | 0.224 / 0.367 → 0.220 / 0.363 | 0.192 / 0.254 → 0.188 / 0.227 | -2.19% |
| tiny16 | tfidf | 0.211 / 0.311 → 0.216 / 0.275 | 0.195 / 0.252 → 0.188 / 0.250 | -3.54% |
| tiny-no-progress | bm25 | 0.083 / 0.196 → 0.089 / 0.179 | 0.071 / 0.108 → 0.068 / 0.088 | -3.75% |
| tiny-no-progress | tfidf | 0.081 / 0.133 → 0.079 / 0.129 | 0.069 / 0.121 → 0.067 / 0.142 | -3.51% |
| tiny4-no-progress | bm25 | 0.161 / 0.302 → 0.156 / 0.317 | 0.134 / 0.215 → 0.153 / 0.240 | +14.55% |
| tiny4-no-progress | tfidf | 0.151 / 0.244 → 0.145 / 0.219 | 0.130 / 0.205 → 0.139 / 0.239 | +6.21% |
| tiny16-no-progress | bm25 | 0.234 / 0.347 → 0.222 / 0.388 | 0.193 / 0.298 → 0.205 / 0.291 | +6.37% |
| tiny16-no-progress | tfidf | 0.196 / 0.309 → 0.215 / 0.320 | 0.187 / 0.261 → 0.204 / 0.272 | +9.36% |

| Case | Startup median ms: old → new | First TF-IDF median ms: old → new | Peak RSS median MiB: old → new |
| --- | --- | --- | --- |
| canonical | 5382.830 → 5411.419 | 25.831 → 24.391 | 1724.516 → 1724.688 |
| numpy | 145.791 → 156.536 | 1.952 → 1.894 | 57.391 → 57.297 |
| wide | 4.221 → 4.144 | 0.510 → 0.542 | 7.750 → 7.609 |
| tiny | 4.271 → 3.952 | 0.241 → 0.243 | 5.766 → 5.719 |
| tiny4 | 3.658 → 3.744 | 0.270 → 0.262 | 5.859 → 5.859 |
| tiny16 | 3.789 → 3.680 | 0.294 → 0.273 | 6.047 → 6.031 |
| tiny-no-progress | 3.582 → 3.616 | 0.136 → 0.130 | 5.766 → 5.734 |
| tiny4-no-progress | 3.878 → 3.601 | 0.198 → 0.208 | 5.906 → 5.859 |
| tiny16-no-progress | 4.320 → 4.410 | 0.293 → 0.299 | 6.125 → 6.062 |

## Full small-Core target and regression controls

These separate campaigns compare freshly built exact registry 0.4.21 and 0.4.22 with the candidate, using the same programs. Seven rotating triples per campaign run both the original six cases and a 96-case extension. Warm-up, full original-JSON comparison and index setup occur before timers. Each campaign has **42 processes and 2,142,000 timed Core calls**. Both together have 84 processes, 4,284,000 calls and **2,688 complete original matrix-response comparisons**, with identical results. Signature checks also cover every original-six and timed matrix sample. This is repeated warm Core time, without MCP transport or DHAT instrumentation.

Keep the two campaigns separate. Original six-case calls build SearchOptions inside the measured loop; the extension reuses fixed options. Do not compare their absolute times or merge distributions. The 96 cases combine four entry counts, both rankers and twelve fixed query/filter/limit cases. Different-word, wide, punctuated and Unicode queries retain fallback analysis; short canonical repetitions can take the new path. Unknown-only ASCII queries, scope rejection and zero limits are also retained; no case is removed for missing the optimized path. Historical absolute values with different programs/loads are not a reference substitute.

### First small Core campaign

#### Original six warm cases

| Fixture | 0.4.21 median ns | 0.4.22 median ns | 0.4.23 median ns | vs 0.4.22 | vs 0.4.21 |
| --- | --- | --- | --- | --- | --- |
| tiny16_bm25 | 919.125 | 935.458 | 916.959 | -1.98% | -0.24% |
| tiny16_tfidf | 995.708 | 1010.042 | 979.625 | -3.01% | -1.62% |
| tiny4_bm25 | 290.917 | 293.625 | 289.667 | -1.35% | -0.43% |
| tiny4_tfidf | 310.875 | 310.416 | 314.333 | +1.26% | +1.11% |
| tiny_bm25 | 126.083 | 121.875 | 121.042 | -0.68% | -4.00% |
| tiny_tfidf | 125.667 | 124.375 | 125.250 | +0.70% | -0.33% |

Higher candidate medians versus 0.4.22: **2 of 6**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](https://github.com/LEVOGNE/airust/blob/v0.4.23/docs/measurements/0.4.23/small-reference-summary.json). No threshold-based rejection or removal of slow timing samples is used.

#### All 96 extended cases

| Fixture | 0.4.21 median ns | 0.4.22 median ns | 0.4.23 median ns | vs 0.4.22 | vs 0.4.21 |
| --- | --- | --- | --- | --- | --- |
| 16_Bm25_disjoint | 451.458 | 458.167 | 431.666 | -5.78% | -4.38% |
| 16_Bm25_limit_one | 571.667 | 574.750 | 576.625 | +0.33% | +0.87% |
| 16_Bm25_limit_zero | 2.708 | 2.666 | 2.833 | +6.26% | +4.62% |
| 16_Bm25_multi | 1581.375 | 1594.542 | 1627.292 | +2.05% | +2.90% |
| 16_Bm25_punctuation | 143.667 | 142.375 | 147.084 | +3.31% | +2.38% |
| 16_Bm25_repeated | 1173.917 | 1215.500 | 1076.083 | -11.47% | -8.33% |
| 16_Bm25_scope | 956.500 | 956.875 | 957.875 | +0.10% | +0.14% |
| 16_Bm25_single | 1063.625 | 1083.500 | 1062.417 | -1.95% | -0.11% |
| 16_Bm25_unicode | 1336.166 | 1347.000 | 1335.292 | -0.87% | -0.07% |
| 16_Bm25_unknown | 19.834 | 19.791 | 19.666 | -0.63% | -0.85% |
| 16_Bm25_unknown_added | 1152.542 | 1185.625 | 1180.500 | -0.43% | +2.43% |
| 16_Bm25_wide | 16820.375 | 16963.750 | 16877.333 | -0.51% | +0.34% |
| 16_Tfidf_disjoint | 455.292 | 479.458 | 426.125 | -11.12% | -6.41% |
| 16_Tfidf_limit_one | 683.333 | 692.125 | 664.583 | -3.98% | -2.74% |
| 16_Tfidf_limit_zero | 2.709 | 2.709 | 2.792 | +3.06% | +3.06% |
| 16_Tfidf_multi | 1681.916 | 1696.375 | 1664.292 | -1.89% | -1.05% |
| 16_Tfidf_punctuation | 143.958 | 144.583 | 143.584 | -0.69% | -0.26% |
| 16_Tfidf_repeated | 1278.875 | 1299.459 | 1184.875 | -8.82% | -7.35% |
| 16_Tfidf_scope | 1022.416 | 1048.375 | 1011.042 | -3.56% | -1.11% |
| 16_Tfidf_single | 1161.833 | 1209.250 | 1166.542 | -3.53% | +0.41% |
| 16_Tfidf_unicode | 1410.167 | 1447.750 | 1414.208 | -2.32% | +0.29% |
| 16_Tfidf_unknown | 19.750 | 19.792 | 19.458 | -1.69% | -1.48% |
| 16_Tfidf_unknown_added | 1262.333 | 1279.000 | 1248.500 | -2.38% | -1.10% |
| 16_Tfidf_wide | 17111.459 | 17597.583 | 17123.458 | -2.69% | +0.07% |
| 1_Bm25_disjoint | 60.000 | 59.959 | 56.958 | -5.01% | -5.07% |
| 1_Bm25_limit_one | 127.041 | 127.333 | 123.250 | -3.21% | -2.98% |
| 1_Bm25_limit_zero | 2.667 | 2.667 | 2.792 | +4.69% | +4.69% |
| 1_Bm25_multi | 276.917 | 265.625 | 267.084 | +0.55% | -3.55% |
| 1_Bm25_punctuation | 143.833 | 144.375 | 144.125 | -0.17% | +0.20% |
| 1_Bm25_repeated | 265.584 | 272.666 | 153.041 | -43.87% | -42.38% |
| 1_Bm25_scope | 135.417 | 132.250 | 131.833 | -0.32% | -2.65% |
| 1_Bm25_single | 151.958 | 146.542 | 145.333 | -0.83% | -4.36% |
| 1_Bm25_unicode | 435.208 | 424.791 | 425.667 | +0.21% | -2.19% |
| 1_Bm25_unknown | 19.709 | 19.875 | 19.292 | -2.93% | -2.12% |
| 1_Bm25_unknown_added | 230.292 | 226.500 | 252.083 | +11.29% | +9.46% |
| 1_Bm25_wide | 4418.208 | 4383.500 | 4400.083 | +0.38% | -0.41% |
| 1_Tfidf_disjoint | 60.750 | 61.000 | 59.041 | -3.21% | -2.81% |
| 1_Tfidf_limit_one | 134.000 | 132.958 | 134.166 | +0.91% | +0.12% |
| 1_Tfidf_limit_zero | 2.666 | 2.667 | 2.792 | +4.69% | +4.73% |
| 1_Tfidf_multi | 266.334 | 263.250 | 270.250 | +2.66% | +1.47% |
| 1_Tfidf_punctuation | 144.959 | 145.250 | 144.209 | -0.72% | -0.52% |
| 1_Tfidf_repeated | 256.791 | 251.625 | 153.667 | -38.93% | -40.16% |
| 1_Tfidf_scope | 141.208 | 140.125 | 139.958 | -0.12% | -0.89% |
| 1_Tfidf_single | 136.375 | 133.791 | 134.833 | +0.78% | -1.13% |
| 1_Tfidf_unicode | 436.000 | 429.375 | 457.084 | +6.45% | +4.84% |
| 1_Tfidf_unknown | 19.750 | 19.834 | 19.375 | -2.31% | -1.90% |
| 1_Tfidf_unknown_added | 234.209 | 229.417 | 237.500 | +3.52% | +1.41% |
| 1_Tfidf_wide | 4552.500 | 4419.500 | 4349.125 | -1.59% | -4.47% |
| 33_Bm25_disjoint | 997.958 | 989.167 | 893.959 | -9.63% | -10.42% |
| 33_Bm25_limit_one | 1190.791 | 1185.250 | 1158.417 | -2.26% | -2.72% |
| 33_Bm25_limit_zero | 2.667 | 2.625 | 2.750 | +4.76% | +3.11% |
| 33_Bm25_multi | 2399.542 | 2473.208 | 2448.250 | -1.01% | +2.03% |
| 33_Bm25_punctuation | 145.708 | 143.625 | 146.500 | +2.00% | +0.54% |
| 33_Bm25_repeated | 1782.625 | 1798.375 | 1712.000 | -4.80% | -3.96% |
| 33_Bm25_scope | 1772.916 | 1781.541 | 1712.000 | -3.90% | -3.44% |
| 33_Bm25_single | 1642.083 | 1675.041 | 1622.208 | -3.15% | -1.21% |
| 33_Bm25_unicode | 1913.083 | 1892.292 | 1903.750 | +0.61% | -0.49% |
| 33_Bm25_unknown | 19.958 | 19.708 | 19.583 | -0.63% | -1.88% |
| 33_Bm25_unknown_added | 1746.292 | 1802.250 | 1793.500 | -0.49% | +2.70% |
| 33_Bm25_wide | 26849.042 | 26344.792 | 26263.750 | -0.31% | -2.18% |
| 33_Tfidf_disjoint | 963.958 | 972.500 | 902.875 | -7.16% | -6.34% |
| 33_Tfidf_limit_one | 1367.625 | 1365.792 | 1295.458 | -5.15% | -5.28% |
| 33_Tfidf_limit_zero | 2.667 | 2.667 | 2.791 | +4.65% | +4.65% |
| 33_Tfidf_multi | 2651.583 | 2687.875 | 2613.375 | -2.77% | -1.44% |
| 33_Tfidf_punctuation | 146.875 | 144.959 | 144.834 | -0.09% | -1.39% |
| 33_Tfidf_repeated | 1972.291 | 1979.167 | 1850.417 | -6.51% | -6.18% |
| 33_Tfidf_scope | 1833.500 | 1896.084 | 1810.583 | -4.51% | -1.25% |
| 33_Tfidf_single | 1861.958 | 1878.667 | 1837.250 | -2.20% | -1.33% |
| 33_Tfidf_unicode | 2040.500 | 2041.000 | 2026.458 | -0.71% | -0.69% |
| 33_Tfidf_unknown | 19.875 | 19.583 | 19.458 | -0.64% | -2.10% |
| 33_Tfidf_unknown_added | 1944.416 | 1961.750 | 1939.416 | -1.14% | -0.26% |
| 33_Tfidf_wide | 26987.375 | 26511.292 | 26612.875 | +0.38% | -1.39% |
| 4_Bm25_disjoint | 112.542 | 111.750 | 105.041 | -6.00% | -6.67% |
| 4_Bm25_limit_one | 196.542 | 188.916 | 180.459 | -4.48% | -8.18% |
| 4_Bm25_limit_zero | 2.625 | 2.667 | 2.834 | +6.26% | +7.96% |
| 4_Bm25_multi | 551.250 | 544.708 | 546.333 | +0.30% | -0.89% |
| 4_Bm25_punctuation | 145.458 | 143.917 | 145.042 | +0.78% | -0.29% |
| 4_Bm25_repeated | 437.625 | 436.875 | 336.458 | -22.99% | -23.12% |
| 4_Bm25_scope | 233.959 | 237.916 | 230.875 | -2.96% | -1.32% |
| 4_Bm25_single | 317.000 | 324.958 | 314.333 | -3.27% | -0.84% |
| 4_Bm25_unicode | 613.666 | 597.584 | 599.834 | +0.38% | -2.25% |
| 4_Bm25_unknown | 19.833 | 19.875 | 19.417 | -2.30% | -2.10% |
| 4_Bm25_unknown_added | 426.292 | 418.750 | 416.125 | -0.63% | -2.38% |
| 4_Bm25_wide | 7344.125 | 7376.708 | 7254.959 | -1.65% | -1.21% |
| 4_Tfidf_disjoint | 109.833 | 112.458 | 105.875 | -5.85% | -3.60% |
| 4_Tfidf_limit_one | 210.500 | 216.125 | 214.167 | -0.91% | +1.74% |
| 4_Tfidf_limit_zero | 2.667 | 2.709 | 3.667 | +35.36% | +37.50% |
| 4_Tfidf_multi | 569.625 | 573.584 | 572.208 | -0.24% | +0.45% |
| 4_Tfidf_punctuation | 147.792 | 145.125 | 144.250 | -0.60% | -2.40% |
| 4_Tfidf_repeated | 454.250 | 454.916 | 360.042 | -20.86% | -20.74% |
| 4_Tfidf_scope | 243.792 | 246.416 | 239.667 | -2.74% | -1.69% |
| 4_Tfidf_single | 339.625 | 344.833 | 343.792 | -0.30% | +1.23% |
| 4_Tfidf_unicode | 636.833 | 638.542 | 632.583 | -0.93% | -0.67% |
| 4_Tfidf_unknown | 19.583 | 19.833 | 19.292 | -2.73% | -1.49% |
| 4_Tfidf_unknown_added | 439.292 | 445.375 | 440.375 | -1.12% | +0.25% |
| 4_Tfidf_wide | 7301.500 | 7352.375 | 7217.875 | -1.83% | -1.15% |

Higher candidate medians versus 0.4.22: **27 of 96**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](https://github.com/LEVOGNE/airust/blob/v0.4.23/docs/measurements/0.4.23/small-reference-summary.json). No threshold-based rejection or removal of slow timing samples is used.

### Second preplanned small Core campaign

#### Original six warm cases

| Fixture | 0.4.21 median ns | 0.4.22 median ns | 0.4.23 median ns | vs 0.4.22 | vs 0.4.21 |
| --- | --- | --- | --- | --- | --- |
| tiny16_bm25 | 1155.833 | 1155.792 | 1140.458 | -1.33% | -1.33% |
| tiny16_tfidf | 1397.042 | 1392.208 | 1371.125 | -1.51% | -1.86% |
| tiny4_bm25 | 368.875 | 370.959 | 374.792 | +1.03% | +1.60% |
| tiny4_tfidf | 430.083 | 426.000 | 434.416 | +1.98% | +1.01% |
| tiny_bm25 | 168.667 | 164.417 | 165.542 | +0.68% | -1.85% |
| tiny_tfidf | 171.416 | 173.500 | 175.750 | +1.30% | +2.53% |

Higher candidate medians versus 0.4.22: **4 of 6**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](https://github.com/LEVOGNE/airust/blob/v0.4.23/docs/measurements/0.4.23/small-confirmation-summary.json). No threshold-based rejection or removal of slow timing samples is used.

#### All 96 extended cases

| Fixture | 0.4.21 median ns | 0.4.22 median ns | 0.4.23 median ns | vs 0.4.22 | vs 0.4.21 |
| --- | --- | --- | --- | --- | --- |
| 16_Bm25_disjoint | 493.208 | 506.541 | 472.833 | -6.65% | -4.13% |
| 16_Bm25_limit_one | 621.208 | 648.667 | 667.250 | +2.86% | +7.41% |
| 16_Bm25_limit_zero | 3.958 | 3.000 | 4.167 | +38.90% | +5.28% |
| 16_Bm25_multi | 1636.666 | 1685.791 | 1953.458 | +15.88% | +19.36% |
| 16_Bm25_punctuation | 163.250 | 157.375 | 155.792 | -1.01% | -4.57% |
| 16_Bm25_repeated | 1258.458 | 1248.375 | 1355.167 | +8.55% | +7.68% |
| 16_Bm25_scope | 1066.959 | 1034.250 | 998.209 | -3.48% | -6.44% |
| 16_Bm25_single | 1100.750 | 1129.708 | 1325.291 | +17.31% | +20.40% |
| 16_Bm25_unicode | 1384.916 | 1438.125 | 1510.167 | +5.01% | +9.04% |
| 16_Bm25_unknown | 24.042 | 22.375 | 21.209 | -5.21% | -11.78% |
| 16_Bm25_unknown_added | 1238.500 | 1252.750 | 1448.666 | +15.64% | +16.97% |
| 16_Bm25_wide | 19251.042 | 18665.417 | 20132.292 | +7.86% | +4.58% |
| 16_Tfidf_disjoint | 500.041 | 488.583 | 478.125 | -2.14% | -4.38% |
| 16_Tfidf_limit_one | 743.875 | 721.041 | 740.709 | +2.73% | -0.43% |
| 16_Tfidf_limit_zero | 2.959 | 2.833 | 3.000 | +5.89% | +1.39% |
| 16_Tfidf_multi | 2212.417 | 2232.500 | 2171.291 | -2.74% | -1.86% |
| 16_Tfidf_punctuation | 164.417 | 145.292 | 166.625 | +14.68% | +1.34% |
| 16_Tfidf_repeated | 1421.292 | 1750.000 | 1597.875 | -8.69% | +12.42% |
| 16_Tfidf_scope | 1146.583 | 1100.541 | 1266.417 | +15.07% | +10.45% |
| 16_Tfidf_single | 1269.208 | 1243.750 | 1577.500 | +26.83% | +24.29% |
| 16_Tfidf_unicode | 1928.833 | 1654.500 | 1911.708 | +15.55% | -0.89% |
| 16_Tfidf_unknown | 23.917 | 22.167 | 24.084 | +8.65% | +0.70% |
| 16_Tfidf_unknown_added | 1411.417 | 1732.916 | 1704.708 | -1.63% | +20.78% |
| 16_Tfidf_wide | 19274.458 | 19692.709 | 20264.542 | +2.90% | +5.14% |
| 1_Bm25_disjoint | 82.750 | 80.208 | 79.209 | -1.25% | -4.28% |
| 1_Bm25_limit_one | 169.292 | 170.291 | 166.500 | -2.23% | -1.65% |
| 1_Bm25_limit_zero | 3.292 | 3.000 | 3.875 | +29.17% | +17.71% |
| 1_Bm25_multi | 333.458 | 329.917 | 355.375 | +7.72% | +6.57% |
| 1_Bm25_punctuation | 165.084 | 164.375 | 170.125 | +3.50% | +3.05% |
| 1_Bm25_repeated | 347.583 | 330.708 | 202.500 | -38.77% | -41.74% |
| 1_Bm25_scope | 178.750 | 180.542 | 176.916 | -2.01% | -1.03% |
| 1_Bm25_single | 193.375 | 192.417 | 185.875 | -3.40% | -3.88% |
| 1_Bm25_unicode | 563.083 | 546.208 | 565.125 | +3.46% | +0.36% |
| 1_Bm25_unknown | 23.667 | 23.750 | 23.750 | +0.00% | +0.35% |
| 1_Bm25_unknown_added | 298.208 | 288.625 | 341.958 | +18.48% | +14.67% |
| 1_Bm25_wide | 5249.416 | 5253.959 | 5335.000 | +1.54% | +1.63% |
| 1_Tfidf_disjoint | 78.291 | 79.958 | 64.459 | -19.38% | -17.67% |
| 1_Tfidf_limit_one | 181.375 | 182.250 | 143.041 | -21.51% | -21.14% |
| 1_Tfidf_limit_zero | 3.541 | 3.000 | 2.917 | -2.77% | -17.62% |
| 1_Tfidf_multi | 351.000 | 285.083 | 287.167 | +0.73% | -18.19% |
| 1_Tfidf_punctuation | 163.625 | 156.667 | 160.375 | +2.37% | -1.99% |
| 1_Tfidf_repeated | 328.083 | 497.333 | 213.125 | -57.15% | -35.04% |
| 1_Tfidf_scope | 192.917 | 191.250 | 148.666 | -22.27% | -22.94% |
| 1_Tfidf_single | 188.625 | 186.292 | 182.916 | -1.81% | -3.03% |
| 1_Tfidf_unicode | 550.750 | 471.834 | 463.291 | -1.81% | -15.88% |
| 1_Tfidf_unknown | 24.042 | 22.541 | 20.750 | -7.95% | -13.69% |
| 1_Tfidf_unknown_added | 316.375 | 276.708 | 312.417 | +12.90% | -1.25% |
| 1_Tfidf_wide | 5230.625 | 4596.750 | 4850.125 | +5.51% | -7.27% |
| 33_Bm25_disjoint | 1049.000 | 1066.958 | 985.708 | -7.62% | -6.03% |
| 33_Bm25_limit_one | 1342.125 | 1408.417 | 1346.750 | -4.38% | +0.34% |
| 33_Bm25_limit_zero | 3.000 | 3.042 | 3.083 | +1.35% | +2.77% |
| 33_Bm25_multi | 2602.375 | 2583.625 | 2920.750 | +13.05% | +12.23% |
| 33_Bm25_punctuation | 155.209 | 156.208 | 156.167 | -0.03% | +0.62% |
| 33_Bm25_repeated | 1871.708 | 1955.709 | 1777.917 | -9.09% | -5.01% |
| 33_Bm25_scope | 1958.291 | 2113.916 | 1989.458 | -5.89% | +1.59% |
| 33_Bm25_single | 1734.667 | 1888.292 | 1698.334 | -10.06% | -2.09% |
| 33_Bm25_unicode | 2055.041 | 2397.125 | 2018.208 | -15.81% | -1.79% |
| 33_Bm25_unknown | 21.375 | 23.750 | 20.959 | -11.75% | -1.95% |
| 33_Bm25_unknown_added | 1892.916 | 1920.166 | 2118.875 | +10.35% | +11.94% |
| 33_Bm25_wide | 31847.083 | 32126.542 | 31500.208 | -1.95% | -1.09% |
| 33_Tfidf_disjoint | 1079.166 | 1046.791 | 980.209 | -6.36% | -9.17% |
| 33_Tfidf_limit_one | 1439.917 | 1483.500 | 1393.417 | -6.07% | -3.23% |
| 33_Tfidf_limit_zero | 3.000 | 3.042 | 4.125 | +35.60% | +37.50% |
| 33_Tfidf_multi | 2808.958 | 2878.750 | 2827.167 | -1.79% | +0.65% |
| 33_Tfidf_punctuation | 160.625 | 155.459 | 154.083 | -0.89% | -4.07% |
| 33_Tfidf_repeated | 2058.500 | 2150.125 | 1933.709 | -10.07% | -6.06% |
| 33_Tfidf_scope | 2047.666 | 2116.708 | 1912.708 | -9.64% | -6.59% |
| 33_Tfidf_single | 1931.333 | 2066.584 | 1905.375 | -7.80% | -1.34% |
| 33_Tfidf_unicode | 2181.959 | 2245.500 | 2168.875 | -3.41% | -0.60% |
| 33_Tfidf_unknown | 22.208 | 22.667 | 20.917 | -7.72% | -5.81% |
| 33_Tfidf_unknown_added | 2044.709 | 2129.958 | 2042.542 | -4.10% | -0.11% |
| 33_Tfidf_wide | 31269.375 | 32708.833 | 28498.708 | -12.87% | -8.86% |
| 4_Bm25_disjoint | 139.084 | 118.584 | 115.875 | -2.28% | -16.69% |
| 4_Bm25_limit_one | 242.709 | 201.375 | 235.125 | +16.76% | -3.12% |
| 4_Bm25_limit_zero | 3.791 | 2.834 | 4.125 | +45.55% | +8.81% |
| 4_Bm25_multi | 683.250 | 573.917 | 604.208 | +5.28% | -11.57% |
| 4_Bm25_punctuation | 164.958 | 153.292 | 155.125 | +1.20% | -5.96% |
| 4_Bm25_repeated | 546.625 | 441.459 | 370.458 | -16.08% | -32.23% |
| 4_Bm25_scope | 300.083 | 247.291 | 250.083 | +1.13% | -16.66% |
| 4_Bm25_single | 399.000 | 324.459 | 346.292 | +6.73% | -13.21% |
| 4_Bm25_unicode | 772.500 | 617.500 | 652.375 | +5.65% | -15.55% |
| 4_Bm25_unknown | 23.583 | 22.042 | 20.833 | -5.48% | -11.66% |
| 4_Bm25_unknown_added | 528.167 | 431.000 | 453.042 | +5.11% | -14.22% |
| 4_Bm25_wide | 8463.291 | 8896.167 | 9053.417 | +1.77% | +6.97% |
| 4_Tfidf_disjoint | 116.542 | 121.750 | 118.666 | -2.53% | +1.82% |
| 4_Tfidf_limit_one | 219.917 | 229.041 | 237.209 | +3.57% | +7.86% |
| 4_Tfidf_limit_zero | 3.791 | 2.958 | 3.875 | +31.00% | +2.22% |
| 4_Tfidf_multi | 664.417 | 605.500 | 652.625 | +7.78% | -1.77% |
| 4_Tfidf_punctuation | 152.958 | 157.375 | 160.000 | +1.67% | +4.60% |
| 4_Tfidf_repeated | 531.166 | 622.500 | 394.167 | -36.68% | -25.79% |
| 4_Tfidf_scope | 255.042 | 259.500 | 270.208 | +4.13% | +5.95% |
| 4_Tfidf_single | 370.667 | 482.042 | 413.709 | -14.18% | +11.61% |
| 4_Tfidf_unicode | 672.500 | 687.459 | 725.375 | +5.52% | +7.86% |
| 4_Tfidf_unknown | 22.958 | 24.417 | 21.834 | -10.58% | -4.90% |
| 4_Tfidf_unknown_added | 515.291 | 605.625 | 495.500 | -18.18% | -3.84% |
| 4_Tfidf_wide | 7743.667 | 8068.666 | 8415.333 | +4.30% | +8.67% |

Higher candidate medians versus 0.4.22: **46 of 96**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](https://github.com/LEVOGNE/airust/blob/v0.4.23/docs/measurements/0.4.23/small-confirmation-summary.json). No threshold-based rejection or removal of slow timing samples is used.

## Separate Core development trials

Both trials retain seven rotating triples with the original six and all 96 cases, identical full compared results and signatures, and separate source/worker identities. They are excluded from final timing distributions. The initial trial retains its original version-key display error, corrected derived summary and unchanged raw samples.

### Initial repeated-word recognition

| Repeated fixture | 0.4.21 median ns | 0.4.22 median ns | Candidate median ns | vs 0.4.22 |
| --- | --- | --- | --- | --- |
| 16_Bm25_repeated | 1255.000 | 1234.833 | 1136.375 | -7.97% |
| 16_Tfidf_repeated | 1332.833 | 1343.834 | 1248.125 | -7.12% |
| 1_Bm25_repeated | 284.416 | 298.500 | 161.500 | -45.90% |
| 1_Tfidf_repeated | 262.000 | 260.833 | 158.917 | -39.07% |
| 33_Bm25_repeated | 1880.000 | 1848.958 | 1803.250 | -2.47% |
| 33_Tfidf_repeated | 2025.208 | 1966.208 | 2027.208 | +3.10% |
| 4_Bm25_repeated | 444.833 | 439.291 | 347.791 | -20.83% |
| 4_Tfidf_repeated | 469.500 | 468.542 | 384.625 | -17.91% |

Largest non-target rise: **+41.76% (33_Bm25_limit_zero)**. All 96 positive/negative cases remain in [the complete development summary](https://github.com/LEVOGNE/airust/blob/v0.4.23/docs/measurements/0.4.23/development-pilot-repeats-summary.json). This is not a final or independent holdout result.

### Selected short-circuit fallback

| Repeated fixture | 0.4.21 median ns | 0.4.22 median ns | Candidate median ns | vs 0.4.22 |
| --- | --- | --- | --- | --- |
| 16_Bm25_repeated | 1206.292 | 1194.833 | 1103.667 | -7.63% |
| 16_Tfidf_repeated | 1297.584 | 1329.750 | 1228.375 | -7.62% |
| 1_Bm25_repeated | 255.917 | 271.042 | 168.625 | -37.79% |
| 1_Tfidf_repeated | 256.458 | 251.166 | 161.125 | -35.85% |
| 33_Bm25_repeated | 1859.584 | 1860.375 | 1703.875 | -8.41% |
| 33_Tfidf_repeated | 1940.916 | 1936.334 | 1885.875 | -2.61% |
| 4_Bm25_repeated | 434.667 | 422.291 | 345.250 | -18.24% |
| 4_Tfidf_repeated | 460.666 | 453.375 | 371.709 | -18.01% |

Largest non-target rise: **+21.00% (1_Bm25_unknown_added)**. All 96 positive/negative cases remain in [the complete development summary](https://github.com/LEVOGNE/airust/blob/v0.4.23/docs/measurements/0.4.23/development-pilot-repeats-shortcircuit-summary.json). This is not a final or independent holdout result.

## Direct canonical search with explicit control modes

Fourteen processes (seven alternating pairs) retain one warmed Engine each. Ten rounds rotate plain, deadline and progress within each query/ranker pair: **8,400 timed Retriever calls**, excluding loading/readiness, stdio and profiling. Each progress request includes an observer and the same deadline/cancellation setup. Every signature agrees; **140 complete original response JSON pairs match**, and control modes also match plain output. Full readiness source text stays private. These are direct retrieval boundaries, not MCP round trips or capacity evidence.

| Control | Ranker | 0.4.22 median / p95 ms | 0.4.23 median / p95 ms | Median change |
| --- | --- | --- | --- | --- |
| plain | Bm25 | 1.585 / 4.358 | 1.533 / 3.598 | -3.26% |
| plain | Tfidf | 1.793 / 5.762 | 1.623 / 5.155 | -9.47% |
| deadline | Bm25 | 1.900 / 5.426 | 1.801 / 4.543 | -5.17% |
| deadline | Tfidf | 2.124 / 6.962 | 1.966 / 5.807 | -7.43% |
| progress | Bm25 | 1.914 / 5.291 | 1.807 / 4.463 | -5.56% |
| progress | Tfidf | 2.143 / 7.210 | 1.994 / 6.451 | -6.97% |

| Query | Candidates | Progress BM25 ms: old → new | Progress TF-IDF ms: old → new |
| --- | --- | --- | --- |
| 0 | 12365 | 1.856 → 1.750 | 2.110 → 2.017 |
| 1 | 13864 | 1.988 → 1.837 | 2.261 → 2.028 |
| 2 | 24299 | 3.483 → 3.275 | 4.222 → 3.538 |
| 3 | 596 | 0.458 → 0.341 | 0.338 → 0.325 |
| 4 | 15403 | 2.077 → 1.890 | 2.299 → 2.100 |
| 5 | 24140 | 3.325 → 3.031 | 3.810 → 3.328 |
| 6 | 3358 | 0.852 → 0.732 | 0.815 → 0.765 |
| 7 | 29600 | 3.226 → 3.049 | 3.930 → 3.689 |
| 8 | 11504 | 1.795 → 1.719 | 1.888 → 1.805 |
| 9 | 0 | 0.002 → 0.002 | 0.001 → 0.001 |

## Separate canonical three-version reference

Five rotating triples compare fresh exact 0.4.21/0.4.22/candidate servers on canonical knowledge with requested progress. Fifteen servers validate **1,020 actual schema replies**; **680 complete structured comparisons** against the candidate match by RPC ID except checked status version and version-qualified schema URNs. Schemas, source ranges and inputs agree. This control is separate from the two ordinary campaigns. It checks whether the previous large-query benefit remains visible against the older release; it does not establish the cause of any historical regression.

| Version | Ranker | Initial median / p95 ms | Repeated median / p95 ms | Peak child RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.21 | bm25 | 3.125 / 5.098 | 2.849 / 4.939 | 1724.625 |
| 0.4.21 | tfidf | 23.022 / 40.736 | 3.656 / 6.226 | 1724.625 |
| 0.4.22 | bm25 | 2.985 / 5.066 | 2.580 / 4.650 | 1724.375 |
| 0.4.22 | tfidf | 22.270 / 35.409 | 3.138 / 5.907 | 1724.375 |
| 0.4.23 | bm25 | 3.571 / 8.091 | 3.344 / 7.902 | 1724.812 |
| 0.4.23 | tfidf | 24.834 / 43.738 | 4.078 / 9.878 | 1724.812 |

| Query | 0.4.21 repeated TF-IDF ms | 0.4.22 repeated TF-IDF ms | 0.4.23 repeated TF-IDF ms |
| --- | --- | --- | --- |
| 0 | 3.735 | 3.150 | 4.526 |
| 1 | 3.656 | 3.126 | 4.078 |
| 2 | 5.815 | 5.257 | 6.399 |
| 3 | 0.957 | 0.853 | 1.130 |
| 4 | 3.984 | 3.244 | 4.282 |
| 5 | 5.414 | 4.785 | 6.027 |
| 6 | 1.750 | 1.354 | 1.828 |
| 7 | 5.820 | 4.958 | 7.146 |
| 8 | 3.265 | 2.735 | 3.986 |
| 9 | 0.163 | 0.122 | 0.164 |

## Canonical first-pass heap traffic

Two instrumented processes index all 167,942 records before profiling the same ten initial TF-IDF queries, then drop results before net deltas. The existing direct candidate collection remains; the previous large-query posting cursors and norm memo/cache still exist. Heap traffic, net persistent hints, stack state and whole-process RSS are different costs. Instrumented durations are excluded from latency comparisons.

| Metric | 0.4.22 | 0.4.23 |
| --- | --- | --- |
| Retained hint bytes | 2228312 | 2228312 |
| Retained hint blocks | 2 | 2 |
| Total query allocated bytes | 7213183 | 7212357 |
| Total query allocation blocks | 409 | 401 |

## Development trials and host-load limits

The full three-triple development MCP pilot is retained separately from all final campaigns. Its source snapshots include the MCP implementation and Core; all complete compared structured results agree. Initial/repeated data stay separate and no final sample is replaced by a pilot or a faster rerun. Earlier releases retain their own rejected trials. These are development observations, not fresh production or semantic holdouts.

Two-second monitoring retains **591 snapshots**, including **423 with other rustc processes**, maximum 16. All managed compilers finish before timing. Other workloads are not stopped, and snapshots do not prove continuous idleness or identify a timing cause. CPU counts include the active benchmark workers.

| Stage start | Other rustc count | Other rustc CPU total | Processes over 50% CPU |
| --- | --- | --- | --- |
| small-mcp-standard | 2 | 1217.800 | 3 |
| small-reference | 0 | 0.000 | 1 |
| large-standard | 0 | 0.000 | 2 |
| wide-standard | 2 | 185.800 | 4 |
| retrieval | 3 | 292.700 | 5 |
| query | 0 | 0.000 | 2 |
| build | 3 | 1317.200 | 5 |
| controlled | 0 | 0.000 | 3 |
| mcp | 2 | 1338.000 | 3 |
| corpus | 0 | 0.000 | 3 |
| mcp-confirmation | 0 | 0.000 | 3 |
| reference | 0 | 0.000 | 4 |
| wide-confirmation | 0 | 0.000 | 2 |
| wide-extended | 0 | 0.000 | 2 |
| large-confirmation | 0 | 0.000 | 2 |
| small-confirmation | 2 | 199.400 | 6 |
| small-mcp-confirmation | 2 | 1382.300 | 4 |
| limits | 0 | 0.000 | 3 |

All observed slower cases remain in their own distributions; no sample is replaced because of load. One machine, randomized hashing, exposed questions, tuned fixtures and finite trials limit generalization.

## Retained Core controls

These controls retain positive and negative timing changes. Single known-term requests retain the direct candidate path and exactly two retain ordered merging. Wider requests keep the heap/set strategies; large frequency cursors remain available afterward. Large sorted candidate sets can use the bounded document cursor after 128 candidates; earlier shared explanation intersection remains. Read every row independently, including slower cases. Seven alternating pairs give fourteen plain processes per family. Norms are initialized for warm timing; two separate DHAT processes measure allocations. Instrumented durations are excluded from latency comparisons. Every result signature agrees and temporary-call retained deltas are zero.

### Small warm Core: 84,000 calls on 1/4/16 entries

| Fixture | Median change | Blocks/call: old → new | Allocated bytes/call: old → new |
| --- | --- | --- | --- |
| tiny16_bm25 | -1.94% | 26.0 → 26.0 | 2156.0 → 2156.0 |
| tiny16_tfidf | -1.75% | 26.0 → 26.0 | 2156.0 → 2156.0 |
| tiny4_bm25 | +0.34% | 12.0 → 12.0 | 824.0 → 824.0 |
| tiny4_tfidf | +0.24% | 12.0 → 12.0 | 824.0 → 824.0 |
| tiny_bm25 | -2.49% | 6.0 → 6.0 | 398.0 → 398.0 |
| tiny_tfidf | -0.36% | 6.0 → 6.0 | 398.0 → 398.0 |

### All 25 warm controls: 282,800 calls

| Fixture | Median change | Blocks/call: old → new | Allocated bytes/call: old → new |
| --- | --- | --- | --- |
| coverage_repeated | -0.38% | 1047.0 → 1047.0 | 92942.0 → 92942.0 |
| coverage_small | -0.98% | 784.0 → 784.0 | 78046.0 → 78046.0 |
| excerpt_ascii | +1.47% | 762.0 → 762.0 | 252438.0 → 252438.0 |
| excerpt_mixed | +0.44% | 569.0 → 569.0 | 417113.0 → 417113.0 |
| excerpt_unicode | +2.02% | 197.0 → 197.0 | 72254.0 → 72254.0 |
| long_repeated | +7.79% | 4.0 → 4.0 | 18798.0 → 18798.0 |
| mixed_unicode | -5.53% | 7.0 → 7.0 | 3155.0 → 3155.0 |
| punctuation | -11.68% | 1.0 → 1.0 | 768.0 → 768.0 |
| relevant_retrieval | -5.15% | 812.0 → 812.0 | 93562.0 → 93562.0 |
| repeated_search | -5.44% | 35.0 → 31.0 | 19432.0 → 18156.0 |
| repeated_short | +12.86% | 4.0 → 4.0 | 1656.0 → 1656.0 |
| sparse_bm25_128 | +5.33% | 359.0 → 359.0 | 29074.0 → 29074.0 |
| sparse_bm25_33 | -0.46% | 241.0 → 241.0 | 16309.0 → 16309.0 |
| sparse_bm25_512 | +1.86% | 1337.0 → 1337.0 | 119124.0 → 119124.0 |
| sparse_tfidf_128 | +9.82% | 474.0 → 474.0 | 35140.0 → 35140.0 |
| sparse_tfidf_33 | +8.07% | 241.0 → 241.0 | 16309.0 → 16309.0 |
| sparse_tfidf_512 | -7.54% | 1337.0 → 1337.0 | 119124.0 → 119124.0 |
| tfidf_union_32 | +3.94% | 244.0 → 244.0 | 19834.0 → 19834.0 |
| tfidf_union_33 | -0.36% | 296.0 → 296.0 | 27854.0 → 27854.0 |
| tfidf_warm | +6.52% | 71.0 → 71.0 | 19335.0 → 19335.0 |
| union_32 | +9.60% | 244.0 → 244.0 | 19834.0 → 19834.0 |
| union_33 | +16.12% | 296.0 → 296.0 | 27854.0 → 27854.0 |
| unique_long | -0.19% | 299.0 → 299.0 | 32672.0 → 32672.0 |
| unique_search | -0.55% | 72.0 → 72.0 | 19350.0 → 19350.0 |
| unique_short | +3.29% | 296.0 → 296.0 | 18548.0 → 18548.0 |

### Cold norm and heap controls

Six fixtures make twenty fresh-index-clone calls each across fourteen processes: 1,680 calls. Clone/setup is outside timing. Process/instruction/filesystem caches remain; cold means norm hints, not disks. Separate heap runs drop results before net retained measurements. The existing inline/optional heap word memo and persistent norm storage remain. Large small-term requests may add up to four temporary posting cursors; canonical allocation traffic below records that cost. Heap profiles exclude stack state/control fields and temporary frames.

| Fixture | Old median / p95 ns | New median / p95 ns |
| --- | --- | --- |
| index | 119271.000 / 148416.050 | 119000.000 / 137130.900 |
| wide | 36020.500 / 73885.400 | 38708.500 / 70270.850 |
| sparse | 446937.500 / 534666.950 | 452750.000 / 546366.650 |
| tiny | 500.000 / 2127.100 | 583.000 / 2254.200 |
| tiny4 | 750.000 / 1340.200 | 834.000 / 1670.200 |
| tiny16 | 2625.000 / 4381.250 | 2542.000 / 4173.200 |

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
| new_group_query_ns | 290250.000 / 370329.500 | 298792.000 / 390139.550 |
| post_readers_first_ns | 195709.000 / 235325.400 | 229083.000 / 272225.200 |
| post_readers_second_ns | 94083.000 / 118657.800 | 127250.000 / 134558.600 |

The private document corpus retains 72 Markdown files, 2,755,468 bytes and 1,910 chunks. All 342 authored questions (276 positive, 66 unsupported) are exposed regressions, not fresh holdouts or logged production queries. Literal BM25, prepared BM25 and coverage retain six passages, 100 candidates, 2,000 bytes/pass and 12,000 total bytes. All **1,026 complete original response JSON strings match byte-for-byte**, including scores, ordering, evidence, source text and positions. One warm-up and five repeats/question/configuration/version give **10,260 timed round trips**. These three configurations are BM25 controls; independent score oracles and actual protocol pairs cover TF-IDF. No new relevance judgment or model reply is produced.

All document timing aggregates and per-question results remain in the records. Construction uses forty plain processes on canonical data and three synthetic distributions, plus six separate profiles. Parsing/generation is outside construction timing; stats/signatures/live derived heap agree. The execution/shared-index worker checks all 167,942 entries through 128 changes/restoration, rebuilds, JSON round trips, retained clones, parallel readers and cancellation/deadline rejection. Portable exports and source hashes agree.

### Document round trips

| Case | Old median / p95 ms | New median / p95 ms |
| --- | --- | --- |
| baseline | 0.854 / 1.356 | 0.848 / 1.342 |
| prepared | 0.965 / 1.533 | 0.956 / 1.501 |
| evidence | 0.959 / 1.505 | 0.947 / 1.514 |

### Construction only

| Case | Old median / p95 ms | New median / p95 ms |
| --- | --- | --- |
| canonical | 6003.550 / 6919.640 | 5876.926 / 7491.761 |
| ascii | 16.086 / 16.925 | 15.431 / 18.282 |
| mixed | 38.469 / 39.108 | 35.986 / 44.110 |
| long_tokens | 1.787 / 2.092 | 1.751 / 2.106 |

## Limits and preservation

Measurements use one AC-powered Apple M5 Max with 64 GB, Rust 1.98.0 and Python 3.15.2. Stage-start CPU/compiler snapshots remain. Own compilers finish before timing; this does not establish continuously idle hardware. Randomized hashing, scheduling, one machine/corpus, exposed questions and finite process counts limit generalization. The separate development pilot remains; assess final per-case medians and tails independently. Host-load sampling records observations rather than continuous idleness. No universal latency, total-memory, semantic-quality, token-saving or security-audit claim is made.

Canonical knowledge remains 167,942 entries from 35 PDFs and 49,086 physical pages, SHA-256 `13168d87872a137355fd8499c90a6084f52243a2ec7456f29b5fcb2a1c7423c3`. The PDF parser is unchanged; this is not a fresh large-PDF extraction campaign. Lexical overlap and exact quotes do not prove factual support or entailment. No model, daemon or DenkMaps integration is added.

Preserve PDFs, knowledge, snapshots, all rejected/selected pilot and final measurements, slower results, raw transcripts, profiles, exact small executables and locks. Raw measurement JSON stays in the tagged GitHub repository and local archives, outside the registry source package; API guides, Markdown reports, method source, examples and required fixtures remain in that package. After verified publication archive package/proof, then delete only the named temporary Cargo/Miri targets. Default features remain empty.
