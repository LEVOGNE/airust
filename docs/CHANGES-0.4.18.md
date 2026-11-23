# AIRust 0.4.18: direct merging for large two-term searches

When exactly two known indexed query terms remain, the Core now merges their existing sorted posting-ID lists directly. It compares the two current IDs, keeps the earlier one, and advances both lists on equality. It avoids constructing and maintaining a general merge heap for only two lists. The private helper has an inline-never boundary to keep this work outside the caller’s generated body. Matching live Arc identities is a shortcut; equal IDs with separate allocations use exact string order. Frequencies and scores are not approximated.

The single-list path from 0.4.17 and large-query frequency cursors from 0.4.16 remain. More than two known terms retain the existing heap/set strategies. Query analysis, context/custom filters, original contribution order, norm hints, winner explanations, exact source positions, API signatures, formats and schemas retain their contracts. No dependency, feature, model, new cache or DenkMaps integration is added.

In two preplanned ordinary MCP campaigns, repeated canonical TF-IDF median changes are **-34.76% and -13.45%** versus fresh exact registry 0.4.17. One-entry repeated MCP changes are **-2.42% and -11.22%**. Negative means shorter time. These mixed-query aggregates are not per-query guarantees: slower cases, tails, initial queries, startup, allocation traffic and RAM remain in [the complete records](measurements/0.4.18/README.md). A separate fresh 0.4.16/0.4.17/0.4.18 reference checks the earlier large-query regression independently.

The direct large-query controls have roughly **10–14% shorter medians**. Initial canonical TF-IDF medians in the two ordinary campaigns change by **-19.08% and -2.70%**. Counterfindings remain: the separate canonical reference has a **+6.09%** repeated TF-IDF median versus 0.4.17, and the second ordinary synthetic wide case has **+67.35%**. A separate exploratory wide follow-up observes **+3.58%**, still slower. The original values stay visible. These results do not establish that every large or small 0.4.17 latency is preserved. Read the per-query, mean and process-total views alongside mixed-query medians.

## Execution and resource contract

Every visited posting invokes the original checkpoint/progress visitor, including both postings of an overlapping ID. Both remaining-list tails use the same visitor. Candidate IDs stay sorted and unique; error propagation returns no partial public result. The candidate Vec initially reserves min(32, the larger posting count) borrowed slots, a lower bound on the union up to that cap. It can still grow with input. Two temporary BTree iterators replace the temporary merge heap; no state is retained by the index. Iterator/stack state is not measured by heap profiles. This is not a total-memory or hard real-time bound.

The same Engine supplies direct Core, Retriever and optional MCP work. Worker admission, connection cancellation/drain, original absolute deadlines, progress delivery deadlines and frame/rate/result budgets remain. The small MCP timer fix remains and startup indexing stays outside tool deadlines. No checkpoint is removed or sampled away to obtain speed. Read [Core performance](CORE-PERFORMANCE.md), [API](API.md), [cookbook](API-COOKBOOK.md), [MCP](MCP.md) and [security limits](MCP-SECURITY.md).

## Executed validation

The full matrix passes **241 default tests, 891 all-feature checks including 39 documentation examples, 39 isolated feature entries and five Python tests**. Strict Clippy/docs, Rust 1.89 provider-neutral and Rust 1.92 all-feature checks pass. API comparison with tagged 0.4.17 reports **229 pass, 31 skip**. An independent Rust 1.89 consumer checks retrieval, coverage, quotes, document positions and MCP/schema construction. The unchanged PDF backend passes 14 regressions and its 381-source-file provenance check. A refreshed dependency audit reports zero known vulnerabilities in its checked database; maintenance notices remain in the resource record.

Three new regressions compare the direct pair merge with an independent set union across shared/separately allocated equal IDs, Unicode/control IDs, empty/disjoint/intersecting lists and both tails. Every posting can be interrupted, including overlap visits; the public query rejects partial results before filtering and leaves knowledge unchanged. Existing hand-calculated small cases and the 1,025-record independent full-scan oracle check both rankers, one/two/many known terms, scopes, limits and cold/warm states. Retained hostile MCP, cancellation/deadline, stalled reader, duplicate token, disconnect and worker-drain regressions remain.

Twenty-three finite Tree Borrows Miri cases include the three new regressions. Only extra floating-point rounding injection is disabled; memory/alias checks remain. Tokio/PDF/FFI/ONNX execution is outside this finite scope. These checks are not an independent penetration test or security/soundness audit. Extracted-package tests, exact-commit CI and a fresh registry consumer gate publication, with package/VCS/checksum/tag/release-asset agreement.

## Frozen method and development

The baseline is fresh exact registry 0.4.17 (VCS c82e61530c201eed1967293fc0e117ee42a3e1c2). Separate references use fresh exact registry 0.4.16 (VCS 869a814934737ded051e9e2ec64a73395e3285a5). Candidate 0.4.18 uses identical programs, data and build profiles. Baseline/reference registry source files also match every file of their preserved checksum-verified published crate archives byte-for-byte before timing. Actual private source/manifest copies, locks, input hashes and measured binaries are verified and retained; read the [reproduction method](validation-0.4.18/README.md).

The separate three-triple development pilot compared direct plain/deadline/progress retrieval, canonical/tiny MCP and the original six small Core cases across all three versions. Complete responses and signatures agreed. Its raw samples, sources, locks and exact binaries remain separate from final timing. Improving a concrete two-list merge cost does not establish why the earlier 0.4.17 latency distributions differed. No pilot sample replaces a slower final result. An initial candidate showed slower cold MCP calls; all its samples remain separately preserved. A five-triple follow-up supported the compiler boundary, after which the complete selected-source validation and measurement plan was repeated.

Final timing starts after every managed compiler job. Two ordinary nine-case MCP campaigns, direct controlled retrieval, a fresh canonical three-version reference and two small Core three-version campaigns remain separate. The small campaigns protect previous gains and retain a 96-case query/filter/limit extension. All slower outcomes remain. Host-load samples record other work. Other workloads are not stopped; samples do not prove continuous idleness or identify a timing cause. Questions are exposed authored regressions rather than fresh production/holdout queries. No model-answer, new relevance judgment or fresh PDF extraction is claimed.

## Actual MCP: two complete ordinary campaigns

Each ordinary campaign uses nine cases and five alternating old/new pairs: **90 servers per campaign, 180 total**. Canonical knowledge, a revision-bound NumPy snapshot and broad synthetic input request progress; 1/4/16-entry inputs run with and without progress. Each server makes ten queries with both rankers, two repeated passes, four concurrent searches, matching/relations, rejected arguments and health. Snapshot cases check exact UTF-8 source readbacks.

Across both campaigns **13,880 actual result/error replies validate against advertised schemas**, and **6,940 complete old/new structured pairs match by RPC ID**, excluding only checked status version and version-qualified schema URNs. Counts: 3,600 initial searches, 7,200 repeats, 720 concurrent searches, 1,640 readbacks, 360 matching/relations, 180 rejected arguments and 180 health calls. Discovery, progress/no-progress, bounds, ordering, unchanged inputs and EOF shutdown pass.

Initial/first/repeated latency and startup remain separate. Search timing includes serialization, stdio, Python parsing and requested notifications; source follow-ups are outside it. Child peak RSS includes runtime/buffers/index and is distinct from allocation traffic. Read every unfavorable median and tail from its own row.

### First complete campaign

