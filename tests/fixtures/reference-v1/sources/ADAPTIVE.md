# Explicit adaptive retrieval

Enable `adaptive`. It enables retrieval only; phrase analysis, embeddings, local models, servers and storage are separate choices. Register already configured sources, then select with a policy. Nothing downloads or loads a model automatically.

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use airust::{Engine, KnowledgeEntry, adaptive::*, retrieval::*};
let index = Engine::from_entries([KnowledgeEntry::new("a", "Rust ownership")])?;
let source = AdaptiveSource::default()
    .with_source("words", SourceRole::Lexical, LexicalSource::new(&index))?;
let mut retriever = Retriever::new(source);
let response = retriever.query("Rust ownership", &RetrievalOptions::new(3))?;
assert_eq!(response.hits()[0].id, "a");
assert_eq!(response.recipe(), "airust-lexical-v1");
assert_eq!(retriever.source().last_route().unwrap().source(), "words");
# Ok(()) }
```

## Default policy and replacement

SourceRole labels describe capabilities asserted by the registrant: Lexical, Text, Semantic, Hybrid or Custom. The default policy counts at most four normalized query tokens to distinguish queries of up to three terms from longer queries. Short queries prefer Text, Lexical, Hybrid, Semantic, then Custom. Longer queries prefer Hybrid, Semantic, Text, Lexical, then Custom. Within a role, the first registered source wins. These are visible heuristics, not language understanding or a guarantee that the cheapest/best source is chosen.

An ASCII quoted query requires a Text-role source under the default policy. Without one, it returns a configuration error instead of silently treating a phrase as unrelated words. A Text-role backend must actually honor its declared phrase contract; the engine cannot establish the semantics of arbitrary custom code. A model-free registry remains valid for unquoted long queries.

Implement RoutingPolicy::choose(query, available) and construct a validated RouteChoice to replace these priorities. Policy choices must name a registered backend. The registry permits sixteen sources, names up to 64 bytes and reasons up to 256 bytes, with nonblank/control-free labels. Routing also caps queries at 16 KiB and honors a smaller retrieval query cap. Limits do not constrain arbitrary provider allocation. Registration does not create a shared global model manager.

## Frozen planning and analyzer identity

RetrievalSource adds default prepare_query(query, options). It is a metadata-only preflight, not a provider call, index mutation or lazy inference hook. Existing implementers inherit a no-op. Retriever validates basic query and primitive assessment caps before preflight, then requests the selected analyzer and performs analyzer-dependent policy checks before actual retrieval. Zero result limits never call preflight/source methods.

AdaptiveSource chooses once during preflight and freezes the route/analyzer for that query. retrieve consumes that plan. This prevents a stateful policy from selecting one analyzer and then executing another backend. Boxed and borrowed sources forward preflight explicitly. Direct RetrievalSource users must keep their prepared query/options paired; normal Retriever calls manage the protocol. Failed preflight is a technical error, never an insufficient-result verdict.

Original retrieval options, scope and execution controls are forwarded. The selected source retains its recipe, scores, original coordinates and fallback reason. No adaptive prefix changes score-gate identity. Backend/provider errors and interruptions are propagated; the router does not secretly try another source. Existing explicit hybrid/reranking fallback policies still apply inside those providers and remain visible.

## Trace and execution

Retriever gains source(), source_mut() and into_source() for coordinator inspection/ownership. AdaptiveSource::last_route returns a bounded RouteTrace with selected name/role, reason, Planned/Called/Returned/Failed state, actual returned recipe and candidate count. It retains no query text after execution. Returned means the backend returned a batch; later response validation/budgeting may still fail. Use the query Result as the authoritative outcome. Traces are separate from response JSON and its byte budget; budget any combined application envelope yourself.

With execution enabled, routing reports a real Routing phase and honors cancellation/deadlines before planning and provider work. The coordinator uses mutable per-request trait objects and promises no cross-thread/shared scheduler. Construct it inside a compute worker from immutable dataset views and configured provider handles; move its owned RetrievalResponse across a transport boundary. When registering lexical/text/semantic views of one dataset, acquire one DatasetSnapshot and use those consistent views. The engine does not verify that unrelated registered sources belong to the same corpus.

## Evaluation and limits

The fixed authored German/phrase fixture improves expected top-one/empty matches from four of eight with legacy lexical search to eight with the available configured TextIndex. This reuses existing German analysis, not a new semantic algorithm. A separate 1,000-query constant-vector orchestration fixture makes 200 query-embedding calls instead of 1,000. It tests provider-call avoidance, not real neural quality or inference speed. Default routing can be wrong for short paraphrases, long exact queries or another corpus; supply an evaluated policy where needed.

Run `adaptive`, `adaptive_quality` (also advanced-search) and `adaptive_benchmark` (also hybrid). See [release evidence](CHANGES-0.2.22.md). No DenkMaps/MCP integration is performed.
