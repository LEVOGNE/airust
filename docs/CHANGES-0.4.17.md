# AIRust 0.4.17: less overhead for small Core searches

When a query has exactly one known indexed term, AIRust now reads that term's already sorted, unique posting IDs directly. It does not build a heap to merge one list. The borrowed candidate Vec starts with space for at most 32 IDs, bounded by that posting count, avoiding several small growth steps. Larger Vec growth remains proportional to the candidates. This applies to both rankers and to queries with repeated or unknown words that still have only one known term. It introduces no index, model, dependency, configuration or public field.

The 0.4.16 large-query frequency cursors and 0.4.15 MCP timer behavior remain. Query analysis, candidate order, filters, score arithmetic, norm reuse, explanations, final ties and source positions retain their contracts. Every posting still invokes the original checkpoint and progress accounting; cancellation returns an error before partial results can escape. Queries with multiple known terms retain the existing merge/set strategies. Calling an operation small does not make its resource usage bounded independently of its inputs.

Across two preplanned ordinary MCP campaigns, canonical repeated TF-IDF median changes are **-1.73% and +0.99%** against fresh exact registry 0.4.16. One-entry repeated MCP medians change by **-5.27% and +7.37%**. Negative means shorter time. The separate small-Core three-version tables below are the primary evidence for this release's target; transport timing is a different boundary. All slower medians, tails, memory costs and host-load samples remain in [sanitized records](measurements/0.4.17/README.md). No universal speedup, statistical significance or new semantic-quality claim is made.

The six target warm Core cases improve in both fresh three-version campaigns: **-14.99% to -3.67%** and **-19.06% to -5.53%** versus 0.4.16. The 96-case extensions retain **17** and **24** higher medians respectively. The largest ordinary repeated MCP median rise is **+7.37%** (one-entry TF-IDF in the second campaign, 0.119 → 0.128 ms). A separate canonical reference has **+9.50%** repeated TF-IDF median versus 0.4.16 (3.420 → 3.745 ms), while remaining **-43.10%** versus 0.4.15. These are different distributions, not samples to merge. This release establishes improvement in the targeted small Core cases; it does not establish that every previous 0.4.16 latency is preserved.

## Executed validation and public API

The full matrix passes **238 default tests, 888 all-feature checks including 39 documentation examples, 39 isolated feature entries and five Python tests**. Strict Clippy/docs, Rust 1.89 provider-neutral and Rust 1.92 all-feature checks pass. API comparison with tagged 0.4.16 reports **229 pass, 31 skip**. An independent Rust 1.89 consumer covers retrieval, coverage, quotes, document positions and MCP/schema construction. The unchanged PDF backend passes 14 regressions and its 381-source-file provenance check. The refreshed dependency audit reports zero known vulnerabilities in its checked database; maintenance notices remain in the resource record.

Two new regressions check checkpoint failure during direct candidate collection and exact hand-calculated small-index scores/ties/limits/scopes with repeated Unicode and unknown terms. The existing independent full-scan oracle covers 1,025 native records, single/multiple/wide queries, both rankers, filters and cold/warm states. Mutation, retained clones, reused IDs, metadata, sources/spans, hostile MCP input, cancellation, absolute deadlines, stalled readers, admission, duplicate tokens, disconnect and worker drain remain tested.

Twenty finite Tree Borrows Miri cases include both new regressions with scaled inputs. Only extra floating-point rounding injection is disabled; memory/alias checks remain. Tokio/PDF/FFI/ONNX execution is outside this finite interpreter scope. These checks are not an independent security or soundness audit. Extracted-package tests, exact-commit CI and a fresh registry consumer gate publication; package checksum, VCS, tag and release asset must agree.

Existing Engine, SearchOptions, Retriever, WorkControl, McpServer and McpLimits calls remain. Default features stay empty. There is no new switch or data migration. Use WorkControl or MCP timeout_ms for cooperative bounds as before. MCP admission, frame/result/rate budgets, progress delivery deadlines and per-connection cancellation/drain remain. Startup indexing is outside tool deadlines. Read [Core performance](CORE-PERFORMANCE.md), [API](API.md), [cookbook](API-COOKBOOK.md), [MCP](MCP.md) and [security scope](MCP-SECURITY.md).

## Frozen method and development record

Fresh exact registry 0.4.16 (VCS 869a814934737ded051e9e2ec64a73395e3285a5) is the baseline. Separate three-version controls also build exact registry 0.4.15 (VCS 6ac8f081b46bc63cbc9c999e0b88d944524fc62e). Candidate 0.4.17 uses the same programs and inputs. All programs, actual source/manifest copies, locks, binaries and input hashes are preserved. Read the [reproduction method](validation-0.4.17/README.md).

The development pilot ran seven rotating fresh 0.4.15/0.4.16/0.4.17 triples on the six previously slower warm Core cases. Every signature matched. All six candidate medians were lower than both references in that pilot. It is retained separately, with complete source hashes and raw samples; it is neither a final sample nor proof of the original regression's cause. The implementation removes a concrete unnecessary heap operation rather than attributing historical timing changes solely to compiler layout or noise.

Final timing begins after all managed compiler work. Two separate small Core campaigns each retain seven rotating triples, the original six cases and a fixed 96-case extension on 1/4/16/33 entries. The extension covers single/multiple/repeated/unknown/Unicode terms, empty matches, scope/disjoint filters, top-one/zero limits and wide queries. Each complete warm-up response is compared as original JSON outside timing. The same exposed authored document questions, canonical knowledge and revision-bound NumPy snapshot remain; this is not a fresh relevance holdout, model-answer evaluation or large-PDF extraction campaign. Host-load observations are retained without stopping other workloads or replacing slow samples.

## Actual MCP: two complete ordinary campaigns

Each ordinary campaign uses nine cases and five alternating old/new pairs: **90 servers per campaign, 180 total**. Canonical knowledge, NumPy and broad synthetic input request progress; 1/4/16-entry inputs run with and without progress. Each server makes ten queries with both rankers, two repeated passes, four concurrent searches, matching/relations, rejected arguments and health. Snapshot inputs also verify exact UTF-8 source readbacks.

Together **13,880 actual result/error replies validate against advertised schemas**, and **6,940 complete old/new structured pairs match by RPC ID**, excluding only checked status version and version-qualified schema URNs. Counts: 3,600 initial searches, 7,200 repeats, 720 concurrent searches, 1,640 readbacks, 360 matching/relations, 180 rejected arguments and 180 health calls. Discovery, progress/no-progress, bounds, ordering, unchanged inputs and EOF shutdown pass.

Initial/first/repeated timings and startup are separate. Search latency includes serialization, stdio, Python parsing and requested notifications; source follow-ups are outside it. Child peak RSS includes runtime, buffers and source/index memory, unlike allocation profiles. All unfavorable medians and tails are retained in their own rows.

### First complete campaign

