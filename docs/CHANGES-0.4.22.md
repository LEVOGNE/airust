# AIRust 0.4.22: less duplicate work preparing bounded MCP replies

A worker result now carries a private immutable witness that its data fits the configured byte budget. Response preparation reuses that check when the budget is unchanged and moves the owned JSON data into the envelope. It avoids an extra walk of the same payload and an intermediate deep JSON copy. A different budget triggers a new check. The final encoded reply is still independently counted, including escaped text and duplicated structured content. Oversized replies retain the existing error and never expose a partial successful result.

Core search is byte-for-byte unchanged from 0.4.21. Ranking, source positions, scopes, progress, admission, cancellation and existing deadlines retain their contracts. This is a small response-preparation change, not proof of the historical cause of every earlier regression. Final response construction remains after worker search completion; startup indexing and final transport are outside tool computation deadlines. This release does not claim those lifecycle limits are solved.

The primary evidence is two separate small-MCP three-version campaigns. Across six cases (1/4/16 entries, with/without progress) and both rankers, repeated median changes versus fresh exact 0.4.21 range **-8.98% to +9.02%** and **-7.33% to -1.26%**. Negative means shorter observed time. The two ordinary MCP campaigns additionally observe canonical repeated TF-IDF changes of **-33.73% / -4.34%** and one-entry TF-IDF changes of **-0.05% / -1.89%**. The canonical TF-IDF observations disagree across ordinary and separate large-corpus campaigns, so they do not establish a stable large-corpus speedup. All slower cases remain. No universal speedup, statistical significance or semantic-quality claim is made.

## Executed validation and API contract

The full matrix passes **249 default tests, 902 all-feature checks including 39 documentation examples, 39 isolated feature entries and five Python tests**. Strict Clippy/docs, Rust 1.89 provider-neutral and Rust 1.92 all-feature checks pass. API comparison with tagged 0.4.21 reports **229 pass, 31 skip**. The independent Rust 1.89 consumer covers retrieval, coverage, quotes, document positions and MCP/schema construction. The unchanged PDF backend passes 14 regressions and its 381-source-file provenance check. The listed ignored crash-writer helper is intentionally launched by its parent crash-recovery test, not counted as a separately requested standalone test. The refreshed dependency audit reports zero known vulnerabilities in its checked database; maintenance notices remain in the resource record.

Three new regressions independently measure encoded JSON lengths, compare the entire reply byte-for-byte with the legacy construction, recheck changed budgets, and reject replies whose data fits but escaped/duplicated/enveloped output does not. Cases include Unicode, controls, nesting, null, empty and numeric payloads. Existing native RPC bounds and hostile input, timeout, disconnect and drain tests remain. The native full-scan oracle still checks 1,025 records, both rankers, wide queries, scopes, limits and cold/warm states.

**Thirty-three finite Tree Borrows Miri cases pass:** the previous 30 search/text/source cases plus the three new pure MCP response cases. These response tests do not execute Tokio I/O. Previous scaled cases retain their scope; native tests cover larger boundaries. Only extra floating-point rounding injection is disabled; memory/alias checks remain. Tokio/PDF/FFI/ONNX execution is outside this interpreter scope. These are not independent security or soundness audits. Extracted-package tests, exact-commit CI, a fresh Rust 1.89 registry consumer, tag/VCS/checksum and release-asset agreement gate completion.

Engine, SearchOptions, Retriever, WorkControl, McpServer and McpLimits usage stays the same. Default features stay empty. No dependency, tool argument, advertised schema or saved-format migration is added. Read [Core performance](CORE-PERFORMANCE.md), [API](API.md), [cookbook](API-COOKBOOK.md), [MCP](MCP.md) and [security scope](MCP-SECURITY.md).

## Frozen method and development record

Baseline is exact registry 0.4.21, VCS `8ce456be12218f38cffc32ed1737accbe24f8668`; the older reference is exact registry 0.4.20, VCS `e30a055fab3dae32c8e59047fc01a2e32226dbed`. Every cached registry source file is compared byte-for-byte with its checksum-verified published crate archive before final timing. Candidate 0.4.22 runs the same programs and inputs. Actual worker sources, manifests, locks, executables, runtime, compile-time schemas and inputs are hashed and preserved. Read the [reproduction method](validation-0.4.22/README.md).

A separate three-triple development pilot keeps large, standard-wide, extended-wide and tiny MCP cases plus small Core controls. Complete compared responses agree. Snapshots include MCP and Core source. Its observations remain separate from every final distribution. Earlier releases retain their own selected and rejected trials in their original archives. No final observation is replaced by a faster pilot.

## Primary small-MCP target and confirmation

Each campaign runs seven rotating triples of exact 0.4.20/0.4.21/0.4.22 for all six cases: **126 fresh servers, 8,568 actual schema-valid replies and 5,712 complete structured comparisons**. Both together: **252 servers, 17,136 replies and 11,424 complete comparisons**. Schemas, RPC-ID response content, input hashes, requested/absent progress and EOF shutdown agree, excluding only checked status version and version-qualified schema URNs. Each query/ranker has seven initial and fourteen repeated samples. Initial, repeated, per-query, process-sum, startup and RSS records stay separate. Search round trips include Rust work, serialization, stdio, Python parsing and requested progress.

The two retained small-Core campaigns still cover all original six and 96 extended cases: 84 processes, 4,284,000 timed calls and 2,688 complete original matrix-response comparisons. Their median changes across the original six range **-0.92% to +2.34%** and **-3.04% to +2.65%**. Core code is unchanged; these are regression controls, not an optimization claim.

### Standard small-MCP campaign

| Case | Ranker | 0.4.20 repeated median / p95 ms | 0.4.21 repeated median / p95 ms | 0.4.22 repeated median / p95 ms | vs 0.4.21 |
| --- | --- | --- | --- | --- | --- |
| tiny | bm25 | 0.130 / 0.225 | 0.137 / 0.236 | 0.150 / 0.258 | +9.02% |
| tiny | tfidf | 0.136 / 0.255 | 0.143 / 0.250 | 0.149 / 0.281 | +4.47% |
| tiny4 | bm25 | 0.199 / 0.270 | 0.188 / 0.252 | 0.176 / 0.253 | -6.47% |
| tiny4 | tfidf | 0.189 / 0.270 | 0.187 / 0.242 | 0.171 / 0.231 | -8.59% |
| tiny16 | bm25 | 0.200 / 0.239 | 0.201 / 0.268 | 0.194 / 0.230 | -3.69% |
| tiny16 | tfidf | 0.201 / 0.249 | 0.201 / 0.262 | 0.195 / 0.249 | -3.19% |
| tiny-no-progress | bm25 | 0.072 / 0.132 | 0.074 / 0.122 | 0.069 / 0.098 | -7.04% |
| tiny-no-progress | tfidf | 0.071 / 0.166 | 0.073 / 0.168 | 0.068 / 0.156 | -5.60% |
| tiny4-no-progress | bm25 | 0.129 / 0.169 | 0.132 / 0.209 | 0.122 / 0.172 | -7.20% |
| tiny4-no-progress | tfidf | 0.127 / 0.156 | 0.130 / 0.192 | 0.118 / 0.147 | -8.98% |
| tiny16-no-progress | bm25 | 0.155 / 0.205 | 0.152 / 0.205 | 0.141 / 0.215 | -7.31% |
| tiny16-no-progress | tfidf | 0.152 / 0.191 | 0.151 / 0.175 | 0.141 / 0.199 | -7.11% |

All initial/repeated per-query means, medians, p95, process sums, startup, RSS and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.4.22/docs/measurements/0.4.22/small-mcp-standard-summary.json). No slower sample is removed.

### Confirmation small-MCP campaign

| Case | Ranker | 0.4.20 repeated median / p95 ms | 0.4.21 repeated median / p95 ms | 0.4.22 repeated median / p95 ms | vs 0.4.21 |
| --- | --- | --- | --- | --- | --- |
| tiny | bm25 | 0.124 / 0.176 | 0.124 / 0.157 | 0.120 / 0.182 | -3.69% |
| tiny | tfidf | 0.126 / 0.226 | 0.123 / 0.208 | 0.121 / 0.210 | -1.26% |
| tiny4 | bm25 | 0.178 / 0.228 | 0.182 / 0.236 | 0.169 / 0.211 | -7.33% |
| tiny4 | tfidf | 0.177 / 0.224 | 0.180 / 0.229 | 0.168 / 0.188 | -6.63% |
| tiny16 | bm25 | 0.207 / 0.291 | 0.200 / 0.228 | 0.195 / 0.347 | -2.39% |
| tiny16 | tfidf | 0.208 / 0.325 | 0.202 / 0.252 | 0.197 / 0.364 | -2.37% |
| tiny-no-progress | bm25 | 0.073 / 0.099 | 0.072 / 0.095 | 0.068 / 0.094 | -6.23% |
| tiny-no-progress | tfidf | 0.071 / 0.112 | 0.072 / 0.135 | 0.068 / 0.105 | -5.29% |
| tiny4-no-progress | bm25 | 0.130 / 0.186 | 0.125 / 0.163 | 0.122 / 0.175 | -2.51% |
| tiny4-no-progress | tfidf | 0.127 / 0.186 | 0.124 / 0.150 | 0.120 / 0.171 | -3.49% |
| tiny16-no-progress | bm25 | 0.152 / 0.204 | 0.151 / 0.225 | 0.141 / 0.215 | -6.61% |
| tiny16-no-progress | tfidf | 0.152 / 0.188 | 0.151 / 0.216 | 0.142 / 0.183 | -5.78% |

