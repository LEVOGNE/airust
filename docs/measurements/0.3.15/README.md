# AIRust 0.3.15 evidence

Public aggregates omit private text and paths. Raw inputs, failed attempts, code, binaries and results remain in the local PDF lab.

- `pdf-recheck.json` records 36 complete import runs / 35 distinct sources, matching frozen 0.3.14 outputs. The repeated byte-identical PGF download is identified explicitly.
- `references.json` records fresh MuPDF page coverage and selected pypdf literal search probes for three manuals, including the PGF control. `sources.json` pins their source URLs, hashes and snapshot sizes.
- `memory-cost.json` / `memory-samples.json` record the isolated metadata helper comparison with real Object arrays and a fake loader. No native PDF parsing is timed there.
- `native-cost.json` / `native-samples.json` record 36 complete supervised converter processes. Parent RSS excludes the worker.
- `preview-cost.json` / `preview-samples.json` record 18 API preview processes. Those processes include the native parser.
- `structural-probes.json`, `structural-cases.json`, `reference-limit.json` and `mutations.json` record bounded authored/malformed-input checks. Accepted arbitrary mutants have no complete-text oracle.
- `retention.json` identifies the new 167,942-entry knowledge export and all eight retained checkpoints. All previous 160,085 objects and original PDFs are unchanged.
- `validation.json`, `backend.json` and dated audit aggregates record validation scope and unchanged backend identity. Final source/package/registry/CI/tag/release-asset agreement belongs to the release verification asset.

The initial test-generator lint failure is retained and corrected. Earlier core matching/cache/soak measurements are inherited through unchanged source, while current PDF/reference/memory runs are identified separately. This is the final planned 0.3.x stability release. See [executed results](../../CHANGES-0.3.15.md), [reproduction](../../validation-0.3.15/README.md) and [API/limits](../../PDF-PAGE-TREE-MEMORY.md).
