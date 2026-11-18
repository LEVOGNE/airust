# Migrating to AIRust 0.2.16

This patch introduces the optional dependency-free `execution` feature. Existing uncontrolled calls, defaults, persisted formats and JSON remain unchanged. Explicitly attach a WorkControl through supported options or use controlled variants. Errors add non-exhaustive Execution variants; a cancelled call is not a no-answer result. Continue handling wildcard error cases.

Cancellation cannot terminate arbitrary blocking code. Reuse the same control for preparation and controlled publication; ordinary publish intentionally does not consult a draft's earlier cancellation. An operation that crosses its final commit checkpoint can still succeed if cancellation arrives later. See [full contract](EXECUTION.md). No DenkMaps/MCP integration is performed.