| Case | Ranker | Initial median / p95 ms: old → new | Repeated median / p95 ms: old → new | Repeated median change |
| --- | --- | --- | --- | --- |
| canonical | bm25 | 4.899 / 15.037 → 3.287 / 5.641 | 3.692 / 6.932 → 2.875 / 5.493 | -22.15% |
| canonical | tfidf | 24.639 / 51.549 → 19.937 / 34.366 | 4.730 / 10.185 → 3.086 / 6.294 | -34.76% |
| numpy | bm25 | 1.050 / 1.490 → 1.023 / 1.423 | 0.960 / 1.420 → 0.891 / 1.297 | -7.11% |
| numpy | tfidf | 1.310 / 3.141 → 1.278 / 3.072 | 0.867 / 1.497 → 0.827 / 1.319 | -4.55% |
| wide | bm25 | 0.531 / 0.684 → 0.596 / 0.728 | 0.448 / 0.794 → 0.459 / 0.751 | +2.60% |
| wide | tfidf | 0.481 / 0.894 → 0.536 / 0.901 | 0.436 / 0.828 → 0.457 / 0.857 | +4.87% |
| tiny | bm25 | 0.143 / 0.234 → 0.134 / 0.233 | 0.123 / 0.236 → 0.119 / 0.164 | -3.16% |
| tiny | tfidf | 0.135 / 0.183 → 0.136 / 0.178 | 0.122 / 0.153 → 0.119 / 0.138 | -2.42% |
| tiny4 | bm25 | 0.196 / 0.356 → 0.202 / 0.321 | 0.171 / 0.204 → 0.171 / 0.190 | -0.15% |
| tiny4 | tfidf | 0.195 / 0.261 → 0.195 / 0.257 | 0.176 / 0.214 → 0.169 / 0.218 | -3.79% |
| tiny16 | bm25 | 0.218 / 0.348 → 0.213 / 0.356 | 0.199 / 0.244 → 0.195 / 0.227 | -1.71% |
| tiny16 | tfidf | 0.222 / 0.343 → 0.214 / 0.332 | 0.198 / 0.261 → 0.195 / 0.235 | -1.16% |
| tiny-no-progress | bm25 | 0.080 / 0.177 → 0.079 / 0.177 | 0.069 / 0.193 → 0.069 / 0.111 | -0.48% |
| tiny-no-progress | tfidf | 0.077 / 0.121 → 0.076 / 0.124 | 0.068 / 0.135 → 0.069 / 0.087 | +2.13% |
| tiny4-no-progress | bm25 | 0.133 / 0.270 → 0.140 / 0.278 | 0.125 / 0.147 → 0.129 / 0.173 | +3.17% |
| tiny4-no-progress | tfidf | 0.131 / 0.193 → 0.140 / 0.221 | 0.124 / 0.175 → 0.130 / 0.192 | +4.97% |
| tiny16-no-progress | bm25 | 0.160 / 0.294 → 0.157 / 0.284 | 0.152 / 0.188 → 0.145 / 0.170 | -4.58% |
| tiny16-no-progress | tfidf | 0.170 / 0.315 → 0.157 / 0.275 | 0.151 / 0.196 → 0.146 / 0.182 | -3.46% |

| Case | Startup median ms: old → new | First TF-IDF median ms: old → new | Peak RSS median MiB: old → new |
| --- | --- | --- | --- |
| canonical | 6927.143 → 5173.964 | 32.283 → 23.525 | 1726.266 → 1724.406 |
| numpy | 155.649 → 151.194 | 1.992 → 1.730 | 57.453 → 57.141 |
| wide | 4.209 → 4.119 | 0.494 → 0.554 | 7.938 → 7.812 |
| tiny | 3.509 → 3.560 | 0.175 → 0.175 | 5.750 → 5.812 |
| tiny4 | 3.665 → 3.268 | 0.258 → 0.263 | 5.844 → 5.922 |
| tiny16 | 3.558 → 3.639 | 0.277 → 0.279 | 6.047 → 6.031 |
| tiny-no-progress | 3.454 → 3.520 | 0.120 → 0.122 | 5.734 → 5.781 |
| tiny4-no-progress | 3.837 → 3.853 | 0.195 → 0.216 | 5.891 → 5.859 |
| tiny16-no-progress | 3.776 → 3.814 | 0.253 → 0.223 | 6.047 → 6.156 |

### Second preplanned complete campaign

| Case | Ranker | Initial median / p95 ms: old → new | Repeated median / p95 ms: old → new | Repeated median change |
| --- | --- | --- | --- | --- |
| canonical | bm25 | 3.497 / 6.189 → 3.253 / 8.084 | 3.173 / 7.317 → 2.715 / 7.107 | -14.41% |
| canonical | tfidf | 20.937 / 39.571 → 20.371 / 39.103 | 3.551 / 9.485 → 3.073 / 8.074 | -13.45% |
| numpy | bm25 | 1.088 / 1.510 → 1.027 / 1.594 | 0.953 / 1.425 → 0.891 / 1.360 | -6.50% |
| numpy | tfidf | 1.258 / 3.154 → 1.136 / 3.135 | 0.846 / 1.510 → 0.829 / 1.754 | -1.92% |
| wide | bm25 | 0.855 / 5.309 → 0.881 / 5.533 | 0.636 / 3.213 → 0.856 / 6.054 | +34.50% |
| wide | tfidf | 0.883 / 6.154 → 0.953 / 5.904 | 0.640 / 2.837 → 1.072 / 6.768 | +67.35% |
| tiny | bm25 | 0.319 / 5.496 → 0.307 / 5.585 | 0.189 / 4.842 → 0.163 / 4.212 | -13.66% |
| tiny | tfidf | 0.290 / 4.121 → 0.249 / 3.378 | 0.183 / 3.590 → 0.163 / 2.164 | -11.22% |
| tiny4 | bm25 | 0.336 / 2.988 → 0.287 / 3.869 | 0.253 / 2.281 → 0.263 / 3.006 | +3.62% |
| tiny4 | tfidf | 0.322 / 4.747 → 0.269 / 1.721 | 0.258 / 1.370 → 0.270 / 3.062 | +4.67% |
| tiny16 | bm25 | 0.293 / 0.440 → 0.285 / 0.493 | 0.275 / 0.368 → 0.259 / 0.324 | -5.96% |
| tiny16 | tfidf | 0.288 / 0.450 → 0.289 / 0.490 | 0.272 / 0.374 → 0.260 / 0.342 | -4.21% |
| tiny-no-progress | bm25 | 0.114 / 0.200 → 0.106 / 0.226 | 0.079 / 0.166 → 0.081 / 0.166 | +2.11% |
| tiny-no-progress | tfidf | 0.108 / 0.195 → 0.107 / 0.199 | 0.079 / 0.134 → 0.080 / 0.104 | +0.92% |
| tiny4-no-progress | bm25 | 0.183 / 0.343 → 0.177 / 0.329 | 0.154 / 0.212 → 0.139 / 0.195 | -9.72% |
| tiny4-no-progress | tfidf | 0.176 / 0.241 → 0.166 / 0.248 | 0.155 / 0.226 → 0.139 / 0.200 | -10.34% |
| tiny16-no-progress | bm25 | 0.168 / 0.288 → 0.177 / 0.297 | 0.152 / 0.177 → 0.153 / 0.199 | +1.15% |
| tiny16-no-progress | tfidf | 0.175 / 0.291 → 0.179 / 0.307 | 0.154 / 0.189 → 0.155 / 0.206 | +0.72% |

| Case | Startup median ms: old → new | First TF-IDF median ms: old → new | Peak RSS median MiB: old → new |
| --- | --- | --- | --- |
| canonical | 5021.861 → 5297.137 | 24.534 → 21.301 | 1724.500 → 1724.219 |
| numpy | 152.821 → 145.694 | 1.826 → 1.840 | 57.344 → 57.203 |
| wide | 8.903 → 7.201 | 0.689 → 0.739 | 8.000 → 7.953 |
| tiny | 10.303 → 8.386 | 2.122 → 1.094 | 5.797 → 5.859 |
| tiny4 | 10.018 → 8.412 | 0.368 → 0.329 | 5.938 → 5.922 |
| tiny16 | 4.276 → 4.076 | 0.363 → 0.370 | 6.109 → 6.047 |
| tiny-no-progress | 3.715 → 3.717 | 0.141 → 0.155 | 5.734 → 5.750 |
| tiny4-no-progress | 4.015 → 3.600 | 0.235 → 0.244 | 5.922 → 5.938 |
| tiny16-no-progress | 3.678 → 3.846 | 0.228 → 0.235 | 6.062 → 6.078 |

## Retained small Core controls: fresh three-version comparisons

These separate campaigns compare freshly built exact registry 0.4.16 and 0.4.17 with the candidate, using the same programs. Seven rotating triples per campaign run both the original six cases and a 96-case extension. Warm-up, full original-JSON comparison and index setup occur before timers. Each campaign has **42 processes and 2,142,000 timed Core calls**. Both together have 84 processes, 4,284,000 calls and **2,688 complete original matrix-response comparisons**, with identical results. Signature checks also cover every original-six and timed matrix sample. This is repeated warm Core time, without MCP transport or DHAT instrumentation.

