use airust::agent::TrainableAgent;
use airust::context_agent::ContextAgent;
use airust::knowledge::TRAINING_DATA;
use std::io::{self, Write};

fn main() {
    println!("=== ContextAgent Test ===");
    println!("Type questions. The agent will try to use the context.");
    println!("Type 'exit' to quit.");

    let mut ai = ContextAgent::new(3); // Store the last 3 interactions
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

        let answer = ai.predict(input);
        println!("Answer: {}", answer);

        // Add the current interaction to the context
        ai.add_to_context(input.to_string(), answer);
    }
}
