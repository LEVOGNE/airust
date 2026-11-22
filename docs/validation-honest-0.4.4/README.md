# Honest local comparison of published AIRust 0.4.4

This directory contains a benchmark harness, not a library feature. The library source, crate version and MCP behavior are unchanged. The worker resolves the exact published registry crate, not a path dependency or the unpublished 0.4.5 branch.

The comparison uses one local LLM with three retrieval implementations and an additional embedding ablation:

| Arm | Retrieval |
| --- | --- |
| A | Linear keyword search over frozen file chunks. Count distinct matching query terms, then term occurrences; ascending IDs break ties. No inverted index or IDF. |
| B | SQLite FTS5 BM25 plus BGE-M3 exact cosine, fused with equal-weight reciprocal ranks. |
| C | AIRust 0.4.4 BM25 plus the same BGE-M3 vectors and fusion settings. |
| D | AIRust 0.4.4 BM25 alone. This extra control separates lexical retrieval from the effect of embeddings. |

These are measured implementations. Python, SQLite, NumPy and Rust have different runtime costs. This does not isolate language-independent algorithm speed. A is a warm in-memory scan of file-derived chunks, not a disk scan on every question. B uses a normal FTS5 table and application-owned dense vectors; it is not a claim about every possible SQLite vector extension or optimized deployment.

## Inputs and labels

Private frozen inputs and full model replies remain under `local-lab/honest-comparison-044/`, which is excluded from Git. They include 72 Markdown files from AIRust, Flasm and Vecasm, 1,910 chunks and 100 document-derived reference questions. There are 88 answerable questions and 12 deliberately unsupported questions.

The questions were authored by the assistant. They are **not 100 logged user questions, independent human judgments, a broad semantic benchmark or production traffic**. Most ask for bounded documented facts. Project documents can contain historical plans and stale descriptions. Reference answers describe those snapshots, not independently verified behavior of the other applications. Exact quote judgments are not exhaustive: another passage may support the same answer.

`protocol.json` and `frozen-sha256.json` in the private directory identify settings and inputs frozen before retrieval results. Each chunk preserves an exact original UTF-8 byte range. Every arm receives the same original chunks, project scope, verbatim question, six-hit cap and 12,000-byte source-text budget. All lexical inputs use the same NFKC/lowercase alphanumeric terms; no stopword removal, stemming or per-arm query rewrite is applied. SQLite additionally tokenizes that normalized text with `unicode61 remove_diacritics 0`. AIRust's tokenizer is checked against every prepared input.

B and C use 100 candidates per component, RRF k=60, weights 0.5/0.5 and no tuned similarity threshold. Project filters apply before top-K. Document embeddings are shared. Each measured hybrid answer also pays for a fresh local single-query embedding; its vectors and retrieved IDs are checked against the frozen run. Embedding setup and inference costs are reported separately.

## Local model execution

The run used the existing `qwen3.6:35b` Q4_K_M model and `bge-m3:latest` F16 model. Recorded full digests identify the actual local artifacts; their names alone do not certify upstream provenance. Generation uses `think=false`, temperature 0, seed 441104, context 16,384 and at most 256 generated tokens. The same schema, prompt and context formatter apply to every arm. Source text is data, not executable instructions. No answer key is provided to the LLM.

An isolated Ollama daemon binds to `127.0.0.1:11464`, uses existing local model files and disables cloud inference and pruning. The Python client disables proxies and redirects. No model is downloaded or project content sent to a remote service by this harness. The user's original Ollama server is left alone.

Answer order interleaves projects. All 24 permutations of A/B/C/D occur four times, followed by ABCD, DCBA, BADC and CDAB. Each arm occupies each position 25 times, and each pair precedes the other 50 times. Prompt caching stays enabled and its actual counters are reported; counterbalancing does not make caching disappear.