All initial/repeated per-query means, medians, p95, process sums, startup, RSS and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.4.22/docs/measurements/0.4.22/small-mcp-confirmation-summary.json). No slower sample is removed.

## Retained large and broad MCP controls

The two large-corpus campaigns each run seven rotating 0.4.20/0.4.21/0.4.22 triples on the same 167,942-entry knowledge input and ten retained queries. Each has 21 fresh servers, 1,428 schema-valid replies and 952 complete cross-version structured comparisons. Together: **42 servers, 2,856 schema-valid replies and 1,904 complete comparisons**. All compared results agree except asserted status version and version-qualified schema URNs. Each query/ranker has seven initial and fourteen repeated samples. Input candidate counts range from zero to 29,600; do not infer gains for all corpus densities or gaps. Dense, filtered, sparse and cancellation correctness also has independent native tests.

Three separate retained broad controls each use 21 servers: **63 servers, 4,284 actual schema replies and 2,856 complete structured comparisons**. They guard the 0.4.20 broad-explanation improvement rather than replace large-corpus evidence. Standard/confirmation retain the 257-record sparse/dense vocabulary fixture and ten original queries. Extended probes 32/33/64/65/128/129/192/255/256 known terms and 255 known terms plus a duplicate and unknown word. Existing normalized-term and byte limits remain. Keep all five distributions separate.

Search timing includes Rust work, serialization, stdio, Python parsing and requested progress. Initial calls, repeated calls, startup, per-query means/medians/tails, per-process totals and RSS are separate. Original standard queries are not all equally wide. Aggregate medians alone can obscure differences; retain every per-query row and raw timing observation. No sample is removed or replaced.

### Standard large-corpus campaign

| Ranker / phase | 0.4.20 median / p95 ms | 0.4.21 median / p95 ms | 0.4.22 median / p95 ms | vs 0.4.21 | vs 0.4.20 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 3.982 / 10.519 | 3.952 / 9.760 | 4.436 / 11.275 | +12.24% | +11.41% |
| tfidf / initial | 22.566 / 41.551 | 22.239 / 40.417 | 23.793 / 43.236 | +6.99% | +5.44% |
| bm25 / repeated | 3.254 / 7.564 | 3.549 / 8.144 | 3.353 / 9.329 | -5.51% | +3.04% |
| tfidf / repeated | 4.092 / 10.194 | 4.022 / 9.910 | 4.257 / 8.837 | +5.84% | +4.02% |

| Query | Repeated TF-IDF median ms: 0.4.20 / 0.4.21 / 0.4.22 | vs 0.4.21 | Initial TF-IDF mean ms: 0.4.21 → 0.4.22 |
| --- | --- | --- | --- |
| 0 | 3.939 / 4.152 / 4.300 | +3.56% | 26.093 → 29.838 |
| 1 | 4.262 / 3.941 / 4.050 | +2.76% | 27.559 → 30.666 |
| 2 | 6.895 / 5.895 / 6.036 | +2.40% | 39.236 → 43.834 |
| 3 | 1.016 / 1.008 / 1.049 | +4.10% | 4.367 → 4.356 |
| 4 | 4.262 / 3.704 / 4.315 | +16.50% | 19.674 → 21.608 |
| 5 | 6.244 / 6.066 / 5.969 | -1.59% | 31.367 → 34.289 |
| 6 | 2.039 / 1.705 / 1.933 | +13.42% | 7.586 → 7.455 |
| 7 | 7.200 / 6.288 / 7.281 | +15.79% | 34.891 → 35.626 |
| 8 | 3.758 / 3.663 / 3.585 | -2.12% | 6.191 → 5.262 |
| 9 | 0.145 / 0.120 / 0.126 | +4.81% | 1.764 → 1.197 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.20 | 200.234 | 89.713 | 5551.963 | 1726.672 |
| 0.4.21 | 205.791 | 99.836 | 5513.841 | 1726.766 |
| 0.4.22 | 221.232 | 95.595 | 5870.789 | 1725.359 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.4.22/docs/measurements/0.4.22/large-standard-summary.json).

### Confirmation large-corpus campaign

| Ranker / phase | 0.4.20 median / p95 ms | 0.4.21 median / p95 ms | 0.4.22 median / p95 ms | vs 0.4.21 | vs 0.4.20 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 2.835 / 4.636 | 2.776 / 4.441 | 2.934 / 4.782 | +5.69% | +3.49% |
| tfidf / initial | 18.808 / 34.070 | 19.383 / 34.425 | 20.620 / 37.286 | +6.39% | +9.64% |
| bm25 / repeated | 2.581 / 4.396 | 2.387 / 4.510 | 2.431 / 4.191 | +1.84% | -5.82% |
| tfidf / repeated | 2.957 / 5.997 | 2.607 / 5.557 | 2.655 / 5.270 | +1.85% | -10.21% |

| Query | Repeated TF-IDF median ms: 0.4.20 / 0.4.21 / 0.4.22 | vs 0.4.21 | Initial TF-IDF mean ms: 0.4.21 → 0.4.22 |
| --- | --- | --- | --- |
| 0 | 3.270 / 2.487 / 2.623 | +5.44% | 24.629 → 25.754 |
| 1 | 2.943 / 2.710 / 2.748 | +1.41% | 23.429 → 24.796 |
| 2 | 5.139 / 4.134 / 4.444 | +7.50% | 35.471 → 36.359 |
| 3 | 0.805 / 0.766 / 0.741 | -3.26% | 1.709 → 1.718 |
| 4 | 3.228 / 2.947 / 2.703 | -8.27% | 15.403 → 16.280 |
| 5 | 4.795 / 4.097 / 3.932 | -4.02% | 27.044 → 26.384 |
| 6 | 1.438 / 1.329 / 1.206 | -9.23% | 5.341 → 5.109 |
| 7 | 5.059 / 4.426 / 4.145 | -6.36% | 26.891 → 27.985 |
| 8 | 2.723 / 2.407 / 2.418 | +0.44% | 2.701 → 2.971 |
| 9 | 0.123 / 0.116 / 0.114 | -2.17% | 0.181 → 0.182 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.20 | 158.760 | 55.704 | 5227.048 | 1724.250 |
| 0.4.21 | 157.810 | 50.269 | 5109.726 | 1724.188 |
| 0.4.22 | 165.094 | 50.153 | 5290.303 | 1724.344 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.4.22/docs/measurements/0.4.22/large-confirmation-summary.json).

### Standard broad-query campaign

| Ranker / phase | 0.4.20 median / p95 ms | 0.4.21 median / p95 ms | 0.4.22 median / p95 ms | vs 0.4.21 | vs 0.4.20 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 0.630 / 2.960 | 0.629 / 2.829 | 0.606 / 4.647 | -3.58% | -3.82% |
| tfidf / initial | 0.592 / 5.030 | 0.598 / 1.822 | 0.732 / 5.309 | +22.47% | +23.57% |
| bm25 / repeated | 0.570 / 4.117 | 0.621 / 3.856 | 0.575 / 4.763 | -7.52% | +0.91% |
| tfidf / repeated | 0.585 / 4.072 | 0.688 / 4.491 | 0.593 / 3.921 | -13.86% | +1.35% |

| Query | Repeated TF-IDF median ms: 0.4.20 / 0.4.21 / 0.4.22 | vs 0.4.21 | Initial TF-IDF mean ms: 0.4.21 → 0.4.22 |
| --- | --- | --- | --- |
| 0 | 0.515 / 0.839 / 0.490 | -41.67% | 0.632 → 1.736 |
| 1 | 0.453 / 0.618 / 0.442 | -28.51% | 0.670 → 1.627 |
| 2 | 0.501 / 0.551 / 0.492 | -10.72% | 0.561 → 1.374 |
| 3 | 0.845 / 0.881 / 0.862 | -2.21% | 0.908 → 0.971 |
| 4 | 0.553 / 0.527 / 0.535 | +1.66% | 0.537 → 2.352 |
| 5 | 0.525 / 0.504 / 0.467 | -7.26% | 0.496 → 1.070 |
| 6 | 0.510 / 0.510 / 0.533 | +4.52% | 2.219 → 1.099 |
| 7 | 0.850 / 0.876 / 0.906 | +3.44% | 1.091 → 2.011 |
| 8 | 0.456 / 0.451 / 0.450 | -0.09% | 1.302 → 1.485 |
| 9 | 0.846 / 0.834 / 0.877 | +5.16% | 1.155 → 0.839 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.20 | 7.323 | 23.706 | 7.321 | 7.922 |
| 0.4.21 | 7.483 | 20.324 | 7.106 | 7.875 |
| 0.4.22 | 10.983 | 24.248 | 7.387 | 7.734 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.4.22/docs/measurements/0.4.22/wide-standard-summary.json).

### Confirmation broad-query campaign

