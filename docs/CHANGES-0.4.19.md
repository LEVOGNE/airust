# AIRust 0.4.19: less repeated work in broad search explanations

Broad queries previously inspected every known query word again for each returned winner, even when that document contained few different words. Scoring already had a cheaper exact intersection for sparse documents. Both now share the same checked dictionary intersection. For more than 32 known query terms and more than twice the document vocabulary, walk sorted document words and binary-search the prepared query. Otherwise keep the query-side scan. Output matched terms stay alphabetically ordered and exact; score arithmetic and contribution order remain unchanged. There is no new dependency, cache, stored field, API, switch or data migration.

Explanation work checks cancellation/deadlines before its first inspected word and every 128 inspected words, as well as before each winner. Sparse explanations inspect fewer words and can make fewer checkpoints because they do less work. Failed checks discard the partial explanation. Posting visits, candidate collection, score/norm controls, scopes, final ties and source positions retain their contracts. The single-term path, non-inlined two-term merge, large-query frequency cursors and MCP timer behavior remain.

The primary evidence is the three separate broad-query campaigns below, including both rankers and all slower cases. Two ordinary campaigns additionally observe canonical repeated MCP TF-IDF median changes of **+1.80% / +0.77%** versus freshly built exact registry 0.4.18. One-entry repeated MCP TF-IDF changes are **+0.09% / -0.76%**. Negative means shorter observed time. These are finite measurements on one host, not a universal speedup or statistical-significance claim. No new relevance or model-answer judgment is made.

## Executed validation and API contract

The full matrix passes **243 default tests, 893 all-feature checks including 39 documentation examples, 39 isolated feature entries and five Python tests**. Strict Clippy/docs, Rust 1.89 provider-neutral and Rust 1.92 all-feature checks pass. API comparison with tagged 0.4.18 reports **229 pass, 31 skip**. The independent Rust 1.89 consumer covers retrieval, coverage, quotes, document positions and MCP/schema construction. The unchanged PDF backend passes 14 regressions and its 381-source-file provenance check. The refreshed dependency audit reports zero known vulnerabilities in its checked database; maintenance notices remain in the resource record.

Two new regressions compare matched words and exact frequencies with an independent sorted intersection across branch boundaries, repeated/unknown words and Unicode, and reject partially built explanations when a checkpoint fails. The existing independent full-scan oracle covers 1,025 native records, both rankers, query widths, scopes, limits and cold/warm states. Mutation, reused IDs, retained clones, sources/spans, malformed MCP input, cancellation, absolute deadlines, stalled readers, admission, duplicate tokens, disconnect and drain remain tested.

**Twenty-five finite Tree Borrows Miri cases pass.** The new intersection case scales vocabulary and widths; the interruption case uses the query-side branch in Miri and the sparse branch on native 512-term input. Only extra floating-point rounding injection is disabled; memory/alias checks remain. Tokio/PDF/FFI/ONNX execution is outside this interpreter scope. These tests are not an independent security or soundness audit. Extracted-package tests, exact-commit CI and a fresh registry consumer gate publication; package checksum, VCS, annotated tag and release asset must agree.

Engine, SearchOptions, Retriever, WorkControl, McpServer and McpLimits usage stays the same. Default features stay empty. Use WorkControl or MCP timeout_ms for cooperative limits as before. Admission, absolute deadlines, worker leases, frame/result/rate budgets, progress delivery and connection cancellation/drain remain. Startup indexing is outside tool deadlines. Read [Core performance](CORE-PERFORMANCE.md), [API](API.md), [cookbook](API-COOKBOOK.md), [MCP](MCP.md) and [security scope](MCP-SECURITY.md).

## Frozen method and development record

Baseline is exact registry 0.4.18, VCS `fd5a0e182ece603e135feb609351cc3559bca4bf`; the older reference is exact registry 0.4.17, VCS `c82e61530c201eed1967293fc0e117ee42a3e1c2`. Every cached registry source file is compared byte-for-byte with its checksum-verified published crate archive before final timing. Candidate 0.4.19 runs the same programs and inputs. Actual worker sources, manifests, locks, executables, runtime and inputs are hashed and preserved. Read the [reproduction method](validation-0.4.19/README.md).

The initial development pilot stopped when 257/512-term queries hit the existing 256-unique-term assessment bound in registry 0.4.17. Its logs and inputs remain. A separate supported pilot uses three rotating fresh-version triples on standard-wide, extended-wide, canonical and tiny MCP plus six small Core cases. Complete compared responses agree. Pilot timing stays separate from every final campaign. Rejected requests are not included in successful-search timing.

## Primary target: broad MCP queries

Each preplanned campaign has seven rotating 0.4.17/0.4.18/0.4.19 triples: **21 fresh servers, 1,428 actual schema-valid replies and 952 complete structured comparisons**. Across the three separate campaigns: **63 servers, 4,284 schema replies and 2,856 complete comparisons**, all agreeing except asserted status version and version-qualified schema URNs. Standard and confirmation use the same retained 257-entry fixture, sparse 16-term documents, one dense 512-word vocabulary document and the original ten queries. Extended probes 32/33/64/65/128/129/192/255/256 known terms and 255 known terms plus a duplicate and one unknown word. Existing normalized-term and byte bounds remain.

Search timing includes Rust work, serialization, stdio, Python parsing and requested progress. Initial calls, repeated calls, startup, per-query means/medians/tails, per-process totals and RSS are separate. Original standard queries are not all equally wide. Aggregate medians alone can obscure differences; retain every per-query row and raw timing observation. No sample is removed or replaced.

### Standard broad-query campaign

| Ranker / phase | 0.4.17 median / p95 ms | 0.4.18 median / p95 ms | 0.4.19 median / p95 ms | vs 0.4.18 | vs 0.4.17 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 0.523 / 0.658 | 0.544 / 0.656 | 0.503 / 0.601 | -7.52% | -3.93% |
| tfidf / initial | 0.462 / 0.837 | 0.469 / 0.845 | 0.464 / 0.760 | -1.06% | +0.33% |
| bm25 / repeated | 0.429 / 0.737 | 0.426 / 0.746 | 0.424 / 0.626 | -0.59% | -1.08% |
| tfidf / repeated | 0.423 / 0.809 | 0.420 / 0.805 | 0.418 / 0.750 | -0.38% | -1.22% |

| Query | Repeated TF-IDF median ms: 0.4.17 / 0.4.18 / 0.4.19 | vs 0.4.18 | Initial TF-IDF mean ms: 0.4.18 → 0.4.19 |
| --- | --- | --- | --- |
| 0 | 0.390 / 0.394 / 0.396 | +0.57% | 0.524 → 0.497 |
| 1 | 0.379 / 0.380 / 0.386 | +1.52% | 0.430 → 0.439 |
| 2 | 0.431 / 0.424 / 0.418 | -1.43% | 0.486 → 0.463 |
| 3 | 0.797 / 0.801 / 0.730 | -8.89% | 0.856 → 0.763 |
| 4 | 0.407 / 0.409 / 0.415 | +1.52% | 0.422 → 0.415 |
| 5 | 0.382 / 0.375 / 0.389 | +3.69% | 0.454 → 0.433 |
| 6 | 0.431 / 0.429 / 0.418 | -2.42% | 0.447 → 0.428 |
| 7 | 0.772 / 0.757 / 0.685 | -9.51% | 0.794 → 0.711 |
| 8 | 0.383 / 0.386 / 0.380 | -1.60% | 0.397 → 0.387 |
| 9 | 0.785 / 0.773 / 0.704 | -8.88% | 0.783 → 0.697 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.17 | 5.508 | 10.289 | 4.059 | 7.875 |
| 0.4.18 | 5.563 | 10.267 | 3.948 | 7.750 |
| 0.4.19 | 5.208 | 9.798 | 4.045 | 7.812 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](measurements/0.4.19/wide-standard-summary.json).

### Confirmation broad-query campaign

