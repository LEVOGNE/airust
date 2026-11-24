# Advanced text search

Enable `advanced-search` for explicitly selected Unicode analysis, optional German word stemming and quoted token phrases. Existing `Engine` methods, default features, saved formats and rankings are unchanged. Rust 1.89 is supported. No model, HTTP service or database is required.

```toml
[dependencies]
airust = { version = "=0.2.10", default-features = false, features = ["advanced-search"] }
```

```rust
use airust::{KnowledgeEntry, text_search::{AnalyzerConfig, TextIndex, TextSearchOptions}};
let index = TextIndex::from_entries(AnalyzerConfig::german(), [
    KnowledgeEntry::new("a", "Wir bauen große Häuser mit Gärten."),
    KnowledgeEntry::new("b", "Häuser sind groß. Wir bauen später."),
])?;
let hits = index.search("\"große Häuser\"", &TextSearchOptions::new(5))?;
assert_eq!(hits.len(), 1);
assert_eq!(hits[0].entry.id, "a");
assert_eq!(hits[0].phrases[0].spans[0].slice(&hits[0].entry.text), Some("große Häuser"));
assert_eq!(index.search("Haus", &TextSearchOptions::new(5))?.len(), 2);
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Ownership and updates

`TextIndex` privately owns one shared Engine plus positional token evidence. It reuses the existing posting storage, BM25/TF-IDF mathematics and top-K ordering rather than maintaining a second source corpus or independent ranking formula. It does not expose that internally analyzed Engine as a legacy search index. `new(config)`, `from_entries(config, entries)`, `upsert`, `remove`, `clear`, `get`, `entries`, `len`, `is_empty` and `stats` follow the existing source-entry conventions. Duplicate constructor IDs fail. Replacements validate complete analysis before removing previous source/posting/position state.

`analyzer()` borrows the selected `TextAnalyzer`; its immutable config applies to both documents and queries. `tokens(id)` exposes analyzed terms, positions and original UTF-8 spans. There is no automatic in-place analyzer switch, semantic-index wrapper conversion or persistence/cache conversion. Retain original records and the analyzer recipe/config and rebuild explicitly when changing it. Existing cache/vector formats do not suddenly become positional-index files.

## Unicode and German forms

`AnalyzerConfig::default()` selects UAX #29 `UnicodeWords`, no stemming and typographic-apostrophe folding. `AnalyzerConfig::german()` additionally selects `Stemming::German`. Choose `WordTokenizer::Alphanumeric` for letters/numbers/combining-mark runs, or use `with_tokenizer`, `with_stemming` and `with_apostrophe_folding` explicitly. The original `text::tokenize` remains unchanged.

Unicode word rules retain examples such as `can't`, `l’homme`, decimal numbers and snake-case words according to their word boundaries. The alphanumeric mode instead splits punctuation/underscores/decimal points. Apostrophe folding maps left/right typographic apostrophes and the modifier apostrophe to ASCII for analysis only. Hyphens and path punctuation are token boundaries under the selected rules, not a new path/identifier query language. CJK segmentation follows Unicode rules, without a language dictionary.

Normalization applies NFKC/lowercase processing while mapping normalized segments back to original graphemes. Analyzed terms are normalized again before optional stemming. Compatibility ligatures, combining marks and expansions keep safe original UTF-8 spans. Multiple expanded tokens can refer to the same original grapheme; offsets are not normalized-string offsets. A phrase span covers the first through last matching original token, including intervening punctuation/whitespace. It is an analyzed token phrase, not a byte-exact substring.

German Snowball maps useful forms such as Haus/Häusern, Katze/Katzen and Führerschein/Führerscheine. It does not provide full lemmatization, compound splitting, synonym recognition or irregular-verb understanding: gehen and ging remain different. Stemming may conflate unrelated meanings, including schön/schon. It is fixed, explicit analysis, not a truth or semantic-relevance model. The selected implementation is `rust-stemmers` 1.2.0; do not assume every rule in later upstream Snowball revisions is included.

`config().recipe()` identifies the analyzer format, normalization, standard-library and segmentation Unicode versions, tokenizer, pinned stemmer version/language and apostrophe policy. Keep this identity with derived data. Unicode tables can differ across compiler/library versions. Recipe equality does not establish that an external file is trusted.

## Query grammar

`search(query, &TextSearchOptions)` accepts unquoted words and required double-quoted phrases. Every quoted phrase must match in its analyzed token order with no intervening token. Overlapping occurrences are supported. Punctuation and whitespace can separate adjacent tokens. No stop words are removed, so phrases do not silently bridge removed words. Stemming also applies inside phrases when enabled.

