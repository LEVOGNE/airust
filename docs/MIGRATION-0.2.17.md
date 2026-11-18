# Migrating to AIRust 0.2.17

The optional execution feature adds WorkControl::with_timeout, with_deadline, deadline and remaining. Existing default/uncontrolled behavior remains unchanged. ExecutionError is non-exhaustive and adds DeadlineExceeded and InvalidBudget. Continue handling wildcard errors and distinguish interruptions from no-answer results.

Budgets start at construction. Reusing a control across several calls shares one deadline. Derived controls can shorten that deadline but cannot extend it. A blocked provider can return after the budget; AIRust then rejects its late response. Controlled atomic publication retains its existing commit-boundary semantics. See [execution guide](EXECUTION.md). No DenkMaps/MCP integration is performed.