| Ranker / phase | 0.4.17 median / p95 ms | 0.4.18 median / p95 ms | 0.4.19 median / p95 ms | vs 0.4.18 | vs 0.4.17 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 0.530 / 0.689 | 0.541 / 0.651 | 0.502 / 0.601 | -7.19% | -5.22% |
| tfidf / initial | 0.476 / 0.862 | 0.465 / 0.880 | 0.449 / 0.774 | -3.35% | -5.59% |
| bm25 / repeated | 0.432 / 0.678 | 0.442 / 0.700 | 0.422 / 0.622 | -4.55% | -2.31% |
| tfidf / repeated | 0.427 / 0.807 | 0.431 / 0.838 | 0.416 / 0.734 | -3.51% | -2.68% |

| Query | Repeated TF-IDF median ms: 0.4.17 / 0.4.18 / 0.4.19 | vs 0.4.18 | Initial TF-IDF mean ms: 0.4.18 → 0.4.19 |
| --- | --- | --- | --- |
| 0 | 0.393 / 0.396 / 0.400 | +1.14% | 0.487 → 0.486 |
| 1 | 0.390 / 0.386 / 0.376 | -2.76% | 0.432 → 0.429 |
| 2 | 0.435 / 0.441 / 0.421 | -4.46% | 0.468 → 0.456 |
| 3 | 0.799 / 0.794 / 0.725 | -8.68% | 0.890 → 0.772 |
| 4 | 0.410 / 0.411 / 0.406 | -1.26% | 0.429 → 0.413 |
| 5 | 0.380 / 0.389 / 0.379 | -2.50% | 0.452 → 0.417 |
| 6 | 0.434 / 0.425 / 0.416 | -2.11% | 0.463 → 0.445 |
| 7 | 0.769 / 0.782 / 0.697 | -10.87% | 0.823 → 0.724 |
| 8 | 0.384 / 0.397 / 0.381 | -4.04% | 0.392 → 0.395 |
| 9 | 0.789 / 0.817 / 0.711 | -13.04% | 0.781 → 0.701 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.17 | 5.551 | 10.378 | 4.212 | 7.891 |
| 0.4.18 | 5.639 | 10.465 | 4.268 | 7.812 |
| 0.4.19 | 5.241 | 9.851 | 4.110 | 7.828 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](measurements/0.4.19/wide-confirmation-summary.json).

### Extended broad-query campaign

| Ranker / phase | 0.4.17 median / p95 ms | 0.4.18 median / p95 ms | 0.4.19 median / p95 ms | vs 0.4.18 | vs 0.4.17 |
| --- | --- | --- | --- | --- | --- |
| bm25 / initial | 0.651 / 1.257 | 0.645 / 1.238 | 0.584 / 0.986 | -9.50% | -10.29% |
| tfidf / initial | 0.853 / 1.229 | 0.861 / 1.217 | 0.780 / 0.955 | -9.45% | -8.58% |
| bm25 / repeated | 0.602 / 1.178 | 0.602 / 1.187 | 0.539 / 0.936 | -10.52% | -10.52% |
| tfidf / repeated | 0.776 / 1.313 | 0.781 / 1.440 | 0.710 / 1.053 | -9.12% | -8.52% |

| Query | Repeated TF-IDF median ms: 0.4.17 / 0.4.18 / 0.4.19 | vs 0.4.18 | Initial TF-IDF mean ms: 0.4.18 → 0.4.19 |
| --- | --- | --- | --- |
| 0 | 0.402 / 0.398 / 0.402 | +1.03% | 0.494 → 0.492 |
| 1 | 0.411 / 0.426 / 0.414 | -2.97% | 0.422 → 0.436 |
| 2 | 0.438 / 0.437 / 0.429 | -1.98% | 0.468 → 0.477 |
| 3 | 0.467 / 0.471 / 0.462 | -1.81% | 0.464 → 0.495 |
| 4 | 0.768 / 0.769 / 0.710 | -7.57% | 0.852 → 0.788 |
| 5 | 0.787 / 0.789 / 0.709 | -10.18% | 0.866 → 0.789 |
| 6 | 1.001 / 1.025 / 0.833 | -18.72% | 1.052 → 0.876 |
| 7 | 1.726 / 1.817 / 1.465 | -19.39% | 1.201 → 0.944 |
| 8 | 1.149 / 1.158 / 0.907 | -21.65% | 1.199 → 0.932 |
| 9 | 1.188 / 1.149 / 0.918 | -20.08% | 1.168 → 0.921 |

| Version | Initial TF-IDF process sum median ms | Repeated TF-IDF process sum median ms | Startup median ms | Peak RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.17 | 8.257 | 16.732 | 3.877 | 8.312 |
| 0.4.18 | 8.173 | 16.686 | 3.777 | 8.125 |
| 0.4.19 | 7.065 | 14.602 | 3.796 | 8.094 |

All ranker/query means, medians, p95 and individual samples remain in [the complete record](measurements/0.4.19/wide-extended-summary.json).

### Existing limit and recovery checks

Three additional fresh servers reject 257/512 known-term requests with both rankers: twelve expected errors, no partial successful results, error progress and successful later health requests. The existing 256-unique-term assessment limit remains; these are not successful-search latency samples. This also explains the initial stopped development pilot. Large inputs remain subject to cooperative bounds, not an OS CPU/RAM quota.

## Actual MCP: two ordinary campaigns

Each ordinary campaign uses nine cases and five alternating old/new pairs: **90 servers each, 180 total**. Canonical knowledge, NumPy and broad synthetic input request progress; 1/4/16-entry inputs run with and without progress. Each makes ten queries with both rankers, two repeated passes, four concurrent searches, matching/relations, rejected arguments and health; snapshot inputs check exact UTF-8 readbacks.

Together **13,880 actual result/error replies validate against advertised schemas**, and **6,940 complete old/new structured pairs agree by RPC ID**, excluding only checked status version and version-qualified schema URNs. Discovery, progress/no-progress, bounds, ordering, input hashes and EOF shutdown pass. Search and readback time are separate. Child peak RSS includes runtime, buffers and source/index memory, unlike allocation profiles. Retain all slower rows.

### First complete campaign

| Case | Ranker | Initial median / p95 ms: old → new | Repeated median / p95 ms: old → new | Repeated median change |
| --- | --- | --- | --- | --- |
| canonical | bm25 | 2.975 / 4.950 → 3.072 / 5.097 | 2.530 / 4.617 → 2.627 / 4.772 | +3.85% |
| canonical | tfidf | 16.850 / 30.697 → 17.363 / 31.495 | 2.774 / 4.951 → 2.824 / 5.022 | +1.80% |
| numpy | bm25 | 0.922 / 1.218 → 0.917 / 1.259 | 0.788 / 1.114 → 0.794 / 1.092 | +0.72% |
| numpy | tfidf | 1.044 / 2.802 → 1.059 / 2.867 | 0.731 / 1.165 → 0.738 / 1.186 | +0.95% |
| wide | bm25 | 0.517 / 0.645 → 0.501 / 0.571 | 0.425 / 0.697 → 0.410 / 0.625 | -3.49% |
| wide | tfidf | 0.455 / 0.830 → 0.455 / 0.761 | 0.414 / 0.791 → 0.407 / 0.724 | -1.72% |
| tiny | bm25 | 0.136 / 0.233 → 0.130 / 0.221 | 0.117 / 0.151 → 0.116 / 0.145 | -0.75% |
| tiny | tfidf | 0.133 / 0.173 → 0.132 / 0.170 | 0.117 / 0.142 → 0.117 / 0.130 | +0.09% |
| tiny4 | bm25 | 0.189 / 0.312 → 0.192 / 0.315 | 0.171 / 0.191 → 0.174 / 0.190 | +1.52% |
| tiny4 | tfidf | 0.180 / 0.262 → 0.187 / 0.252 | 0.173 / 0.202 → 0.173 / 0.223 | +0.42% |
| tiny16 | bm25 | 0.215 / 0.335 → 0.212 / 0.332 | 0.196 / 0.223 → 0.193 / 0.218 | -1.59% |
| tiny16 | tfidf | 0.212 / 0.323 → 0.214 / 0.310 | 0.197 / 0.234 → 0.194 / 0.230 | -1.47% |
| tiny-no-progress | bm25 | 0.077 / 0.172 → 0.080 / 0.171 | 0.069 / 0.123 → 0.070 / 0.152 | +1.45% |
| tiny-no-progress | tfidf | 0.083 / 0.119 → 0.077 / 0.124 | 0.068 / 0.088 → 0.070 / 0.081 | +1.58% |
| tiny4-no-progress | bm25 | 0.136 / 0.247 → 0.137 / 0.257 | 0.125 / 0.157 → 0.125 / 0.145 | +0.13% |
| tiny4-no-progress | tfidf | 0.133 / 0.182 → 0.131 / 0.187 | 0.126 / 0.173 → 0.124 / 0.162 | -1.80% |
| tiny16-no-progress | bm25 | 0.153 / 0.267 → 0.156 / 0.283 | 0.147 / 0.171 → 0.145 / 0.163 | -1.11% |
| tiny16-no-progress | tfidf | 0.156 / 0.266 → 0.158 / 0.255 | 0.148 / 0.184 → 0.147 / 0.189 | -0.59% |

