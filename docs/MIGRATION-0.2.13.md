# Migrating to AIRust 0.2.13

Update the exact dependency to 0.2.13. Default features remain empty; existing retrieval prefix output and public hit/evidence/batch struct literals stay compatible. The retrieval feature now includes the already pinned unicode-segmentation dependency. No model, database, HTTP or MCP dependency is added.

Opt into RetrievalOptions::with_relevant_excerpts(ExcerptOptions::default()). Hit spans may then start after zero. Treat all positions as original UTF-8 byte ranges, retain exact text/revision and use checked slice/relative_to helpers. A new optional excerpt_details JSON field explains selected highlights/fallbacks; existing schema version 1 permits additive fields. Prefix-mode JSON omits it; with_prefix_excerpts restores that mode explicitly.

Configure and measure analysis/highlight budgets separately from text and JSON limits. Oversized analysis returns errors; too many displayed highlights sets omission flags. Semantic wordless paraphrases retain their ranking with a visible lexical-localization fallback. Custom sources can override excerpt_analyzer when their matched_terms use another supported analyzer; boxed/borrowed adapters forward it. See [EXCERPTS](EXCERPTS.md) and [API cookbook](API-COOKBOOK.md). No DenkMaps integration or dependency update is performed.
