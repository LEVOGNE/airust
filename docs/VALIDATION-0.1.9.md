# Additional validation of AIRust 0.1.9

Date: November 16, 2026.

The published AIRust 0.1.9 package passed these additional checks. No engine defect was found in the checked cases. This report adds evidence and documents limits; it does not introduce a new crate version.

The AIRust library and application source remain unchanged. No DenkMaps or MCP integration was implemented. The report, a separate test consumer, public fixtures, and aggregate measurements are now part of this repository. Private map text and databases are excluded.

## What was checked

The test consumer used `airust = { version = "=0.1.9", default-features = false }` from the crates.io registry. Cargo metadata confirmed the registry source. It also compiled with `rusqlite = "=0.41.1"`, the version used by the local application. AIRust's optional SQLite and web features were disabled.

The tests ran on macOS ARM64 with 64 GiB physical memory and Rust 1.98.0. The published crate's complete default-feature test suite passed: **187 tests**, including its README example. The test consumer used Cargo's default release profile. This assessment did not repeat all optional web/PDF tests or perform a new security audit. The earlier release checks remain recorded in the [0.1.9 change report](CHANGES-0.1.9.md).

| Area | Result | What this establishes |
| --- | --- | --- |
| Existing data | Read 722 entries from three local maps containing 500, 198, and 24 nodes. Text and notes totaled 258,584 UTF-8 bytes. | The supplied records can be indexed. It is not a relevance-quality score. |
| Search and JSON | Ran 100 in-memory term queries. BM25 and TF-IDF results matched after a JSON round trip. | Source-entry export and rebuild preserved the checked rankings and scores. |
| Updates | Changed or removed 200 entries. Compared statistics, both rankers, duplicate groups, fuzzy results, and relation proposals with a fresh rebuild. | Incremental operations matched a rebuild in the checked workload. |
| SQLite replay | Ran 300 steps in a separate in-memory SQLite database, including replacements, deletions, committed writes, and rollbacks. | A caller that replays committed changes can maintain an equivalent index. AIRust does not monitor SQLite itself. |
| Scope | Queried with a collection metadata filter and checked every returned entry. | Search honored the selected filter. Application authorization was not audited. |
| German text | Equivalent composed and decomposed umlauts matched exactly. `Kfz-Zulassnug durchführen` scored 0.92 against `Kfz-Zulassung durchführen`. | Unicode normalization and this spelling-error case worked. This is not a German-language quality benchmark. |
| Meaning | The vehicle-registration synonym pair shared no tokens and produced no relation proposal. | The lexical engine does not infer those synonyms. |
| Opposing statements | Positive and negative registration statements could receive a `Related` proposal. | Shared words do not establish agreement, a prerequisite, or a contradiction. |

The local corpus was read through a read-only SQLite connection. Application source, database, and running service were not modified. The test database was separate and in memory. Private map text is excluded from this report and its accompanying files.

The 100 queries were the first 100 tokens of at least four characters encountered in the corpus. Repeated tokens were retained. These corpus-derived queries check that terms can be retrieved; they are not an independent query set with labeled relevance judgments. There was no separate warm-up loop for this real-corpus timing. The historical JSON fields named `warm_query_count` and `warm_query_total_ms` retain their original names. The metadata filter was checked for returned entries. This does not establish the application's permission model.

The real-corpus run built the index in 22.461 ms and executed those 100 queries in 8.955 ms total. These are single-run engine timings after export. They exclude reading SQLite, constructing application records, network calls, rendering, and the existing application's search. They must not be described as a measured application speedup.

## Scale measurements

Each size used synthetic five-token German records, including umlauts, common terms, and 10,000 bucket terms. Each query had one warm-up followed by twelve samples. Limit: ten results. The common query matched every record; the rare query matched about 1, 10, and 100 records respectively.

| Entries | Build (ms) | Rare query median (ms) | Common query median (ms) | Peak RSS (MiB) |
| ---: | ---: | ---: | ---: | ---: |
| 10,000 | 24.257 | 0.000604 | 2.737 | 25.62 |
| 100,000 | 209.020 | 0.001896 | 28.509 | 187.50 |
| 1,000,000 | 2,370.228 | 0.018708 | 313.901 | 1,914.19 |

