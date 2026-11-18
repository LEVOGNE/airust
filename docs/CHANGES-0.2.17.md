# AIRust 0.2.17: cooperative time budgets

The optional execution controls now carry monotonic deadlines. WorkControl::with_timeout constructs a checked relative budget; with_deadline selects an absolute Instant. Derived controls and combined request/provider/scorer controls preserve the earlier deadline. Unlimited work remains the default. Existing APIs, record formats, disabled response JSON and dependencies remain compatible. No DenkMaps/MCP integration is performed.

Every supported cancellation checkpoint also enforces time budgets. DeadlineExceeded is distinct from cancellation, insufficient evidence and provider failure. Cancellation wins if both conditions apply. Duration overflow returns InvalidBudget without panicking. Remaining time saturates at zero. A late provider result is discarded before fallback rules can consume it. Controlled dataset publication checks its deadline before the existing atomic commit boundary and never pretends an already completed commit was rolled back.

## Executed verification

The complete suite passes **189 default and 523 all-feature Rust tests**, including compiled documentation, plus five Python helper tests. Eight focused deadline tests cover absent/zero/past budgets, clock overflow, shortened parent/combined limits in both orders, cancellation priority, preserved successful ranking, a late custom source with fallback, expired reranking without provider invocation and unchanged data after expired publication. The execution-only deadline suite runs five cases. Earlier cancellation, provider, snapshot-concurrency and reference-model tests remain enabled.

Strict formatting, all-target default/all-feature/execution Clippy, rustdoc, minimum Rust 1.89/1.92, static patch compatibility against 0.2.16, fresh audit and an independent consumer are release gates. Extracted-package tests, publish dry run, exact-commit GitHub CI and registry/asset/tag/VCS checks gate publication. No fresh neural quality claim is made.

## Measured behavior and limits

The release-build fixture has 10,000 short records and 200 common-term controlled searches per pass. Untimed-control runs take 360.72 and 355.14 ms; deadline-enabled runs take 369.07 and 372.59 ms. Score-bit signatures are identical. This measures added checking cost on this fixture, not a universal overhead or speed claim. Whole-child peak RSS is 12.73 MiB.

The deliberately blocking source sleeps for 25 ms with a 5 ms budget. The observed call takes 30.02 ms and returns DeadlineExceeded. This counterexample is intentional: a cooperative deadline cannot force arbitrary blocking code to stop at 5 ms. Large normalization/clone/sort operations and lock waits likewise finish before the next checkpoint. Use separately configured provider/transport timeouts where needed; provider side effects are outside the rollback guarantee.

```bash
cargo build --release --locked --features execution,retrieval --example time_budget
python3 scripts/measure.py /tmp/budgets.json target/release/examples/time_budget
```

Budgets start at control construction; time in a queue or earlier shared calls counts. Create one per task when that is the intended scope. Instant is monotonic process-local clock data, not a persisted wall-clock timestamp. Specialized operations without execution integration retain their previous contracts. Raw outputs and OS resources: [measurements/0.2.17](measurements/0.2.17/). See [execution API](EXECUTION.md), [cookbook](API-COOKBOOK.md), [migration](MIGRATION-0.2.17.md) and [audit metadata](measurements/0.2.17/audit.json).
