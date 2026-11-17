# Migrating to AIRust 0.2.11

Existing features, defaults and saved formats remain unchanged. Enable `storage` for a provider-neutral original-record contract and `MemoryStore`; add `sqlite-storage` for a separate AIRust-owned SQLite file. This does not migrate any application database or persist an inverted/vector index.

Read the complete store revision before each operation. Handle conflicts by rereading/reconciling. Page against one pinned revision or use bounded `read_snapshot`; a concurrent commit invalidates the complete load. Build/swap derived indexes explicitly after successful source commits. Rotate the store identity when restoring or independently copying a database lineage.

SQLite files must live in a trusted private directory. Unknown/unrelated schemas are rejected. Backup with SQLite-aware coordination; copying a live main file can omit WAL data. See [STORAGE](STORAGE.md) and [API cookbook](API-COOKBOOK.md). No DenkMaps/MCP integration is performed.
