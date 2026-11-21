# AIRust 0.3.10 measurement evidence

These aggregate files contain counts, source identities, checksums, test outcomes and scoped cost measurements. Original document text and private source paths stay in the retained local lab.

- `sources.json` identifies the three new official PDF snapshots; `corpus.json` and `document-comparisons.json` describe the 33-document comparison.
- `oracle-summary.json` and `cleaner-properties.json` report independent cleaning checks. Native full exports match 0.3.9; generated st-form cases expose the corrected behavior.
- `page-coverage.json`, `source-excerpts.json` and `raw-chunks.json` report new-source coverage and contiguous chunks. Prior source-excerpt proof carries forward only through exact unchanged exports.
- `paired-cli.json` reports complete real PDF imports. `cleaning-cost.json` reports a separate supplied-text helper benchmark. Their timing and memory scopes differ.
- The three fuzz summaries and classification reports describe 12,288 deterministic owned-input mutations. `formats.json`, `deadline.json` and `worker-failure.json` cover separate format/failure tests.
- `retention.json` identifies the current 160,085-entry export and unchanged previous knowledge. `audit.json`, `backend-audit.json` and `backend.json` retain dependency and backend provenance outcomes.
- `validation.json` records local release gates. The GitHub release verification asset records final CI, registry consumer and artifact identity after publication.

See the [plain-language report and limitations](../../CHANGES-0.3.10.md) and [reproduction guide](../../validation-0.3.10/README.md). Counts are observations from these fixtures, not an exhaustive correctness or security guarantee.
