# Relevant excerpts and original positions

AIRust 0.2.13 adds an explicit relevant-excerpt mode to the existing `retrieval` feature. It finds an original-text window around matching words or supplied phrase evidence. The default prefix behavior remains compatible with 0.2.12. No model, server or application integration is added. Retrieval now uses the already pinned Unicode segmentation library; default crate features remain empty.

```rust
use airust::{Engine, KnowledgeEntry, retrieval::{ExcerptOptions, LexicalSource,
    RetrievalOptions, Retriever}};
let original = format!("{} Rust ownership prevents memory errors.", "Background. ".repeat(100));
let index = Engine::from_entries([KnowledgeEntry::new("manual", &original)])?;
let options = RetrievalOptions::new(5)
    .with_byte_budgets(1024, 1024, 120, 8192)?
    .with_relevant_excerpts(ExcerptOptions::default());
let response = Retriever::new(LexicalSource::new(&index)).query("Rust ownership", &options)?;
let hit = &response.hits()[0];
assert!(hit.span.start > 0);
assert!(hit.excerpt.contains("Rust ownership"));
assert_eq!(hit.span.slice(&original), Some(hit.excerpt.as_str()));
for matched in &response.excerpt_details().unwrap()[0].matches {
    let relative = matched.span.relative_to(hit.span).unwrap();
    assert_eq!(relative.slice(&hit.excerpt), matched.span.slice(&original));
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

Run `cargo run --features retrieval --example relevant_excerpts`. Add `advanced-search` to also run its German stem/phrase example. `select_excerpt` exposes the same borrowed selection helper for callers already holding text and RankingEvidence. Its analyzer must agree with the producer of the evidence.

## Selection and evidence

The bounded selector analyzes the original text and finds complete matching tokens. Supplied analyzed `matched_terms` are used when available. Otherwise it analyzes the query to look for literal lexical overlap, including for semantic candidates. Supplied phrase spans are treated as producer-provided hints; they must reference valid, nonempty original grapheme boundaries. A custom producer is responsible for the truth and query relevance of its hints. The helper does not prove them by rereading the query as a phrase grammar.

Candidate windows are compared by: containing a complete phrase hint first, then the number of distinct matching terms, then the number of matching token occurrences. Equal scores retain the earliest window. A sweep over sorted endpoints and a heap avoids a quadratic scan of all window/match combinations. The selected match cluster receives surrounding context within the byte budget. This is a deterministic lexical heuristic, not a model-based passage ranker or a guarantee that a complete answer is present.

Relevant-mode window boundaries and highlights follow extended Unicode grapheme boundaries. Combining marks, emoji ZWJ sequences, flags and CRLF are preserved. A cluster larger than the available budget is omitted whole. Original text is never normalized, rewritten, HTML-escaped or decorated in the returned excerpt. Analysis uses NFKC/lowercase and maps matches back to original graphemes; mapped legacy words are normalized again to preserve composition across original graphemes (including compatibility Jamo). Normalization expansion can yield overlapping highlights.

`LexicalSource`, SemanticSource and HybridSource use the legacy alphanumeric analysis for localization. `TextSource::excerpt_analyzer` supplies its exact tokenizer, apostrophe and stemming configuration, including optional German Snowball forms. Boxed/borrowed source forwarding preserves it. Custom RetrievalSource implementations inherit Legacy analysis and can override the additive default `excerpt_analyzer` method. Analysis budgets come from ExcerptOptions, not from changing the source index. These additions do not merge TextIndex and SemanticIndex ownership or change document ranking.

`RetrievalResponse::excerpt_details()` is Some only when relevant mode is requested. Its entries have the same order and IDs as retained hits. Each detail contains a reason, bounded original-position matches and `matches_truncated`. JSON adds the optional `excerpt_details` field under the existing schema version 1. Existing public hit/evidence/batch structs gain no fields, preserving their literal construction. Prefix-mode JSON omits the new field. `with_prefix_excerpts()` restores the original prefix mode.

Reasons are:

| Reason | Meaning |
| --- | --- |
| `phrase` | A complete supplied phrase span fits inside the selected window. |
| `terms` | Complete matching analyzed tokens locate the window. |
| `no_lexical_match` | No token overlap or usable phrase hint exists; return a grapheme-safe prefix without highlights. |
| `no_fitting_match` | Matches exist, but none fits the byte budget completely; return a prefix without highlights. |
| `empty_budget` | No text bytes remain; return the empty 0..0 span without analyzing the record. |

Semantic relevance can exist without matching words. In that case, `no_lexical_match` is honest: cosine ranking is retained, but the engine does not invent a localized semantic justification. Small budgets can also leave a fallback excerpt empty. Match reasons are not confidence probabilities or no-answer classifications. TextAnalyzer phrases may span punctuation and use stems, so a phrase hint does not imply a byte-exact quotation of the query.

## Positions and display

Every span is a zero-based half-open UTF-8 **byte** range in the exact `KnowledgeEntry.text` used for that query. `hit.span.slice(original)` equals the returned excerpt. Every returned highlight is contained by that span; `matched.span.relative_to(hit.span)` converts its offsets to excerpt-local bytes. The checked helpers return None for reversed/outside spans or invalid UTF-8 slicing. They do not convert offsets to characters, JavaScript UTF-16 units, PDF pages or source-file coordinates.

Keep the original record revision alongside these positions. Revalidate against the exact text after edits; the numeric span alone proves neither freshness nor provenance. Escape raw text for the target UI, and translate byte positions before using a character-based rendering API. Do not concatenate highlights into a synthesized quotation or treat overlapping normalized highlights as independent source passages. Existing original ranking evidence can reference spans outside the selected excerpt; `excerpt_details.matches` contains only the selected window's markers.

## Limits and failure behavior

Relevant excerpts share RetrievalOptions' per-excerpt, total-text and complete-JSON byte budgets. ExcerptOptions additionally defaults to 1 MiB input text, 4 MiB normalized analysis/total supplied term bytes, 100,000 tokens, a 16 KiB query, 256 supplied/query terms, 4,096 matching occurrences plus phrase hints, and 64 returned highlights. Positive builders are `with_analysis_limits`, `with_query_limits` and `with_match_limits`; getters expose each setting. These are byte/count bounds, not exact heap or execution-time bounds.

Input, query, normalization, token, term and match-count overruns return typed retrieval budget errors. Invalid supplied hints/terms return output errors. The selector does not quietly scan only a prefix of an oversized record and label it the best passage. Reduce/chunk authoritative records or explicitly raise budgets after measuring. Selection runs only on returned hits after candidate validation and optional reranking; a budget error can therefore occur after model inference. Rerankers still receive the full bounded original records, never the selected snippets.

Highlight output truncation preserves the selected window and sets `matches_truncated` plus the response's `truncated` flag. Text/metadata/JSON truncation remains explicit. Complete JSON limits include the new details, repeated IDs and highlights. If tail hits are removed to fit, their details are removed together. An envelope that cannot fit returns an error instead of partial JSON. Empty-result/zero-limit requests retain an empty details list when relevant mode is selected.

The feature performs no database synchronization, network access, document import, automatic chunking, semantic passage embedding or DenkMaps/MCP integration. See [retrieval](RETRIEVAL.md), [API cookbook](API-COOKBOOK.md) and the [0.2.13 validation report](CHANGES-0.2.13.md) for executed checks and measurements.