Keep the two campaigns separate. Original six-case calls build SearchOptions inside the measured loop; the extension reuses fixed options. Do not compare their absolute times or merge distributions. The 96 cases combine four entry counts, both rankers and twelve fixed query/filter/limit cases. Wide query preparation, scope rejection, unknown-only queries and zero limits are retained even though they are outside the optimized single-list work. Historical absolute values with different programs/loads are not a reference substitute.

### First small Core campaign

#### Original six warm cases

| Fixture | 0.4.16 median ns | 0.4.17 median ns | 0.4.18 median ns | vs 0.4.17 | vs 0.4.16 |
| --- | --- | --- | --- | --- | --- |
| tiny16_bm25 | 1170.250 | 1008.791 | 1032.084 | +2.31% | -11.81% |
| tiny16_tfidf | 1220.875 | 1074.167 | 1054.458 | -1.83% | -13.63% |
| tiny4_bm25 | 385.500 | 349.792 | 352.625 | +0.81% | -8.53% |
| tiny4_tfidf | 404.375 | 366.334 | 371.917 | +1.52% | -8.03% |
| tiny_bm25 | 213.208 | 192.292 | 193.250 | +0.50% | -9.36% |
| tiny_tfidf | 213.750 | 184.375 | 186.750 | +1.29% | -12.63% |

Higher candidate medians versus 0.4.17: **5 of 6**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](measurements/0.4.18/small-reference-summary.json). No threshold-based rejection or removal of slow timing samples is used.

#### All 96 extended cases

| Fixture | 0.4.16 median ns | 0.4.17 median ns | 0.4.18 median ns | vs 0.4.17 | vs 0.4.16 |
| --- | --- | --- | --- | --- | --- |
| 16_Bm25_disjoint | 644.750 | 474.625 | 491.875 | +3.63% | -23.71% |
| 16_Bm25_limit_one | 812.834 | 624.667 | 649.375 | +3.96% | -20.11% |
| 16_Bm25_limit_zero | 2.708 | 2.750 | 2.708 | -1.53% | +0.00% |
| 16_Bm25_multi | 2075.208 | 2137.291 | 1620.333 | -24.19% | -21.92% |
| 16_Bm25_punctuation | 146.292 | 145.792 | 146.042 | +0.17% | -0.17% |
| 16_Bm25_repeated | 1385.042 | 1215.542 | 1180.917 | -2.85% | -14.74% |
| 16_Bm25_scope | 1168.709 | 981.750 | 1000.583 | +1.92% | -14.39% |
| 16_Bm25_single | 1325.417 | 1139.917 | 1124.500 | -1.35% | -15.16% |
| 16_Bm25_unicode | 1506.167 | 1326.125 | 1334.167 | +0.61% | -11.42% |
| 16_Bm25_unknown | 67.625 | 68.917 | 68.500 | -0.61% | +1.29% |
| 16_Bm25_unknown_added | 1338.542 | 1161.750 | 1168.916 | +0.62% | -12.67% |
| 16_Bm25_wide | 19150.834 | 18641.209 | 19240.583 | +3.22% | +0.47% |
| 16_Tfidf_disjoint | 642.667 | 484.708 | 481.917 | -0.58% | -25.01% |
| 16_Tfidf_limit_one | 903.292 | 714.000 | 701.792 | -1.71% | -22.31% |
| 16_Tfidf_limit_zero | 2.791 | 2.667 | 2.667 | +0.00% | -4.44% |
| 16_Tfidf_multi | 2188.583 | 2180.083 | 1670.375 | -23.38% | -23.68% |
| 16_Tfidf_punctuation | 146.083 | 147.584 | 146.625 | -0.65% | +0.37% |
| 16_Tfidf_repeated | 1428.542 | 1235.000 | 1253.542 | +1.50% | -12.25% |
| 16_Tfidf_scope | 1229.375 | 1064.417 | 1051.792 | -1.19% | -14.44% |
| 16_Tfidf_single | 1382.875 | 1182.083 | 1200.709 | +1.58% | -13.17% |
| 16_Tfidf_unicode | 1593.708 | 1423.625 | 1402.458 | -1.49% | -12.00% |
| 16_Tfidf_unknown | 68.166 | 68.166 | 68.166 | +0.00% | +0.00% |
| 16_Tfidf_unknown_added | 1414.292 | 1216.875 | 1232.958 | +1.32% | -12.82% |
| 16_Tfidf_wide | 19227.208 | 19390.083 | 19026.875 | -1.87% | -1.04% |
| 1_Bm25_disjoint | 137.458 | 108.000 | 110.917 | +2.70% | -19.31% |
| 1_Bm25_limit_one | 212.833 | 174.333 | 180.792 | +3.70% | -15.05% |
| 1_Bm25_limit_zero | 3.584 | 2.708 | 2.666 | -1.55% | -25.61% |
| 1_Bm25_multi | 304.042 | 290.208 | 261.625 | -9.85% | -13.95% |
| 1_Bm25_punctuation | 141.959 | 145.250 | 145.167 | -0.06% | +2.26% |
| 1_Bm25_repeated | 273.917 | 246.292 | 238.459 | -3.18% | -12.94% |
| 1_Bm25_scope | 221.417 | 187.375 | 186.750 | -0.33% | -15.66% |
| 1_Bm25_single | 238.833 | 244.041 | 213.417 | -12.55% | -10.64% |
| 1_Bm25_unicode | 463.750 | 428.916 | 424.000 | -1.15% | -8.57% |
| 1_Bm25_unknown | 69.458 | 66.375 | 69.041 | +4.02% | -0.60% |
| 1_Bm25_unknown_added | 251.042 | 227.417 | 223.917 | -1.54% | -10.80% |
| 1_Bm25_wide | 4579.792 | 4366.625 | 4364.500 | -0.05% | -4.70% |
| 1_Tfidf_disjoint | 132.042 | 109.375 | 111.709 | +2.13% | -15.40% |
| 1_Tfidf_limit_one | 212.250 | 184.625 | 191.000 | +3.45% | -10.01% |
| 1_Tfidf_limit_zero | 2.667 | 2.708 | 2.708 | +0.00% | +1.54% |
| 1_Tfidf_multi | 293.500 | 287.084 | 262.042 | -8.72% | -10.72% |
| 1_Tfidf_punctuation | 141.917 | 145.083 | 143.041 | -1.41% | +0.79% |
| 1_Tfidf_repeated | 270.459 | 249.042 | 242.708 | -2.54% | -10.26% |
| 1_Tfidf_scope | 216.375 | 189.584 | 197.583 | +4.22% | -8.68% |
| 1_Tfidf_single | 207.750 | 186.792 | 188.834 | +1.09% | -9.11% |
| 1_Tfidf_unicode | 455.000 | 431.625 | 426.125 | -1.27% | -6.35% |
| 1_Tfidf_unknown | 68.417 | 66.709 | 68.000 | +1.94% | -0.61% |
| 1_Tfidf_unknown_added | 248.042 | 225.042 | 223.333 | -0.76% | -9.96% |
| 1_Tfidf_wide | 4406.875 | 4355.375 | 4345.333 | -0.23% | -1.40% |
| 33_Bm25_disjoint | 1297.458 | 958.709 | 937.792 | -2.18% | -27.72% |
| 33_Bm25_limit_one | 1576.167 | 1209.458 | 1170.250 | -3.24% | -25.75% |
| 33_Bm25_limit_zero | 2.750 | 3.750 | 2.667 | -28.88% | -3.02% |
| 33_Bm25_multi | 3385.667 | 3585.375 | 2353.542 | -34.36% | -30.49% |
| 33_Bm25_punctuation | 146.500 | 147.791 | 146.458 | -0.90% | -0.03% |
| 33_Bm25_repeated | 2056.083 | 1818.875 | 1728.083 | -4.99% | -15.95% |
| 33_Bm25_scope | 2121.334 | 1751.459 | 1707.250 | -2.52% | -19.52% |
| 33_Bm25_single | 2040.583 | 1709.625 | 1668.917 | -2.38% | -18.21% |
| 33_Bm25_unicode | 2253.584 | 1896.584 | 1831.042 | -3.46% | -18.75% |
| 33_Bm25_unknown | 71.208 | 69.417 | 69.333 | -0.12% | -2.63% |
| 33_Bm25_unknown_added | 2052.417 | 1812.167 | 1717.375 | -5.23% | -16.32% |
| 33_Bm25_wide | 29213.333 | 28491.167 | 28370.541 | -0.42% | -2.88% |
| 33_Tfidf_disjoint | 1273.792 | 957.875 | 974.541 | +1.74% | -23.49% |
| 33_Tfidf_limit_one | 1685.000 | 1350.792 | 1317.083 | -2.50% | -21.83% |
| 33_Tfidf_limit_zero | 2.791 | 2.667 | 2.791 | +4.65% | +0.00% |
| 33_Tfidf_multi | 3548.500 | 3668.708 | 2563.416 | -30.13% | -27.76% |
| 33_Tfidf_punctuation | 142.709 | 143.958 | 143.542 | -0.29% | +0.58% |
| 33_Tfidf_repeated | 2205.292 | 1914.416 | 1880.958 | -1.75% | -14.71% |
| 33_Tfidf_scope | 2135.834 | 1837.791 | 1832.583 | -0.28% | -14.20% |
| 33_Tfidf_single | 2147.334 | 1868.208 | 1814.208 | -2.89% | -15.51% |
| 33_Tfidf_unicode | 2310.000 | 2028.084 | 1984.417 | -2.15% | -14.09% |
| 33_Tfidf_unknown | 69.041 | 69.959 | 69.334 | -0.89% | +0.42% |
| 33_Tfidf_unknown_added | 2218.916 | 1915.500 | 1842.959 | -3.79% | -16.94% |
| 33_Tfidf_wide | 29241.833 | 28719.459 | 28897.125 | +0.62% | -1.18% |
| 4_Bm25_disjoint | 205.667 | 157.667 | 157.917 | +0.16% | -23.22% |
| 4_Bm25_limit_one | 288.750 | 239.500 | 237.417 | -0.87% | -17.78% |
| 4_Bm25_limit_zero | 2.667 | 2.708 | 2.667 | -1.51% | +0.00% |
| 4_Bm25_multi | 652.292 | 656.500 | 542.625 | -17.35% | -16.81% |
| 4_Bm25_punctuation | 144.875 | 150.666 | 143.333 | -4.87% | -1.06% |
| 4_Bm25_repeated | 478.041 | 416.167 | 415.291 | -0.21% | -13.13% |
| 4_Bm25_scope | 332.333 | 298.958 | 285.917 | -4.36% | -13.97% |
| 4_Bm25_single | 418.334 | 365.542 | 367.916 | +0.65% | -12.05% |
| 4_Bm25_unicode | 650.542 | 614.041 | 590.084 | -3.90% | -9.29% |
| 4_Bm25_unknown | 66.666 | 68.625 | 68.375 | -0.36% | +2.56% |
| 4_Bm25_unknown_added | 449.875 | 397.917 | 399.042 | +0.28% | -11.30% |
| 4_Bm25_wide | 7933.125 | 7914.708 | 8113.625 | +2.51% | +2.28% |
| 4_Tfidf_disjoint | 201.667 | 157.041 | 164.333 | +4.64% | -18.51% |
| 4_Tfidf_limit_one | 311.791 | 267.583 | 262.417 | -1.93% | -15.84% |
| 4_Tfidf_limit_zero | 2.667 | 2.792 | 2.708 | -3.01% | +1.54% |
| 4_Tfidf_multi | 685.500 | 688.291 | 579.708 | -15.78% | -15.43% |
| 4_Tfidf_punctuation | 146.042 | 147.458 | 145.375 | -1.41% | -0.46% |
| 4_Tfidf_repeated | 496.709 | 448.041 | 446.167 | -0.42% | -10.18% |
| 4_Tfidf_scope | 340.667 | 294.875 | 293.708 | -0.40% | -13.78% |
| 4_Tfidf_single | 438.833 | 391.291 | 387.250 | -1.03% | -11.75% |
| 4_Tfidf_unicode | 678.792 | 636.375 | 631.000 | -0.84% | -7.04% |
| 4_Tfidf_unknown | 67.750 | 67.875 | 69.459 | +2.33% | +2.52% |
| 4_Tfidf_unknown_added | 473.917 | 429.875 | 430.041 | +0.04% | -9.26% |
| 4_Tfidf_wide | 7852.791 | 7900.291 | 8095.500 | +2.47% | +3.09% |