| Case | Ranker | Initial median / p95 ms: old → new | Repeated median / p95 ms: old → new | Repeated median change |
| --- | --- | --- | --- | --- |
| canonical | bm25 | 3.415 / 5.621 → 3.419 / 5.397 | 2.952 / 5.374 → 2.997 / 5.053 | +1.52% |
| canonical | tfidf | 18.726 / 33.227 → 18.340 / 33.463 | 3.416 / 6.120 → 3.356 / 5.593 | -1.73% |
| numpy | bm25 | 0.985 / 1.317 → 0.958 / 1.262 | 0.828 / 1.144 → 0.814 / 1.153 | -1.67% |
| numpy | tfidf | 1.096 / 2.903 → 1.115 / 2.852 | 0.781 / 1.191 → 0.745 / 1.224 | -4.66% |
| wide | bm25 | 0.565 / 0.754 → 0.610 / 0.743 | 0.496 / 0.832 → 0.500 / 0.904 | +0.83% |
| wide | tfidf | 0.524 / 0.969 → 0.555 / 1.019 | 0.493 / 0.978 → 0.486 / 0.966 | -1.29% |
| tiny | bm25 | 0.167 / 0.284 → 0.169 / 0.299 | 0.139 / 0.238 → 0.137 / 0.243 | -1.51% |
| tiny | tfidf | 0.159 / 0.233 → 0.162 / 0.243 | 0.142 / 0.226 → 0.135 / 0.221 | -5.27% |
| tiny4 | bm25 | 0.267 / 0.448 → 0.259 / 0.455 | 0.195 / 0.282 → 0.194 / 0.325 | -0.43% |
| tiny4 | tfidf | 0.237 / 0.353 → 0.218 / 0.330 | 0.190 / 0.304 → 0.187 / 0.324 | -1.44% |
| tiny16 | bm25 | 0.251 / 0.420 → 0.257 / 0.416 | 0.216 / 0.324 → 0.230 / 0.363 | +6.46% |
| tiny16 | tfidf | 0.263 / 0.394 → 0.267 / 0.401 | 0.215 / 0.330 → 0.228 / 0.337 | +5.78% |
| tiny-no-progress | bm25 | 0.139 / 2.541 → 0.124 / 0.256 | 0.112 / 0.399 → 0.102 / 0.245 | -9.09% |
| tiny-no-progress | tfidf | 0.119 / 0.319 → 0.111 / 0.190 | 0.110 / 2.540 → 0.103 / 0.213 | -6.83% |
| tiny4-no-progress | bm25 | 0.248 / 1.805 → 0.174 / 0.362 | 0.150 / 3.306 → 0.143 / 0.225 | -4.99% |
| tiny4-no-progress | tfidf | 0.185 / 1.574 → 0.156 / 0.266 | 0.159 / 2.708 → 0.146 / 0.249 | -8.38% |
| tiny16-no-progress | bm25 | 0.714 / 6.579 → 0.363 / 2.931 | 0.242 / 3.937 → 0.245 / 3.621 | +1.23% |
| tiny16-no-progress | tfidf | 0.577 / 11.273 → 0.353 / 4.088 | 0.244 / 2.663 → 0.255 / 3.375 | +4.58% |

| Case | Startup median ms: old → new | First TF-IDF median ms: old → new | Peak RSS median MiB: old → new |
| --- | --- | --- | --- |
| canonical | 4927.422 → 4939.242 | 22.044 → 22.467 | 1724.250 → 1724.297 |
| numpy | 140.519 → 140.010 | 1.645 → 1.616 | 57.156 → 57.156 |
| wide | 4.551 → 4.337 | 0.555 → 0.550 | 7.734 → 7.781 |
| tiny | 3.939 → 4.128 | 0.209 → 0.194 | 5.766 → 5.734 |
| tiny4 | 4.240 → 4.327 | 0.297 → 0.319 | 5.906 → 5.922 |
| tiny16 | 4.041 → 3.898 | 0.313 → 0.331 | 6.031 → 6.000 |
| tiny-no-progress | 7.050 → 5.154 | 0.199 → 0.165 | 5.797 → 5.750 |
| tiny4-no-progress | 4.608 → 4.030 | 0.251 → 0.239 | 5.891 → 5.875 |
| tiny16-no-progress | 7.125 → 8.178 | 0.406 → 0.388 | 6.031 → 6.031 |

### Second preplanned complete campaign

| Case | Ranker | Initial median / p95 ms: old → new | Repeated median / p95 ms: old → new | Repeated median change |
| --- | --- | --- | --- | --- |
| canonical | bm25 | 3.743 / 8.783 → 3.838 / 10.324 | 3.490 / 8.720 → 3.634 / 8.638 | +4.14% |
| canonical | tfidf | 22.629 / 48.021 → 23.171 / 44.772 | 4.419 / 10.957 → 4.462 / 11.712 | +0.99% |
| numpy | bm25 | 1.000 / 1.377 → 0.958 / 1.324 | 0.845 / 1.280 → 0.825 / 1.164 | -2.45% |
| numpy | tfidf | 1.077 / 2.877 → 1.119 / 2.850 | 0.795 / 1.434 → 0.768 / 1.339 | -3.29% |
| wide | bm25 | 0.533 / 0.656 → 0.543 / 0.664 | 0.430 / 0.687 → 0.437 / 0.784 | +1.70% |
| wide | tfidf | 0.473 / 0.857 → 0.472 / 0.906 | 0.424 / 0.821 → 0.434 / 0.840 | +2.29% |
| tiny | bm25 | 0.144 / 0.236 → 0.148 / 0.248 | 0.121 / 0.154 → 0.127 / 0.275 | +4.92% |
| tiny | tfidf | 0.138 / 0.184 → 0.142 / 0.227 | 0.119 / 0.149 → 0.128 / 0.227 | +7.37% |
| tiny4 | bm25 | 0.197 / 0.339 → 0.201 / 0.334 | 0.174 / 0.204 → 0.176 / 0.227 | +1.20% |
| tiny4 | tfidf | 0.191 / 0.262 → 0.190 / 0.287 | 0.173 / 0.210 → 0.177 / 0.243 | +2.23% |
| tiny16 | bm25 | 0.209 / 0.338 → 0.218 / 0.355 | 0.199 / 0.236 → 0.196 / 0.236 | -1.05% |
| tiny16 | tfidf | 0.216 / 0.346 → 0.221 / 0.344 | 0.199 / 0.234 → 0.197 / 0.240 | -0.91% |
| tiny-no-progress | bm25 | 0.092 / 0.180 → 0.076 / 0.182 | 0.070 / 0.122 → 0.067 / 0.114 | -4.30% |
| tiny-no-progress | tfidf | 0.087 / 0.146 → 0.078 / 0.119 | 0.068 / 0.108 → 0.069 / 0.093 | +1.31% |
| tiny4-no-progress | bm25 | 0.138 / 0.264 → 0.133 / 0.266 | 0.123 / 0.148 → 0.122 / 0.141 | -0.74% |
| tiny4-no-progress | tfidf | 0.135 / 0.192 → 0.138 / 0.198 | 0.121 / 0.174 → 0.121 / 0.165 | +0.29% |
| tiny16-no-progress | bm25 | 0.173 / 0.280 → 0.169 / 0.285 | 0.150 / 0.203 → 0.149 / 0.174 | -0.51% |
| tiny16-no-progress | tfidf | 0.169 / 0.286 → 0.170 / 0.275 | 0.152 / 0.201 → 0.152 / 0.198 | +0.21% |

