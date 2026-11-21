# AIRust 0.4.3 measurements

These source-free artifacts describe current 0.4.3 versus registry 0.4.2. See [the report](../../CHANGES-0.4.3.md) and [reproduction](../../validation-0.4.3/README.md).

- `validation.json`: default/all-feature, independent BM25 oracle, feature, MSRV, lint and API scope.
- `core-correctness.json`: 2,520 exact old/new ranking, mutation, retained-reader and cache comparisons.
- `full-retrieval-compatibility.json`: complete 36 direct-oracle result files agree with 0.4.2.
- `core-cost.json` / `core-cost-samples.json`: twelve fresh ordinary-call children, builds, first/repeated query passes and lifetime RSS.
- `index-heap.json`: retained/peak index heap and build/warm/clone/drop phases. Cumulative allocation traffic increases while retained heap falls; timings under instrumentation are not benchmarks.
- `mcp.json`, `mcp-cases.json`, `mcp-schemas.json`: retained-document and canonical protocol/oracle/schema checks.
- `mcp-cost.json` / `mcp-cost-samples.json`: thirty paired fresh servers, startup/round-trip costs and child RSS.
- `matching.json`: positive ordinary/controlled matching and relation compatibility.
- `retention.json`, `environment.json`, `audit.json`, `backend-audit.json`, `backend.json`: preserved data, environment/advisories and unchanged parser checks.

Warm caches and uncontrolled desktop activity limit generalization. Payload accounting is not a RAM quota. Literal probes are not relevance labels. Exact-commit CI and actual registry/source identities are verified separately with release artifacts. Private source, expected responses/transcripts/caches, failed attempts and profiler traces stay local.