Higher candidate medians versus 0.4.17: **29 of 96**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](measurements/0.4.18/small-reference-summary.json). No threshold-based rejection or removal of slow timing samples is used.

### Second preplanned small Core campaign

#### Original six warm cases

| Fixture | 0.4.16 median ns | 0.4.17 median ns | 0.4.18 median ns | vs 0.4.17 | vs 0.4.16 |
| --- | --- | --- | --- | --- | --- |
| tiny16_bm25 | 1193.250 | 1153.792 | 1018.334 | -11.74% | -14.66% |
| tiny16_tfidf | 1306.792 | 1274.542 | 1088.333 | -14.61% | -16.72% |
| tiny4_bm25 | 392.500 | 356.916 | 360.625 | +1.04% | -8.12% |
| tiny4_tfidf | 414.417 | 376.375 | 373.708 | -0.71% | -9.82% |
| tiny_bm25 | 225.167 | 196.708 | 192.292 | -2.24% | -14.60% |
| tiny_tfidf | 236.167 | 189.375 | 187.833 | -0.81% | -20.47% |

Higher candidate medians versus 0.4.17: **1 of 6**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](measurements/0.4.18/small-confirmation-summary.json). No threshold-based rejection or removal of slow timing samples is used.

#### All 96 extended cases

| Fixture | 0.4.16 median ns | 0.4.17 median ns | 0.4.18 median ns | vs 0.4.17 | vs 0.4.16 |
| --- | --- | --- | --- | --- | --- |
| 16_Bm25_disjoint | 673.750 | 510.416 | 491.916 | -3.62% | -26.99% |
| 16_Bm25_limit_one | 837.333 | 663.083 | 639.292 | -3.59% | -23.65% |
| 16_Bm25_limit_zero | 3.000 | 3.833 | 3.833 | +0.00% | +27.77% |
| 16_Bm25_multi | 2145.000 | 2244.625 | 1646.125 | -26.66% | -23.26% |
| 16_Bm25_punctuation | 150.458 | 152.250 | 146.500 | -3.78% | -2.63% |
| 16_Bm25_repeated | 1404.625 | 1277.916 | 1205.125 | -5.70% | -14.20% |
| 16_Bm25_scope | 1222.541 | 1075.875 | 1004.667 | -6.62% | -17.82% |
| 16_Bm25_single | 1372.709 | 1248.333 | 1229.750 | -1.49% | -10.41% |
| 16_Bm25_unicode | 1579.791 | 1420.291 | 1341.708 | -5.53% | -15.07% |
| 16_Bm25_unknown | 72.167 | 72.292 | 72.541 | +0.34% | +0.52% |
| 16_Bm25_unknown_added | 1375.167 | 1240.792 | 1173.542 | -5.42% | -14.66% |
| 16_Bm25_wide | 19438.709 | 20009.083 | 19125.500 | -4.42% | -1.61% |
| 16_Tfidf_disjoint | 662.167 | 499.292 | 501.334 | +0.41% | -24.29% |
| 16_Tfidf_limit_one | 952.042 | 735.250 | 741.416 | +0.84% | -22.12% |
| 16_Tfidf_limit_zero | 4.083 | 2.917 | 2.959 | +1.44% | -27.53% |
| 16_Tfidf_multi | 2200.667 | 2268.375 | 1684.041 | -25.76% | -23.48% |
| 16_Tfidf_punctuation | 147.875 | 151.208 | 151.917 | +0.47% | +2.73% |
| 16_Tfidf_repeated | 1458.792 | 1272.708 | 1314.208 | +3.26% | -9.91% |
| 16_Tfidf_scope | 1286.000 | 1087.292 | 1097.542 | +0.94% | -14.65% |
| 16_Tfidf_single | 1444.958 | 1222.292 | 1249.333 | +2.21% | -13.54% |
| 16_Tfidf_unicode | 1635.500 | 1440.458 | 1452.666 | +0.85% | -11.18% |
| 16_Tfidf_unknown | 72.167 | 70.542 | 71.292 | +1.06% | -1.21% |
| 16_Tfidf_unknown_added | 1437.208 | 1246.125 | 1261.916 | +1.27% | -12.20% |
| 16_Tfidf_wide | 20492.250 | 19481.042 | 19516.208 | +0.18% | -4.76% |
| 1_Bm25_disjoint | 141.667 | 109.500 | 112.167 | +2.44% | -20.82% |
| 1_Bm25_limit_one | 230.375 | 181.750 | 186.292 | +2.50% | -19.14% |
| 1_Bm25_limit_zero | 3.000 | 2.709 | 2.791 | +3.03% | -6.97% |
| 1_Bm25_multi | 310.291 | 293.958 | 266.875 | -9.21% | -13.99% |
| 1_Bm25_punctuation | 147.750 | 149.209 | 151.125 | +1.28% | +2.28% |
| 1_Bm25_repeated | 299.542 | 284.250 | 246.500 | -13.28% | -17.71% |
| 1_Bm25_scope | 242.625 | 190.500 | 187.000 | -1.84% | -22.93% |
| 1_Bm25_single | 288.833 | 263.708 | 230.667 | -12.53% | -20.14% |
| 1_Bm25_unicode | 469.542 | 446.625 | 441.542 | -1.14% | -5.96% |
| 1_Bm25_unknown | 73.000 | 71.333 | 72.042 | +0.99% | -1.31% |
| 1_Bm25_unknown_added | 265.375 | 226.666 | 223.209 | -1.53% | -15.89% |
| 1_Bm25_wide | 4828.083 | 4397.292 | 4433.916 | +0.83% | -8.16% |
| 1_Tfidf_disjoint | 133.250 | 117.875 | 114.041 | -3.25% | -14.42% |
| 1_Tfidf_limit_one | 227.958 | 197.750 | 204.792 | +3.56% | -10.16% |
| 1_Tfidf_limit_zero | 2.750 | 2.750 | 2.792 | +1.53% | +1.53% |
| 1_Tfidf_multi | 310.958 | 292.875 | 277.583 | -5.22% | -10.73% |
| 1_Tfidf_punctuation | 147.750 | 152.875 | 152.000 | -0.57% | +2.88% |
| 1_Tfidf_repeated | 286.792 | 253.625 | 246.917 | -2.64% | -13.90% |
| 1_Tfidf_scope | 224.417 | 200.792 | 202.750 | +0.98% | -9.65% |
| 1_Tfidf_single | 224.875 | 191.042 | 191.417 | +0.20% | -14.88% |
| 1_Tfidf_unicode | 470.708 | 450.750 | 456.500 | +1.28% | -3.02% |
| 1_Tfidf_unknown | 70.459 | 71.458 | 75.500 | +5.66% | +7.15% |
| 1_Tfidf_unknown_added | 269.667 | 228.792 | 224.542 | -1.86% | -16.73% |
| 1_Tfidf_wide | 4544.500 | 4527.458 | 4621.625 | +2.08% | +1.70% |
| 33_Bm25_disjoint | 1342.875 | 991.750 | 996.709 | +0.50% | -25.78% |
| 33_Bm25_limit_one | 1649.000 | 1263.583 | 1242.375 | -1.68% | -24.66% |
| 33_Bm25_limit_zero | 3.042 | 3.167 | 2.709 | -14.46% | -10.95% |
| 33_Bm25_multi | 3458.250 | 3598.958 | 2519.042 | -30.01% | -27.16% |
| 33_Bm25_punctuation | 150.458 | 148.416 | 146.125 | -1.54% | -2.88% |
| 33_Bm25_repeated | 2195.333 | 1798.458 | 1783.750 | -0.82% | -18.75% |
| 33_Bm25_scope | 2127.708 | 1786.000 | 1843.542 | +3.22% | -13.36% |
| 33_Bm25_single | 2143.084 | 1753.042 | 1727.708 | -1.45% | -19.38% |
| 33_Bm25_unicode | 2255.208 | 1904.583 | 1916.708 | +0.64% | -15.01% |
| 33_Bm25_unknown | 73.250 | 71.125 | 70.584 | -0.76% | -3.64% |
| 33_Bm25_unknown_added | 2177.584 | 1789.541 | 1813.125 | +1.32% | -16.74% |
| 33_Bm25_wide | 31343.000 | 29385.042 | 29308.667 | -0.26% | -6.49% |
| 33_Tfidf_disjoint | 1363.667 | 962.667 | 947.542 | -1.57% | -30.52% |
| 33_Tfidf_limit_one | 1739.875 | 1395.625 | 1342.625 | -3.80% | -22.83% |
| 33_Tfidf_limit_zero | 2.875 | 2.791 | 2.750 | -1.47% | -4.35% |
| 33_Tfidf_multi | 3823.833 | 3738.875 | 2647.208 | -29.20% | -30.77% |
| 33_Tfidf_punctuation | 151.208 | 148.750 | 148.417 | -0.22% | -1.85% |
| 33_Tfidf_repeated | 2370.083 | 1920.041 | 1903.500 | -0.86% | -19.69% |
| 33_Tfidf_scope | 2354.041 | 1860.750 | 1866.041 | +0.28% | -20.73% |
| 33_Tfidf_single | 2354.583 | 1867.125 | 1950.084 | +4.44% | -17.18% |
| 33_Tfidf_unicode | 2643.583 | 2136.291 | 2028.083 | -5.07% | -23.28% |
| 33_Tfidf_unknown | 72.916 | 70.458 | 71.042 | +0.83% | -2.57% |
| 33_Tfidf_unknown_added | 2358.708 | 1925.458 | 1900.459 | -1.30% | -19.43% |
| 33_Tfidf_wide | 30669.875 | 29871.791 | 29210.167 | -2.21% | -4.76% |
| 4_Bm25_disjoint | 204.750 | 166.375 | 167.167 | +0.48% | -18.36% |
| 4_Bm25_limit_one | 301.416 | 248.584 | 248.042 | -0.22% | -17.71% |
| 4_Bm25_limit_zero | 2.750 | 2.958 | 2.708 | -8.45% | -1.53% |
| 4_Bm25_multi | 714.209 | 687.250 | 560.292 | -18.47% | -21.55% |
| 4_Bm25_punctuation | 148.250 | 150.958 | 145.583 | -3.56% | -1.80% |
| 4_Bm25_repeated | 488.917 | 433.208 | 435.208 | +0.46% | -10.99% |
| 4_Bm25_scope | 336.709 | 294.750 | 303.167 | +2.86% | -9.96% |
| 4_Bm25_single | 434.417 | 379.750 | 372.542 | -1.90% | -14.24% |
| 4_Bm25_unicode | 675.208 | 619.042 | 641.125 | +3.57% | -5.05% |
| 4_Bm25_unknown | 71.917 | 70.833 | 73.750 | +4.12% | +2.55% |
| 4_Bm25_unknown_added | 462.417 | 413.792 | 412.708 | -0.26% | -10.75% |
| 4_Bm25_wide | 8262.250 | 8028.083 | 8071.250 | +0.54% | -2.31% |
| 4_Tfidf_disjoint | 203.000 | 163.250 | 172.000 | +5.36% | -15.27% |
| 4_Tfidf_limit_one | 336.584 | 269.875 | 286.334 | +6.10% | -14.93% |
| 4_Tfidf_limit_zero | 3.833 | 3.625 | 3.042 | -16.08% | -20.64% |
| 4_Tfidf_multi | 743.916 | 710.958 | 609.375 | -14.29% | -18.09% |
| 4_Tfidf_punctuation | 152.250 | 150.459 | 146.542 | -2.60% | -3.75% |
| 4_Tfidf_repeated | 559.125 | 457.000 | 479.000 | +4.81% | -14.33% |
| 4_Tfidf_scope | 353.000 | 305.125 | 319.500 | +4.71% | -9.49% |
| 4_Tfidf_single | 493.000 | 402.125 | 416.458 | +3.56% | -15.53% |
| 4_Tfidf_unicode | 708.833 | 651.292 | 675.334 | +3.69% | -4.73% |
| 4_Tfidf_unknown | 74.375 | 71.291 | 75.625 | +6.08% | +1.68% |
| 4_Tfidf_unknown_added | 533.708 | 447.125 | 454.584 | +1.67% | -14.83% |
| 4_Tfidf_wide | 8651.625 | 8554.833 | 8413.083 | -1.66% | -2.76% |