| Case | Startup median ms: old → new | First TF-IDF median ms: old → new | Peak RSS median MiB: old → new |
| --- | --- | --- | --- |
| canonical | 5630.086 → 5299.610 | 26.227 → 25.844 | 1724.766 → 1724.500 |
| numpy | 138.906 → 138.643 | 1.667 → 1.578 | 57.125 → 57.203 |
| wide | 4.378 → 3.991 | 0.511 → 0.500 | 7.750 → 7.922 |
| tiny | 3.661 → 3.801 | 0.178 → 0.182 | 5.766 → 5.766 |
| tiny4 | 3.508 → 3.640 | 0.264 → 0.260 | 5.859 → 5.859 |
| tiny16 | 3.555 → 3.722 | 0.289 → 0.294 | 6.047 → 6.062 |
| tiny-no-progress | 3.959 → 3.553 | 0.137 → 0.117 | 5.781 → 5.734 |
| tiny4-no-progress | 3.590 → 3.535 | 0.190 → 0.200 | 5.891 → 5.891 |
| tiny16-no-progress | 3.803 → 3.433 | 0.239 → 0.243 | 6.031 → 6.031 |

## Primary target: fresh three-version small Core controls

These separate campaigns compare freshly built exact registry 0.4.15 and 0.4.16 with the candidate, using the same programs. Seven rotating triples per campaign run both the original six cases and a 96-case extension. Warm-up, full original-JSON comparison and index setup occur before timers. Each campaign has **42 processes and 2,142,000 timed Core calls**. Both together have 84 processes, 4,284,000 calls and **2,688 complete original matrix-response comparisons**, with identical results. Signature checks also cover every original-six and timed matrix sample. This is repeated warm Core time, without MCP transport or DHAT instrumentation.

Keep the two campaigns separate. Original six-case calls build SearchOptions inside the measured loop; the extension reuses fixed options. Do not compare their absolute times or merge distributions. The 96 cases combine four entry counts, both rankers and twelve fixed query/filter/limit cases. Wide query preparation, scope rejection, unknown-only queries and zero limits are retained even though they are outside the optimized single-list work. Historical absolute values with different programs/loads are not a reference substitute.

### First small Core campaign

#### Original six warm cases

| Fixture | 0.4.15 median ns | 0.4.16 median ns | 0.4.17 median ns | vs 0.4.16 | vs 0.4.15 |
| --- | --- | --- | --- | --- | --- |
| tiny16_bm25 | 1126.250 | 1168.458 | 993.333 | -14.99% | -11.80% |
| tiny16_tfidf | 1206.375 | 1224.208 | 1046.625 | -14.51% | -13.24% |
| tiny4_bm25 | 369.000 | 376.541 | 334.875 | -11.07% | -9.25% |
| tiny4_tfidf | 389.667 | 394.667 | 357.959 | -9.30% | -8.14% |
| tiny_bm25 | 207.667 | 217.708 | 189.292 | -13.05% | -8.85% |
| tiny_tfidf | 197.292 | 195.083 | 187.917 | -3.67% | -4.75% |

Higher candidate medians versus 0.4.16: **0 of 6**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](measurements/0.4.17/small-reference-summary.json). No threshold-based rejection or removal of slow timing samples is used.

#### All 96 extended cases

