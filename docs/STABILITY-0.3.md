# AIRust 0.3.x stability gates

The 0.3.x series stabilizes existing capabilities before the next feature stage. A release number alone does not establish readiness. New patches require a demonstrated defect or measured benefit, preserved knowledge and reproducible checks.

| Gate | Required evidence |
| --- | --- |
| Existing APIs | Default/all-feature tests, strict checks, feature isolation, minimum Rust versions and patch compatibility. |
| Real documents | Identified sources and hashes, page coverage, retained imports, independent reference checks and visible extraction limits. |
| Retrieval and matching | Fixed queries, complete result comparisons and separate quality/cost statements. |
| Updates | Incremental state agrees with a complete rebuild; failed work does not replace existing knowledge. |
| Persistence | Saved originals remain unchanged; reconstructed working views produce the same results. |
| Concurrent readers | One consistent view per request, repeatable results and explicit update-conflict behavior. |
| Resource behavior | Separate build/search/update/persistence costs, fresh-process peak RSS and scoped stress tests. |
| Failures | Bounded damaged-input tests, deadlines/owned-worker failures and old-output preservation. |
| Publication | Clean package, extracted tests, exact-commit CI, registry-only consumer and matching artifact identities. |

The [0.3.10 report](CHANGES-0.3.10.md) records the expanded PDF corpus and cleaner fix. The [0.3.11 report](CHANGES-0.3.11.md) records matching and real-knowledge stability work. The [0.3.12 report](CHANGES-0.3.12.md) measures shared identifiers, and the [0.3.13 report](CHANGES-0.3.13.md) addresses the demonstrated native structure-limit omissions. Each report distinguishes observed results from inherited proof. Completion of these gates permits planning a focused 0.4.0; it is not a claim that all possible documents or workloads are flawless.

## Boundaries that remain explicit

Native PDF reading order/glyph decoding and cleaning are heuristic. Image-only PDFs require external OCR. Password entry is not exposed. Synchronous native/library matching calls have no hard process CPU/RAM bound. The supervised CLI worker adds a wall-clock budget rather than an OS memory sandbox. Optional model quality depends on supplied weights and corpus; lexical tests do not validate every model. Application database transactions and access-control rules remain the application's responsibility.

## Work toward 0.4.x

The first proposed scope is a general document-and-passage contract. It should identify source/configuration revisions and passage positions without confusing cleaned-text offsets with original PDF byte positions. It should work with existing generic entries and optional adapters, preserve failure atomicity and expose real phase/count progress.

Before implementing it, retain failing examples for filename collisions, changed-source revisions, Unicode offsets and cleaning-induced text changes. Compare a small optional prototype against the existing APIs. Keep model loading, application integrations and transport formats separate. The [0.4 preparation plan](PREPARING-0.4.md) maps current APIs to concrete acceptance cases. Controlled matching and bounded batch ingestion can follow as separately measured steps. No automatic 0.4.0 publication follows merely from finishing this checklist.


## Resolution of the native 0.3.12 findings

The 0.3.12 structural tests reproduce skipped text in deeply nested Form XObjects and incomplete fallback page counting after a depth limit. These were existing 0.3.4 backend behaviors. Version 0.3.13 pins backend 0.3.5 and turns all eight demonstrated incomplete successes into explicit errors. Supported known witnesses remain complete; failed direct and supervised imports preserve old output. Repeated extraction tests reject partial cached success while allowing complete results from valid caches. See [the original findings](CHANGES-0.3.12.md) and [the fix, measurements and limits](CHANGES-0.3.13.md).

The 64-level recursive prototype failed a default-stack test and was rejected. The published fix retains existing native limits. Other best-effort recovery and extraction heuristics remain explicit boundaries. Continue 0.3.x maintenance for a new reproducible defect; do not generate releases solely to fill version numbers.
