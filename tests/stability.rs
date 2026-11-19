use airust::Engine;
use serde_json::Value;

fn identity() -> Value {
    serde_json::from_str(include_str!("fixtures/compat-0.2.30/identity.json")).unwrap()
}

#[test]
fn records_written_by_0230_keep_original_text_and_search_order() {
    let engine = Engine::from_json(include_str!("fixtures/compat-0.2.30/engine.json")).unwrap();
    let ids: Vec<_> = engine
        .search("Rust ownership", 2)
        .iter()
        .map(|h| h.entry.id.clone())
        .collect();
    assert_eq!(serde_json::to_value(ids).unwrap(), identity()["search_ids"]);
    assert_eq!(
        engine.get("de").unwrap().text,
        "Die Häuser haben Türen. Rust ownership prevents errors."
    );
    let restored = Engine::from_json(&engine.to_json().unwrap()).unwrap();
    assert_eq!(restored.to_json().unwrap(), engine.to_json().unwrap());
}

#[cfg(feature = "learned-ranking")]
#[test]
fn model_written_by_0230_retains_fingerprint_and_scoring() {
    use airust::{learned_ranking::*, reranking::Reranker};
    let json = include_str!("fixtures/compat-0.2.30/model.json");
    let model = LinearModel::from_json(json).unwrap();
    let direct: LinearModel = serde_json::from_str(json).unwrap();
    assert_eq!(
        model.fingerprint().unwrap(),
        identity()["model_fingerprint"]
    );
    assert_eq!(model.fingerprint().unwrap(), direct.fingerprint().unwrap());
    assert_eq!(model.to_json().unwrap(), json);
    let mut features = LexicalFeatures;
    let scores = LinearReranker::new(&model, &mut features)
        .unwrap()
        .score(
            "Rust ownership",
            &["Rust ownership prevents errors", "Unrelated archive"],
        )
        .unwrap();
    assert!(scores[0] > scores[1]);
}

#[cfg(feature = "feedback")]
#[test]
fn feedback_written_by_0230_retains_profile_and_votes() {
    use airust::feedback::*;
    let ledger = FeedbackLedger::from_json(
        include_str!("fixtures/compat-0.2.30/feedback.json"),
        FeedbackLimits::default(),
    )
    .unwrap();
    assert_eq!(
        ledger.profile_fingerprint("compat").unwrap(),
        identity()["feedback_profile"]
    );
    assert_eq!(ledger.len(), 1);
    assert_eq!(ledger.event("vote-1").unwrap().rating(), Rating::Helpful);
}

#[cfg(feature = "disk-index")]
#[test]
fn original_0230_disk_file_preserves_data_and_enforces_analysis_compatibility() {
    use airust::{
        disk_index::*,
        storage::{StoreChange, StoreRevision},
        KnowledgeEntry,
    };
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("legacy.db");
    std::fs::write(&path, include_bytes!("fixtures/compat-0.2.30/legacy.db")).unwrap();
    let expected: StoreRevision =
        serde_json::from_value(identity()["disk_revision"].clone()).unwrap();
    let mut disk = match DiskIndex::open(&path, "compat", DiskOptions::default()) {
        Ok(disk) => disk,
        Err(DiskError::Invalid("namespace/analysis recipe mismatch; rebuild required")) => {
            let fresh = DiskIndex::open(
                folder.path().join("fresh.db"),
                "compat",
                DiskOptions::default(),
            )
            .unwrap();
            let connection = rusqlite::Connection::open(&path).unwrap();
            let json: String = connection
                .query_row("SELECT data FROM meta WHERE id=1", [], |r| r.get(0))
                .unwrap();
            let state: Value = serde_json::from_str(&json).unwrap();
            assert_ne!(fresh.recipe(), state["recipe"].as_str().unwrap());
            assert_eq!(
                std::fs::read(&path).unwrap(),
                include_bytes!("fixtures/compat-0.2.30/legacy.db")
            );
            return;
        }
        Err(error) => panic!("Unexpected legacy database error: {error}"),
    };
    assert_eq!(disk.revision().unwrap(), expected);
    disk.integrity_check(&DiskWork::default()).unwrap();
    assert_eq!(
        disk.search("ownership", &DiskSearchOptions::new(2).unwrap())
            .unwrap()
            .hits()[0]
            .entry
            .id,
        "de"
    );
    let next = disk
        .apply(
            &expected,
            &[StoreChange::Put(KnowledgeEntry::new(
                "new",
                "Updated durable knowledge",
            ))],
            &DiskWork::default(),
        )
        .unwrap();
    drop(disk);
    let mut disk = DiskIndex::open(&path, "compat", DiskOptions::default()).unwrap();
    assert_eq!(disk.revision().unwrap(), next);
    assert_eq!(disk.len().unwrap(), 3);
    disk.integrity_check(&DiskWork::default()).unwrap();
}
