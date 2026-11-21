# AIRust 0.4.2 measurements

Public artifacts contain counts, hashes and timing samples without private document/query text. See [the release report](../../CHANGES-0.4.2.md) and [reproduction programs](../../validation-0.4.2/README.md).

- `validation.json`: default/all-feature/MSRV/lint/API scope.
- `core-correctness.json`: 2,520 exact old/new ranking checks, mutations, retained readers and cache interoperability.
- `index-heap.json`: default-index tracked allocations and build/warm/clone/drop phases with positive probes; no instrumented latency claim.
- `core-cost.json` / `core-cost-samples.json`: twelve fresh ordinary-call processes, first/repeated passes, per-query timings/hit counts, builds and child RSS.
- `mcp.json` / `mcp-cases.json`: final 35 snapshot connections plus the complete canonical collection, direct-API equality and exact readbacks.
- `mcp-schemas.json`: advertised schemas, independent conformance of 1,192 actual structured results and schema bundle hash.
- `mcp-cost.json` / `mcp-cost-samples.json`: thirty paired old/new server processes, full stdio timings and child RSS without validator/transcript overhead.
- `matching.json`: positive ordinary/controlled matching and relation equality against 0.4.1.
- `retention.json`: unchanged original PDFs, eight exports and 35 retained snapshots.
- `environment.json`, `audit.json`, `backend-audit.json`, `backend.json`: compiler/profile, dependency advisories and unchanged parser checks.

Warm caches and uncontrolled desktop activity limit generalization. Payload accounting is not a process-RAM quota. Fixed literal probes are not human relevance labels. Source text, complete expected/transcript/cache data, failed attempts and profiler traces stay private. Publication identities and actual registry checks are recorded separately in the release verification artifact.
