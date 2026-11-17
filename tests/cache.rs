#![cfg(feature = "cache")]

use airust::{
    cache::{CacheError, CacheIdentity, CacheKind, CacheLimits, IndexCache},
    Context, Engine, KnowledgeEntry, MatchOptions, SearchAlgorithm, SearchOptions,
};
use serde_json::json;
use std::fs;

fn entries() -> Vec<KnowledgeEntry> {
    vec![
        KnowledgeEntry::new("a", "Rust memory safety Grüße 世界")
            .with_metadata(json!({"tenant":"a"})),
        KnowledgeEntry::new("b", "RUST memory safety Grüße 世界"),
        KnowledgeEntry::new("c", "Python automation and programming"),
        KnowledgeEntry::new("d", "!!!"),
    ]
}
fn identity() -> CacheIdentity {
    CacheIdentity::new("test-dataset")
        .unwrap()
        .with_revision("committed-1")
        .unwrap()
}
fn cache(root: &tempfile::TempDir) -> IndexCache {
    IndexCache::new(root.path().join("derived.cache"), identity())
}

fn compare(a: &Engine, b: &Engine) {
    assert_eq!(a.stats(), b.stats());
    assert_eq!(a.to_json().unwrap(), b.to_json().unwrap());
    for query in ["Rust", "Grüße", "世界", "Python", "missing", "!!!"] {
        for algorithm in [SearchAlgorithm::Bm25, SearchAlgorithm::Tfidf] {
            let options = SearchOptions {
                limit: 10,
                algorithm,
                ..SearchOptions::default()
            };
            let result = |engine: &Engine| {
                engine
                    .search_with_options(query, &options)
                    .into_iter()
                    .map(|r| (r.entry.id.clone(), r.score.to_bits(), r.matched_terms))
                    .collect::<Vec<_>>()
            };
            assert_eq!(result(a), result(b));
        }
        assert_eq!(
            serde_json::to_value(
                a.match_with_options(query, &MatchOptions::default())
                    .unwrap()
            )
            .unwrap(),
            serde_json::to_value(
                b.match_with_options(query, &MatchOptions::default())
                    .unwrap()
            )
            .unwrap()
        );
    }
    assert_eq!(
        serde_json::to_value(a.duplicates()).unwrap(),
        serde_json::to_value(b.duplicates()).unwrap()
    );
    for entry in a.entries() {
        assert_eq!(
            a.relate(&entry.id, 10).unwrap(),
            b.relate(&entry.id, 10).unwrap()
        );
    }
    let scope = Context::new().with_metadata("tenant", "a");
    assert_eq!(
        serde_json::to_value(a.search_in_context("Rust", &scope, 10)).unwrap(),
        serde_json::to_value(b.search_in_context("Rust", &scope, 10)).unwrap()
    );
}

#[test]
fn lexical_restart_preserves_exact_search_matching_duplicates_relations_and_original_metadata() {
    let root = tempfile::tempdir().unwrap();
    let cache = cache(&root);
    let engine = Engine::from_entries(entries()).unwrap();
    let saved = cache.save(&engine).unwrap();
    assert_eq!(saved.kind(), CacheKind::Lexical);
    assert_eq!(saved.entries(), 4);
    assert!(saved.dimensions().is_none());
    let mut originals = entries();
    originals.reverse();
    let loaded = cache.load(&mut originals).unwrap();
    assert!(originals.is_empty());
    assert_eq!(
        loaded.metadata().source_fingerprint(),
        saved.source_fingerprint()
    );
    assert_eq!(saved.source_fingerprint_hex().len(), 64);
    assert!(saved.analysis_signature().contains("std="));
    compare(&engine, loaded.index());
    let mut originals = entries();
    let again = cache.load(&mut originals).unwrap().into_index();
    compare(&engine, &again);
}

#[test]
fn misses_and_identity_text_or_metadata_changes_retain_original_ownership_for_rebuild() {
    let root = tempfile::tempdir().unwrap();
    let cache = cache(&root);
    let mut originals = entries();
    let before = originals.clone();
    assert!(cache.load(&mut originals).is_err());
    assert_eq!(originals, before);
    cache
        .save(&Engine::from_entries(entries()).unwrap())
        .unwrap();
    for field in 0..3 {
        let mut originals = entries();
        match field {
            0 => originals[0].text.push_str(" changed"),
            1 => originals[0].id.push_str("-changed"),
            _ => originals[0].metadata = Some(json!({"tenant":"private"})),
        }
        let before = originals.clone();
        assert!(matches!(
            cache.load(&mut originals),
            Err(CacheError::SourceMismatch)
        ));
        assert_eq!(originals, before);
        assert_eq!(Engine::from_entries(originals).unwrap().len(), 4);
    }
    let wrong = IndexCache::new(cache.path(), CacheIdentity::new("other-dataset").unwrap());
    let mut originals = entries();
    assert!(matches!(
        wrong.load(&mut originals),
        Err(CacheError::IdentityMismatch)
    ));
    assert_eq!(originals, entries());
    let wrong = IndexCache::new(
        cache.path(),
        CacheIdentity::new("test-dataset")
            .unwrap()
            .with_revision("committed-2")
            .unwrap(),
    );
    assert!(matches!(
        wrong.load(&mut originals),
        Err(CacheError::IdentityMismatch)
    ));
}

