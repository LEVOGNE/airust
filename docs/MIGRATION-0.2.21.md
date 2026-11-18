# Migrating to AIRust 0.2.21

Existing update calls, defaults and persisted/response formats remain unchanged. TextIndex gains Clone. PreparationStats adds diagnostic fields for reused text, metadata-only puts and unchanged puts; its text_entries_rebuilt value now counts actual changed-record analysis. Non-exhaustive consumers should continue accepting additive fields. These receipts are not a new backup format.

Metadata changes preserve analysis while updating returned payloads. The exact serialized metadata representation is retained, including signed-zero/integer-vs-float distinctions. Any original text byte change refreshes positions even when normalized terms agree. shared-index remains optional; model refresh and external transaction ownership are unchanged. See [contract](INCREMENTAL.md). No DenkMaps/MCP integration is performed.
