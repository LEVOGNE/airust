# AIRust 0.2.11 acceptance plan

Deliver one object-safe record-storage contract, a memory reference implementation and a standalone durable SQLite adapter. Preserve the default library and existing APIs. Every nonempty accepted batch commits once with its revision; every failed batch changes neither. Bound encoded entries, metadata, batches, pages and snapshots. Fence reads and writes with namespace, identity and generation.

Validate shared adapter conformance, independent randomized reference state, stale/concurrent connections, reopening/read-only use, SQL fault rollback, generation exhaustion, invalid schemas/records, safe file paths, malformed third-party pages and writes during snapshot loading. Measure synthetic 10k/100k records and disclose that the search index remains in RAM. Run complete release checks, static patch compatibility, minimum compilers, independent consumer, fresh advisory audit, extracted package and publication verification. No application integration.