| Fixture | 0.4.15 median ns | 0.4.16 median ns | 0.4.17 median ns | vs 0.4.16 | vs 0.4.15 |
| --- | --- | --- | --- | --- | --- |
| 16_Bm25_disjoint | 653.917 | 625.208 | 481.375 | -23.01% | -26.39% |
| 16_Bm25_limit_one | 832.209 | 803.208 | 623.625 | -22.36% | -25.06% |
| 16_Bm25_limit_zero | 2.708 | 2.750 | 2.708 | -1.53% | +0.00% |
| 16_Bm25_multi | 2071.000 | 2030.291 | 2064.458 | +1.68% | -0.32% |
| 16_Bm25_punctuation | 145.167 | 143.875 | 143.708 | -0.12% | -1.01% |
| 16_Bm25_repeated | 1365.708 | 1338.333 | 1135.542 | -15.15% | -16.85% |
| 16_Bm25_scope | 1166.542 | 1142.292 | 980.208 | -14.19% | -15.97% |
| 16_Bm25_single | 1313.083 | 1267.625 | 1098.542 | -13.34% | -16.34% |
| 16_Bm25_unicode | 1505.834 | 1491.625 | 1291.375 | -13.42% | -14.24% |
| 16_Bm25_unknown | 68.959 | 67.209 | 66.333 | -1.30% | -3.81% |
| 16_Bm25_unknown_added | 1345.292 | 1317.084 | 1116.125 | -15.26% | -17.03% |
| 16_Bm25_wide | 18629.166 | 18785.375 | 18594.750 | -1.01% | -0.18% |
| 16_Tfidf_disjoint | 665.792 | 645.542 | 464.041 | -28.12% | -30.30% |
| 16_Tfidf_limit_one | 897.291 | 887.208 | 700.334 | -21.06% | -21.95% |
| 16_Tfidf_limit_zero | 2.666 | 2.666 | 2.667 | +0.04% | +0.04% |
| 16_Tfidf_multi | 2169.875 | 2158.417 | 2160.292 | +0.09% | -0.44% |
| 16_Tfidf_punctuation | 141.542 | 142.750 | 144.584 | +1.28% | +2.15% |
| 16_Tfidf_repeated | 1450.083 | 1431.750 | 1228.708 | -14.18% | -15.27% |
| 16_Tfidf_scope | 1222.000 | 1216.459 | 1026.791 | -15.59% | -15.97% |
| 16_Tfidf_single | 1401.625 | 1367.167 | 1172.084 | -14.27% | -16.38% |
| 16_Tfidf_unicode | 1604.000 | 1575.875 | 1414.041 | -10.27% | -11.84% |
| 16_Tfidf_unknown | 68.083 | 68.292 | 68.417 | +0.18% | +0.49% |
| 16_Tfidf_unknown_added | 1439.084 | 1411.000 | 1212.500 | -14.07% | -15.75% |
| 16_Tfidf_wide | 18747.458 | 18592.458 | 18891.000 | +1.61% | +0.77% |
| 1_Bm25_disjoint | 137.583 | 130.750 | 108.917 | -16.70% | -20.84% |
| 1_Bm25_limit_one | 206.542 | 202.750 | 173.750 | -14.30% | -15.88% |
| 1_Bm25_limit_zero | 2.666 | 2.667 | 2.667 | +0.00% | +0.04% |
| 1_Bm25_multi | 290.417 | 284.791 | 288.000 | +1.13% | -0.83% |
| 1_Bm25_punctuation | 141.708 | 142.917 | 144.959 | +1.43% | +2.29% |
| 1_Bm25_repeated | 273.291 | 281.625 | 245.041 | -12.99% | -10.34% |
| 1_Bm25_scope | 210.416 | 210.541 | 182.292 | -13.42% | -13.37% |
| 1_Bm25_single | 238.042 | 249.208 | 235.500 | -5.50% | -1.07% |
| 1_Bm25_unicode | 464.000 | 448.250 | 421.875 | -5.88% | -9.08% |
| 1_Bm25_unknown | 69.250 | 68.500 | 67.292 | -1.76% | -2.83% |
| 1_Bm25_unknown_added | 250.875 | 240.458 | 221.042 | -8.07% | -11.89% |
| 1_Bm25_wide | 4382.209 | 4467.333 | 4316.750 | -3.37% | -1.49% |
| 1_Tfidf_disjoint | 130.958 | 135.875 | 107.250 | -21.07% | -18.10% |
| 1_Tfidf_limit_one | 210.958 | 212.375 | 182.375 | -14.13% | -13.55% |
| 1_Tfidf_limit_zero | 2.625 | 2.708 | 2.708 | +0.00% | +3.16% |
| 1_Tfidf_multi | 290.375 | 285.084 | 282.583 | -0.88% | -2.68% |
| 1_Tfidf_punctuation | 143.125 | 143.834 | 146.416 | +1.80% | +2.30% |
| 1_Tfidf_repeated | 274.500 | 279.125 | 240.750 | -13.75% | -12.30% |
| 1_Tfidf_scope | 215.167 | 220.083 | 192.417 | -12.57% | -10.57% |
| 1_Tfidf_single | 215.250 | 215.416 | 183.000 | -15.05% | -14.98% |
| 1_Tfidf_unicode | 456.000 | 456.334 | 427.000 | -6.43% | -6.36% |
| 1_Tfidf_unknown | 68.750 | 68.833 | 67.458 | -2.00% | -1.88% |
| 1_Tfidf_unknown_added | 245.250 | 249.208 | 217.083 | -12.89% | -11.49% |
| 1_Tfidf_wide | 4320.583 | 4432.917 | 4329.084 | -2.34% | +0.20% |
| 33_Bm25_disjoint | 1328.416 | 1246.375 | 973.542 | -21.89% | -26.71% |
| 33_Bm25_limit_one | 1600.250 | 1505.416 | 1220.041 | -18.96% | -23.76% |
| 33_Bm25_limit_zero | 2.667 | 2.625 | 2.667 | +1.60% | +0.00% |
| 33_Bm25_multi | 3401.416 | 3356.792 | 3436.584 | +2.38% | +1.03% |
| 33_Bm25_punctuation | 140.291 | 142.583 | 145.958 | +2.37% | +4.04% |
| 33_Bm25_repeated | 2100.750 | 2083.333 | 1702.167 | -18.30% | -18.97% |
| 33_Bm25_scope | 2115.541 | 2063.417 | 1749.542 | -15.21% | -17.30% |
| 33_Bm25_single | 2025.250 | 2015.875 | 1698.041 | -15.77% | -16.16% |
| 33_Bm25_unicode | 2197.709 | 2196.583 | 1852.791 | -15.65% | -15.69% |
| 33_Bm25_unknown | 68.292 | 68.208 | 67.875 | -0.49% | -0.61% |
| 33_Bm25_unknown_added | 2084.250 | 2071.875 | 1704.459 | -17.73% | -18.22% |
| 33_Bm25_wide | 28379.250 | 28878.875 | 28620.458 | -0.89% | +0.85% |
| 33_Tfidf_disjoint | 1299.917 | 1261.291 | 966.708 | -23.36% | -25.63% |
| 33_Tfidf_limit_one | 1647.500 | 1643.458 | 1375.375 | -16.31% | -16.52% |
| 33_Tfidf_limit_zero | 2.667 | 2.625 | 2.667 | +1.60% | +0.00% |
| 33_Tfidf_multi | 3538.208 | 3523.666 | 3595.500 | +2.04% | +1.62% |
| 33_Tfidf_punctuation | 142.750 | 141.708 | 142.959 | +0.88% | +0.15% |
| 33_Tfidf_repeated | 2226.459 | 2218.625 | 1906.000 | -14.09% | -14.39% |
| 33_Tfidf_scope | 2155.334 | 2146.250 | 1823.833 | -15.02% | -15.38% |
| 33_Tfidf_single | 2179.584 | 2151.709 | 1858.375 | -13.63% | -14.74% |
| 33_Tfidf_unicode | 2345.041 | 2329.125 | 1973.375 | -15.27% | -15.85% |
| 33_Tfidf_unknown | 67.917 | 69.250 | 67.916 | -1.93% | -0.00% |
| 33_Tfidf_unknown_added | 2209.125 | 2189.958 | 1872.875 | -14.48% | -15.22% |
| 33_Tfidf_wide | 28071.000 | 28897.125 | 28941.542 | +0.15% | +3.10% |
| 4_Bm25_disjoint | 196.708 | 193.375 | 158.542 | -18.01% | -19.40% |
| 4_Bm25_limit_one | 293.708 | 285.291 | 235.791 | -17.35% | -19.72% |
| 4_Bm25_limit_zero | 2.667 | 2.667 | 2.667 | +0.00% | +0.00% |
| 4_Bm25_multi | 643.958 | 657.792 | 651.834 | -0.91% | +1.22% |
| 4_Bm25_punctuation | 141.291 | 145.459 | 145.417 | -0.03% | +2.92% |
| 4_Bm25_repeated | 467.291 | 471.333 | 413.209 | -12.33% | -11.57% |
| 4_Bm25_scope | 326.584 | 323.250 | 281.917 | -12.79% | -13.68% |
| 4_Bm25_single | 423.125 | 427.375 | 365.167 | -14.56% | -13.70% |
| 4_Bm25_unicode | 653.666 | 645.875 | 586.667 | -9.17% | -10.25% |
| 4_Bm25_unknown | 68.750 | 67.542 | 66.542 | -1.48% | -3.21% |
| 4_Bm25_unknown_added | 452.834 | 452.667 | 398.584 | -11.95% | -11.98% |
| 4_Bm25_wide | 7882.791 | 7851.792 | 7750.333 | -1.29% | -1.68% |
| 4_Tfidf_disjoint | 201.416 | 193.666 | 156.958 | -18.95% | -22.07% |
| 4_Tfidf_limit_one | 311.792 | 313.750 | 256.375 | -18.29% | -17.77% |
| 4_Tfidf_limit_zero | 2.667 | 2.667 | 2.667 | +0.00% | +0.00% |
| 4_Tfidf_multi | 682.583 | 687.166 | 676.875 | -1.50% | -0.84% |
| 4_Tfidf_punctuation | 142.166 | 142.542 | 145.208 | +1.87% | +2.14% |
| 4_Tfidf_repeated | 498.000 | 494.083 | 434.709 | -12.02% | -12.71% |
| 4_Tfidf_scope | 341.583 | 333.750 | 286.958 | -14.02% | -15.99% |
| 4_Tfidf_single | 437.166 | 439.625 | 378.292 | -13.95% | -13.47% |
| 4_Tfidf_unicode | 685.083 | 675.250 | 612.709 | -9.26% | -10.56% |
| 4_Tfidf_unknown | 69.833 | 67.958 | 67.250 | -1.04% | -3.70% |
| 4_Tfidf_unknown_added | 482.792 | 471.750 | 415.041 | -12.02% | -14.03% |
| 4_Tfidf_wide | 7792.791 | 7803.583 | 7750.250 | -0.68% | -0.55% |

