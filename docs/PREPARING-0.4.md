# AIRust 0.4.0 scope and acceptance decisions

The planned 0.3.x cycle ended at 0.3.15. Version 0.4.0 implements the focused optional document contract proposed during that cycle. It preserves existing core APIs and knowledge while identifying exact text revisions and their positions. Read [the implemented API](DOCUMENTS.md) and [executed evidence](CHANGES-0.4.0.md).

## Reuse instead of duplicate

| Existing capability | How 0.4.0 composes with it |
| --- | --- |
| `KnowledgeEntry` | A checked passage becomes a generic entry with a revision/range descriptor. |
| Provenance `TextSpan` and `SourceReference` | Reuse UTF-8 bounds and attach the document revision, physical page and explicit coordinate-space metadata. |
| Retrieval `ExcerptSpan` | Validate the entry, then project its original-text byte range through `Passage::subspan`. |
| `DatasetManager` | Stage entries and source references using existing prepare/publish checks. Applications retain the corresponding document snapshots. |
| Existing storage | Persist complete checked JSON in the application's current transaction. No competing authoritative store is introduced. |
| PDF loader and workers | Keep 0.3.15 extraction/validation behavior and existing native-process boundaries; documents do not enable PDF automatically. |

## Completed acceptance work

Twenty-six document tests include same-basename separation, changed-source and same-length changed-text rejection, recipe/text-space/page-map identity, Unicode boundaries, repeated occurrences, bounded JSON and source readers, independent version-one hash fixtures, retained clones/readers and staged dataset replacement. The 4,096-edit manifest property test retains the original state and validates every successful restoration.

Fresh real-document checks use all 35 retained PDFs / 49,086 physical pages. The 48,983 provided native text pages yield complete page passages plus 146,949 selected relative ranges. Snapshot/passages restore, provenance and entry descriptors remain checked. Current/previous-core results agree on 632 literal probes over identical new page entries. Quiet restore/index/position costs remain separate from native validation timings.

The old 167,942-entry knowledge export and all eight checkpoints remain byte-identical. New snapshots and derived page entries are additional retained artifacts, not an automatic replacement of the old chunked corpus. The private seven-test design prototype remains historical evidence; the public 0.4.0 implementation has its own tests, durable format and measured scope.

## Boundaries that remain deliberate

Raw source bytes, extracted text, cleaned text and entry text are different coordinate spaces. A passage identifies exact snapshot UTF-8 bytes. It does not invent original PDF bytes, drawing positions, printed page labels, OCR or extraction completeness. Hashes identify supplied data rather than authenticate it. Applications coordinate stable source input, access rules, persistence and snapshot retention across requests.

Cargo requires an explicit move from the 0.3 range to 0.4. Existing core signatures are checked at patch compatibility despite that new range. Future changes should address concrete measured gaps and retain failed examples. Model quality, OCR, database adapters and transports are separate work. No DenkMaps or MCP integration is included.
