# AIRust 0.2.10: phrases, German forms and Unicode analysis

The new optional `advanced-search` feature supplies TextIndex, configurable TextAnalyzer/AnalyzerConfig and bounded quoted-token queries. It privately reuses the shared Engine's source/posting storage, BM25/TF-IDF scoring and top-K machinery. Existing Engine methods, default features and persistence/cache formats are unchanged. No DenkMaps/MCP integration is performed.

The analyzer selects Unicode UAX #29 word boundaries or alphanumeric runs, optional typographic-apostrophe folding and optional German Snowball stemming. Terms retain original UTF-8 spans across combining marks and compatibility expansion. Required phrases are matched in token order before top-K, with overlapping occurrences and bounded evidence. Invalid syntax and oversized analysis/query work fail explicitly. Updates stage all validation before changing original entries, postings or positions.

German examples include Haus/Häusern, Katze/Katzen, Führerschein/Führerscheine and Straße/Straßen. This is suffix stemming, not full German lemmatization, compound splitting or semantic understanding. Gehen/ging remain different and schön/schon can merge. Analyzed phrases can span punctuation/whitespace and are not byte-exact quotations. The [complete API contract](TEXT-SEARCH.md) and [cookbook](API-COOKBOOK.md) explain these limits.

## Correctness and bug hunting

Twenty advanced-search tests cover explicit analyzer choices, original spans, normalization expansion, apostrophes/decimals, noun forms and limitations, malformed/escaped quotes, query budgets, overlapping phrases, bounded/truncated evidence, scope-before-top-K and error-atomic replacement. A deliberately higher-scoring nonphrase or excluded document cannot hide the valid top-one result. KMP phrase matching is checked against an independent sliding-window reference over 500 seeded sequences; 600 mixed Unicode sequences are analyzed in both tokenizer modes. After 150 incremental operations, source/posting/position results agree with full reconstruction. Comparable alphanumeric BM25 and TF-IDF scores agree bit-for-bit with the existing Engine.

Source positions are captured against original graphemes rather than treating expanded normalized offsets as original offsets. Token collection stops at its configured cap instead of collecting an arbitrarily larger word vector first. Context is checked before phrase evaluation; phrase constraints are checked before result truncation. Invalid phrase syntax never degrades to unrestricted word search. These are targeted boundary tests, not a completed independent security audit.

The release check covers **189 default and 398 all-feature Rust tests**, five existing Python helper tests, strict formatting/Clippy/rustdoc, isolated features, Rust 1.89 provider-independent and Rust 1.92 complete-feature builds. An independent Rust 1.89 consumer checks original APIs plus German/phrase search. All-feature cargo-semver-checks against 0.2.9 passes 229 checks, with 31 skipped, assuming a patch release. Static API compatibility is not a guarantee that opting into a new analyzer preserves old ranking.

## Reproducible scale check

The release binary builds public synthetic short records with a unique key, German nouns and either an adjacent or gapped English phrase. Source generation is outside build timing. Four queries use five warmups and thirty observations per corpus; up to one hundred replacements are checked against a simultaneous fresh full rebuild. The exact generator and assertions are in `examples/text_search_benchmark.rs`.

| Entries | Build (ms) | Haus median (ms) | Adjacent phrase median (ms) | Missing phrase median (ms) | Unique-key median (ms) | Peak RSS (MiB) |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10,000 | 45.342 | 1.815 | 2.083 | 1.426 | 0.000750 | 58.33 |
| 100,000 | 481.347 | 21.615 | 26.085 | 17.454 | 0.000791 | 564.91 |

The Haus and phrase queries visit many documents; rare-key lookup is a different workload. Memory includes source, positional structures, returned evidence and a simultaneous fresh rebuild, not minimal steady-state storage. Runs use release Rust 1.98.0 on Apple M5 Max/macOS ARM64, 18 logical CPUs and 64 GiB RAM. The raw thirty-sample arrays and OS CPU/RSS are in [measurements/0.2.10](measurements/0.2.10/). These small single-host series do not establish production p95, independent relevance accuracy, token savings or million-large-document capacity.

```bash
cargo build --release --locked --features advanced-search --example text_search_benchmark
python3 scripts/measure.py /tmp/text-100000.json target/release/examples/text_search_benchmark 100000 30
```

## Dependencies and publication

The feature reuses Unicode segmentation 1.13.3 (MIT/Apache-2.0) and rust-stemmers 1.2.0 (MIT/BSD-3-Clause). No upstream algorithm implementation is copied. Default features remain empty; no model or network/database dependency is added to the default core. The analyzer recipe records the standard-library/normalization/segmentation Unicode tables and exact analysis options.

The fresh RustSec audit reports zero known vulnerabilities. The same optional unmaintained warnings for paste/tokenizers and ttf-parser/PDF remain visible in [audit metadata](measurements/0.2.10/audit.json). Clean package/extracted tests, publish dry run, GitHub CI and registry/tag/asset checks gate publication. [Migration](MIGRATION-0.2.10.md), [GitHub release](https://github.com/LEVOGNE/airust/releases/tag/v0.2.10), [crates.io](https://crates.io/crates/airust/0.2.10).
