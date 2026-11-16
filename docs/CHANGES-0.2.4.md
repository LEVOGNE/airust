# AIRust 0.2.4: optional source provenance

The independent `provenance` feature adds structured source locations and a shared sidecar store. It enables no graph, embedding, HTTP or database dependency. `KnowledgeEntry` and existing source JSON stay unchanged. No external application or MCP integration is introduced.

`SourceReference` records a stable ID and optional document, page, UTF-8 byte span, file/code span, revision, URI and metadata. Private fields, checked builders and validating deserialization enforce the same location contract. Code columns are one-based UTF-8 bytes; source byte ranges have exclusive ends. Exact supplied text can be checked for bounds and character boundaries. Files and URIs are never opened automatically. Missing location details are not inferred.

`ProvenanceStore` keeps a reference definition once and associates sorted reference-ID sets with entry IDs. Register once with `insert_source`, then `bind` many entries without cloning metadata. `add`, complete replacement, merging entry sources, conflict-checked store merging, removal, pruning and versioned JSON are available. Conflicting definitions are rejected before mutation. Missing sources remain explicit.

Combined with `context-assembly`, `with_provenance` returns borrowed definitions for each selected source entry. Normalized duplicate content retains its distinct provenance. `Missing`, `Referenced` and deliberately `Omitted` source states are separate. External source coordinates are preserved, not computed from knowledge-excerpt offsets. Source evidence and metadata remain outside the explicitly named context text-byte budget.

## Validation and review

Eighteen new integration tests check shared definitions, missing locations, all builder/deserialization boundaries, UTF-8 range sweeps, LF/CRLF and trailing-line positions, malformed/dangling/duplicate imports, format versions, explicit byte limits, stable round trips, source retention on entry merges, removal/pruning, and conflict atomicity. A 600-step mutation sequence is compared against an independent binding map and repeatedly serialized/restored. Context cases check separate duplicate sources, borrowed pointer identity, omitted/missing states, scoped output, and unchanged external offsets.

The review added a distinct omission state so hidden and absent evidence are not confused, and a register/bind path so repeated references do not require copying source metadata. Failed replacements and merges leave both existing bindings and source definitions unchanged. Empty source spans are valid positions but are not presented as supporting quotations.

The complete release script passed **189 default tests** and **299 all-feature tests**, including the compiled source-reference guide. Strict Clippy, formatting, rustdoc, isolated features and runnable examples passed. The provider-independent feature remains within Rust 1.89; the existing local ONNX adapter retains its Rust 1.92 boundary. An independent consumer checks old APIs plus the new provenance and context-reference APIs.

The audit reports zero known vulnerabilities and the same two documented optional unmaintained-dependency warnings, `paste` and `ttf-parser`. No dependency is added by this release.

## Reproducible measurements

The release-mode benchmark creates up to one thousand distinct references and binds them to 1k/10k/100k entry IDs. It exports the full sidecar, restores it, compares equality, then samples one thousand source lookups. The host is macOS ARM64 with 64 GiB memory and Rust 1.98.0.

| Entry bindings | Shared references | Build (ms) | JSON bytes | Export (ms) | Import (ms) | Lookup p50 (µs) | Lookup p95 (µs) | Peak RSS (MiB) |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1,000 | 1,000 | 0.970 | 160,037 | 0.146 | 1.012 | 0.250 | 0.375 | 5.38 |
| 10,000 | 1,000 | 5.564 | 772,037 | 1.139 | 6.813 | 0.667 | 1.083 | 16.17 |
| 100,000 | 1,000 | 53.822 | 6,892,037 | 13.028 | 72.041 | 1.167 | 1.791 | 128.09 |

```bash
cargo build --locked --release --features provenance --example provenance_benchmark
/usr/bin/time -l target/release/examples/provenance_benchmark 1000
/usr/bin/time -l target/release/examples/provenance_benchmark 10000
/usr/bin/time -l target/release/examples/provenance_benchmark 100000
```

Raw lookup samples and operating-system counters are in [measurements/0.2.4](measurements/0.2.4/). Peak memory includes the original store, JSON, restored store and import staging; it is not a minimal steady-state storage claim. This workload has one reference per entry and small metadata, not knowledge text, inference or network access. Large metadata and many references per entry change the costs. No production capacity or relevance-quality claim follows from these measurements.

## Limits and caller responsibilities

Source presence is not proof of truth, freshness, a valid URL, a verified revision or a real PDF page count. Structural validation cannot establish those facts. The caller updates or invalidates bindings when knowledge text changes and supplies exact source bytes for location checks when needed. IDs alone do not establish that old evidence supports new text.

The versioned sidecar is caller-owned provenance data, separate from the knowledge index. JSON imports have a default 16 MiB input cap with an explicit override; generic deserialization uses the caller's resource policy. Unbound definitions remain until pruning. See [PROVENANCE.md](PROVENANCE.md) and [migration notes](MIGRATION-0.2.4.md).

## Package and release gate

The inspected package contains 162 files, approximately 289 KiB compressed, including required sources, tests, guides, examples and measurements. All 299 all-feature tests passed from the extracted crate. The independent Rust 1.89 consumer passed old retrieval/graph/context APIs and shared references, JSON and cited context. Rust 1.92 checks the complete optional-feature set. Candidate CI, a clean package and dry run, and registry/tag/asset verification gate publication and the next stage. Artifacts: [GitHub v0.2.4](https://github.com/LEVOGNE/airust/releases/tag/v0.2.4), [crates.io 0.2.4](https://crates.io/crates/airust/0.2.4).
