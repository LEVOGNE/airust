# PDF stability and retained knowledge

Later stabilization: [0.3.2 import budgets](PDF-LIMITS.md) and [0.3.3 supervised jobs](PDF-WORKERS.md) extend these protections. The real-document evidence below identifies its original 0.3.1 baseline.

PDF support remains optional. These tests exercise the existing importer and search engine without a DenkMaps or MCP integration.

## What 0.3.1 fixes

A destination created while conversion was running could previously be overwritten even without `--overwrite`. The CLI now publishes new outputs using an exclusive atomic operation. Competing writers cannot replace an already-created destination. An existing complete file remains unchanged when publication fails. `--overwrite` remains the explicit replacement path.

`PdfLoader::save_knowledge_base_if_absent(&knowledge, path)` exposes this behavior for library users. It validates the training examples, serializes the complete result, writes a private staging file and publishes it only if the destination does not exist. This requires filesystem hard-link support in the destination directory. Unsupported filesystems fail rather than falling back to an unsafe overwrite. The directory is application-owned; this is not a filesystem authorization service. Parent directories must already exist when calling the library method.

A PDF without usable extracted text now causes conversion to fail. This covers empty/image-only documents and some unsupported text encodings; the message does not pretend to identify the exact cause. Existing output is preserved even with `--overwrite` when extraction fails. `extract_pages_from_pdf` remains available to inspect an empty extraction result. AIRust does not add OCR automatically.

Incremental parenthesis tracking avoids rescanning the whole current sentence at every punctuation mark. Valid-corpus content and checked search ordering are preserved. An experimental interleaved extraction/chunking change was rejected after repeated tests showed no reliable benefit and higher peak RSS on the large Python manual: the growing training collection stayed alive together with parser caches. The final importer retains the original extraction-then-chunking phases and chunk ownership. There is no constant-memory or universal speed claim.

## Real document tests

The retained local lab contains 14 user-provided manuals (6,472 pages) and the official Python 3.14 library reference (2,488 pages): 15 PDFs, 8,960 pages and 33,708 imported chunks. User documents and extracted texts are not redistributed in the repository or crate. Public Python documentation came from [the official PDF archive](https://docs.python.org/3.14/archives/python-3.14-docs-pdf-a4.tar.bz2); its download identity is recorded in the measurements.

A separate pypdf extraction, with FontTools installed for embedded-font handling, supplies literal query anchors near document starts, middles and ends. These are extraction/page-retrieval probes, not independent human relevance judgments. Each source contains 7–9 probes; the total is 115. An inspected rendered page supplements extraction checks. Reading-order and glyph disagreements between parsers remain possible.

Version 0.3.0 and the candidate use the same standalone harness, compiler and release profile. All 15 imports have identical serialized-content hashes, page sets, chunk counts and checked search rankings. Four pages produce no chunks: the independent parser finds three blank pages and one page containing only its page number. The latter is removed by the existing default cleaning policy.

On this run, BM25 returns the expected page in its first ten chunks for 111/115 probes; TF-IDF does so for 102/115. Those misses are retained, not counted as perfect retrieval. The candidate's repeated-query p50/p95 are approximately 0.205/1.615 ms for BM25 and 1.766/13.667 ms for TF-IDF. These measurements use separate per-document indexes, five repetitions per probe and a host doing other validation. They are descriptive, not a controlled performance improvement claim or a test of a combined production corpus.

The 2,488-page document imports in approximately 7.53 s in the initial baseline run and 7.75 s in the final minimal candidate corpus run. Experimental allocation revisions varied from 8.39 to 18.63 s under background load and were not retained as optimizations. Their separate repetition records are labeled preliminary rather than attributed to final release performance. No speed improvement is claimed. A 494-page manual takes roughly 21–34 s across the recorded runs, including approximately 24.68 s in the final minimal candidate: a direct dependency profile attributes about 22.7 s to first-page extraction. The existing PDF dependency remains 0.3.78. This slow recovery/extraction path is an outstanding investigation, not a solved optimization. A synchronous parser call has no hard in-process time or memory guarantee.

## Adversarial probes

The first campaign uses six authored special inputs plus 128 deterministic byte mutations of an authored valid PDF. Cases include blank, empty, broken, encrypted and punctuation-heavy documents. Each runs in a separate child process with three CPU seconds, four wall seconds and an 8 MiB output-file limit. The repeated campaign after the panic fix has no unexpected exits or timeouts: 43 inputs are accepted and 91 return a normal error. Both accepted recoverable documents and rejected malformed documents are recorded.

A separate real CLI race reproduces data loss in 0.3.0. The corrected CLI refuses the destination and preserves the concurrent writer's data. The permanent regression generates a 500-page PDF at test runtime. Concurrent exclusive-writer tests also require exactly one complete winner and no staging leftovers.

The initial campaign uncovered a PDF-library Rust panic on a malformed UTF-8 header. The first summary counted only signal exits as abnormal and missed exit code 101; the corrected classifier treats every exit other than expected success/error (0/1) as abnormal. The saved input is a permanent library and CLI regression. AIRust now catches unwinding panics from opening, page counting and page extraction and returns `PdfError::PdfError`, with no partial import result. The process-wide panic hook is left unchanged and can still emit a diagnostic. Panic-abort builds, aborts, allocation failures and OS signals cannot be contained by this mechanism.

These checks are a bounded robustness campaign. They are not exhaustive fuzzing, an independent penetration audit or proof that a native PDF parser cannot crash. Worker deadlines belong to the lab harness, not to the existing synchronous library API. No other systems or services are penetration-tested.

## Reuse the retained knowledge

The local lab lives under `local-lab/pdf-stability/` and is excluded from Git and crate packaging. It retains source manifests, original downloads, reference page text, queries, parsed indexes, baseline/candidate binaries, raw measurements, rendered samples and regression evidence. The original user PDFs stay in their original folder. See the lab's own README for exact paths and commands. Preserve this directory if moving the checkout.

For a trusted local index export:

```bash
cargo run --locked --features retrieval --example saved_knowledge -- PRIVATE_INDEX.json "your question"
```

This uses the existing compact retrieval API. It does not modify the index or start a service. Returned offsets refer to the imported knowledge-entry text, not to exact glyph coordinates in the PDF. Original PDF page numbers and chunk indexes remain in the retained entry metadata.

The benchmark utility accepts a JSON array such as `[{"page": 10, "query": "distinctive literal terms"}]`:

```bash
cargo run --release --locked --features pdf,quality-gates --example pdf_stability -- INPUT.pdf QUERIES.json PRIVATE_INDEX_EXPORT.json
cargo run --release --locked --features pdf --example pdf_parser_profile -- INPUT.pdf
```

The optional export is created exclusively and contains document text. Store it as application data. Benchmark stdout contains metrics, IDs and hashes rather than the source text; a hash is an identity check, not anonymization or authentication. Measure cancellation, corruption, scope and source-position behavior again when relevant code or parser versions change. New general passage-indexing features remain deferred to a later 0.4.0 decision.

## Package isolation

Release preflight caught unanchored Cargo include patterns selecting a local lab README and a dependency license file. No artifact was published. Top-level metadata patterns are now anchored and `local-lab/**` is explicitly excluded. `scripts/check_package.py` checks the selected package paths against intended repository areas, and runs in `test.sh`. Its `--allow-dirty` flag is used only with the read-only `cargo package --list` inspection; actual release packaging requires a clean commit. The package list also passes under Cargo 1.89. See [Cargo include-pattern semantics](https://doc.rust-lang.org/cargo/reference/manifest.html#the-exclude-and-include-fields).