The API's actual prompt/generated token counters are used. UTF-8 bytes and approximate character counts are not passed off as model tokens. The prompt includes system instructions and JSON wrappers. Equal byte budgets need not produce equal token counts. See [Ollama chat](https://docs.ollama.com/api/chat) and [embedding](https://docs.ollama.com/api/embed) contracts. SQLite BM25 has its own score/IDF behavior; raw SQLite and AIRust scores are not assumed equal. See [SQLite FTS5 BM25](https://sqlite.org/fts5.html#the_bm25_function).

## Run on retained inputs

Use a private working directory and an existing local Ollama installation. Running these commands starts real inference and can use substantial RAM. They assume the frozen private dataset and installed models already exist; they do not scan arbitrary files or download models.

```bash
python3 -m venv local-lab/honest-comparison-044/venv
local-lab/honest-comparison-044/venv/bin/python -m pip install numpy==2.5.5
```

Copy `consumer.toml` to the private consumer directory as `Cargo.toml` and copy `consumer.rs` next to it. Retain its generated `Cargo.lock`; the recorded lock pins the complete measured dependency graph. The AIRust package checksum must be `96e5fca10668f2deb31cac485d699bc149a7242061d11f74fb475fded82f137f`.

The exact measured lockfile is included as [consumer.lock](consumer.lock). Copy it to the private consumer directory as `Cargo.lock`. Start the isolated daemon in a separate terminal before inference, pointing it at your existing model directory:

```bash
OLLAMA_HOST=127.0.0.1:11464 OLLAMA_NO_CLOUD=1 OLLAMA_NOPRUNE=1 \
OLLAMA_NUM_PARALLEL=1 OLLAMA_MAX_LOADED_MODELS=2 \
OLLAMA_CONTEXT_LENGTH=16384 OLLAMA_MODELS=/absolute/path/to/existing/models \
ollama serve
```

```bash
cargo build --release --locked \
  --manifest-path local-lab/honest-comparison-044/consumer/Cargo.toml \
  --target-dir /private/tmp/airust-honest-044-target

local-lab/honest-comparison-044/venv/bin/python docs/validation-honest-0.4.4/run.py embed
local-lab/honest-comparison-044/venv/bin/python docs/validation-honest-0.4.4/run.py retrieve
local-lab/honest-comparison-044/venv/bin/python docs/validation-honest-0.4.4/run.py answer
local-lab/honest-comparison-044/venv/bin/python docs/validation-honest-0.4.4/analyze.py
```

`AIRUST_BENCH_DIR` selects another prepared directory. `embed` checks input hashes and reuses matching batch checkpoints. `answer` skips complete checkpoints and writes new replies atomically. To run a new experiment, use a new directory and manifest; do not overwrite judgments after seeing rankings. `retrieve` rebuilds only its own `baseline.sqlite` file.

`monitor.py` samples only the isolated daemon and its child processes. Create `monitor.stop` in the private directory to finish sampling. Model RSS, retriever RSS, GPU allocations and total system memory are different measurements. Shared mappings prevent simple addition of process RSS into a reliable total. One-second samples can miss short peaks. There is no claim of a measured hard memory bound.

## Interpretation

Reference Hit@K asks whether a selected exact supporting quote occurs in the first K chunks. MRR@6 uses the first such match. These are reference-evidence metrics, not exhaustive relevance recall.

The automatic answer check requires predefined fact patterns and a supplied reference citation, or explicit abstention on unsupported questions. It can reject correct paraphrases/alternative citations and accept misleading extra statements. It is labeled an **answer proxy**, not an independently verified correct-answer rate. A separate exploratory assistant audit inspects anonymized responses; it is not an independent human security or quality audit.

Warm retrieval has five timed repetitions after a warm-up for each question. End-to-end samples include live embedding where applicable, retrieval, formatting and LLM completion. Their percentiles describe this finite single-machine run. Paired bootstrap intervals resample these authored questions; they do not establish production significance or predict new query distributions. No features, weights or thresholds are tuned on this test set.