Higher candidate medians versus 0.4.16: **17 of 96**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](measurements/0.4.17/small-reference-summary.json). No threshold-based rejection or removal of slow timing samples is used.

### Second preplanned small Core campaign

#### Original six warm cases

| Fixture | 0.4.15 median ns | 0.4.16 median ns | 0.4.17 median ns | vs 0.4.16 | vs 0.4.15 |
| --- | --- | --- | --- | --- | --- |
| tiny16_bm25 | 1301.334 | 1267.625 | 1031.833 | -18.60% | -20.71% |
| tiny16_tfidf | 1321.750 | 1359.958 | 1100.792 | -19.06% | -16.72% |
| tiny4_bm25 | 409.750 | 401.917 | 359.041 | -10.67% | -12.38% |
| tiny4_tfidf | 462.875 | 437.000 | 371.791 | -14.92% | -19.68% |
| tiny_bm25 | 231.250 | 217.959 | 195.750 | -10.19% | -15.35% |
| tiny_tfidf | 222.875 | 210.334 | 198.709 | -5.53% | -10.84% |

Higher candidate medians versus 0.4.16: **0 of 6**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](measurements/0.4.17/small-confirmation-summary.json). No threshold-based rejection or removal of slow timing samples is used.

#### All 96 extended cases

| Fixture | 0.4.15 median ns | 0.4.16 median ns | 0.4.17 median ns | vs 0.4.16 | vs 0.4.15 |
| --- | --- | --- | --- | --- | --- |
| 16_Bm25_disjoint | 705.458 | 676.000 | 495.709 | -26.67% | -29.73% |
| 16_Bm25_limit_one | 906.875 | 856.500 | 661.000 | -22.83% | -27.11% |
| 16_Bm25_limit_zero | 2.917 | 2.875 | 3.125 | +8.70% | +7.13% |
| 16_Bm25_multi | 2278.167 | 2254.958 | 2226.250 | -1.27% | -2.28% |
| 16_Bm25_punctuation | 147.584 | 150.000 | 150.167 | +0.11% | +1.75% |
| 16_Bm25_repeated | 1540.458 | 1450.458 | 1223.458 | -15.65% | -20.58% |
| 16_Bm25_scope | 1258.375 | 1243.333 | 1035.208 | -16.74% | -17.73% |
| 16_Bm25_single | 1507.875 | 1385.875 | 1189.541 | -14.17% | -21.11% |
| 16_Bm25_unicode | 1643.000 | 1606.916 | 1364.333 | -15.10% | -16.96% |
| 16_Bm25_unknown | 72.416 | 72.333 | 70.167 | -2.99% | -3.11% |
| 16_Bm25_unknown_added | 1475.542 | 1419.583 | 1209.958 | -14.77% | -18.00% |
| 16_Bm25_wide | 21127.667 | 20478.166 | 19889.584 | -2.87% | -5.86% |
| 16_Tfidf_disjoint | 713.583 | 662.542 | 514.834 | -22.29% | -27.85% |
| 16_Tfidf_limit_one | 1015.875 | 927.583 | 775.208 | -16.43% | -23.69% |
| 16_Tfidf_limit_zero | 3.042 | 2.917 | 2.875 | -1.44% | -5.49% |
| 16_Tfidf_multi | 2340.791 | 2332.125 | 2373.041 | +1.75% | +1.38% |
| 16_Tfidf_punctuation | 158.333 | 149.958 | 151.958 | +1.33% | -4.03% |
| 16_Tfidf_repeated | 1600.333 | 1624.417 | 1308.917 | -19.42% | -18.21% |
| 16_Tfidf_scope | 1369.375 | 1295.625 | 1095.958 | -15.41% | -19.97% |
| 16_Tfidf_single | 1488.417 | 1550.708 | 1222.375 | -21.17% | -17.87% |
| 16_Tfidf_unicode | 1734.125 | 1670.500 | 1442.417 | -13.65% | -16.82% |
| 16_Tfidf_unknown | 74.209 | 75.542 | 71.583 | -5.24% | -3.54% |
| 16_Tfidf_unknown_added | 1579.250 | 1481.959 | 1323.125 | -10.72% | -16.22% |
| 16_Tfidf_wide | 21121.500 | 20678.208 | 20006.667 | -3.25% | -5.28% |
| 1_Bm25_disjoint | 141.333 | 139.791 | 114.167 | -18.33% | -19.22% |
| 1_Bm25_limit_one | 212.083 | 211.125 | 199.083 | -5.70% | -6.13% |
| 1_Bm25_limit_zero | 2.792 | 2.708 | 2.916 | +7.68% | +4.44% |
| 1_Bm25_multi | 304.458 | 294.542 | 301.291 | +2.29% | -1.04% |
| 1_Bm25_punctuation | 159.875 | 148.458 | 154.000 | +3.73% | -3.67% |
| 1_Bm25_repeated | 288.000 | 290.167 | 252.125 | -13.11% | -12.46% |
| 1_Bm25_scope | 222.292 | 216.833 | 195.083 | -10.03% | -12.24% |
| 1_Bm25_single | 248.459 | 260.417 | 242.042 | -7.06% | -2.58% |
| 1_Bm25_unicode | 540.458 | 461.042 | 444.125 | -3.67% | -17.82% |
| 1_Bm25_unknown | 73.208 | 71.542 | 77.625 | +8.50% | +6.03% |
| 1_Bm25_unknown_added | 250.542 | 249.834 | 234.708 | -6.05% | -6.32% |
| 1_Bm25_wide | 4714.542 | 5089.375 | 4922.042 | -3.29% | +4.40% |
| 1_Tfidf_disjoint | 138.667 | 136.042 | 113.916 | -16.26% | -17.85% |
| 1_Tfidf_limit_one | 227.833 | 221.375 | 200.875 | -9.26% | -11.83% |
| 1_Tfidf_limit_zero | 2.792 | 2.833 | 2.792 | -1.45% | +0.00% |
| 1_Tfidf_multi | 311.792 | 303.041 | 301.500 | -0.51% | -3.30% |
| 1_Tfidf_punctuation | 149.791 | 153.500 | 155.500 | +1.30% | +3.81% |
| 1_Tfidf_repeated | 291.834 | 280.958 | 256.458 | -8.72% | -12.12% |
| 1_Tfidf_scope | 228.125 | 228.292 | 207.416 | -9.14% | -9.08% |
| 1_Tfidf_single | 223.750 | 219.125 | 197.041 | -10.08% | -11.94% |
| 1_Tfidf_unicode | 473.500 | 485.708 | 455.333 | -6.25% | -3.84% |
| 1_Tfidf_unknown | 71.625 | 71.083 | 77.041 | +8.38% | +7.56% |
| 1_Tfidf_unknown_added | 259.250 | 254.167 | 234.917 | -7.57% | -9.39% |
| 1_Tfidf_wide | 4677.458 | 4770.292 | 4808.583 | +0.80% | +2.80% |
| 33_Bm25_disjoint | 1410.791 | 1382.250 | 1032.750 | -25.28% | -26.80% |
| 33_Bm25_limit_one | 1672.959 | 1640.750 | 1331.958 | -18.82% | -20.38% |
| 33_Bm25_limit_zero | 2.875 | 2.792 | 3.000 | +7.45% | +4.35% |
| 33_Bm25_multi | 3744.291 | 3561.291 | 3601.125 | +1.12% | -3.82% |
| 33_Bm25_punctuation | 153.125 | 154.375 | 148.333 | -3.91% | -3.13% |
| 33_Bm25_repeated | 2274.542 | 2210.417 | 1898.833 | -14.10% | -16.52% |
| 33_Bm25_scope | 2426.167 | 2197.917 | 1890.333 | -13.99% | -22.09% |
| 33_Bm25_single | 2267.666 | 2149.667 | 1778.125 | -17.28% | -21.59% |
| 33_Bm25_unicode | 2373.041 | 2341.541 | 1943.084 | -17.02% | -18.12% |
| 33_Bm25_unknown | 73.000 | 77.750 | 69.000 | -11.25% | -5.48% |
| 33_Bm25_unknown_added | 2396.959 | 2185.833 | 1905.375 | -12.83% | -20.51% |
| 33_Bm25_wide | 30162.292 | 30924.875 | 31688.417 | +2.47% | +5.06% |
| 33_Tfidf_disjoint | 1518.250 | 1330.542 | 1097.958 | -17.48% | -27.68% |
| 33_Tfidf_limit_one | 1877.084 | 1785.542 | 1516.041 | -15.09% | -19.23% |
| 33_Tfidf_limit_zero | 2.959 | 2.833 | 2.875 | +1.48% | -2.84% |
| 33_Tfidf_multi | 4041.417 | 3777.875 | 3957.000 | +4.74% | -2.09% |
| 33_Tfidf_punctuation | 153.917 | 151.791 | 154.000 | +1.46% | +0.05% |
| 33_Tfidf_repeated | 2399.042 | 2426.583 | 2295.041 | -5.42% | -4.34% |
| 33_Tfidf_scope | 2398.500 | 2463.416 | 1968.709 | -20.08% | -17.92% |
| 33_Tfidf_single | 2440.000 | 2285.625 | 2008.625 | -12.12% | -17.68% |
| 33_Tfidf_unicode | 2565.833 | 2550.084 | 2264.333 | -11.21% | -11.75% |
| 33_Tfidf_unknown | 73.000 | 74.833 | 71.708 | -4.18% | -1.77% |
| 33_Tfidf_unknown_added | 2556.958 | 2409.917 | 2232.500 | -7.36% | -12.69% |
| 33_Tfidf_wide | 30143.167 | 30403.750 | 30860.583 | +1.50% | +2.38% |
| 4_Bm25_disjoint | 207.750 | 201.625 | 171.959 | -14.71% | -17.23% |
| 4_Bm25_limit_one | 303.625 | 288.750 | 264.375 | -8.44% | -12.93% |
| 4_Bm25_limit_zero | 2.750 | 2.791 | 3.458 | +23.90% | +25.75% |
| 4_Bm25_multi | 731.417 | 690.625 | 706.958 | +2.36% | -3.34% |
| 4_Bm25_punctuation | 159.167 | 149.208 | 154.833 | +3.77% | -2.72% |
| 4_Bm25_repeated | 487.209 | 502.125 | 458.959 | -8.60% | -5.80% |
| 4_Bm25_scope | 346.833 | 343.291 | 313.375 | -8.71% | -9.65% |
| 4_Bm25_single | 437.084 | 450.417 | 388.500 | -13.75% | -11.12% |
| 4_Bm25_unicode | 670.541 | 685.209 | 664.792 | -2.98% | -0.86% |
| 4_Bm25_unknown | 73.792 | 70.041 | 75.542 | +7.85% | +2.37% |
| 4_Bm25_unknown_added | 462.417 | 472.791 | 436.625 | -7.65% | -5.58% |
| 4_Bm25_wide | 8601.375 | 8595.625 | 8709.000 | +1.32% | +1.25% |
| 4_Tfidf_disjoint | 204.416 | 203.542 | 173.541 | -14.74% | -15.10% |
| 4_Tfidf_limit_one | 333.708 | 339.042 | 285.334 | -15.84% | -14.50% |
| 4_Tfidf_limit_zero | 2.917 | 3.000 | 3.000 | +0.00% | +2.85% |
| 4_Tfidf_multi | 747.625 | 749.375 | 737.042 | -1.65% | -1.42% |
| 4_Tfidf_punctuation | 154.417 | 157.625 | 155.292 | -1.48% | +0.57% |
| 4_Tfidf_repeated | 520.750 | 538.541 | 490.791 | -8.87% | -5.75% |
| 4_Tfidf_scope | 358.334 | 352.333 | 317.291 | -9.95% | -11.45% |
| 4_Tfidf_single | 471.000 | 467.708 | 425.084 | -9.11% | -9.75% |
| 4_Tfidf_unicode | 726.000 | 756.458 | 655.959 | -13.29% | -9.65% |
| 4_Tfidf_unknown | 71.500 | 75.833 | 76.667 | +1.10% | +7.23% |
| 4_Tfidf_unknown_added | 505.584 | 540.209 | 455.791 | -15.63% | -9.85% |
| 4_Tfidf_wide | 8778.542 | 8604.042 | 8406.917 | -2.29% | -4.23% |

