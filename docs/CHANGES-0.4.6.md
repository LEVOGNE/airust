# AIRust 0.4.6: inspectable lexical questions and retained context

AIRust adds two explicit retrieval options: prepare a natural-language lexical question, and retain more surrounding source text. Ordinary Engine BM25, default retrieval JSON, existing citation validation and optional MCP behavior remain unchanged. No model, external service, new dependency or DenkMaps integration is required.

## Why change evidence delivery

The previous release's compact paragraph mode saved tokens but lost some reference evidence. On its 174 positive authored questions, old prefix delivery retained the reference quotation in 137 cases; compact delivery retained it in 129. In that compact arm, the reference entry was actually among the delivered IDs in 138 cases. Shorter excerpts can therefore lose useful information even when the right entry was found. Most remaining baseline failures occur before excerpt construction: the reference entry is absent from the first six results.

All 210 earlier questions are now exposed development data. Development experiments tried simple query filtering, identifier splitting and diversity penalties. Raw BM25 has reference-entry presence 137/174 and reference-ID MRR@6 0.6472. An initial filtering/identifier prototype reached 142/174 and 0.6649 on those same exposed labels. Diversity penalties offered little improvement and were not added. These development results guided design; they are not independent test claims. Prototypes and unsuccessful attempts remain privately retained.

## Explicit lexical query preparation

`LexicalSource::with_query_analysis(LexicalQueryOptions::default())` prepares a question before the existing lexical scorer. It removes a fixed list of common English/German function and question words, protects negation and quoted/scoped names, keeps complete technical identifiers and adds ASCII CamelCase/acronym/digit parts. For example, `ProvenanceStore` keeps its normalized whole identifier while adding `provenance` and `store`. No project-specific aliases, translation, embeddings or guessed synonyms are used.

Call `LexicalQueryOptions::analyze` to inspect the effective query, removed terms, added terms and original-term fallback. Disable either operation or set explicit positive budgets. Default input is 16 KiB, normalized/effective text 64 KiB and unique terms 256. Limit failures return errors, not silently shortened queries. An all-filtered query retains its original normalized terms; punctuation has no searchable terms. Source scope intersection and cooperative cancellation still apply, and no previous query is cached in the source.

The response recipe records the selected flags and limits. Scores and matched terms belong to the actual prepared query. Do not reuse relevance thresholds calibrated on a different query preparation without measurement. Existing evidence assessment still evaluates the original caller question; preparation does not silently bypass its coverage policy. Original questions and original source text remain unchanged. See [every public query method and limitation](LEXICAL-QUERY.md).

## Retain surrounding evidence

`ExcerptOptions::with_leading_context()` starts from the existing relevant window and prefers a full grapheme-safe prefix only if it retains every distinct matched term and every explicit phrase-hint position in that window. Otherwise it keeps that window. When the whole entry fits, it stays whole. It does not shrink to an isolated paragraph. Paragraph preference and leading-context preference are alternatives; the last builder wins.

This is conservative lexical coverage, not a guarantee of complete claims or qualifications. A small byte budget can still omit important context. No disjoint excerpts are joined and no original positions are invented. Exact-excerpt deduplication remains separately optional, with bounded refill and preserved source ranks. Literal citation checks still apply to the final immutable delivered response; finding exact quote bytes does not establish entailment, truth, freshness or permissions. See [excerpt contract](EXCERPTS.md), [citation contract](CITATIONS.md) and [API cookbook](API-COOKBOOK.md).

## Executed validation

The complete local script passes **221 default tests, 849 all-feature checks including 38 documentation examples, 39 isolated feature entries and five Python tests**. Strict Clippy/documentation checks pass. Rust 1.89 compiles provider-neutral features; Rust 1.92 compiles all features. Public API comparison with tagged 0.4.5 reports **229 passes and 31 tool skips**. A separate Rust 1.89 application exercises the new source/excerpt API, old citation behavior and document-coordinate projection.

Five small Tree Borrows Miri cases complete: original-term fallback/punctuation, protected quoted/scoped names, complete-context retention, explicit phrase positions/builder precedence and overlapping citation occurrences. Extra floating-point rounding injection alone is disabled; memory/alias checks remain enabled. These finite interpreter cases exclude native PDF/ONNX and async MCP and are not a proof of soundness. Native Unicode/coverage oracles and public source-score comparisons exercise the wider paths.

The unchanged PDF backend passes 14 regressions and its 381-source-file provenance check. Dependency audit reports zero known vulnerabilities in the checked database, with existing informational maintenance notices for `chrono`, `paste` and `ttf-parser`; those optional/transitive migrations remain separate work. This is not an independent security audit. An initial Clippy condition-style failure was corrected, and the complete matrix then rerun; its first log is retained. Package extraction, exact-commit CI, registry consumption and checksum/VCS matching are publication gates.

## Honest document comparison

