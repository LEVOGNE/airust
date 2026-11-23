# AIRust: the route to 0.5.0

This is an authorized working sequence, not a claim that future releases already exist. Keep stability, measured performance, reliability and the Rust API/optional MCP contracts as release priorities. No DenkMaps integration is part of this sequence.

| Planned release | Work | Required evidence |
| --- | --- | --- |
| 0.4.22 | Reduce small MCP response preparation work | Complete wire/schema equality, escaped/full-output byte limits, cancellation/deadline behavior, fresh small/large MCP comparisons |
| 0.4.23 | Address remaining small repeated/multiword Core costs | Independent normalization/frequency/score checks, bounded temporary memory, both rankers, all small and large controls |
| 0.4.24 | Check response work, worker admission and deadlines under load | Slow/stalled clients, cancellation, shutdown/drain, output failures, latency tails and retained resource bounds |
| 0.4.25 | Reduce demonstrated startup/loading costs | Large retained knowledge, peak/live memory distinguished, malformed/oversized input checks and unchanged knowledge |
| 0.5.0 | Stabilize and audit the existing API/MCP release line | Full native/features/MSRV/API/consumer/package/CI validation, honest performance report and documented remaining limits |

Insert a corrective patch if a blocking defect appears. Do not publish an unverified change to satisfy a version number. Do not add integrations or large model dependencies to fill the sequence. Each released version gets its own tests, report, README/API update, GitHub tag/release, verified crates.io package and temporary-build cleanup. Keep knowledge, source inputs, rejected trials, measurements and proof.

Version 0.5.0 is a new Cargo compatibility boundary for 0.x dependencies. Any migration requirements must be documented from the actual final API comparison; a version number alone is not proof of stability or a security audit.
