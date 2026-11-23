# AIRust: completed stability route to 0.5.0

Keep Core/MCP runtime source unchanged from 0.4.25, retaining earlier measured fixes. Add a fixed-seed 250-step state/cache/clone regression against independent records, with a separately reduced Miri form. Compare exact 0.4.21/0.4.25/0.5.0 through 30 finite million-entry/500,000-entry clone processes and all retained RPC/startup/heap/retrieval controls. Preserve slower outcomes. Document explicit Cargo 0.5 selection, unchanged calls/formats and version-qualified MCP schema refresh. No new algorithm, dependency, default feature or DenkMaps integration.

| Release | Implemented work | Evidence |
| --- | --- | --- |
| 0.4.22 | Reuse bounded MCP payload checks and owned JSON | [Report](CHANGES-0.4.22.md) |
| 0.4.23 | Borrow short repeated normalized query words | [Report](CHANGES-0.4.23.md) |
| 0.4.24 | Keep final reply work admitted; correct late-error progress | [Report](CHANGES-0.4.24.md) |
| 0.4.25 | Free raw knowledge JSON before indexing | [Heap versus RSS evidence](CHANGES-0.4.25.md) |
| 0.5.0 | Independent state tests, finite scale/clone controls and upgrade | [Report](CHANGES-0.5.0.md), [migration](MIGRATION-0.5.0.md) |

Each release has its own validation and benchmark record. Publication completion requires four exact-commit CI jobs, a fresh registry consumer, tag/VCS/registry/asset checksum agreement and temporary-build cleanup. Knowledge, rejected trials and all measurements remain. [Power metadata errata](BENCHMARK-ERRATA.md) correct older labels without replacing samples. Cargo 0.5 is a new compatibility line, not a 1.0, zero-defect, semantic-quality or security-audit guarantee. Future work should follow real defects and representative application measurements.
