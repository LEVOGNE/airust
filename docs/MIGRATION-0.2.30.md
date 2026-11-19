# Migrating to AIRust 0.2.30

Enable `disk-index` explicitly. Existing default APIs and behavior are retained. a dedicated disk-backed full-text index with atomic revisions, scoped retrieval, bounded original-record export and integrity checks.

See [contract](DISK-INDEX.md) for bounds, data ownership, failure states and compatibility. No application integration.