Higher candidate medians versus 0.4.17: **46 of 96**. Positive changes are slower and remain reported. p95, all seven individual samples per arm and actual call counts remain in [the complete records](measurements/0.4.18/small-confirmation-summary.json). No threshold-based rejection or removal of slow timing samples is used.

## Direct canonical search with explicit control modes

Fourteen processes (seven alternating pairs) retain one warmed Engine each. Ten rounds rotate plain, deadline and progress within each query/ranker pair: **8,400 timed Retriever calls**, excluding loading/readiness, stdio and profiling. Each progress request includes an observer and the same deadline/cancellation setup. Every signature agrees; **140 complete original response JSON pairs match**, and control modes also match plain output. Full readiness source text stays private. These are direct retrieval boundaries, not MCP round trips or capacity evidence.

| Control | Ranker | 0.4.17 median / p95 ms | 0.4.18 median / p95 ms | Median change |
| --- | --- | --- | --- | --- |
| plain | Bm25 | 2.145 / 4.854 | 1.836 / 4.279 | -14.45% |
| plain | Tfidf | 2.365 / 6.430 | 2.035 / 5.110 | -13.98% |
| deadline | Bm25 | 2.375 / 5.591 | 2.112 / 4.726 | -11.06% |
| deadline | Tfidf | 2.717 / 7.622 | 2.384 / 5.835 | -12.28% |
| progress | Bm25 | 2.380 / 5.522 | 2.127 / 4.759 | -10.64% |
| progress | Tfidf | 2.665 / 6.896 | 2.409 / 5.692 | -9.63% |

