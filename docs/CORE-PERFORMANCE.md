# Core performance and bounded retrieval

AIRust 0.4.25 keeps Core source byte-for-byte unchanged from 0.4.24. Its MCP knowledge loader releases the owned raw input after parsing and before constructing the index. Entry values remain owned and source files unchanged. Startup RSS, retained heap, allocation traffic and search times are separate measurements; read [the full report](CHANGES-0.4.25.md).

AIRust 0.4.4 reduces temporary allocation traffic during index construction. It keeps the construction/ranking design from 0.4.3 and the shared strings/TF-IDF reuse introduced in 0.4.2. Applications keep the same `Engine`, entries and search methods. There is no model to download or new default feature to enable.

## What is shared

Public `KnowledgeEntry` fields remain owned ID/text strings and optional metadata. The index privately shares immutable strings for vocabulary words, document-map/posting/duplicate-group IDs and identical normalized text. A vocabulary word is stored once across postings and document term maps. Original source text is preserved. Normal insertion groups a bounded sorted prefix of borrowed tokens, then streams the remaining tokens into the shared term map. It does not construct an intermediate owned-string token map.

This does not turn the whole index into one copy of the corpus. Source text, normalized text, tree nodes, counts and metadata still consume memory. Distinct normalized documents remain distinct. `shared-index` additionally shares complete immutable document/posting blocks between index clones; the default clone still copies containers while sharing their immutable strings. Later clone edits retain their existing copy-on-write behavior.

## Bounded construction and vocabulary lookup

A private TokenBatch value owns at most 1,024 borrowed token/count pairs. It sorts by word and merges repeated counts in place, preserving the number of original token occurrences separately. The completed pairs are consumed into the existing ordered per-entry term map. Remaining tokens stream through the original counter path, including terms already seen in the prefix. Original text, Unicode normalization and separators retain their existing behavior.

Reservation uses a bounded estimate of normalized text bytes. Empty token streams allocate no batch storage. A single token reserves one slot even when its text is very long. Multi-token storage can grow only to the fixed prefix bound. No scratch buffer is retained in KnowledgeIndex or between documents, so one unusual entry does not increase every later index clone. Tree construction, normalization and source storage still allocate. The measured Rust compiler can reuse the consumed pair allocation during collection; that is a standard-library implementation detail, not an AIRust API or correctness requirement. No unsafe memory conversion is used.

The byte estimate can reserve more slots than needed when an entry contains only a few very long words. Such entries can request more cumulative working bytes than 0.4.3 despite making fewer allocation calls. The temporary buffer remains bounded to 1,024 pairs and completed live storage stays unchanged in the measured cases. See the authored [allocation distributions](measurements/0.4.4/heap-edges.json) alongside the real-document profile; allocation count, allocated bytes and peak memory describe different costs.

KnowledgeIndex remains the single owner of lexical state. TokenBatch only prepares counts; every insertion still reaches the same prepared-insertion path and existing vocabulary/duplicate/statistics maintenance. Cache restoration and advanced analyzers supply their already-prepared counts to that path. No second index, global mutable pool, public option or required dependency is introduced.

`Engine` is a public alias for `KnowledgeIndex`, not another index wrapper. The private value types encapsulate preparation and ranking work; the authoritative state stays in one place.

```mermaid
flowchart LR
    entries[Normal entries] --> batch[Temporary TokenBatch]
    batch --> index[Engine = KnowledgeIndex]
    prepared[Cached or analyzed counts] --> index
    application[Application or optional MCP] --> index
    index --> state[Documents, vocabulary, postings and duplicates]
    index --> hints[Derived TF-IDF norm hints]
```

