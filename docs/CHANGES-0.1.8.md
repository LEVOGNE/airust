# AIRust 0.1.8: changes and purpose

Historical release report dated November 16, 2026. Version 0.1.8 was published from commit `8599d37bcb80251dafeed09f8b82e37d830dddd1` after the checked changes were attached to the original GitHub history.

AIRust became a knowledge library first. It retrieves text, compares content, and proposes lexical connections. Applications can use the core without a dashboard, web server, or language model.

## The shared core

| Change | Purpose |
| --- | --- |
| `KnowledgeEntry` with ID, text, and optional metadata. | Accept application-owned records from any Rust caller. |
| One shared `Engine` index. | Keep retrieval, duplicates, and relation candidates consistent. |
| BM25 and a distinct TF-IDF mode. | Rank lexical matches with either scoring method. |
| `upsert()` and `remove()` by ID. | Update one entry without rebuilding unrelated postings. |
| Shared Unicode processing. | Preserve numbers, split punctuation boundaries, and count characters consistently. |
| Exact, fuzzy, and token-overlap matching. | Expose different kinds of comparison evidence. |
| Duplicate groups. | Review source text and metadata before deciding to merge or delete. |
| Explicit `Context` filters. | Restrict eligible IDs and metadata without rewriting the query. |
| Explained `relate()` suggestions. | Show shared terms and leave relationship decisions to the application. |
| JSON import and export. | Save source entries and reconstruct derived indexes. |

No-match queries return an empty list in the new API. Invalid entries are rejected before mutation. Duplicate import IDs are errors. Score ties use entry IDs.

Word overlap does not prove a prerequisite, hierarchy, or contradiction. Semantic providers remained optional future work.

## Dependency footprint

The previous default build enabled the dashboard, HTTP, SQLite, bots, and PDF processing. Version 0.1.8 made these integrations optional.

On the checked Mac, the normal dependency tree dropped from **256 to 23 packages**, including AIRust. The previous `--no-default-features` build still contained 120 packages. Counts used `cargo tree -e normal` and counted each package version once.

The all-features tree contained 286 packages after compatible dependency updates. The updated optional PDF library introduced additional processing dependencies. Those packages were absent from the default core.

Unused direct color, plotting, and old IndexMap dependencies were removed. `colors` and `plotting` remained empty compatibility feature names. The lockfile recorded the checked dependency versions. PDF support required at least the tested `pdf_oxide 0.3.78`; the project minimum was Rust 1.89.

## Corrected behavior

- The historical BM25 adapter reused cached corpus lengths and shared postings.
- Appending training examples updated new entries without retokenizing the existing corpus.
- Tokenization preserved numbers and separated words at punctuation.
- Fuzzy distance and thresholds used characters rather than UTF-8 byte counts.
- Agent confidence forwarding became consistent. Stored answer text could contain a fallback-looking phrase without becoming a false no-match result.
- Agent and conversation adapters forwarded metadata. HTTP results could include it.
- Mixed plain-string and structured training answers retained metadata.
- PDF provenance included file, page, and chunk information through conversion to `KnowledgeBase`.
- PDF configuration was validated. Oversized overlap produced an error. Long Unicode chunks respected their limit, and the final short passage was retained.
- Reused PDF regular expressions were initialized once. A valid high Unicode code point alone no longer marked text as damaged.
- A compatible SQLite metadata column preserved imported provenance through restarts.
- Exported dashboard training records could be imported again. Export field names were accepted, and JSON answer formats were retained.
- `merge_kb` excluded its output, removed completely identical examples, validated all sources, and replaced the destination through a temporary file.
- The build script wrote into Cargo's build directory rather than the source tree.
- Invalid main ports and missing port values were rejected before console-port arithmetic.

## Tools and documentation

The first rewrite replaced repeated multilingual guides with a shorter German guide. The subsequent 0.1.9 development work translates the project to English; this report is its English historical copy.

The 0.1.8 sample dataset retained 124 original questions and weights while correcting product descriptions, versions, search claims, and commands. The interface took its version from package metadata. Shipped translations were refreshed only if unchanged from their old defaults.

