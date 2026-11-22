# AIRust 0.4.7 retained measurement records

Read the [executed report](../../CHANGES-0.4.7.md) and [reproduction protocol](../../validation-0.4.7/README.md) before interpreting these records. All 282 previous questions are exposed development data. The sixty new questions are assistant-authored known-source cases, not independent human judgments or production traffic.

- `retrieval-summary.json` and `retrieval-scored-rows.json`: reference-ID/quotation ranks and context counts for all 342 queries and three configurations. Negatives have no positive reference label.
- `retrieval-timings.json`: 5,130 warm context/IPC samples; not isolated BM25 timing.
- `scored-rows.json`, `summary.json` and `paired-comparisons.json`: 180 fresh local replies on sixty new questions, technical failures, descriptive answer proxies, real token/cache counters and costs. The summary's empty development-answer section has zero replies; historical answers are not recounted.
- `coverage-components.json` and `coverage-oracle.json`: selected numeric scoring components and the independent canonical-token/formula/order check over 31,744 components. Query terms, source IDs and passage text are omitted.
- `default-compatibility.json`: 684 complete old/new literal/prepared JSON response comparisons.
- `audit-summary.json` and `audit-freeze-receipt.json`: 36 masked same-assistant semantic judgments fixed before opening their mapping. No independent-human accuracy claim follows.
- `development-*.json`: unsuccessful rank/reserve experiments and coverage/raw-score blends on exposed questions, with no new-holdout tuning.
- `resources.json`, `retriever-rss.json`, `retrieval-builds.json`, `settings.json`, `holdout-design.json` and `answer-order.json`: actual configuration, host/model identity, process samples, source distribution and balanced execution order.
- `validation.json`, `frozen-sha256.json`, `method-sha256.json` and `raw-answer-receipt.json`: test gates, input/method identities and a receipt for complete private replies. Hashes identify data; they do not authenticate or anonymize it.

A is registry 0.4.6 literal BM25/prefix; P is registry 0.4.6 prepared/leading/distinct; C is candidate 0.4.7 coverage/leading/distinct. All use six final passages, 100 candidates and the same 2,000/12,000-byte source budgets. No fresh SQLite/embedding or large-PDF speed comparison is claimed. Same-machine caches and different outcomes preclude treating pipeline differences as quality-equivalent savings. Private raw inputs, complete replies, failed attempts, measured executables and exact locks are retained outside the published crate. Aggregate records cannot independently reconstruct those private sources.
