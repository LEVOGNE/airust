# AIRust engineering priorities

The user's standing priorities are memory safety and performance, together with stability, robustness and honesty. Treat these as release criteria throughout the project.

- Preserve `#![forbid(unsafe_code)]` in owned library code. Keep allocations, file/network input, output, concurrency and model work explicitly bounded; retain original data on validation/publication failure.
- Measure representative behavior before claiming an improvement. Compare compatible baseline/candidate builds, retain raw samples and every regression, and distinguish warm retrieval from complete request, startup and external-service costs.
- Prefer small coherent modules and existing validated APIs. Keep default features small and preserve documented ownership, scopes, revisions, error categories and compatibility boundaries.
- Run meaningful tests for changed behavior, feature/platform/compiler gates and scoped Miri controls where applicable. State interpreter/FFI exclusions and failures explicitly; a partial or unavailable run is not a pass.
- Preserve frozen datasets, source/model identities and historical evidence. Never present generated examples, replayed user questions or self-authored judgments as independent customer traffic or human review.
- Do not mark a quality or adoption gate complete without its actual inputs and observed result. No unmeasured speed, safety, accuracy, token-saving or production-capacity claims.

Follow CONTRIBUTING.md for release, backend provenance and compatibility requirements. Original knowledge, user databases and retained evidence are not build artifacts.

## Resume roadmap work

For roadmap/release continuation, read docs/SESSION-HANDOFF.md first, then ROADMAP.md and docs/WORK-PROGRAM-NEXT.json. Verify current Git/release receipts before repeating completed work. Keep the handoff and work-program statuses current after each coherent step; retain original test/measurement identities and external-input gates. The user's planning preference is approximately 70% quality/documentation/integration/real validation and 30% justified new capabilities. Do not blindly implement or publish every roadmap version, duplicate existing control mechanisms, or create nominal crates merely to match conceptual component names.
