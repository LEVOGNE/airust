# Migrating to AIRust 0.2.22

Enable adaptive explicitly for the new coordinator; defaults and existing response JSON remain unchanged. RetrievalSource gains a default metadata-only prepare_query hook. Existing implementations need no change; custom wrappers should forward it, as the built-in Box and &mut wrappers do. Source preparation must not call an inference provider or mutate an index. Basic limits precede it; analyzer-dependent checks follow selected metadata.

Retriever adds source/source_mut/into_source accessors. RouteTrace is optional separate diagnostics; it is not query success or a calibrated relevance measure. Actual backend recipes stay unchanged. The default policy rejects quoted queries without a Text-role backend, and never retries another backend after technical failures. Evaluate other policies on your corpus. See [contract](ADAPTIVE.md). No DenkMaps/MCP integration is performed.
