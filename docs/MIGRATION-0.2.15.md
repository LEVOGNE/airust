# Migrating to AIRust 0.2.15

This patch adds optional assessment under `retrieval`, without new dependencies. Existing defaults, ranking, old public struct literals and response JSON remain unchanged when disabled. Enable it explicitly with `.with_evidence_policy(EvidencePolicy::default())`; consume `response.assessment()` and handle every non-exhaustive verdict with a wildcard. Assessment adds bytes to the response and can reduce returned hits under the same output budget. Limits may return typed budget errors before provider work.

Match score scale and exact recipe only after evaluating thresholds for your own complete pipeline/corpus. Coverage is lexical evidence, not an answer or proof. No database, DenkMaps or MCP change is made. See [contract](ASSESSMENT.md).
