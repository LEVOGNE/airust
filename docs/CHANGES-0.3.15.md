# AIRust 0.3.15: Compact PDF validation and stability closure

AIRust now keeps a compact list of pending child references for each active PDF page-tree branch. The previous validator scheduled a full work item for every child. Very wide trees made that temporary work list unnecessarily large. The new traversal visits one child at a time and retains every count, duplicate, depth and resource check.

This is a measured memory improvement. The new real-document campaign found no additional confirmed text-loss defect. Text extraction, cleaning, chunking, search and the native backend are unchanged. No public method, format, flag or default dependency is added. See [API and upgrade](PDF-PAGE-TREE-MEMORY.md).

## Executed checks

All 202 default tests, 729 all-feature tests, 32 documentation examples, 37 isolated feature entries and five Python tests pass. Strict lint/docs and Rust 1.89/1.92 checks pass. Patch API checks pass 229 checks with 31 tool skips against 0.3.14. Fourteen unchanged-backend regressions and provenance checks pass. Fresh dated dependency audits report zero known vulnerabilities, with three main and four standalone-backend maintenance warnings retained.

Eleven metadata unit tests include 128 varied generated nested trees. An independent expected leaf count checks each valid tree; a corrupted root count must fail. Twenty-six owned PDF structures run across both frozen versions: nine required complete successes and seventeen required rejections per version. All known witnesses remain ordered, no orphan text enters successful outputs, and rejected conversions preserve old output. A real one-million-edge fixture still fails at the reference guard before scheduling; it is not a valid million-page capacity test.

The final deterministic 4,096-input mutation campaign accepts 2,108 cases and returns 1,988 normal errors. There are zero caught parser panics, worker signals, worker deadline errors or outer timeouts. Every failed import preserves the previous output. Arbitrary accepted mutations have no complete-text oracle; this measures containment and publication behavior, not exhaustive security.

Independent Rust 1.89 consumers check the public feature APIs and parse the current complete knowledge export before running a 1,000-entry prefix for one checked epoch. The latter verifies updates against rebuilds, JSON restoration, concurrent readers and cancellation/deadlines. Earlier larger core/soak measurements are inherited through unchanged relevant source, rather than presented as fresh full-corpus runs.

## Large manuals and retained knowledge

Thirty-six final-runtime complete import runs represent 35 distinct PDFs. Every complete output is byte-identical to frozen 0.3.14. Unique coverage is 49,086 physical pages and 167,942 chunks. The extra PGF run is a byte-identical source control and is excluded from corpus growth.

