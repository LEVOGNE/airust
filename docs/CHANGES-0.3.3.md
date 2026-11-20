# AIRust 0.3.3: Supervised PDF jobs

This patch makes the existing PDF converter optionally interruptible across blocking parser calls. `--timeout-seconds` runs conversion in an owned child, enables bounded import defaults and leaves final publication to the parent. The parent validates a bounded complete result before saving it. `--max-result-mib` controls the accepted JSON size. See the [worker contract and commands](PDF-WORKERS.md).

The child never receives the final destination. Parser failures, timeouts, excess diagnostics and rejected results preserve existing output. Success retains existing chunk/cleanup/weight/provenance settings and atomic overwrite/exclusive-create behavior. Lifecycle messages report real elapsed time rather than an invented percentage. An additional closed-pipe probe found console panics (exit 101); best-effort message writes now preserve normal file outcomes even when stdout/stderr are closed. Default library/PDF APIs and ranking algorithms retain their behavior. The CLI additionally enables the already optional tempfile dependency; no server, database, model, DenkMaps or MCP integration is added.

## Real-document and stateful checks

The actual slow first-page case stops with a one-second budget in about 1.03 seconds and keeps its previous output. With adequate time, all fifteen documents (8,960 pages, 33,708 chunks) match checked baseline text and provenance. The 134-case supervised malformed-input campaign has no unexpected parent exit or outer-watchdog timeout, and every failed conversion preserves the previous destination. A separate 4,096-case fixture mutation campaign, repeated after the console fix, contains 17 known dependency header panics as ordinary import errors. A targeted SIGKILL to the observed owned worker also preserves output and leaves no worker process.

Two index soak runs use the unchanged 0.3.2 engine on the combined real-PDF chunks. Each performs 20,000 operation attempts including invalid/rejected inputs, 9,920 exact ranking comparisons against rebuilds, four parallel readers and selected clone/serialization checks. Both retain the same final index with 34,253 entries. Single-process peak RSS is about 2,108 MiB for the default run and 1,893 MiB for the shared run, including source/canonical/rebuilt/cloned comparison copies. Runs overlap background work; no general performance improvement is claimed. [Measurements](measurements/0.3.3/) identify source versions and scope.

## Limits and release gates

The native slow path is contained by a job deadline, not fixed or accelerated. Supervision is not a hard whole-command real-time limit, OS sandbox, RAM quota or disk quota. Parent validation/publication and forced-parent termination have separate limits. The [contract](PDF-WORKERS.md) explains them.

The full check script passes 197 default and 687 all-feature tests, five Python tests, compiled examples, all 37 isolated feature entries and strict lint/docs. Rust 1.89/1.92, 229 patch API checks against 0.3.2 (31 tool skips), a fresh audit and an independent consumer also pass. Clean package/extracted tests and dry-run remain publication gates. Exact-commit CI precedes publication; registry consumer and checksum/VCS/tag/asset equality are verified afterward. This remains measured stabilization rather than a perfection claim or independent security certification.