| Case | Startup median ms: old → new | First TF-IDF median ms: old → new | Peak RSS median MiB: old → new |
| --- | --- | --- | --- |
| canonical | 4665.106 → 4703.946 | 20.038 → 20.940 | 1724.094 → 1724.203 |
| numpy | 134.987 → 135.452 | 1.558 → 1.580 | 57.094 → 57.125 |
| wide | 3.819 → 3.835 | 0.487 → 0.486 | 7.875 → 7.719 |
| tiny | 3.421 → 3.200 | 0.176 → 0.170 | 5.797 → 5.703 |
| tiny4 | 3.343 → 3.271 | 0.264 → 0.256 | 5.969 → 5.812 |
| tiny16 | 3.313 → 3.342 | 0.280 → 0.281 | 6.125 → 6.047 |
| tiny-no-progress | 3.376 → 3.393 | 0.115 → 0.126 | 5.812 → 5.703 |
| tiny4-no-progress | 3.244 → 3.296 | 0.185 → 0.188 | 5.938 → 5.844 |
| tiny16-no-progress | 3.239 → 3.382 | 0.233 → 0.233 | 6.078 → 6.062 |

### Second preplanned complete campaign

| Case | Ranker | Initial median / p95 ms: old → new | Repeated median / p95 ms: old → new | Repeated median change |
| --- | --- | --- | --- | --- |
| canonical | bm25 | 3.008 / 5.080 → 3.033 / 4.977 | 2.610 / 4.699 → 2.649 / 4.700 | +1.52% |
| canonical | tfidf | 17.314 / 31.133 → 17.348 / 30.666 | 2.796 / 5.071 → 2.817 / 4.963 | +0.77% |
| numpy | bm25 | 0.917 / 1.233 → 0.918 / 1.245 | 0.794 / 1.123 → 0.769 / 1.082 | -3.13% |
| numpy | tfidf | 1.067 / 2.792 → 1.052 / 2.786 | 0.756 / 1.143 → 0.726 / 1.135 | -3.93% |
| wide | bm25 | 0.517 / 0.626 → 0.508 / 0.567 | 0.417 / 0.667 → 0.421 / 0.608 | +0.79% |
| wide | tfidf | 0.459 / 0.834 → 0.460 / 0.747 | 0.413 / 0.794 → 0.409 / 0.717 | -1.06% |
| tiny | bm25 | 0.134 / 0.233 → 0.133 / 0.231 | 0.118 / 0.154 → 0.117 / 0.149 | -0.69% |
| tiny | tfidf | 0.131 / 0.169 → 0.130 / 0.167 | 0.118 / 0.133 → 0.117 / 0.137 | -0.76% |
| tiny4 | bm25 | 0.188 / 0.316 → 0.188 / 0.311 | 0.172 / 0.193 → 0.172 / 0.197 | -0.36% |
| tiny4 | tfidf | 0.181 / 0.256 → 0.181 / 0.250 | 0.172 / 0.210 → 0.171 / 0.226 | -0.82% |
| tiny16 | bm25 | 0.211 / 0.353 → 0.209 / 0.341 | 0.194 / 0.217 → 0.193 / 0.213 | -0.62% |
| tiny16 | tfidf | 0.213 / 0.337 → 0.204 / 0.305 | 0.198 / 0.230 → 0.195 / 0.231 | -1.28% |
| tiny-no-progress | bm25 | 0.078 / 0.168 → 0.080 / 0.168 | 0.070 / 0.104 → 0.070 / 0.101 | -0.71% |
| tiny-no-progress | tfidf | 0.077 / 0.129 → 0.077 / 0.132 | 0.070 / 0.084 → 0.070 / 0.079 | -0.59% |
| tiny4-no-progress | bm25 | 0.140 / 0.257 → 0.133 / 0.249 | 0.126 / 0.151 → 0.126 / 0.144 | +0.05% |
| tiny4-no-progress | tfidf | 0.132 / 0.184 → 0.129 / 0.192 | 0.125 / 0.165 → 0.125 / 0.165 | +0.13% |
| tiny16-no-progress | bm25 | 0.151 / 0.282 → 0.154 / 0.286 | 0.144 / 0.166 → 0.145 / 0.179 | +0.69% |
| tiny16-no-progress | tfidf | 0.154 / 0.265 → 0.154 / 0.264 | 0.145 / 0.177 → 0.147 / 0.184 | +1.39% |

| Case | Startup median ms: old → new | First TF-IDF median ms: old → new | Peak RSS median MiB: old → new |
| --- | --- | --- | --- |
| canonical | 4683.466 → 4691.490 | 20.712 → 20.858 | 1724.109 → 1724.172 |
| numpy | 137.314 → 136.156 | 1.576 → 1.553 | 57.047 → 57.031 |
| wide | 4.007 → 4.076 | 0.494 → 0.491 | 7.750 → 7.859 |
| tiny | 3.259 → 3.369 | 0.174 → 0.168 | 5.812 → 5.703 |
| tiny4 | 3.193 → 3.189 | 0.258 → 0.250 | 5.906 → 5.875 |
| tiny16 | 3.385 → 3.297 | 0.282 → 0.277 | 6.141 → 6.031 |
| tiny-no-progress | 3.521 → 3.395 | 0.131 → 0.134 | 5.828 → 5.703 |
| tiny4-no-progress | 3.387 → 3.149 | 0.183 → 0.196 | 5.938 → 5.844 |
| tiny16-no-progress | 3.566 → 3.604 | 0.212 → 0.227 | 6.094 → 5.984 |

## Small Core regression controls

These separate campaigns compare freshly built exact registry 0.4.17 and 0.4.18 with the candidate, using the same programs. Seven rotating triples per campaign run both the original six cases and a 96-case extension. Warm-up, full original-JSON comparison and index setup occur before timers. Each campaign has **42 processes and 2,142,000 timed Core calls**. Both together have 84 processes, 4,284,000 calls and **2,688 complete original matrix-response comparisons**, with identical results. Signature checks also cover every original-six and timed matrix sample. This is repeated warm Core time, without MCP transport or DHAT instrumentation.

Keep the two campaigns separate. Original six-case calls build SearchOptions inside the measured loop; the extension reuses fixed options. Do not compare their absolute times or merge distributions. The 96 cases combine four entry counts, both rankers and twelve fixed query/filter/limit cases. Wide query preparation, scope rejection, unknown-only queries and zero limits are retained even though they are outside the optimized sparse explanation work. Historical absolute values with different programs/loads are not a reference substitute.

### First small Core campaign

#### Original six warm cases

