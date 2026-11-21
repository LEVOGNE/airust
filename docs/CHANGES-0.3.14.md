# AIRust 0.3.14: Check actual PDF page counts

The retained two-page diagnostic proved that a plausible but false root count could hide a page. Additional authored cases show that intermediate counts can conceal or repeat text while their declarations add up to the correct root total.

Version 0.3.14 validates the complete reachable page tree before extraction. It independently counts leaves, checks all present subtree counts, rejects duplicate/cyclic or malformed references and applies bounded work/page limits. For a nonempty live tree the parser count must then agree. An empty live tree returns no pages and bypasses the native orphan-recovery scan. Failure returns no import result and preserves existing output in direct and supervised conversions. See [API and upgrade](PDF-PAGE-COUNTS.md).

Public signatures, data formats and default features remain. The maintained backend stays at 0.3.5 with unchanged source, notices and provenance. Knowledge remains the unchanged 160,085-entry export from 33 PDFs / 46,756 physical pages. All originals, seven canonical checkpoints, failed attempts, references, fixtures and measurements remain retained; deliberately damaged inputs never enter canonical knowledge.

## Validation design

Ten unit regressions cover root/subtree counts, mixed empty branches, indirect counts, type errors, cycles, repeated/generation references and exact resource boundaries. Three public regressions cover ordered complete text, library/preview rejection, actual page limits and unchanged output in both CLI modes. The independent Rust 1.89 consumer builds its own count fixtures through public methods.

The frozen previous/current converter comparison uses 26 owned structures: nine new-version successes require complete ordered witnesses and no orphan text; seventeen cases require rejection. Historical acceptance and known witness omissions are separate categories. A real million-edge fixture checks the reference guard before child scheduling. Large real-document equality, damaged-input containment and separate preview/converter costs supplement the authored oracle.


## Executed checks

All 202 default tests, 728 all-feature tests, 32 final documentation examples, 37 isolated feature entries and five Python tests pass. Strict lint/docs, Rust 1.89/1.92 and both independent local consumers pass. The corpus consumer actually runs a 1,000-entry prefix and one checked epoch. Patch API checks pass 229 checks with 31 tool skips against 0.3.13. Fourteen unchanged-backend regressions and provenance checks pass. Dated dependency audits find zero known vulnerabilities, retaining three main and four standalone-backend maintenance warnings.

All 33 PDFs were freshly imported with the final production build. Every complete output is byte-identical to 0.3.13: 46,756 physical pages and 160,085 chunks. All original hashes and seven canonical exports remain unchanged. Earlier independent extraction/cleanup/excerpt proof is inherited through complete output identity; it is not another fresh reference-extraction campaign.

The 26 paired tree fixtures produce nine required successes and seventeen required rejections in 0.3.14. Every success contains complete ordered witnesses and no orphan text. The underdeclared root and forged-subtree cases have known witness omissions in the previous version; other historical acceptances include malformed declarations or structures beyond the new budget and are kept as a separate category. Every failed conversion preserves old output, with no abnormal parent exit or outer timeout. A 6,000,901-byte authored file with one million repeated child edges fails at the reference-work guard before scheduling. This is a guard test, not valid million-page capacity.

An additional empty-live-tree fixture contains two unreachable page objects. Version 0.3.13 imports them through native zero-count recovery; 0.3.14 returns no library/preview pages and refuses the empty converter result without replacing existing data. The final guard skips that recovery scan altogether. An initial candidate that instead compared the native count is retained separately.

The final 4,096-case damaged-input run accepts 2,108 cases and returns 1,988 normal errors. There are zero caught parser panics, worker signal errors, worker deadline errors or outer timeouts. All failed imports preserve previous output. Count mismatches appear in 383 diagnostics; 1,902 failures occur during tree validation, including those mismatches. These categories overlap. Successful arbitrary mutations have no complete-text oracle; this campaign checks containment and failure atomicity. The initial candidate campaign and corrected harness depth/fixture-reuse assumptions remain retained separately.

## Complete converter costs

Thirty-six fresh release processes compare six PDFs, with three alternating repeats per case/version. The six documents cover 21,414 physical pages. Both versions use the repository's thin-LTO release profile, the same flags/compiler and warm input/file caches. Deliberate build/check/reimport/fuzz work has finished; other desktop activity is uncontrolled. Times include the complete supervised command: worker startup, metadata/extraction/cleanup/chunks, parent result validation and atomic publication.

| Case | Physical pages | Median seconds, 0.3.13 → 0.3.14 | Change |
| --- | ---: | ---: | ---: |
| Vulkan specification | 5,491 | 8.942 → 8.721 | -2.47% |
| Intel manuals | 5,364 | 7.969 → 8.018 | +0.61% |
| R reference | 3,995 | 6.072 → 6.016 | -0.91% |
| SciPy reference | 3,582 | 7.497 → 7.554 | +0.76% |
| Retained pdf-15 | 2,488 | 7.008 → 7.253 | +3.51% |
| Original slow pdf-12 | 494 | 0.613 → 0.599 | -2.28% |

All complete outputs remain byte-identical. Slower individual samples remain in the public sample data, including a current Intel run at 10.636 seconds. Three warm samples per case/version do not establish a statistically significant speed change or cold-start bound. Converter `wait4` RSS belongs to the parent supervisor and excludes its native worker, so it is not a whole-job RAM metric.

## Separate bounded-preview costs

Eighteen fresh processes request one usable page through the library API, with three alternating repeats per version/case. Each selects one aliased version in the same comparison binary. Both use the common unchanged backend 0.3.5; the prerelease recipe overrides both aliases with that byte-identical local backend. The public reproduction recipe uses both exact registry versions. This independent harness uses Cargo's default release profile, with the same compiler and warm input/file caches within the comparison.

API time includes file opening, native metadata, the first usable page, cleanup and projection; result hashing/stdout is excluded. Process wall/RSS include those checks and the native parser inside this process. These numbers are separate from converter parent RSS and pure-validator cost.

| Case | Median API milliseconds, 0.3.13 → 0.3.14 | Change | Peak MiB, 0.3.13 → 0.3.14 |
| --- | ---: | ---: | ---: |
| Vulkan specification | 1872.694 → 1868.263 | -0.24% | 219.297 → 219.625 |
| SciPy reference | 2039.188 → 2052.584 | +0.66% | 302.531 → 303.141 |
| Original slow pdf-12 | 144.856 → 144.884 | +0.02% | 73.188 → 73.188 |

All returned text and physical page numbers are identical. Preview now checks full-tree metadata but still only extracts its usable prefix. Observed differences do not promise the same cost on every tree, file cache or host. Full samples, binary hashes and conditions remain in the aggregates. Clean packaging, extracted tests, dry-run, exact-commit Linux CI, registry-only consumers and crate/VCS/tag/asset identity finish publication; final outcomes belong to the release verification asset.

## Limits

Validation covers the live catalog page tree and declared counts. It does not prove complete content/glyph decoding or recover arbitrary unreachable/deleted pages. Preview performs a whole-tree metadata check and only prefix content extraction. Resource counters do not provide hard native-call preemption or an OS RAM sandbox. No Docker execution, new mandatory model, DenkMaps or MCP integration is claimed.

See [aggregates](measurements/0.3.14/), [reproduction](validation-0.3.14/README.md) and [stability gates](STABILITY-0.3.md). Earlier core matching/cache/memory/soak evidence remains explicitly inherited only where exact source identity supports it; new PDF runs are identified separately.
