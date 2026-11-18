# AIRust 0.2.19 acceptance plan

Ship reproducible guarded scale, retained-index-copy and concurrent snapshot workloads without adding mandatory runtime modules. Execute two million synthetic lexical records with concurrent readers, a 500k-record/three-clone baseline, and 100k source-coherent snapshots with four readers/two competing writers. Separate broad/selective/missing queries, preparation/publication times and whole-process resources. Report failures and costly cases honestly.

Add seeded long mutation and concurrent-reader regressions. Run CI smoke profiles at 1,000 records plus the complete test/lint/docs/minimum-Rust/compatibility/audit/consumer/package/CI/registry gates. Preserve all existing APIs/defaults and no application integration.