| Query | Candidates | Progress BM25 ms: old → new | Progress TF-IDF ms: old → new |
| --- | --- | --- | --- |
| 0 | 12365 | 2.197 → 1.945 | 2.520 → 2.211 |
| 1 | 13864 | 2.630 → 2.239 | 2.812 → 2.451 |
| 2 | 24299 | 4.370 → 3.759 | 4.670 → 4.074 |
| 3 | 596 | 0.333 → 0.302 | 0.319 → 0.303 |
| 4 | 15403 | 2.735 → 2.306 | 2.961 → 2.543 |
| 5 | 24140 | 4.249 → 3.674 | 4.543 → 4.055 |
| 6 | 3358 | 0.814 → 0.720 | 0.827 → 0.748 |
| 7 | 29600 | 4.523 → 4.151 | 5.093 → 4.553 |
| 8 | 11504 | 2.077 → 1.894 | 2.176 → 2.020 |
| 9 | 0 | 0.002 → 0.002 | 0.001 → 0.001 |

## Separate canonical three-version reference

Five rotating triples compare fresh exact 0.4.16/0.4.17/candidate servers on canonical knowledge with requested progress. Fifteen servers validate **1,020 actual schema replies**; **680 complete structured comparisons** against the candidate match by RPC ID except checked status version and version-qualified schema URNs. Schemas, source ranges and inputs agree. This control is separate from the two ordinary campaigns. It checks whether the previous large-query benefit remains visible against the older release; it does not establish the cause of any historical regression.

| Version | Ranker | Initial median / p95 ms | Repeated median / p95 ms | Peak child RSS median MiB |
| --- | --- | --- | --- | --- |
| 0.4.16 | bm25 | 3.411 / 5.593 | 3.138 / 5.419 | 1724.391 |
| 0.4.16 | tfidf | 19.240 / 33.858 | 3.460 / 6.292 | 1724.391 |
| 0.4.17 | bm25 | 3.489 / 6.457 | 3.007 / 5.646 | 1724.281 |
| 0.4.17 | tfidf | 20.723 / 36.164 | 3.303 / 6.624 | 1724.281 |
| 0.4.18 | bm25 | 3.238 / 6.798 | 2.878 / 5.558 | 1724.422 |
| 0.4.18 | tfidf | 21.018 / 38.596 | 3.504 / 9.673 | 1724.422 |

| Query | 0.4.16 repeated TF-IDF ms | 0.4.17 repeated TF-IDF ms | 0.4.18 repeated TF-IDF ms |
| --- | --- | --- | --- |
| 0 | 3.026 | 2.893 | 3.092 |
| 1 | 3.721 | 3.432 | 3.504 |
| 2 | 5.720 | 5.184 | 5.432 |
| 3 | 0.744 | 0.714 | 0.754 |
| 4 | 3.375 | 3.386 | 3.368 |
| 5 | 5.360 | 5.334 | 4.921 |
| 6 | 1.311 | 1.274 | 1.272 |
| 7 | 6.067 | 5.748 | 5.635 |
| 8 | 3.876 | 3.447 | 4.030 |
| 9 | 0.121 | 0.117 | 0.130 |

### Other repeated TF-IDF views in the canonical reference

| Version | Mean ms | Mixed-query median ms | p95 ms | Median twenty-call sum/process ms |
| --- | --- | --- | --- | --- |
| 0.4.16 | 3.448 | 3.460 | 6.292 | 65.458 |
| 0.4.17 | 3.324 | 3.303 | 6.624 | 64.507 |
| 0.4.18 | 3.710 | 3.504 | 9.673 | 64.392 |

These are separate views of all recorded samples. No slower process or request is dropped, and no single timing cause is established.

## Canonical first-pass heap traffic

Two instrumented processes index all 167,942 records before profiling the same ten initial TF-IDF queries, then drop results before net deltas. Direct candidate collection can reduce allocation traffic; the previous large-query posting cursors and norm memo/cache still exist. Heap traffic, net persistent hints, stack state and whole-process RSS are different costs. Instrumented durations are excluded from latency comparisons.

| Metric | 0.4.17 | 0.4.18 |
| --- | --- | --- |
| Retained hint bytes | 2228312 | 2228312 |
| Retained hint blocks | 2 | 2 |
| Total query allocated bytes | 7216291 | 7213987 |
| Total query allocation blocks | 439 | 415 |

## Development pilot and host-load limits

The [development pilot](measurements/0.4.18/development-pilot-summary.json) remains separate from all final samples. Its separate small Core, direct controlled retrieval and MCP triples supported the candidate assessment. Measurements are workload-specific and do not prove the cause of historical large-query variation. All source/lock/binary identities and raw observations remain locally retained.

Two-second monitoring retains **363 snapshots**, including **203 with other rustc processes**, maximum 3. All managed compilers finish before timing. Other workloads are not stopped, and snapshots do not prove continuous idleness or identify a timing cause. CPU counts include the active benchmark workers.

| Stage start | Other rustc count | Other rustc CPU total | Processes over 50% CPU |
| --- | --- | --- | --- |
| small-reference | 1 | 100.000 | 2 |
| retrieval | 0 | 0.000 | 2 |
| query | 1 | 312.700 | 2 |
| build | 1 | 1281.100 | 3 |
| controlled | 0 | 0.000 | 2 |
| mcp | 2 | 388.800 | 3 |
| corpus | 0 | 0.000 | 1 |
| mcp-confirmation | 0 | 0.000 | 2 |
| reference | 1 | 99.500 | 5 |
| small-confirmation | 1 | 98.500 | 3 |

All observed slower cases remain in their own distributions; no sample is replaced because of load. One machine, randomized hashing, exposed questions, tuned fixtures and finite trials limit generalization.

## Retained Core controls

These controls retain positive and negative timing changes. Single known-term requests retain their direct path. Exactly two known terms now use direct ordered merging; larger queries keep the heap/set strategies. Large frequency cursors remain available afterward. Read every row independently, including slower cases. Seven alternating pairs give fourteen plain processes per family. Norms are initialized for warm timing; two separate DHAT processes measure allocations. Instrumented durations are excluded from latency comparisons. Every result signature agrees and temporary-call retained deltas are zero.

### Small warm Core: 84,000 calls on 1/4/16 entries

| Fixture | Median change | Blocks/call: old → new | Allocated bytes/call: old → new |
| --- | --- | --- | --- |
| tiny16_bm25 | -2.75% | 29.0 → 29.0 | 2536.0 → 2536.0 |
| tiny16_tfidf | -2.37% | 29.0 → 29.0 | 2536.0 → 2536.0 |
| tiny4_bm25 | -1.46% | 15.0 → 15.0 | 1204.0 → 1204.0 |
| tiny4_tfidf | -0.91% | 15.0 → 15.0 | 1204.0 → 1204.0 |
| tiny_bm25 | +2.77% | 9.0 → 9.0 | 778.0 → 778.0 |
| tiny_tfidf | -0.85% | 9.0 → 9.0 | 778.0 → 778.0 |

