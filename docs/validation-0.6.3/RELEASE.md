# AIRust 0.6.3 release verification

On 2026-11-27 the user authorized publishing **0.6.3**, on the condition that every R1 criterion passed in the third soak run. The approval covered the master branch, an annotated tag, crates.io and a GitHub Release. R1 passed on 2026-11-28 ([results](RESULTS.md)). The published 0.6.2, 0.6.1 and 0.6.0 crates and the PDF backend 0.3.5 are unchanged. The backend is not republished.

## Receipts

- The [GitHub release](https://github.com/LEVOGNE/airust/releases/tag/v0.6.3) carries the exact crate, checksums and the release manifest.
- The [registry version](https://crates.io/crates/airust/0.6.3) must match the tag commit and the crate checksum.
- The local record is `local-lab/release-0.6.3-2026-11-28/`. `final-closeout.json` exists only after all checks are complete.

## Release gates

- **Fresh checks on the release commit:**
  - `full` profile of the [test control](../TESTPLAN.md)
  - clean `cargo package` and dry run, with a content inspection
  - RustSec audit
  - extracted-package consumers with Rust 1.89 and 1.92 (real worker cycles)
  - exact-source master and tag CI, including `native-worker-linux`
  - anonymous registry download, compared byte for byte
  - registry consumers
- **Robustness evidence from the [criteria](CRITERIA.md) runs.** These ran on the same worker source, and the release commit changes only version and documentation text.
  - R1 passed in the third run; the two earlier failures remain recorded.
  - R2 passed.
  - R3 passed in CI run `46057398768`.

## Limits

- **Latency tail depends on host load.** The three-hour drift criterion failed twice under heavy foreign load and passed on a host that was quiet at start. The worker's latency tail is therefore stated as stable on an otherwise idle host. It is not stated as independent of competing load.
- **Memory.** Active native models exceed 1 GiB per worker, and each pooled worker holds its own model copy in memory and on disk. Direct in-process loading still retains allocator memory under the default macOS allocator.
- **Not covered.** Windows and Intel native execution, actual 1-GiB hardware and E5 inference.
- **Independent review.** Zero of fifty independently reviewed questions are established.
- **CSP.** Only the report-only policy is in place; enforcement needs an interface rewrite.
- **Parallel callers.** The pool bounds parallel callers but does not provide fairness or priority.
