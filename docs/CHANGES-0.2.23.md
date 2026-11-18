# AIRust 0.2.23: Explicit feedback learning

a bounded local feedback ledger and inspectable smoothed ranking adjustments.

## Validation

The complete suite passes **194 default and 580 all-feature Rust tests**, including compiled documentation, plus five Python tests. Seven integration cases cover stable-ID replay/replacement/retraction, exact document versions, namespace separation, strict round trips, limits/revision exhaustion, 1,500 seeded changes against independent full-scan counts, bounded score arithmetic, provider/recipe failures and pre-provider scope filtering.

Formatting, strict default/all-feature/isolated Clippy, rustdoc, minimum Rust 1.89/1.92, patch compatibility against 0.2.22, fresh audit, local/registry consumers, extracted-package tests, dry-run and exact-commit GitHub CI gate publication. Known dependency notices are recorded in [audit metadata](measurements/0.2.23/audit.json); these checks are not an independent security audit.

## Limits and measurements

The authored neutral-scorer fixture promotes an explicitly rated answer and restores the original order after retraction. It measures memorized preferences, not generalization or semantic understanding. The 10,000-event run measures ledger updates and JSON round trips with an identical namespace fingerprint; actual timing/size/resource values are stored in measurements. Feedback is caller supplied, unauthenticated and text-version scoped; hashes do not anonymize data. No dataset records are changed.

No default model, automatic telemetry, DenkMaps/MCP integration or unmeasured universal quality claim is introduced. See [API contract](FEEDBACK.md), [API cookbook](API-COOKBOOK.md) and [raw measurements](measurements/0.2.23/).

## Recorded fixture output

The executed release fixture returned the following measurements. These values describe this run and the explicitly synthetic setup above.

```json
{
  "after_retraction_first": "a",
  "baseline_first": "a",
  "events": 10000,
  "feedback_first": "b",
  "fixture": "authored explicit preference, not held-out generalization",
  "json_bytes": 2225606,
  "record_ms": 20.731625,
  "roundtrip_ms": 10.318083,
  "schema": 1
}
```

Raw child-process timing, CPU and peak RSS are in the adjacent `.resources.json`; timings are descriptive, not an SLA.

## Native inference boundary

These releases exercise provider-neutral algorithms and authored fixtures. The complete suite retains 1 ignored native-model test; native assets/inference are not revalidated by these runs. Synthetic features, vectors and symbolic rules do not establish real-model semantic quality.