Rare query: `bucket7919`. Common query: `Wissen Straße`. Miss: `nichtvorhandenerbegriff`, which returned no results. Raw samples are in [the results directory](validation-0.1.9/results/), named `scale-*.json`. Process resource measurements are named `time-*.txt`. Peak RSS covers the complete process, including the index, query work, and allocator overhead.

These are single-process runs with a small repeated query set. They are not production capacity guarantees, application latency measurements, or tail-latency estimates. The synthetic text is much shorter than many real documents. Different consumer build profiles and query workloads prevent direct comparison with the earlier release benchmark.

## Application investigation and integration status

The application inspection below was read-only. It used the local DenkMaps source and an export of its current records. It did not change its dependencies, data storage, service, or search implementation.

| Step | Status | Remaining work |
| --- | --- | --- |
| Inspect SQLite tables and node structure | Inspected the map schema, tree shape, and record counts. | A full application and authorization review remains outside this assessment. |
| Compare existing search with AIRust | Compared the algorithms in source code. | Run both implementations on the same queries and label relevance. |
| Integrate AIRust while preserving storage | Not implemented. | Define and test an application-owned adapter and synchronization policy before integration. |
| Test real maps | Tested an exported 722-entry corpus in a separate AIRust consumer, including incremental changes. | Measure search quality and end-to-end performance inside the application. Test crash recovery and concurrent updates. |
| Optimize MCP and TOON | Not implemented. | Evaluate only after a separately authorized and tested application integration. |

This work does not approve a production integration. No comparative application benchmark, labeled retrieval-quality study, or live database synchronization test was completed.

## Decisions before an application integration

The inspected application stores map trees as JSON in SQLite, rather than one independent SQL row per node. Its current search requires every query term, checks substrings, and gives node titles more weight than notes. It also folds German umlauts and uses simple stemming. AIRust's default BM25 search accepts overlapping tokens, preserves accents, and does not stem. A replacement must deliberately choose and test the intended behavior.

An application should own durable data, user permissions, namespaced record IDs, and synchronization. It should validate changes, commit them, then update the index. A crash between commit and index update needs a rebuild or an application-owned durable replay mechanism. The replay test above does not establish crash recovery or multi-process consistency.

`Context` filters search results. `find_matches()`, `duplicates()`, and `relate()` do not accept that filter. Applications must supply an appropriately scoped engine or otherwise enforce their authorized record set before using these operations. No adapter or permission model was added here.

German transliterations such as `Straße`/`Strasse` and `Führerschein`/`Fuehrerschein` are not exact duplicates under the current normalization contract. Relation scores measure shared words. They must not be presented as semantic certainty or saved automatically as logical facts.

An optional embedding-provider interface remains future work. It should stay independent of a particular model service. Semantic relevance needs labeled examples before any quality claim. No embeddings, model downloads, provider client, structural adapter, MCP output, or TOON format was introduced.

## Reproduce the public checks

The [consumer directory on GitHub](https://github.com/LEVOGNE/airust/tree/master/docs/validation-0.1.9/consumer) contains a separate test program, its lockfile, and a small public sample corpus. The sample allows the checks to run without private data. It cannot reproduce the original 722-entry corpus or its timings. The consumer depends on the published registry version, rather than the current source checkout, and adds no dependencies to AIRust itself. This nested test package is available in the repository source checkout; it is not included in the AIRust crate archive.

From the repository root:

```bash
cd docs/validation-0.1.9/consumer
cargo run --locked --release -- sample-corpus.json
cargo build --locked --release
/usr/bin/time -l target/release/airust-qualification-019 scale 10000
/usr/bin/time -l target/release/airust-qualification-019 scale 100000
/usr/bin/time -l target/release/airust-qualification-019 scale 1000000
```

The resource command uses macOS `/usr/bin/time -l`. Other systems need their own process-memory measurement. JSON latency and build measurements come from the test program itself.

For another corpus, pass a JSON array of entries with `id`, `text`, and optional `metadata`. The collection-filter check uses a numeric `collection` metadata field. The harness contains assertions and exits unsuccessfully when a check fails. German strings are intentional test data; program text and documentation use English.

[qualification-published.json](validation-0.1.9/results/qualification-published.json) contains the aggregate real-corpus results. [published-tests.txt](validation-0.1.9/results/published-tests.txt) records the published crate's default-feature test run; local filesystem paths have been redacted. The public consumer uses a generic corpus label when run on other data. The recorded historical results retain their original labels.

The additional checks do not establish semantic understanding, token savings, production synchronization, or a completed security audit.
