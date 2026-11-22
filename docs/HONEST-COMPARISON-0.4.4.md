# AIRust 0.4.4: an honest local retrieval comparison

Measured on 22 November 2026. **This run does not demonstrate that AIRust hybrid retrieval produces better answers than SQLite FTS5 plus the same embeddings.** AIRust BM25 alone found more reference passages than either hybrid configuration on this particular fact-lookup set. More features would not, by themselves, fix the observed failures.

The library source and version are unchanged. The unpublished 0.4.5 work is preserved separately. No DenkMaps integration or new MCP integration was made, and no crate was released for this report.

## What was actually tested

One installed local Qwen model answered the same 100 questions with four retrieval configurations. The first three implement the requested comparison. The fourth separates the effect of embeddings from the existing AIRust lexical core.

| Arm | Configuration |
| --- | --- |
| A | Simple linear keyword search over file-derived chunks. |
| B | SQLite FTS5 BM25 + BGE-M3 exact cosine + equal-weight RRF. |
| C | Published AIRust 0.4.4 BM25 + the same BGE-M3 vectors + equal-weight RRF. |
| D | Published AIRust 0.4.4 BM25 alone, as an additional control. |

The corpus contains **72 Markdown files, 2,755,468 source bytes and 1,910 chunks** from AIRust, Flasm and Vecasm. There are 35 AIRust questions, 35 Flasm questions and 30 Vecasm questions. Every arm receives the same project scope, original chunks, six-hit limit, 12,000-byte source-text limit and answer instructions. Hybrids use 100 candidates per component, k=60 and weights 0.5/0.5. No weights, thresholds or query rewrites were tuned after seeing results.

**The requested 100 genuine historical user questions were not available.** These 100 questions were authored from project documents before inspecting search results. Eighty-eight ask for documented facts; twelve ask for information absent from the selected documents. This is a real executed reference experiment, not production traffic or an independently labeled benchmark. Most questions are narrow fact lookups, with an emphasis on API names and architecture responsibilities. The negative questions concern missing measurements, guarantees or certifications; they are relatively recognizable and do not establish performance on subtle near-answer cases. Claims about broad semantic understanding would require a different evaluation.

All **400 LLM completions** finished. All returned schema-shaped JSON; none stopped at the output-token limit. All 1,910 original byte ranges and 400 scoped retrieval result sets were checked, with no cross-project result IDs. For every query, SQLite/NumPy and AIRust returned identical top-100 dense ranks; maximum cosine-score difference was below 6.1 × 10⁻⁹. Differences between these hybrid rankings therefore come from lexical retrieval and fusion, not different embedding inputs. All 39 pairs with byte-identical complete request payloads also returned identical parsed answers.

## Retrieval results

“Reference found” means that an exact, preselected supporting quote occurs in one of the returned chunks. Other passages can support the same fact. This is **reference Hit@K, not exhaustive relevance recall**.

| Configuration | Reference in first hit / 88 | Reference in first six / 88 | Hit@6 |
| --- | ---: | ---: | ---: |
| File keyword search | 28 | 58 | 65.9% |
| SQLite hybrid | 35 | 66 | 75.0% |
| AIRust hybrid | 37 | 64 | 72.7% |
| AIRust BM25 | 51 | 69 | 78.4% |

AIRust hybrid retrieved six more reference answers than simple keyword search, two fewer than SQLite hybrid, and five fewer than its own BM25 control. Its first-hit result improved over SQLite hybrid while its six-hit result declined. There is no single universal winner across these cutoffs.

## Answer results and their limits

The automatic check looks for predefined facts and at least one supplied reference citation. It also requires explicit JSON-null abstention on unsupported questions. It does not verify every statement in an answer. Alternative valid citations can fail this check, while a misleading answer can contain all expected words. **These counts are answer proxies, not independently established correct-answer rates.**

| Configuration | Positive answers matching expected facts / 88 | Full reference-answer proxy / 100 | Correct JSON-null abstentions / 12 |
| --- | ---: | ---: | ---: |
| File keyword search | 54 | 63 | 12 |
| SQLite hybrid | 65 | 69 | 12 |
| AIRust hybrid | 60 | 67 | 12 |
| AIRust BM25 | 64 | 72 | 12 |

The full proxy includes the twelve negative cases. AIRust hybrid minus SQLite hybrid is **−2 percentage points**, with a paired question-bootstrap interval of **−7 to +3 points**. That interval describes resampling this authored set; it is not a production confidence guarantee. It does not establish an AIRust quality advantage.

An exploratory audit inspected **80 anonymized answers to 20 questions**, including complete cited passages. Labels were fixed before mapping answers back to configurations. The sample contains seventeen answerable and three unsupported questions per arm. The reviewer was the same assistant, not an independent human assessor; the sample was selected after the experiment protocol and aggregate retrieval results were available.

