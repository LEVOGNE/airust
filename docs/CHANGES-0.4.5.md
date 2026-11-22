# AIRust 0.4.5: explicit evidence selection and checked citations

BM25 remains the baseline. AIRust can optionally deliver distinct compact passages and validate a model's references against exactly the text delivered for that request. The new API detects invented IDs, missing or changed quotations and invalid UTF-8 positions. It does not establish whether an answer follows logically from a quotation. No DenkMaps integration, mandatory model, new dependency or default ranking change is included.

## What applications gain

Enable the existing `retrieval` feature. Use `RetrievalOptions::with_distinct_evidence()` to remove byte-identical delivered excerpts and refill from the same bounded ranked candidates. Use `ExcerptOptions::with_paragraph_boundaries()` with relevant excerpts to prefer a complete matching paragraph. This preserves original entry positions, source ranks and BM25 scores. The complete JSON output budget can still remove tail hits; final hits are the authoritative delivered catalogue.

Paragraph preference preserves all distinct query terms and every phrase-hint interval present in the original relevant window. If no fitting complete block retains them, AIRust keeps that window. It does not prove that qualifications without matching query terms remain present. Separate identical sources may be useful corroboration, so deduplication is optional. Existing ordinary retrieval and MCP behavior remains unchanged.

Parse bounded model output with `CitedAnswer::from_json` and inspect it with `CitationRegistry::new(&response).check`. The registry borrows the immutable delivered response. Exact quotes resolve to exclusive-end original-entry UTF-8 byte positions. Repeated and overlapping occurrences require an explicit valid span; the checker never silently chooses the first occurrence. Strict JSON rejects unknown/duplicate fields, wrong types, blank answers, the string `"null"`, citations on abstention and answers without references. A real abstention is `{"answer":null,"citations":[]}`.

ID-only references remain visibly `ReferenceOnly`. `quotes_verified()` requires a nonempty collection of exact quotations; an empty abstention is not a proof. Recheck document revisions and permissions in the application. To project positions, validate a `documents::Passage` against the retained snapshot and use `Passage::subspan`. No stale revision is guessed. See the [citation API and all statuses](CITATIONS.md), [selection contract](EVIDENCE-SELECTION.md), [API cookbook](API-COOKBOOK.md) and runnable `checked_evidence` example.

## Reproduced weakness and the correction

The first exploratory implementation could replace a relevant window with a shorter paragraph containing fewer query terms. On the 88 positive old questions, reference-quote presence fell from 69 to 62; on 50 new positive questions, it fell from 40 to 33. No LLM answers were generated in that exploratory round. These are failures, retained with the original source and outputs. The entire 162-question set became exposed development data.

A regression now requires preservation of every distinct matched term in the original final window and all of its retained phrase hints. An independent exhaustive paragraph oracle checks this rule. The corrected code was frozen before evaluating another 48 questions (36 positive, 12 unsupported). Those questions share the same document corpus and assistant author; they are not independent production observations. No further selection tuning uses their results. Authored unsupported questions are not a comprehensive adversarial hallucination test.

## Executed validation

The full local script passes **221 default tests, 835 all-feature checks including 37 documentation examples, 39 isolated feature entries and five Python tests**. Strict Clippy and documentation builds pass. Provider-neutral features compile with Rust 1.89; all features compile with Rust 1.92. Public API comparison against tagged 0.4.4 reports **229 passes and 31 tool skips**. Dependency audit reports zero known vulnerabilities in the checked database, with informational unmaintained notices for `chrono`, `paste` and `ttf-parser`. Those existing optional/transitive dependencies are not replaced in this release; migrations need separate compatibility checks. This is not a security audit.

The new integration suites cover strict admission budgets, Unicode character boundaries, overlapping occurrences, explicit ranges, duplicate references, scope, immutable older responses, output truncation, retained document projection, blank-line/CRLF paragraphs, exact-text duplicate refill, negation and phrase preservation. Independent exhaustive oracles check all substrings of a Unicode sample and eligible complete paragraphs. Four small Tree Borrows Miri cases complete on the final implementation. A separate large interpreter attempt was stopped after 13:45, is retained as incomplete and is excluded from passing counts; its native regression passes. Only extra floating-point rounding injection is disabled, not interpreter memory or alias checks. Native PDF/ONNX and asynchronous MCP are outside these four Miri cases.

The unchanged PDF backend passes 14 regressions and its 381-source-file provenance check. An external Rust 1.89 consumer exercises the new public API, rejected unknown sources, exact quotations, strict null handling and document projection. Package extraction, registry consumer, exact commit/checksum and GitHub CI are release gates. These finite checks do not promise a bug-free system.

## Fresh comparison

The [reproduction guide and executable harness](validation-0.4.5/README.md) describe the unchanged private 72-document / 1,910-chunk corpus, the three arms, input hashes, prompt, model, budgets and order balancing. The retained original 0.4.4 comparison remains available separately. Its ID-only answer prompt differs from this experiment's quote-output prompt, so old LLM timings are not directly comparable.

