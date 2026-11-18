# AIRust 0.2.17 acceptance plan

Extend optional shared execution control with monotonic deadlines and typed expiry errors. Default work remains unlimited. Derived/combined budgets retain the earlier parent deadline; checked duration overflow is an error. Reuse every supported cancellation checkpoint, including post-provider and pre-publication checks, without claiming forced interruption.

Validate zero/past/absent deadlines, overflow, token composition, cancellation priority, unchanged successful ranking, late providers, fallback refusal and unchanged data on expired publication. Measure checking cost and a deliberately blocking provider. Run all/isolated tests, minimum compilers, strict lint/docs, patch compatibility, audit, independent consumer, extracted package, exact-commit CI and registry verification. No application integration.
