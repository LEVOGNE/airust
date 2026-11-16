# AIRust 0.1.9: English and hardening

Release date: November 16, 2026. The baseline is the published 0.1.8 release. That original release and tag remain unchanged. Version artifacts are available through [GitHub Releases](https://github.com/LEVOGNE/airust/releases/tag/v0.1.9) and [crates.io](https://crates.io/crates/airust/0.1.9).

## English project surface

Project documentation uses English. Historical analysis, migration notes, and release reports have English filenames. Runnable examples and current interface text follow the same language. Unicode fixtures still exercise non-ASCII text because language-neutral processing is part of the library contract.

Source comments are removed as requested. Public contracts, configuration requirements, design decisions, and operating limits belong in the [API guide](API.md), [architecture guide](ARCHITECTURE.md), and [security guide](SECURITY.md).

## Persistence and input validation

`KnowledgeBase` validates finite, nonnegative weights on load and save. Invalid input is rejected before replacing stored data. Saving rejects symlink or non-file destinations, writes an exclusively created private temporary file, flushes and synchronizes it, preserves existing permissions, and then replaces the destination.

This gives complete file replacement. It does not add multi-writer transactions or merge concurrent updates.

## Bounded retrieval work

Search, matching, and relation proposals retain only the requested top results in a bounded heap. For limit K, retained result memory is O(K). Candidate IDs can still require O(M) memory for M candidates. Score and ID ordering remain deterministic.

Fuzzy retrieval uses a bounded Unicode edit-distance calculation to stop work for candidates that cannot meet the requested threshold. Reference-parity and top-result-prefix tests check that pruning and result selection preserve the expected matches and ordering.

## CLI and sample data

- Private runtime state moves from shared `/tmp` names to an absolute `AIRUST_RUNTIME_DIR`, defaulting to `$HOME/.local/state/airust`.
- Existing insecure or symlink runtime directories are rejected. Unix runtime mode is `0700`.
- `server.json` records PID, executable, and process-start identity. `stop` verifies the process before signaling it. Logs are private.
- Command-line parsing rejects unknown commands and malformed options. Query parsing stops at `cli`, preserving option-looking question text.
- `pdf2kb` validates before creating output directories, rejects identical input and output, requires `--overwrite` for an existing destination, supports `--`, and truncates previews on Unicode boundaries.
- The embedded sample dataset now contains 54 English questions. The generic example uses application-neutral research metadata.
- `examples/index_benchmark.rs` supports explicit entry and query counts and reports JSON timings and query percentiles.

## Optional web surface

- `AIRUST_API_TOKEN` is required on non-loopback bind addresses and optional locally. Configured tokens protect API and console access.
- Host and Origin checks use explicit allowed hosts and service ports. Cross-site browser requests are rejected. Ordinary nonbrowser clients remain supported.
- WebSocket credentials use the `airust` and `token.<token>` subprotocols rather than URL parameters. The browser retains its token only in page memory.
- Administrative file, SQL, knowledge load/save, and reset capabilities require `AIRUST_ENABLE_ADMIN=1`. They are disabled by default.
- Enabled filesystem administration rejects absolute, parent, hidden, and symlink paths and stays within the working directory.
- API tokens use 32–256 allowed ASCII characters. Queries are limited to 4,096 UTF-8 bytes; training input and serialized output to 1 MiB each.
- Imports are limited to 1,000 items, validate the complete batch, and use one SQL transaction.
- HTTP, file-read, WebSocket, console-command, SQL-pagination, and crawler boundaries have explicit limits.
- Crawlers accept eligible public HTTP/HTTPS destinations, validate and pin DNS results, ignore proxies, disable redirects, and bound depth and page count.
- Settings changes require an explicit `/settings` command prefix or the existing settings API.
- The active interface is English. Existing local data and customized database rows are retained.

The [migration guide](MIGRATION-0.1.9.md) records upgrade steps. The [security guide](SECURITY.md) states configuration rules and operating limits. These changes are not a comprehensive independent production security audit.

## Dependency audit

The dependency check used `cargo-audit 0.23.0` and RustSec advisory database commit `02e2517f457a359fc6d3440ea4560c8ea188fee8`, dated November 15, 2026.

The initial audit reported `RUSTSEC-2026-0459` through `libsqlite3-sys 0.28`. Updating the optional database integration to `rusqlite 0.43` and `libsqlite3-sys 0.40` resolved that advisory. Updating `scraper` to `0.29` also removed the former `fxhash` maintenance warning from that dependency path.

The final audit reported **zero vulnerabilities**. One maintenance warning remains: `RUSTSEC-2026-0192` for `ttf-parser 0.25.1`, reached through the checked `pdf_oxide 0.3.78` dependency. This path is optional PDF functionality, including the web feature that enables PDF support. It is absent from the default library dependency tree.

The remaining maintenance warning is retained in the report and is not hidden or suppressed. A maintenance warning is distinct from a reported vulnerability. Continue checking the PDF dependency path when updating the lockfile; this audit result is specific to the checked versions and advisory snapshot.

## Validation results

The final local source was checked on macOS ARM64 with Rust/Cargo 1.98.0. These results belong to 0.1.9; the separate 0.1.8 report remains historical.

| Check | Result |
| --- | --- |
| Default library | 187 tests, including the README example, passed |
| All features | 224 tests, including the README example, passed |
| Formatting and Clippy | Passed; Clippy used `-D warnings` for default, all features, and CLI |
| Independent `pdf`, `bots`, `async` feature builds | Passed |
| Rustdoc | Passed with `RUSTDOCFLAGS="-D warnings"` |
| Rust 1.89.0 | All features and targets compiled successfully |
| Source cleanup | Rust lexer verified zero comments across source, tests, examples, and build script |
| Live optional server checks | Bearer authentication, Host/Origin rejection, admin opt-in, query and batch rejection, English defaults, security headers, and authenticated WebSocket handshake passed using temporary runtime state and database |
| Packaged source | All 224 all-feature tests passed from the extracted crate; required docs, examples, and fixtures are included |
| Dependency audit | Zero known vulnerabilities; one optional PDF maintenance warning remains as documented above |

The full source checks run through `./test.sh`. Additional regression cases exercise concurrent atomic writes, invalid weights, symlinks, process identity, failed startup configuration, bounded-distance parity, deterministic top-result selection, request limits, transaction rollback, and crawler destinations.

The table records the local validation runs. GitHub CI results are tracked separately in [Actions](https://github.com/LEVOGNE/airust/actions). No local Docker build was performed. The optional web surface still requires deployment-specific TLS and access controls; filesystem checks do not isolate a hostile local process.

The [follow-up validation report](VALIDATION-0.1.9.md) separately records a rerun of the published package's 187 default-feature tests, exported real-corpus checks, in-memory SQLite replay, and synthetic common-query measurements. It adds no new crate version or application integration. Its workload and consumer build profile differ from the release benchmark below.

## Reproducible scale measurements

The release-mode `index_benchmark` example was run three times at each size on this ARM64 Mac with 64 GiB physical memory. Each run used 300 warm BM25 queries, result limit 10, 1,000 replacements and 1,000 deletions. The corpus consists of short synthetic records with 10,000 bucket terms. Queries select one bucket: about 1, 10, or 100 candidate entries at the three corpus sizes. They do not represent common-term queries over the full corpus.

Times below are medians across three runs. Peak RSS is the median process maximum reported by macOS `/usr/bin/time -l`, including the benchmark process and allocator overhead.

| Entries | Build (ms) | Query p50 (µs) | Query p95 (µs) | Query p99 (µs) | Peak RSS (MiB) | 1,000 updates (ms) | 1,000 deletes (ms) |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10,000 | 19.533 | 1.208 | 1.750 | 2.375 | 24.92 | 2.532 | 0.763 |
| 100,000 | 223.014 | 12.542 | 25.416 | 38.083 | 180.94 | 2.676 | 0.831 |
| 1,000,000 | 2544.711 | 195.625 | 354.583 | 421.292 | 1530.77 | 2.873 | 0.823 |

Raw per-run measurements and environment details are preserved in [benchmarks-0.1.9.json](benchmarks-0.1.9.json). Build first, then time the executable so compilation is excluded:

```bash
cargo build --locked --release --example index_benchmark
/usr/bin/time -l target/release/examples/index_benchmark 10000 300
/usr/bin/time -l target/release/examples/index_benchmark 100000 300
/usr/bin/time -l target/release/examples/index_benchmark 1000000 300
```

These measurements establish this synthetic selective-search workload only. They do not measure relevance quality, semantic understanding, token savings, million-entry fuzzy matching, durable index startup, or performance of an external application. Larger texts, metadata, common terms, and different hardware can change both latency and memory substantially. The roadmap retains those broader evaluations as separate work.

## Scope boundaries

No external application or MCP repository is integrated by this change. AIRust remains a universal engine accepting application-owned entries and metadata. The [roadmap](../ROADMAP.md) describes optional semantic providers, typed relationship evidence, persistence, structural adapters, and larger benchmarks as future work.
