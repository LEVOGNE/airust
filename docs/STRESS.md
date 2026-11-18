# Reproducible stress validation

These are opt-in synthetic development workloads, not a new required runtime subsystem or a capacity guarantee for every corpus. The library defaults and APIs are unchanged. No private data, native model or external application is used. Memory numbers include the entire child process and all its measured phases.

## Build and run

```bash
cargo build --release --locked --example stress_index
cargo build --release --locked --features snapshots,execution --example stress_snapshots
python3 scripts/measure.py /tmp/scale.json target/release/examples/stress_index scale 2000000
python3 scripts/measure.py /tmp/clones.json target/release/examples/stress_index clone 500000
python3 scripts/measure.py /tmp/concurrent.json target/release/examples/stress_snapshots 100000
```

Run smaller sizes first on a different machine. CLI guards permit 1,000–2,000,000 scale records, 1,000–500,000 retained-clone records and 1,000–100,000 snapshot records. Guards cap counts, not exact RAM. Do not run the largest profiles on a small-memory host. The recorded host has 64 GiB RAM. Run without a concurrent compiler or unrelated heavy load when comparing versions. Retain raw JSON, OS resource metadata and stderr. Each command exits unsuccessfully when its correctness assertions fail; a failed run is not recorded as a capacity success.

## What is checked

The scale fixture builds a bounded vocabulary corpus with explicit IDs and repeated content patterns. It separates 100 selective topic queries, eight common-term queries and ten absent-term queries instead of hiding them in one median. It then applies 100 text updates, removes 20 records, checks final counts and an update-only marker, and performs 80 selective queries across four simultaneous read threads. Scores are folded into a deterministic bit signature. Broad queries scan large posting sets; these results expose that cost.

The clone fixture retains three complete index clones at once, changes one record in one clone and asserts the original/other clones remain unchanged. It measures clone times and one changed clone, while whole-process RSS includes the build and all retained copies. Reuse this exact generator/count/retention pattern when evaluating sharing or memory changes. A low single-index memory value does not establish cheap copies, and a cheap clone does not establish cheap mutation.

The snapshot fixture uses 100,000 lexical records at the largest configured profile. Four readers acquire immutable views while two writers race to perform five successful commits each. Every reader checks that its state marker text and source-reference revision equal that view's generation, then searches the same view. Competing stale publications are rejected. The final generation must be eleven and the retained initial view must still expose generation one. The run has a shared 120-second cooperative execution budget plus operation/conflict guards. It uses no vectors, durable storage or external database transaction.

Reader latencies include view acquisition, marker/source checks and a selective search. Writer preparation is separate from the externally timed publish call. A rejected publish can include destruction of its large discarded candidate outside the publication lock, so its elapsed time is not the write-lock hold time. The test deliberately exercises this cost.

## CI and interpretation

Normal CI runs the stress examples at 1,000 records, including concurrent publication and retained-copy assertions. Two integration regressions add 4,000 seeded mutations checked against authoritative records/fresh rebuilds and eight concurrent immutable readers. Large profiles are executed separately and their actual outputs ship in versioned measurements. Historical results are immutable; repeat the same executable workload before/after an optimization.

This is synthetic lexical/source evidence. It does not validate real application schemas, long documents, German semantic meaning, model inference at millions of entries, HNSW recall, disk throughput, multi-process transactions or production availability. Small query sample counts make reported percentiles descriptive, not SLA estimates. The [0.2.19 report](CHANGES-0.2.19.md) records both successes and expensive cases. No DenkMaps/MCP integration is performed.
