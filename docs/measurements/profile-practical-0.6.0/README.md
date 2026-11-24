# Local answer pilot and bounded CPU reranking investigation

Status: baseline local answer attempts and exploratory native measurements are complete; larger native controls, full-MCP confirmation and candidate validation are recorded as they finish. Do not infer a final performance/quality pass from this initial record. Exact evolving state is retained in ignored `local-lab/profile-practical-0.6.0-2026-11-24/PROGRESS.json`.

## Frozen inputs and honest origin

The baseline is published AIRust 0.6.0 source `8c098f15a39f6085f748c56ac52e5737abdff423`, independently rebuilt with `pilot,local-reranking`, release profile and the retained lock. Binary SHA-256: `45abd1eef5cae133762514f7ecdf61c133f22f3ff73d73170348c307f9b665ad`.

Eighteen selected documents from that exact source produce 729 immutable passages. Corpus size is 240718 bytes; workspace file SHA-256 `7e09899cbd87d6d9aed525fb6ea6afee2a26aa42cf6388a759ef634bb342dd75`; vector cache SHA-256 `53dc4648a85b34f7f355f6f73576a8cd9c02cb4b3526c85a6ec1f5b139e8ba2e`. Original question/source/model identities and failed attempts remain. This does not change the historical 41-document reference-v1 corpus or its previous measurements.

The inputs are three verbatim earlier user questions replayed from this conversation and one current user improvement request used as an exploratory task. Short continuation questions lose their original conversational context in this stateless test. These are four exposed local probes, not independent customer traffic or four human-reviewed quality labels. Private prompts/answers/service environment logs remain outside Git/package/public assets. Independently reviewed eligible case count remains zero.

Existing models only: Ollama 0.40.17, BGE-M3 digest `7907646426070047a77226ac3e684fbbe8410524f7b4a74d02837e43f2146bab` / 1024 dimensions; pinned multilingual MiniLMv2 and ONNX Runtime 1.32.0 with the original max512 pair recipe; answer model Qwen3.8:27b digest `22130167c4c20e20c7b71454612966ca8e8171e9b3cc8ab6ce8aa6cbfec79643`. The dedicated loopback-only service disables cloud/pruning, admits one model request at a time and at most two loaded models. No assets were downloaded.

## Baseline complete retrieval requests

One warmup per input and five timed repetitions produce 20 timed requests per profile. Timers cover complete MCP request write through response receipt/decoding; model loading is separate. All source/revision/UTF-8 range checks passed and repeated result signatures matched.

| Profile | Warm p50 ms | Warm p95 ms | MCP startup ms | Worker peak bytes |
| --- | ---: | ---: | ---: | ---: |
| BM25 | 0.641 | 1.285 | 30.828 | 25051136 |
| Hybrid | 23.801 | 32.089 | 77.104 | 69009408 |
| Hybrid + reranking, 2 threads/batch16 | 398.875 | 474.982 | 2243.020 | 2049081344 |

These figures use a different workload from the historical 150-question ~982 ms reranking chart. Their difference is not an optimization claim or a universal latency bound. Twenty small-workload samples do not establish production tail latency or capacity.

## Actual local answer attempts

Sixteen bounded Qwen requests cover no-context control plus the three profiles, using the same source-only German-answer instructions, exact quote schema, temperature 0, seed 42, no thinking, context 8192 and at most 512 generated tokens. Warmup is separate. Every pipeline retains fresh retrieval, chat HTTP/full-pipeline timing, model-reported token/caching costs and raw outputs. A source-only no-context prompt is expected to abstain; it is not an unconstrained answer-quality baseline.

- All four no-context controls abstained without invented citations.
- BM25 returned no evidence for one short German continuation question and could not describe the actual version roadmap from its supplied excerpts.
- Hybrid retrieved useful continuation instructions, but its roadmap answer also lacked the actual milestone list. One performance-request answer changed a supposedly verbatim quotation; independent exact-byte verification rejected it. That failed answer is retained.
- Reranking produced valid verbatim citations for its four attempts, but its confident roadmap answer relied on historical 0.5.x material and omitted the actual conditional roadmap. Its `insufficient_evidence=false` did not establish completeness. The actual version table was outside the original 20-candidate prefix for that input.

This is assistant assessment of visible outputs, not independent semantic judgment or a passed nDCG/adoption gate. Valid spans/quotations are necessary but insufficient. No new production-quality recommendation follows. The example still returns evidence; this study's separate answer harness is the calling application, not an implicit answer generator added to the engine.

## Exploratory native options

Before timing, the native probe verified that its fixed 20-candidate prefixes reproduce the actual MCP top-six IDs/scores. Two shuffled campaigns each compared all 16 combinations of CPU threads 1/2/4/8 and outer batch sizes 1/4/8/16, with identical model/text/token recipe. Each worker uses one warmup and two timed repetitions per input. Exact-child peak RSS, CPU time, startup and all score/ranking differences are retained. No answer model was loaded during timing.

| Threads / outer batch | Campaign 1 p95 ms | Campaign 2 p95 ms | Worst ratio to 2/16 |
| --- | ---: | ---: | ---: |
| 2 / 16 baseline | 500.487 | 471.242 | 1.000 |
| 2 / 1 | 265.181 | 266.929 | 0.566 |
| 8 / 1 | 184.773 | 181.000 | 0.384 |

The selected rows have no observed top-six rank changes across these probes. Maximum absolute logit difference is approximately 9.78e-6; exact logits are not claimed identical. The declarations before execution were abs 1e-4 plus relative 1e-5, unchanged prefix/token bounds, all losses retained and at least 20% repeated gain before adoption. Additional CPU threads are an explicit resource tradeoff. Larger/native/full-MCP controls remain required before a completed optimization claim.

Underlying mechanism: FastEmbed pads each inference batch to its longest tokenized pair. Shorter outer batches can reduce padded work for this CPU/model/workload. The model, maximum pair length and candidate set were not reduced. Larger batches and more threads are not universally better; all 32 worker results and slower observations remain available locally.

## Reproduction and remaining work

Retained methods: `prepare_practical.py`, `run_practical.py`, the standalone Rust `native-probe`, `tune_native.py` and `validate_native_reference.py`, plus exact locks/binaries/inputs and every raw failure. The first probe build omitted a harness-only tempfile dependency; the corrected build and failed log are both retained. Generated quote errors are not retried away or relabelled successful.

Complete the frozen 150-case regression controls and paired full-MCP confirmation before selecting final settings, then record candidate tests/Miri/compiler/package results in [validation](../../validation-0.6.1/README.md). Human reviewed data, independent onboarding/reproduction, native interpreter exclusions, process/shared GPU accounting and cross-host performance remain open. User-requested future development is not automatic publication authorization.
