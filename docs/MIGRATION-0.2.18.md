# Migrating to AIRust 0.2.18

The optional execution feature adds ProgressEvent, WorkPhase, ProgressObserver and WorkControl::with_observer/report plus diagnostic counters. Existing controls without an observer retain their behavior and response JSON. New enums are non-exhaustive where extensible. A known phase fraction is not a global pipeline percentage, confidence or answer ETA.

Callbacks run synchronously outside publication locks. Keep them nonblocking. Busy/reentrant slots drop notifications, and panics are counted during unwinding. Always use the operation Result and returned revision to decide completion. A completion callback cannot undo a completed publication. See [contract](EXECUTION.md). No DenkMaps/MCP integration is performed.
