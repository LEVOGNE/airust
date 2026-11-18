# AIRust 0.2.25: Evidence-backed rule proposals

an optional bounded rule interpreter that proposes conclusions with explicit premises, sources, revision, conflicts and incomplete-result status.

## Validation

The complete suite passes **194 default and 602 all-feature Rust tests**, including compiled documentation, plus five Python tests. Eleven integration cases cover thirty seeded BFS-reference closures with cycles, safe binding, deterministic proof DAGs, source integrity, missing support, explicit negative conflicts, hidden bridges, round/join/proposal/output limits and cooperative cancellation/deadlines/progress.

Formatting, strict default/all-feature/isolated Clippy, rustdoc, minimum Rust 1.89/1.92, patch compatibility against 0.2.24, fresh audit, local/registry consumers, extracted-package tests, dry-run and exact-commit GitHub CI gate publication. Known dependency notices are recorded in [audit metadata](measurements/0.2.25/audit.json); these checks are not an independent security audit.

## Limits and measurements

The synthetic prerequisite-chain example reports actual joins, rounds, output size and timing/resources. It demonstrates declared symbolic rules, not natural-language understanding. Every proposal carries an earlier-premise proof chain and registered source IDs. Registration is not source verification. Missing support or limits produce complete=false; inferred proposals never change canonical facts. The final application guide describes a future adapter; no DenkMaps schemas or production data were changed.

No default model, automatic telemetry, DenkMaps/MCP integration or unmeasured universal quality claim is introduced. See [API contract](REASONING.md), [API cookbook](API-COOKBOOK.md) and [raw measurements](measurements/0.2.25/).

The [complete application integration guide](APPLICATION-INTEGRATION.md) covers the optional APIs through 0.2.25. No actual application integration is performed.

## Recorded fixture output

The executed release fixture returned the following measurements. These values describe this run and the explicitly synthetic setup above.

```json
{
  "elapsed_ms": 2.8760410000000003,
  "fixture": "explicit synthetic prerequisite chain, not inferred natural-language truth",
  "input_facts": 12,
  "response_bytes": 24791,
  "schema": 1,
  "result_summary": {
    "complete": true,
    "join_trials": 13368,
    "missing_support": [],
    "missing_support_count": 0,
    "revision": "demo-revision-1",
    "rounds": 5,
    "schema_version": 1,
    "stop": "fixed_point"
  },
  "proposals": 66,
  "conflicts": 0,
  "example_proof": {
    "fact_ids": [
      "edge-0",
      "edge-1",
      "edge-10",
      "edge-11",
      "edge-2",
      "edge-3",
      "edge-4",
      "edge-5",
      "edge-6",
      "edge-7",
      "edge-8",
      "edge-9"
    ],
    "id": 66,
    "premises": [
      {
        "id": 22,
        "kind": "proposal"
      },
      {
        "id": 56,
        "kind": "proposal"
      }
    ],
    "rule_fingerprint": "0eb8fb28d8e72eb77c65af704db8dc6fb5352101f5c5d5a01bf2c6713073a4ff",
    "rule_id": "transitive",
    "source_ids": [
      "manual-source"
    ],
    "statement": {
      "negative": false,
      "object": "step-12",
      "predicate": "requires",
      "subject": "step-0"
    }
  }
}
```

Raw child-process timing, CPU and peak RSS are in the adjacent `.resources.json`; timings are descriptive, not an SLA.

## Native inference boundary

These releases exercise provider-neutral algorithms and authored fixtures. The complete suite retains 1 ignored native-model test; native assets/inference are not revalidated by these runs. Synthetic features, vectors and symbolic rules do not establish real-model semantic quality.
