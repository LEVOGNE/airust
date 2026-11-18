# AIRust 0.2.22: explicit adaptive retrieval

The optional adaptive feature adds a bounded source registry, replaceable routing policy and decision traces over the existing RetrievalSource API. Only configured backends can run. There is no model download/loading, automatic schema change or hidden provider retry. Default features, original ranking recipes/scores, scopes and response JSON remain unchanged. No DenkMaps/MCP integration is performed.

Metadata-only preflight freezes the selected route and analyzer before assessment/excerpt planning. A stateful policy is called once, so it cannot silently execute a different analyzer/backend later in the same request. Boxed and borrowed sources forward this hook. A regression caught and fixed missing wrapper forwarding: rejected preflight must prevent retrieval, and boxed adaptive sources must retain German analysis.

## Executed validation

The complete suite passes **194 default and 572 all-feature Rust tests**, including compiled documentation, plus five Python helper tests. Thirteen adaptive integration cases cover role selection, model-free long queries, preserved recipes/scores/JSON/assessment, phrase capability enforcement, original spans, stateful analyzer planning, scope/budgets, registration/choice limits, early assessment caps, zero limits, boxed/borrowed forwarding, technical failures and cancellation. The adaptive-only suite runs nine cases.

Strict formatting, default/all-feature/adaptive Clippy, rustdoc, minimum Rust 1.89/1.92, patch compatibility against 0.2.21, audit and an independent boxed consumer gate release. Extracted package/dry run, exact-commit CI and registry/asset/tag/VCS verification gate publication.

## Fixed measurements and limits

The authored German/phrase fixture gets 4/8 expected top-one/empty cases with legacy lexical search and 8/8 with the configured positional backend selected. This demonstrates composition with existing German analysis, not independent semantic quality or a universal routing gain. A default policy can still choose poorly for another corpus, a short paraphrase or a long exact query.

A separate one-document constant-2D-vector fixture makes 1000 query embedding calls for 1,000 always-hybrid requests and 200 when short requests use the registered lexical backend. Elapsed values are 2.234/2.201 ms; the stand-in provider is deliberately cheap and these tiny timings establish no real-model speed claim. They verify call avoidance/orchestration only. Raw cases, route traces and OS metadata: [measurements/0.2.22](measurements/0.2.22/).

Source roles are caller assertions. Registry/query/name/reason bounds constrain this layer, not arbitrary custom provider allocation or blocking. Quotes require a declared Text backend under the default policy. Technical errors remain errors. The selected source's actual recipe stays intact for explicit calibrated score gates; trace bytes are outside the response budget. Coordinators are per-request mutable trait-object holders, not a new shared scheduler. Use consistent dataset views and returned Result for final status.

See [routing contract](ADAPTIVE.md), [API cookbook](API-COOKBOOK.md), [migration](MIGRATION-0.2.22.md) and [audit metadata](measurements/0.2.22/audit.json).
