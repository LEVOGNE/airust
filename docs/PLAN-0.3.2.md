# AIRust 0.3.2 acceptance plan

Harden optional PDF imports without changing the default engine or successful checked corpus output.

- Add validated, opt-in file/page/text/chunk budgets, including overlap amplification and cumulative document counts.
- Preserve existing APIs, enum variants and loader auto-traits. Keep the previous unbounded policy unless explicitly selected.
- Reproduce and fix non-Unicode CLI argument panics and raw controls in the CLI's own terminal output.
- Preserve old destinations on all failed conversions; retain concurrent-output regression tests.
- Compare all fifteen PDFs and the same independent page probes with 0.3.1; repeat the deterministic 134-case malformed-input campaign.
- Retain local knowledge and raw measurements while excluding the lab from Git, crate and Docker context.
- Run the full suite, isolated features, strict lint/docs, minimum Rust, patch API comparison, fresh audit, external consumer, extracted package tests, dry-run and exact-commit CI.
- Verify registry consumer, artifact checksum, VCS commit, tag and GitHub release asset after publication.

No DenkMaps/MCP integration, hard native CPU/RAM guarantee, OCR addition or 0.4.0 feature expansion.