Fresh independent extraction covers the [PGF/TikZ manual](https://mirrors.ctan.org/graphics/pgf/base/doc/generic/pgf/pgfmanual.pdf), [Racket 8.12 reference](https://download.racket-lang.org/releases/8.12/pdf-doc/reference.pdf) and [Guile manual](https://www.gnu.org/software/guile/manual/guile.pdf). Their snapshots, redirect URLs, sizes and hashes are pinned in [sources.json](measurements/0.3.15/sources.json); these are identified snapshots, not a claim to the latest language releases. PGF matches an existing source exactly.

Across their 3,654 physical pages, every MuPDF nonempty page also has native text. Selected pypdf page probes and rendered pages supplement that coverage check. Both raw and cleaned chunks remain contiguous excerpts of the corresponding actual native page, with complete ranked IDs and score bits identical to 0.3.14. Of 62 cleaned BM25/TF-IDF literal probes, 58 put the expected page in the top ten. Four failures remain documented; these probes are not human semantic relevance labels. Reader agreement on nonempty pages does not prove correct glyph decoding, reading order or complete table/diagram extraction.

Racket and Guile contribute 7,857 new generic knowledge records. All previous 160,085 records retain their exact IDs, text and metadata. The new canonical export has SHA-256 `13168d87872a137355fd8499c90a6084f52243a2ec7456f29b5fcb2a1c7423c3`. All eight checkpoints, source files, references, renders, raw outputs, setup failures and earlier benchmarks remain local. Test mutations never enter canonical knowledge.

## Isolated metadata memory

Twenty-four fresh processes compare the actual old/new helper runtime through the same fake loader, compiled with Rust 1.89 and Cargo's default release profile. Six alternating repeats per case/version run after deliberate builds, imports and other validation finish. Input Object construction, traversal and destruction are included; native PDF file parsing is absent. Other desktop activity is uncontrolled.

| Case | Median peak MiB, 0.3.14 → 0.3.15 | Change | Median wall milliseconds, old → new |
| --- | ---: | ---: | ---: |
| 999,999 child refs, reject at actual page two | 122.203 → 99.305 | -18.74% | 14.742 → 13.953 |
| 100,000 complete fake leaf nodes | 19.281 → 16.250 | -15.72% | 15.284 → 15.826 |

Both versions return the same required limit error or exact leaf total. This isolates a temporary-work allocation cost. It does not establish a native-parser, whole-import or million-page RAM improvement; compact references and native Object arrays still grow with input width. Raw samples and helper/source/binary hashes are retained, including slower samples and the earlier separate prototype. The [public memory recipe](validation-0.3.15/README.md) reproduces this workload.

## Complete converter costs

Thirty-six fresh processes compare six real PDFs, with three alternating repeats per case/version. Both frozen converters use the same compiler, thin-LTO repository release profile, flags and warm input/file caches. Wall time includes worker startup, validation, extraction, cleanup, chunks, parent checks and atomic output. Parent `wait4` RSS excludes the worker and cannot describe whole-job native memory.

| Case | Physical pages | Median seconds, 0.3.14 → 0.3.15 | Change |
| --- | ---: | ---: | ---: |
| vkspec | 5,491 | 8.301 → 8.262 | -0.47% |
| intel-sdm | 5,364 | 7.434 → 7.433 | -0.02% |
| r-reference | 3,995 | 5.586 → 5.588 | +0.04% |
| scipy | 3,582 | 7.002 → 7.162 | +2.29% |
| racket-reference | 1,438 | 1.661 → 1.663 | +0.09% |
| guile-manual | 892 | 2.241 → 2.262 | +0.93% |

Complete results are byte-identical. Three warm repeats do not establish a statistically significant general speed change, cold-start bound or every-host guarantee. Individual outliers remain in the sample data.

## Separate preview costs

Eighteen fresh processes request one usable page through the library API. Three alternating repeats per version/case use one comparison binary, a common unchanged backend 0.3.5 and Cargo's default release profile. Method time includes opening, complete tree metadata, prefix extraction, cleanup and projection; final result hashing/stdout is excluded. This process contains the native parser, so its RSS is separate from converter-parent RSS and the isolated helper.

| Case | Median API milliseconds, 0.3.14 → 0.3.15 | Peak MiB, old → new |
| --- | ---: | ---: |
| vkspec | 1774.034 → 1771.328 | 218.750 → 217.938 |
| racket-reference | 505.059 → 507.841 | 60.750 → 60.781 |
| guile-manual | 651.845 → 645.473 | 69.328 → 69.469 |

Every returned physical page number and complete text signature agrees. Preview still checks the full tree before extracting a usable prefix; native opening can dominate. The published reproduction uses the exact previous registry dependency and verified extracted current crate as a path dependency, because compatible exact registry patch aliases conflict in one Cargo graph.

## Closure and remaining limits

0.3.15 is the final planned 0.3.x stability release. Subsequent 0.3.x patches require a concrete defect. The next planned scope is a small optional document/revision/position contract for 0.4.0, with source identity and Unicode coordinate acceptance tests. This report does not publish or integrate that prototype. See [0.4 preparation](PREPARING-0.4.md).

Initial strict-lint and restricted-network setup failures remain retained and were corrected before acceptance. Clean packaging, extracted-package tests, dry-run, exact-commit Linux CI, actual registry consumers and matching crate/VCS/tag/release assets gate publication. Final publication identity belongs to the release verification asset.

Resource guards are not hard native CPU/RAM isolation. Glyph decoding, reading order, OCR, arbitrary damaged-file recovery and semantic truth remain scoped limitations. Hashes identify retained bytes rather than prove their meaning. No Docker execution, new mandatory model, DenkMaps or MCP integration is claimed. See [aggregates](measurements/0.3.15/), [reproduction](validation-0.3.15/README.md), [API cookbook](API-COOKBOOK.md) and [stability gates](STABILITY-0.3.md).