### All 25 warm controls: 282,800 calls

| Fixture | Median change | Blocks/call: old → new | Allocated bytes/call: old → new |
| --- | --- | --- | --- |
| coverage_repeated | -0.11% | 1047.0 → 1047.0 | 92942.0 → 92942.0 |
| coverage_small | -5.32% | 784.0 → 784.0 | 78046.0 → 78046.0 |
| excerpt_ascii | +5.53% | 762.0 → 762.0 | 252438.0 → 252438.0 |
| excerpt_mixed | +5.58% | 569.0 → 569.0 | 417113.0 → 417113.0 |
| excerpt_unicode | +3.10% | 197.0 → 197.0 | 72254.0 → 72254.0 |
| long_repeated | +1.34% | 4.0 → 4.0 | 18798.0 → 18798.0 |
| mixed_unicode | -5.84% | 7.0 → 7.0 | 3155.0 → 3155.0 |
| punctuation | -6.31% | 1.0 → 1.0 | 768.0 → 768.0 |
| relevant_retrieval | -2.50% | 812.0 → 812.0 | 93562.0 → 93562.0 |
| repeated_search | -1.77% | 35.0 → 35.0 | 19432.0 → 19432.0 |
| repeated_short | +3.54% | 4.0 → 4.0 | 1656.0 → 1656.0 |
| sparse_bm25_128 | +10.45% | 359.0 → 359.0 | 29074.0 → 29074.0 |
| sparse_bm25_33 | +7.84% | 241.0 → 241.0 | 16309.0 → 16309.0 |
| sparse_bm25_512 | +8.38% | 1337.0 → 1337.0 | 119124.0 → 119124.0 |
| sparse_tfidf_128 | +8.65% | 474.0 → 474.0 | 35140.0 → 35140.0 |
| sparse_tfidf_33 | +1.33% | 241.0 → 241.0 | 16309.0 → 16309.0 |
| sparse_tfidf_512 | +5.27% | 1337.0 → 1337.0 | 119124.0 → 119124.0 |
| tfidf_union_32 | +4.09% | 244.0 → 244.0 | 19834.0 → 19834.0 |
| tfidf_union_33 | +5.30% | 296.0 → 296.0 | 27854.0 → 27854.0 |
| tfidf_warm | +2.96% | 71.0 → 71.0 | 19335.0 → 19335.0 |
| union_32 | +4.02% | 244.0 → 244.0 | 19834.0 → 19834.0 |
| union_33 | -1.53% | 296.0 → 296.0 | 27854.0 → 27854.0 |
| unique_long | -0.90% | 299.0 → 299.0 | 32672.0 → 32672.0 |
| unique_search | -5.45% | 72.0 → 72.0 | 19350.0 → 19350.0 |
| unique_short | -2.52% | 296.0 → 296.0 | 18548.0 → 18548.0 |

### Cold norm and heap controls

Six fixtures make twenty fresh-index-clone calls each across fourteen processes: 1,680 calls. Clone/setup is outside timing. Process/instruction/filesystem caches remain; cold means norm hints, not disks. Separate heap runs drop results before net retained measurements. The existing inline/optional heap word memo and persistent norm storage remain. Large small-term requests may add up to four temporary posting cursors; canonical allocation traffic below records that cost. Heap profiles exclude stack state/control fields and temporary frames.

| Fixture | Old median / p95 ns | New median / p95 ns |
| --- | --- | --- |
| index | 119854.000 / 138624.650 | 114458.000 / 133304.450 |
| wide | 44687.500 / 74341.350 | 33354.500 / 43177.400 |
| sparse | 501333.500 / 564656.550 | 513563.000 / 603093.100 |
| tiny | 583.000 / 1377.050 | 625.000 / 1603.850 |
| tiny4 | 875.000 / 2008.300 | 875.000 / 1546.150 |
| tiny16 | 2583.000 / 4292.000 | 2542.000 / 4361.050 |

| Cold fixture | Allocated bytes: old → new | Retained hint bytes: old → new |
| --- | --- | --- |
| index | 92356 → 92356 | 34904 → 34904 |
| wide | 19549 → 19165 | 4440 → 4440 |
| sparse | 140704 → 140704 | 8792 → 8792 |
| tiny | 934 → 934 | 156 → 156 |
| tiny4 | 1504 → 1504 | 224 → 224 |
| tiny16 | 3668 → 3668 | 632 → 632 |

Canonical first-pass heap records, all samples, blocks and cold tails remain in public JSON. Retained hints, heap traffic and whole-process RAM are different measures.

## Finite mixed control, documents and construction

The 6,000-entry mixed control retains four continuous warm readers while 100 initially uncached-group driver queries finish, then makes two quiescent probes. Five alternating pairs give 1,000 driver calls and twenty quiescent probes. Warm counts vary with the stop condition and new-group calls can become warm. This finite control is not fixed-load throughput, capacity or fairness evidence.

| Mixed work, ns | Old median / p95 | New median / p95 |
| --- | --- | --- |
| new_group_query_ns | 209646.000 / 331516.700 | 224041.500 / 303077.100 |
| post_readers_first_ns | 229792.000 / 248566.800 | 242875.000 / 252308.400 |
| post_readers_second_ns | 119500.000 / 127399.600 | 117583.000 / 124658.000 |

The private document corpus retains 72 Markdown files, 2,755,468 bytes and 1,910 chunks. All 342 authored questions (276 positive, 66 unsupported) are exposed regressions, not fresh holdouts or logged production queries. Literal BM25, prepared BM25 and coverage retain six passages, 100 candidates, 2,000 bytes/pass and 12,000 total bytes. All **1,026 complete original response JSON strings match byte-for-byte**, including scores, ordering, evidence, source text and positions. One warm-up and five repeats/question/configuration/version give **10,260 timed round trips**. These three configurations are BM25 controls; independent score oracles and actual protocol pairs cover TF-IDF. No new relevance judgment or model reply is produced.

All document timing aggregates and per-question results remain in the records. Construction uses forty plain processes on canonical data and three synthetic distributions, plus six separate profiles. Parsing/generation is outside construction timing; stats/signatures/live derived heap agree. The execution/shared-index worker checks all 167,942 entries through 128 changes/restoration, rebuilds, JSON round trips, retained clones, parallel readers and cancellation/deadline rejection. Portable exports and source hashes agree.

### Document round trips

| Case | Old median / p95 ms | New median / p95 ms |
| --- | --- | --- |
| baseline | 0.817 / 1.254 | 0.821 / 1.284 |
| prepared | 0.866 / 1.348 | 0.882 / 1.351 |
| evidence | 0.861 / 1.338 | 0.864 / 1.351 |

### Construction only

| Case | Old median / p95 ms | New median / p95 ms |
| --- | --- | --- |
| canonical | 5479.839 / 7009.766 | 5458.107 / 6042.230 |
| ascii | 15.673 / 19.397 | 15.035 / 19.982 |
| mixed | 35.790 / 41.864 | 35.506 / 41.702 |
| long_tokens | 1.686 / 1.769 | 1.713 / 2.099 |

## Initial-query details and pre-release follow-up

Mixed-query medians combine different costs. Repeated-query views below retain each warm query and the corresponding mean and per-process twenty-call sum as well. The following tables also retain each initial TF-IDF query's median, the overall arithmetic mean and the median of each process's sum over all ten initial TF-IDF calls. These are computed from the original protocol samples without rerunning, replacing or regrouping samples between campaigns. Startup and source follow-ups remain outside search timing. Initial means partly cold/partly warm hints, not cold disks or ten isolated fresh-cache queries.

### First selected-source campaign

| Query | 0.4.17 initial TF-IDF median ms | 0.4.18 initial TF-IDF median ms | Change |
| --- | --- | --- | --- |
| 0 | 32.283 | 23.525 | -27.13% |
| 1 | 41.279 | 23.051 | -44.16% |
| 2 | 50.215 | 34.377 | -31.54% |
| 3 | 2.084 | 1.564 | -24.93% |
| 4 | 24.857 | 15.386 | -38.10% |
| 5 | 39.891 | 27.814 | -30.27% |
| 6 | 7.276 | 4.998 | -31.31% |
| 7 | 37.211 | 27.048 | -27.31% |
| 8 | 4.275 | 2.985 | -30.18% |
| 9 | 0.256 | 0.113 | -56.08% |

