# AIRust 0.2.12: compact Knowledge Retrieval API

Optional `retrieval` provides one provider-neutral source/request/response boundary over existing search algorithms. LexicalSource, TextSource, SemanticSource and HybridSource reuse their existing ranking implementations; optional explicit reranking retains original component evidence. Custom sources may be borrowed or boxed. Selected output is owned and can outlive the source read guard. The feature adds no dependency, server, model or database. Default features and specialized APIs/formats remain unchanged. No DenkMaps/MCP integration is performed.

The [complete contract](RETRIEVAL.md), [API cookbook](API-COOKBOOK.md) and [migration](MIGRATION-0.2.12.md) define scopes, limits and ownership. Returned prefixes retain original UTF-8 spans and expose clipped text, metadata status, source ranks, component scores and explicit fallback diagnostics. A complete encoded-JSON cap includes escaping, envelope and metadata. Tail hits are removed with truncation when needed; an empty envelope that cannot fit is an error, never partial JSON.

This is a transport-neutral result adapter, not a new ranking model or automatic semantic improvement. It does not supply a no-answer classifier, query-centered snippets, model token counts, citations, synchronization, permissions or timeouts. Use existing scoped ContextAssembler/ProvenanceStore explicitly for structural sourced context and retain retrieval evidence separately. The [proposed DenkMaps upgrade guide](DENKMAPS-UPGRADE.md) explains a later application trial without changing that application.

## Correctness and bug hunting

The complete release suite passes **189 default and 434 all-feature Rust tests**, plus five Python helper tests. The isolated retrieval-only suite passes eleven tests. Nineteen new retrieval tests cover all four adapters, dynamic/borrowed sources, original lexical scores/IDs, owned-response lifetime, access intersections before top-K, semantic deny-all without provider calls, phrase spans/German forms, and independent rerank/component evidence. Handwritten test vectors/scorers validate composition, not neural relevance.

Custom duplicate/non-finite/malformed/oversized/out-of-scope candidates are rejected before reranking. Provider errors remain errors by default; explicit fallback preserves original order and absent scores with a visible reason. Zero-result requests skip source/model work, and query limits fail first. Metadata byte/depth limits are checked before cloning. Tests sweep complete output budgets in 17-byte steps and verify full parseable JSON and exact serialized length, including escaping. Five hundred seeded mixed-Unicode cases check excerpt boundaries and total byte caps against original text. Response Serialize and its cached validated JSON agree.

Strict formatting, Clippy/rustdoc, isolated retrieval/advanced/semantic/hybrid/reranking combinations, Rust 1.89 provider-independent and Rust 1.92 full builds pass. An independent Rust 1.89 consumer exercises old APIs, advanced analysis, both storage adapters and all four compact retrieval sources. Static all-feature patch compatibility against 0.2.11 passes 229 checks with 31 skipped. Fresh RustSec reports zero known vulnerabilities and the same optional unmaintained paste/ttf-parser warnings in [audit metadata](measurements/0.2.12/audit.json). This is not an independent security audit. Existing native models were not rebenchmarked for this API-only release; their measurements remain historical.

## Reproducible end-to-end scale check

The release example generates short public synthetic Rust/Unicode records, constructs one Engine, then performs five warmups and thirty observations per query. Timings include candidate retrieval, validation, excerpts, evidence and complete JSON serialization. Every returned ID is checked against the original Engine's top ten. Request settings: ten returned hits, one hundred candidates, 4 KiB total text and 16 KiB encoded output. Build includes record generation; correctness comparisons sit outside timed samples.

| Records | Build ms | Common query median ms | Unique-number median ms | Miss median ms | Common response bytes | Peak RSS MiB |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10,000 | 51.753 | 2.886 | 0.001250 | 0.000458 | 4,592 | 32.58 |
| 100,000 | 346.641 | 34.528 | 0.001417 | 0.000500 | 4,572 | 300.50 |

The common query visits many postings; a rare-number lookup is a different workload. Peak RSS includes the complete RAM corpus/index and output work, not just JSON. Host: Apple M5 Max, 18 logical CPUs, 64 GiB RAM, macOS ARM64, Rust 1.98.0. Raw thirty-sample arrays, commands, CPU and memory are in [measurements/0.2.12](measurements/0.2.12/). These small same-host synthetic runs do not establish production tails, real-map relevance, million-document capacity or token savings.

```bash
cargo build --release --locked --features retrieval --example retrieval_benchmark
python3 scripts/measure.py /tmp/retrieval-100000.json target/release/examples/retrieval_benchmark 100000
```

Clean package/extracted tests, publish dry run, exact-commit GitHub CI and registry/tag/asset digests gate publication. [GitHub release](https://github.com/LEVOGNE/airust/releases/tag/v0.2.12), [crates.io](https://crates.io/crates/airust/0.2.12).
