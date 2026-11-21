# AIRust 0.3.11: Avoid discarded character comparisons

Token-overlap matching previously calculated a full character-edit distance for every candidate passing the word-overlap threshold, even when only ten entries were returned. The score for this strategy uses word overlap, so those discarded distances cannot improve its ranking.

Version 0.3.11 calculates character explanations only after selecting the token winners. Complete returned values remain compatible, including exact-match flags, character similarities, token similarities, scores and stable ID ordering. Exact/fuzzy strategies keep their existing behavior. Public methods and JSON fields do not change. See [API and limits](TOKEN-MATCHING.md).

## Evidence

The retained corpus contains 160,085 entries from 33 real PDFs / 46,756 physical pages. Source snapshots, all canonical exports and previous release evidence remain retained locally. New index mutations stay in disposable working views. Private text and source paths remain outside public reports.

Three full-comparison regressions cover Unicode, empty/tokenless queries, thresholds, zero/small/large limits, ties, updates and JSON round trips. Fixed real-corpus matching cases compare complete returned IDs/order and bitwise scores/similarities. The separate real-knowledge workload compares incremental updates with full rebuilds, JSON reconstruction and concurrent readers. See [reproduction](validation-0.3.11/README.md) and [0.3.x stability gates](STABILITY-0.3.md).

## Executed results

All 200 default tests, 711 all-feature tests, 32 final documentation examples, 37 isolated feature checks, strict lint/docs and five Python tests pass. Ten maintained-backend regressions and provenance checks pass. Rust 1.89/1.92 and both independent local consumers pass; patch API checks pass 229 checks with 31 tool skips. Dated audits find zero known vulnerabilities, retaining three main and four standalone-backend maintenance warnings.

Every one of the 33 PDFs was freshly imported with a supervised 90-second worker. All 160,085 chunks match the retained 0.3.10 text and metadata, with matching default training output/weight. The native/worker/persistence sources and backend remain unchanged. The prior independent cleaner/excerpt proof and 12,288 native mutation executions remain explicitly inherited from 0.3.10 rather than claimed as another fresh campaign. An initial harness assumption about the Text JSON envelope was corrected; its first run remains retained.

The 24 real-corpus matching cases preserve complete returned IDs/order and every score/similarity bit. Another 1,024 generated known-script cases check token/fuzzy/exact strategies, thresholds and limits. The real-corpus soak passes 32 complete update/rebuild/JSON epochs plus concurrent reader and clone checks. It takes 13.41 minutes under other validation load, with 6.94 GiB process peak RSS. This workload retains multiple complete reference views and JSON strings. It does not measure a single engine's requirement or prove absence of every possible leak.

## Matched token-overlap costs

Six fresh alternating release processes run the same fixed 24 cases, giving three samples per case/version. Each process builds one default-feature index of 160,085 entries. File caches are warm; deliberate build/validation/soak work has finished. Other desktop activity is uncontrolled. Method time excludes result projection/serialization. All paired complete results remain identical.

| Query | Threshold / limit | Method ms, 0.3.10 → 0.3.11 |
| --- | --- | ---: |
| memory allocation function | 0.0 / 10 | 596.162 → 21.686 |
| memory allocation function | 0.1 / 10 | 22.852 → 21.214 |
| memory allocation function | 0.8 / 10 | 22.363 → 20.927 |
| matrix vector array | 0.0 / 10 | 295.059 → 14.808 |
| matrix vector array | 0.1 / 10 | 15.201 → 14.440 |
| matrix vector array | 0.8 / 10 | 14.319 → 13.944 |
| Speicher Verarbeitung Unicode | 0.0 / 10 | 29.440 → 0.739 |
| Speicher Verarbeitung Unicode | 0.1 / 10 | 0.576 → 0.525 |
| Speicher Verarbeitung Unicode | 0.8 / 10 | 0.527 → 0.496 |
| Longer fixed query | 0.0 / 10 | 4644.024 → 98.882 |
| Longer fixed query | 0.1 / 10 | 100.722 → 98.529 |
| Longer fixed query | 0.8 / 10 | 102.793 → 98.477 |