The [protocol, worker manifests and executable harness](validation-0.4.6/README.md) use the unchanged 72-file, 1,910-chunk private source snapshot. The earlier 210 authored questions are development data. Another 72 questions were frozen before runtime implementation or inspection of their rankings: 54 positive and 18 intentionally unsupported, 24 per project, with 12 English and 60 German questions. The 54 new positive questions draw from only four known files (9, 9, 18 and 18 questions per file). This narrow, correlated sample is assistant-authored, not independent human judgment, logged user traffic or unseen projects. Question-level bootstrap intervals describe these labels and do not correct for source-file clustering or establish population uncertainty.

A compares exact published 0.4.5 BM25/prefix delivery; C compares 0.4.6 with the explicit query preparation, leading context and exact distinct excerpts; S compares SQLite FTS5/BGE-M3 exact cosine with equal-weight RRF. All use the same scope, six hits, 100 candidates, 2,000 bytes per passage and 12,000 text bytes total. Rust receives case-preserved original questions; S uses the common lexical normalization for FTS5 and original questions for embeddings. C intentionally changes its effective query. This compares complete configurations, not an isolated causal effect for one component.

All 282 questions receive fresh local Qwen replies from each arm, for 846 replies with an identical prompt and quote-object schema. Fixed seed is 461106, temperature zero, thinking disabled, context 16,384 tokens and output cap 512. Six execution orders are balanced within each partition. Real cache counters, malformed output, quote errors, abstentions and length stops are retained without quality-based retries. S end-to-end costs include fresh query embedding. The isolated loopback Ollama server uses installed models, with cloud disabled; the original user daemon is untouched.

### Answer results on the 72 fresh questions

There are 54 positive reference questions and 18 intentionally unsupported questions. Automatic fact-pattern checks are descriptive proxies, not semantic correctness judgments. Requiring an authored reference ID can reject valid alternative support. The same literal quotation checker is applied to all three arms.

| Holdout observation | A: 0.4.5 BM25 | C: prepared/leading | S: SQLite hybrid |
| --- | ---: | ---: | ---: |
| Reference quotation present in delivered context, /54 | 45 | 44 | 45 |
| Reference-quotation MRR@6, missing reference = 0 | 0.7235 | 0.7469 | 0.7244 |
| Answer matches authored fact patterns, /54 | 41 | 39 | 42 |
| Fact patterns + verified quotations + consistent inline IDs, /54 | 37 | 36 | 40 |
| Same proxy plus an authored reference ID, /54 | 37 | 36 | 37 |
| Proper abstention on unsupported questions, /18 | 18 | 18 | 18 |
| Abstention on a corpus-answerable question, /54 | 10 | 12 | 6 |

The paired C−A difference in the combined automatic fact/quote-or-abstention proxy over all 72 cases is -1.39 percentage points; the 10,000-resample percentile interval is [-6.94, +4.17]. This interval describes the authored set, not population uncertainty. No independent user-accuracy advantage is established.

### Cost of the complete delivery configurations

The table uses the 72 holdout questions, including unanswered cases. Lower cost is not a quality-equivalent saving: the delivered evidence and answer outcomes differ. Shared prompt caching is enabled; real returned cache counters are retained. The six execution orders are balanced within each partition. Model generation is capped at 512 output tokens and 16,384 context tokens, with temperature zero and fixed seed. All 846 replies, including malformed output, are retained without quality-based retries.

| Holdout cost | A: 0.4.5 BM25 | C: prepared/leading | S: SQLite hybrid |
| --- | ---: | ---: | ---: |
| Mean prompt tokens | 3198.7 | 3198.0 | 3358.0 |
| Mean generated tokens | 62.9 | 62.5 | 68.3 |
| Mean cached prompt tokens | 332.4 | 430.4 | 0.0 |
| Median end-to-end seconds | 1.835 | 1.737 | 1.973 |
| p95 end-to-end seconds | 2.709 | 2.618 | 2.986 |
| Median delivered UTF-8 bytes | 10394 | 10430 | 10506 |

C changes mean prompt tokens by -0.02% on these holdout cases. This is measured with changed evidence, not proof of equal-answer-quality token savings.

End-to-end includes query embedding for S, retrieval, context construction, local generation and final citation validation. The temporary validation adapter checks passages against canonical scoped entries before checking quotations. Its admission/check subphase and complete IPC wall time are retained separately. Retrieval-only timing contains source work, excerpt construction, serialization/IPC and common context construction; worker subphase timings differ in scope and are not interchangeable algorithm benchmarks.

### All replies, failures and resource limits

| Observation over 282 replies per arm | A | C | S |
| --- | ---: | ---: | ---: |
| Strict JSON shape | 282 | 282 | 282 |
| Bounded answer parser admission | 282 | 282 | 282 |
| Nonempty exact quotation sets verified | 156 | 156 | 159 |
| Generation length stops | 0 | 0 | 0 |
| Quotes exceeding the prompt request of 180 characters | 1 | 2 | 2 |

Parser admission is not citation approval, and a proper abstention has no nonempty quote-proof claim. Citation statuses, parser failures, tokens, timings and every query-level outcome remain in the [sanitized records](measurements/0.4.6/README.md).


| All-query warm context/validation cost | A | C | S |
| --- | ---: | ---: | ---: |
| Median retrieval/context construction ms | 1.032 | 1.416 | 2.268 |
| Median complete quotation adapter/IPC ms | 0.181 | 0.181 | 0.180 |
| Median parser/registry-check subphase ms | 0.0106 | 0.0105 | 0.0106 |

