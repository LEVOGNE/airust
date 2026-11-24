# Checking delivered citations

Enable `retrieval`. `retrieval::citations` validates references against one immutable delivered `RetrievalResponse`. It performs no model inference, file reads, network calls or source authentication.

`CitedAnswer::from_json` accepts an object with required `answer` (text or JSON null) and `citations` (an array of objects with `id`, optional `quote` and optional `span`). Unknown fields fail. Text answers need references; abstentions must have none. Blank text and the string `"null"` are rejected; discuss a literal null value in a sentence instead of returning that ambiguous sentinel. `CitationLimits` bounds encoded JSON, combined decoded answer/reference bytes, answer bytes, reference counts, ID bytes and quote bytes. Typed constructors and checks enforce the same payload and per-field limits.

`CitationRegistry::new(&response).check(&answer, &limits)` returns one status per reference. An ID must occur in the delivered hits. A quote must match exact delivered UTF-8 bytes. Without an explicit span it must occur only once, including overlapping occurrences. Explicit spans use exclusive-end original-entry byte positions and must lie completely inside the delivered excerpt. A repeated quote needs a valid explicit span. Duplicate identical references, missing or blank quotes, wrong IDs, malformed ranges and mismatching text remain visible failures.

ID-only citations receive `ReferenceOnly`. `references_valid()` accepts these references, but `quotes_verified()` requires a nonempty set of exact verified quotes. A valid abstention has no quote-proof claim. These statuses do not prove that a quote logically supports the answer, that the document is true, that the underlying record is still current, or that access is still authorized. Inline citation syntax belongs to the application; check any parsed inline IDs against the structured list and delivered IDs. No provider-specific inline format is silently assumed.

The registry borrows the immutable response and checks exactly the text delivered for that request. For revision-bound source coordinates, validate the entry against its `documents::Passage` and project a verified entry-relative span with `Passage::subspan` against the retained `DocumentSnapshot`. No numeric offset is guessed against another revision. The caller retains responsibility for authoritative revisions and permissions.

## Input and limits

```json
{
  "answer": "Rust protects memory.",
  "citations": [{"id": "manual", "quote": "Rust protects memory."}]
}
```

Use `{"answer":null,"citations":[]}` for no answer. Optional `span` has exactly `start` and `end` nonnegative integers in original-entry UTF-8 bytes. Required fields, duplicate fields, unknown fields and types are checked by the parser. A missing quote is permitted only as an unverified ID reference. A span without a quote does not verify anything.

| Default limit | Value | Builder |
| --- | ---: | --- |
| Encoded JSON and combined decoded answer/ID/quote bytes | 128 KiB | `with_input_bytes` |
| Answer UTF-8 bytes | 64 KiB | `with_answer_bytes` |
| References | 128 | `with_citations` |
| One reference ID | 4,096 UTF-8 bytes | `with_reference_bytes(id, quote)` |
| One quote | 8,192 UTF-8 bytes | `with_reference_bytes(id, quote)` |

All builders reject zero. Limits are byte/count budgets, not exact process-memory or hard time bounds. JSON expansion counts toward encoded input; raw source text is not normalized during checking. Typed answers built with larger limits are rechecked against the limits supplied to the registry. Errors do not mutate the response or publish a partly approved answer. Serialize reports within your application's own complete-output budget.

## Result statuses

| Status | Meaning |
| --- | --- |
| `ReferenceOnly` | ID was delivered, without a quote or span claim. |
| `QuoteVerified` | Exact quote has one unambiguous location, or matches the explicit valid span. |
| `UnknownReference` | ID was not in the delivered response. |
| `MissingQuote` | A span was supplied without quoted text. |
| `EmptyQuote` | Quote is empty or only whitespace. |
| `QuoteNotFound` | Exact quote does not occur in that delivered excerpt. |
| `AmbiguousQuote` | Quote occurs more than once, including overlaps; provide its intended span. |
| `InvalidSpan` | Range is reversed, outside the excerpt or not on UTF-8 character boundaries. |
| `QuoteMismatch` | A valid range selects different bytes. |
| `DuplicateReference` | An identical ID/quote/span claim was already checked. |

Malformed answer JSON and admission-budget errors return `CitationError`. Well-shaped references return a bounded report with one observation per input reference; one failure keeps `references_valid()` and `quotes_verified()` false. Distinct quotations from one ID are allowed. Handle valid abstention separately before requiring quotes. The report owns IDs/statuses/spans but is not a durable revision certificate; keep the original response and document snapshot when later verification is needed.

`Citation::new`, `with_quote`, `with_span`, `id`, `quote` and `span` build and inspect references. `CitedAnswer::new` and `from_json` validate shape/budgets; `answer` and `citations` borrow accepted fields. `CitationRegistry::new` borrows a response and `check` validates an answer. `CitationReport` exposes `abstained`, `references_valid`, `quotes_verified` and `checks`; each `CitationCheck` exposes `id`, `status` and its optional verified `span`.

```rust
use airust::{Engine, KnowledgeEntry, retrieval::{
    ExcerptOptions, LexicalSource, RetrievalOptions, Retriever,
    citations::{Citation, CitationLimits, CitationRegistry, CitedAnswer},
}};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let engine = Engine::from_entries([KnowledgeEntry::new("manual", "Intro.\n\nRust protects memory.\n\nAppendix.")])?;
let options = RetrievalOptions::new(3)
    .with_distinct_evidence()
    .with_relevant_excerpts(ExcerptOptions::default().with_paragraph_boundaries());
let response = Retriever::new(LexicalSource::new(&engine)).query("Rust memory", &options)?;
let limits = CitationLimits::default();
let answer = CitedAnswer::new(Some("Rust protects memory.".into()),
    vec![Citation::new("manual").with_quote("Rust protects memory.")], &limits)?;
let checked = CitationRegistry::new(&response).check(&answer, &limits)?;
assert!(checked.quotes_verified());
let span = checked.checks()[0].span().unwrap();
assert_eq!(span.slice(&engine.get("manual").unwrap().text), Some("Rust protects memory."));
# Ok(()) }
```