Every phrase is required. If unquoted terms exist, default `TermOperator::Any` additionally requires at least one of them; `All` requires all of them. For example, `"red car" blue green` requires the red-car phrase plus blue or green; select All to require both. Phrase constraints and Context filters apply before top-K, not as a filter on an already truncated result set. Ranking remains the shared BM25/TF-IDF score over analyzed query terms; there is no hidden phrase boost.

Backslash escapes a quote or another backslash; other backslashes remain ordinary text, supporting literal Windows paths. Empty phrases, phrases without searchable tokens and unclosed quotes return typed errors instead of bypassing phrase constraints. Empty/whitespace queries fail; punctuation-only unquoted queries can return no hits. This is not a Boolean/wildcard/regex/fuzzy query language.

## Options and evidence

`TextSearchOptions::new(limit)` configures the result count. `with_search(SearchOptions)` selects BM25/TF-IDF and its parameters; `with_context` restricts IDs/metadata; `with_operator` selects unquoted Any/All. The existing empty-Context-ID-set-means-unrestricted rule remains: callers with no authorized IDs return no hits before querying. Corpus statistics still describe the complete index. This API is not an authentication service.

Results borrow their original entry and preserve score/matched analyzed terms. `phrases` contains each phrase's analyzed terms and original `TextSpan`s. `truncated` means additional occurrences were omitted; it is not the exact total count. `TextSpan::slice(original)` safely returns None for invalid caller-constructed offsets. Result scores are not probabilities. Original metadata/text are retained and not automatically sent anywhere.

Default analysis limits are 1 MiB original bytes, 4 MiB normalized bytes and 100,000 tokens per analysis. `with_limits` changes positive limits. Query defaults are 16 KiB, 256 total analyzed tokens, 16 phrases and 64 tokens per phrase; `with_query_limits` changes them. `with_max_spans` changes the default 64 stored occurrences per phrase/result. Limits reject oversized work without partial mutation or silently truncated tokenization. A zero result count still validates query syntax. These budgets bound individual inputs/evidence, not exact aggregate RAM or worst-case corpus-wide latency.

Positions cost memory. Common-term/phrase queries may visit many candidate documents; input scanning and KMP phrase matching remain corpus-dependent. There is no claim of constant-time common queries or production tail latency. Returned entries remain borrowed full records; choose compact excerpts before external serialization. [Measured methods and results](https://github.com/LEVOGNE/airust/blob/v0.2.10/docs/CHANGES-0.2.10.md) state corpus and resource scope.

## Composition

With the separate `reranking` feature, `TextSearchResult` implements `Rerankable`. The scorer receives only original candidate text; phrase spans and original ranking evidence stay on its candidate. Reapply scope at every later retrieval/context stage. A TextIndex is an alternative lexical owner, not an automatic semantic-vector-index conversion. Source-entry, graph and provenance reconciliation remain application work. No DenkMaps or MCP integration is performed.

Primary references: [Unicode segmentation API](https://docs.rs/unicode-segmentation/1.13.3/unicode_segmentation/trait.UnicodeSegmentation.html), [Snowball German design and limitations](https://snowballstem.org/algorithms/german/stemmer.html), and [rust-stemmers 1.2.0](https://docs.rs/rust-stemmers/1.2.0/rust_stemmers/). Unicode segmentation is MIT/Apache-2.0; rust-stemmers is MIT/BSD-3-Clause. Dependencies are reused; no upstream algorithm source is copied.

## Cloning and metadata-only updates

TextIndex implements Clone. Exact unchanged text retains its analyzed positions when metadata changes. With shared-index enabled, unchanged token buffers can be shared across clones; edits preserve older views. See [incremental computation](INCREMENTAL.md).

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use airust::{KnowledgeEntry, text_search::{AnalyzerConfig, TextIndex}};
let mut current = TextIndex::from_entries(AnalyzerConfig::german(), [KnowledgeEntry::new("a", "Häuser haben Türen")])?;
let earlier = current.clone();
current.upsert(KnowledgeEntry::new("a", "Häuser haben Türen").with_metadata(serde_json::json!({"version": 2})))?;
assert!(earlier.get("a").unwrap().metadata.is_none());
assert_eq!(current.get("a").unwrap().metadata, Some(serde_json::json!({"version": 2})));
assert_eq!(earlier.tokens("a"), current.tokens("a"));
# Ok(()) }
```
