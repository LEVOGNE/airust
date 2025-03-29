use airust::agent::TrainableAgent;
use airust::context_agent::ContextAgent;
use airust::knowledge::TRAINING_DATA;
use std::io::{self, Write};

fn main() {
    println!("=== ContextAgent Test ===");
    println!("Tippen Sie Fragen ein. Der Agent wird versuchen, den Kontext zu nutzen.");
    println!("Zum Beenden 'exit' eingeben.");

    let mut ai = ContextAgent::new(3); // Speichere die letzten 3 Interaktionen
    ai.train(&TRAINING_DATA);

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.to_lowercase() == "exit" {
            break;
        }

        let antwort = ai.predict(input);
        println!("Antwort: {}", antwort);

        // Füge die aktuelle Interaktion zum Kontext hinzu
        ai.add_to_context(input.to_string(), antwort);
    }
}