| Ranker / phase | 0.4.20 median / p95 ms | 0.4.21 median / p95 ms | 0.4.22 median / p95 ms | vs 0.4.21 | vs 0.4.20 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 0.569 / 0.717 | 0.620 / 0.841 | 0.602 / 0.798 | -2.79% | +5.81% |
| tfidf / initial | 0.560 / 0.925 | 0.636 / 1.035 | 0.576 / 0.986 | -9.38% | +2.81% |
| bm25 / repeated | 0.509 / 0.928 | 0.577 / 1.359 | 0.565 / 1.508 | -2.14% | +11.00% |
| tfidf / repeated | 0.500 / 0.903 | 0.626 / 1.045 | 0.580 / 0.956 | -7.46% | +15.99% |

| Query | Repeated TF-IDF median ms: 0.4.20 / 0.4.21 / 0.4.22 | vs 0.4.21 | Initial TF-IDF mean ms: 0.4.21 → 0.4.22 |
| --- | --- | --- | --- |
| 0 | 0.465 / 0.583 / 0.526 | -9.73% | 0.613 → 0.607 |
| 1 | 0.421 / 0.524 / 0.456 | -13.00% | 0.548 → 0.567 |
| 2 | 0.481 / 0.548 / 0.549 | +0.27% | 0.579 → 0.612 |
| 3 | 0.759 / 0.902 / 0.839 | -7.04% | 0.956 → 0.893 |
| 4 | 0.464 / 0.549 / 0.504 | -8.19% | 0.519 → 0.507 |
| 5 | 0.455 / 0.527 / 0.533 | +1.14% | 0.537 → 0.474 |
| 6 | 0.490 / 0.545 / 0.525 | -3.64% | 0.631 → 0.634 |
| 7 | 0.835 / 0.905 / 0.873 | -3.54% | 0.876 → 0.882 |
| 8 | 0.444 / 0.507 / 0.511 | +0.92% | 0.501 → 0.531 |
| 9 | 0.799 / 0.847 / 0.833 | -1.64% | 0.920 → 0.882 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.20 | 6.323 | 11.443 | 4.494 | 7.906 |
| 0.4.21 | 6.557 | 13.651 | 4.395 | 7.922 |
| 0.4.22 | 6.558 | 12.634 | 4.831 | 7.703 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.4.22/docs/measurements/0.4.22/wide-confirmation-summary.json).

### Extended broad-query campaign

| Ranker / phase | 0.4.20 median / p95 ms | 0.4.21 median / p95 ms | 0.4.22 median / p95 ms | vs 0.4.21 | vs 0.4.20 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 0.630 / 1.053 | 0.695 / 1.117 | 0.617 / 1.017 | -11.22% | -2.06% |
| tfidf / initial | 0.785 / 1.111 | 0.812 / 1.144 | 0.749 / 1.012 | -7.76% | -4.56% |
| bm25 / repeated | 0.557 / 1.284 | 0.643 / 1.494 | 0.541 / 1.228 | -15.87% | -2.84% |
| tfidf / repeated | 0.757 / 1.053 | 0.806 / 1.217 | 0.707 / 0.982 | -12.20% | -6.60% |