| Configuration | Correct, sufficiently supported positive answers / 17 | Correct fact with insufficient citation support | Unnecessary abstentions / 17 | Correct abstentions / 3 |
| --- | ---: | ---: | ---: | ---: |
| File keyword search | 12 | 1 | 4 | 3 |
| SQLite hybrid | 13 | 0 | 4 | 3 |
| AIRust hybrid | 12 | 0 | 5 | 3 |
| AIRust BM25 | 13 | 1 | 3 | 3 |

Two audited answers also had citation-format issues despite a supporting structured citation. Across all 400 replies, every structured citation ID belonged to the supplied context, but one answer contained a malformed inline source ID. Two positive answers returned the **string** `"null"` instead of JSON `null`. A JSON schema alone did not ensure useful answers or consistent citations.

## Time, tokens and RAM

Machine: **Apple M5 Max, 64 GiB RAM, macOS 27.0.1, AC power**. Runtime: Ollama 0.40.16, Python 3.14.7, SQLite 3.53.4, NumPy 2.5.5, Rust 1.98.0. The Rust consumer resolves registry `airust = "=0.4.4"`, without default features and with `hybrid`. The registry checksum matches the published package.

LLM: installed `qwen3.6:35b`, Q4_K_M, digest `a7eb95c53bcf96b4bdd008d0fab4a5dac88047d9c1a7a9ab88ed453423fbd87c`. Embeddings: installed `bge-m3:latest`, F16, digest `7907646426070047a77226ac3e684fbbe8410524f7b4a74d02837e43f2146bab`. Names identify local artifacts, not independent certification of upstream provenance. Generation used `think=false`, temperature 0, seed 441104, 16,384 context tokens and at most 256 generated tokens.

The final retrieval measurement uses four process series, each with five timed repetitions per question after warm-up: **2,000 timed calls per arm**. It excludes model inference. The LLM series includes a fresh single-query embedding for each hybrid answer, actual retrieval and complete generation.

| Configuration | Warm retrieval median | Warm retrieval p95 | Complete answer median | Complete answer p95 | Mean LLM input tokens | Mean generated tokens |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| File keyword search | 0.90 ms | 2.04 ms | 1.77 s | 2.28 s | 3,242 | 47.3 |
| SQLite hybrid | 2.10 ms | 4.59 ms | 1.57 s | 2.16 s | 3,224 | 54.8 |
| AIRust hybrid | 2.31 ms | 4.11 ms | 1.61 s | 2.29 s | 3,229 | 52.7 |
| AIRust BM25 | 0.84 ms | 1.55 ms | 1.77 s | 2.35 s | 3,226 | 53.4 |

These times are descriptive. Initial and later series varied substantially: SQLite hybrid series medians ranged from 1.88 to 3.41 ms, AIRust hybrid from 2.13 to 3.15 ms. Their relative ordering changed in one series. Python/NumPy and Rust also have different runtime overhead. These measurements do not establish a universal engine speed advantage or production tail latency.

Faster completion can mean an unhelpful short abstention. On the **same 53 answerable questions where both hybrid arms passed the reference proxy**, complete-answer medians were **1.65 s for SQLite and 1.66 s for AIRust**. Mean complete-answer difference over all questions, AIRust minus SQLite, was −0.085 s; its paired interval crossed zero (−0.257 to +0.085 s).

Prompt caching remained enabled. All four positions and pair orderings were balanced, but actual cache reuse differed: SQLite hybrid reported 68,328 cached prompt tokens, AIRust hybrid 86,645, and A/D zero. Those differences limit causal timing claims. Actual total prompt tokens were 324,249 / 322,435 / 322,911 / 322,559 for A/B/C/D. **There is no material demonstrated token saving for AIRust here.** The context budget was intentionally held constant. The model's real counters were used, not characters divided by four. Mean AIRust-minus-SQLite input difference was +4.76 tokens, interval −8.58 to +16.95.

Preparing 1,910 document embeddings took **47.92 s** in 120 local batches, including initial embedding-model load; reported input tokens totaled 1,019,334 in the embedding tokenizer. Each live hybrid query embedding took about **17–18 ms median**. Embedding tokens and LLM tokens use different tokenizers and are reported separately. Retrieval-only construction took approximately 34–37 ms for A, 50–80 ms for B and 232–291 ms for C; process startup and common file preparation are additional costs. Both hybrids pay the same embedding preparation cost.

Dedicated warm-worker RSS observations ranged from **86.6–100.7 MiB for A**, **94.4–119.1 MiB for B**, and **66.9–75.5 MiB for C**. These include each implementation's runtime and loaded data, not isolated index heaps. Unnecessary NumPy loading in A and retained source-JSON copies in A/B were removed before these final measurements; all 400 rankings and scores stayed unchanged. D shared C's semantic worker, so a separate minimal-BM25 RAM figure was **not measured**.