Higher candidate medians versus 0.4.16: **24 of 96**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](measurements/0.4.17/small-confirmation-summary.json). No threshold-based rejection or removal of slow timing samples is used.

## Direct canonical search with explicit control modes

Fourteen processes (seven alternating pairs) retain one warmed Engine each. Ten rounds rotate plain, deadline and progress within each query/ranker pair: **8,400 timed Retriever calls**, excluding loading/readiness, stdio and profiling. Each progress request includes an observer and the same deadline/cancellation setup. Every signature agrees; **140 complete original response JSON pairs match**, and control modes also match plain output. Full readiness source text stays private. These are direct retrieval boundaries, not MCP round trips or capacity evidence.

| Control | Ranker | 0.4.16 median / p95 ms | 0.4.17 median / p95 ms | Median change |
| --- | --- | --- | --- | --- |
| plain | Bm25 | 1.977 / 3.952 | 1.971 / 3.916 | -0.30% |
| plain | Tfidf | 2.152 / 4.130 | 2.185 / 4.133 | +1.54% |
| deadline | Bm25 | 2.248 / 4.481 | 2.238 / 4.457 | -0.48% |
| deadline | Tfidf | 2.479 / 4.816 | 2.517 / 4.849 | +1.54% |
| progress | Bm25 | 2.233 / 4.494 | 2.254 / 4.471 | +0.91% |
| progress | Tfidf | 2.469 / 4.958 | 2.497 / 4.931 | +1.13% |

