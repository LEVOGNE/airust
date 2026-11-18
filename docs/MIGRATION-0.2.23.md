# Migrating to AIRust 0.2.23

Enable `feedback` explicitly. Existing default behavior, storage data and retrieval JSON remain unchanged. a bounded local feedback ledger and inspectable smoothed ranking adjustments.

The authored neutral-scorer fixture promotes an explicitly rated answer and restores the original order after retraction. It measures memorized preferences, not generalization or semantic understanding. The 10,000-event run measures ledger updates and JSON round trips with an identical namespace fingerprint; actual timing/size/resource values are stored in measurements. Feedback is caller supplied, unauthenticated and text-version scoped; hashes do not anonymize data. No dataset records are changed.

See [full API contract](FEEDBACK.md). No DenkMaps/MCP integration is performed.

Event IDs are global upsert keys within a ledger. Namespace isolates preference matching, but does not create a separate event-ID key space. Use separate tenant ledgers or globally prefixed IDs, and authorize replacement/retraction of existing events.