The common Qwen process reached approximately **31.7 GiB observed RSS**, and the additional BGE process approximately **1.28 GiB**. Model RSS dominates these small retrievers. Models used shared mappings/unified memory; RSS values must not simply be summed into a total physical-memory claim. One-second sampling missed a disk-full interruption and can miss short peaks. Reported model GPU allocations are a different quantity from OS RSS. No million-entry RAM comparison was performed in this experiment.

## Failures that matter

1. **Equal-weight hybrid retrieval can push useful exact matches out of the context.** Both hybrids lost reference hits compared with AIRust BM25 on this set. An embedding is not automatically a better ranking signal for names, limits and class lookups.
2. **Relevant evidence does not guarantee a correct or useful LLM answer.** Some delivered reference passages still resulted in missing facts or abstention. The retrieval engine cannot certify an arbitrary model's answer.
3. **A reference-only grader misses legitimate alternatives.** A seek-related answer cited an alternative ADR that explicitly supported the fact, while the reference quote came from the concept document. Conversely, other answers cited merely related revision passages that did not establish the requested property. Citation existence and citation support are different checks.
4. **Some basic lookups still fail across every arm.** The audit's I18n filename question received no usable answer in any configuration. This is a concrete regression case for later retrieval and context testing.
5. **Documentation mixes historical introductions with current capabilities.** Some older feature guides describe the initial release as having no MCP/hybrid/ANN support, while 0.4.4 provides optional versions elsewhere. Version-scoped wording and a current capability table would reduce confusing retrieval context. No other project's implementation was audited to resolve its document history.
6. **Operational robustness includes the benchmark harness.** The disk filled after 101 saved answers. Only regenerable Cargo incremental artifacts were removed; all documents, knowledge and results were kept. One completed but unsaved response was repeated, existing replies were not selected or rerun by score, and checkpoints were changed to atomic writes. The interruption can affect cache/timing and is disclosed in the raw incident record. This was not an AIRust search failure.

## Improvements supported by this evidence

1. **Keep BM25 as the initial baseline for technical fact lookup.** Compare any proposed query routing or fusion policy against it. Do not replace it with equal-weight hybrid by default based on feature availability.
2. **Build a genuinely independent evaluation set.** Collect consented real project questions, including paraphrases, multi-source answers, ambiguous versions, incorrect premises and difficult no-answer cases. Have a human check reference answers and all supporting alternatives. Split development and untouched test questions before tuning.
3. **Evaluate existing controls before adding new modules.** AIRust already has optional German analysis, phrases, reranking, relevant excerpts and evidence policies. Test these individually on a separate development set. Do not tune them on these 100 questions and then present the result as independent validation. A matched SQLite-only lexical control is also still needed before claiming that AIRust's lexical engine outperforms SQLite's.
4. **Improve evidence selection and version handling.** Preserve headings, document identities and revision metadata; examine redundant/overlapping chunks and mixed historical/current guidance. Measure supported-answer quality after the exact final context budget, not merely candidate scores.
5. **Validate the answer boundary.** Check actual supporting passages, inline/structured citation consistency and null semantics. Reject fabricated guarantees and malformed source IDs. Treat relevance, sufficiency and factual truth as separate properties.
6. **Profile meaningful bottlenecks.** Here generation takes seconds and retrieval milliseconds. Small retrieval changes alone are unlikely to deliver a large complete-answer speedup. Measure cold start, update costs, large-corpus memory and real concurrency separately before making scale claims.

The justified next step is a quality-focused evaluation cycle with the existing APIs. This report gives no evidence for replacing an already working SQLite hybrid stack solely to obtain better answers, and no evidence that a larger AIRust feature list would improve this workload.

## Evidence and reproduction

The [harness and detailed method](validation-honest-0.4.4/README.md), [sanitized measurements](measurements/honest-0.4.4/README.md), [summary](measurements/honest-0.4.4/summary.json), [four warm series](measurements/honest-0.4.4/warm-summary.json), [manual audit](measurements/honest-0.4.4/manual-audit-summary.json) and [frozen dataset manifest](measurements/honest-0.4.4/dataset-manifest.json) accompany this report. Full project text, questions, reference quotes, model replies and individual source citations remain available locally under `local-lab/honest-comparison-044/`; they are excluded from Git. Original PDF knowledge and previous release evidence remain intact.

The [Ollama chat contract](https://docs.ollama.com/api/chat) defines actual generation/token counters, and its [embedding contract](https://docs.ollama.com/api/embed) documents explicit overlength rejection. [SQLite's FTS5 documentation](https://sqlite.org/fts5.html#the_bm25_function) explains its ranking behavior. These primary sources describe interfaces, not the quality of this experiment.
