# Optional derived-index cache

Enable `cache` for `IndexCache`, `CacheIdentity`, `CacheLimits`, cache metadata and owned load results. It uses the existing SHA-256 dependency and optional Postcard 1.1.3 with minimal standard-library support. Models, graphs, HTTP clients and databases remain optional.

Original application records remain authoritative. The cache stores derived normalized text, token frequencies and lengths, plus optional model-specific vectors. Loading requires the original entries and checks their complete fingerprint, including metadata. Posting and duplicate lookup structures are reconstructed from cached frequencies; normalization, tokenization and model inference are skipped. This is a derived cache, not a second source database.

`IndexCache::new(path, CacheIdentity::new(dataset)?)` selects the file and dataset. Optionally add a committed-source revision to the identity. `save(&Engine)` writes derived lexical state. `load(&mut original_entries)` returns `CacheLoad<Engine>` with `index()`, `metadata()`, `into_index()` and `into_parts()`. The original vector is emptied only after successful validation and ownership transfer. On any returned error, the vector remains available for `Engine::from_entries(original_entries)` without cloning the corpus.

When `embeddings` is enabled separately, `save_semantic` and `load_semantic` also preserve vectors. Loading requires the expected `EmbeddingSpace`; incompatible model recipes or dimensions fail. Vector bytes are retained without renormalizing so valid round trips preserve exact scores. No provider is called by a cache load.

The format has a fixed header, format and text-analysis versions, an exact payload length and a SHA-256 checksum. Analysis metadata includes both standard-library and NFKC Unicode table versions. The writer streams to an exclusive private temporary file, syncs it, atomically renames it and syncs the parent directory on Unix. Cache replacements use mode 0600 on Unix. Existing portable source-JSON writers retain their permission behavior. Symlink/nonregular destinations are rejected.

`CacheLimits::default()` permits at most 512 MiB encoded file bytes, one million entries, 16,384 vector dimensions and ten million term bindings. Builders can lower or explicitly change these limits. Counts and vector lengths are checked before their corresponding loops/allocations. Limits constrain declared encoded data and counts, not a precise process-memory ceiling; configure them for the host and corpus. Load validation checks IDs, counts, source/identity/analysis/model matches, checked length arithmetic, vector integrity, checksums, truncation and trailing data.

Checksums detect accidental damage, not an attacker who can modify and rehash a cache. Use a trusted application-controlled directory, preserve source-record authority, and rebuild from originals on a miss or error. Cache data can contain normalized private text. Multiple processes must coordinate their source revision; atomic replacement provides complete snapshots with last-writer-wins behavior, not a database transaction or locking protocol. Uncommitted temporary files are ignored. Incremental in-memory updates use the existing engine methods; saving creates a complete new derived snapshot. There is no append journal or automatic SQLite watcher.

Source provenance and graph relationships remain separate caller-owned structures. A cache does not update their bindings or permissions. The complete serialized prompt and its token budget remain the application's responsibility.

## Rebuild on a cache miss

```no_run
use airust::{cache::{CacheIdentity, IndexCache}, Engine, KnowledgeEntry};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let cache = IndexCache::new("/application-private/knowledge.cache", CacheIdentity::new("dataset")?);
let mut originals = vec![KnowledgeEntry::new("note-1", "Rust memory safety")];
let index = match cache.load(&mut originals) {
    Ok(loaded) => loaded.into_index(),
    Err(_) => {
        let index = Engine::from_entries(originals)?;
        cache.save(&index)?;
        index
    }
};
assert_eq!(index.search("Rust", 1)[0].entry.id, "note-1");
# Ok(())
# }
```

The example rebuilds on any returned cache error. Applications should log the specific error and decide whether a failed optional cache write is fatal. Supply records from a consistent committed application snapshot; a caller-supplied revision does not substitute for the full source fingerprint. See `examples/cache.rs` and `examples/cache_benchmark.rs` for executable recovery and measurements.
