# AIRust 0.3.12: Reuse entry identifiers

The large real-corpus workload exposed repeated copies of each complete entry ID in word lists. Version 0.3.12 reuses immutable IDs across those lists and normalized duplicate groups, including cache reconstruction. Public entry IDs, result fields/order, JSON/cache formats and clone-isolation behavior keep their contracts. No public API, flag or mandatory dependency is added. See [API and upgrade](SHARED-IDENTIFIERS.md).

The canonical knowledge remains the unchanged 160,085-entry export from 33 PDFs / 46,756 physical pages. Temporary mutations and synthetic databases stay separate. All originals, prior exports, baseline/current candidates, references and measurements remain retained. The initial isolated prototype and single-pair result remain separate from final acceptance measurements.

## Validation scope

Two regressions cover raw Unicode/control-byte IDs, duplicate groups, lookup, JSON reconstruction, retained clones, updates/removal and clear. Complete cross-version results, persisted caches, full tests/features, minimum Rust versions and independent consumers supplement real-corpus and synthetic costs. See [reproduction](validation-0.3.12/README.md), [aggregates](measurements/0.3.12/) and [stability gates](STABILITY-0.3.md).

## Executed checks

All 202 default tests, 713 all-feature tests, 32 final documentation examples, 37 isolated feature checks and five Python tests pass. Strict lint/documentation checks, Rust 1.89/1.92, independent local consumers and 229 patch API checks pass; the compatibility tool skips 31 checks. Backend provenance and ten owned regressions pass. Audits dated 2026-11-21 find zero known vulnerabilities, with three main and four standalone-backend maintenance warnings retained in the aggregates.

All 24 fixed real-corpus matching cases and 1,024 generated cases preserve complete result IDs, order and every score/similarity bit against 0.3.11. Existing and newly written caches restore all 160,085 entries in both version directions, preserving complete exports and query results. Cache payloads differ only in the producer version label; the files and original metadata remain retained. Existing cache trust, quota and analysis-signature requirements still apply.

Every retained PDF was freshly imported. All 33 complete outputs, covering 46,756 physical pages and 160,085 chunks, are byte-identical to 0.3.11. PDF loading, cleaning, workers, persistence and backend sources are unchanged in this release. Earlier independent extraction/cleaner/mutation evidence remains explicitly inherited, rather than counted as another fresh campaign.

The real-corpus soak passes 32 complete update/rebuild/JSON epochs, concurrent readers, isolated clones and cancellation/deadline checks. It takes 564.760 seconds with 5.548 GiB peak RSS under other validation load. This process retains multiple complete reference views and JSON buffers; it does not measure one engine's requirement or prove that every memory leak is absent.

## Paired resource measurements

Fifty-four fresh release processes compare 0.3.11 and 0.3.12. Versions alternate, with the same public harness, profiles, compiler and warm input/file caches. Deliberate validation/build/soak work has finished; other desktop activity is uncontrolled. Process peak RSS comes from each child's `wait4` result. Times include the workload's correctness checks. All real exports and complete checked results remain identical.

Each real process parses the trusted export, selects a fixed prefix, runs three update/rebuild/JSON epochs and checks four concurrent readers. The table reports medians of three fresh processes per version and size. RSS includes multiple comparison views, rebuilt indexes and JSON buffers.

| Real entries | Process seconds, 0.3.11 → 0.3.12 | Peak GiB, 0.3.11 → 0.3.12 | Peak reduction |
| ---: | ---: | ---: | ---: |
| 10,000 | 3.139 → 2.971 | 0.606 → 0.528 | 12.94% |
| 100,000 | 36.704 → 34.949 | 3.829 → 3.138 | 18.04% |
| 160,085 | 66.495 → 60.732 | 6.286 → 4.898 | 22.08% |

The remaining 36 processes use short synthetic records. They are separate workloads, not equivalent numbers of real PDF passages. Default/shared scale and clone binaries differ only by `shared-index`; snapshots also enable execution/snapshots. Each row has three processes per version, except snapshots with six. Clone runs retain three views and check isolation after mutation; snapshot runs check four readers, two competing writers and ten committed updates.

| Synthetic workload | Entries | Configuration | Process seconds, 0.3.11 → 0.3.12 | Peak GiB, 0.3.11 → 0.3.12 |
| --- | ---: | --- | ---: | ---: |
| Scale | 1,000,000 | default | 5.249 → 5.023 | 1.772 → 1.553 |
| Scale | 1,000,000 | shared-index | 5.208 → 5.102 | 1.666 → 1.447 |
| Three retained clones | 500,000 | default | 2.539 → 2.178 | 3.376 → 2.895 |
| Three retained clones | 500,000 | shared-index | 1.755 → 1.603 | 1.171 → 0.981 |
| Concurrent snapshots | 100,000 | shared-index, execution, snapshots | 0.302 → 0.301 | 0.197 → 0.181 |

These observations support this representation change on identified workloads. They do not establish a universal percentage, maximum capacity or improved semantic relevance. All samples, per-algorithm query categories, binary hashes and conditions remain in [the aggregates](measurements/0.3.12/). The initial prototype's single pair is retained separately and is not used as the acceptance result. Clean packaging, extracted-crate tests, dry-run, exact-commit Linux CI, registry consumers and byte/VCS/tag/asset checks finish publication; the final identity belongs to the release verification asset.

## Native findings requiring follow-up

Twenty-four additional owned structural fixtures expose existing limits in the unchanged 0.3.4 backend. Text inside Form XObjects nested beyond ten levels is skipped. A Form XObject is a reusable block of PDF content; a deeper block can still contain ordinary text. Fallback page counting on a missing `/Count` can also skip a branch after its depth limit. The accepted fixtures with missing witnesses are identified in the structural report.

Small explicit baselines are complete. All rejected outputs preserve their previous file; no abnormal parent exit or outer timeout is observed in these probes. These results establish bounded behavior on the fixtures, not complete extraction of every PDF. The native follow-up remains an open prerequisite before 0.4.x. It is tracked separately from this internal index representation change.

## Limits

Indexing still retains text, per-document word frequencies and tree/map metadata. RSS includes allocator and complete working-view effects. Sharing ID bytes does not bound total RAM/CPU, make all workloads faster or prove semantic quality. Native PDF/cleaning/worker limits remain documented separately. No Docker execution, exhaustive security proof, mandatory model or DenkMaps/MCP integration is claimed.