The full query text for the longer case remains in the aggregate. The full fixed 24-query process has a median wall time of 18.568 → 7.633 seconds, including index construction. Index build itself is 5.931 → 5.896 seconds. Process peak RSS is 2.650 → 2.650 GiB. The change removes discarded edit-distance work; it does not establish a general RAM, PDF-parser or semantic-quality improvement.

## Real-knowledge validation costs

Nine fresh processes use fixed prefixes of the same read-only real export, three processes per size and three mutation/rebuild/JSON epochs per process. Each process also checks four concurrent readers. Load/hash/parse, index construction, checked-query, rebuild and JSON-restore categories remain separate in the aggregates. The checked-query timer includes result projection and equality checks. The table reports the median of the three per-run p95 values, not a pooled p95. The process wall/RSS columns include full comparison views and verification work.

| Real entries | Checked BM25 p95 ms | Checked TF-IDF p95 ms | Validation process s | Process peak GiB |
| ---: | ---: | ---: | ---: | ---: |
| 10,000 | 0.661 | 5.938 | 3.101 | 0.599 |
| 100,000 | 10.169 | 97.420 | 36.094 | 3.784 |
| 160,085 | 18.034 | 206.847 | 65.275 | 6.181 |

## Separate synthetic workloads

The remaining 36 fresh processes exercise short synthetic records, retained clones, snapshot concurrency and retained SQLite databases. These data differ from the real PDF entries; a million short records is not a million real PDF passages. Default and shared-index scale/clone binaries differ only by the shared-index feature. Snapshot runs additionally enable execution/snapshots. SQLite uses its documented native scoring recipe; timing does not prove equivalence with the RAM scoring recipe.

| Workload | Entries | Configuration | Runs | Process s | Process peak GiB |
| --- | ---: | --- | ---: | ---: | ---: |
| clone | 500,000 | default | 3 | 2.548 | 3.376 |
| clone | 500,000 | shared | 3 | 1.757 | 1.171 |
| disk | 100,000 | disk-index; isolated retained DB | 3 | 0.779 | 0.008 |
| disk | 1,000,000 | disk-index; isolated retained DB | 3 | 8.231 | 0.009 |
| scale | 10,000 | default | 3 | 0.048 | 0.020 |
| scale | 10,000 | shared | 3 | 0.047 | 0.019 |
| scale | 100,000 | default | 3 | 0.482 | 0.180 |
| scale | 100,000 | shared | 3 | 0.482 | 0.170 |
| scale | 1,000,000 | default | 3 | 5.378 | 1.771 |
| scale | 1,000,000 | shared | 3 | 5.285 | 1.666 |
| snapshots | 100,000 | shared-index,execution,snapshots | 6 | 0.309 | 0.187 |

Scale runs check search categories, updates/deletions and four readers. Clone runs retain three views and change one while checking the others. Six snapshot runs check four readers/two competing writers, ten total committed updates and old-view consistency. Six isolated SQLite databases are built, reopened, searched and integrity-checked, then retained locally. SQLite process RSS excludes operating-system file cache; this is not total machine memory. No synthetic data enters canonical knowledge.

The [public aggregates](measurements/0.3.11/) and [reproduction sources](validation-0.3.11/README.md) retain these scopes and full sample categories. Clean packaging, extracted tests, dry-run, exact-commit Linux CI, registry-only consumers and matching crate/VCS/tag/assets finish the publication gates; their final outcomes are retained in the release verification asset.

## Limits

Candidate collection and token scoring still depend on the corpus. Returned character explanations still cost edit-distance work proportional to their text lengths and requested count. Direct matching has no hard deadline or OS memory sandbox. Timing categories remain separate; a supplied-text/matching improvement does not prove faster PDF parsing or semantic quality. No exhaustive security proof, Docker execution, new model, DenkMaps or MCP integration is claimed.
