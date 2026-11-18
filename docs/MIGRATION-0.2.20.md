# Migrating to AIRust 0.2.20

This patch adds optional shared-index=[] without changing defaults, dependencies or runtime methods. Select it explicitly when retaining several lexical index copies or snapshots. Existing clone/edit ownership and serialized data remain compatible. The snapshots feature does not enable sharing automatically.

Clones still copy outer maps and keys. Shared buffers detach on mutation; a common-term edit can copy a large posting map. A single-owner workload can pay allocation/reference-counting costs, so inspect the measured tradeoffs. Nothing becomes process-shared or synchronized with an external database. See [contract](SHARED-INDEX.md). No DenkMaps/MCP integration is performed.
