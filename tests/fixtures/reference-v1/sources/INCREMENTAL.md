# Incremental text computation

This release improves existing index and snapshot operations without a new mandatory module or feature. Advanced positional search remains optional. The shared-index flag is still explicitly selected when immutable buffer sharing is useful; default features remain empty.

## What is reused

Engine::upsert recognizes exact unchanged text. It updates metadata without removing/rebuilding term postings, normalized duplicate bindings or corpus token counts. An identical stored payload can keep its record buffer. The return value remains the actual previous record. Bulk construction retains its direct ingestion path rather than paying an extra metadata comparison for every new record.

TextIndex now implements Clone. A metadata-only upsert retains analyzed tokens and original coordinates while returning the new metadata. With shared-index, cloned token vectors are shared until replaced; without it, owned clones still keep the same independent behavior. Analyzer configuration stays unchanged within an index. Metadata numbers and object iteration order are compared by their serialized representation, so 0.0/-0.0 or 1/1.0 cannot silently keep an older payload. This refers to the stored JSON value, not original JSON-file whitespace.

DatasetManager preparation clones/updates only affected positional records. If every record payload is identical and only source bindings/revision markers change, it can retain the complete old TextIndex Arc. Deletions remove positional state. Changed original text always reanalyses that record, even when normalization produces identical terms: fullwidth characters, composition or spacing can change original byte positions. Metadata-only updates still replace visible metadata in every projection.

A model refresh still deliberately rebuilds all requested vectors. Literal-text vector reuse from earlier releases is preserved; it is not presented as a new improvement. Model-only changes can retain the positional projection. Failed analysis/provider work, cancellation and stale publication preserve the previous published generation. External database transactions and graph/cache/HNSW synchronization remain application-owned.

## Work receipts

PreparationStats remains non-exhaustive and adds text_entries_reused, metadata_only_puts and unchanged_puts. text_entries_rebuilt now counts records actually reanalysed, not the full corpus for every update. Reused counts cover retained text analysis in the final projection, including metadata-only edits; removed records are excluded. Metadata-only/unchanged put counts refer to KnowledgeEntry payloads, separately from changed source references. No positional projection means zero positional counts. Model-only publication can report reused positional entries.

These diagnostic JSON fields are additive. Consumers serializing receipts should accept additional fields; receipts are not a new durable backup format. Counts explain work, not relevance or answer confidence.

## Evidence and usage

Run `cargo run --features snapshots,advanced-search --example incremental`, then repeat with shared-index enabled. The same public update methods are used; there is no required migration to a new engine type. The compiled text-search documentation demonstrates Clone. The [release report](CHANGES-0.2.21.md) compares the unchanged 10k positional snapshot workload to 0.2.20, including initial build, metadata/text updates, deletion and RSS. Its handwritten 2D vectors test consistency only.

Tests compare 600 seeded updates with full independent rebuilds, including German phrase evidence, metadata and retained readers. Other regressions cover serialized metadata numbers, normalized-equal text with changed original byte positions, failed analysis and cancellation before publication. No DenkMaps/MCP integration or general semantic understanding is added.

Source bindings retain their complete-replacement contract: include the full intended source list on every put, including metadata-only edits. An omitted list clears earlier citations.
