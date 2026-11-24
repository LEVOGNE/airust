# Migrate from 0.6.0 to the 0.6.1 candidate

0.6.1 is currently an unpublished source candidate. Registry installation examples continue to select published 0.6.0. Use this checkout for the additional profile controls until a separately verified release exists.

Default features stay empty. Rust 1.89 supports core/provider-neutral/pilot code; supplied native ONNX adapters require Rust 1.92. Public library APIs, source/document/workspace/pilot formats, tokenization/model identities and five MCP tools remain compatible. MCP schema identifiers include the package version, so clients rediscover the 0.6.1-qualified identifiers as for other patch versions.

Existing schema-one profiles continue working. A missing reranker batch field means 16. A missing or null candidate field retains max(20, requested result limit). This preserves legacy custom MCP callers returning more than 20 hits. Library limits and application/transport bounds still apply.

For an explicitly measured workload, optional fields make the execution policy visible:

```json
{
  "directory": "/models/mmarco-minilm-v2",
  "runtime": "/runtime/libonnxruntime.dylib",
  "threads": 2,
  "batch_size": 1,
  "candidates": 20
}
```

This is the `reranker` object within a `hybrid_rerank` profile, not a complete profile. Threads remain 1..64; batch size is 1..16; an explicit candidate prefix is 1..100. A request for more results than the explicit prefix is a configuration error before model work; the server does not silently enlarge that budget or invoke fallback. To retain automatic legacy behavior, omit `candidates` or use null. To compare batching alone, keep the candidate prefix, assets, text/token recipe and query workload fixed.

Older 0.6.0 readers reject the new fields as unknown. They can consume a profile with those optional fields removed, using their original automatic prefix/batch. This is backward compatibility for existing profiles, not forward compatibility for new options in an old reader. Persisted workspaces/cases and original evidence do not need rewriting. Cache identities remain bound to the unchanged source/embedding recipe.

Check a profile before expensive loading:

```bash
cargo build --locked --features pilot --example reference_workspace
"${CARGO_TARGET_DIR:-target}/debug/examples/reference_workspace" profile-check /path/to/profile.json
```

The report contains configuration-only status, false `assets_checked`/`service_contacted`, and validated settings with paths resolved against the config directory. Missing asset files can therefore be configuration-valid; actual query/setup still verifies them and may fail. The JSON report is a bounded diagnostic result, not a new storage format or service-readiness proof. Failed checks produce no success JSON and preserve the profile file.

See [configuration ownership](CONFIGURATION.md), [pilot usage](PILOT.md), [measured scope and regressions](measurements/profile-practical-0.6.0/README.md) and [validation](validation-0.6.1/README.md). Select threads/batches from actual compatible measurements; defaults are not silently retuned for one machine.
