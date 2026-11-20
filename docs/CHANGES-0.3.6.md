# AIRust 0.3.6: More faithful PDF text

The PDF cleaner now protects digit boundaries and blank paragraphs, joins chained wraps without consuming adjacent letters, normalizes Unicode after transformations and preserves numeric rows inside a page. Only the first/last nonempty extracted lines are considered for page-number removal. Existing flags can disable these heuristics. Public signatures and JSON structure are unchanged, but corrected text can alter chunk identities and ranking. See [API, controls, upgrade advice and limits](PDF-TEXT-FIDELITY.md).

The three new official references contain 3,178, 5,491 and 696 pages. Together with retained sources, 21 documents/25,321 physical pages are checked. A pinned differential harness compares registry 0.3.5 and current 0.3.6 with the same parser and dependency versions. Every page matches the independently implemented cleanup policy. Its 2,782 changed pages are retained as private before/after evidence. All previously imported pages remain present; unchanged cleaned pages have identical chunk text and metadata. The fresh corpus contains 91,766 entries. Older 60,266- and 33,708-entry exports remain unchanged.

The fixed literal probe groups retain their expected-page top-ten outcomes. Twenty-four rank lists change, with no gained or lost expected-page hit. No semantic quality improvement is claimed. A 4,096-case flag/Unicode string campaign and the separate 4,096-case malformed-PDF campaign supplement the corpus. The latter has zero caught parser panics, abnormal parent exits or outer timeouts; failed outputs remain intact. Protected/image-only format cases still fail normally. A real worker deadline and a verified owned-worker SIGKILL preserve previous knowledge.

The unchanged optional backend remains 0.3.4, with its separate published source/tag/checksum. No new core dependency, OCR, password API, DenkMaps/MCP integration or 0.4.0 expansion is added. Independent Python/Unicode tools are lab-only dependencies, excluded from Git, Cargo packages and Docker context.

The complete suite passes 197 default and 694 all-feature tests, compiled examples, five Python tests, strict lint/docs and all 37 isolated feature entries. After the final README update, 32 module/README doctests and strict default/all-feature documentation checks pass again. Rust 1.89/1.92, 229 patch API checks against 0.3.5 (31 tool skips) and the independent local Rust 1.89 consumer with cleaner/control assertions also pass. Refreshed main and backend audits report zero known vulnerabilities, with three/four maintenance warnings retained respectively. These scans are not security certification. Clean package/extracted tests and dry-run precede exact-commit Linux CI and publication. Exact registry consumer, downloaded bytes, VCS, tag and release asset checksums are verified afterward. The retained sources and all raw measurements remain local; public evidence contains aggregates and IDs rather than document text.


## Measured cost

Three fresh-process default samples per version and document ran in alternating order, with the same thin-LTO/codegen1 release profiles on Rust/macOS ARM64. Input/file caches were warm; deliberate build/fuzz/API jobs had completed, while other desktop activity was not controlled. Individual `wait4` usage supplies direct-child peak RSS.

| Input | 0.3.5 median | 0.3.6 median | 0.3.5 RSS | 0.3.6 RSS |
| --- | ---: | ---: | ---: | ---: |
| PostgreSQL, 3,178 pages | 4.736 s | 4.777 s | 179.95 MiB | 185.17 MiB |
| Vulkan, 5,491 pages | 7.726 s | 7.877 s | 392.67 MiB | 393.22 MiB |
| RISC-V, 696 pages | 1.175 s | 1.190 s | 72.67 MiB | 72.44 MiB |

Default imports intentionally retain different text and have slightly higher measured median times and file-specific RAM changes. This is a text-fidelity correction, not a speedup claim. Six additional `--no-clean` runs produce byte-identical paired output for all three new references. All 24 runs and binary identities remain available in the [aggregate evidence](measurements/0.3.6/paired-cli.json).