This release comparison uses 210 authored questions, including 162 exposed development cases and 48 fresh holdout cases. A is published 0.4.4 BM25 with prefix excerpts; C is 0.4.5 BM25 with explicit paragraph preference and exact-excerpt deduplication; S is SQLite FTS5 plus BGE-M3 exact cosine with equal-weight reciprocal-rank fusion. All receive the same project scope, six hits, 100 candidates, 2,000 bytes per excerpt and 12,000 total source bytes. SQLite uses identical retained document embeddings and freshly measured query embeddings. A and C need no embeddings. This compares complete delivery configurations: relevant-window selection, paragraph preference and deduplication change together in C, so it does not isolate a causal effect for any one option.

All 210 default old/new retrieval JSON objects are identical. The new behavior is opt-in. Reference-quote presence on the fresh positive set is **28/36 for A, 26/36 for C and 30/36 for S**. On exposed positives it is **109/138, 103/138 and 110/138** respectively. Compact delivery therefore has not demonstrated better reference coverage. These authored reference labels may miss alternative valid passages.

### Answer results on the 48 fresh questions

There are 36 positive reference questions and 12 intentionally unsupported questions. Automatic fact-pattern checks are descriptive proxies, not semantic correctness judgments. Requiring an authored reference ID can reject valid alternative support. The same literal quotation checker is applied to all three arms.

| Holdout observation | A: old BM25 | C: optional selection | S: SQLite hybrid |
| --- | ---: | ---: | ---: |
| Reference quotation present in delivered context, /36 | 28 | 26 | 30 |
| Answer matches authored fact patterns, /36 | 29 | 27 | 28 |
| Fact patterns + verified quotations + consistent inline IDs, /36 | 26 | 25 | 22 |
| Same proxy plus an authored reference ID, /36 | 25 | 24 | 21 |
| Proper abstention on unsupported questions, /12 | 12 | 12 | 12 |
| Abstention on a corpus-answerable question, /36 | 7 | 9 | 6 |

The paired C−A difference in the combined automatic fact/quote-or-abstention proxy over all 48 cases is -2.08 percentage points; the 10,000-resample percentile interval is [-10.42, +4.17]. This interval describes the authored set, not population uncertainty. No independent user-accuracy advantage is established.

### Cost of the complete delivery configurations

The table uses the 48 holdout questions, including unanswered cases. Lower cost is not a quality-equivalent saving: the delivered evidence and answer outcomes differ. Shared prompt caching is enabled; real returned cache counters are retained. The six execution orders are balanced within each partition. Model generation is capped at 512 output tokens and 16,384 context tokens, with temperature zero and fixed seed. All 630 replies, including malformed output, are retained without quality-based retries.

| Holdout cost | A: old BM25 | C: optional selection | S: SQLite hybrid |
| --- | ---: | ---: | ---: |
| Mean prompt tokens | 3249.2 | 2683.4 | 3295.0 |
| Mean generated tokens | 64.7 | 61.3 | 70.5 |
| Mean cached prompt tokens | 832.7 | 663.4 | 140.1 |
| Median end-to-end seconds | 1.668 | 1.333 | 2.004 |
| p95 end-to-end seconds | 2.636 | 2.558 | 2.931 |
| Median delivered UTF-8 bytes | 10419 | 9219 | 10364 |

C uses 17.41% fewer mean prompt tokens on these holdout cases. This is measured with changed evidence, not proof of equal-answer-quality token savings.

End-to-end includes query embedding for S, retrieval, context construction, local generation and final citation validation. The temporary validation adapter checks passages against canonical scoped entries before checking quotations. Its admission/check subphase and complete IPC wall time are retained separately. Retrieval-only timing contains source work, excerpt construction, serialization/IPC and common context construction; worker subphase timings differ in scope and are not interchangeable algorithm benchmarks.

### All replies, failures and resource limits

| Observation over 210 replies per arm | A | C | S |
| --- | ---: | ---: | ---: |
| Strict JSON shape | 210 | 210 | 210 |
| Bounded answer parser admission | 210 | 210 | 210 |
| Nonempty exact quotation sets verified | 116 | 116 | 116 |
| Generation length stops | 0 | 0 | 0 |
| Quotes exceeding the prompt request of 180 characters | 2 | 3 | 2 |

Parser admission is not citation approval, and a proper abstention has no nonempty quote-proof claim. Citation statuses, parser failures, tokens, timings and every query-level outcome remain in the [sanitized records](measurements/0.4.5/README.md).

