# AIRust 0.6.1 — Configurable bounded reranking and offline profile checks

This is an **unreleased development candidate**. Published 0.6.0 and its tag/package/evidence are unchanged. Publication of 0.6.1 needs its own release instruction.

The work follows an actual local-model pilot before adding configuration. Four real conversation inputs were evaluated over 18 current project documents with BM25, hybrid, hybrid/rerank and Qwen3.8:27b. Complete raw answers, exact source ranges, a generated quote mismatch and a confident historical-roadmap answer are retained. This is exposed local evidence, not independent customer traffic or judgments. The separately frozen 150 authored questions remain regression controls.

The native investigation compared existing CPU-thread and batch controls with the same model, original text, 20-candidate prefix and max512 pair recipe. Smaller outer batches avoid padding every document to the longest document in a larger batch. Results and all numerical/ranking/latency differences belong in [the measurement record](measurements/profile-practical-0.6.0/README.md); a tunable setting alone is not a speed claim. Full-request confirmation and interpreter/native checks are recorded separately.

## Implemented behavior

- The reference `hybrid_rerank` JSON can specify `reranker.batch_size` in 1..16 and `reranker.candidates` in 1..100. Existing validated `RerankOptions` execute those settings; no new scorer, database, tensor framework or global configuration facade is added.
- Missing `batch_size` keeps 16. Missing/null `candidates` keeps the original automatic max(20, requested result limit), including application-owned MCP callers requesting more than 20 results. An explicit candidate budget is not expanded by a larger result request; that request fails before model work and cannot become lexical fallback.
- Cancellation remains checked before request-specific configuration processing/model admission. Original scope, source-revision checks, ownership, failure policies and deadlines remain.
- `reference_workspace profile-check CONFIG` validates the bounded JSON and reports effective settings/relative paths without loading a workspace/cache/native runtime or contacting a service. It requires `pilot`; output is capped at 32 KiB. This is configuration preflight, not an assertion that assets or an inference service are available.
- [CONFIGURATION.md](CONFIGURATION.md) maps existing defaults, validation, ownership and feature interactions. The small default Engine API and feature set remain unchanged.

Read [migration](MIGRATION-0.6.1.md), [pilot usage](PILOT.md) and [validation](validation-0.6.1/README.md). Tests cover legacy defaults, explicit budgets, invalid/unknown fields, relative paths, no-network preflight, cancellation priority, non-fallback configuration errors and supported feature/compiler boundaries. Actual command results, including any failed attempts, govern completion.

## Limits

CPU/batch choices are workload/hardware-dependent and can change floating-point outputs slightly; retained comparisons use declared numerical tolerances and list every ranking/labeled-reference loss. The native adapter still has an internal maximum batch of 16 and a maximum combined pair length of 512 tokens. Smaller batches do not recover candidates outside the first-stage prefix or restore truncated text. Extra threads consume more CPU resources; do not treat a wall-time gain as an energy-efficiency measurement.

Local LLM quotations require exact verification. Valid quotations do not establish a complete or correct answer. Three historical question replays and one current improvement request do not supply the minimum 50 genuine independently reviewed cases. Independent onboarding/reproduction and a new production-profile recommendation remain open. Native ONNX/Metal/HTTP execution is outside Miri. No new libFuzzer campaign or universal performance/safety/quality claim is made.
