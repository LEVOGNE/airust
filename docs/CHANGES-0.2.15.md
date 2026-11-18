# AIRust 0.2.15: explicit retrieval evidence assessment

AIRust can now report whether delivered evidence meets an explicit policy, is insufficient or cannot be judged. Assessment is optional under the existing `retrieval` feature and introduces no dependency or mandatory model. Default search, ranking, struct literals and response JSON remain compatible. No DenkMaps/MCP integration is performed.

The default policy checks all distinct informative query terms in the same nonblank delivered excerpt. It does not combine words across unrelated documents or trust a provider's claimed matched terms. Configured text analyzers are reused. Semantic, fused and custom scores need explicit evaluated gates with matching score scale and recipe, or an explicitly requested lexical check. Provider fallbacks are uncertain; technical and malformed-output errors stay errors. Full JSON includes the report; tail removal reassesses retained hits rather than retaining stale acceptance.

## Executed verification

The complete suite passes **189 default and 501 all-feature Rust tests**, including compiled documentation, plus five Python helper tests. Eighteen targeted all-feature assessment integration cases cover independent set-reference coverage over 600 seeded cases, German morphology and compatibility Jamo, relevant versus prefix excerpts, access scope, stopwords, zero limits, malformed scores, exact score/recipe matching, reranking/fallbacks, early limits, old response shape and repeated output-budget truncation. The isolated retrieval-only assessment suite runs sixteen cases.

Strict formatting, all-target Clippy, rustdoc warnings, provider-independent Rust 1.89 and all-feature Rust 1.92 checks, static patch compatibility against 0.2.14 and an independent flat consumer are release gates. No native neural quality run is claimed. Extracted-package tests, publish dry run, exact-commit CI and registry/asset/VCS checksum verification also gate publication.

## Fixed evaluation and cost

The authored twelve-case lexical fixture contains six labeled answerable and six unanswerable cases. Accepting any delivered hit gives five false acceptances. Default assessment reduces that to one, with one false rejection. The remaining false acceptance is a glossary containing every keyword but no explanation. The false rejection is a relevant paraphrase lacking literal terms. These counterexamples demonstrate why passing the policy is not answer correctness or semantic understanding. The fixture is small and authored here; it is not independent production validation.

A release-build run on this host takes 27.39 ms for 10,000 tiny unassessed queries and 55.26 ms with assessment. Serialized bytes rise from 5,430,000 to 8,890,000 across those calls. This is added checking and output cost, not a search speed improvement. Whole-child peak RSS is 2.84 MiB. Values are descriptive for this one cached tiny fixture, not latency guarantees. Raw cases, outputs, arguments and OS resource metadata are in [measurements/0.2.15](measurements/0.2.15/).

```bash
cargo build --release --locked --features retrieval --example assessment_quality
python3 scripts/measure.py /tmp/assessment.json target/release/examples/assessment_quality
cargo run --locked --features retrieval --example evidence_assessment
```

## Limits and maintenance

Default bounded analysis permits 16 KiB query text, 256 distinct query terms, 100 requested hits, 64 KiB per excerpt, 256 KiB normalized text and 8192 tokens. Limits do not constrain arbitrary third-party provider allocations. Stopword removal is a configurable heuristic. Generic recipes do not fingerprint every corpus/model parameter; threshold calibration must be repeated when the complete evaluated pipeline changes. Assessment cannot establish factual truth, detect every negation or prove that knowledge is absent outside the delivered scope.

Fresh RustSec results and database revision are in [audit metadata](measurements/0.2.15/audit.json). Existing optional unmaintained notices remain visible. See the [complete API contract](ASSESSMENT.md), [migration](MIGRATION-0.2.15.md) and [cookbook](API-COOKBOOK.md).
