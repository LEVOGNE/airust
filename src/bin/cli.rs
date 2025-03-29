use airust::agent::TrainableAgent;
use airust::context_agent::ContextAgent;
use airust::fuzzy_agent::FuzzyAgent;
use airust::knowledge::TRAINING_DATA;
use airust::simple_agent::SimpleAgent;
use airust::structured_agent::StructuredAgent;
use airust::tfidf_agent::TfidfAgent;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: cli <agent> <question>");
        eprintln!("Available agents: simple, fuzzy, context, tfidf, structured");
        return;
    }

    let agent_type = &args[1];
    let question = &args[2..].join(" ");

    let answer = match agent_type.as_str() {
        "simple" => {
            let mut ai = SimpleAgent::new();
            ai.train(&TRAINING_DATA);
            ai.predict(question)
        }
        "fuzzy" => {
            let mut ai = FuzzyAgent::new();
            ai.train(&TRAINING_DATA);
            ai.predict(question)
        }
        "context" => {
            let mut ai = ContextAgent::new(5); // Store 5 context elements
            ai.train(&TRAINING_DATA);
            ai.predict(question)
        }
        "tfidf" => {
            let mut ai = TfidfAgent::new();
            ai.train(&TRAINING_DATA);
            ai.predict(question)
        }
        "structured" => {
            let mut ai = StructuredAgent::new();
            ai.train(&TRAINING_DATA);
            ai.predict(question)
        }
        _ => "Unknown agent. Available: simple, fuzzy, context, tfidf, structured".to_string(),
    };

    println!("Answer: {}", answer);
}
