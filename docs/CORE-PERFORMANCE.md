# Core reuse in AIRust 0.4.2

AIRust keeps less repeated index text and avoids recalculating an unchanged document's TF-IDF length for every query. Applications keep the same `Engine`, entries and search methods. There is no model to download or new default feature to enable.

## What is shared

Public `KnowledgeEntry` fields remain owned ID/text strings and optional metadata. The index privately shares immutable strings for vocabulary words, document-map/posting/duplicate-group IDs and identical normalized text. A vocabulary word is stored once across postings and document term maps. Original source text is preserved. Normal insertion counts borrowed tokens directly into the shared term map, avoiding an intermediate owned-string map.

This does not turn the whole index into one copy of the corpus. Source text, normalized text, tree nodes, counts and metadata still consume memory. Distinct normalized documents remain distinct. `shared-index` additionally shares complete immutable document/posting blocks between index clones; the default clone still copies containers while sharing their immutable strings. Later clone edits retain their existing copy-on-write behavior.

## TF-IDF reuse

BM25 follows its existing path and creates no TF-IDF norm hints. A nonempty TF-IDF candidate set initializes a small shared hint store. Each visited candidate's complete document norm is calculated using the unchanged sorted-term formula. Later queries reuse that value. The query vector and candidate ranking still depend on the current query.

The hint store contains at most one norm per visited indexed document. It is lazy and may eventually cover the whole index. It adds memory rather than providing a zero-cost speedup. There is no persistent file or separate configuration: hints are derived work and never become authoritative knowledge.

Text edits, insertions, successful removals and `clear()` invalidate hints for the changed index. Any document-frequency change can affect every TF-IDF norm, so a local edit deliberately clears that index's complete store. Metadata-only updates preserve norms; failed updates or removal of an unknown ID leave valid hints intact. An index clone may share already-initialized hints while statistics match. Changing the clone invalidates only its derived store, leaving retained readers correct.

Queries use nonblocking read/write attempts. If another thread holds the hint lock, or a previous panic poisoned it, the query computes the exact value instead. It never waits for that hint lock. Simultaneous first queries may repeat a calculation; this favors predictable progress over forcing all readers to wait.

With `execution`, a cold norm calculation checks cancellation/deadlines within the term loop and before storing the completed value. A partial sum is never cached. Existing per-candidate and operation checkpoints remain. Cancellation stays cooperative, not a hard real-time or process-memory guarantee.

## Use the existing APIs

Keep one `Engine` alive across requests when possible. `search()` uses BM25; `search_with_options()` selects `SearchAlgorithm::Tfidf`. No warm-up call is required. Use `upsert()` and `remove()` for changes. Result entries still borrow the index. JSON serialization retains original records in deterministic ID order.

The 0.4.1 and 0.4.2 cache files cross-load through the public cache API. Their analysis/metadata payloads agree after the expected producer label. The label includes the crate version, so whole-file checksums differ. The private TF-IDF hints are not serialized. Knowledge JSON, document revisions, source fingerprints and reserved descriptors keep their existing formats.

MCP uses this core automatically. It remains optional and in the same crate. Its [response schemas](MCP.md#response-schemas) are independent of the internal memory layout. Default core/MCP normal dependency checks exclude the test validator, heap profiler, HTTP, SQLite and native model packages.

## Measured scope and remaining costs

See [the release report](CHANGES-0.4.2.md) and [machine-readable measurements](measurements/0.4.2/). Separate programs measure ordinary release calls, server process RSS and allocator-instrumented index heap. These metrics describe different boundaries and must not be treated as interchangeable.

Repeated TF-IDF queries over stable data are the intended beneficiary. A first query still performs uncached calculations. Frequent text edits discard that work; lock contention can repeat it. Small records can pay additional reference-count/tree overhead. BM25/matching do not gain TF-IDF reuse. The disk-backed index and optional model algorithms have their own implementations. A lower measured index heap is not a hard process-RAM ceiling or a guarantee for every corpus.

Exact score-bit/order comparisons cover fixed real-text probes, updates/deletes, retained views and cache interoperability. They establish compatibility for those cases, not human relevance quality. The unchanged PDF backend is tested separately; this release does not re-extract the original PDFs or add OCR. No DenkMaps code or database is integrated.
