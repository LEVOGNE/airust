# AIRust 0.6.0 measurement record

These are complete, finite measurements on one macOS ARM64 host, using the previously exposed assistant-authored reference-v1 corpus. They are regression/resource evidence, not independent customer quality, throughput capacity or a universal speed claim. The numerical [policy](../../evaluation/pilot-0.6.0-measurement-policy.json) was committed before execution. Dataset questions, judgments and original v1 results were not retuned.

## Published 0.5.3 baseline controls

Four identical-source Core builds (baseline/candidate, default/shared storage) have exactly matching ordered IDs, f64 score bits, matched terms and original records across 150 questions plus eight boundary controls, both rankers, three scopes and original/edited/retained/restored states. Two alternating seven-pair campaigns compare warm query p95 and median exact-child process peak RSS on 9304 entries.

| Storage | p95 ratio, campaign 1 / 2 | Median peak-RSS ratio, campaign 1 / 2 |
| --- | ---: | ---: |
| default | 0.9595 / 1.1124 | 0.9975 / 0.9980 |
| shared | 0.9796 / 0.9887 | 1.0034 / 1.0000 |

Ratios are candidate/baseline; no aggregate crossed the 1.10 regression criterion in both campaigns. Individual slower query means remain in [core-summary.json](core-summary.json). Timing differences on a shared host do not imply a new Core algorithm: owned index runtime source is unchanged from 0.5.3.

## Complete lexical MCP comparison

The corrected comparison uses two alternating five-pair campaigns, each worker warming all 150 queries and measuring three repetitions. Full evidence data signatures, including exact parsed numeric fields, match. Pacing keeps request starts at least 5 ms apart outside the timer; this is sequential latency under the normal rate limit, not maximum throughput.

| Campaign | Median worker p95, baseline / candidate ms | p95 ratio | Median peak-RSS ratio |
| --- | ---: | ---: | ---: |
| 1 | 3.284 / 3.123 | 0.9510 | 1.0018 |
| 2 | 3.488 / 3.466 | 0.9939 | 1.0000 |

The initial MCP comparison stopped after an exact-data mismatch: the external baseline lacked `serde_json/float_roundtrip`, which the published/root example selects through its jsonschema dev dependencies. All 276 differences were floating fields within 1e-12; IDs, ranks, text and source data matched. The baseline was rebuilt with matching features and the full comparison repeated. No expected signature, dataset or numerical gate was relaxed. The original failed run, binary, manifests and difference list remain locally retained; [the amendment](mcp-method-amendment.json) records this correction. Feature graphs belong to a reproducible comparison.

## Finite scale and retained views

The stress worker is byte-identical to the checksum-verified published 0.5.3 example. Two campaigns run three alternating pairs per 10k/100k/1m size and storage configuration, plus two pairs retaining three 500k-entry clones per configuration. All 88 children exit successfully, scale score signatures agree and retained-reader assertions pass. There is no reproduced aggregate regression above the predeclared 1.10 limit.

Every build, RSS, selective-query/clone median and ratio is retained in [scale-summary.json](scale-summary.json). Negative observations remain: for example, the second million-entry default campaign had a slower candidate selective p95 and construction; the criterion requires reproduction in both campaigns. Short synthetic records do not establish real-document capacity, sustained operation or an OS memory quota. The reference application separately caps its imported workspace at 100000 passages.

## Actual live profiles

All three profiles use the same immutable 41-document, 1127-passage workspace, the same 150 questions and six-hit/excerpt bounds. Each question has one warm-up and three measured requests: 450 timed requests per profile, 1350 total. Timers run from client request write through complete response receipt and JSON decoding. Query embedding, digest checks, retrieval, optional reranking and final server framing are included. Setup/model loading is separate.

| Profile | Complete warm p50 / p95 ms | Process startup to initialization ms | Worker peak MiB | Max sampled service RSS MiB | Eval known-quote Hit@6 |
| --- | ---: | ---: | ---: | ---: | ---: |
| bm25 | 1.878 / 3.501 | 31.09 | 31.17 | — | 0.7292 |
| hybrid | 28.525 / 30.354 | 98.43 | 93.66 | 1343.27 | 0.8750 |
| hybrid_rerank | 495.332 / 981.886 | 2321.17 | 2594.97 | 1343.52 | 0.9167 |

All predeclared latency/worker/service observation budgets pass, with zero source-span mismatches or loaded-collection leaks. Every timed sample, known-quote loss and per-split result is retained. Six delivered excerpts do not establish nDCG@10. All profiles returned candidates on all authored unsupported cases; that is a diagnostic, not an answer-error rate. The application generates no answers and does not infer answerability from nonempty hits.

The actual BGE-M3 provider uses Ollama 0.40.17, 1024 dimensions and digest `7907646426070047a77226ac3e684fbbe8410524f7b4a74d02837e43f2146bab`. The new cache contains 1127 actual workspace IDs and 1102 unique texts; preparation is separately timed in the local sequence receipt. The native MiniLMv2 reranker uses the five pinned files at revision `1427fd652930e4ba29e8149678df786c240d8825`, ONNX Runtime 1.32.0 and two threads. Models/runtime are supplied explicitly and not downloaded by ordinary checks.

Worker RSS is exact-child `wait4` peak including setup, data, model and worker bookkeeping. Service RSS is separately sampled for the task-owned Ollama server/descendants; it is not a measured continuous peak. Service-reported model `size`/`size_vram` values are retained in the raw observations. Unique GPU/shared physical allocations were not independently available, and process RSS values must not be added as unique physical memory. File/service caches are warm; startup is not a cold-OS-cache claim. The task-owned service was stopped after the campaign.

## Real-data readiness and retention

The three locally captured historical user-question replays remain explicitly unreviewed replays, with zero genuine independently reviewed pilot judgments. Raw replay questions/case files stay local and are excluded from Git/package. At least 50 genuine independently reviewed questions, settled labels, unsupported cases, provenance review and a separately frozen source-family/policy evaluation remain prerequisites for a new production-quality recommendation. This authored regression campaign does not satisfy that gate.

The historical [reference-v1 comparison](../reference-v1/README.md) keeps all nine configurations, including SQLite comparators and losses. The new live campaign adds complete request and separate service-memory observations; it does not establish AIRust superiority over SQLite or another engine.

Raw worker reports, failed trials, stderr, samples, model/cache/lock/source hashes and build inputs remain under ignored `local-lab/pilot-0.6.0/`. Public summaries are versioned beside this file; numeric measurement JSON is deliberately excluded from the registry package. The [artifact manifest](artifact-manifest.json) identifies the retained evidence. Original knowledge/database and the separately maintained PDF backend are preserved.
