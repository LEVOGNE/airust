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
        eprintln!("Verwendung: cli <agent> <frage>");
        eprintln!("Verfügbare Agenten: simple, fuzzy, context, tfidf, structured");
        return;
    }

    let agent_type = &args[1];
    let frage = &args[2..].join(" ");

    let antwort = match agent_type.as_str() {
        "simple" => {
            let mut ai = SimpleAgent::new();
            ai.train(&TRAINING_DATA);
            ai.predict(frage)
        }
        "fuzzy" => {
            let mut ai = FuzzyAgent::new();
            ai.train(&TRAINING_DATA);
            ai.predict(frage)
        }
        "context" => {
            let mut ai = ContextAgent::new(5); // 5 Kontextelemente speichern
            ai.train(&TRAINING_DATA);
            ai.predict(frage)
        }
        "tfidf" => {
            let mut ai = TfidfAgent::new();
            ai.train(&TRAINING_DATA);
            ai.predict(frage)
        }
        "structured" => {
            let mut ai = StructuredAgent::new();
            ai.train(&TRAINING_DATA);
            ai.predict(frage)
        }
        _ => "Unbekannter Agent. Verfügbar: simple, fuzzy, context, tfidf, structured".to_string(),
    };

    println!("Antwort: {}", antwort);
}
