use airust::{Context, KnowledgeEntry};
use serde_json::json;

#[test]
fn intersecting_filters_never_turns_an_empty_intersection_into_all_ids() {
    let unrestricted = Context::new();
    let a = Context::new().with_entry("a");
    let b = Context::new().with_entry("b");
    assert!(a.intersection(&b).is_none());
    assert_eq!(
        a.intersection(&unrestricted).unwrap().entry_ids,
        a.entry_ids
    );
    assert_eq!(
        unrestricted.intersection(&a).unwrap().entry_ids,
        a.entry_ids
    );
    let both = a.clone().with_entry("b");
    assert_eq!(both.intersection(&b).unwrap().entry_ids, b.entry_ids);
    assert!(unrestricted
        .intersection(&unrestricted)
        .unwrap()
        .entry_ids
        .is_empty());
}

#[test]
fn metadata_intersection_combines_constraints_and_conflicts_fail_closed() {
    let a = Context::new().with_metadata("tenant", "a");
    let b = Context::new().with_metadata("tenant", "b");
    assert!(a.intersection(&b).is_none());
    let private = Context::new().with_metadata("private", true);
    let combined = a.intersection(&private).unwrap();
    assert!(combined.includes(
        &KnowledgeEntry::new("a", "visible").with_metadata(json!({"tenant":"a","private":true}))
    ));
    assert!(!combined.includes(
        &KnowledgeEntry::new("b", "hidden").with_metadata(json!({"tenant":"b","private":true}))
    ));
    assert_eq!(a.intersection(&a).unwrap().metadata.len(), 1);
}