Across all arms there are 421 non-abstaining replies. Complete quotation sets pass for 348; **73 replies have at least one failed quotation/reference check** (A: 24/140, C: 20/136, S: 29/145). Individual failures comprise 93 missing exact quotations, two ambiguous locations and one unknown source ID; multiple failures may belong to one reply. This demonstrates the technical checker's practical use without claiming that all passing answers are correct. The benchmark retains the unfiltered model output; an application chooses whether to reject, abstain or request review after a failed check. All 630 outputs have the requested JSON shape and pass bounded parser admission; there are no parser failures or output-length stops in this run.

| All-query warm cost | A | C | S |
| --- | ---: | ---: | ---: |
| Context retrieval/construction median ms | 1.015 | 1.807 | 2.348 |
| Complete quote-validation adapter/IPC median ms | 0.178 | 0.175 | 0.183 |
| Parser + registry check subphase median ms | 0.0112 | 0.0111 | 0.0115 |

C's excerpt analysis adds local retrieval cost. The warm table uses prepared query vectors for S and excludes embedding inference; the end-to-end table includes fresh S query inference. These boundaries, shared caches and changed answer outcomes prevent interpreting the end-to-end difference as a BM25 acceleration or an equal-quality cost guarantee.

C skips 21 byte-identical excerpts across 210 queries and changes the delivered passage set for 151 queries. The comparison retains 3,150 warm timed retrieval samples (five per question/arm). Whole-worker RSS samples are A: 68.5 MiB, C: 69.0 MiB, S: 118.9 MiB. The isolated Ollama process family peaks at 30.09 GiB sampled RSS. This is neither isolated index heap nor a complete accounting of Apple GPU/unified memory; loaded model residency is retained separately, and the two figures must not be added as independent allocations. No index-RAM improvement is claimed.

The local model is Qwen3.6 35B Q4_K_M, with BGE-M3 F16 for S, on an AC-powered Apple M5 Max with 64 GiB. Ollama is 0.40.16, Rust is 1.98.0 for measured workers, Python 3.14.7, NumPy 2.5.5 and SQLite 3.53.4. Small document/label samples, one machine, shared cache, warm models and one generation per question/arm limit generalization. Full private source data and replies are retained; sanitized counters alone cannot independently reconstruct this private corpus.


## Anonymized assistant review

A preselected stratified subset contains 24 questions, including 18 answerable and six unsupported questions, with three replies each. Half the questions are development and half holdout. Variant labels were masked during review; all 72 judgments were saved and hashed before opening their mapping. The reviewer is the same assistant that authored the questions, not an independent human.

| Semantic judgment | A: old BM25 | C: optional selection | S: SQLite hybrid |
| --- | ---: | ---: | ---: |
| Correct and supported by delivered cited evidence | 13 | 13 | 16 |
| Correct abstention on unsupported question | 6 | 6 | 6 |
| Correct conclusion, weak cited support | 1 | 1 | 0 |
| Abstention on a corpus-answerable question | 3 | 3 | 1 |
| Overclaim beyond the cited source | 1 | 1 | 0 |
| Incomplete answer | 0 | 0 | 1 |

An abstention on an answerable question can be appropriate when retrieval omitted the needed evidence; this table is not a pure model-error count. Semantic support is judged separately from exact quotation syntax: some correct answers quote fabricated ellipses, remove Markdown or change letter case. The checker rejects these exact-quote claims. Conversely, two replies copy real source wording but turn “not required” into “excluded”; literal quote verification does not catch that logical overclaim. Alternative valid citations also explain some reference-ID proxy failures. This small same-author sample supports no general accuracy or superiority claim.



## Limits and useful next work

A quotation can be copied correctly while an answer misrepresents it. Exact checking is a technical acceptance boundary, not entailment verification, source authentication, freshness, permissions or prompt-injection immunity. Source passages remain untrusted data. Applications should preserve immutable response/snapshot provenance, enforce access before retrieval, require citations appropriate to their use case and treat uncertain answers as uncertain.

Keep ordinary BM25 delivery as the safe starting configuration for this measured corpus. Enable compaction only after measuring answer quality and cost on the intended workload. Next useful work is independent human relevance/support labels, paragraph-aware ingestion boundaries and evidence selection judged on complete claims and qualifications. More modules or implicit model calls would not substitute for this evidence.

## Preserved knowledge and artifact policy

The canonical knowledge remains **167,942 entries from 35 PDFs and 49,086 physical pages**. Its SHA-256 remains `13168d87872a137355fd8499c90a6084f52243a2ec7456f29b5fcb2a1c7423c3`. Original PDFs, document snapshots, prior measurements, failed exploratory outputs, frozen questions and full local model answers are retained privately and excluded from the crate and Git. Sanitized counters and reproduction source are public; private project text, quotations, model responses and credentials are not published.

Build checks use explicit temporary Cargo targets outside the project with debug information and incremental compilation disabled. Only these named, regenerable build directories are removed after release verification. Release archives, locks, logs and checksums are saved first. The earlier 118 GB removal was performed by the user; it is not counted as cleanup by this release.