| Fixture | 0.4.17 median ns | 0.4.18 median ns | 0.4.19 median ns | vs 0.4.18 | vs 0.4.17 |
| --- | --- | --- | --- | --- | --- |
| tiny16_bm25 | 989.750 | 987.916 | 976.458 | -1.16% | -1.34% |
| tiny16_tfidf | 1042.042 | 1051.209 | 1045.000 | -0.59% | +0.28% |
| tiny4_bm25 | 339.625 | 344.875 | 340.625 | -1.23% | +0.29% |
| tiny4_tfidf | 356.125 | 363.083 | 359.583 | -0.96% | +0.97% |
| tiny_bm25 | 189.250 | 185.500 | 185.375 | -0.07% | -2.05% |
| tiny_tfidf | 178.917 | 185.209 | 185.458 | +0.13% | +3.66% |

Higher candidate medians versus 0.4.18: **1 of 6**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](measurements/0.4.19/small-reference-summary.json). No threshold-based rejection or removal of slow timing samples is used.

#### All 96 extended cases

| Fixture | 0.4.17 median ns | 0.4.18 median ns | 0.4.19 median ns | vs 0.4.18 | vs 0.4.17 |
| --- | --- | --- | --- | --- | --- |
| 16_Bm25_disjoint | 476.833 | 471.833 | 463.083 | -1.85% | -2.88% |
| 16_Bm25_limit_one | 627.834 | 641.917 | 629.750 | -1.90% | +0.31% |
| 16_Bm25_limit_zero | 2.667 | 2.667 | 2.666 | -0.04% | -0.04% |
| 16_Bm25_multi | 2138.625 | 1609.833 | 1560.000 | -3.10% | -27.06% |
| 16_Bm25_punctuation | 147.292 | 140.875 | 143.958 | +2.19% | -2.26% |
| 16_Bm25_repeated | 1141.125 | 1156.542 | 1169.125 | +1.09% | +2.45% |
| 16_Bm25_scope | 991.375 | 968.916 | 972.292 | +0.35% | -1.92% |
| 16_Bm25_single | 1087.375 | 1109.459 | 1133.708 | +2.19% | +4.26% |
| 16_Bm25_unicode | 1369.125 | 1301.916 | 1327.500 | +1.97% | -3.04% |
| 16_Bm25_unknown | 69.917 | 68.000 | 68.250 | +0.37% | -2.38% |
| 16_Bm25_unknown_added | 1161.167 | 1145.792 | 1137.125 | -0.76% | -2.07% |
| 16_Bm25_wide | 18776.416 | 19107.833 | 17170.875 | -10.14% | -8.55% |
| 16_Tfidf_disjoint | 475.125 | 478.541 | 472.333 | -1.30% | -0.59% |
| 16_Tfidf_limit_one | 705.750 | 708.333 | 701.291 | -0.99% | -0.63% |
| 16_Tfidf_limit_zero | 2.667 | 2.667 | 2.667 | +0.00% | +0.00% |
| 16_Tfidf_multi | 2163.958 | 1638.208 | 1648.958 | +0.66% | -23.80% |
| 16_Tfidf_punctuation | 144.250 | 145.333 | 145.125 | -0.14% | +0.61% |
| 16_Tfidf_repeated | 1225.625 | 1250.042 | 1226.708 | -1.87% | +0.09% |
| 16_Tfidf_scope | 1043.084 | 1034.875 | 1034.292 | -0.06% | -0.84% |
| 16_Tfidf_single | 1168.542 | 1207.208 | 1187.208 | -1.66% | +1.60% |
| 16_Tfidf_unicode | 1392.875 | 1399.375 | 1377.792 | -1.54% | -1.08% |
| 16_Tfidf_unknown | 67.958 | 69.584 | 67.417 | -3.11% | -0.80% |
| 16_Tfidf_unknown_added | 1217.667 | 1221.333 | 1222.167 | +0.07% | +0.37% |
| 16_Tfidf_wide | 19011.375 | 18747.750 | 16881.042 | -9.96% | -11.21% |
| 1_Bm25_disjoint | 111.208 | 114.042 | 116.000 | +1.72% | +4.31% |
| 1_Bm25_limit_one | 179.625 | 185.000 | 191.833 | +3.69% | +6.80% |
| 1_Bm25_limit_zero | 2.625 | 2.667 | 2.708 | +1.54% | +3.16% |
| 1_Bm25_multi | 283.292 | 256.667 | 269.500 | +5.00% | -4.87% |
| 1_Bm25_punctuation | 144.541 | 144.083 | 144.833 | +0.52% | +0.20% |
| 1_Bm25_repeated | 260.875 | 248.959 | 274.291 | +10.18% | +5.14% |
| 1_Bm25_scope | 189.084 | 194.708 | 200.583 | +3.02% | +6.08% |
| 1_Bm25_single | 239.583 | 231.125 | 228.750 | -1.03% | -4.52% |
| 1_Bm25_unicode | 434.125 | 421.792 | 430.041 | +1.96% | -0.94% |
| 1_Bm25_unknown | 69.292 | 71.917 | 72.667 | +1.04% | +4.87% |
| 1_Bm25_unknown_added | 217.917 | 219.333 | 244.625 | +11.53% | +12.26% |
| 1_Bm25_wide | 4330.000 | 4357.958 | 4490.958 | +3.05% | +3.72% |
| 1_Tfidf_disjoint | 108.333 | 114.625 | 113.875 | -0.65% | +5.12% |
| 1_Tfidf_limit_one | 188.334 | 187.333 | 196.625 | +4.96% | +4.40% |
| 1_Tfidf_limit_zero | 2.667 | 2.708 | 2.625 | -3.06% | -1.57% |
| 1_Tfidf_multi | 289.417 | 260.375 | 258.750 | -0.62% | -10.60% |
| 1_Tfidf_punctuation | 143.625 | 143.250 | 146.875 | +2.53% | +2.26% |
| 1_Tfidf_repeated | 249.625 | 242.042 | 252.666 | +4.39% | +1.22% |
| 1_Tfidf_scope | 195.917 | 193.833 | 204.125 | +5.31% | +4.19% |
| 1_Tfidf_single | 186.625 | 187.584 | 197.292 | +5.18% | +5.72% |
| 1_Tfidf_unicode | 430.209 | 431.792 | 434.500 | +0.63% | +1.00% |
| 1_Tfidf_unknown | 68.708 | 67.666 | 68.959 | +1.91% | +0.37% |
| 1_Tfidf_unknown_added | 223.125 | 222.167 | 227.500 | +2.40% | +1.96% |
| 1_Tfidf_wide | 4512.833 | 4295.917 | 4430.542 | +3.13% | -1.82% |
| 33_Bm25_disjoint | 949.375 | 945.750 | 946.208 | +0.05% | -0.33% |
| 33_Bm25_limit_one | 1248.792 | 1178.833 | 1173.792 | -0.43% | -6.01% |
| 33_Bm25_limit_zero | 2.667 | 2.667 | 2.666 | -0.04% | -0.04% |
| 33_Bm25_multi | 3428.709 | 2372.416 | 2364.417 | -0.34% | -31.04% |
| 33_Bm25_punctuation | 144.917 | 140.291 | 142.666 | +1.69% | -1.55% |
| 33_Bm25_repeated | 1768.000 | 1715.541 | 1716.708 | +0.07% | -2.90% |
| 33_Bm25_scope | 1735.458 | 1720.125 | 1741.625 | +1.25% | +0.36% |
| 33_Bm25_single | 1723.042 | 1692.208 | 1657.542 | -2.05% | -3.80% |
| 33_Bm25_unicode | 1864.084 | 1826.000 | 1837.500 | +0.63% | -1.43% |
| 33_Bm25_unknown | 67.667 | 67.834 | 67.709 | -0.18% | +0.06% |
| 33_Bm25_unknown_added | 1746.875 | 1683.333 | 1695.333 | +0.71% | -2.95% |
| 33_Bm25_wide | 28630.333 | 28324.000 | 26145.167 | -7.69% | -8.68% |
| 33_Tfidf_disjoint | 947.084 | 966.916 | 909.750 | -5.91% | -3.94% |
| 33_Tfidf_limit_one | 1341.959 | 1353.000 | 1315.500 | -2.77% | -1.97% |
| 33_Tfidf_limit_zero | 2.667 | 2.666 | 2.666 | +0.00% | -0.04% |
| 33_Tfidf_multi | 3619.167 | 2579.167 | 2569.917 | -0.36% | -28.99% |
| 33_Tfidf_punctuation | 144.334 | 142.667 | 143.708 | +0.73% | -0.43% |
| 33_Tfidf_repeated | 1875.167 | 1890.625 | 1922.250 | +1.67% | +2.51% |
| 33_Tfidf_scope | 1822.541 | 1841.541 | 1782.625 | -3.20% | -2.19% |
| 33_Tfidf_single | 1836.708 | 1842.583 | 1826.250 | -0.89% | -0.57% |
| 33_Tfidf_unicode | 2028.666 | 2006.750 | 1993.166 | -0.68% | -1.75% |
| 33_Tfidf_unknown | 71.500 | 68.083 | 67.750 | -0.49% | -5.24% |
| 33_Tfidf_unknown_added | 1871.667 | 1878.416 | 1861.292 | -0.91% | -0.55% |
| 33_Tfidf_wide | 28548.916 | 28983.750 | 26717.250 | -7.82% | -6.42% |
| 4_Bm25_disjoint | 159.042 | 161.042 | 162.083 | +0.65% | +1.91% |
| 4_Bm25_limit_one | 240.583 | 238.250 | 249.541 | +4.74% | +3.72% |
| 4_Bm25_limit_zero | 2.667 | 2.667 | 2.709 | +1.57% | +1.57% |
| 4_Bm25_multi | 658.625 | 546.625 | 558.583 | +2.19% | -15.19% |
| 4_Bm25_punctuation | 146.000 | 143.458 | 146.291 | +1.97% | +0.20% |
| 4_Bm25_repeated | 435.500 | 419.834 | 428.583 | +2.08% | -1.59% |
| 4_Bm25_scope | 282.167 | 285.292 | 296.667 | +3.99% | +5.14% |
| 4_Bm25_single | 371.750 | 361.666 | 384.375 | +6.28% | +3.40% |
| 4_Bm25_unicode | 591.208 | 603.125 | 619.708 | +2.75% | +4.82% |
| 4_Bm25_unknown | 68.084 | 67.916 | 70.125 | +3.25% | +3.00% |
| 4_Bm25_unknown_added | 410.250 | 398.292 | 418.667 | +5.12% | +2.05% |
| 4_Bm25_wide | 7811.000 | 7888.167 | 7392.208 | -6.29% | -5.36% |
| 4_Tfidf_disjoint | 157.667 | 162.583 | 160.583 | -1.23% | +1.85% |
| 4_Tfidf_limit_one | 265.625 | 258.667 | 262.667 | +1.55% | -1.11% |
| 4_Tfidf_limit_zero | 2.750 | 2.666 | 2.667 | +0.04% | -3.02% |
| 4_Tfidf_multi | 683.417 | 561.625 | 567.083 | +0.97% | -17.02% |
| 4_Tfidf_punctuation | 144.125 | 143.500 | 145.917 | +1.68% | +1.24% |
| 4_Tfidf_repeated | 440.792 | 447.042 | 457.542 | +2.35% | +3.80% |
| 4_Tfidf_scope | 287.708 | 289.834 | 298.375 | +2.95% | +3.71% |
| 4_Tfidf_single | 383.500 | 387.708 | 404.917 | +4.44% | +5.58% |
| 4_Tfidf_unicode | 627.833 | 621.500 | 638.625 | +2.76% | +1.72% |
| 4_Tfidf_unknown | 67.167 | 68.917 | 68.583 | -0.48% | +2.11% |
| 4_Tfidf_unknown_added | 420.958 | 425.042 | 438.291 | +3.12% | +4.12% |
| 4_Tfidf_wide | 7934.917 | 7875.250 | 7215.709 | -8.37% | -9.06% |