The web server bound to loopback by default, with `AIRUST_BIND_ADDR` for an explicit alternative. Blanket CORS was removed. The console retained fixed administrative commands rather than arbitrary shell execution.

Docker explicitly built the web feature and exposed both ports. Its build context included the guide embedded by the server and excluded the local database, archive, and logs.

A license file, migration guide, architecture description, changelog, and contributing guide were added. `test.sh` became a real validation script. GitHub Actions ran the same checks and compiled the minimum supported Rust version.

All Rust files were formatted consistently. Some older web files changed only because of formatting. Existing optional web vector-storage formats remained supported.

## Historical runtime comparison

The same example program was built against the preserved 0.1.7 source and version 0.1.8. It measured 1,000 training examples, 100 queries, and one appended example. Query terms appeared in all examples. Each version ran three times in release mode on the same Mac. Values below are medians without compilation time.

| Operation | 0.1.7 | 0.1.8 |
| --- | ---: | ---: |
| Initial index build | 0.794 ms | 1.711 ms |
| 100 queries | 228.691 ms | 17.529 ms |
| Append one example | 0.704 ms | 0.004 ms |

Search was about 13 times faster for this workload. Initial construction took about twice as long because more derived state was prepared. Append benefited from leaving existing entries indexed.

These were local example measurements. The new tokenizer also preserved numbers in the corpus. The comparison measured two complete versions, not an isolated algorithm. It did not establish general latency or capacity guarantees.

Repeat the workload with:

```bash
cargo run --locked --release --example benchmark -- 1000 100
```

## Recorded validation

The local release checks ran on ARM64 macOS with Rust/Cargo 1.98.0. Rust 1.89.0 compiled every feature and target. The original release's Linux GitHub checks also succeeded.

| Check | Recorded result |
| --- | --- |
| Original 0.1.7 | 168 passing tests; six Clippy warnings. |
| Default 0.1.8 library | 180 passing tests including the README example. |
| All 0.1.8 features | 200 passing tests including CLI, PDF, database migration, and the README example. |
| Formatting | `cargo fmt --all -- --check` passed. |
| Clippy for default, all features, and CLI alone | Passed with `-D warnings`. |
| Individual `pdf`, `bots`, and `async` library configurations | Compiled. |
| Generated documentation | Passed with `RUSTDOCFLAGS="-D warnings"`. |
| Executable core example | Passed, including duplicates, filters, and JSON round trip. |
| Rust 1.89.0 | Every feature and target passed `cargo check`. |
| Local HTTP behavior | Status, dashboard, guide, metadata, export/import, console handshake, and restart persistence passed. |
| Package | Built and verified without the database or backup. |
| Registry consumer | An independent temporary project installed 0.1.8 from crates.io and exercised the three core operations. |

The new regression tests included independent BM25 and TF-IDF reference calculations, incremental-index comparison with a fresh rebuild, Unicode boundaries, invalid settings, duplicates, provenance, and preservation of the merge destination after invalid input.

The original local validation log was `/private/tmp/airust-validation-0.1.8.log`. The minimum-version log was `/private/tmp/airust-msrv-1.89.log`. Temporary local paths are historical records and need not exist on another installation.

## Preservation and published artifacts

The original archive was retained at `backups/airust-v0.1.7-original.tar.gz`. Its checksum and original tree details are in [ANALYSIS-0.1.7.md](ANALYSIS-0.1.7.md). The original local `airust.db` file was unchanged by the release work.

The supplied tree matched original GitHub commit `052510449fe99a047e69e2dd6781a369498eedd1`. The release was built on that history rather than replacing it.

The published package checksum is `4d1fa32845369b3693e7d1c4d3c1803427d26f38d6db952775d178cef0e00ca0`. The registry package and GitHub release asset matched.

Published artifacts: [GitHub release](https://github.com/LEVOGNE/airust/releases/tag/v0.1.8), [crates.io version](https://crates.io/crates/airust/0.1.8), and [Linux CI](https://github.com/LEVOGNE/airust/actions/runs/43818149120).

No external application or MCP repository was changed. Docker was unavailable on the local machine, so the container build was not run there.
