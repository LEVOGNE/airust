use airust::{KnowledgeBase, ResponseFormat, TrainingExample};
use std::fs;

fn sample(input: &str, weight: f32) -> KnowledgeBase {
    KnowledgeBase::from_examples(vec![TrainingExample {
        input: input.to_owned(),
        output: ResponseFormat::Text("answer".to_owned()),
        weight,
        metadata: None,
    }])
}

#[test]
fn invalid_weights_are_rejected_before_replacing_existing_data() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("knowledge.json");
    sample("original", 1.0).save(Some(path.clone())).unwrap();
    let original = fs::read(&path).unwrap();
    for weight in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -1.0] {
        assert!(sample("invalid", weight).save(Some(path.clone())).is_err());
        assert_eq!(fs::read(&path).unwrap(), original);
    }
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
}

#[test]
fn loading_invalid_weights_returns_an_error() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("knowledge.json");
    for weight in ["-1", "1e39", "null"] {
        fs::write(
            &path,
            format!(r#"[{{"input":"test","output":"answer","weight":{weight}}}]"#),
        )
        .unwrap();
        assert!(KnowledgeBase::load(path.clone()).is_err());
    }
}

#[test]
fn concurrent_saves_leave_complete_valid_json_and_no_temporary_files() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("knowledge.json");
    std::thread::scope(|scope| {
        for i in 0..8 {
            let path = path.clone();
            scope.spawn(move || {
                sample(&format!("writer-{i}"), 1.0)
                    .save(Some(path))
                    .unwrap()
            });
        }
    });
    assert!(KnowledgeBase::load(path).unwrap().get_examples()[0]
        .input
        .starts_with("writer-"));
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
}

#[cfg(unix)]
#[test]
fn save_rejects_symlinks_and_creates_private_files() {
    use std::os::unix::fs::{symlink, PermissionsExt};
    let directory = tempfile::tempdir().unwrap();
    let target = directory.path().join("target.json");
    sample("original", 1.0).save(Some(target.clone())).unwrap();
    assert_eq!(
        fs::metadata(&target).unwrap().permissions().mode() & 0o777,
        0o600
    );
    let link = directory.path().join("link.json");
    symlink(&target, &link).unwrap();
    assert!(sample("replacement", 1.0).save(Some(link.clone())).is_err());
    assert!(fs::symlink_metadata(link).unwrap().is_symlink());
    assert_eq!(
        KnowledgeBase::load(target.clone()).unwrap().get_examples()[0].input,
        "original"
    );
    fs::set_permissions(&target, fs::Permissions::from_mode(0o640)).unwrap();
    sample("replacement", 1.0)
        .save(Some(target.clone()))
        .unwrap();
    assert_eq!(
        fs::metadata(&target).unwrap().permissions().mode() & 0o777,
        0o640
    );
}
