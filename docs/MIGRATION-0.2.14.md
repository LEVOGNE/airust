# Migrating to AIRust 0.2.14

Existing APIs, empty default features, prefix/relevant excerpt modes and persisted formats remain unchanged. Opt into `snapshots` for an additional coordinated owner over existing Engine/SemanticIndex and ProvenanceStore. It includes storage validation/provider-neutral embedding/provenance interfaces, not SQLite, HTTP or native models.

Acquire one immutable DatasetSnapshot per request. Build DatasetRecords with complete source lists, prepare changes/full reconciliation against its revision and publish with CAS. Retained readers stay coherent. Unchanged literal text reuses vectors; new/changed text requires a matching provider in semantic mode. Sources are replaced on every Put and default to absent, preventing silent retention after edits. Explicit model changes rebuild all vectors.

The optional separate source_revision is a caller-provided external marker with monotonic lineage checks. It does not make SQLite and RAM one transaction. Reconcile from a consistent full source image, preserve source/access rechecks and handle conflicts explicitly. Bound preparation queues and retained views because staged index/source bookkeeping is copied. Existing app-owned source history remains authoritative. See [SNAPSHOTS](SNAPSHOTS.md) and [API cookbook](API-COOKBOOK.md). No DenkMaps dependency, files or data are changed.
