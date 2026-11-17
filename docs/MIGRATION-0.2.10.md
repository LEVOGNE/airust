# Migrating to AIRust 0.2.10

Updating the dependency preserves the existing Engine's analyzer, ranking behavior, entry fields and persisted formats. Default features are empty. Select `advanced-search` and construct a TextIndex explicitly to use positional phrases or German stemming. Do not silently substitute a new analyzer for an application's existing literal-identifier search.

Keep original records and the immutable analyzer config/recipe. Rebuild when changing tokenization, stemming, apostrophe policy or Unicode recipes. TextIndex owns its internally analyzed Engine privately; it does not expose it as a legacy Engine, convert it into SemanticIndex or pass it through old caches. `entries()` lets a caller reconstruct source-owned data deliberately.

Quoted phrases require adjacent analyzed tokens in order and apply before top-K. They can span punctuation/whitespace and use selected stems, so they are not byte-exact original quotations. Original span coordinates remain UTF-8 bytes in the supplied knowledge text; they are not automatically PDF/page/source-file coordinates. Expanded tokens can share one original grapheme.

Use TextSearchOptions for scope, shared BM25/TF-IDF parameters, Any/All unquoted terms and query/evidence budgets. Existing empty Context ID sets are unrestricted; preserve deny-all checks in the application. German stemming is not full morphology or semantic reasoning and can merge meanings. With `reranking`, returned text hits retain phrase evidence through the original candidate.

Rust 1.89 remains supported for provider-independent features. Native model adapters retain their separate Rust 1.92 requirement. No existing database or application is changed. See [TEXT-SEARCH.md](TEXT-SEARCH.md) and [the API cookbook](API-COOKBOOK.md).
