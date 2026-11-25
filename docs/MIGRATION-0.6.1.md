# Migrate from 0.6.0 to 0.6.1

Select `airust = { version = "=0.6.1", default-features = false }` and enable only the features your application needs. [Release verification](validation-0.6.1/RELEASE.md) records source/tag/registry identities and open limits.

Default features stay empty. Rust 1.89 supports core/provider-neutral/pilot code; supplied native ONNX adapters require Rust 1.92. Public library APIs, source/document/workspace/pilot formats, tokenization/model identities and five MCP tools remain compatible. MCP schema identifiers include the package version, so clients rediscover the 0.6.1-qualified identifiers as for other patch versions.

Existing schema-one profiles continue working. A missing reranker batch field means 16. A missing or null candidate field retains max(20, requested result limit). This preserves legacy custom MCP callers returning more than 20 hits. Library limits and application/transport bounds still apply.

For an explicitly measured workload, optional fields make the execution policy visible:

```json
{
  "directory": "/models/mmarco-minilm-v2",
  "runtime": "/runtime/libonnxruntime.dylib",
  "threads": 2,
  "batch_size": 1,
  "candidates": 20,
  "model_snapshot_directory": "/private/model-snapshots"
}
```

This is the `reranker` object within a `hybrid_rerank` profile, not a complete profile. Threads remain 1..64; batch size is 1..16; an explicit candidate prefix is 1..100. A request for more results than the explicit prefix is a configuration error before model work; the server does not silently enlarge that budget or invoke fallback. To retain automatic legacy behavior, omit `candidates` or use null. To compare batching alone, keep the candidate prefix, assets, text/token recipe and query workload fixed.

`model_snapshot_directory` is an additional explicit opt-in: the existing directory must be application-owned and have room for a 470883696-byte private copy plus filesystem overhead. Missing/null keeps memory loading, with no writable-directory requirement. Normal drop and setup failures clean up the copy; abrupt process termination can leave it behind. The supplied model/runtime/recipe remain unchanged. Offline checks only resolve this path and do not create it or assert disk readiness.

`KnowledgeBase` snapshots now share examples privately and detach on mutation. Values/order/serialization remain compatible; the first write to shared data can cost a full collection copy. `load_bounded(path, max_bytes)` is new and optional. Existing `load()` remains an unbounded trusted-file API. Special/nonregular inputs to the hardened bounded/MCP/native paths are rejected; retained original files and source bindings are unchanged. E5 now rejects model files over 512 MiB, tokenizers over 64 MiB and each config over 1 MiB before native initialization. See [resource contracts](RESOURCE-LIMITS.md).

Large serde JSON data-error messages from restoration/import paths now contain a bounded beginning/ending and a truncation marker instead of retaining the entire invalid string. The cap is 2048 UTF-8 bytes for the underlying JSON diagnostic; an outer error type may add its own short prefix. JSON category, line and column remain available and unchanged. Small data diagnostics and syntax/EOF/I/O errors keep their original error objects. Treat display text as diagnostic prose, not a stable parseable interface. This caps retained diagnostics, while parsing itself can still have a transient allocation.

Older 0.6.0 readers reject the new fields as unknown. They can consume a profile with those optional fields removed, using their original automatic prefix/batch. This is backward compatibility for existing profiles, not forward compatibility for new options in an old reader. Persisted workspaces/cases and original evidence do not need rewriting. Cache identities remain bound to the unchanged source/embedding recipe.

Check a profile before expensive loading:

```bash
cargo build --locked --features pilot --example reference_workspace
"${CARGO_TARGET_DIR:-target}/debug/examples/reference_workspace" profile-check /path/to/profile.json
```

The report contains configuration-only status, false `assets_checked`/`service_contacted`, and validated settings with paths resolved against the config directory. Missing asset files can therefore be configuration-valid; actual query/setup still verifies them and may fail. The JSON report is a bounded diagnostic result, not a new storage format or service-readiness proof. Failed checks produce no success JSON and preserve the profile file.

See [configuration ownership](CONFIGURATION.md), [pilot usage](PILOT.md), [measured scope and regressions](measurements/profile-practical-0.6.0/README.md) and [validation](validation-0.6.1/README.md). Select threads/batches from actual compatible measurements; defaults are not silently retuned for one machine.
