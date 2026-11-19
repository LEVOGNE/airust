# AIRust 0.3.0: Verified stability baseline

This release strengthens the existing engine. It adds no DenkMaps or MCP integration and no mandatory native model, server or database dependency. See [migration](MIGRATION-0.3.0.md), [stability contract](STABILITY.md) and the [API cookbook](API-COOKBOOK.md).

## Fixes and regression coverage

Direct Serde imports of learned feature schemas, training parameters, training reports and models now enforce the same invariants as their constructors and validated model-use paths. The new regression failed on 0.2.30 because a duplicate-name feature schema was accepted. Invalid configurations, report counts, weights and format versions are rejected. Valid model JSON and fingerprints remain unchanged.

Disk metadata size checking and reading now happen in one SQL statement, eliminating the gap between separate reads outside a transaction. The existing 16 KiB bound remains. Tests cover the exact byte boundary and oversized/empty metadata through revision and count APIs.

Compatibility fixtures are real outputs from unchanged v0.2.30 code: engine JSON, a trained model, a feedback ledger and an original SQLite disk-index file. Tests preserve search ordering, model identity, feedback profile and revision; update/reopen and full integrity checks cover the old disk file on a matching recipe. A separate Rust 1.89 test verifies explicit recipe-mismatch rejection without rewriting the file: Unicode/SQLite version changes still require an intentional rebuild, as in 0.2.30. The [fixture manifest](../tests/fixtures/compat-0.2.30/manifest.json) identifies the producer commit and SHA-256 values. A standalone producer is included in validation materials.

Every declared Cargo feature is now checked independently with all targets by a manifest-driven script. It runs as part of `test.sh` and therefore Linux CI. This catches feature-isolation problems that an all-feature build can hide.

## Executed validation

The complete check script passes **195 default tests and 656 all-feature tests**, including documentation, plus five Python tests. It also runs the existing examples, isolated behavior tests, strict Clippy/rustdoc and all **37 manifest entries** independently with all targets (36 optional features plus the empty default entry). Rust 1.89 and 1.92 checks pass. The complete API comparison against 0.2.30 passes 229 checks with 31 checks skipped by the tool, using the stricter patch-compatibility setting.

A standalone Rust 1.89 consumer covers the provider-neutral API. The release acceptance plan additionally requires clean-package tests, publication dry-run, exact-commit CI and registry/asset identity verification. Those final publication gates are recorded externally because they occur after the source commit. Reproduction commands are in [validation materials](validation-0.3.0/README.md).

### Actual native model execution

The adapters verify the explicitly supplied E5 and cross-encoder assets. The run uses ONNX Runtime 1.32.0 on macOS ARM64 with two model threads. The build record identifies binary, model recipes and runtime hashes. E5 passes identical-text vector and German registration-paraphrase checks; all 16 queries in the 12-record authored multilingual smoke corpus have the expected first result. Peak RSS is 1609.31 MiB for that whole process.

A second process loads both adapters together, checks finite logits and batch/single consistency, then reranks the unchanged candidate prefixes over the 30-record/36-query authored reference. Its whole-process peak RSS is 2732.78 MiB. Mean nDCG@3 below uses the 34 positively judged queries; the two no-answer cases remain visible in raw data.

| First-stage source / candidate window | Before reranking | After reranking | Queries whose nDCG worsens |
| --- | ---: | ---: | ---: |
| BM25 / 5 | 0.393850 | 0.438788 | 0 |
| Exact E5 / 5 | 0.858319 | 0.925105 | 2 |
| Exact E5 / 20 | 0.858319 | 0.914250 | 2 |
| Equal-weight hybrid / 5 | 0.598120 | 0.736657 | 0 |
| Equal-weight hybrid / 20 | 0.598120 | 0.914250 | 0 |

Reranking is functional and can improve this aggregate while worsening individual queries. A larger candidate window is not automatically better. These are authored smoke cases, not an independent semantic benchmark or no-answer guarantee.

### Independent lexical quality

Both immutable dataset identities from the 0.2.26 independent evaluation are rerun. Every per-query ranking, metric and case identity matches that reference exactly; elapsed time is excluded from equality. Source revisions/hashes remain linked in the original source manifests. Raw source text is not redistributed.

| Dataset | Documents / queries | BM25 mean nDCG@10 | TF-IDF mean nDCG@10 |
| --- | ---: | ---: | ---: |
| BEIR SciFact | 5,183 / 300 | 0.661676 | 0.577451 |
| MTEB GermanQuAD test-context mirror | 474 / 2,204 | 0.906913 | 0.852131 |

The default comparison gate continues to **reject TF-IDF as a replacement candidate** on both datasets. That expected failed quality verdict is not a failed implementation test. There is no claim of retrieval improvement from this stability release. The German mirror is not the full Wikipedia corpus.

### Scale, consistency and disk tests

| Workload | Executed result | Practical limit |
| --- | --- | --- |
| 2,000,000 synthetic RAM records; four concurrent readers; updates/deletes | Build 6.904 s; peak process RSS 3625.00 MiB; selective query p95 4.638 ms; concurrent selective p95 2.925 ms | A common term takes p50 550.871 ms and up to 610.965 ms; selective timings do not describe all queries. |
| 100,000 source snapshots; four readers and two competing writers | Ten commits; three correctly detected conflicts; all old views unchanged; peak RSS 635.69 MiB | Synthetic lexical/source state; no external database or real embedding model in this workload. |
| 100,000 synthetic disk-index records | Build 519.451 ms; query 1.405 ms; reopen 0.142 ms; full integrity scan passes; peak RSS 7.95 MiB; database 19,374,080 bytes | OS file cache is outside much of RSS; reopen is not guaranteed cold I/O. This is not an identical native-model workload. |

These are single local runs performed alongside other validation, not controlled latency experiments or cross-version speed claims. Resource records retain exact commands, sample counts, process scope and platform. Earlier identical-record memory/disk comparison remains in the 0.2.30 report.

### Dependency audit

The refreshed RustSec database reports zero known vulnerabilities in Cargo.lock. It also reports maintenance notices for optional dependency paths through `chrono`, `paste` and `ttf-parser`; the [audit record](measurements/0.3.0/audit.json) preserves package versions and advisory URLs. Those notices are not silently treated as a clean maintenance bill. No dependency versions are changed in this release.

## Limits

The ignored test in the ordinary Rust test output is an intentional crash-writer helper. The parent crash-recovery test invokes it separately; it is not a missing native-model test. Earlier 0.2.26–0.2.30 reports incorrectly described it and now contain a correction.

Local model checks use supplied model files and an ONNX runtime on macOS ARM64. Linux CI validates builds and portable tests, not those native assets. Synthetic fixtures do not represent application workloads. Independent relevance evaluations do not make every query or language reliable. A dependency audit reports known advisories and maintenance notices; it is not an independent security audit. Application authorization, durable synchronization and integration tests remain caller responsibilities.