Warm S retrieval uses prepared query vectors and excludes inference. End-to-end measurements include fresh S embeddings. These scopes and different cache counts prevent calling the end-to-end difference a BM25 acceleration.

Across all arms, 561 replies are non-abstaining; 90 fail at least one literal citation check (A: 28/184, C: 28/184, S: 34/193). This measures technical quotation admission, not answer truth.

C skips 0 byte-identical excerpts across 282 queries and changes the delivered passage set for 238 queries. The comparison retains 4,230 warm timed retrieval samples (five per question/arm). Whole-worker RSS samples are A: 68.4 MiB, C: 69.0 MiB, S: 118.9 MiB. The isolated Ollama process family peaks at 32.71 GiB sampled RSS. This is neither isolated index heap nor a complete accounting of Apple GPU/unified memory; loaded model residency is retained separately, and the two figures must not be added as independent allocations. No index-RAM improvement is claimed.

The local model is Qwen3.6 35B Q4_K_M, with BGE-M3 F16 for S, on an AC-powered Apple M5 Max with 64 GiB. Ollama is 0.40.16, Rust is 1.98.0 for measured workers, Python 3.14.7, NumPy 2.5.5 and SQLite 3.53.4. Small document/label samples, one machine, shared cache, warm models and one generation per question/arm limit generalization. Full private source data and replies are retained; sanitized counters alone cannot independently reconstruct this private corpus.



### Separate evidence-configuration checks

After the timed replies, a separate public consumer evaluates all 12 combinations of query preparation, excerpt mode and exact duplicate filtering on/off. No rule or parameter is tuned from those results. Both main configurations reproduce their measured IDs and original spans exactly. These are source/excerpt diagnostics, with no new LLM replies or quiet timing claim.

| Configuration, duplicate filter on | Development reference ID / quote, /174 | Holdout reference ID / quote, /54 |
| --- | ---: | ---: |
| literal, prefix | 137 / 137 | 45 / 45 |
| literal, leading | 137 / 137 | 45 / 45 |
| literal, compact | 138 / 129 | 45 / 44 |
| prepared, prefix | 141 / 141 | 44 / 44 |
| prepared, leading | 141 / 141 | 44 / 44 |
| prepared, compact | 141 / 134 | 44 / 44 |

The complete on/off factorial results are in the sanitized measurements. Leading and prefix delivery have the same reference counts on this corpus; the new fallback behavior is additionally checked with authored UTF-8/phrase-location oracles. Relative to compact paragraphs, wider context avoids known reference omissions, with more source text. It has not demonstrated an advantage over ordinary prefix delivery here. Exact duplicate filtering skips no excerpts in the main prepared/leading arm; it is not the source of its rank differences. Query preparation improves exposed reference presence but loses one fresh reference case. Keep the options workload-specific and leave literal BM25 as the starting configuration.

### Anonymized assistant semantic review

The preselected sample contains 24 questions, 18 answerable and six unsupported, with all three replies each. Half the questions are development and half holdout. Variant labels were masked; all 72 judgments were fixed and hashed before opening their mapping. The reviewer is the same assistant, not an independent human.

| Judgment | A | C | S |
| --- | ---: | ---: | ---: |
| Correct, complete and supported | 14 | 15 | 14 |
| Correct unsupported-question abstention | 6 | 6 | 6 |
| Abstention on a corpus-answerable question | 3 | 3 | 2 |
| Incomplete answer | 1 | 0 | 1 |
| Answer about a different component | 0 | 0 | 1 |

C has one more complete supported answer in this small sample; that is not a general accuracy gain. Some otherwise correct answers alter Markdown or quote punctuation, so semantic support and literal validity are separate. An exact valid quotation about vector deletion answers a question about positional-index state incorrectly; weak word-pattern proxies can miss that subject substitution. Other answers omit a requested UTF-8 ordering detail. An abstention on a positive question can be appropriate when retrieval omitted the fact.

Independent human questions/judgments, broader source/domain coverage and quality-equivalent cost comparisons remain necessary. The evidence here supports a conservative optional API, not automatically enabling query preparation, asserting semantic understanding or claiming a universally superior retrieval engine.

## Preserved data and artifact discipline

The canonical knowledge remains 167,942 entries from 35 PDFs and 49,086 physical pages. Its SHA-256 is `13168d87872a137355fd8499c90a6084f52243a2ec7456f29b5fcb2a1c7423c3`. Source PDFs, snapshots, previous releases, failed prototypes, frozen questions and full local answers remain private. This release compares Markdown project documents; it does not claim a new large-PDF performance result.

Builds use one named temporary Cargo directory with dev/test debug and incremental compilation disabled. Miri uses a separate temporary directory. Release archives, small measured workers, exact locks, logs and checksums are preserved before removing those two regenerable trees after registry/CI verification. Knowledge and measurement directories are never targets of broad deletion. Private source passages, questions, model replies and credentials are excluded from Git and crates.io. Public counters and hashes cannot independently reconstruct a private corpus.