Higher candidate medians versus 0.4.18: **56 of 96**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](measurements/0.4.19/small-reference-summary.json). No threshold-based rejection or removal of slow timing samples is used.

### Second preplanned small Core campaign

#### Original six warm cases

| Fixture | 0.4.17 median ns | 0.4.18 median ns | 0.4.19 median ns | vs 0.4.18 | vs 0.4.17 |
| --- | --- | --- | --- | --- | --- |
| tiny16_bm25 | 982.375 | 998.666 | 974.541 | -2.42% | -0.80% |
| tiny16_tfidf | 1038.208 | 1061.416 | 1046.375 | -1.42% | +0.79% |
| tiny4_bm25 | 338.125 | 347.333 | 341.041 | -1.81% | +0.86% |
| tiny4_tfidf | 353.125 | 373.167 | 356.792 | -4.39% | +1.04% |
| tiny_bm25 | 190.167 | 183.625 | 181.875 | -0.95% | -4.36% |
| tiny_tfidf | 186.250 | 186.042 | 185.583 | -0.25% | -0.36% |

Higher candidate medians versus 0.4.18: **0 of 6**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](measurements/0.4.19/small-confirmation-summary.json). No threshold-based rejection or removal of slow timing samples is used.

#### All 96 extended cases

| Fixture | 0.4.17 median ns | 0.4.18 median ns | 0.4.19 median ns | vs 0.4.18 | vs 0.4.17 |
| --- | --- | --- | --- | --- | --- |
| 16_Bm25_disjoint | 479.916 | 480.416 | 463.916 | -3.43% | -3.33% |
| 16_Bm25_limit_one | 623.625 | 633.417 | 618.625 | -2.34% | -0.80% |
| 16_Bm25_limit_zero | 2.708 | 2.666 | 2.667 | +0.04% | -1.51% |
| 16_Bm25_multi | 2075.208 | 1576.459 | 1568.208 | -0.52% | -24.43% |
| 16_Bm25_punctuation | 143.333 | 144.125 | 144.583 | +0.32% | +0.87% |
| 16_Bm25_repeated | 1160.000 | 1152.875 | 1165.000 | +1.05% | +0.43% |
| 16_Bm25_scope | 987.416 | 998.416 | 969.667 | -2.88% | -1.80% |
| 16_Bm25_single | 1104.042 | 1133.500 | 1111.083 | -1.98% | +0.64% |
| 16_Bm25_unicode | 1315.583 | 1317.042 | 1344.000 | +2.05% | +2.16% |
| 16_Bm25_unknown | 68.917 | 68.125 | 69.750 | +2.39% | +1.21% |
| 16_Bm25_unknown_added | 1139.875 | 1155.708 | 1143.625 | -1.05% | +0.33% |
| 16_Bm25_wide | 18982.208 | 19011.916 | 17204.791 | -9.51% | -9.36% |
| 16_Tfidf_disjoint | 474.958 | 485.083 | 454.125 | -6.38% | -4.39% |
| 16_Tfidf_limit_one | 711.750 | 715.375 | 706.292 | -1.27% | -0.77% |
| 16_Tfidf_limit_zero | 2.709 | 2.667 | 2.667 | +0.00% | -1.55% |
| 16_Tfidf_multi | 2156.916 | 1698.042 | 1674.625 | -1.38% | -22.36% |
| 16_Tfidf_punctuation | 141.250 | 143.959 | 143.166 | -0.55% | +1.36% |
| 16_Tfidf_repeated | 1228.958 | 1254.584 | 1275.666 | +1.68% | +3.80% |
| 16_Tfidf_scope | 1049.459 | 1075.125 | 1045.292 | -2.77% | -0.40% |
| 16_Tfidf_single | 1191.500 | 1201.209 | 1219.667 | +1.54% | +2.36% |
| 16_Tfidf_unicode | 1386.041 | 1422.000 | 1430.708 | +0.61% | +3.22% |
| 16_Tfidf_unknown | 67.709 | 69.042 | 68.584 | -0.66% | +1.29% |
| 16_Tfidf_unknown_added | 1217.167 | 1252.584 | 1271.250 | +1.49% | +4.44% |
| 16_Tfidf_wide | 19184.459 | 18871.000 | 17193.500 | -8.89% | -10.38% |
| 1_Bm25_disjoint | 107.000 | 114.833 | 110.958 | -3.37% | +3.70% |
| 1_Bm25_limit_one | 174.208 | 185.291 | 181.916 | -1.82% | +4.42% |
| 1_Bm25_limit_zero | 2.666 | 2.667 | 2.667 | +0.00% | +0.04% |
| 1_Bm25_multi | 282.375 | 266.750 | 261.417 | -2.00% | -7.42% |
| 1_Bm25_punctuation | 140.834 | 148.084 | 143.417 | -3.15% | +1.83% |
| 1_Bm25_repeated | 248.958 | 252.875 | 251.833 | -0.41% | +1.15% |
| 1_Bm25_scope | 181.791 | 187.792 | 188.334 | +0.29% | +3.60% |
| 1_Bm25_single | 211.750 | 221.166 | 207.459 | -6.20% | -2.03% |
| 1_Bm25_unicode | 421.000 | 429.625 | 427.208 | -0.56% | +1.47% |
| 1_Bm25_unknown | 68.166 | 69.458 | 69.375 | -0.12% | +1.77% |
| 1_Bm25_unknown_added | 218.542 | 224.500 | 223.833 | -0.30% | +2.42% |
| 1_Bm25_wide | 4326.709 | 4365.542 | 4325.083 | -0.93% | -0.04% |
| 1_Tfidf_disjoint | 109.041 | 112.208 | 112.791 | +0.52% | +3.44% |
| 1_Tfidf_limit_one | 183.959 | 189.125 | 193.459 | +2.29% | +5.16% |
| 1_Tfidf_limit_zero | 2.708 | 2.667 | 2.625 | -1.57% | -3.06% |
| 1_Tfidf_multi | 283.417 | 264.459 | 258.334 | -2.32% | -8.85% |
| 1_Tfidf_punctuation | 144.625 | 143.917 | 142.167 | -1.22% | -1.70% |
| 1_Tfidf_repeated | 240.375 | 247.625 | 255.000 | +2.98% | +6.08% |
| 1_Tfidf_scope | 190.500 | 195.875 | 198.666 | +1.42% | +4.29% |
| 1_Tfidf_single | 181.958 | 191.334 | 189.208 | -1.11% | +3.98% |
| 1_Tfidf_unicode | 428.042 | 429.958 | 430.250 | +0.07% | +0.52% |
| 1_Tfidf_unknown | 68.791 | 69.333 | 68.583 | -1.08% | -0.30% |
| 1_Tfidf_unknown_added | 219.792 | 226.375 | 224.875 | -0.66% | +2.31% |
| 1_Tfidf_wide | 4545.166 | 4518.375 | 4315.583 | -4.49% | -5.05% |
| 33_Bm25_disjoint | 967.666 | 962.625 | 937.750 | -2.58% | -3.09% |
| 33_Bm25_limit_one | 1205.833 | 1183.709 | 1200.000 | +1.38% | -0.48% |
| 33_Bm25_limit_zero | 2.666 | 2.667 | 2.667 | +0.00% | +0.04% |
| 33_Bm25_multi | 3469.625 | 2411.333 | 2432.875 | +0.89% | -29.88% |
| 33_Bm25_punctuation | 144.916 | 142.625 | 148.458 | +4.09% | +2.44% |
| 33_Bm25_repeated | 1721.375 | 1693.417 | 1786.334 | +5.49% | +3.77% |
| 33_Bm25_scope | 1733.750 | 1725.458 | 1706.250 | -1.11% | -1.59% |
| 33_Bm25_single | 1743.875 | 1644.167 | 1746.042 | +6.20% | +0.12% |
| 33_Bm25_unicode | 1844.167 | 1822.166 | 1892.292 | +3.85% | +2.61% |
| 33_Bm25_unknown | 67.833 | 69.834 | 69.208 | -0.90% | +2.03% |
| 33_Bm25_unknown_added | 1724.084 | 1680.250 | 1761.458 | +4.83% | +2.17% |
| 33_Bm25_wide | 28778.542 | 28682.542 | 26395.750 | -7.97% | -8.28% |
| 33_Tfidf_disjoint | 954.250 | 939.083 | 914.792 | -2.59% | -4.13% |
| 33_Tfidf_limit_one | 1374.458 | 1295.208 | 1336.666 | +3.20% | -2.75% |
| 33_Tfidf_limit_zero | 2.667 | 2.708 | 2.667 | -1.51% | +0.00% |
| 33_Tfidf_multi | 3647.500 | 2555.542 | 2543.958 | -0.45% | -30.25% |
| 33_Tfidf_punctuation | 144.250 | 146.250 | 141.875 | -2.99% | -1.65% |
| 33_Tfidf_repeated | 1903.459 | 1888.375 | 1865.750 | -1.20% | -1.98% |
| 33_Tfidf_scope | 1864.750 | 1818.000 | 1779.458 | -2.12% | -4.57% |
| 33_Tfidf_single | 1830.041 | 1836.750 | 1824.625 | -0.66% | -0.30% |
| 33_Tfidf_unicode | 1982.917 | 1988.750 | 1970.542 | -0.92% | -0.62% |
| 33_Tfidf_unknown | 69.166 | 68.875 | 67.792 | -1.57% | -1.99% |
| 33_Tfidf_unknown_added | 1910.334 | 1876.084 | 1856.875 | -1.02% | -2.80% |
| 33_Tfidf_wide | 29134.708 | 28470.708 | 26651.333 | -6.39% | -8.52% |
| 4_Bm25_disjoint | 164.500 | 163.792 | 154.500 | -5.67% | -6.08% |
| 4_Bm25_limit_one | 245.542 | 240.917 | 240.958 | +0.02% | -1.87% |
| 4_Bm25_limit_zero | 3.000 | 2.667 | 2.666 | -0.04% | -11.13% |
| 4_Bm25_multi | 670.250 | 547.125 | 533.334 | -2.52% | -20.43% |
| 4_Bm25_punctuation | 143.875 | 144.750 | 143.709 | -0.72% | -0.12% |
| 4_Bm25_repeated | 435.834 | 432.500 | 425.333 | -1.66% | -2.41% |
| 4_Bm25_scope | 284.750 | 289.584 | 287.792 | -0.62% | +1.07% |
| 4_Bm25_single | 378.625 | 376.000 | 366.292 | -2.58% | -3.26% |
| 4_Bm25_unicode | 599.709 | 597.667 | 605.000 | +1.23% | +0.88% |
| 4_Bm25_unknown | 69.083 | 68.291 | 68.958 | +0.98% | -0.18% |
| 4_Bm25_unknown_added | 412.875 | 409.250 | 406.042 | -0.78% | -1.65% |
| 4_Bm25_wide | 8042.292 | 8173.250 | 7177.167 | -12.19% | -10.76% |
| 4_Tfidf_disjoint | 161.792 | 163.375 | 156.458 | -4.23% | -3.30% |
| 4_Tfidf_limit_one | 260.458 | 264.000 | 266.500 | +0.95% | +2.32% |
| 4_Tfidf_limit_zero | 2.625 | 2.625 | 2.667 | +1.60% | +1.60% |
| 4_Tfidf_multi | 694.166 | 581.750 | 571.250 | -1.80% | -17.71% |
| 4_Tfidf_punctuation | 145.459 | 145.041 | 142.750 | -1.58% | -1.86% |
| 4_Tfidf_repeated | 458.792 | 449.375 | 447.958 | -0.32% | -2.36% |
| 4_Tfidf_scope | 293.083 | 294.541 | 298.709 | +1.42% | +1.92% |
| 4_Tfidf_single | 389.375 | 386.958 | 394.042 | +1.83% | +1.20% |
| 4_Tfidf_unicode | 635.541 | 646.083 | 631.958 | -2.19% | -0.56% |
| 4_Tfidf_unknown | 68.208 | 70.417 | 66.833 | -5.09% | -2.02% |
| 4_Tfidf_unknown_added | 431.958 | 442.583 | 430.125 | -2.81% | -0.42% |
| 4_Tfidf_wide | 8056.584 | 7976.958 | 7229.875 | -9.37% | -10.26% |

