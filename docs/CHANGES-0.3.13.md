# AIRust 0.3.13: Reject covered incomplete PDF imports

Owned structural tests found two existing native failure paths: deep Form content was skipped after a work limit, and fallback page counting could omit a deep branch. Both could appear successful. Version 0.3.13 pins optional backend 0.3.5, propagates these failures and prevents failed Form spans from becoming successful cached results. Existing public signatures, default features, JSON and core index behavior remain. See [API and upgrade](PDF-STRUCTURE-LIMITS.md).

The original knowledge remains the unchanged 160,085-entry export from 33 PDFs / 46,756 physical pages. All seven canonical checkpoints, originals, references, candidate imports, failed experiments and raw measurements remain retained. Synthetic mutations never enter canonical knowledge.

## Validation design

Four new backend regressions cover supported Form depths, excessive depth, processing-budget boundaries and fallback counts. Repeated text/span/word/character calls require an explicit error or complete known witnesses when valid caches permit completion. Deep failed Forms must not become a partial cached success. A proposed 64-level recursive implementation overflowed the default test-thread stack; it was rejected. The final fix retains the existing ten-level limit.

Two public regressions cover complete supported text, library error propagation and unchanged existing output for direct/supervised conversions. An independent Rust 1.89 consumer constructs its own nested Forms and exercises the published loader contract. Provenance checks retain all 381 original source hashes, identify three maintained upstream files and the owned helper, and preserve original notices.

## Executed checks

All 202 default tests, 715 all-feature tests, 32 final documentation examples, 37 isolated feature entries and five Python tests pass. Strict lint/documentation checks, Rust 1.89/1.92 and both independent local consumers pass. The corpus consumer actually runs a 1,000-entry prefix and one checked epoch under Rust 1.89. Main patch API checks pass 229 checks with 31 tool skips against 0.3.12. The backend separately passes 14 regressions, C-ABI build checks, Rust 1.89, provenance and 229 default-feature API checks with 31 tool skips against backend 0.3.4. Dated audits find zero known vulnerabilities, retaining three main and four standalone-backend maintenance warnings.

All 33 retained PDFs were freshly imported with the production build and supervised 90-second workers. Every complete training output is byte-identical to 0.3.12: 46,756 physical pages and 160,085 chunks. Earlier independent reference/cleanup/excerpt evidence is inherited through the complete output comparisons; this is not another independent reference-extraction campaign. All original source hashes and seven canonical exports remain unchanged.

The 24 owned structural fixtures preserve complete known witnesses on successful cases. All eight previously incomplete successes now fail explicitly; every failed output retains the previous file. No abnormal parent exit or outer timeout occurs. A separate fresh production campaign mutates three owned bases using eight deterministic byte-edit categories: 4,096 executions, 3,190 accepted and 906 normal errors, with zero parser panics, worker deadline errors or outer timeouts. Every failed import preserves existing output. These arbitrary accepted mutations have no complete-text oracle; the campaign checks containment and failure atomicity. The earlier prototype campaign uses the same fixture seed and remains a separate retained execution set.

The first restricted full test run could not invoke `ps` for an existing own-process identity test. The complete rerun with approved host access passes. That attempt, initial test setup errors, the rejected 64-level stack experiment and corrected cache-boundary assumptions remain retained alongside successful results.

## Paired complete-converter costs

Thirty-six fresh release processes compare six PDFs, with three alternating repeats per case/version. The six documents contain 21,414 physical pages. Both versions use the same flags, compiler/profile, warm file/input caches and complete-output comparisons. Deliberate builds, checks, native reimports and mutation work have finished; other desktop activity is uncontrolled. The table reports median wall time for the complete supervised CLI, including worker startup, extraction, cleaning, chunks, parent validation and atomic publication.

| Case | Physical pages | Seconds, 0.3.12 → 0.3.13 | Change |
| --- | ---: | ---: | ---: |
| Vulkan specification | 5,491 | 8.352 → 8.352 | −0.01% |
| Intel manuals | 5,364 | 7.525 → 7.462 | −0.83% |
| R reference | 3,995 | 5.672 → 5.666 | −0.10% |
| SciPy reference | 3,582 | 7.109 → 7.163 | +0.75% |
| Retained pdf-15 | 2,488 | 6.812 → 6.790 | −0.32% |
| Original slow pdf-12 | 494 | 0.578 → 0.587 | +1.61% |

All complete outputs remain byte-identical. Slower samples remain visible, including a current SciPy run at 7.766 seconds and Vulkan run at 8.738 seconds. Three warm samples per case/version do not establish a statistically significant speed change or cold-start bound. The fix protects failure behavior; no parser-only acceleration is claimed. The aggregate's `wait4` peak RSS belongs to the parent supervisor and excludes the native worker peak, so it is not a whole-job RAM measurement.

## Evidence scope

The 0.3.12 matching/cache/resource results remain inherited only where core and serialization sources are byte-identical. New native structural, real-document, malformed-input and cost runs are identified separately in [the measurements](measurements/0.3.13/). See [reproduction](validation-0.3.13/README.md) and [0.3.x gates](STABILITY-0.3.md).

## Limits

Covered work-limit failures now reject the import; the patch does not establish complete recovery from every malformed PDF. Other native recovery and text heuristics remain. Synchronous calls have no hard preemption or operating-system RAM bound. A supervised job contains ordinary worker failures and applies its documented wall budget. Real-corpus equality is not semantic-quality or exhaustive security certification. No Docker execution, mandatory model, DenkMaps or MCP integration is claimed.

Three additional root-count diagnostics retain a concrete recovery boundary: a deliberately underdeclared positive `/Count` can omit a page while returning success. The unchanged standard path trusts that field; the overdeclared variant fails. These are corrupted metadata cases, not the fixed fallback recursion limit. Strict extraction-coverage validation belongs to the [0.4 acceptance work](PREPARING-0.4.md); current success must not be treated as a proof of complete arbitrary-file recovery.
