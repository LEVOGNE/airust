use airust::agent::TrainableAgent;
use airust::dynamic_knowledge::KnowledgeBase;
use airust::tfidf_agent::TfidfAgent;
use std::io::{self, Write};
use std::path::PathBuf;

fn main() {
    println!("=== Dynamische Wissensbasis Test ===");
    println!("1. Neue Wissensbasis erstellen");
    println!("2. Beispiele hinzufügen");
    println!("3. Speichern und testen");

    // 1. Neue Wissensbasis erstellen
    let mut kb = KnowledgeBase::new();

    // 2. Beispiele hinzufügen
    kb.add_example(
        "Wie heißt das Projekt?".to_string(),
        "Das Projekt heißt airust.".to_string(),
        1.0,
    );
    kb.add_example(
        "Wer hat das entwickelt?".to_string(),
        "Das wurde als Open-Source-Projekt entwickelt.".to_string(),
        1.0,
    );
    kb.add_example(
        "Welche Sprache wird verwendet?".to_string(),
        "Das Projekt ist in Rust geschrieben.".to_string(),
        1.0,
    );

    // 3. Speichern
    let path = PathBuf::from("knowledge/dynamic_test.json");
    match kb.save(Some(path.clone())) {
        Ok(_) => println!("Wissensbasis gespeichert in {:?}", path),
        Err(e) => println!("Fehler beim Speichern: {}", e),
    }

    // 4. Laden und testen
    match KnowledgeBase::load(path) {
        Ok(loaded_kb) => {
            println!("Wissensbasis geladen! Testen Sie mit Fragen oder 'exit' zum Beenden.");

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

                let antwort = agent.predict(input);
                println!("Antwort: {}", antwort);
            }
        }
        Err(e) => println!("Fehler beim Laden: {}", e),
    }
}
