# AIRust 0.3.5: Earlier PDF budget rejection

Conversion now checks chunk count and overlap-expanded chunk bytes after each page. It stages one copy of validated chunks, releases the native parser and then creates training input/output pairs. Previously, a failing chunk budget could still extract the entire document first. The regression test reproduces the old failure ordering and verifies the earlier rejection without a fragile timing threshold. Public signatures and successful output formats remain unchanged. See [API, behavior, reproduction and limits](PDF-IMPORT-PIPELINE.md).

The retained real corpus grows from fifteen to eighteen PDFs, with 15,956 physical pages and 60,266 canonical entries. The new official documents contain 5,364, 756 and 876 pages. Independent MuPDF page extraction and selected pypdf probes complement direct old/new comparisons. The new literal probes retain identical checked rankings: BM25 26/27 and TF-IDF 18/27 expected pages in the top ten. This is not a semantic relevance benchmark.

Eight authored format cases add 2,163 physical test pages per converter, including object streams/incremental updates, rotations, Unicode and encryption. Protected and image-only documents fail normally with previous output preserved. The 4,096-case mutation campaign has zero caught parser panics, abnormal parent exits or outer timeouts, with unchanged acceptance classifications and all failure outputs intact. The original fifteen-document worker pass preserves every checked text and metadata field; a real worker deadline still stops the larger Python reference.

Measurements found increased full-import RAM in the first candidate. That candidate was corrected before publication: the native parser is now released before training text is duplicated. Initial and final evidence remain available. Full imports do not have a universal latency or RAM improvement; the targeted gain is rejecting an already exhausted chunk budget earlier.

The unchanged optional backend stays at 0.3.4 with its separate published source identity. Original knowledge and source PDFs remain intact, and the new combined canonical export excludes authored and damaged test data. No DenkMaps/MCP integration or 0.4.0 feature expansion is added.

The complete runtime suite passes 197 default and 689 all-feature tests, all 37 isolated feature entries, strict lint/docs, compiled examples and five Python tests. Minimum Rust 1.89/1.92, 229 patch API checks against 0.3.4 (31 tool skips), a fresh audit and independent local Rust 1.89 consumer pass. After the final README update, all 32 README/module doctests and strict default/all-feature documentation checks also pass. Clean package/extracted tests and dry-run precede exact-commit CI and publication. Registry consumer, source/VCS/tag/checksum/asset equality are verified afterward. See [validation instructions](validation-0.3.5/README.md) and [measurements](measurements/0.3.5/).


## Final paired measurements

After the intentional build/fuzz/API workloads completed, three fresh processes per version and case ran in alternating order on the same Rust/macOS ARM64 release profiles. Input/file caches were warm; other desktop activity was not controlled. Individual direct-child `wait4` records peak RSS. All 18 successful outputs in this follow-up match their paired JSON byte for byte. Six budget failures preserve previous output.

| Case | 0.3.4 median | 0.3.5 median | 0.3.4 RSS | 0.3.5 RSS |
| --- | ---: | ---: | ---: | ---: |
| 5,364-page full import | 7.354 s | 7.483 s | 311.66 MiB | 313.62 MiB |
| 756-page full import | 1.418 s | 1.379 s | 346.39 MiB | 345.61 MiB |
| 876-page full import | 1.747 s | 1.752 s | 181.41 MiB | 180.89 MiB |
| 5,364 pages; maximum one chunk | 7.392 s | 1.072 s | 311.44 MiB | 247.20 MiB |

The targeted rejection is about 6.9 times faster with 20.6% lower peak RSS in this case. Complete imports remain similar; small differences are not treated as a general improvement. An earlier 40-run final-candidate comparison overlapped builds/fuzz/API work and is retained alongside this 24-run follow-up. All 64 final-candidate comparison runs preserve paired output or previous failure output. The rejected first candidate has its own separate record and binary.
