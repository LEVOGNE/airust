# AIRust 0.5.3

Version 0.5.3 turns the verified 0.5.2 baseline into a reproducible local document-search reference application. It combines a complete source-checkout example, a frozen evaluation and measured maintenance work. Existing public calls, default features and readable saved formats remain compatible. The maintained `airust-pdf-backend` stays pinned at 0.3.5 with unchanged source.

## Complete reference workflow

`reference_workspace` imports a visible UTF-8 Markdown directory into a new workspace, saves immutable document snapshots, returns bounded evidence with exact source coordinates, and serves the same collection through read-only stdio MCP. It rejects symlinks, stale inner document identities and corrupted workspaces; saving never replaces an existing destination. Queries return at most six excerpts, 2,000 bytes each, 12,000 excerpt bytes total and 64 KiB complete JSON. Nonempty evidence leaves answerability unassessed; no answer generator is added.

The example owns its version-one workspace format. It is separate from the library's existing persisted formats. [Use case, input limits and reproduction](REFERENCE-APPLICATION.md).

## Frozen, explicitly authored evaluation

The fixture contains 41 byte-pinned documents from 0.5.2 and 150 assistant-authored German/English questions. Development has 90 cases, evaluation has 60, source families are separated, and each split includes 20% unsupported private-deployment questions. Exact supporting quotations, UTF-8 ranges, question origin, source hashes and predeclared policy remain reproducible.

Nine existing profiles were compared before this release-version change. Development selection precedes reserved evaluation. The selected optional BGE-M3 hybrid plus multilingual MiniLMv2 reranker reaches reference nDCG@10 0.8505 and Hit@6 0.9167 on evaluation; literal BM25 reaches 0.6001 and 0.7083. The selected profile adds roughly 836 ms warm p95 and 2.36 GiB worker peak RSS on the recorded host. External embedding time and service memory have separate boundaries; service RSS was not measured. SQLite uses the same pretokenized terms and vectors and does not establish an AIRust speed advantage.

Literal lexical defaults remain unchanged and fail the frozen absolute quality objective on this corpus. Every per-question loss and cost stays visible. These are known-reference measurements, not customer traffic, independent exhaustive relevance labels or generated-answer accuracy. The model campaign was not rerun after the private index extraction or this version change. [All results and limitations](measurements/reference-v1/README.md).

## Maintenance and compatibility

Task-based README, startup instructions and cookbook cover record search, source positions, bounded retrieval and MCP. Private ordered cursors and query-weight memoization have separate modules with concise invariant comments. Published-0.5.2 controls preserve exact ordered IDs, score bits, matched terms, original records, scopes, mutation behavior, retained clones and JSON restoration. Two paired campaigns with 56 workers remain inside the frozen 10% p95/RSS budget. [Index boundaries and evidence](INDEX-INTERNALS.md).

A private semantic-provider feature gate now follows its actual snapshots/execution callers, keeping minimal `mcp,hybrid` combinations warning-free. No new scoring algorithm, dependency, public library API or default feature is introduced. The 39-entry [support matrix](SUPPORT.md) distinguishes Linux/macOS checks, Rust 1.89 provider-neutral code, Rust 1.92 local ONNX adapters and unvalidated targets. The [PDF maintenance contract](PDF-MAINTENANCE.md) records owned patches and their retirement gates.

MCP tool names and schema shapes remain compatible; schema identifiers now contain `0.5.3`. Clients refresh cached identifiers. [Migration](MIGRATION-0.5.3.md).

## Validation and release identity

The release gate repeats `./test.sh`, clean packaging and `cargo publish --locked --dry-run` on the final source. Exact-commit hosted checks cover Linux, macOS, both minimum Rust versions and the separate PDF backend. Both lockfiles receive advisory checks. A registry-pinned Rust 1.89 consumer verifies the published version, public calls, original records, source coordinates, cancellation and all five versioned MCP schemas after publication.

Tag `v0.5.3`, `master`, the crate's VCS metadata, the crates.io checksum and the GitHub `.crate` asset must agree. The completed machine-readable manifest and SHA-256 list are GitHub release assets. [Commands and verification boundaries](validation-0.5.3/README.md).

No new libFuzzer, Miri or native-model measurement campaign is claimed by this release. Previous campaigns retain their original source identities and limits. Passing finite controls and CI does not prove correctness for every input. Original knowledge, historical evidence and model assets remain retained; dedicated Cargo output is removed after release artifacts and logs are preserved.
