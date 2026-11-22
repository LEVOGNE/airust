# AIRust 0.4.12 retained results

Read the [report](../../CHANGES-0.4.12.md), [method/programs](../../validation-0.4.12/README.md), [API/performance](../../CORE-PERFORMANCE.md) and [MCP boundaries](../../MCP.md).

- `cold-timings.json` and `cold-summary.json`: 840 cold norm-cache calls, fourteen plain processes; index cloning/setup excluded. No disk-cache claim.
- `norm-memory.json`: three cold synthetic first-query heap cases. `canonical-norm-memory.json`: two full 167,942-entry first-pass heap processes. Net hint growth is separate from total allocated query bytes and whole-process RSS.
- Query records retain all 25 fixtures, 282,800 timed calls and 5,000 separate instrumented calls. Slower rows remain.
- Original/confirmation MCP records retain 60 actual servers, 5,720 schema-valid replies and 2,860 complete structured pairs. Initial and repeated requests are separate.
- Document records retain 342 exposed questions, three BM25 configurations, 1,026 complete JSON comparisons and 10,260 timings; no new holdout/model/semantic judgment.
- Construction records retain 40 plain processes and six profiles. Corpus controls validate updates/rebuilds/clones/concurrent reads/source export identities.
- `ci-example-fix.json` preserves the initial CI failure and example-only correction; `stress-example-repeats.json` retains all 24 follow-up concurrency runs.
- Current plan, frozen/extended input hashes, executable/method identities and validation/resource/host-activity records preserve conditions. Stage-start snapshots do not establish continuous isolation.

Baseline is exact registry 0.4.11; candidate is 0.4.12. Raw transcripts, PDFs, snapshots, knowledge, profiles, exact programs/locks and logs remain private and preserved. A smaller hint cache is not a comparable total-process RAM reduction. No DenkMaps integration is performed.
