# Local comparison evidence

Read the [report](../../HONEST-COMPARISON-0.4.4.md) before interpreting these numbers. This is an executed single-machine experiment on authored project-document questions. It is not a set of 100 logged user questions or an independent human evaluation.

| File | Contents |
| --- | --- |
| `protocol.json`, `dataset-manifest.json` | Settings frozen before retrieval, corpus sizes and private-input hashes. |
| `environment.json` | Hardware, software, exact installed model digests and registry crate checksum. |
| `summary.json`, `scored-rows.json` | Aggregates and all 400 sanitized automatic answer checks, token counts and timings. The checks are proxies, not adjudicated correctness. |
| `warm-summary.json`, `warm-series.json` | Four final process series, 500 timed searches per arm per series, retained samples, startup measurements and RSS observations. |
| `manual-audit-summary.json`, `manual-audit-rows.json` | Exploratory assistant review of 80 anonymized answers; judgment categories without private source text. |
| `citation-format-diagnostics.json`, `identical-prompt-diagnostics.json` | Actual citation/null-format issues and repeated identical-prompt pairs. |
| `semantic-equivalence-all100.json` | Independent NumPy/AIRust top-100 dense-rank agreement for every query. |
| `embedding-setup.json`, `model-memory.json` | Common embedding setup costs and sampled model-process RSS, including limitations. |
| `dataset-validation.json`, `retention.json` | Exact source-byte ranges, scope checks and preservation of the earlier PDF knowledge file. |
| `incidents.json` | Disk-full interruption, safe recovery and the single unsaved inference repeated on resume. |
| `artifact-sha256.json` | Hashes of the published measurement files. |

Query IDs join rows across files; project names indicate scope. Private questions, source excerpts, answer text and prompts are retained in `local-lab/honest-comparison-044/` on the originating Mac, excluded from Git. These sanitized files allow aggregate calculations to be checked. Rejudging answers requires the retained private inputs; the public hashes alone do not supply their contents.

The first final idle series is also retained in `summary.json` under `first_idle_retrieval_500_per_arm`. Use `warm-summary.json` for the combined four-series figures in the report. Original pre-cleanup timings remain in the private directory and are disclosed through their initial medians; they are not silently substituted for the final four-series samples.
