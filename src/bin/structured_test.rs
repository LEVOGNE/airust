use airust::agent::TrainableAgent;
use airust::agent::TrainingExample;
use airust::structured_agent::{ResponseFormat, StructuredAgent};
use std::io::{self, Write};

fn main() {
    println!("=== Strukturierte Antworten Test ===");

    // Manuell strukturierte Trainingsdaten erstellen
    let mut training_data = Vec::new();

    // Text-Antwort
    training_data.push(TrainingExample {
        input: "Was ist airust?".to_string(),
        output: "Eine Rust-Bibliothek für einfache KI-Agenten.".to_string(),
        weight: 1.0,
    });

    // Intern wird der StructuredAgent diese als Text behandeln

    // Weitere Tests könnten hier hinzugefügt werden, um ein erweitertes
    // JSON-Format zu verwenden, das die ResponseFormat-Felder enthält.
    // Dies erfordert Anpassungen in knowledge.rs, um das erweiterte Format zu unterstützen.

    let mut agent = StructuredAgent::new();
    agent.train(&training_data);

    println!("Stellen Sie Fragen oder 'exit' zum Beenden.");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.to_lowercase() == "exit" {
            break;
        }

        // Normale Textantwort
        let antwort = agent.predict(input);
        println!("Text-Antwort: {}", antwort);

        // Strukturierte Antwort (in diesem einfachen Test immer Text)
        let strukturierte_antwort = agent.predict_structured(input);
        match strukturierte_antwort {
            ResponseFormat::Text(text) => println!("Strukturierte Antwort (Text): {}", text),
            ResponseFormat::Markdown(md) => println!("Strukturierte Antwort (Markdown):\n{}", md),
            ResponseFormat::Json(json) => println!("Strukturierte Antwort (JSON):\n{}", json),
        }
    }
}