Higher candidate medians versus 0.4.18: **30 of 96**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](measurements/0.4.19/small-confirmation-summary.json). No threshold-based rejection or removal of slow timing samples is used.

## Direct canonical search with explicit control modes

Fourteen processes (seven alternating pairs) retain one warmed Engine each. Ten rounds rotate plain, deadline and progress within each query/ranker pair: **8,400 timed Retriever calls**, excluding loading/readiness, stdio and profiling. Each progress request includes an observer and the same deadline/cancellation setup. Every signature agrees; **140 complete original response JSON pairs match**, and control modes also match plain output. Full readiness source text stays private. These are direct retrieval boundaries, not MCP round trips or capacity evidence.

| Control | Ranker | 0.4.18 median / p95 ms | 0.4.19 median / p95 ms | Median change |
| --- | --- | --- | --- | --- |
| plain | Bm25 | 1.755 / 3.576 | 1.798 / 3.689 | +2.48% |
| plain | Tfidf | 1.932 / 3.818 | 1.957 / 3.854 | +1.30% |
| deadline | Bm25 | 2.014 / 3.978 | 2.044 / 4.274 | +1.50% |
| deadline | Tfidf | 2.241 / 4.415 | 2.295 / 4.538 | +2.42% |
| progress | Bm25 | 2.007 / 3.969 | 2.050 / 4.202 | +2.14% |
| progress | Tfidf | 2.234 / 4.432 | 2.278 / 4.428 | +1.96% |

