use airust::{
    Agent, ConfidenceAgent, ContextAgent, ContextualAgent, KnowledgeBase, MatchAgent,
    ResponseFormat, TfidfAgent, TrainableAgent, TrainingExample,
};
use serde_json::json;

fn example(input: &str) -> TrainingExample {
    TrainingExample {
        input: input.to_owned(),
        output: ResponseFormat::Text(
            "No matching answer found is a valid document title".to_owned(),
        ),
        weight: 1.0,
        metadata: Some(json!({"source": "manual"})),
    }
}

#[test]
fn trained_answer_text_does_not_determine_match_confidence() {
    let mut agent = MatchAgent::new_exact();
    agent.train(&[example("rust")]);
    assert_eq!(agent.confidence("rust"), 1.0);
    assert_eq!(
        agent.predict_with_metadata("rust").metadata.unwrap()["source"],
        "manual"
    );
    assert_eq!(agent.confidence("absent"), 0.0);
}

#[test]
fn fuzzy_threshold_counts_unicode_characters_instead_of_bytes() {
    let mut agent = MatchAgent::new_fuzzy();
    agent.train(&[example("äö")]);
    assert_eq!(agent.calculate_confidence("ää"), 0.0);
    assert_eq!(agent.predict_with_metadata("äö").confidence, 1.0);
}

#[test]
fn confidence_and_provenance_are_consistent_through_adapters() {
    let mut agent = TfidfAgent::new();
    agent.train(&[example("rust")]);
    let expected = agent.calculate_confidence("rust");
    assert_eq!(agent.confidence("rust"), expected);
    assert_eq!(agent.predict_with_metadata("rust").confidence, expected);
    let mut context = ContextAgent::new(agent, 2);
    context.add_text_context("rust".to_owned(), "rust".to_owned());
    let prediction = context.predict_with_metadata("rust");
    assert_eq!(prediction.confidence, context.confidence("rust"));
    assert_eq!(prediction.metadata.unwrap()["source"], "manual");
}

#[test]
fn mixed_old_and_new_training_json_keeps_metadata() {
    let file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(
        file.path(),
        r#"[
        {"input":"old","output":"answer","metadata":{"source":"old"}},
        {"input":"new","output":{"Markdown":"**answer**"},"weight":2.0}
    ]"#,
    )
    .unwrap();
    let kb = KnowledgeBase::load(file.path().to_owned()).unwrap();
    assert_eq!(kb.get_examples().len(), 2);
    assert_eq!(
        kb.get_examples()[0].metadata.as_ref().unwrap()["source"],
        "old"
    );
    assert_eq!(kb.get_examples()[1].weight, 2.0);
}

#[test]
fn invalid_bm25_parameters_and_weights_do_not_produce_nan_results() {
    assert!(TfidfAgent::new().try_with_bm25_params(0.0, 0.75).is_err());
    let mut agent = TfidfAgent::new().with_bm25_params(f32::NAN, f32::INFINITY);
    let mut examples = vec![example("rust")];
    for weight in [f32::NAN, f32::INFINITY, -1.0, 0.0] {
        let mut item = example("rust");
        item.weight = weight;
        examples.push(item);
    }
    agent.train(&examples);
    let hits = agent.predict_top_n("rust", 10);
    assert_eq!(hits.len(), 1);
    assert!(hits[0].confidence.is_finite());
}

#[cfg(not(feature = "embedded-knowledge"))]
#[test]
fn default_library_embeds_no_training_data() {
    assert!(KnowledgeBase::from_embedded().get_examples().is_empty());
}