| Other initial TF-IDF view | 0.4.17 ms | 0.4.18 ms | Change |
| --- | --- | --- | --- |
| Arithmetic mean | 23.005 | 16.584 | -27.91% |
| Median ten-call sum per process | 249.039 | 164.807 | -33.82% |

### Second selected-source campaign

| Query | 0.4.17 initial TF-IDF median ms | 0.4.18 initial TF-IDF median ms | Change |
| --- | --- | --- | --- |
| 0 | 24.534 | 21.301 | -13.18% |
| 1 | 26.025 | 22.309 | -14.28% |
| 2 | 35.782 | 34.505 | -3.57% |
| 3 | 1.784 | 1.629 | -8.68% |
| 4 | 16.671 | 15.620 | -6.31% |
| 5 | 27.114 | 25.746 | -5.04% |
| 6 | 5.027 | 4.738 | -5.75% |
| 7 | 28.137 | 27.605 | -1.89% |
| 8 | 3.074 | 2.913 | -5.23% |
| 9 | 0.149 | 0.142 | -4.48% |

| Other initial TF-IDF view | 0.4.17 ms | 0.4.18 ms | Change |
| --- | --- | --- | --- |
| Arithmetic mean | 17.640 | 17.365 | -1.56% |
| Median ten-call sum per process | 168.314 | 154.701 | -8.09% |

### Repeated query details: First selected-source campaign

| Query | 0.4.17 repeated TF-IDF median ms | 0.4.18 repeated TF-IDF median ms | Change |
| --- | --- | --- | --- |
| 0 | 4.244 | 2.835 | -33.19% |
| 1 | 4.794 | 3.267 | -31.85% |
| 2 | 6.825 | 5.040 | -26.16% |
| 3 | 0.862 | 0.692 | -19.80% |
| 4 | 4.730 | 3.142 | -33.59% |
| 5 | 7.034 | 5.104 | -27.44% |
| 6 | 1.665 | 1.273 | -23.56% |
| 7 | 7.601 | 5.709 | -24.89% |
| 8 | 4.729 | 3.526 | -25.44% |
| 9 | 0.153 | 0.111 | -27.37% |

| Other repeated TF-IDF view | 0.4.17 ms | 0.4.18 ms | Change |
| --- | --- | --- | --- |
| Arithmetic mean | 4.839 | 3.163 | -34.62% |
| Median twenty-call sum per process | 85.229 | 63.014 | -26.06% |

### Repeated query details: Second selected-source campaign

| Query | 0.4.17 repeated TF-IDF median ms | 0.4.18 repeated TF-IDF median ms | Change |
| --- | --- | --- | --- |
| 0 | 3.206 | 2.720 | -15.17% |
| 1 | 3.763 | 3.081 | -18.13% |
| 2 | 6.015 | 4.802 | -20.16% |
| 3 | 0.810 | 0.663 | -18.18% |
| 4 | 3.951 | 3.025 | -23.44% |
| 5 | 5.836 | 4.537 | -22.25% |
| 6 | 1.480 | 1.159 | -21.70% |
| 7 | 6.719 | 5.203 | -22.57% |
| 8 | 4.290 | 3.756 | -12.45% |
| 9 | 0.141 | 0.112 | -20.33% |

| Other repeated TF-IDF view | 0.4.17 ms | 0.4.18 ms | Change |
| --- | --- | --- | --- |
| Arithmetic mean | 4.029 | 3.497 | -13.20% |
| Median twenty-call sum per process | 76.603 | 55.577 | -27.45% |

## Preserved initial candidate and compiler-isolation pilot

The first unpublished 0.4.18 candidate used the same direct pair algorithm without an inlining boundary. Its full planned validation and benchmark sequence completed with exact responses, but canonical initial TF-IDF medians were higher by **+2.44% and +11.27%** versus 0.4.17. Several per-query cold observations were also slower. It was not published. Its entire raw dataset, source, programs, locks, failed setup attempt and slower outcomes remain retained; [sanitized initial records](measurements/0.4.18/initial/README.md) stay separate from selected-source results.

The selected source adds only `#[inline(never)]` to the private pair helper. This keeps its iterator/merge work outside the caller's generated body. It adds no algorithm, checkpoint, data, option or response change. A separate five-triple codegen-isolation pilot compares registry 0.4.17, the initial 0.4.18 binary and the isolated helper build with identical programs/data. All complete structured replies and small signatures match. The pilot showed a better combined cold/warm tradeoff for isolation; it does not establish the cause of every prior variation or guarantee future compiler behavior.

| Pilot view | Registry 0.4.17 | Initial candidate S1 | Isolated candidate S2 |
| --- | --- | --- | --- |
| Initial MCP TF-IDF mixed-query median ms | 19.175 | 21.158 | 18.656 |
| Initial MCP TF-IDF arithmetic mean ms | 16.009 | 20.207 | 15.956 |
| Initial MCP ten-call sum/process median ms | 157.477 | 174.321 | 153.963 |
| Repeated MCP TF-IDF median ms | 3.427 | 3.860 | 3.109 |

After selection, the entire native/feature/MSRV/API/consumer/backend/Miri suite and every planned benchmark stage were rerun from frozen selected-source inputs and binaries. Final campaign samples replace none of the initial candidate samples: they form separate distributions. The initial source is reconstructible by removing the single attribute above; its source hash and original-path mapping remain in [preservation metadata](measurements/0.4.18/initial/preservation-record.json). Pilot, initial-candidate and selected-source trials occurred at different times with recorded but uncontrolled external load. Their absolute times are not interchangeable, and no result establishes statistical significance or a unique timing cause.

## Separate exploratory wide-MCP follow-up

After all planned selected-source stages completed, the slower synthetic wide fixture was investigated with ten additional alternating 0.4.17/0.4.18 pairs. Source, measured binaries, inputs, schemas and existing probe stayed unchanged. The additional driver was frozen separately before this diagnostic. Twenty servers validate 1,360 actual schema replies and 680 complete response pairs agree. This is an exploratory follow-up, not a preplanned replacement for an unfavorable campaign.

| Wide follow-up | Ranker | 0.4.17 median / p95 ms | 0.4.18 median / p95 ms | Median change |
| --- | --- | --- | --- | --- |
| initial | bm25 | 0.557 / 0.675 | 0.571 / 0.696 | +2.38% |
| initial | tfidf | 0.487 / 0.871 | 0.505 / 0.915 | +3.77% |
| repeated | bm25 | 0.439 / 0.781 | 0.449 / 0.842 | +2.26% |
| repeated | tfidf | 0.436 / 0.830 | 0.451 / 0.846 | +3.58% |

The second ordinary campaign retains its +67.35% wide repeated TF-IDF median. This follow-up instead observes about +3.58%, still slower. Neither distribution is removed or merged, and these observations do not establish a unique cause or latency guarantee. Read the separate plan, source hash, raw reports, per-query summary and sampled host load in the measurement directory.

## Limits and preservation

Measurements use one Apple M5 Max on battery power at campaign start with 64 GB, Rust 1.98.0 and Python 3.15.2. Stage-start CPU/compiler snapshots remain. Own compilers finish before timing; this does not establish continuously idle hardware. Randomized hashing, scheduling, one machine/corpus, exposed questions and finite process counts limit generalization. The separate development pilot remains; assess final per-case medians and tails independently. Host-load sampling records observations rather than continuous idleness. No universal latency, total-memory, semantic-quality, token-saving or security-audit claim is made.

Canonical knowledge remains 167,942 entries from 35 PDFs and 49,086 physical pages, SHA-256 `13168d87872a137355fd8499c90a6084f52243a2ec7456f29b5fcb2a1c7423c3`. The PDF parser is unchanged; this is not a fresh large-PDF extraction campaign. Lexical overlap and exact quotes do not prove factual support or entailment. No model, daemon or DenkMaps integration is added.

Preserve PDFs, knowledge, snapshots, all pilot/final measurements, slower results, raw transcripts, profiles, exact small executables and locks. After verified publication archive package/proof, then delete only the named temporary Cargo/Miri targets. Default features remain empty.

Power metadata correction: the original AC label was hard-coded and incorrect. The preserved environment record reports Battery Power at campaign start. Power was not continuously sampled. Timing samples remain unchanged; see [benchmark errata](BENCHMARK-ERRATA.md).
