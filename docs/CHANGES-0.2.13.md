# AIRust 0.2.13: relevant excerpts with original positions

Version 0.2.13 adds explicit query-related windows to the existing retrieval feature. `with_relevant_excerpts(ExcerptOptions::default())` selects complete matching words or supplied phrase evidence from anywhere within a bounded original record. Returned excerpts and highlights retain exact original UTF-8 byte positions. Prefix defaults, document ranking, existing public hit/evidence/batch struct literals and persisted formats remain compatible. Default crate features stay empty. No DenkMaps/MCP integration is performed.

The selector prefers a window containing a complete phrase hint, then distinct matched terms, then matching occurrences, with deterministic earliest ties. It centers context around the selected evidence, including left context at a document's end. A sorted sweep/heap avoids quadratic window evaluation. Grapheme boundaries preserve combining marks, emoji sequences, flags and CRLF. TextSource forwards its actual tokenizer/apostrophe/stem configuration through borrowed/boxed sources. Other built-in sources locate legacy lexical overlap; semantic-only paraphrases retain their scores with an explicit localization fallback rather than an invented supporting passage.

The [complete excerpt contract](EXCERPTS.md), [API cookbook](API-COOKBOOK.md) and [migration](MIGRATION-0.2.13.md) explain selection, analyzer agreement, strict analysis budgets, coordinate conversion, fallback reasons and compatibility. The new optional `excerpt_details` JSON field stays aligned with retained hit IDs/order. It participates in the complete JSON byte cap. Original and excerpt-relative positions are checked helpers, not automatic PDF/page/file coordinates or freshness proofs.

## Correctness and bug hunting

The full release suite passes **189 default and 457 all-feature Rust tests**, including compiled documentation, plus five Python helper tests. Twenty-one new integration tests cover late-document hits, unique-term versus repetition scoring, phrase priority, no-overlap/too-wide/empty-budget cases, NFC/NFKC/combining source mapping, extended graphemes, malformed hints, independent analysis/query/match/highlight limits, total-text/JSON budgets, old prefix JSON, source permissions, German stems/phrases, dynamic source forwarding, full-document reranking inputs/order and semantic localization fallback. The isolated retrieval-only excerpt suite passes eighteen tests.

A separate unit test compares the optimized window sweep with an independent exhaustive reference over **1,500 seeded cases**, including nested spans, phrase hints and variable-width grapheme boundaries. **600 mixed-Unicode generated cases** check original slices, grapheme boundaries, contained highlights and relative-offset round trips. JSON budgets are swept in 19-byte steps; hit/details alignment and complete parseable output are checked after tail removal. Existing 0.2.12 retrieval tests remain unchanged and pass.

An additional Korean compatibility-Jamo probe found a real mismatch between per-grapheme localization and whole-word core normalization. Normalizing each mapped token through the shared text normalizer fixes cross-grapheme composition while retaining the complete original span. A direct regression and 600 additional compositional-Unicode cases compare localized terms to the core tokenizer. Publication was stopped before tagging, and the corrected source was checked again.

Boundary review corrected context allocation at the end of a document and ensured omitted highlights mark both their local omission flag and the complete response truncated. Supplied term bytes are checked before cloning. Strict count/analysis overruns return errors; they do not quietly convert a partial scan into a claimed best passage. The first sandboxed full run was blocked by the existing CLI test's `ps` call; the complete suite passed with the required system-call permission. No product change was made to bypass that test.

Strict formatting, Clippy/rustdoc, isolated retrieval/analyzer/provider combinations, Rust 1.89 provider-independent and Rust 1.92 all-feature builds pass. An independent Rust 1.89 consumer checks old public struct literals, old APIs, all four retrieval sources, both storage adapters, German forms and new original/relative excerpt positions. All-feature static patch compatibility against 0.2.12 passes 229 checks with 31 skipped. Native models were not rebenchmarked; their original ranking algorithms are unchanged.

## Reproducible localization and cost check

The public synthetic fixture puts literal query terms after a long unrelated introduction in roughly 1 KiB records. One release process builds the Engine, then runs prefix and relevant modes in that fixed order, with five warmups and thirty timed end-to-end samples per query/mode. Both modes return identical original IDs and score bits. Position, highlight and literal-coverage checks are outside timed samples. Limits: ten returned hits, one hundred candidates, 160 bytes per excerpt, 2 KiB total text and 16 KiB complete output.

| Records | Query | Prefix median ms | Relevant median ms | Prefix literal coverage | Relevant literal coverage |
| ---: | --- | ---: | ---: | --- | --- |
| 10,000 | Rust ownership | 4.260 | 4.749 | 0/10 | 10/10 |
| 100,000 | Rust ownership | 47.124 | 48.021 | 0/10 | 10/10 |
| 10,000 | Unique key | 0.001583 | 0.054708 | 0/1 | 1/1 |
| 100,000 | Unique key | 0.001625 | 0.053229 | 0/1 | 1/1 |

Miss medians were 0.000500 / 0.000500 ms at 10k and 0.000459 / 0.000459 ms at 100k, returning zero hits. Build including generation took 247.718 / 2466.655 ms. Common-query JSON grew from 5,582 to 7,712 bytes at 10k and 5,562 to 7,692 bytes at 100k because relevant details/highlights are included. Whole-process peak RSS was 88.77 / 858.30 MiB, including the full original RAM index and all phases, not just the selector.

This deliberately constructed late-hit check demonstrates lexical localization, not independent relevance judgments, universal answer quality, token savings or a production latency guarantee. Sequential mode ordering and single-host samples limit performance conclusions. No million-record run was performed. Host: Apple M5 Max, 18 logical CPUs, 64 GiB RAM, macOS ARM64, Rust 1.98.0. Raw samples, commands and resource metadata: [measurements/0.2.13](measurements/0.2.13/).

```bash
cargo build --release --locked --features retrieval --example excerpt_benchmark
python3 scripts/measure.py /tmp/excerpts-100000.json target/release/examples/excerpt_benchmark 100000
```

## Limits, dependencies and release gates

Relevant mode reuses pinned unicode-segmentation 1.13.3 (MIT/Apache-2.0), already used by advanced search. It adds that optional dependency to retrieval without changing empty crate defaults. No external implementation was copied. Defaults bound input/normalized bytes, tokens, terms, occurrences and returned highlights; the [contract](EXCERPTS.md) lists exact limits. Context outside the matched words may begin/end inside a word, but never inside a grapheme. The heuristic does not generate summaries, perform semantic passage embedding, split documents automatically or synchronize an application database. Existing ContextAssembler behavior is unchanged.

A fresh RustSec database check reports zero known vulnerabilities and three optional unmaintained dependency notices: chrono, paste and ttf-parser. The chrono notice is new compared with the earlier release check. Details and database revision are preserved in [audit metadata](measurements/0.2.13/audit.json); this is not an independent security audit.

Clean-package extraction/tests, publish dry run, exact-commit GitHub CI and matching registry/tag/asset checks gate publication. [GitHub release](https://github.com/LEVOGNE/airust/releases/tag/v0.2.13), [crates.io](https://crates.io/crates/airust/0.2.13), [documentation-only proposed application guide](DENKMAPS-UPGRADE.md).