#[test]
fn updated_and_deleted_entries_can_be_resaved_as_a_new_derived_snapshot() {
    let root = tempfile::tempdir().unwrap();
    let cache = cache(&root);
    let mut engine = Engine::from_entries(entries()).unwrap();
    cache.save(&engine).unwrap();
    engine
        .upsert(KnowledgeEntry::new("c", "Rust updated knowledge"))
        .unwrap();
    engine.remove("a");
    let mut source: Vec<_> = engine.entries().cloned().collect();
    let before = source.clone();
    assert!(cache.load(&mut source).is_err());
    assert_eq!(source, before);
    cache.save(&engine).unwrap();
    let restored = cache.load(&mut source).unwrap().into_index();
    compare(&engine, &restored);
}

#[test]
fn corruption_truncation_trailing_bytes_bad_magic_and_versions_fail_without_source_consumption() {
    let root = tempfile::tempdir().unwrap();
    let cache = cache(&root);
    cache
        .save(&Engine::from_entries(entries()).unwrap())
        .unwrap();
    let original = fs::read(cache.path()).unwrap();
    for variant in 0..7 {
        let mut bytes = original.clone();
        match variant {
            0 => bytes[64] ^= 1,
            1 => bytes.truncate(10),
            2 => {
                bytes.pop();
            }
            3 => bytes.push(0),
            4 => bytes[0] ^= 1,
            5 => bytes[8..12].copy_from_slice(&99u32.to_le_bytes()),
            _ => bytes[12..16].copy_from_slice(&99u32.to_le_bytes()),
        }
        fs::write(cache.path(), bytes).unwrap();
        let mut originals = entries();
        assert!(cache.load(&mut originals).is_err());
        assert_eq!(originals, entries());
    }
}

#[test]
fn explicit_limits_apply_to_save_load_and_declared_header_lengths() {
    assert!(CacheLimits::new(63, 1).is_err());
    assert!(CacheLimits::default().with_max_dimensions(0).is_err());
    assert!(CacheIdentity::new("").is_err());
    assert!(identity().with_revision("\n").is_err());
    let root = tempfile::tempdir().unwrap();
    let cache = cache(&root);
    let engine = Engine::from_entries(entries()).unwrap();
    cache.save(&engine).unwrap();
    let before = fs::read(cache.path()).unwrap();
    let tiny = cache.clone().with_limits(CacheLimits::new(70, 10).unwrap());
    assert!(tiny.save(&engine).is_err());
    assert_eq!(fs::read(cache.path()).unwrap(), before);
    let mut originals = entries();
    assert!(tiny.load(&mut originals).is_err());
    assert_eq!(originals, entries());
    let limited = cache
        .clone()
        .with_limits(CacheLimits::new(1024 * 1024, 1).unwrap());
    assert!(limited.save(&engine).is_err());
    assert!(limited.load(&mut originals).is_err());
    let limited = cache
        .clone()
        .with_limits(CacheLimits::default().with_max_term_bindings(1));
    assert!(limited.save(&engine).is_err());
    assert!(limited.load(&mut originals).is_err());
    let mut forged = before;
    forged[24..32].copy_from_slice(&u64::MAX.to_le_bytes());
    fs::write(cache.path(), forged).unwrap();
    assert!(cache.load(&mut originals).is_err());
    assert_eq!(originals, entries());
}

#[test]
fn empty_indexes_punctuation_only_entries_and_invalid_originals_are_explicit() {
    let root = tempfile::tempdir().unwrap();
    let cache = cache(&root);
    cache.save(&Engine::new()).unwrap();
    let mut empty = Vec::new();
    assert!(cache.load(&mut empty).unwrap().index().is_empty());
    let source = vec![KnowledgeEntry::new("p", "!!! 世界")];
    let engine = Engine::from_entries(source.clone()).unwrap();
    cache.save(&engine).unwrap();
    let mut input = source.clone();
    compare(&engine, cache.load(&mut input).unwrap().index());
    for input in [
        vec![KnowledgeEntry::new("", "text")],
        vec![KnowledgeEntry::new("a", " ")],
        vec![source[0].clone(), source[0].clone()],
    ] {
        let mut originals = input.clone();
        assert!(cache.load(&mut originals).is_err());
        assert_eq!(input, originals);
    }
}

