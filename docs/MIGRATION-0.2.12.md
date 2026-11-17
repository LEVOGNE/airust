# Migrating to AIRust 0.2.12

Existing APIs, empty default features and persisted formats stay compatible. Enable `retrieval` explicitly for the compact source/request/response contract. It introduces no required model, database, web or MCP dependency. Optional source adapters require their corresponding existing features.

Choose LexicalSource, TextSource, SemanticSource or HybridSource, then Retriever::query with explicit RetrievalOptions. Candidate/return limits come from the retrieval request; adapter options retain ranking settings and base scope. Metadata is omitted by default. Responses own selected excerpts and immutable, prevalidated complete JSON, so they may outlive source borrows. They do not replace authoritative records or source revisions.

Budget complete encoded output, not just text. Tail hits may be removed and truncation is explicit; an envelope too large for its budget is an error. Keep model token limits, permissions, freshness and timeouts separate. Original and rerank scores retain different meanings. See [RETRIEVAL](RETRIEVAL.md), [API cookbook](API-COOKBOOK.md) and the documentation-only [proposed application guide](DENKMAPS-UPGRADE.md). No application integration is performed.
