use airust::{
    cache::{CacheIdentity, IndexCache},
    Engine, KnowledgeEntry,
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let root = std::env::temp_dir().join(format!("airust-cache-example-{}", std::process::id()));
    std::fs::create_dir_all(&root)?;
    let cache = IndexCache::new(
        root.join("knowledge.cache"),
        CacheIdentity::new("example")?.with_revision("committed-1")?,
    );
    let mut originals = vec![KnowledgeEntry::new("a", "Rust memory safety")];
    let engine = match cache.load(&mut originals) {
        Ok(loaded) => loaded.into_index(),
        Err(_) => {
            let rebuilt = Engine::from_entries(originals)?;
            cache.save(&rebuilt)?;
            rebuilt
        }
    };
    assert_eq!(engine.search("memory", 1)[0].entry.id, "a");
    let mut originals = vec![KnowledgeEntry::new("a", "Rust memory safety")];
    let loaded = cache.load(&mut originals)?;
    assert!(originals.is_empty());
    println!(
        "Loaded {} entries from a validated derived cache",
        loaded.index().len()
    );
    std::fs::remove_dir_all(root)?;
    Ok(())
}
