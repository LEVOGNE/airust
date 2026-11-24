# Reference application work-program validation

This is the historical development closeout at commit `4552e916e27b41cd9db39805c2b3765ba40e2733`, before release authorization. For the published 0.5.3 source and final release gates, see [release verification](validation-0.5.3/README.md). The development receipts below retain their original identities.

This completes the seven-stage development work program after the verified published AIRust 0.5.2 baseline. The current version is **0.5.3-dev**, an unreleased development branch. It changes no default Cargo feature or public library API. Published 0.5.2, its tag and archive remain unchanged. The maintained PDF backend stays unchanged at 0.3.5.

## Delivered work

| Stage | Result and evidence |
| --- | --- |
| Release baseline | [Pinned release closeout and registry consumer](validation-0.5.2/closeout.json). |
| Reference application | [Local Markdown search, source coordinates and predeclared acceptance policy](REFERENCE-APPLICATION.md). |
| Evaluation dataset | 41 byte-pinned source files, 150 assistant-authored DE/EN questions, family-separated 90/60 splits and explicit unsupported questions. [Frozen fixture](../tests/fixtures/reference-v1/fixture-manifest.json). |
| Complete application example | Import, bounded durable workspace, exact citations, real query CLI and stdio MCP. Six Rust example tests and five real CLI/MCP tests. [Workflow receipt](evaluation/workflow-validation.json). |
| Retrieval comparison | All nine profiles and both splits, actual local model inference, independent metric check, all per-case losses and costs. [Complete reference report](measurements/reference-v1/README.md). |
| Onboarding | Task-based [README](../README.md), [startup](../STARTUP.md) and [API cookbook](API-COOKBOOK.md). Both complete introductory programs also run against registry 0.5.2 on Rust 1.89. |
| Private index structure | Exact matching registry/candidate signatures in default/shared modes, 197 targeted tests and 56 paired cost workers. [Maintenance boundaries](INDEX-INTERNALS.md), [all observed costs](measurements/index-refactor/README.md). |
| Support and maintenance | All 39 feature entries, Linux/macOS CI, explicit minimum-Rust/native-model boundaries, 14 backend regressions and patch-retirement gates. [Support](SUPPORT.md), [PDF maintenance](PDF-MAINTENANCE.md). |

The local `./test.sh` run passed 253 default checks, 911 all-feature checks including 39 doctests, all 39 isolated feature entries, strict Clippy/docs, the additional feature combinations and executable examples. Repeated targeted suites overlap those counts; they must not be reported as unique additional tests. The backend's 14 owned regressions, C-API build and 381-file provenance check passed separately. The [completion receipt](evaluation/completion-validation.json) records source commits, exact commands, actual hosted job results, package identity and raw-log hashes.

The complete 41-document demo workspace is retained locally in `local-lab/reference-workflow/demo-workspace`; the executable and three real query outputs are in `local-lab/reference-workflow/validation`. German/English queries resolved eleven exact citations, and the missing-term control returned no evidence. These local artifacts are ignored by Git and excluded from the package. Source-checkout reproduction remains in STARTUP.md; applications do not depend on these local paths.

The development-selected optional hybrid/reranking profile meets the frozen reference quality objective. Literal lexical retrieval remains the application/MCP default and still fails that objective on this authored dataset. A nonempty result leaves answerability unassessed. The reference scores do not establish customer adoption, independent relevance judgments, generated-answer accuracy or general superiority. External embedding time is separate, and embedding-service RSS was not measured. The model campaign preceded the private index extraction; its source is pinned and its costs were not rerun afterward.

No new coverage-guided fuzz or Miri campaign is claimed for this development work. The recorded 0.5.2 historical campaigns remain historical. Original knowledge, source documents, raw measurements, model assets and rejected trials are retained. Dedicated regenerable Cargo output is removed after preserving receipts, packages, binaries and locks. This work program publishes no crate or new release tag.
