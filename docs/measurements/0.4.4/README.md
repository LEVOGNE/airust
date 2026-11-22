# AIRust 0.4.4 measurements

These source-free artifacts describe current 0.4.4 versus registry 0.4.3. See [the report](../../CHANGES-0.4.4.md) and [reproduction](../../validation-0.4.4/README.md).

`heap-edges.json` adds eight authored 64-entry allocation distributions. It includes few-long-word cases where cumulative bytes rise despite fewer allocation calls. Completed live heaps remain equal. These traces complement the real-document profile and do not supply latency or process-RAM guarantees.

`mcp-canonical-followup.json` retains one fixed six-process follow-up of an initial canonical latency outlier. The original 30-process samples are unchanged. Both pooled query medians and medians of process medians are available; no process is removed.

- `validation.json`: default/all-feature, independent BM25 oracle, feature, MSRV, lint and API scope.
- `core-correctness.json`: 2,520 exact old/new ranking, mutation, retained-reader and cache comparisons.
- `full-retrieval-compatibility.json`: complete 36 direct-oracle result files agree with 0.4.3.
- `core-cost.json` / `core-cost-samples.json`: twelve fresh ordinary-call children, builds, first/repeated query passes and lifetime RSS.
- `index-heap.json`: retained/peak index heap and build/warm/clone/drop phases. Cumulative allocation traffic falls while retained heap remains identical; timings under instrumentation are not benchmarks.
- `mcp.json`, `mcp-cases.json`, `mcp-schemas.json`: retained-document and canonical protocol/oracle/schema checks.
- `mcp-cost.json` / `mcp-cost-samples.json`: thirty paired fresh servers, startup/round-trip costs and child RSS.
- `mcp-churn.json`: published-baseline serial allocator comparison and same-process parallel connection cleanup.
- `miri.json`: interpreter flags, selected ownership/ranking paths and multi-seed lock/Tree Borrows scope.
- `overnight-soak.json`: fixed-seed default/shared differential mutations, concurrent reads, cancellation and cache recovery counts.
- `scale.json`: fresh million/two-million short synthetic record and retained-clone checks under deliberate concurrent load.
- `review.json`: public-source review scope, reproduced cross-connection shutdown defect and rejected batch hypothesis.
- `matching.json`: positive ordinary/controlled matching and relation compatibility.
- `retention.json`, `environment.json`, `audit.json`, `backend-audit.json`, `backend.json`: preserved data, environment/advisories and unchanged parser checks.

Warm caches and uncontrolled desktop activity limit generalization. Payload accounting is not a RAM quota. Literal probes are not relevance labels. Exact-commit CI and actual registry/source identities are verified separately with release artifacts. Private source, expected responses/transcripts/caches, failed attempts and profiler traces stay local.
