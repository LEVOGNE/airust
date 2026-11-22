# AIRust 0.4.7: inspectable lexical evidence coverage

The release adds an optional `LexicalCoverageSource`. It balances the prepared lexical score with the share of distinct question terms found in each candidate. Literal BM25 stays the default. Existing source APIs, source text, saved knowledge, citation validation and optional MCP contracts remain. No model, new dependency family or DenkMaps integration is added. [Release priorities](../ROADMAP.md#release-priorities) are stability, performance, reliability and clear API/MCP contracts.

The result is modest: one more reference passage on exposed development questions, unchanged reference coverage on 48 fresh answerable questions, and slightly better mean reference ranks. This does not establish a general semantic-quality improvement. The new option should be selected and calibrated for a measured workload.

## API and boundaries

Enable `retrieval`, construct `LexicalCoverageSource::new(&index)` and use the existing `Retriever`. `LexicalCoverageOptions` configures the query preparation and a finite coverage weight from zero to one. The default is 0.5. One prepared lexical search returns at most the requested candidates; those same candidates are reordered. No additional search, model or larger context budget is hidden in the operation.

The final score is `(1-weight)*(raw_score/maximum_returned_raw_score) + weight*distinct_term_fraction`. A zero maximum gives normalized score zero. Exact ties use entry ID. Terms include prepared identifier parts, not semantic concepts. A candidate outside the initial search results cannot be recovered by rescoring.

`inspect` returns the prepared query and, for each borrowed entry, the original lexical rank/score, normalized score, distinct-term counts, coverage and final score. The ordinary response uses existing `custom` evidence with lexical rank/score retained. Semantic fields stay empty. The final source rank and original lexical rank have different meanings. Query-local normalized scores are not confidence probabilities, and existing BM25 score thresholds must not be reused.

Scope intersection applies before preparation/search. Existing query and excerpt budgets remain, with cooperative execution checks during search/rescoring and after sorting. A blocked allocation, normalization or sort cannot be forcibly interrupted. The source retains no previous-query state. Existing evidence assessment still uses the original caller question; preparation does not bypass that policy. Source positions and citation checks apply to final immutable delivered text. An exact valid quotation does not prove that it answers the question.

See the [complete API, every getter and scoring limitation](LEXICAL-COVERAGE.md), [query preparation](LEXICAL-QUERY.md), [excerpt contract](EXCERPTS.md) and [API cookbook](API-COOKBOOK.md).

## Executed stability and security checks

The final complete local script passes **221 default tests, 859 all-feature checks including 39 documentation examples, 39 isolated feature entries and five Python tests**. Strict Clippy/docs pass. Rust 1.89 checks provider-neutral features; Rust 1.92 checks all features. Public API comparison against tagged 0.4.6 reports **229 passes and 31 tool skips**. A separate Rust 1.89 application exercises the new source/trace plus previous preparation, excerpt, citation and document-coordinate behavior. Existing MCP schema, malformed-input, progress, cancellation and connection tests remain in the complete/minimal matrix. This is not a new independent penetration audit.

Seven new coverage tests with all features exercise the score calculation against an independent search/text-token oracle, BM25/TF-IDF, weight boundaries, stable ties, scope intersections, query budgets, original UTF-8 source spans, no prior-query state and cancellation. The minimal `retrieval` build runs six of those cases. The PDF backend remains 0.3.5 and passes its 14 regressions and 381-source-file provenance check.

Five targeted Tree Borrows Miri cases pass: weight validation, broader-term/stable-tie behavior, scope intersection, query fallback and repeated citation occurrences. Extra floating-point rounding injection alone is disabled; ordinary memory and alias checks remain enabled. These finite interpreter cases exclude native PDF/ONNX and async MCP. They are not proof of complete soundness.

The refreshed dependency audit found **RUSTSEC-2026-0592** in the previous `url` 2.5.8 resolution: nested `blob:` origin calculation can recurse until stack exhaustion. The optional web dependency now requires `url >=2.5.9`; the lock records 2.5.9 and `form_urlencoded` 1.2.3. Two regressions exercise 10,000 nested prefixes on a 2 MiB thread and rejection by existing web-origin/crawler policies. Existing policy already rejects non-HTTP schemes; this is not a claim that an exposed AIRust route was demonstrated exploitable. See the [upstream fix](https://github.com/servo/rust-url/pull/1186) and [patched release](https://github.com/servo/rust-url/releases/tag/v2.5.9).

The final checked dependency graph has zero known vulnerabilities in that database, with existing informational maintenance notices for `chrono`, `paste` and `ttf-parser`. The full matrix was rerun after the dependency patch. Initial audit/test-fixture failures and prepatch logs remain retained; security advisories can change later. This is not a security certification.

## Frozen comparison

The [public consumers, manifests and harness](validation-0.4.7/README.md) use the unchanged private snapshot of 72 Markdown files, 2,755,468 source bytes and 1,910 chunks from three projects. This release does not claim a fresh large-PDF performance result. All 282 earlier questions are now exposed development data. Rank-fusion, reserved-head and coverage/score-blend experiments are retained, including unsuccessful variants. The 0.5 coverage weight was selected on exposed labels, not on the new evaluation results.

Another **60 questions were frozen before inspecting new rankings**: 48 answerable and 12 intentionally unsupported, twenty per project, with sixteen English and forty-four German questions. Positive labels cover ten known files: four AIRust and four Flasm files with four questions each, and two Vecasm files with eight each. These are same-assistant-authored questions about known sources, not independent human judgments, real user logs or unseen projects. Source-file clustering, possible conceptual overlap and multiple development experiments limit generalization.

Three configurations use identical project scope, six final passages, 100 source candidates, 2,000 bytes per passage and 12,000 total source-text bytes:

- **A:** exact registry 0.4.6, literal BM25 and prefix excerpts.
- **P:** exact registry 0.4.6, prepared BM25, leading context and exact distinct excerpts.
- **C:** candidate 0.4.7, prepared coverage scoring, leading context and exact distinct excerpts.

All receive the original case-preserved question. Both old configurations reproduce identical complete JSON against the candidate implementation on all 342 questions: **684 compatibility comparisons**. An independent canonical-token/formula/order oracle checks **31,744 candidate components**. Every delivered passage is checked against scoped canonical text and its original byte positions. Retrieval has **5,130 warm timed samples**, five per question/configuration after an untimed warm-up. Own compiler jobs finish before timing. A first pre-inference pass reached retrieval completion but its RSS diagnostic was sandbox-blocked; it is retained separately, and the complete pass was rerun with the required process diagnostic access. No answer was retried for quality.

### Reference coverage and ranks

The metric asks whether a particular authored reference quotation survives in delivered context, not whether every useful passage or every required fact was found. Alternative valid sources can fail this metric. MRR@6 uses the first delivered reference quotation's reciprocal rank, with missing references scored zero.

| Reference observation | A: literal | P: prepared | C: coverage |
| --- | ---: | ---: | ---: |
| Exposed positive questions | 228 | 228 | 228 |
| Exposed reference quotation present | 182 | 185 | 186 |
| Exposed reference MRR@6 | 0.6653 | 0.6845 | 0.6887 |
| Fresh positive questions | 48 | 48 | 48 |
| Fresh reference quotation present | 36 | 39 | 39 |
| Fresh reference MRR@6 | 0.6240 | 0.6594 | 0.6615 |
| Fresh reference at rank one | 26 | 28 | 28 |

C gains one exposed reference and loses none against P; the new questions have no reference-presence changes between P and C. The aggregate count of 45 on the former 54-question sample is restored, but by finding a different previously missing reference. The two literal-only references lost through preparation remain missed. A restored total does not mean every earlier miss has been repaired. Across all 276 positives, C gains nine and loses two against A; that combined set is mostly exposed data and is not an independent test result.

Every selected reference ID also retains its reference quotation in these runs. Wider excerpt behavior and all text budgets are unchanged from P; the coverage result is not purchased by increasing context limits. C changes passage order/content for 164 of 342 queries, including thirty of sixty new queries.

### Fresh local answers and technical quotation checks

Only the sixty new questions receive fresh model replies: **180 replies**, sixty per configuration. The earlier 846 replies remain historical data and are not counted again. The same installed Qwen3.6 35B Q4_K_M model, prompt/schema, temperature zero, seed 471107, thinking disabled, context 16,384 and output cap 512 are used. Six execution orders occur ten times; each configuration occupies each position twenty times and precedes each other configuration thirty times. Shared prompt caching remains enabled and its actual counters are retained. Malformed replies, bad quotes and abstentions are not silently replaced. This focused release does not rerun the earlier SQLite/embedding comparison.

| Observation | A | P | C |
| --- | ---: | ---: | ---: |
| Positive answer matches authored fact patterns, /48 | 35 | 37 | 38 |
| Fact patterns + verified quotes + consistent inline IDs, /48 | 30 | 33 | 34 |
| Same proxy plus an authored reference ID, /48 | 27 | 31 | 32 |
| Proper abstention on unsupported questions, /12 | 12 | 12 | 12 |
| Abstention on a corpus-answerable question, /48 | 10 | 8 | 7 |
| Strict JSON shape / bounded parser admission, /60 | 60 / 60 | 60 / 60 | 60 / 60 |
| Nonempty quotation sets verified, /60 | 33 | 35 | 37 |
| Invalid quotation sets / non-abstaining replies | 5 / 38 | 5 / 40 | 4 / 41 |

All configurations have zero generation length stops and zero quotes exceeding the prompt request of 180 characters. Parser admission is not citation approval. An unsupported-question abstention has no nonempty quotation-proof claim. The combined fact/quote-or-abstention proxy changes C−P by +1.67 percentage points over sixty questions, with a question-bootstrap 95% percentile interval [0, +5]. Against A it changes by +6.67 points, interval [+1.67, +13.33]. These 10,000-resample intervals describe authored questions, do not correct for source-file clustering or multiple development trials, and do not establish population accuracy. Word-pattern checks are especially limited.

### Masked assistant semantic review

Twelve questions were preselected before ranking: three positive and one unsupported per project. All 36 variant judgments were frozen and hashed before opening the configuration mapping. The reviewer is the same assistant, not an independent human.

| Semantic judgment | A | P | C |
| --- | ---: | ---: | ---: |
| Correct, complete and supported | 5 | 7 | 7 |
| Correct unsupported abstention | 3 | 3 | 3 |
| Abstention on a corpus-answerable question | 3 | 1 | 1 |
| Wrong component and claim | 1 | 1 | 1 |

The review does not distinguish P from C on this small sample. In one case, all configurations confuse a document's content revision with another index's monotone counter and call it a timestamp despite their valid quotation saying otherwise. Both literal quotation checks and weak word-pattern scoring can pass such a wrong answer. Another answer gives the correct fact but deletes Markdown backticks in its quote, so literal validation properly fails. Semantic support, exact quotation validity and authored-reference coverage remain separate observations.

## Measured costs and limits

| Fresh sixty-query cost | A | P | C |
| --- | ---: | ---: | ---: |
| Mean prompt tokens | 3200.2 | 3228.9 | 3241.1 |
| Mean generated tokens | 69.9 | 71.8 | 72.9 |
| Mean cached prompt tokens | 237.8 | 785.0 | 864.0 |
| Median complete pipeline seconds | 1.929 | 1.808 | 1.760 |
| p95 complete pipeline seconds | 2.798 | 2.844 | 2.928 |
| Median source-text bytes | 10224 | 10213 | 10318 |

C has a lower median but a higher p95 complete-pipeline time. Real cache counts, contexts and answer outcomes differ. This is not a BM25 acceleration, a consistent latency improvement or a quality-equivalent token saving. Pipeline time includes retrieval/context, local generation and final quotation checking. The latter is approximately 0.2 ms median including adapter/IPC; its parser/registry subphase is approximately 0.011 ms. Neither is an entailment checker.

Across all 342 queries, median warm retrieval/context time is **0.879 / 1.259 / 1.266 ms** for A/P/C. Corresponding p95 values are **1.355 / 1.727 / 1.736 ms**. These include source work, excerpt construction, serialization/IPC and common context construction, not isolated scoring. C adds a small measured optional cost over P; ordinary core behavior is unchanged. No speedup claim is made.

End-of-run whole-worker RSS samples are **68.61 / 68.66 / 68.80 MiB**. The isolated Ollama process family reaches **30.51 GiB sampled RSS** over 316 samples. These are not isolated index heap, complete GPU/unified-memory accounting or a demonstrated RAM improvement. The machine is an AC-powered Apple M5 Max with 64 GB, Ollama 0.40.16, Rust 1.98.0 and Python 3.14.7. One host, warm processes, shared caching, one reply per question/configuration and a small authored corpus limit all performance/quality conclusions.

The [sanitized measurements](measurements/0.4.7/README.md) retain each query's scalar outcomes, timings, selected score components and failed development configurations. Full private replies and sources remain available locally. Public hashes/counters alone cannot independently reconstruct the private corpus. Independent user questions, broader domains, human judgments and quality-equivalent cost comparisons remain necessary. Literal BM25 stays the starting configuration.

## Knowledge and artifact discipline

The retained canonical knowledge is unchanged: **167,942 entries from 35 PDFs and 49,086 physical pages**, SHA-256 `13168d87872a137355fd8499c90a6084f52243a2ec7456f29b5fcb2a1c7423c3`. Original PDFs, source snapshots, prior releases, failed prototypes, frozen questions and all fresh answers remain private. No original data or application database is migrated or deleted.

Builds use only the named temporary Cargo and Miri directories with incremental/dev/test debug output disabled. Release packages, measured workers, exact locks and logs are archived before removing those regenerable trees after publication verification. Exact-commit CI, extracted-package tests, a fresh registry consumer, checksum/VCS/tag matching and a clean Git state are publication gates. Model evaluation used a private loopback daemon with cloud disabled; only that owned daemon was stopped. The user's original daemon is untouched.
