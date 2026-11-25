use airust::{KnowledgeBase, ResponseFormat, TrainingExample};

pub struct KnowledgeCampaign;

impl KnowledgeCampaign {
    pub fn exercise(data: &[u8]) {
        let data = &data[..data.len().min(4096)];
        let selector = data.first().copied().unwrap_or(0);
        let count = usize::from(selector % 8);
        let examples = (0..count)
            .map(|i| TrainingExample {
                input: format!(
                    "{i}: {}",
                    String::from_utf8_lossy(&data[..data.len().min(128)])
                ),
                output: match (usize::from(selector) + i) % 3 {
                    0 => ResponseFormat::Text("Straße 東京".into()),
                    1 => ResponseFormat::Markdown("**réponse**".into()),
                    _ => ResponseFormat::Json(serde_json::json!({"sequence": i})),
                },
                weight: if selector % 11 == 0 { -1.0 } else { i as f32 },
                metadata: Some(serde_json::json!({"scope": i % 3})),
            })
            .collect();
        let mut original = KnowledgeBase::from_examples(examples);
        let snapshot = original.clone();
        let before = serde_json::to_value(snapshot.get_examples()).unwrap();
        let _ = original.remove_example(usize::from(selector % 12));
        original.add_example("new".into(), "answer", 1.0);
        original.merge(&snapshot);
        assert_eq!(
            serde_json::to_value(snapshot.get_examples()).unwrap(),
            before
        );
        assert_eq!(snapshot.get_examples().len(), count);
        let mut second = snapshot.clone();
        second.push_example(TrainingExample {
            input: "another".into(),
            output: "value".into(),
            weight: 0.0,
            metadata: None,
        });
        assert_eq!(snapshot.get_examples().len(), count);
        assert_eq!(second.get_examples().len(), count + 1);

        let directory = tempfile::tempdir().expect("owned knowledge fixture");
        let arbitrary = directory.path().join("arbitrary.json");
        std::fs::write(&arbitrary, data).unwrap();
        let maximum = if selector % 2 == 0 {
            data.len()
        } else {
            data.len().saturating_sub(1)
        };
        let result = KnowledgeBase::load_bounded(arbitrary.clone(), maximum);
        if data.len() > maximum {
            assert!(result.is_err());
        }
        if let Ok(loaded) = result {
            assert!(loaded
                .get_examples()
                .iter()
                .all(|example| example.validate().is_ok()));
        }
        assert_eq!(std::fs::read(&arbitrary).unwrap(), data);

        let encoded = serde_json::to_vec(snapshot.get_examples()).unwrap();
        let path = directory.path().join("known.json");
        std::fs::write(&path, &encoded).unwrap();
        let loaded = KnowledgeBase::load_bounded(path.clone(), encoded.len());
        let valid = snapshot
            .get_examples()
            .iter()
            .all(|example| example.validate().is_ok());
        assert_eq!(loaded.is_ok(), valid);
        if let Ok(loaded) = loaded {
            assert_eq!(serde_json::to_value(loaded.get_examples()).unwrap(), before);
        }
        let destination = directory.path().join("retained.json");
        std::fs::write(&destination, b"retained original").unwrap();
        // Successful atomic publication includes a native directory fsync.
        // Miri still checks invalid-weight rejection before publication.
        #[cfg(not(miri))]
        assert_eq!(snapshot.save(Some(destination.clone())).is_ok(), valid);
        #[cfg(miri)]
        if !valid {
            assert!(snapshot.save(Some(destination.clone())).is_err());
        }
        if !valid {
            assert_eq!(std::fs::read(destination).unwrap(), b"retained original");
        }
        assert_eq!(std::fs::read(path).unwrap(), encoded);
    }
}
