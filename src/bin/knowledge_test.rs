use airust::agent::TrainableAgent;
use airust::dynamic_knowledge::KnowledgeBase;
use airust::tfidf_agent::TfidfAgent;
use std::io::{self, Write};
use std::path::PathBuf;

fn main() {
    println!("=== Dynamic Knowledge Base Test ===");
    println!("1. Create new knowledge base");
    println!("2. Add examples");
    println!("3. Save and test");

    // 1. Create new knowledge base
    let mut kb = KnowledgeBase::new();

    // 2. Add examples
    kb.add_example(
        "What is the name of the project?".to_string(),
        "The project is called airust.".to_string(),
        1.0,
    );
    kb.add_example(
        "Who developed it?".to_string(),
        "It was developed as an open source project.".to_string(),
        1.0,
    );
    kb.add_example(
        "What language is used?".to_string(),
        "The project is written in Rust.".to_string(),
        1.0,
    );

    // 3. Save
    let path = PathBuf::from("knowledge/dynamic_test.json");
    match kb.save(Some(path.clone())) {
        Ok(_) => println!("Knowledge base saved in {:?}", path),
        Err(e) => println!("Error saving: {}", e),
    }

    // 4. Load and test
    match KnowledgeBase::load(path) {
        Ok(loaded_kb) => {
            println!("Knowledge base loaded! Test with questions or type 'exit' to quit.");

            let mut agent = TfidfAgent::new();
            agent.train(loaded_kb.get_examples());

            loop {
                print!("> ");
                io::stdout().flush().unwrap();

                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                let input = input.trim();

                if input.to_lowercase() == "exit" {
                    break;
                }

                let answer = agent.predict(input);
                println!("Answer: {}", answer);
            }
        }
        Err(e) => println!("Error loading: {}", e),
    }
}
