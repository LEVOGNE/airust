# Reference-v1 retrieval comparison

This campaign evaluates the frozen local technical-document search application. The dataset and thresholds were committed before profile execution. Selection used 90 development questions; the 60 reserved evaluation questions were run afterward. All nine profiles completed with zero span mismatches and zero collection leaks. The source, policy, compiler, hardware, model identities and individual losses are retained in [summary.json](summary.json).

## Decision

The development rule selected `hybrid_rerank`, and it also qualified on evaluation. This is an optional quality-oriented profile for this reference corpus. It combines exact BGE-M3 cosine retrieval with lexical BM25 using equal-weight RRF (k=60), then rescores the actual first 20 candidates with multilingual MiniLMv2. The complete workspace example and MCP continue to use literal BM25; this evaluation does not silently configure an application or change any library default. Applications opting into models must supply the named vectors/provider and reranker explicitly.

Hybrid retrieval without reranking also meets the absolute quality and resource objectives on both splits. It is the lower-cost model option in these measurements. Reranking improved this reference metric substantially but added roughly 0.8–0.9 seconds at p95 and about 2.4 GiB worker RSS. Those are observed costs, not a general latency promise.

## All observed profiles

Reference nDCG@10 and Hit@6 below apply only to answerable questions. RSS includes the exact worker process, data, model and evaluation bookkeeping; it is not isolated index memory. Warm p95 includes bounded six-hit `RetrievalResponse` JSON serialization. External query embedding and construction are outside that timer.

| Profile | Dev nDCG@10 | Dev Hit@6 | Eval nDCG@10 | Eval Hit@6 | Eval p95 ms | Eval RSS MiB | Eval quality target |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `bm25` | 0.6077 | 0.7222 | 0.6001 | 0.7083 | 0.713 | 16.78 | fail |
| `prepared` | 0.6194 | 0.6944 | 0.6041 | 0.7083 | 0.418 | 16.66 | fail |
| `coverage` | 0.6129 | 0.6944 | 0.6159 | 0.7292 | 0.449 | 16.66 | fail |
| `german` | 0.6076 | 0.7639 | 0.6055 | 0.7292 | 1.160 | 20.47 | fail |
| `sqlite` | 0.6129 | 0.7222 | 0.5981 | 0.7500 | 1.282 | 14.75 | fail |
| `hybrid` | 0.7675 | 0.8472 | 0.7634 | 0.8750 | 2.845 | 26.78 | pass |
| `sqlite_hybrid` | 0.7758 | 0.8472 | 0.7666 | 0.8750 | 2.420 | 20.39 | pass |
| `bm25_rerank` | 0.7262 | 0.7778 | 0.7803 | 0.8333 | 825.972 | 2503.42 | pass |
| `hybrid_rerank` | 0.8711 | 0.9444 | 0.8505 | 0.9167 | 835.605 | 2412.55 | pass |

Every profile met its resource budget. Literal BM25 and the other lexical variants did not meet the frozen quality objective (nDCG ≥0.75 and Hit@6 ≥0.80). Prepared queries and coverage ranking are workload choices; neither qualified for recommendation here. The German analyzer did not solve cross-language retrieval over English source documents. `bm25_rerank` passed the evaluation quality objective but failed it on development and was not eligible for promotion.

SQLite FTS5 used the exact AIRust-tokenized terms, with vocabulary, document frequencies and term frequencies checked against Core. Scope filtering precedes top-K. Native FTS5 ranking retains its own score formula. SQLite hybrid uses the same vectors, scope, candidate limit, cosine definition and RRF settings as Core. It was slightly faster on this small corpus and had similar reference quality; this does not establish an AIRust speed advantage. SQLite is a comparator and was not eligible as a recommended AIRust profile.

## Costs, model identity and timing boundaries

The host was an Apple M5 Max, arm64, 18 logical CPUs and 64 GiB RAM, on AC power. Rust 1.98.0 release workers used the root lockfile. Each question has one untimed quality probe, one warm-up and seven measured six-hit retrievals. The quality probe at ten is separate from the delivered six-hit response; Hit@6 checks that delivered evidence contains the exact known supporting quote. Query order uses the policy seed.

BGE-M3 was served by local Ollama 0.40.17 with raw text and `truncate=false`, 1,024 dimensions and model digest `7907646426070047a77226ac3e684fbbe8410524f7b4a74d02837e43f2146bab`. Document vector preparation took 12.971 seconds; the separately measured single-query embedding p50/p95 was 12.203/12.664 ms. Summing these with retrieval percentiles does not produce a measured end-to-end percentile. The temporary owned model service was stopped after vector creation.

The native reranker is `cross-encoder/mmarco-mMiniLMv2-L12-H384-v1` at revision `1427fd652930e4ba29e8149678df786c240d8825`, using the five verified [asset hashes](../../models/mmarco-minilm-v2.json), local ONNX Runtime 1.32.0 and two intra-op threads. Runtime and executable hashes are in the summary. Model loading is reported in construction, not warm query time. Assets are retained locally and are not committed or included in the crate. No model or runtime is downloaded by normal tests.

The timed worker produces a six-hit RetrievalResponse, not the workspace CLI’s outer citation wrapper, file restoration or process startup. The separate workflow tests exercise actual CLI startup, full wrapper bounds and stdio MCP. These timing boundaries must stay distinct when using the result.

## Regressions and limits

The full per-case rankings, seven latencies and evidence are retained for all 18 profile/split runs in adjacent JSON files. `per_query_regressions_against_bm25` in the summary retains every lower nDCG or Hit@6 result, including two evaluation losses for the selected profile (`mcp-positive-1` and `versioning-positive-4`). No question, positive reference or resource limit was changed after evaluation. The [selection receipt](selection-before-evaluation.json) records the choice before evaluation started.

The 150 questions and known supporting paragraphs are assistant-authored and are not customer traffic or independent exhaustive relevance judgments. The same assistant authored the reserved questions and evaluation code. Another passage can be useful despite being unjudged. German/English subgroup metrics and every negative observation remain in the summary. All profiles returned some candidates for most unsupported private-deployment questions; the model profiles did so for all of them. That is a candidate-return diagnostic, not an answer error rate: this application generates no answers and leaves nonempty results’ answerability unassessed.

One corpus and one shared host do not establish general semantic quality, speed, token savings or production capacity. The independent Python metric formula checks the Rust metric implementation, not the validity or completeness of the relevance judgments.

## Reproduce

Follow the explicit local-asset commands in [STARTUP.md](../../../STARTUP.md#evaluate-search-configurations). The frozen [policy](../../evaluation/reference-v1-policy.json) and [dataset](../../../tests/fixtures/reference-v1/) must validate first. `scripts/run_reference_evaluation.py` writes development selection before opening evaluation and rejects source changes during a campaign. A new campaign needs a new output directory.

[Artifact manifest](artifact-manifest.json) records SHA-256 identities of all raw worker reports and logs. The source campaign and logs remain under ignored `local-lab/reference-workflow/campaign-01`; numeric reports are versioned here but excluded from the registry package. Historical 0.5.2 receipts and older measurement reports remain unchanged.