| Query | Candidates | Progress BM25 ms: old → new | Progress TF-IDF ms: old → new |
| --- | --- | --- | --- |
| 0 | 12365 | 1.861 → 1.884 | 2.088 → 2.133 |
| 1 | 13864 | 2.027 → 2.074 | 2.327 → 2.368 |
| 2 | 24299 | 3.434 → 3.477 | 3.887 → 3.932 |
| 3 | 596 | 0.292 → 0.299 | 0.286 → 0.293 |
| 4 | 15403 | 2.157 → 2.203 | 2.413 → 2.456 |
| 5 | 24140 | 3.397 → 3.450 | 3.849 → 3.868 |
| 6 | 3358 | 0.648 → 0.654 | 0.702 → 0.711 |
| 7 | 29600 | 3.754 → 3.867 | 4.375 → 4.382 |
| 8 | 11504 | 1.702 → 1.760 | 1.892 → 1.890 |
| 9 | 0 | 0.002 → 0.002 | 0.001 → 0.001 |

## Separate canonical three-version reference

Five rotating triples compare fresh exact 0.4.17/0.4.18/candidate servers on canonical knowledge with requested progress. Fifteen servers validate **1,020 actual schema replies**; **680 complete structured comparisons** against the candidate match by RPC ID except checked status version and version-qualified schema URNs. Schemas, source ranges and inputs agree. This control is separate from the two ordinary campaigns. It checks whether the previous large-query benefit remains visible against the older release; it does not establish the cause of any historical regression.

| Version | Ranker | Initial median / p95 ms | Repeated median / p95 ms | Peak child RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.17 | bm25 | 3.256 / 5.663 | 2.876 / 4.830 | 1724.234 |
| 0.4.17 | tfidf | 17.476 / 31.360 | 3.062 / 5.422 | 1724.234 |
| 0.4.18 | bm25 | 3.061 / 5.183 | 2.732 / 4.914 | 1724.219 |
| 0.4.18 | tfidf | 18.076 / 31.470 | 2.915 / 5.439 | 1724.219 |
| 0.4.19 | bm25 | 2.998 / 5.005 | 2.619 / 4.761 | 1724.234 |
| 0.4.19 | tfidf | 17.972 / 31.939 | 2.887 / 5.231 | 1724.234 |

| Query | 0.4.17 repeated TF-IDF ms | 0.4.18 repeated TF-IDF ms | 0.4.19 repeated TF-IDF ms |
| --- | --- | --- | --- |
| 0 | 2.807 | 2.622 | 2.636 |
| 1 | 3.161 | 3.064 | 2.966 |
| 2 | 4.939 | 4.670 | 4.568 |
| 3 | 0.660 | 0.657 | 0.664 |
| 4 | 3.079 | 2.986 | 3.030 |
| 5 | 4.885 | 4.531 | 4.674 |
| 6 | 1.191 | 1.194 | 1.130 |
| 7 | 5.271 | 5.140 | 5.124 |
| 8 | 3.112 | 3.310 | 3.158 |
| 9 | 0.109 | 0.113 | 0.112 |

## Canonical first-pass heap traffic

Two instrumented processes index all 167,942 records before profiling the same ten initial TF-IDF queries, then drop results before net deltas. The existing direct candidate collection remains; the previous large-query posting cursors and norm memo/cache still exist. Heap traffic, net persistent hints, stack state and whole-process RSS are different costs. Instrumented durations are excluded from latency comparisons.

| Metric | 0.4.18 | 0.4.19 |
| --- | --- | --- |
| Retained hint bytes | 2228312 | 2228312 |
| Retained hint blocks | 2 | 2 |
| Total query allocated bytes | 7213987 | 7213987 |
| Total query allocation blocks | 415 | 415 |

## Development pilot and host-load limits

The [development pilot](measurements/0.4.19/development-pilot-summary.json) remains separate from all final samples. Its separate small/wide/canonical/tiny rotating triples supported the candidate choice. The initial stopped out-of-bound pilot is retained as well. Pilot samples do not enter final distributions. All source/lock/binary identities and raw observations remain locally retained.

Two-second monitoring retains **347 snapshots**, including **34 with other rustc processes**, maximum 3. All managed compilers finish before timing. Other workloads are not stopped, and snapshots do not prove continuous idleness or identify a timing cause. CPU counts include the active benchmark workers.

| Stage start | Other rustc count | Other rustc CPU total | Processes over 50% CPU |
| --- | --- | --- | --- |
| wide-standard | 0 | 0.000 | 1 |
| small-reference | 0 | 0.000 | 1 |
| retrieval | 0 | 0.000 | 1 |
| query | 0 | 0.000 | 1 |
| build | 0 | 0.000 | 1 |
| controlled | 0 | 0.000 | 1 |
| mcp | 0 | 0.000 | 1 |
| corpus | 0 | 0.000 | 1 |
| mcp-confirmation | 0 | 0.000 | 1 |
| reference | 0 | 0.000 | 1 |
| small-confirmation | 0 | 0.000 | 1 |
| wide-confirmation | 0 | 0.000 | 1 |
| wide-extended | 0 | 0.000 | 1 |
| limits | 0 | 0.000 | 1 |

All observed slower cases remain in their own distributions; no sample is replaced because of load. One machine, randomized hashing, exposed questions, tuned fixtures and finite trials limit generalization.

## Retained Core controls

These controls retain positive and negative timing changes. Single known-term requests retain the direct candidate path and exactly two retain ordered merging. Wider requests keep the heap/set strategies; large frequency cursors remain available afterward. Explanation dictionaries now share the checked exact intersection. Read every row independently, including slower cases. Seven alternating pairs give fourteen plain processes per family. Norms are initialized for warm timing; two separate DHAT processes measure allocations. Instrumented durations are excluded from latency comparisons. Every result signature agrees and temporary-call retained deltas are zero.

### Small warm Core: 84,000 calls on 1/4/16 entries