| Query | Candidates | Progress BM25 ms: old → new | Progress TF-IDF ms: old → new |
| --- | --- | --- | --- |
| 0 | 12365 | 2.034 → 2.087 | 2.289 → 2.335 |
| 1 | 13864 | 2.303 → 2.356 | 2.644 → 2.627 |
| 2 | 24299 | 3.882 → 3.871 | 4.285 → 4.307 |
| 3 | 596 | 0.308 → 0.298 | 0.298 → 0.294 |
| 4 | 15403 | 2.395 → 2.429 | 2.640 → 2.683 |
| 5 | 24140 | 3.797 → 3.890 | 4.269 → 4.316 |
| 6 | 3358 | 0.699 → 0.712 | 0.742 → 0.759 |
| 7 | 29600 | 4.106 → 4.037 | 4.723 → 4.601 |
| 8 | 11504 | 1.877 → 1.825 | 2.062 → 2.008 |
| 9 | 0 | 0.002 → 0.002 | 0.001 → 0.001 |

## Separate canonical three-version reference

Five rotating triples compare fresh exact 0.4.15/0.4.16/candidate servers on canonical knowledge with requested progress. Fifteen servers validate **1,020 actual schema replies**; **680 complete structured comparisons** against the candidate match by RPC ID except checked status version and version-qualified schema URNs. Schemas, source ranges and inputs agree. This control is separate from the two ordinary campaigns. It checks whether the previous large-query benefit remains visible against the older release; it does not establish the cause of any historical regression.

| Version | Ranker | Initial median / p95 ms | Repeated median / p95 ms | Peak child RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.15 | bm25 | 7.379 / 20.864 | 6.531 / 16.865 | 1724.469 |
| 0.4.15 | tfidf | 22.373 / 46.056 | 6.581 / 20.051 | 1724.469 |
| 0.4.16 | bm25 | 3.572 / 9.614 | 3.157 / 8.242 | 1724.422 |
| 0.4.16 | tfidf | 21.253 / 39.218 | 3.420 / 10.173 | 1724.422 |
| 0.4.17 | bm25 | 3.443 / 6.114 | 3.244 / 5.810 | 1725.000 |
| 0.4.17 | tfidf | 20.887 / 34.301 | 3.745 / 7.343 | 1725.000 |

| Query | 0.4.15 repeated TF-IDF ms | 0.4.16 repeated TF-IDF ms | 0.4.17 repeated TF-IDF ms |
| --- | --- | --- | --- |
| 0 | 5.739 | 2.900 | 3.474 |
| 1 | 6.997 | 3.471 | 3.778 |
| 2 | 12.499 | 5.285 | 6.203 |
| 3 | 0.725 | 0.696 | 0.729 |
| 4 | 6.503 | 3.390 | 3.687 |
| 5 | 12.652 | 5.785 | 5.903 |
| 6 | 2.411 | 1.272 | 1.434 |
| 7 | 13.614 | 6.056 | 6.516 |
| 8 | 6.268 | 4.002 | 3.863 |
| 9 | 0.131 | 0.120 | 0.143 |

## Canonical first-pass heap traffic

Two instrumented processes index all 167,942 records before profiling the same ten initial TF-IDF queries, then drop results before net deltas. Direct candidate collection can reduce allocation traffic; the previous large-query posting cursors and norm memo/cache still exist. Heap traffic, net persistent hints, stack state and whole-process RSS are different costs. Instrumented durations are excluded from latency comparisons.

| Metric | 0.4.16 | 0.4.17 |
| --- | --- | --- |
| Retained hint bytes | 2228312 | 2228312 |
| Retained hint blocks | 2 | 2 |
| Total query allocated bytes | 7217203 | 7216291 |
| Total query allocation blocks | 451 | 439 |

The zero-limit extension includes an immediate empty-return boundary, so its tiny absolute ns/call values and percentage changes are not comparable with ranked searches. Every original value remains in its row.

## Development pilot and host-load limits

The [development pilot](measurements/0.4.17/development-pilot-summary.json) remains separate from all final samples. Its six-case rotating triples supported the candidate choice; the implementation and measurements are workload-specific, not proof that all historical small-query variation came from the heap. All source/lock/binary identities and raw observations remain locally retained.

Two-second monitoring retains **348 snapshots**, including **92 with other rustc processes**, maximum 3. All managed compilers finish before timing. Other workloads are not stopped, and snapshots do not prove continuous idleness or identify a timing cause. CPU counts include the active benchmark workers.

| Stage start | Other rustc count | Other rustc CPU total | Processes over 50% CPU |
| --- | --- | --- | --- |
| small-reference | 0 | 0.000 | 1 |
| retrieval | 0 | 0.000 | 1 |
| query | 0 | 0.000 | 1 |
| build | 0 | 0.000 | 1 |
| controlled | 0 | 0.000 | 1 |
| mcp | 0 | 0.000 | 1 |
| corpus | 1 | 1240.100 | 2 |
| mcp-confirmation | 1 | 199.800 | 2 |
| reference | 0 | 0.000 | 2 |
| small-confirmation | 1 | 99.300 | 3 |

All observed slower cases remain in their own distributions; no sample is replaced because of load. One machine, randomized hashing, exposed questions, tuned fixtures and finite trials limit generalization.

## Retained Core controls

These controls retain positive and negative timing changes. Single known-term requests now use the direct candidate path. Multiple known-term requests retain the heap/set strategies; large frequency cursors remain available afterward. Read every row independently, including slower cases. Seven alternating pairs give fourteen plain processes per family. Norms are initialized for warm timing; two separate DHAT processes measure allocations. Instrumented durations are excluded from latency comparisons. Every result signature agrees and temporary-call retained deltas are zero.

### Small warm Core: 84,000 calls on 1/4/16 entries

| Fixture | Median change | Blocks/call: old → new | Allocated bytes/call: old → new |
| --- | --- | --- | --- |
| tiny16_bm25 | -15.96% | 32.0 → 29.0 | 2712.0 → 2536.0 |
| tiny16_tfidf | -14.40% | 32.0 → 29.0 | 2712.0 → 2536.0 |
| tiny4_bm25 | -9.48% | 16.0 → 15.0 | 1284.0 → 1204.0 |
| tiny4_tfidf | -10.70% | 16.0 → 15.0 | 1284.0 → 1204.0 |
| tiny_bm25 | -13.18% | 10.0 → 9.0 | 858.0 → 778.0 |
| tiny_tfidf | -5.38% | 10.0 → 9.0 | 858.0 → 778.0 |

### All 25 warm controls: 282,800 calls