| Query | Repeated TF-IDF median ms: 0.4.20 / 0.4.21 / 0.4.22 | vs 0.4.21 | Initial TF-IDF mean ms: 0.4.21 → 0.4.22 |
| --- | --- | --- | --- |
| 0 | 0.422 / 0.434 / 0.392 | -9.85% | 0.589 → 0.494 |
| 1 | 0.409 / 0.415 / 0.385 | -7.13% | 0.489 → 0.448 |
| 2 | 0.444 / 0.483 / 0.416 | -13.88% | 0.541 → 0.479 |
| 3 | 0.445 / 0.472 / 0.433 | -8.13% | 0.542 → 0.455 |
| 4 | 0.762 / 0.827 / 0.712 | -13.90% | 0.877 → 0.743 |
| 5 | 0.729 / 0.749 / 0.695 | -7.17% | 0.817 → 0.741 |
| 6 | 0.889 / 0.926 / 0.875 | -5.53% | 1.027 → 0.931 |
| 7 | 0.942 / 0.980 / 0.895 | -8.67% | 1.065 → 0.988 |
| 8 | 0.949 / 1.020 / 0.928 | -9.05% | 1.028 → 0.926 |
| 9 | 0.945 / 0.952 / 0.888 | -6.70% | 1.007 → 0.948 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.20 | 7.431 | 13.731 | 4.060 | 8.203 |
| 0.4.21 | 7.622 | 14.904 | 4.495 | 8.188 |
| 0.4.22 | 7.063 | 13.035 | 4.385 | 8.078 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](https://github.com/LEVOGNE/airust/blob/v0.4.22/docs/measurements/0.4.22/wide-extended-summary.json).

### Existing limit and recovery checks

Three additional fresh servers reject 257/512 known-term requests with both rankers: twelve expected errors, no partial successful results, error progress and successful later health requests. The existing 256-unique-term assessment limit remains; these are not successful-search latency samples. Large inputs remain subject to cooperative bounds, not an OS CPU/RAM quota.

## Actual MCP: two ordinary campaigns

Each ordinary campaign uses nine cases and five alternating old/new pairs: **90 servers each, 180 total**. Canonical knowledge, NumPy and broad synthetic input request progress; 1/4/16-entry inputs run with and without progress. Each makes ten queries with both rankers, two repeated passes, four concurrent searches, matching/relations, rejected arguments and health; snapshot inputs check exact UTF-8 readbacks.

Together **13,880 actual result/error replies validate against advertised schemas**, and **6,940 complete old/new structured pairs agree by RPC ID**, excluding only checked status version and version-qualified schema URNs. Discovery, progress/no-progress, bounds, ordering, input hashes and EOF shutdown pass. Search and readback time are separate. Child peak RSS includes runtime, buffers and source/index memory, unlike allocation profiles. Retain all slower rows.

### First complete campaign

| Case | Ranker | Initial median / p95 ms: old → new | Repeated median / p95 ms: old → new | Repeated median change |
| --- | --- | --- | --- | --- |
| canonical | bm25 | 4.144 / 13.663 → 2.899 / 4.683 | 3.751 / 6.847 → 2.608 / 4.762 | -30.45% |
| canonical | tfidf | 26.585 / 48.893 → 20.089 / 35.311 | 4.457 / 10.594 → 2.954 / 5.933 | -33.73% |
| numpy | bm25 | 0.943 / 1.300 → 0.914 / 1.267 | 0.823 / 1.226 → 0.830 / 1.247 | +0.89% |
| numpy | tfidf | 1.160 / 2.865 → 1.176 / 2.728 | 0.776 / 1.230 → 0.769 / 1.538 | -0.84% |
| wide | bm25 | 0.527 / 0.737 → 0.490 / 0.579 | 0.448 / 0.698 → 0.399 / 0.766 | -10.91% |
| wide | tfidf | 0.512 / 0.828 → 0.457 / 0.740 | 0.437 / 0.717 → 0.396 / 0.672 | -9.49% |
| tiny | bm25 | 0.143 / 0.259 → 0.146 / 0.250 | 0.122 / 0.174 → 0.122 / 0.194 | -0.07% |
| tiny | tfidf | 0.135 / 0.199 → 0.143 / 0.224 | 0.123 / 0.215 → 0.123 / 0.220 | -0.05% |
| tiny4 | bm25 | 0.208 / 0.341 → 0.195 / 0.325 | 0.183 / 0.231 → 0.171 / 0.216 | -6.76% |
| tiny4 | tfidf | 0.202 / 0.282 → 0.183 / 0.249 | 0.179 / 0.219 → 0.169 / 0.209 | -5.70% |
| tiny16 | bm25 | 0.239 / 0.386 → 0.214 / 0.354 | 0.202 / 0.255 → 0.194 / 0.226 | -4.27% |
| tiny16 | tfidf | 0.226 / 0.308 → 0.211 / 0.283 | 0.204 / 0.252 → 0.194 / 0.242 | -4.98% |
| tiny-no-progress | bm25 | 0.091 / 0.188 → 0.091 / 0.186 | 0.075 / 0.105 → 0.069 / 0.094 | -8.33% |
| tiny-no-progress | tfidf | 0.088 / 0.145 → 0.084 / 0.128 | 0.074 / 0.139 → 0.068 / 0.104 | -7.19% |
| tiny4-no-progress | bm25 | 0.151 / 0.281 → 0.154 / 0.265 | 0.132 / 0.216 → 0.123 / 0.176 | -6.34% |
| tiny4-no-progress | tfidf | 0.140 / 0.213 → 0.143 / 0.233 | 0.130 / 0.203 → 0.121 / 0.163 | -6.33% |
| tiny16-no-progress | bm25 | 0.219 / 0.344 → 0.185 / 0.305 | 0.171 / 0.239 → 0.149 / 0.223 | -12.63% |
| tiny16-no-progress | tfidf | 0.194 / 0.289 → 0.181 / 0.243 | 0.169 / 0.230 → 0.148 / 0.204 | -12.27% |

| Case | Startup median ms: old → new | First TF-IDF median ms: old → new | Peak RSS median MiB: old → new |
| --- | --- | --- | --- |
| canonical | 5654.229 → 5348.318 | 34.939 → 25.017 | 1725.000 → 1724.797 |
| numpy | 143.500 → 143.809 | 1.818 → 1.625 | 57.156 → 57.156 |
| wide | 4.332 → 4.036 | 0.545 → 0.513 | 7.734 → 7.734 |
| tiny | 3.693 → 3.790 | 0.201 → 0.203 | 5.609 → 5.781 |
| tiny4 | 3.530 → 3.668 | 0.289 → 0.256 | 5.750 → 5.859 |
| tiny16 | 3.657 → 3.635 | 0.310 → 0.285 | 5.938 → 6.062 |
| tiny-no-progress | 3.632 → 3.510 | 0.143 → 0.132 | 5.656 → 5.781 |
| tiny4-no-progress | 3.720 → 3.591 | 0.213 → 0.183 | 5.828 → 5.859 |
| tiny16-no-progress | 3.740 → 4.008 | 0.278 → 0.230 | 5.922 → 6.031 |

### Second preplanned complete campaign

| Case | Ranker | Initial median / p95 ms: old → new | Repeated median / p95 ms: old → new | Repeated median change |
| --- | --- | --- | --- | --- |
| canonical | bm25 | 2.916 / 4.618 → 3.107 / 8.806 | 2.559 / 4.298 → 2.492 / 5.489 | -2.59% |
| canonical | tfidf | 19.810 / 34.618 → 23.053 / 56.465 | 2.885 / 5.362 → 2.760 / 8.474 | -4.34% |
| numpy | bm25 | 1.032 / 1.409 → 0.989 / 1.399 | 0.904 / 1.308 → 0.917 / 1.458 | +1.38% |
| numpy | tfidf | 1.265 / 2.970 → 1.321 / 3.023 | 0.832 / 1.396 → 0.835 / 2.167 | +0.31% |
| wide | bm25 | 0.632 / 3.172 → 0.564 / 0.766 | 0.574 / 0.873 → 0.506 / 0.879 | -11.80% |
| wide | tfidf | 0.629 / 2.705 → 0.577 / 0.968 | 0.566 / 1.017 → 0.507 / 0.870 | -10.46% |
| tiny | bm25 | 0.184 / 1.440 → 0.228 / 10.831 | 0.147 / 0.264 → 0.145 / 1.386 | -1.46% |
| tiny | tfidf | 0.182 / 2.943 → 0.222 / 5.152 | 0.151 / 0.320 → 0.148 / 0.767 | -1.89% |
| tiny4 | bm25 | 0.301 / 0.781 → 0.293 / 1.566 | 0.234 / 0.314 → 0.227 / 0.303 | -2.76% |
| tiny4 | tfidf | 0.289 / 0.502 → 0.261 / 4.108 | 0.230 / 0.339 → 0.230 / 0.304 | +0.26% |
| tiny16 | bm25 | 0.232 / 0.376 → 0.253 / 0.406 | 0.206 / 0.272 → 0.214 / 0.297 | +3.71% |
| tiny16 | tfidf | 0.222 / 0.296 → 0.253 / 0.381 | 0.207 / 0.307 → 0.212 / 0.324 | +2.23% |
| tiny-no-progress | bm25 | 0.121 / 0.231 → 0.108 / 0.217 | 0.099 / 0.162 → 0.077 / 0.132 | -22.21% |
| tiny-no-progress | tfidf | 0.117 / 0.172 → 0.108 / 0.208 | 0.099 / 0.174 → 0.078 / 0.166 | -21.25% |
| tiny4-no-progress | bm25 | 0.178 / 0.307 → 0.173 / 0.335 | 0.157 / 0.240 → 0.152 / 1.070 | -3.38% |
| tiny4-no-progress | tfidf | 0.159 / 0.245 → 0.159 / 0.268 | 0.146 / 0.230 → 0.146 / 0.852 | -0.01% |
| tiny16-no-progress | bm25 | 0.188 / 0.309 → 0.180 / 0.306 | 0.159 / 0.231 → 0.146 / 0.246 | -7.79% |
| tiny16-no-progress | tfidf | 0.184 / 0.266 → 0.157 / 0.272 | 0.157 / 0.235 → 0.146 / 0.227 | -7.31% |

| Case | Startup median ms: old → new | First TF-IDF median ms: old → new | Peak RSS median MiB: old → new |
| --- | --- | --- | --- |
| canonical | 5352.399 → 5322.166 | 25.482 → 25.848 | 1725.031 → 1724.438 |
| numpy | 150.405 → 149.489 | 1.856 → 1.703 | 57.344 → 57.234 |
| wide | 4.947 → 4.704 | 0.583 → 0.547 | 7.812 → 7.766 |
| tiny | 8.589 → 7.126 | 0.271 → 1.270 | 5.750 → 5.859 |
| tiny4 | 3.860 → 4.011 | 0.383 → 0.288 | 5.859 → 5.922 |
| tiny16 | 3.700 → 4.060 | 0.296 → 0.297 | 5.938 → 6.094 |
| tiny-no-progress | 3.744 → 3.912 | 0.143 → 0.138 | 5.734 → 5.844 |
| tiny4-no-progress | 3.848 → 3.523 | 0.228 → 0.222 | 5.844 → 5.906 |
| tiny16-no-progress | 3.757 → 3.737 | 0.276 → 0.237 | 6.047 → 6.141 |

## Retained small Core regression controls

These separate campaigns compare freshly built exact registry 0.4.20 and 0.4.21 with the candidate, using the same programs. Seven rotating triples per campaign run both the original six cases and a 96-case extension. Warm-up, full original-JSON comparison and index setup occur before timers. Each campaign has **42 processes and 2,142,000 timed Core calls**. Both together have 84 processes, 4,284,000 calls and **2,688 complete original matrix-response comparisons**, with identical results. Signature checks also cover every original-six and timed matrix sample. This is repeated warm Core time, without MCP transport or DHAT instrumentation.

Keep the two campaigns separate. Original six-case calls build SearchOptions inside the measured loop; the extension reuses fixed options. Do not compare their absolute times or merge distributions. The 96 cases combine four entry counts, both rankers and twelve fixed query/filter/limit cases. Wide, repeated, punctuated and Unicode queries retain fallback analysis. Unknown-only ASCII queries, scope rejection and zero limits are also retained; no case is removed for missing the optimized path. Historical absolute values with different programs/loads are not a reference substitute.

### First small Core campaign

#### Original six warm cases

| Fixture | 0.4.20 median ns | 0.4.21 median ns | 0.4.22 median ns | vs 0.4.21 | vs 0.4.20 |
| --- | --- | --- | --- | --- | --- |
| tiny16_bm25 | 1039.750 | 919.792 | 930.292 | +1.14% | -10.53% |
| tiny16_tfidf | 1065.334 | 1000.958 | 995.375 | -0.56% | -6.57% |
| tiny4_bm25 | 350.458 | 293.416 | 291.792 | -0.55% | -16.74% |
| tiny4_tfidf | 366.917 | 310.833 | 310.667 | -0.05% | -15.33% |
| tiny_bm25 | 201.583 | 120.833 | 123.666 | +2.34% | -38.65% |
| tiny_tfidf | 183.875 | 126.959 | 125.792 | -0.92% | -31.59% |

Higher candidate medians versus 0.4.21: **2 of 6**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](https://github.com/LEVOGNE/airust/blob/v0.4.22/docs/measurements/0.4.22/small-reference-summary.json). No threshold-based rejection or removal of slow timing samples is used.

#### All 96 extended cases

| Fixture | 0.4.20 median ns | 0.4.21 median ns | 0.4.22 median ns | vs 0.4.21 | vs 0.4.20 |
| --- | --- | --- | --- | --- | --- |
| 16_Bm25_disjoint | 490.042 | 463.250 | 459.583 | -0.79% | -6.22% |
| 16_Bm25_limit_one | 646.417 | 582.916 | 585.750 | +0.49% | -9.39% |
| 16_Bm25_limit_zero | 2.917 | 2.708 | 2.708 | +0.00% | -7.16% |
| 16_Bm25_multi | 1567.250 | 1631.917 | 1617.834 | -0.86% | +3.23% |
| 16_Bm25_punctuation | 143.791 | 146.167 | 144.666 | -1.03% | +0.61% |
| 16_Bm25_repeated | 1171.875 | 1202.125 | 1167.084 | -2.91% | -0.41% |
| 16_Bm25_scope | 984.125 | 976.291 | 985.417 | +0.93% | +0.13% |
| 16_Bm25_single | 1123.458 | 1089.542 | 1060.583 | -2.66% | -5.60% |
| 16_Bm25_unicode | 1327.584 | 1358.041 | 1356.417 | -0.12% | +2.17% |
| 16_Bm25_unknown | 69.125 | 19.875 | 20.000 | +0.63% | -71.07% |
| 16_Bm25_unknown_added | 1149.584 | 1189.208 | 1169.625 | -1.65% | +1.74% |
| 16_Bm25_wide | 16968.709 | 17260.667 | 16863.458 | -2.30% | -0.62% |
| 16_Tfidf_disjoint | 493.417 | 460.375 | 462.792 | +0.53% | -6.21% |
| 16_Tfidf_limit_one | 716.292 | 683.125 | 685.834 | +0.40% | -4.25% |
| 16_Tfidf_limit_zero | 2.875 | 2.708 | 2.667 | -1.51% | -7.23% |
| 16_Tfidf_multi | 1685.958 | 1727.208 | 1693.958 | -1.93% | +0.47% |
| 16_Tfidf_punctuation | 144.917 | 144.083 | 141.542 | -1.76% | -2.33% |
| 16_Tfidf_repeated | 1256.375 | 1317.416 | 1278.041 | -2.99% | +1.72% |
| 16_Tfidf_scope | 1073.083 | 1043.791 | 1019.958 | -2.28% | -4.95% |
| 16_Tfidf_single | 1215.292 | 1176.417 | 1145.667 | -2.61% | -5.73% |
| 16_Tfidf_unicode | 1420.208 | 1459.750 | 1407.916 | -3.55% | -0.87% |
| 16_Tfidf_unknown | 69.791 | 19.875 | 19.875 | +0.00% | -71.52% |
| 16_Tfidf_unknown_added | 1249.375 | 1293.375 | 1255.167 | -2.95% | +0.46% |
| 16_Tfidf_wide | 17121.958 | 17336.916 | 17066.917 | -1.56% | -0.32% |
| 1_Bm25_disjoint | 113.500 | 59.542 | 58.750 | -1.33% | -48.24% |
| 1_Bm25_limit_one | 185.875 | 126.084 | 126.458 | +0.30% | -31.97% |
| 1_Bm25_limit_zero | 2.917 | 2.708 | 2.667 | -1.51% | -8.57% |
| 1_Bm25_multi | 263.208 | 254.417 | 262.500 | +3.18% | -0.27% |
| 1_Bm25_punctuation | 145.084 | 143.500 | 143.875 | +0.26% | -0.83% |
| 1_Bm25_repeated | 253.250 | 271.917 | 262.166 | -3.59% | +3.52% |
| 1_Bm25_scope | 189.541 | 131.584 | 131.666 | +0.06% | -30.53% |
| 1_Bm25_single | 228.125 | 147.875 | 143.500 | -2.96% | -37.10% |
| 1_Bm25_unicode | 432.625 | 429.875 | 428.916 | -0.22% | -0.86% |
| 1_Bm25_unknown | 71.291 | 19.833 | 19.625 | -1.05% | -72.47% |
| 1_Bm25_unknown_added | 223.041 | 227.041 | 228.333 | +0.57% | +2.37% |
| 1_Bm25_wide | 4502.000 | 4437.000 | 4400.125 | -0.83% | -2.26% |
| 1_Tfidf_disjoint | 112.250 | 59.042 | 60.500 | +2.47% | -46.10% |
| 1_Tfidf_limit_one | 192.500 | 134.416 | 132.583 | -1.36% | -31.13% |
| 1_Tfidf_limit_zero | 2.916 | 2.791 | 2.667 | -4.44% | -8.54% |
| 1_Tfidf_multi | 265.792 | 264.209 | 262.709 | -0.57% | -1.16% |
| 1_Tfidf_punctuation | 147.292 | 147.208 | 143.250 | -2.69% | -2.74% |
| 1_Tfidf_repeated | 255.125 | 256.375 | 257.459 | +0.42% | +0.91% |
| 1_Tfidf_scope | 201.708 | 141.875 | 139.709 | -1.53% | -30.74% |
| 1_Tfidf_single | 198.625 | 135.042 | 135.000 | -0.03% | -32.03% |
| 1_Tfidf_unicode | 449.166 | 449.458 | 439.708 | -2.17% | -2.11% |
| 1_Tfidf_unknown | 71.292 | 20.167 | 19.708 | -2.28% | -72.36% |
| 1_Tfidf_unknown_added | 227.791 | 230.375 | 231.625 | +0.54% | +1.68% |
| 1_Tfidf_wide | 4319.958 | 4479.542 | 4473.458 | -0.14% | +3.55% |
| 33_Bm25_disjoint | 1032.916 | 974.500 | 962.292 | -1.25% | -6.84% |
| 33_Bm25_limit_one | 1269.333 | 1196.209 | 1145.708 | -4.22% | -9.74% |
| 33_Bm25_limit_zero | 2.917 | 2.667 | 2.667 | +0.00% | -8.57% |
| 33_Bm25_multi | 2502.917 | 2523.792 | 2463.250 | -2.40% | -1.58% |
| 33_Bm25_punctuation | 144.625 | 145.375 | 142.667 | -1.86% | -1.35% |
| 33_Bm25_repeated | 1784.916 | 1766.708 | 1750.000 | -0.95% | -1.96% |
| 33_Bm25_scope | 1766.959 | 1795.167 | 1737.833 | -3.19% | -1.65% |
| 33_Bm25_single | 1712.084 | 1638.625 | 1618.375 | -1.24% | -5.47% |
| 33_Bm25_unicode | 1929.625 | 1937.000 | 1880.917 | -2.90% | -2.52% |
| 33_Bm25_unknown | 68.333 | 20.000 | 19.917 | -0.41% | -70.85% |
| 33_Bm25_unknown_added | 1816.916 | 1766.125 | 1737.000 | -1.65% | -4.40% |
| 33_Bm25_wide | 26630.667 | 26715.042 | 26535.000 | -0.67% | -0.36% |
| 33_Tfidf_disjoint | 986.375 | 979.209 | 946.583 | -3.33% | -4.03% |
| 33_Tfidf_limit_one | 1344.958 | 1365.958 | 1302.209 | -4.67% | -3.18% |
| 33_Tfidf_limit_zero | 2.917 | 2.667 | 2.667 | +0.00% | -8.57% |
| 33_Tfidf_multi | 2679.750 | 2720.458 | 2652.458 | -2.50% | -1.02% |
| 33_Tfidf_punctuation | 145.333 | 144.791 | 144.208 | -0.40% | -0.77% |
| 33_Tfidf_repeated | 1952.666 | 2007.042 | 1927.416 | -3.97% | -1.29% |
| 33_Tfidf_scope | 1849.792 | 1911.709 | 1879.709 | -1.67% | +1.62% |
| 33_Tfidf_single | 1897.958 | 1890.916 | 1822.917 | -3.60% | -3.95% |
| 33_Tfidf_unicode | 2031.834 | 2081.833 | 2033.708 | -2.31% | +0.09% |
| 33_Tfidf_unknown | 70.208 | 20.000 | 20.334 | +1.67% | -71.04% |
| 33_Tfidf_unknown_added | 1918.375 | 1989.708 | 1916.833 | -3.66% | -0.08% |
| 33_Tfidf_wide | 26757.417 | 26992.625 | 26578.208 | -1.54% | -0.67% |
| 4_Bm25_disjoint | 164.459 | 109.792 | 110.958 | +1.06% | -32.53% |
| 4_Bm25_limit_one | 244.209 | 185.792 | 190.042 | +2.29% | -22.18% |
| 4_Bm25_limit_zero | 2.917 | 2.708 | 2.958 | +9.23% | +1.41% |
| 4_Bm25_multi | 538.875 | 539.792 | 538.625 | -0.22% | -0.05% |
| 4_Bm25_punctuation | 146.875 | 145.250 | 144.208 | -0.72% | -1.82% |
| 4_Bm25_repeated | 420.750 | 424.209 | 438.250 | +3.31% | +4.16% |
| 4_Bm25_scope | 285.959 | 233.834 | 237.042 | +1.37% | -17.11% |
| 4_Bm25_single | 376.708 | 319.416 | 316.375 | -0.95% | -16.02% |
| 4_Bm25_unicode | 603.750 | 601.292 | 606.792 | +0.91% | +0.50% |
| 4_Bm25_unknown | 68.000 | 19.791 | 20.166 | +1.89% | -70.34% |
| 4_Bm25_unknown_added | 408.958 | 407.750 | 415.208 | +1.83% | +1.53% |
| 4_Bm25_wide | 7195.750 | 7426.500 | 7370.000 | -0.76% | +2.42% |
| 4_Tfidf_disjoint | 161.375 | 112.208 | 110.791 | -1.26% | -31.35% |
| 4_Tfidf_limit_one | 268.584 | 214.166 | 218.000 | +1.79% | -18.83% |
| 4_Tfidf_limit_zero | 2.958 | 2.708 | 2.708 | +0.00% | -8.45% |
| 4_Tfidf_multi | 569.459 | 575.708 | 576.208 | +0.09% | +1.19% |
| 4_Tfidf_punctuation | 148.500 | 146.875 | 144.333 | -1.73% | -2.81% |
| 4_Tfidf_repeated | 443.500 | 463.167 | 457.209 | -1.29% | +3.09% |
| 4_Tfidf_scope | 294.333 | 245.000 | 241.458 | -1.45% | -17.96% |
| 4_Tfidf_single | 398.542 | 343.833 | 341.583 | -0.65% | -14.29% |
| 4_Tfidf_unicode | 632.125 | 630.125 | 647.875 | +2.82% | +2.49% |
| 4_Tfidf_unknown | 69.459 | 19.750 | 19.792 | +0.21% | -71.51% |
| 4_Tfidf_unknown_added | 431.333 | 441.667 | 442.500 | +0.19% | +2.59% |
| 4_Tfidf_wide | 7320.834 | 7524.375 | 7415.000 | -1.45% | +1.29% |

Higher candidate medians versus 0.4.21: **27 of 96**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](https://github.com/LEVOGNE/airust/blob/v0.4.22/docs/measurements/0.4.22/small-reference-summary.json). No threshold-based rejection or removal of slow timing samples is used.

### Second preplanned small Core campaign

#### Original six warm cases

| Fixture | 0.4.20 median ns | 0.4.21 median ns | 0.4.22 median ns | vs 0.4.21 | vs 0.4.20 |
| --- | --- | --- | --- | --- | --- |
| tiny16_bm25 | 992.417 | 907.000 | 904.250 | -0.30% | -8.88% |
| tiny16_tfidf | 1045.000 | 980.375 | 984.875 | +0.46% | -5.75% |
| tiny4_bm25 | 338.875 | 287.084 | 286.666 | -0.15% | -15.41% |
| tiny4_tfidf | 356.000 | 309.084 | 306.792 | -0.74% | -13.82% |
| tiny_bm25 | 185.167 | 120.916 | 124.125 | +2.65% | -32.97% |
| tiny_tfidf | 186.958 | 128.792 | 124.875 | -3.04% | -33.21% |

Higher candidate medians versus 0.4.21: **2 of 6**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](https://github.com/LEVOGNE/airust/blob/v0.4.22/docs/measurements/0.4.22/small-confirmation-summary.json). No threshold-based rejection or removal of slow timing samples is used.

#### All 96 extended cases

| Fixture | 0.4.20 median ns | 0.4.21 median ns | 0.4.22 median ns | vs 0.4.21 | vs 0.4.20 |
| --- | --- | --- | --- | --- | --- |
| 16_Bm25_disjoint | 503.709 | 459.792 | 446.250 | -2.95% | -11.41% |
| 16_Bm25_limit_one | 648.166 | 587.416 | 568.042 | -3.30% | -12.36% |
| 16_Bm25_limit_zero | 2.875 | 2.667 | 2.709 | +1.57% | -5.77% |
| 16_Bm25_multi | 1629.000 | 1608.458 | 1593.417 | -0.94% | -2.18% |
| 16_Bm25_punctuation | 142.833 | 144.709 | 143.500 | -0.84% | +0.47% |
| 16_Bm25_repeated | 1176.958 | 1178.041 | 1179.875 | +0.16% | +0.25% |
| 16_Bm25_scope | 990.333 | 956.875 | 939.709 | -1.79% | -5.11% |
| 16_Bm25_single | 1133.875 | 1082.083 | 1073.000 | -0.84% | -5.37% |
| 16_Bm25_unicode | 1339.083 | 1365.583 | 1319.459 | -3.38% | -1.47% |
| 16_Bm25_unknown | 69.291 | 19.666 | 19.833 | +0.85% | -71.38% |
| 16_Bm25_unknown_added | 1161.792 | 1167.916 | 1163.875 | -0.35% | +0.18% |
| 16_Bm25_wide | 17032.875 | 17276.333 | 17212.834 | -0.37% | +1.06% |
| 16_Tfidf_disjoint | 515.375 | 453.750 | 451.041 | -0.60% | -12.48% |
| 16_Tfidf_limit_one | 744.833 | 688.250 | 672.541 | -2.28% | -9.71% |
| 16_Tfidf_limit_zero | 2.959 | 2.708 | 2.667 | -1.51% | -9.87% |
| 16_Tfidf_multi | 1702.583 | 1732.042 | 1685.958 | -2.66% | -0.98% |
| 16_Tfidf_punctuation | 144.209 | 146.125 | 144.750 | -0.94% | +0.38% |
| 16_Tfidf_repeated | 1256.209 | 1301.250 | 1269.542 | -2.44% | +1.06% |
| 16_Tfidf_scope | 1058.333 | 1035.208 | 1001.083 | -3.30% | -5.41% |
| 16_Tfidf_single | 1222.584 | 1195.291 | 1145.875 | -4.13% | -6.27% |
| 16_Tfidf_unicode | 1445.042 | 1426.458 | 1431.250 | +0.34% | -0.95% |
| 16_Tfidf_unknown | 70.459 | 19.834 | 19.875 | +0.21% | -71.79% |
| 16_Tfidf_unknown_added | 1272.500 | 1298.833 | 1265.500 | -2.57% | -0.55% |
| 16_Tfidf_wide | 17168.416 | 17230.750 | 17331.208 | +0.58% | +0.95% |
| 1_Bm25_disjoint | 113.958 | 60.167 | 60.250 | +0.14% | -47.13% |
| 1_Bm25_limit_one | 187.750 | 125.417 | 127.917 | +1.99% | -31.87% |
| 1_Bm25_limit_zero | 2.917 | 2.708 | 2.667 | -1.51% | -8.57% |
| 1_Bm25_multi | 265.250 | 262.583 | 266.583 | +1.52% | +0.50% |
| 1_Bm25_punctuation | 144.125 | 142.167 | 142.875 | +0.50% | -0.87% |
| 1_Bm25_repeated | 267.917 | 282.000 | 264.542 | -6.19% | -1.26% |
| 1_Bm25_scope | 192.750 | 132.000 | 132.875 | +0.66% | -31.06% |
| 1_Bm25_single | 227.209 | 148.875 | 145.666 | -2.16% | -35.89% |
| 1_Bm25_unicode | 434.375 | 434.666 | 434.917 | +0.06% | +0.12% |
| 1_Bm25_unknown | 70.709 | 19.625 | 19.708 | +0.42% | -72.13% |
| 1_Bm25_unknown_added | 230.041 | 228.583 | 230.250 | +0.73% | +0.09% |
| 1_Bm25_wide | 4385.750 | 4380.625 | 4362.167 | -0.42% | -0.54% |
| 1_Tfidf_disjoint | 115.583 | 60.291 | 60.333 | +0.07% | -47.80% |
| 1_Tfidf_limit_one | 195.458 | 133.125 | 133.000 | -0.09% | -31.95% |
| 1_Tfidf_limit_zero | 2.958 | 2.667 | 2.667 | +0.00% | -9.84% |
| 1_Tfidf_multi | 265.458 | 267.834 | 263.334 | -1.68% | -0.80% |
| 1_Tfidf_punctuation | 142.834 | 142.708 | 145.541 | +1.99% | +1.90% |
| 1_Tfidf_repeated | 255.875 | 259.292 | 251.334 | -3.07% | -1.77% |
| 1_Tfidf_scope | 199.500 | 141.334 | 140.291 | -0.74% | -29.68% |
| 1_Tfidf_single | 193.292 | 133.750 | 134.792 | +0.78% | -30.27% |
| 1_Tfidf_unicode | 443.541 | 434.208 | 432.583 | -0.37% | -2.47% |
| 1_Tfidf_unknown | 71.250 | 19.792 | 19.834 | +0.21% | -72.16% |
| 1_Tfidf_unknown_added | 230.875 | 233.833 | 228.834 | -2.14% | -0.88% |
| 1_Tfidf_wide | 4463.084 | 4502.000 | 4556.250 | +1.21% | +2.09% |
| 33_Bm25_disjoint | 994.041 | 973.709 | 927.917 | -4.70% | -6.65% |
| 33_Bm25_limit_one | 1273.792 | 1178.958 | 1136.208 | -3.63% | -10.80% |
| 33_Bm25_limit_zero | 3.042 | 2.667 | 2.666 | -0.04% | -12.36% |
| 33_Bm25_multi | 2413.542 | 2509.292 | 2456.334 | -2.11% | +1.77% |
| 33_Bm25_punctuation | 143.750 | 143.916 | 142.000 | -1.33% | -1.22% |
| 33_Bm25_repeated | 1807.125 | 1771.916 | 1787.792 | +0.90% | -1.07% |
| 33_Bm25_scope | 1813.917 | 1769.625 | 1728.042 | -2.35% | -4.73% |
| 33_Bm25_single | 1764.750 | 1644.333 | 1622.959 | -1.30% | -8.03% |
| 33_Bm25_unicode | 1914.750 | 1913.250 | 1867.500 | -2.39% | -2.47% |
| 33_Bm25_unknown | 70.000 | 19.667 | 19.750 | +0.42% | -71.79% |
| 33_Bm25_unknown_added | 1729.167 | 1770.833 | 1768.541 | -0.13% | +2.28% |
| 33_Bm25_wide | 26766.542 | 26487.083 | 26073.000 | -1.56% | -2.59% |
| 33_Tfidf_disjoint | 987.500 | 961.083 | 962.000 | +0.10% | -2.58% |
| 33_Tfidf_limit_one | 1379.958 | 1328.209 | 1332.041 | +0.29% | -3.47% |
| 33_Tfidf_limit_zero | 2.917 | 2.708 | 2.708 | +0.00% | -7.16% |
| 33_Tfidf_multi | 2712.084 | 2699.541 | 2599.333 | -3.71% | -4.16% |
| 33_Tfidf_punctuation | 147.167 | 143.500 | 144.375 | +0.61% | -1.90% |
| 33_Tfidf_repeated | 1956.500 | 1963.875 | 1928.166 | -1.82% | -1.45% |
| 33_Tfidf_scope | 1895.292 | 1921.958 | 1829.416 | -4.81% | -3.48% |
| 33_Tfidf_single | 1908.333 | 1833.917 | 1820.167 | -0.75% | -4.62% |
| 33_Tfidf_unicode | 2079.625 | 2032.667 | 2003.916 | -1.41% | -3.64% |
| 33_Tfidf_unknown | 68.458 | 19.875 | 19.792 | -0.42% | -71.09% |
| 33_Tfidf_unknown_added | 1950.000 | 1950.166 | 1888.625 | -3.16% | -3.15% |
| 33_Tfidf_wide | 26711.500 | 26616.791 | 26704.916 | +0.33% | -0.02% |
| 4_Bm25_disjoint | 163.458 | 110.459 | 111.209 | +0.68% | -31.96% |
| 4_Bm25_limit_one | 244.291 | 191.042 | 189.958 | -0.57% | -22.24% |
| 4_Bm25_limit_zero | 2.916 | 2.625 | 2.666 | +1.56% | -8.57% |
| 4_Bm25_multi | 542.167 | 546.792 | 547.792 | +0.18% | +1.04% |
| 4_Bm25_punctuation | 144.292 | 144.083 | 145.167 | +0.75% | +0.61% |
| 4_Bm25_repeated | 429.625 | 426.958 | 436.084 | +2.14% | +1.50% |
| 4_Bm25_scope | 287.959 | 237.375 | 235.541 | -0.77% | -18.20% |
| 4_Bm25_single | 379.083 | 315.041 | 324.833 | +3.11% | -14.31% |
| 4_Bm25_unicode | 603.666 | 612.250 | 608.375 | -0.63% | +0.78% |
| 4_Bm25_unknown | 69.708 | 19.667 | 19.792 | +0.64% | -71.61% |
| 4_Bm25_unknown_added | 406.333 | 413.500 | 417.083 | +0.87% | +2.65% |
| 4_Bm25_wide | 7237.417 | 7441.375 | 7444.750 | +0.05% | +2.86% |
| 4_Tfidf_disjoint | 165.250 | 115.750 | 112.709 | -2.63% | -31.79% |
| 4_Tfidf_limit_one | 269.833 | 212.333 | 215.208 | +1.35% | -20.24% |
| 4_Tfidf_limit_zero | 2.917 | 3.625 | 3.167 | -12.63% | +8.57% |
| 4_Tfidf_multi | 577.625 | 580.541 | 582.542 | +0.34% | +0.85% |
| 4_Tfidf_punctuation | 146.167 | 148.167 | 143.916 | -2.87% | -1.54% |
| 4_Tfidf_repeated | 451.083 | 461.583 | 465.417 | +0.83% | +3.18% |
| 4_Tfidf_scope | 297.583 | 250.708 | 244.375 | -2.53% | -17.88% |
| 4_Tfidf_single | 394.458 | 346.500 | 350.792 | +1.24% | -11.07% |
| 4_Tfidf_unicode | 638.583 | 645.292 | 643.625 | -0.26% | +0.79% |
| 4_Tfidf_unknown | 68.917 | 19.750 | 19.875 | +0.63% | -71.16% |
| 4_Tfidf_unknown_added | 436.083 | 449.708 | 441.125 | -1.91% | +1.16% |
| 4_Tfidf_wide | 7281.833 | 7528.917 | 7510.625 | -0.24% | +3.14% |

Higher candidate medians versus 0.4.21: **39 of 96**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](https://github.com/LEVOGNE/airust/blob/v0.4.22/docs/measurements/0.4.22/small-confirmation-summary.json). No threshold-based rejection or removal of slow timing samples is used.

## Direct canonical search with explicit control modes

Fourteen processes (seven alternating pairs) retain one warmed Engine each. Ten rounds rotate plain, deadline and progress within each query/ranker pair: **8,400 timed Retriever calls**, excluding loading/readiness, stdio and profiling. Each progress request includes an observer and the same deadline/cancellation setup. Every signature agrees; **140 complete original response JSON pairs match**, and control modes also match plain output. Full readiness source text stays private. These are direct retrieval boundaries, not MCP round trips or capacity evidence.

| Control | Ranker | 0.4.21 median / p95 ms | 0.4.22 median / p95 ms | Median change |
| --- | --- | --- | --- | --- |
| plain | Bm25 | 1.769 / 3.726 | 1.885 / 4.549 | +6.57% |
| plain | Tfidf | 2.263 / 5.118 | 2.286 / 5.918 | +1.03% |
| deadline | Bm25 | 2.145 / 4.619 | 2.303 / 5.595 | +7.38% |
| deadline | Tfidf | 2.693 / 6.275 | 2.764 / 7.771 | +2.65% |
| progress | Bm25 | 2.167 / 4.757 | 2.329 / 5.401 | +7.49% |
| progress | Tfidf | 2.713 / 6.433 | 2.814 / 7.492 | +3.73% |

| Query | Candidates | Progress BM25 ms: old → new | Progress TF-IDF ms: old → new |
| --- | --- | --- | --- |
| 0 | 12365 | 2.114 → 2.389 | 2.833 → 2.807 |
| 1 | 13864 | 2.204 → 2.222 | 2.850 → 3.021 |
| 2 | 24299 | 3.934 → 4.352 | 5.172 → 5.547 |
| 3 | 596 | 0.473 → 0.485 | 0.381 → 0.415 |
| 4 | 15403 | 2.318 → 2.451 | 3.018 → 3.050 |
| 5 | 24140 | 3.770 → 3.886 | 4.941 → 4.612 |
| 6 | 3358 | 0.916 → 0.965 | 0.962 → 1.074 |
| 7 | 29600 | 3.753 → 4.155 | 5.290 → 5.328 |
| 8 | 11504 | 2.048 → 2.204 | 2.363 → 2.494 |
| 9 | 0 | 0.002 → 0.002 | 0.001 → 0.001 |

## Separate canonical three-version reference

Five rotating triples compare fresh exact 0.4.20/0.4.21/candidate servers on canonical knowledge with requested progress. Fifteen servers validate **1,020 actual schema replies**; **680 complete structured comparisons** against the candidate match by RPC ID except checked status version and version-qualified schema URNs. Schemas, source ranges and inputs agree. This control is separate from the two ordinary campaigns. It checks whether the previous large-query benefit remains visible against the older release; it does not establish the cause of any historical regression.

| Version | Ranker | Initial median / p95 ms | Repeated median / p95 ms | Peak child RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.20 | bm25 | 3.109 / 5.259 | 2.661 / 5.071 | 1724.516 |
| 0.4.20 | tfidf | 20.315 / 36.903 | 3.356 / 6.261 | 1724.516 |
| 0.4.21 | bm25 | 2.965 / 5.035 | 2.721 / 4.805 | 1724.828 |
| 0.4.21 | tfidf | 22.536 / 39.154 | 3.349 / 6.616 | 1724.828 |
| 0.4.22 | bm25 | 2.983 / 5.226 | 2.759 / 5.186 | 1724.578 |
| 0.4.22 | tfidf | 20.216 / 38.706 | 3.149 / 6.259 | 1724.578 |

| Query | 0.4.20 repeated TF-IDF ms | 0.4.21 repeated TF-IDF ms | 0.4.22 repeated TF-IDF ms |
| --- | --- | --- | --- |
| 0 | 3.379 | 3.417 | 3.397 |
| 1 | 3.356 | 3.517 | 3.498 |
| 2 | 5.793 | 5.778 | 5.330 |
| 3 | 0.928 | 0.919 | 0.883 |
| 4 | 3.920 | 3.557 | 3.079 |
| 5 | 5.952 | 5.334 | 5.446 |
| 6 | 1.599 | 1.671 | 1.364 |
| 7 | 5.190 | 5.967 | 5.075 |
| 8 | 2.908 | 2.957 | 2.948 |
| 9 | 0.164 | 0.167 | 0.149 |

## Canonical first-pass heap traffic

Two instrumented processes index all 167,942 records before profiling the same ten initial TF-IDF queries, then drop results before net deltas. The existing direct candidate collection remains; the previous large-query posting cursors and norm memo/cache still exist. Heap traffic, net persistent hints, stack state and whole-process RSS are different costs. Instrumented durations are excluded from latency comparisons.

| Metric | 0.4.21 | 0.4.22 |
| --- | --- | --- |
| Retained hint bytes | 2228312 | 2228312 |
| Retained hint blocks | 2 | 2 |
| Total query allocated bytes | 7213183 | 7213183 |
| Total query allocation blocks | 409 | 409 |

## Development trials and host-load limits

The full three-triple development MCP pilot is retained separately from all final campaigns. Its source snapshots include the MCP implementation and Core; all complete compared structured results agree. Initial/repeated data stay separate and no final sample is replaced by a pilot or a faster rerun. Earlier releases retain their own rejected trials. These are development observations, not fresh production or semantic holdouts.

Two-second monitoring retains **619 snapshots**, including **257 with other rustc processes**, maximum 22. All managed compilers finish before timing. Other workloads are not stopped, and snapshots do not prove continuous idleness or identify a timing cause. CPU counts include the active benchmark workers.

| Stage start | Other rustc count | Other rustc CPU total | Processes over 50% CPU |
| --- | --- | --- | --- |
| small-mcp-standard | 0 | 0.000 | 3 |
| small-reference | 1 | 7.900 | 1 |
| large-standard | 0 | 0.000 | 1 |
| wide-standard | 0 | 0.000 | 0 |
| retrieval | 0 | 0.000 | 2 |
| query | 0 | 0.000 | 0 |
| build | 0 | 0.000 | 0 |
| controlled | 3 | 602.700 | 2 |
| mcp | 0 | 0.000 | 2 |
| corpus | 1 | 99.400 | 4 |
| mcp-confirmation | 0 | 0.000 | 3 |
| reference | 0 | 0.000 | 3 |
| wide-confirmation | 1 | 100.000 | 4 |
| wide-extended | 0 | 0.000 | 2 |
| large-confirmation | 0 | 0.000 | 1 |
| small-confirmation | 0 | 0.000 | 1 |
| small-mcp-confirmation | 0 | 0.000 | 3 |
| limits | 0 | 0.000 | 1 |

All observed slower cases remain in their own distributions; no sample is replaced because of load. One machine, randomized hashing, exposed questions, tuned fixtures and finite trials limit generalization.

## Retained Core controls

These controls retain positive and negative timing changes. Single known-term requests retain the direct candidate path and exactly two retain ordered merging. Wider requests keep the heap/set strategies; large frequency cursors remain available afterward. Large sorted candidate sets can use the bounded document cursor after 128 candidates; earlier shared explanation intersection remains. Read every row independently, including slower cases. Seven alternating pairs give fourteen plain processes per family. Norms are initialized for warm timing; two separate DHAT processes measure allocations. Instrumented durations are excluded from latency comparisons. Every result signature agrees and temporary-call retained deltas are zero.

### Small warm Core: 84,000 calls on 1/4/16 entries

| Fixture | Median change | Blocks/call: old → new | Allocated bytes/call: old → new |
| --- | --- | --- | --- |
| tiny16_bm25 | +0.15% | 26.0 → 26.0 | 2156.0 → 2156.0 |
| tiny16_tfidf | -0.94% | 26.0 → 26.0 | 2156.0 → 2156.0 |
| tiny4_bm25 | -0.01% | 12.0 → 12.0 | 824.0 → 824.0 |
| tiny4_tfidf | +0.35% | 12.0 → 12.0 | 824.0 → 824.0 |
| tiny_bm25 | +2.42% | 6.0 → 6.0 | 398.0 → 398.0 |
| tiny_tfidf | -1.60% | 6.0 → 6.0 | 398.0 → 398.0 |

### All 25 warm controls: 282,800 calls

| Fixture | Median change | Blocks/call: old → new | Allocated bytes/call: old → new |
| --- | --- | --- | --- |
| coverage_repeated | +0.23% | 1047.0 → 1047.0 | 92942.0 → 92942.0 |
| coverage_small | -0.18% | 784.0 → 784.0 | 78046.0 → 78046.0 |
| excerpt_ascii | +1.97% | 762.0 → 762.0 | 252438.0 → 252438.0 |
| excerpt_mixed | +1.30% | 569.0 → 569.0 | 417113.0 → 417113.0 |
| excerpt_unicode | +1.16% | 197.0 → 197.0 | 72254.0 → 72254.0 |
| long_repeated | +0.29% | 4.0 → 4.0 | 18798.0 → 18798.0 |
| mixed_unicode | +0.35% | 7.0 → 7.0 | 3155.0 → 3155.0 |
| punctuation | +0.91% | 1.0 → 1.0 | 768.0 → 768.0 |
| relevant_retrieval | +2.87% | 812.0 → 812.0 | 93562.0 → 93562.0 |
| repeated_search | -2.68% | 35.0 → 35.0 | 19432.0 → 19432.0 |
| repeated_short | +3.40% | 4.0 → 4.0 | 1656.0 → 1656.0 |
| sparse_bm25_128 | -0.10% | 359.0 → 359.0 | 29074.0 → 29074.0 |
| sparse_bm25_33 | -0.01% | 241.0 → 241.0 | 16309.0 → 16309.0 |
| sparse_bm25_512 | +0.74% | 1337.0 → 1337.0 | 119124.0 → 119124.0 |
| sparse_tfidf_128 | +2.76% | 474.0 → 474.0 | 35140.0 → 35140.0 |
| sparse_tfidf_33 | +0.35% | 241.0 → 241.0 | 16309.0 → 16309.0 |
| sparse_tfidf_512 | -1.38% | 1337.0 → 1337.0 | 119124.0 → 119124.0 |
| tfidf_union_32 | -1.55% | 244.0 → 244.0 | 19834.0 → 19834.0 |
| tfidf_union_33 | +0.90% | 296.0 → 296.0 | 27854.0 → 27854.0 |
| tfidf_warm | +0.84% | 71.0 → 71.0 | 19335.0 → 19335.0 |
| union_32 | +1.56% | 244.0 → 244.0 | 19834.0 → 19834.0 |
| union_33 | -1.08% | 296.0 → 296.0 | 27854.0 → 27854.0 |
| unique_long | -1.04% | 299.0 → 299.0 | 32672.0 → 32672.0 |
| unique_search | +1.42% | 72.0 → 72.0 | 19350.0 → 19350.0 |
| unique_short | +4.74% | 296.0 → 296.0 | 18548.0 → 18548.0 |

### Cold norm and heap controls

Six fixtures make twenty fresh-index-clone calls each across fourteen processes: 1,680 calls. Clone/setup is outside timing. Process/instruction/filesystem caches remain; cold means norm hints, not disks. Separate heap runs drop results before net retained measurements. The existing inline/optional heap word memo and persistent norm storage remain. Large small-term requests may add up to four temporary posting cursors; canonical allocation traffic below records that cost. Heap profiles exclude stack state/control fields and temporary frames.

| Fixture | Old median / p95 ns | New median / p95 ns |
| --- | --- | --- |
| index | 164854.000 / 179365.200 | 153729.000 / 172041.350 |
| wide | 51458.000 / 83175.300 | 45813.000 / 62585.700 |
| sparse | 619229.000 / 679030.900 | 614188.000 / 708253.850 |
| tiny | 750.000 / 2710.100 | 708.000 / 2877.100 |
| tiny4 | 1084.000 / 1631.250 | 1042.000 / 1586.050 |
| tiny16 | 3584.000 / 5958.050 | 3500.000 / 5169.050 |

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
| new_group_query_ns | 175625.000 / 374510.400 | 253104.000 / 394008.300 |
| post_readers_first_ns | 213500.000 / 270967.200 | 240125.000 / 259775.000 |
| post_readers_second_ns | 113208.000 / 123066.400 | 112750.000 / 138133.000 |

The private document corpus retains 72 Markdown files, 2,755,468 bytes and 1,910 chunks. All 342 authored questions (276 positive, 66 unsupported) are exposed regressions, not fresh holdouts or logged production queries. Literal BM25, prepared BM25 and coverage retain six passages, 100 candidates, 2,000 bytes/pass and 12,000 total bytes. All **1,026 complete original response JSON strings match byte-for-byte**, including scores, ordering, evidence, source text and positions. One warm-up and five repeats/question/configuration/version give **10,260 timed round trips**. These three configurations are BM25 controls; independent score oracles and actual protocol pairs cover TF-IDF. No new relevance judgment or model reply is produced.

All document timing aggregates and per-question results remain in the records. Construction uses forty plain processes on canonical data and three synthetic distributions, plus six separate profiles. Parsing/generation is outside construction timing; stats/signatures/live derived heap agree. The execution/shared-index worker checks all 167,942 entries through 128 changes/restoration, rebuilds, JSON round trips, retained clones, parallel readers and cancellation/deadline rejection. Portable exports and source hashes agree.

### Document round trips

| Case | Old median / p95 ms | New median / p95 ms |
| --- | --- | --- |
| baseline | 1.168 / 2.209 | 1.215 / 2.283 |
| prepared | 1.394 / 2.445 | 1.414 / 2.498 |
| evidence | 1.378 / 2.575 | 1.441 / 2.512 |

### Construction only

| Case | Old median / p95 ms | New median / p95 ms |
| --- | --- | --- |
| canonical | 7587.987 / 8026.158 | 7651.365 / 7966.542 |
| ascii | 20.582 / 22.088 | 20.062 / 21.645 |
| mixed | 47.968 / 48.148 | 49.704 / 51.603 |
| long_tokens | 2.204 / 2.337 | 2.154 / 2.458 |

## Limits and preservation

Measurements use one Apple M5 Max on battery power at campaign start with 64 GB, Rust 1.98.0 and Python 3.15.2. Stage-start CPU/compiler snapshots remain. Own compilers finish before timing; this does not establish continuously idle hardware. Randomized hashing, scheduling, one machine/corpus, exposed questions and finite process counts limit generalization. The separate development pilot remains; assess final per-case medians and tails independently. Host-load sampling records observations rather than continuous idleness. No universal latency, total-memory, semantic-quality, token-saving or security-audit claim is made.

Canonical knowledge remains 167,942 entries from 35 PDFs and 49,086 physical pages, SHA-256 `13168d87872a137355fd8499c90a6084f52243a2ec7456f29b5fcb2a1c7423c3`. The PDF parser is unchanged; this is not a fresh large-PDF extraction campaign. Lexical overlap and exact quotes do not prove factual support or entailment. No model, daemon or DenkMaps integration is added.

Preserve PDFs, knowledge, snapshots, all rejected/selected pilot and final measurements, slower results, raw transcripts, profiles, exact small executables and locks. Raw measurement JSON stays in the tagged GitHub repository and local archives, outside the registry source package; API guides, Markdown reports, method source, examples and required fixtures remain in that package. After verified publication archive package/proof, then delete only the named temporary Cargo/Miri targets. Default features remain empty.

Power metadata correction: the original AC label was hard-coded and incorrect. The preserved environment record reports Battery Power at campaign start. Power was not continuously sampled. Timing samples remain unchanged; see [benchmark errata](BENCHMARK-ERRATA.md).
