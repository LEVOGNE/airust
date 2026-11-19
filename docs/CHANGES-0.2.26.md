# AIRust 0.2.26: Comparable quality gates

comparisons of identified retrieval runs, visible per-query regressions and explicit quality/resource criteria.

## Validation

The suite passes 194 default and 610 all-feature Rust tests, including documentation, plus five Python tests. Isolated feature checks, strict Clippy/rustdoc, Rust 1.89/1.92, patch compatibility against 0.2.25, audit, local/registry consumers, extracted-package tests, dry-run and exact-commit CI gate publication. Correction recorded during the 0.3.0 review: the ignored test is a helper launched by the parent crash-recovery test. External native-model inference was not rerun for this release.

See [API and limits](QUALITY-GATES.md) and [measurements](measurements/0.2.26/). Dataset/model identities, explicit uncertainty and application-owned authorization remain part of the contract. Public measurements do not establish production quality or an independent security audit. No DenkMaps/MCP integration.

## Independent scifact retrieval

5183 documents and 300 original test queries were evaluated at K=10. BM25 mean nDCG: 0.661676; TF-IDF: 0.577451. The default comparison verdict is failed, with 90 regressed queries. This is an observed comparison, not a claim that the candidate improves the baseline. GermanQuAD here uses the MTEB mirror of 474 test contexts, not the full Wikipedia corpus. External texts are not redistributed. Dataset URLs/revisions/hashes and full ID-level metrics/rankings are in measurements; native neural assets and actual DenkMaps relevance are not tested.

## Independent germanquad retrieval

474 documents and 2204 original test queries were evaluated at K=10. BM25 mean nDCG: 0.906913; TF-IDF: 0.852131. The default comparison verdict is failed, with 402 regressed queries. This is an observed comparison, not a claim that the candidate improves the baseline. GermanQuAD here uses the MTEB mirror of 474 test contexts, not the full Wikipedia corpus. External texts are not redistributed. Dataset URLs/revisions/hashes and full ID-level metrics/rankings are in measurements; native neural assets and actual DenkMaps relevance are not tested.
