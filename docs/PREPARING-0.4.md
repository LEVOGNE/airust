# Preparing a focused AIRust 0.4 release

The 0.3.x work has produced measured changes: cheaper token-overlap matching, shared entry identifiers and explicit rejection of the covered incomplete PDF imports. Version 0.3.14 additionally validates root/subtree page counts before extraction. The stability reports retain successful cases, failed experiments, source hashes and limits. Version numbers do not certify every possible input. Continue 0.3.x patches when another reproducible defect appears.

## Reuse the current contracts

| Existing capability | Use in the next stage | Remaining gap to test |
| --- | --- | --- |
| `KnowledgeEntry` | Keep application-owned IDs, text and optional metadata. | A filename alone cannot identify two different source documents. |
| `SourceReference` / provenance `TextSpan` | Keep explicit source revisions and validated UTF-8 ranges. | Revisions are caller-provided opaque strings; the reference does not verify current source bytes. |
| Retrieval `ExcerptSpan` | Slice the exact knowledge entry text using byte offsets. | Entry offsets do not automatically locate the original PDF, page or cleaned document text. |
| `DatasetManager` | Prepare complete replacements and publish a consistent revision. | The application still supplies coherent source/text/configuration identities. |
| `WorkControl` | Report real phases/counts and cooperative stops. | A blocking native parser call needs the existing worker boundary. |
| Storage and derived caches | Preserve originals and validate derived representations. | A new document contract must not create a competing authoritative database. |

See the existing [provenance](PROVENANCE.md), [retrieval](RETRIEVAL.md), [snapshot](SNAPSHOTS.md), [execution](EXECUTION.md) and [cache](CACHE.md) guides before adding another module.

## First candidate: an optional text snapshot and passage contract

Evaluate one small provider-neutral contract. It should associate an application-owned document ID, identified source revision, processing recipe and exact UTF-8 text snapshot. A passage should refer to that snapshot and a validated exclusive-end byte range. Record physical page numbers separately when they are known. A changed source or cleanup recipe must produce a new explicit revision.

Raw PDF bytes, extracted text, cleaned text and knowledge-entry text are different coordinate spaces. Report exactly which one a position belongs to. Do not derive original PDF byte positions by adding offsets from cleaned text. A trustworthy mapping requires retained transformation evidence; missing evidence should remain explicit.

Compose with `SourceReference`, `ExcerptSpan` and `DatasetManager`. Keep models, OCR, durable storage and transport adapters optional. Preserve old exports and stage a complete bounded candidate before publishing a replacement. Start with a private prototype and reusable acceptance fixtures; no new API is promised by this document.

## Acceptance work before publication

| Case | Required observation |
| --- | --- |
| Two sources with the same basename | Distinct stable document identities; no entry overwrite. |
| Changed bytes under the same path | A stale revision is detected; old knowledge remains available. |
| NFC, ligatures, emoji and combining marks | Every declared range slices the identified text at UTF-8 boundaries. |
| Cleanup changes text length | Coordinate space remains explicit; no invented source mapping. |
| Repeated identical passages | Each occurrence retains its own location in the identified snapshot. |
| Worker failure or budget exhaustion | No partially prepared document becomes current. |
| Damaged PDF declares too few pages | Reuse 0.3.14 page-tree/count validation. Keep content/glyph coverage separate; a successful structural check alone does not prove perfect extraction. |
| Source/text/configuration mismatch | Validation rejects the candidate before publication. |
| Real large-document reuse | Compare build/load/query/update costs and peak RSS with the existing APIs. |

Use the retained 33-PDF corpus and authored edge cases. The historical root-count diagnostics, fixed structurally in 0.3.14, show why a snapshot's byte identity and extraction coverage need separate evidence. A hash can identify supplied text without proving every original page was extracted. Keep real and synthetic costs separate. Repeat complete-result and independent-consumer checks. Review Cargo pre-1.0 compatibility deliberately: moving from 0.3.x to 0.4.x permits breaking changes but does not require them.

The first 0.4 release should solve one demonstrated contract gap and include its API guide, upgrade path and evidence. The document/passage proposal is preparation, not an implemented or published feature. No DenkMaps or MCP integration is part of this work.
