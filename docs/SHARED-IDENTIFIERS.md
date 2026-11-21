# Entry identifiers in AIRust 0.3.12

An index stores which entries contain each word. Earlier versions copied the complete entry ID into each word list. Long IDs repeated across many words consumed additional memory.

Version 0.3.12 lets word lists and duplicate groups share immutable identifiers. Public entries and the main entry lookup keep ordinary string IDs. Original Unicode/control bytes, sorting and serialized data remain unchanged. Cache reconstruction uses the same representation. The optional `shared-index` feature additionally shares larger immutable analysis/posting groups between cloned indexes.

## Use the existing API

```toml
[dependencies]
airust = { version = "=0.3.12", default-features = false }
```

```rust
use airust::{Engine, KnowledgeEntry};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let id = format!("manual/{}", "section/".repeat(32));
    let mut index = Engine::from_entries([
        KnowledgeEntry::new(&id, "Rust memory ownership"),
        KnowledgeEntry::new("another", "Rust memory allocation"),
    ])?;
    let old = index.clone();
    index.upsert(KnowledgeEntry::new(&id, "Updated source text"))?;
    assert_eq!(old.get(&id).unwrap().text, "Rust memory ownership");
    assert_eq!(index.get(&id).unwrap().text, "Updated source text");
    let restored = Engine::from_json(&index.to_json()?)?;
    assert_eq!(restored.get(&id).unwrap().id, id);
    Ok(())
}
```

There is no new public method, field, flag, mandatory dependency or feature. Existing application-owned IDs remain unchanged; they are not normalized or replaced with generated numbers. Loading an existing JSON export or accepted derived cache builds the current internal representation. No data migration or canonical rewrite is needed.

Build or load one index, then reuse it for requests. Report index construction separately from query latency. `shared-index` remains useful when retaining multiple complete views; both configurations preserve mutation isolation. Deleting/clearing one index does not invalidate retained clones.

## Validation and limits

Regressions cover distinct Unicode forms, control bytes, long common prefixes, lookup/duplicates, JSON reconstruction, retained clones, replacements, removal and clear. Frozen previous/current binaries compare complete results and existing cache files. Real-corpus and synthetic timing/RSS remain separate. Reference counts and tree/map work still have costs; this change does not establish a hard memory bound or universal speedup.

The original knowledge, all seven existing canonical exports, original PDFs, references, raw results and candidates remain retained locally. Native PDF/worker/persistence sources and backend 0.3.4 remain unchanged. Their previous independent proof is inherited only where exact data/code identity supports it; fresh native imports are reported separately.

See [executed results](CHANGES-0.3.12.md), [reproduction](validation-0.3.12/README.md) and [0.3.x gates](STABILITY-0.3.md). No DenkMaps/MCP integration is added.
