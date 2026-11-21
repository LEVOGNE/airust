# AIRust 0.4.1 measurements

These public artifacts contain counts, hashes and timing samples without private document text or query strings. See [the release report](../../CHANGES-0.4.1.md) and [reproduction](../../validation-0.4.1/README.md).

- `validation.json`: executed local suite counts and build/API scope.
- `mcp.json` / `mcp-cases.json`: final canonical and 35 snapshot protocol/oracle checks; 424 searches and 624 exact passage readbacks.
- `mcp-cost.json` / `mcp-cost-samples.json`: 15 quiet fresh-process samples, real stdio timings and server child RSS. No native parsing occurs.
- `matching.json` / `matching-samples.json`: paired ordinary API calls against 0.4.0 with positive real-entry probes and controlled-result equality.
- `security-regressions.json`: reproduced escaped-error budget failure and corrected result, plus scoped hardening cases.
- `retention.json`: unchanged canonical exports, original hashes and existing snapshots.
- `environment.json`, `audit.json`, `backend-audit.json`, `backend.json`: compiler/profile, advisories, unchanged parser and fresh regressions/provenance.

Warm caches and uncontrolled desktop activity limit generalization. Input-byte limits are not process-RAM quotas. Reference queries are literal extraction/retrieval probes, not relevance judgments. Source PDFs, all old knowledge, complete expected responses, failed setup attempts and raw logs remain under the ignored private lab. Exact-commit CI, final package and actual registry checks are recorded separately with immutable release artifacts.