| Fixture | Median change | Blocks/call: old → new | Allocated bytes/call: old → new |
| --- | --- | --- | --- |
| coverage_repeated | +2.99% | 1047.0 → 1047.0 | 92942.0 → 92942.0 |
| coverage_small | +2.58% | 784.0 → 784.0 | 78046.0 → 78046.0 |
| excerpt_ascii | -28.81% | 762.0 → 762.0 | 252438.0 → 252438.0 |
| excerpt_mixed | -25.11% | 569.0 → 569.0 | 417113.0 → 417113.0 |
| excerpt_unicode | -11.86% | 197.0 → 197.0 | 72254.0 → 72254.0 |
| long_repeated | -0.56% | 4.0 → 4.0 | 18798.0 → 18798.0 |
| mixed_unicode | -1.54% | 7.0 → 7.0 | 3155.0 → 3155.0 |
| punctuation | -0.12% | 1.0 → 1.0 | 768.0 → 768.0 |
| relevant_retrieval | -0.21% | 812.0 → 812.0 | 93562.0 → 93562.0 |
| repeated_search | -11.53% | 39.0 → 35.0 | 19736.0 → 19432.0 |
| repeated_short | +3.80% | 4.0 → 4.0 | 1656.0 → 1656.0 |
| sparse_bm25_128 | +1.25% | 359.0 → 359.0 | 29074.0 → 29074.0 |
| sparse_bm25_33 | +1.33% | 241.0 → 241.0 | 16309.0 → 16309.0 |
| sparse_bm25_512 | +0.40% | 1337.0 → 1337.0 | 119124.0 → 119124.0 |
| sparse_tfidf_128 | -2.35% | 474.0 → 474.0 | 35140.0 → 35140.0 |
| sparse_tfidf_33 | +0.08% | 241.0 → 241.0 | 16309.0 → 16309.0 |
| sparse_tfidf_512 | -1.00% | 1337.0 → 1337.0 | 119124.0 → 119124.0 |
| tfidf_union_32 | -1.14% | 244.0 → 244.0 | 19834.0 → 19834.0 |
| tfidf_union_33 | +2.03% | 296.0 → 296.0 | 27854.0 → 27854.0 |
| tfidf_warm | +1.92% | 71.0 → 71.0 | 19335.0 → 19335.0 |
| union_32 | +0.88% | 244.0 → 244.0 | 19834.0 → 19834.0 |
| union_33 | +0.88% | 296.0 → 296.0 | 27854.0 → 27854.0 |
| unique_long | +1.63% | 299.0 → 299.0 | 32672.0 → 32672.0 |
| unique_search | +2.78% | 72.0 → 72.0 | 19350.0 → 19350.0 |
| unique_short | +0.36% | 296.0 → 296.0 | 18548.0 → 18548.0 |

### Cold norm and heap controls

Six fixtures make twenty fresh-index-clone calls each across fourteen processes: 1,680 calls. Clone/setup is outside timing. Process/instruction/filesystem caches remain; cold means norm hints, not disks. Separate heap runs drop results before net retained measurements. The existing inline/optional heap word memo and persistent norm storage remain. Large small-term requests may add up to four temporary posting cursors; canonical allocation traffic below records that cost. Heap profiles exclude stack state/control fields and temporary frames.

| Fixture | Old median / p95 ns | New median / p95 ns |
| --- | --- | --- |
| index | 118500.000 / 132812.800 | 113104.500 / 124474.650 |
| wide | 37812.000 / 57058.300 | 35750.000 / 41802.400 |
| sparse | 491000.000 / 546923.250 | 486333.000 / 542255.950 |
| tiny | 542.000 / 1210.100 | 625.000 / 1340.250 |
| tiny4 | 834.000 / 1334.000 | 854.500 / 1375.000 |
| tiny16 | 2708.000 / 3586.050 | 2500.000 / 3423.250 |

| Cold fixture | Allocated bytes: old → new | Retained hint bytes: old → new |
| --- | --- | --- |
| index | 92660 → 92356 | 34904 → 34904 |
| wide | 19549 → 19549 | 4440 → 4440 |
| sparse | 140704 → 140704 | 8792 → 8792 |
| tiny | 1014 → 934 | 156 → 156 |
| tiny4 | 1584 → 1504 | 224 → 224 |
| tiny16 | 3844 → 3668 | 632 → 632 |

Canonical first-pass heap records, all samples, blocks and cold tails remain in public JSON. Retained hints, heap traffic and whole-process RAM are different measures.

## Finite mixed control, documents and construction

The 6,000-entry mixed control retains four continuous warm readers while 100 initially uncached-group driver queries finish, then makes two quiescent probes. Five alternating pairs give 1,000 driver calls and twenty quiescent probes. Warm counts vary with the stop condition and new-group calls can become warm. This finite control is not fixed-load throughput, capacity or fairness evidence.

| Mixed work, ns | Old median / p95 | New median / p95 |
| --- | --- | --- |
| new_group_query_ns | 131229.000 / 190796.150 | 139187.500 / 276200.350 |
| post_readers_first_ns | 149042.000 / 165550.000 | 172833.000 / 175216.600 |
| post_readers_second_ns | 93292.000 / 99733.600 | 82500.000 / 90282.800 |

The private document corpus retains 72 Markdown files, 2,755,468 bytes and 1,910 chunks. All 342 authored questions (276 positive, 66 unsupported) are exposed regressions, not fresh holdouts or logged production queries. Literal BM25, prepared BM25 and coverage retain six passages, 100 candidates, 2,000 bytes/pass and 12,000 total bytes. All **1,026 complete original response JSON strings match byte-for-byte**, including scores, ordering, evidence, source text and positions. One warm-up and five repeats/question/configuration/version give **10,260 timed round trips**. These three configurations are BM25 controls; independent score oracles and actual protocol pairs cover TF-IDF. No new relevance judgment or model reply is produced.

All document timing aggregates and per-question results remain in the records. Construction uses forty plain processes on canonical data and three synthetic distributions, plus six separate profiles. Parsing/generation is outside construction timing; stats/signatures/live derived heap agree. The execution/shared-index worker checks all 167,942 entries through 128 changes/restoration, rebuilds, JSON round trips, retained clones, parallel readers and cancellation/deadline rejection. Portable exports and source hashes agree.

### Document round trips

| Case | Old median / p95 ms | New median / p95 ms |
| --- | --- | --- |
| baseline | 0.809 / 1.223 | 0.844 / 1.329 |
| prepared | 0.855 / 1.344 | 0.887 / 1.395 |
| evidence | 0.841 / 1.343 | 0.874 / 1.389 |

### Construction only

| Case | Old median / p95 ms | New median / p95 ms |
| --- | --- | --- |
| canonical | 4942.949 / 4990.651 | 4976.051 / 5007.764 |
| ascii | 14.706 / 15.016 | 14.475 / 14.772 |
| mixed | 34.216 / 34.835 | 33.718 / 33.920 |
| long_tokens | 1.565 / 1.605 | 1.570 / 1.592 |

## Limits and preservation

Measurements use one AC-powered Apple M5 Max with 64 GB, Rust 1.98.0 and Python 3.15.2. Stage-start CPU/compiler snapshots remain. Own compilers finish before timing; this does not establish continuously idle hardware. Randomized hashing, scheduling, one machine/corpus, exposed questions and finite process counts limit generalization. The separate development pilot remains; assess final per-case medians and tails independently. Host-load sampling records observations rather than continuous idleness. No universal latency, total-memory, semantic-quality, token-saving or security-audit claim is made.

Canonical knowledge remains 167,942 entries from 35 PDFs and 49,086 physical pages, SHA-256 `13168d87872a137355fd8499c90a6084f52243a2ec7456f29b5fcb2a1c7423c3`. The PDF parser is unchanged; this is not a fresh large-PDF extraction campaign. Lexical overlap and exact quotes do not prove factual support or entailment. No model, daemon or DenkMaps integration is added.

Preserve PDFs, knowledge, snapshots, all pilot/final measurements, slower results, raw transcripts, profiles, exact small executables and locks. After verified publication archive package/proof, then delete only the named temporary Cargo/Miri targets. Default features remain empty.