#[test]
fn interrupted_temporary_files_are_ignored_and_nonregular_destinations_are_rejected() {
    let root = tempfile::tempdir().unwrap();
    let cache = cache(&root);
    cache
        .save(&Engine::from_entries(entries()).unwrap())
        .unwrap();
    fs::write(
        root.path().join(".airust-interrupted.tmp"),
        b"partial payload",
    )
    .unwrap();
    let mut originals = entries();
    assert!(cache.load(&mut originals).is_ok());
    let directory = IndexCache::new(root.path(), identity());
    assert!(directory.save(&Engine::new()).is_err());
    let mut originals = entries();
    assert!(directory.load(&mut originals).is_err());
}

#[cfg(unix)]
#[test]
fn cache_files_are_private_and_symlinks_are_rejected_for_reads_and_writes() {
    use std::os::unix::fs::{symlink, PermissionsExt};
    let root = tempfile::tempdir().unwrap();
    let cache = cache(&root);
    fs::write(cache.path(), b"old").unwrap();
    fs::set_permissions(cache.path(), fs::Permissions::from_mode(0o644)).unwrap();
    cache
        .save(&Engine::from_entries(entries()).unwrap())
        .unwrap();
    assert_eq!(
        fs::metadata(cache.path()).unwrap().permissions().mode() & 0o777,
        0o600
    );
    let linked = IndexCache::new(root.path().join("linked.cache"), identity());
    symlink(cache.path(), linked.path()).unwrap();
    let before = fs::read(cache.path()).unwrap();
    assert!(linked.save(&Engine::new()).is_err());
    let mut originals = entries();
    assert!(linked.load(&mut originals).is_err());
    assert_eq!(fs::read(cache.path()).unwrap(), before);
}

#[cfg(feature = "embeddings")]
#[test]
fn semantic_cache_retains_vector_bits_scores_and_model_identity_without_inference() {
    use airust::semantic::{Embedding, EmbeddingSpace, SemanticIndex, SemanticSearchOptions};
    let root = tempfile::tempdir().unwrap();
    let cache = cache(&root);
    let space = EmbeddingSpace::new("cache-test-v1", 3).unwrap();
    let mut index = SemanticIndex::new(space.clone());
    for (n, entry) in entries().into_iter().enumerate() {
        index
            .upsert_embedded(
                entry,
                Embedding::new(space.clone(), vec![1.0, n as f32 + 0.1, 0.7]).unwrap(),
            )
            .unwrap();
    }
    cache.save_semantic(&index).unwrap();
    let mut originals = entries();
    let loaded = cache.load_semantic(&mut originals, &space).unwrap();
    assert!(originals.is_empty());
    assert_eq!(loaded.metadata().model_recipe(), Some("cache-test-v1"));
    compare(index.lexical(), loaded.index().lexical());
    for entry in index.lexical().entries() {
        assert_eq!(
            index
                .embedding(&entry.id)
                .unwrap()
                .iter()
                .map(|n| n.to_bits())
                .collect::<Vec<_>>(),
            loaded
                .index()
                .embedding(&entry.id)
                .unwrap()
                .iter()
                .map(|n| n.to_bits())
                .collect::<Vec<_>>()
        );
    }
    let query = Embedding::new(space.clone(), vec![0.3, 0.7, 0.2]).unwrap();
    let result = |engine: &SemanticIndex| {
        engine
            .search_embedding(&query, &SemanticSearchOptions::new(10))
            .unwrap()
            .iter()
            .map(|r| (r.entry.id.clone(), r.score.to_bits()))
            .collect::<Vec<_>>()
    };
    assert_eq!(result(&index), result(loaded.index()));
    let wrong = EmbeddingSpace::new("cache-test-v2", 3).unwrap();
    let mut originals = entries();
    assert!(matches!(
        cache.load_semantic(&mut originals, &wrong),
        Err(CacheError::ModelMismatch)
    ));
    assert_eq!(originals, entries());
    assert!(matches!(
        cache.load(&mut originals),
        Err(CacheError::KindMismatch)
    ));
}

#[test]
fn repeated_mutations_and_restart_match_a_fresh_authoritative_rebuild() {
    let root = tempfile::tempdir().unwrap();
    let cache = cache(&root);
    let mut originals = std::collections::BTreeMap::new();
    let mut engine = Engine::new();
    for step in 0..100 {
        let id = format!("entry-{}", step * 17 % 23);
        if step % 4 == 0 {
            originals.remove(&id);
            engine.remove(&id);
        } else {
            let entry = KnowledgeEntry::new(&id, format!("Rust Grüße revision {}", step % 11))
                .with_metadata(json!({"revision":step}));
            originals.insert(id, entry.clone());
            engine.upsert(entry).unwrap();
        }
        cache.save(&engine).unwrap();
        let mut records: Vec<_> = originals.values().cloned().collect();
        let restored = cache.load(&mut records).unwrap().into_index();
        let fresh = Engine::from_entries(originals.values().cloned()).unwrap();
        compare(&fresh, &restored);
        engine = restored;
    }
}
