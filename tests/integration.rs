use airust::agent::{Agent, ConfidenceAgent, ContextualAgent, TrainableAgent};
use airust::{
    ContextAgent, KnowledgeBase, MatchAgent, ResponseFormat, TfidfAgent, TrainingExample,
};

fn make_example(input: &str, output: &str) -> TrainingExample {
    TrainingExample {
        input: input.to_string(),
        output: ResponseFormat::Text(output.to_string()),
        weight: 1.0,
        metadata: None,
    }
}

#[test]
fn test_kb_to_tfidf_pipeline() {
    let mut kb = KnowledgeBase::new();
    kb.add_example("rust programming".to_string(), "Rust is fast", 1.0);
    kb.add_example("python scripting".to_string(), "Python is easy", 1.0);
    kb.add_example("javascript web".to_string(), "JS runs in browsers", 1.0);

    let mut agent = TfidfAgent::new();
    agent.train(kb.get_examples());

    assert!(String::from(agent.predict("rust")).contains("Rust"));
    assert!(String::from(agent.predict("python")).contains("Python"));
    assert!(String::from(agent.predict("javascript")).contains("JS"));
}

#[test]
fn test_kb_to_exact_match_pipeline() {
    let mut kb = KnowledgeBase::new();
    kb.add_example("hello".to_string(), "Hi there!", 1.0);
    kb.add_example("goodbye".to_string(), "See you!", 1.0);

    let mut agent = MatchAgent::new_exact();
    agent.train(kb.get_examples());

    assert_eq!(String::from(agent.predict("hello")), "Hi there!");
    assert_eq!(String::from(agent.predict("goodbye")), "See you!");
}

#[test]
fn test_kb_to_fuzzy_match_pipeline() {
    let mut kb = KnowledgeBase::new();
    kb.add_example("hello".to_string(), "Hi there!", 1.0);

    let mut agent = MatchAgent::new_fuzzy();
    agent.train(kb.get_examples());

    assert_eq!(String::from(agent.predict("helo")), "Hi there!");
}

#[test]
fn test_kb_tfidf_context_pipeline() {
    let mut kb = KnowledgeBase::new();
    kb.add_example(
        "rust programming".to_string(),
        "Rust is a systems language",
        1.0,
    );
    kb.add_example(
        "python scripting".to_string(),
        "Python is great for AI",
        1.0,
    );

    let mut base = TfidfAgent::new();
    base.train(kb.get_examples());

    let mut ctx = ContextAgent::new(base, 5);

    let result1 = String::from(ctx.predict("rust"));
    assert!(result1.contains("Rust"));

    ctx.add_text_context("rust".to_string(), result1);

    let result2 = String::from(ctx.predict("python"));
    assert!(
        result2.contains("Python") || result2.contains("Rust") || result2.contains("No matching")
    );
}

#[test]
fn test_save_load_train_predict_roundtrip() {
    let mut kb = KnowledgeBase::new();
    kb.add_example(
        "what is rust".to_string(),
        "A systems language by Mozilla",
        1.0,
    );
    kb.add_example("what is python".to_string(), "A scripting language", 2.0);

    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path().to_path_buf();
    kb.save(Some(path.clone())).unwrap();

    let loaded_kb = KnowledgeBase::load(path).unwrap();
    assert_eq!(loaded_kb.get_examples().len(), 2);

    let mut agent = TfidfAgent::new();
    agent.train(loaded_kb.get_examples());

    let result = String::from(agent.predict("what is rust"));
    assert!(result.contains("systems language"));

    assert_eq!(loaded_kb.get_examples()[1].weight, 2.0);
}

#[test]
fn test_kb_merge_train_multiple_agents() {
    let mut kb1 = KnowledgeBase::new();
    kb1.add_example("hello".to_string(), "Hi!", 1.0);

    let mut kb2 = KnowledgeBase::new();
    kb2.add_example("goodbye".to_string(), "Bye!", 1.0);

    kb1.merge(&kb2);
    assert_eq!(kb1.get_examples().len(), 2);

    let mut exact = MatchAgent::new_exact();
    exact.train(kb1.get_examples());

    let mut tfidf = TfidfAgent::new();
    tfidf.train(kb1.get_examples());

    assert_eq!(String::from(exact.predict("hello")), "Hi!");
    assert_eq!(String::from(exact.predict("goodbye")), "Bye!");
    assert_eq!(String::from(tfidf.predict("hello")), "Hi!");
}

#[test]
fn test_incremental_training_pipeline() {
    let mut kb = KnowledgeBase::new();
    kb.add_example("hello".to_string(), "Hi!", 1.0);

    let mut agent = TfidfAgent::new();
    agent.train(kb.get_examples());
    assert_eq!(String::from(agent.predict("hello")), "Hi!");

    kb.add_example("goodbye".to_string(), "Bye!", 1.0);
    agent.append(&kb.get_examples()[1..]);

    assert_eq!(String::from(agent.predict("hello")), "Hi!");
    assert_eq!(String::from(agent.predict("goodbye")), "Bye!");
}

#[test]
fn test_confidence_agent_tfidf_pipeline() {
    let mut kb = KnowledgeBase::new();
    kb.add_example("rust programming".to_string(), "Rust answer", 1.0);
    kb.add_example("python scripting".to_string(), "Python answer", 1.0);

    let mut agent = TfidfAgent::new();
    agent.train(kb.get_examples());

    let conf = agent.calculate_confidence("rust programming");
    assert!(conf > 0.0);

    let conf = agent.calculate_confidence("quantum physics");
    assert_eq!(conf, 0.0);

    let results = agent.predict_top_n("programming", 2);
    assert!(!results.is_empty());
}

#[test]
fn test_confidence_agent_match_pipeline() {
    let mut kb = KnowledgeBase::new();
    kb.add_example("hello".to_string(), "Hi!", 1.0);
    kb.add_example("hey".to_string(), "Hey there!", 1.0);

    let mut agent = MatchAgent::new_fuzzy();
    agent.train(kb.get_examples());

    let conf = agent.calculate_confidence("hello");
    assert_eq!(conf, 1.0);

    let conf = agent.calculate_confidence("helo");
    assert!(conf > 0.0 && conf < 1.0);

    let results = agent.predict_top_n("hello", 2);
    assert!(!results.is_empty());
    assert_eq!(results[0].confidence, 1.0);
}

#[test]
fn test_context_fifo_live() {
    let mut base = MatchAgent::new_exact();
    base.train(&[
        make_example("hello", "Hi!"),
        make_example("bye", "Goodbye!"),
    ]);

    let mut ctx = ContextAgent::new(base, 2);

    assert_eq!(String::from(ctx.predict("hello")), "Hi!");

    ctx.add_text_context("prev_q".to_string(), "prev_a".to_string());
    let result = String::from(ctx.predict("hello"));
    assert!(result.contains("No matching answer found"));

    ctx.clear_context();
    assert_eq!(String::from(ctx.predict("hello")), "Hi!");
}
