# AIRust 0.3.4 acceptance plan

Optimize the measured long-line object-loading bottleneck without dropping structure-tree information, changing source PDFs or losing retained knowledge.

- Maintain an explicitly named optional parser backend derived from the exact pdf_oxide0.3.78 source. Preserve original licenses/notices and verify the small source patch automatically.
- Probe at most4KiB only on long physical lines. Accept complete nonstream objects with an explicit end marker using the existing parser. Keep original fallback, decryption, caching and structure-tree semantics.
- Fix the known invalid UTF-8 predecessor slice in object-keyword detection.
- Test complete/truncated/large/stream/literal/Unicode cases, differential object values and full tagged structures, cold parallel file loading, malformed inputs and minimum Rust.
- Compare raw text on all physical pages, cleaned chunk hashes, provenance, counts and actual ranked IDs/page ranks across all15 retained PDFs. Include the2488-page public reference. Repeatedly measure the slow case and representative normal inputs with matched harnesses/profiles.
- Publish the independently tested optional backend first, after exact-commit CI. Then select it from AIRust0.3.4 and verify the complete main release with independent registry consumers.
- Recheck CLI budgets, worker failure/timeout cleanup and protected output. Retain all successful knowledge separately from fuzzed/edited test copies.
- Run the complete feature/test/lint/docs/MSRV/API/audit/package/CI gates, update English README/API/reports, publish GitHub/crates artifacts and verify checksum/VCS/tag/asset identities.

No DenkMaps/MCP integration, model requirement, input rewriting or claim of perfect PDF interpretation. Keep default core dependencies unchanged. All private text and source paths remain outside public artifacts.
