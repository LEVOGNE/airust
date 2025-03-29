use airust::agent::TrainableAgent;
use airust::agent::TrainingExample;
use airust::structured_agent::{ResponseFormat, StructuredAgent};
use std::io::{self, Write};

fn main() {
    println!("=== Structured Responses Test ===");

    // Manually create structured training data
    let mut training_data = Vec::new();

    // Text response
    training_data.push(TrainingExample {
        input: "What is airust?".to_string(),
        output: "A Rust library for simple AI agents.".to_string(),
        weight: 1.0,
    });

    // Internally, the StructuredAgent will treat these as text

    // More tests could be added here to use an extended
    // JSON format that includes the ResponseFormat fields.
    // This requires adjustments in knowledge.rs to support the extended format.

    let mut agent = StructuredAgent::new();
    agent.train(&training_data);

    println!("Ask questions or type 'exit' to quit.");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.to_lowercase() == "exit" {
            break;
        }

        // Normal text answer
        let answer = agent.predict(input);
        println!("Text Answer: {}", answer);

        // Structured answer (in this simple test always text)
        let structured_answer = agent.predict_structured(input);
        match structured_answer {
            ResponseFormat::Text(text) => println!("Structured Answer (Text): {}", text),
            ResponseFormat::Markdown(md) => println!("Structured Answer (Markdown):\n{}", md),
            ResponseFormat::Json(json) => println!("Structured Answer (JSON):\n{}", json),
        }
    }
}
