# AIRust 0.2.18 acceptance plan

Add explicit optional progress observers to existing execution controls, with actual phase counts, unknown totals and no guessed answer ETA. Preserve default response JSON and dependencies. Callback panics must remain telemetry diagnostics. Busy/reentrant observers must not deadlock. Publication notifications must run outside all publication locks and cannot turn an already committed success into an apparent failure.

Validate count invariants, sequencing, observer composition, concurrent and reentrant delivery, panic recovery, real search counts, unchanged response output, cancellation from observers and safe publication callbacks. Measure overhead with a bounded counter observer. Run complete/isolated tests, minimum compilers, strict lint/docs, patch compatibility, audit, consumer/package/CI/registry verification. No application or transport integration.