The global vocabulary dictionary uses Rust's default randomly seeded `HashMap`. AIRust does not install a faster unprotected custom hasher. Entries, postings IDs, per-entry term order, duplicate groups and stable score/ID tie order retain their ordered structures. The global dictionary is used for lookup, not to enumerate results or serialized data. Hash seeds may affect costs/layout, not the ranking contract. See [Rust's standard map contract](https://doc.rust-lang.org/std/collections/struct.HashMap.html).

## Query preparation and winner explanations (since 0.4.3)

Known query words are prepared in their original sorted order. BM25's corpus-frequency logarithm is calculated once for each word, and document-length normalization once for each candidate. Score multiplication/division and sorted contribution order remain the same, including the existing overflow-resistant branch for large `k1`. Repeated query words retain the previous BM25 behavior. Unknown words produce no candidate/statistic entry.

In 0.4.3–0.4.9, candidate collection used an ordered ID set proportional to the union of postings. Starting with 0.4.10, smaller queries use the ordered merge described below; the wider-query fallback retains the ordered set. The top-K heap still retains only borrowed documents and scores. Only selected winners receive owned matched-term strings. Asking for every result still pays for every returned explanation. No result text is cut or omitted by this optimization.

Controlled calls check cancellation/deadlines inside long scoring and explanation loops as well as the existing phases. Selection, scopes, final ordering and progress completion retain their existing contracts. The separate BM25 oracle exercises extreme valid parameters and compares exact score bits; term batching is checked on both sides of the boundary and on long Unicode streams.

## TF-IDF reuse (since 0.4.2)

BM25 follows its existing path and creates no TF-IDF norm hints. A nonempty TF-IDF candidate set initializes a small shared hint store. Each visited candidate's complete document norm is calculated using the unchanged sorted-term formula. Later queries reuse that value. The query vector and candidate ranking still depend on the current query.

The hint store contains at most one norm per visited indexed document. It is lazy and may eventually cover the whole index. It adds memory rather than providing a zero-cost speedup. There is no persistent file or separate configuration: hints are derived work and never become authoritative knowledge.

Text edits, insertions, successful removals and `clear()` invalidate hints for the changed index. Any document-frequency change can affect every TF-IDF norm, so a local edit deliberately clears that index's complete store. Metadata-only updates preserve norms; failed updates or removal of an unknown ID leave valid hints intact. An index clone may share already-initialized hints while statistics match. Changing the clone invalidates only its derived store, leaving retained readers correct.

Queries use nonblocking read/write attempts. If another thread holds the hint lock, or a previous panic poisoned it, the query computes the exact value instead. It never waits for that hint lock. Simultaneous first queries may repeat a calculation; this favors predictable progress over forcing all readers to wait.

With `execution`, a cold norm calculation checks cancellation/deadlines within the term loop and before storing the completed value. A partial sum is never cached. Existing per-candidate and operation checkpoints remain. Cancellation stays cooperative, not a hard real-time or process-memory guarantee.

## Use the existing APIs

Keep one `Engine` alive across requests when possible. `search()` uses BM25; `search_with_options()` selects `SearchAlgorithm::Tfidf`. No warm-up call is required. Use `upsert()` and `remove()` for changes. Result entries still borrow the index. JSON serialization retains original records in deterministic ID order.

The 0.4.3 and 0.4.4 cache files cross-load through the public cache API. Their analysis/metadata payloads agree after the expected producer label. The label includes the crate version, so whole-file checksums differ. The private TF-IDF hints are not serialized. Knowledge JSON, document revisions, source fingerprints and reserved descriptors keep their existing formats.

MCP uses this core automatically. It remains optional and in the same crate. Its [response schemas](MCP.md#response-schemas) are independent of the internal memory layout. Default core/MCP normal dependency checks exclude the test validator, heap profiler, HTTP, SQLite and native model packages.

## Measured scope and remaining costs

See [the release report](CHANGES-0.4.4.md) and [machine-readable measurements](measurements/0.4.4/). Separate programs measure ordinary release calls, server process RSS and allocator-instrumented index heap. These metrics describe different boundaries and must not be treated as interchangeable.

Repeated TF-IDF queries over stable data are the intended beneficiary. A first query still performs uncached calculations. Frequent text edits discard that work; lock contention can repeat it. Small records can pay additional reference-count/tree overhead. BM25 uses query-local preparation rather than a persistent norm store; matching does not gain TF-IDF reuse. The disk-backed index and optional model algorithms have their own implementations. A lower measured index heap is not a hard process-RAM ceiling or a guarantee for every corpus.

Exact score-bit/order comparisons cover fixed real-text probes, updates/deletes, retained views and cache interoperability. They establish compatibility for those cases, not human relevance quality. The unchanged PDF backend is tested separately; this release does not re-extract the original PDFs or add OCR. No DenkMaps code or database is integrated.


After deletions, an empty vocabulary releases its hash allocation. A large dictionary is compacted when its word count falls below a quarter of the retained capacity; a separate high-water value avoids relying on deleted-slot bookkeeping. Small tables keep limited headroom. This adds an occasional rehash cost to deletion, can temporarily hold old and replacement tables, and does not alter retained reader contents. The dictionary, including this capacity hint, remains derived internal state and is not serialized.

## Query working allocations in 0.4.8

`text::term_frequencies` retains one owned key per distinct normalized token. After the first repeated token is observed, later repeats update the existing count through a borrowed lookup, avoiding temporary duplicate token strings. The first duplicate still creates one temporary key; an entirely unique input keeps the existing single entry-lookup path. NFKC/lower normalization, counts, term ordering and all scorer arithmetic remain unchanged. Once reuse is enabled, a new term needs an additional lookup, so mixed distributions can have a different latency tradeoff.

The optional `LexicalCoverageSource` reuses invariants of the existing built-in Engine: prepared question terms are unique, and each returned matched term occurs once in that effective question. Counts no longer require a new query set or a matched-term set per candidate. Original scores, component diagnostics, stable ordering, recipes and source positions remain. This optimization does not apply those assumptions to custom sources. See [coverage API](LEXICAL-COVERAGE.md) and [executed measurements](CHANGES-0.4.8.md).

## Complete retrieval and bounded serialization in 0.4.9

Legacy excerpt analysis uses direct lowercase ASCII tokenization when the complete input is ASCII. The normalized-byte budget is checked before collecting tokens, and token limits remain enforced. Original byte spans are unchanged. Mixed text still uses grapheme normalization and source mapping; its ASCII graphemes avoid unnecessary Unicode normalization. Compatibility expansions, combining marks and CRLF retain their existing rules. Advanced TextAnalyzer behavior is unchanged.

Size-only candidate, metadata and MCP guards share the bounded serializer with retained JSON output. A counting sink discards encoded bytes instead of keeping a temporary JSON buffer. Counting includes quotes, escapes and UTF-8 bytes, and stops on the same budget boundary. Serialization errors still fail explicitly. Actual responses and descriptors are still encoded and stored when needed. These internal helpers are not a new public API, cache or storage format. No option, dependency or default feature is added. Less working allocation is not automatically lower resident RAM or faster construction. Read the [measurements, regressions and limits](CHANGES-0.4.9.md).

## Index normalization and ordered candidates in 0.4.10

`text::normalize` takes a direct path only when the complete source is ASCII. ASCII A–Z are lowercased and the same whitespace characters collapse to one space, including vertical tab. Leading/trailing whitespace disappears as before. Controls that are not Unicode whitespace remain. Non-ASCII input retains the full NFKC/lowercase/NFKC path. Source entries and all persistent index structures remain.

Queries with at most 32 known distinct terms merge sorted postings using a small private min-heap. Duplicate IDs are skipped into a sorted vector borrowing the existing Arc key objects, with no cloned identifiers. The heap is bounded to 32 cursors; it does not retain entry text or rebuild the index. Wider known-term queries use the prior BTreeSet route. Unknown/repeated terms do not inflate the merge width. The query descriptor retains one additional postings reference per known term. The vector can reserve spare capacity; allocation counts and bytes have different meanings.

Each posting occurrence, including duplicates, still reaches a cancellation/deadline checkpoint. Collection visits and final unique counts remain observable. Scoring stays in original sorted ID order and original sorted term order, with the same arithmetic, context checks, top-K ties and winner explanations. The count/shape of timing-dependent progress notifications is not promised. No dependency, public option, cache or storage format is added. See [construction, allocation and MCP measurements with limits](CHANGES-0.4.10.md).

## TF-IDF and broad queries in 0.4.11

The transient norm-hint dictionary uses Rust's default randomly seeded HashMap instead of BTreeMap. The dictionary is used only for lookup, never to enumerate ranking contributions or exports. Complete norms are published after checkpoints. Metadata-only updates retain hints; lexical/corpus changes invalidate them. Busy or poisoned locks still fall back to exact computation without blocking. The table can allocate/reallocate and reserve more or fewer bytes; it is not an unconditional RAM optimization. BM25/unknown queries still avoid allocating the norm cache.

For more than 32 known query terms, when the query exceeds twice a document's distinct word count, scoring scans its sorted words and binary-searches the sorted prepared query. Other cases keep query-word lookups. Both routes accumulate matching terms in the same lexical order and keep exact arithmetic. Each 128 inspected terms reaches a checkpoint. The wider candidate BTreeSet now borrows existing Arc keys instead of wide string slices, preserving sorted unique IDs. No second index, dependency, public flag or storage migration is added. See [initial/warm timings, scratch allocations and retained hint memory](CHANGES-0.4.11.md).

## Cold TF-IDF hints in 0.4.12

The norm cache stores one machine-word key identifying an ID allocation already owned by this KnowledgeIndex. On a 64-bit build this replaces a two-word Arc string key and removes extra ID-reference count updates and string hashing. The standard randomly seeded HashMap remains; there is no custom unprotected hasher. The integer is never dereferenced, serialized, used as a public ID or accepted from a caller. Public identifiers and tie ordering remain original strings.

Unchanged clones retain the ID allocations for their shared valid hint cache. Metadata-only edits retain the same ID key; text/corpus edits, removal and clear discard the changing view's hints before its next query. Retained views keep their own valid data. Loading or rebuilding creates a fresh empty cache. Existing complete-value publication, nonblocking lock/poison fallback, cooperative checkpoints and exact norm arithmetic remain. No per-document field, eager corpus-sized allocation, dependency, public setting or storage migration is added. BM25 and unknown-word queries still do not allocate norm hints.

Measure retained hint bytes after dropping results separately from allocation traffic, warm calls and whole-process RSS. Reduced hint storage does not imply the same reduction in total application memory. See [all cold/warm/RAM results and limits](CHANGES-0.4.12.md).

## Cold TF-IDF requests in 0.4.13

When a document norm is missing, one query can reuse exact word IDF values in a 256-slot direct-mapped memo. Its key is the identity of an interned term allocation owned by the borrowed index. Check the full key before reuse; a collision replaces a slot after recomputing the original IDF formula from the current document frequency. Collision work remains bounded by the existing exact computation. Keys are never dereferenced, persisted or accepted as public IDs. There is no new unsafe code or custom hash dictionary.

The memo is allocated lazily after the first cold norm checkpoint and is freed on completion, cancellation or failure. On the measured 64-bit build it adds a 4,096-byte Box plus existing allocation overhead/temporary stack costs. BM25, unknown queries and complete warm norm hits allocate no memo. Persistent norm storage, corpus invalidation, metadata reuse, shared-clone ownership and nonblocking lock/poison fallback remain. Sorted term order, arithmetic and checkpoints every 128 inspected norm terms stay exact. Startup does not compute or prewarm document norms. No index field, dependency, feature flag, public option or storage migration is added.

Cold gains must be weighed against this temporary allocation and unchanged/slower warm cases, especially on tiny collections. Read [first/repeated MCP, tiny controls, heap and RSS](CHANGES-0.4.13.md).

## Warm and small TF-IDF work in 0.4.14

A request-local norm reader retains one successful try_read guard for at most 128 cache hits. It releases the guard on a miss, at the batch boundary, before a progress callback and before winner explanations. Cache-hit dispatch remains small; exact cold norm computation is a separate non-inlined helper. No cache guard escapes the synchronous query or becomes an index field.

Misses release the query's read guard before computation and try_write publication. Another reader or a poisoned/busy lock can still prevent caching a computed norm; the exact result is returned without waiting. A later quiescent query can populate the hint. Native regressions cover bounded release, misses and overlapping warm/cold calls. This is a best-effort optimization, not a lock-free/fairness or process-time guarantee.

Cold word weights first use eight inline slots (128 payload bytes on the measured 64-bit build). After at least 128 terms from earlier completed cold norms, a later norm can promote to the existing 256-slot / 4,096-byte heap memo, preserving surviving exact weights. Neither warm hits nor interrupted work contribute cold-work credit. Both memos compare full live term-allocation identities; collisions recompute exactly. Small requests can avoid the immediate 4 KiB allocation. Inline state/control fields and temporary frames still cost stack space; heap profiles do not measure it.

Sorted term order, floating-point operations, checkpoints, scope, mutation invalidation, shared-clone ownership, source positions and persistent norm storage remain. Startup does not prewarm norms. No dependency, public option, feature flag or storage migration is added. Read [all small/warm/cold/mixed measurements](CHANGES-0.4.14.md).

## Small MCP transport execution in 0.4.15

Core algorithms and memory structures are unchanged. The optional MCP handler removes the awaited initial progress tick and selects worker completion/cancellation/deadline immediately. Periodic updates start one configured interval later; requested start/completion remain. The change applies to both rankers and source/matching/relation work without a new option. Compare complete MCP round trips separately from microsecond Core work and heap traffic. Larger inputs, slower results, startup, tails and RAM remain in [both campaigns and controls](CHANGES-0.4.15.md).

## Large small-term queries in 0.4.16

The candidate union and document lookup remain. With at least 1,024 candidates and 1–4 known query terms, after 128 candidates pass context/custom filters the query can create a small Vec of posting-frequency cursors. Each reads exact per-document term frequency from its existing sorted posting map in original query-term order. A live Arc ID identity shortcut has an exact string fallback. After 32 skipped entries the cursor checkpoints and seeks a lower bound instead of linearly walking a long gap.

All original checkpoints and floating-point additions remain. Cursor state is query-local and dropped on error/completion. It adds temporary allocation traffic to eligible large calls, with no persistent index field or per-document cache. Small/narrow/wider-term queries keep the prior path. Filtered gaps may pay seek overhead; no universal speedup is promised. The existing IDF memo, norm cache, invalidation, retained clones and source bindings remain. A slower ordered-document prototype was rejected. Read [both ordinary campaigns, direct controls, three-version reference and heap/RAM](CHANGES-0.4.16.md).

## Single known-term queries in 0.4.17

A prepared query with exactly one indexed term already has one sorted, unique posting list. Candidate collection now reads that list directly, calling the same checkpoint/progress visitor on every ID. It avoids constructing the heap used to merge several lists. The borrowed-ID Vec initially reserves the smaller of 32 and the posting count; it can grow for larger candidate sets, just as before. The initial bound is not a total candidate or RAM bound. Repeated/unknown words can still leave exactly one known term; query preparation and scoring retain their existing semantics.

Multiple known terms keep the heap/set strategies. Candidate order, filters, original score arithmetic, norm hints, winner explanations and large-query frequency cursors remain. No scratch is retained by the index and no unsafe code, field, dependency or public option is added. Existing checkpoints and cancellation/deadline precedence remain. Direct Core, full retrieval and MCP transport have different costs: read [both fresh three-version small campaigns, the 96-case extension, allocation controls and all larger/slower cases](CHANGES-0.4.17.md).

## Two known-term queries in 0.4.18

For exactly two known indexed terms, compare the current IDs of both already sorted posting lists. Append the lower ID, or append once and advance both lists when equal. A live Arc identity shortcut uses exact string order when allocations differ. Both sides of an overlap still invoke the same posting visitor, as does every remaining tail ID. No cancellation/deadline check or progress visit is removed. More than two terms retain the previous heap/set strategies; one term keeps the 0.4.17 direct path.

The private helper uses an inline-never boundary to keep merge iterator work outside the caller. The candidate Vec initially reserves min(32, the larger posting count) borrowed slots and can grow with input. Two temporary iterator states replace the pair's temporary heap. There is no persistent index field, new cache, dependency or public option. Heap profiles exclude iterator/stack state; fewer temporary allocations do not imply a total RAM bound. Original scoring/frequency reuse, filters, ties and explanations remain. Read [two ordinary MCP campaigns, direct controls, fresh three-version references, all small guards and slower cases](CHANGES-0.4.18.md).

## Broad-query explanations in 0.4.19

For more than 32 known terms and more than twice the winner document vocabulary, inspect sorted document words and binary-search the prepared query. Otherwise scan prepared query words against the document dictionary. Scoring fallback and winner matched-term construction share this exact intersection; frequency-cursor scoring is unchanged. Both dictionaries are sorted, keeping matched-term order and score contribution order. No extra persistent storage or public API is added.

Explanation checkpoints occur before each winner and at every 128 actually inspected words, including the first. Sparse scans do less work and can therefore make fewer explanation checks; long scans remain checked. Failure drops the partial vector. Posting and scoring/norm controls remain. Dense documents and short queries need not improve. Read [three broad campaigns, complete-response checks, Core guards, allocations, host load and all slower cases](CHANGES-0.4.19.md).

## Ordered document lookup in 0.4.20

Candidate IDs are sorted and unique before ranking. A request with at least 1,024 candidates uses direct lookup for the first 128, then starts a private DocumentCursor at the current ID. Compare exact IDs, using Arc identity only as a shortcut for live immutable allocations. Return the existing map key/document and advance once when found. Read nearby lower keys; after eight, check cancellation/deadlines and seek directly to the target. Original per-candidate checkpoints and filters remain. No additional progress visit is counted for skipped map keys.

The cursor borrows the immutable map and owns no heap storage. Persistent index layout, norm-cache identity and saved formats remain. Query-local range/iterator stack state is excluded from heap profiles. Sparse gaps can add work; this heuristic is not optimal for every distribution. Native tests compare independent lookup, Unicode/equal IDs, gaps, exhaustion, recovery and large-index hand calculations. Read [two large-corpus campaigns, direct controls, broad/small guards, heap costs and all slower observations](CHANGES-0.4.20.md).

## Borrowed single-word preparation in 0.4.21

A raw query made only of a-z and 0-9 is already one unchanged normalized token. Borrow it and yield frequency 1 through a private iterator variant, avoiding a normalized String, owned map key and temporary frequency-map node. The existing sorted-map iterator supplies all other/preanalyzed queries in the same order with identical counts. Result matched terms remain owned, and entry references remain tied to the Engine. Uppercase, spaces, punctuation, Unicode and repeated-word input keep original analysis. No new cache or stored field is added.

The shared checked ranking body keeps all previous checkpoints and progress visits, exact arithmetic, scopes, norm hints, explanations and source positions. The 0.4.20 document cursor remains. Tests compare independently normalized words, hand-calculated scores, ties, metadata filters and the 1,023/1,024/1,025 boundaries, plus exact checkpoint parity and recovery. Thirty finite Miri cases include scaled versions of all three new tests. Read [primary small Core comparisons, rejected trials, heap deltas and all MCP guards](CHANGES-0.4.21.md). Fewer query allocations do not establish a total process-RAM bound.

## Repeated-word preparation in 0.4.23

For at most 4,096 bytes of repeated unchanged lowercase ASCII alphanumeric words, borrow the first word and yield its exact count through a private iterator. Check the first two words for equality before canonical validation, and reject the optimization if any later word differs. The scan has a fixed byte bound and constant auxiliary memory. Different/longer/noncanonical input and preanalyzed maps use original normalization and sorted counts. This is not a new public Core query-size limit. The existing single-word path remains, and search results own their matched strings.

The shared checked ranking body keeps the same word/count order, arithmetic, candidate collection, norm hints, scopes, explanations, source positions and checkpoints. Three independent native regressions cover normalization/frequencies, exact guards, score/filter/limit boundaries and controlled dispatch, plus exact checkpoint/error/recovery parity. Scaled cases join the previous 33 Miri cases, making 36. Bundled MCP uses preanalyzed retrieval; its implementation source is unchanged. Compare all fallback/slower rows rather than infer a universal gain. Read [all Core/MCP and heap controls](CHANGES-0.4.23.md). Temporary allocation traffic and total process RAM remain different measurements.
