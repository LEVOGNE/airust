# Benchmark metadata corrections

The measurement exporter previously used a hard-coded AC power label. Comparison with the preserved environment records found the following campaigns started on battery power. Their timing samples have not been replaced, removed or remeasured. Current reports and resource files correct the label. Previously published crates and tags remain immutable; this page records the correction to those copies.

| AIRust campaign | Observed power at campaign start | Original samples |
| --- | --- | --- |
| 0.4.11 | Battery Power | Preserved unchanged |
| 0.4.12 | Battery Power | Preserved unchanged |
| 0.4.15 | Battery Power | Preserved unchanged |
| 0.4.16 | Battery Power | Preserved unchanged |
| 0.4.18 | Battery Power | Preserved unchanged |
| 0.4.19 | Battery Power | Preserved unchanged |
| 0.4.22 | Battery Power | Preserved unchanged |
| 0.4.24 | Battery Power | Preserved unchanged |

A single start observation does not prove the machine used the same power mode for an entire campaign. Stage CPU/load samples also do not establish uninterrupted idleness. Compare alternating versions within their stated campaigns; do not treat absolute timings across releases as measurements on a controlled, always plugged-in host. The data does not identify how much a power-mode change affected any result.

The exporter now derives the label from the captured power record, reports only the initial observation, and retains unknown states rather than assuming AC. This correction changes metadata and interpretation, not source knowledge, ranking results or runtime behavior.
