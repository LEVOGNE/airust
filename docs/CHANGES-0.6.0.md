# AIRust 0.6.0: bounded local pilot and explicit live inference

The source milestone integrates the planned 0.5.4 pilot into 0.6.0: one local path from Markdown import and MCP setup to explicit capture, immutable review and independently checked export. It adds explicitly selected live lexical/hybrid/reranked profiles without changing lexical defaults or generating answers. Publication is separate from implementing and testing this source version.

## Source and memory boundaries

The pilot retains immutable workspace/document revisions, original supplied text and exact UTF-8 byte-range quotes. Case envelopes bind payload hashes; export manifests bind restorable sources/cases and declared origin/review counts. Tampered text, stale revisions, split UTF-8, unknown fields, duplicates, symlinks and existing destinations fail validation. Caller-declared origin and reviewer independence remain claims, not authenticated facts.

The reference workflow now caps short-paragraph expansion at 100000 passages before constructing an oversized index. Bounded JSON writers stop during serialization, accounting for escaping rather than building arbitrarily oversized output first. Vector preparation checks its raw f32 allocation budget before embedding. Existing file/text/case/output bounds remain explicit. These work/data bounds are not an OS process-RAM quota or protection against a concurrently hostile filesystem.

Owned library code preserves `#![forbid(unsafe_code)]`. Optional ONNX/native dependencies remain outside that owned-code and Miri boundary. Actual model assets/runtime are explicit, verified inputs; ordinary tests download none.

## Inference ownership and live behavior

`McpSearchBackend` and `McpServer::with_search_backend` execute trusted application search inside admitted blocking work. Before publication, the server rechecks registered IDs, exact source excerpts/spans, metadata, context and output limits. Default retrieval stays lexical. MCP tools, payload schema one, final encoded limits, progress, cancellation and worker admission retain their contracts.

The reference example supports `bm25`, `hybrid` and `hybrid_rerank`. The configured loopback Ollama adapter disables proxies/redirects, brackets each embedding call with model-digest checks, uses original text with truncation disabled, and validates response size/count/dimensions/finite nonzero vectors. Caches bind actual passage IDs, workspace revisions and model recipes. Hybrid uses BM25/exact-cosine RRF with fixed k=60/equal weights; reranking uses the actual first 20 candidates. Native model state is loaded once per server and serialized behind a cancellation-aware lock. Per-request control/observer references are released after inference.

Strict inference failure is the default. Opt-in lexical fallback has an explicit recipe/reason. Setup errors and cancellation/deadlines never become successful fallback. HTTP/native calls remain cooperatively bounded; there is no arbitrary-instruction forced cancellation. A cancelled model request cannot publish a late successful result, and subsequent requests recover.

## Evidence and quality boundary

The [pilot guide](PILOT.md) documents collection, review, export, independent verification and a separate reference-v2 freeze path. At least 50 genuine, unique, independently reviewed questions, unsupported cases, settled labels and a frozen source-family/policy plan are required before the intended new evaluation. Freezing inputs does not mark a quality gate passed.

Three earlier real user questions were locally replayed and exported with explicit `user_question_replay` origin and no independent review. Raw questions/case files stay local. They demonstrate capture operation and supply **zero** independent pilot judgments. Reference-v1 and its historical reserved results remain unchanged; new runs over that exposed authored corpus are regression diagnostics.

The current validation and measurement records distinguish executed checks, complete warm MCP request time, startup, worker RSS, separately observed embedding-service memory and unavailable GPU/shared physical accounting. Every failed gate and individual regression is retained. Read [validation](validation-0.6.0/README.md), [measurements](measurements/pilot-0.6.0/README.md), [dependency decisions](DEPENDENCY-DECISIONS-0.6.0.md) and [migration](MIGRATION-0.6.0.md).

No new independent real-world quality recommendation, universal performance gain, production-capacity guarantee or proof of absence of memory errors follows from this milestone. The code, tests and measurements can be completed while the external genuine-data gate remains unmet.
