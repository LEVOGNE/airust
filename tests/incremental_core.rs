use airust::{Engine, KnowledgeEntry};
#[test]
fn metadata_shortcuts_preserve_number_representations_and_return_the_actual_previous_value() {
    for (old, new) in [
        ("-0.0", "0.0"),
        ("1", "1.0"),
        ("[0.0,-0.0]", "[-0.0,0.0]"),
        ("{\"a\":0,\"b\":1}", "{\"b\":1,\"a\":0}"),
    ] {
        let old: serde_json::Value = serde_json::from_str(old).unwrap();
        let new: serde_json::Value = serde_json::from_str(new).unwrap();
        let previous = serde_json::to_string(&old).unwrap();
        let expected = serde_json::to_string(&new).unwrap();
        let mut index =
            Engine::from_entries([KnowledgeEntry::new("a", "Rust unchanged").with_metadata(old)])
                .unwrap();
        let stats = index.stats();
        let returned = index
            .upsert(KnowledgeEntry::new("a", "Rust unchanged").with_metadata(new))
            .unwrap()
            .unwrap();
        assert_eq!(
            serde_json::to_string(returned.metadata.as_ref().unwrap()).unwrap(),
            previous
        );
        assert_eq!(
            serde_json::to_string(index.get("a").unwrap().metadata.as_ref().unwrap()).unwrap(),
            expected
        );
        assert_eq!(index.stats(), stats);
    }
}
#[test]
fn normalized_equal_text_updates_still_replace_the_original_bytes() {
    let mut index = Engine::from_entries([KnowledgeEntry::new("a", "ＡＩ")]).unwrap();
    let old = index.clone();
    let previous = index
        .upsert(KnowledgeEntry::new("a", "AI"))
        .unwrap()
        .unwrap();
    assert_eq!(previous.text, "ＡＩ");
    assert_eq!(index.get("a").unwrap().text, "AI");
    assert_eq!(old.get("a").unwrap().text, "ＡＩ");
    assert_eq!(index.search("AI", 1)[0].entry.text, "AI");
}
#[test]
fn invalid_replacements_never_use_the_metadata_fast_path() {
    let mut index = Engine::from_entries([KnowledgeEntry::new("a", "Rust")]).unwrap();
    let before = index.to_json().unwrap();
    assert!(index.upsert(KnowledgeEntry::new("a", " \n ")).is_err());
    assert!(index.upsert(KnowledgeEntry::new("", "Rust")).is_err());
    assert_eq!(index.to_json().unwrap(), before);
}
