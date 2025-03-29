use airust::agent::TrainableAgent;
use airust::fuzzy_agent::FuzzyAgent;
use airust::knowledge::TRAINING_DATA;
use airust::simple_agent::SimpleAgent;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Verwendung: cli <agent> <frage>");
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
        _ => "Unbekannter Agent".to_string(),
    };

    println!("Antwort: {}", antwort);
}