| Fixture | Median change | Blocks/call: old → new | Allocated bytes/call: old → new |
| --- | --- | --- | --- |
| tiny16_bm25 | -0.61% | 29.0 → 29.0 | 2536.0 → 2536.0 |
| tiny16_tfidf | -0.52% | 29.0 → 29.0 | 2536.0 → 2536.0 |
| tiny4_bm25 | +0.50% | 15.0 → 15.0 | 1204.0 → 1204.0 |
| tiny4_tfidf | +0.71% | 15.0 → 15.0 | 1204.0 → 1204.0 |
| tiny_bm25 | -7.13% | 9.0 → 9.0 | 778.0 → 778.0 |
| tiny_tfidf | +0.58% | 9.0 → 9.0 | 778.0 → 778.0 |

### All 25 warm controls: 282,800 calls

| Fixture | Median change | Blocks/call: old → new | Allocated bytes/call: old → new |
| --- | --- | --- | --- |
| coverage_repeated | -0.81% | 1047.0 → 1047.0 | 92942.0 → 92942.0 |
| coverage_small | -5.01% | 784.0 → 784.0 | 78046.0 → 78046.0 |
| excerpt_ascii | -8.94% | 762.0 → 762.0 | 252438.0 → 252438.0 |
| excerpt_mixed | -7.90% | 569.0 → 569.0 | 417113.0 → 417113.0 |
| excerpt_unicode | -5.34% | 197.0 → 197.0 | 72254.0 → 72254.0 |
| long_repeated | -14.36% | 4.0 → 4.0 | 18798.0 → 18798.0 |
| mixed_unicode | -4.34% | 7.0 → 7.0 | 3155.0 → 3155.0 |
| punctuation | -2.77% | 1.0 → 1.0 | 768.0 → 768.0 |
| relevant_retrieval | -9.13% | 812.0 → 812.0 | 93562.0 → 93562.0 |
| repeated_search | -6.23% | 35.0 → 35.0 | 19432.0 → 19432.0 |
| repeated_short | -17.38% | 4.0 → 4.0 | 1656.0 → 1656.0 |
| sparse_bm25_128 | -18.72% | 359.0 → 359.0 | 29074.0 → 29074.0 |
| sparse_bm25_33 | -12.44% | 241.0 → 241.0 | 16309.0 → 16309.0 |
| sparse_bm25_512 | -25.29% | 1337.0 → 1337.0 | 119124.0 → 119124.0 |
| sparse_tfidf_128 | -17.88% | 474.0 → 474.0 | 35140.0 → 35140.0 |
| sparse_tfidf_33 | -9.20% | 241.0 → 241.0 | 16309.0 → 16309.0 |
| sparse_tfidf_512 | -16.53% | 1337.0 → 1337.0 | 119124.0 → 119124.0 |
| tfidf_union_32 | -9.50% | 244.0 → 244.0 | 19834.0 → 19834.0 |
| tfidf_union_33 | -5.80% | 296.0 → 296.0 | 27854.0 → 27854.0 |
| tfidf_warm | -8.83% | 71.0 → 71.0 | 19335.0 → 19335.0 |
| union_32 | -11.01% | 244.0 → 244.0 | 19834.0 → 19834.0 |
| union_33 | -11.44% | 296.0 → 296.0 | 27854.0 → 27854.0 |
| unique_long | -5.25% | 299.0 → 299.0 | 32672.0 → 32672.0 |
| unique_search | -3.93% | 72.0 → 72.0 | 19350.0 → 19350.0 |
| unique_short | +1.99% | 296.0 → 296.0 | 18548.0 → 18548.0 |

### Cold norm and heap controls

Six fixtures make twenty fresh-index-clone calls each across fourteen processes: 1,680 calls. Clone/setup is outside timing. Process/instruction/filesystem caches remain; cold means norm hints, not disks. Separate heap runs drop results before net retained measurements. The existing inline/optional heap word memo and persistent norm storage remain. Large small-term requests may add up to four temporary posting cursors; canonical allocation traffic below records that cost. Heap profiles exclude stack state/control fields and temporary frames.

| Fixture | Old median / p95 ns | New median / p95 ns |
| --- | --- | --- |
| index | 110125.000 / 122758.350 | 107875.500 / 122125.300 |
| wide | 31750.000 / 41425.350 | 31625.000 / 40713.100 |
| sparse | 481687.000 / 546264.600 | 427854.000 / 490645.500 |
| tiny | 542.000 / 1752.050 | 625.000 / 1460.100 |
| tiny4 | 833.000 / 1516.650 | 834.000 / 1375.000 |
| tiny16 | 2416.500 / 4387.500 | 2312.500 / 3291.050 |

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
| new_group_query_ns | 112291.500 / 144552.100 | 140291.000 / 179387.500 |
| post_readers_first_ns | 149833.000 / 184758.600 | 146542.000 / 174999.600 |
| post_readers_second_ns | 80916.000 / 81599.600 | 83250.000 / 85900.200 |

The private document corpus retains 72 Markdown files, 2,755,468 bytes and 1,910 chunks. All 342 authored questions (276 positive, 66 unsupported) are exposed regressions, not fresh holdouts or logged production queries. Literal BM25, prepared BM25 and coverage retain six passages, 100 candidates, 2,000 bytes/pass and 12,000 total bytes. All **1,026 complete original response JSON strings match byte-for-byte**, including scores, ordering, evidence, source text and positions. One warm-up and five repeats/question/configuration/version give **10,260 timed round trips**. These three configurations are BM25 controls; independent score oracles and actual protocol pairs cover TF-IDF. No new relevance judgment or model reply is produced.

All document timing aggregates and per-question results remain in the records. Construction uses forty plain processes on canonical data and three synthetic distributions, plus six separate profiles. Parsing/generation is outside construction timing; stats/signatures/live derived heap agree. The execution/shared-index worker checks all 167,942 entries through 128 changes/restoration, rebuilds, JSON round trips, retained clones, parallel readers and cancellation/deadline rejection. Portable exports and source hashes agree.

### Document round trips

| Case | Old median / p95 ms | New median / p95 ms |
| --- | --- | --- |
| baseline | 0.784 / 1.196 | 0.788 / 1.177 |
| prepared | 0.838 / 1.311 | 0.843 / 1.281 |
| evidence | 0.828 / 1.293 | 0.828 / 1.306 |

### Construction only

| Case | Old median / p95 ms | New median / p95 ms |
| --- | --- | --- |
| canonical | 4855.553 / 4868.551 | 4876.821 / 4959.901 |
| ascii | 14.555 / 14.711 | 14.379 / 14.453 |
| mixed | 33.597 / 34.165 | 34.084 / 34.330 |
| long_tokens | 1.624 / 1.651 | 1.519 / 1.573 |

## Limits and preservation

Measurements use one AC-powered Apple M5 Max with 64 GB, Rust 1.98.0 and Python 3.15.2. Stage-start CPU/compiler snapshots remain. Own compilers finish before timing; this does not establish continuously idle hardware. Randomized hashing, scheduling, one machine/corpus, exposed questions and finite process counts limit generalization. The separate development pilot remains; assess final per-case medians and tails independently. Host-load sampling records observations rather than continuous idleness. No universal latency, total-memory, semantic-quality, token-saving or security-audit claim is made.

Canonical knowledge remains 167,942 entries from 35 PDFs and 49,086 physical pages, SHA-256 `13168d87872a137355fd8499c90a6084f52243a2ec7456f29b5fcb2a1c7423c3`. The PDF parser is unchanged; this is not a fresh large-PDF extraction campaign. Lexical overlap and exact quotes do not prove factual support or entailment. No model, daemon or DenkMaps integration is added.

Preserve PDFs, knowledge, snapshots, all pilot/final measurements, slower results, raw transcripts, profiles, exact small executables and locks. After verified publication archive package/proof, then delete only the named temporary Cargo/Miri targets. Default features remain empty.
