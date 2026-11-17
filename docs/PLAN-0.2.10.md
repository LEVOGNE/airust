# AIRust 0.2.10 release contract

Baseline: published and artifact-verified 0.2.9 at 469e1bbe63fa7959a72f00523f8c30eed166b006. Add optional phrase queries, configurable Unicode analysis and German suffix stemming while preserving the default Engine and all existing source/cache formats. Reuse existing index storage/ranking and maintained Unicode/Snowball dependencies; copy no third-party algorithm source.

Apply scope and required phrase/term constraints before top-K. Preserve original UTF-8 evidence through normalization expansion, detect malformed syntax, bound input/query/evidence work and validate replacements before mutation. Keep analyzer identity explicit and immutable. Do not claim full German lemmatization, decompounding, semantic truth or unchanged rankings after opting into a different analyzer.

Gate with reference/property/Unicode/update tests, small and 100k public scale measurements, strict Clippy/docs/formatting, isolated features, Rust 1.89/1.92, independent consumer, SemVer comparison, audit, package/extracted checks, CI, publish dry run and artifact verification. Publish and verify 0.2.10 before implementing the 0.2.11 storage backend. No DenkMaps/MCP integration.
